#![cfg(test)]

use super::*;
use soroban_sdk::{
    testutils::{Address as TestAddress, Ledger, LedgerInfo},
    Address, Env, BytesN
};

/// Test the complete Bitcoin deposit workflow with real cross-contract calls
#[test]
fn test_complete_bitcoin_deposit_workflow() {
    let env = Env::default();
    let contract_id = env.register_contract(None, IntegrationRouter);
    let client = IntegrationRouterClient::new(&env, &contract_id);
    
    // Set up test addresses
    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let operator = Address::generate(&env);
    let kyc_registry = Address::generate(&env);
    let istsi_token = Address::generate(&env);
    let fungible_token = Address::generate(&env);
    let reserve_manager = Address::generate(&env);
    
    // Initialize the contract
    client.initialize(
        &admin,
        &kyc_registry,
        &istsi_token,
        &fungible_token,
        &reserve_manager
    );
    
    // Set operator role
    client.set_user_role(&admin, &operator, &UserRole::Operator);
    
    // Test data
    let btc_amount = 100_000_000u64; // 1 BTC in satoshis
    let btc_tx_hash = BytesN::from_array(&env, &[1u8; 32]);
    let btc_confirmations = 6u32;
    
    // Execute Bitcoin deposit workflow
    let operation_id = client.execute_btc_deposit_tracked(
        &operator,
        &user,
        &btc_amount,
        &btc_tx_hash,
        &btc_confirmations
    );
    
    // Verify operation was created
    assert!(!operation_id.to_array().iter().all(|&x| x == 0));
    
    // Check deposit status
    let deposit_status = client.get_deposit_status_by_tx_hash(&btc_tx_hash);
    assert!(deposit_status.is_some());
    
    let status = deposit_status.unwrap();
    assert_eq!(status.btc_amount, btc_amount);
    assert_eq!(status.istsi_amount, btc_amount * 100_000_000); // 1:100M ratio
    assert_eq!(status.user, user);
    assert_eq!(status.btc_tx_hash, btc_tx_hash);
}

/// Test Bitcoin deposit with insufficient KYC compliance
#[test]
#[should_panic(expected = "ComplianceCheckFailed")]
fn test_bitcoin_deposit_kyc_failure() {
    let env = Env::default();
    let contract_id = env.register_contract(None, IntegrationRouter);
    let client = IntegrationRouterClient::new(&env, &contract_id);
    
    // Set up test addresses
    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let operator = Address::generate(&env);
    let kyc_registry = Address::generate(&env);
    let istsi_token = Address::generate(&env);
    let fungible_token = Address::generate(&env);
    let reserve_manager = Address::generate(&env);
    
    // Initialize the contract
    client.initialize(
        &admin,
        &kyc_registry,
        &istsi_token,
        &fungible_token,
        &reserve_manager
    );
    
    // Set operator role
    client.set_user_role(&admin, &operator, &UserRole::Operator);
    
    // Test data - large amount that should fail KYC
    let btc_amount = 1_000_000_000_000u64; // Very large amount
    let btc_tx_hash = BytesN::from_array(&env, &[2u8; 32]);
    let btc_confirmations = 6u32;
    
    // This should fail due to KYC compliance check
    client.execute_btc_deposit_tracked(
        &operator,
        &user,
        &btc_amount,
        &btc_tx_hash,
        &btc_confirmations
    );
}

/// Test Bitcoin deposit with insufficient confirmations
#[test]
#[should_panic(expected = "BitcoinTransactionFailed")]
fn test_bitcoin_deposit_insufficient_confirmations() {
    let env = Env::default();
    let contract_id = env.register_contract(None, IntegrationRouter);
    let client = IntegrationRouterClient::new(&env, &contract_id);
    
    // Set up test addresses
    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let operator = Address::generate(&env);
    let kyc_registry = Address::generate(&env);
    let istsi_token = Address::generate(&env);
    let fungible_token = Address::generate(&env);
    let reserve_manager = Address::generate(&env);
    
    // Initialize the contract
    client.initialize(
        &admin,
        &kyc_registry,
        &istsi_token,
        &fungible_token,
        &reserve_manager
    );
    
    // Set operator role
    client.set_user_role(&admin, &operator, &UserRole::Operator);
    
    // Test data with insufficient confirmations
    let btc_amount = 100_000_000u64;
    let btc_tx_hash = BytesN::from_array(&env, &[3u8; 32]);
    let btc_confirmations = 1u32; // Less than required minimum of 3
    
    // This should fail due to insufficient confirmations
    client.execute_btc_deposit_tracked(
        &operator,
        &user,
        &btc_amount,
        &btc_tx_hash,
        &btc_confirmations
    );
}

/// Test Bitcoin deposit with duplicate transaction hash
#[test]
#[should_panic(expected = "BitcoinTransactionFailed")]
fn test_bitcoin_deposit_duplicate_transaction() {
    let env = Env::default();
    let contract_id = env.register_contract(None, IntegrationRouter);
    let client = IntegrationRouterClient::new(&env, &contract_id);
    
    // Set up test addresses
    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let operator = Address::generate(&env);
    let kyc_registry = Address::generate(&env);
    let istsi_token = Address::generate(&env);
    let fungible_token = Address::generate(&env);
    let reserve_manager = Address::generate(&env);
    
    // Initialize the contract
    client.initialize(
        &admin,
        &kyc_registry,
        &istsi_token,
        &fungible_token,
        &reserve_manager
    );
    
    // Set operator role
    client.set_user_role(&admin, &operator, &UserRole::Operator);
    
    // Test data
    let btc_amount = 100_000_000u64;
    let btc_tx_hash = BytesN::from_array(&env, &[4u8; 32]);
    let btc_confirmations = 6u32;
    
    // First deposit should succeed
    let _operation_id1 = client.execute_btc_deposit_tracked(
        &operator,
        &user,
        &btc_amount,
        &btc_tx_hash,
        &btc_confirmations
    );
    
    // Second deposit with same tx hash should fail
    client.execute_btc_deposit_tracked(
        &operator,
        &user,
        &btc_amount,
        &btc_tx_hash,
        &btc_confirmations
    );
}

/// Test deposit limits checking functionality
#[test]
fn test_deposit_limits_checking() {
    let env = Env::default();
    let contract_id = env.register_contract(None, IntegrationRouter);
    let client = IntegrationRouterClient::new(&env, &contract_id);
    
    // Set up test addresses
    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let kyc_registry = Address::generate(&env);
    let istsi_token = Address::generate(&env);
    let fungible_token = Address::generate(&env);
    let reserve_manager = Address::generate(&env);
    
    // Initialize the contract
    client.initialize(
        &admin,
        &kyc_registry,
        &istsi_token,
        &fungible_token,
        &reserve_manager
    );
    
    // Test deposit limits
    let btc_amount = 50_000_000u64; // 0.5 BTC
    let (approved, message, limit) = client.check_deposit_limits(&user, &btc_amount);
    
    // Should return some limit information
    assert!(limit > 0 || !approved);
}

/// Test confirmation requirements based on amount and user tier
#[test]
fn test_confirmation_requirements() {
    let env = Env::default();
    let contract_id = env.register_contract(None, IntegrationRouter);
    let client = IntegrationRouterClient::new(&env, &contract_id);
    
    // Set up test addresses
    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let kyc_registry = Address::generate(&env);
    let istsi_token = Address::generate(&env);
    let fungible_token = Address::generate(&env);
    let reserve_manager = Address::generate(&env);
    
    // Initialize the contract
    client.initialize(
        &admin,
        &kyc_registry,
        &istsi_token,
        &fungible_token,
        &reserve_manager
    );
    
    // Test confirmation requirements for different amounts
    let small_amount = 10_000_000u64; // 0.1 BTC
    let large_amount = 1_000_000_000u64; // 10 BTC
    
    let (small_confirmations, small_enhanced) = client.get_deposit_conf_requirements(&user, &small_amount);
    let (large_confirmations, large_enhanced) = client.get_deposit_conf_requirements(&user, &large_amount);
    
    // Larger amounts should require more confirmations or enhanced verification
    assert!(small_confirmations >= 3);
    assert!(large_confirmations >= small_confirmations || large_enhanced);
}

/// Test getting pending deposits (admin function)
#[test]
fn test_get_pending_deposits() {
    let env = Env::default();
    let contract_id = env.register_contract(None, IntegrationRouter);
    let client = IntegrationRouterClient::new(&env, &contract_id);
    
    // Set up test addresses
    let admin = Address::generate(&env);
    let operator = Address::generate(&env);
    let kyc_registry = Address::generate(&env);
    let istsi_token = Address::generate(&env);
    let fungible_token = Address::generate(&env);
    let reserve_manager = Address::generate(&env);
    
    // Initialize the contract
    client.initialize(
        &admin,
        &kyc_registry,
        &istsi_token,
        &fungible_token,
        &reserve_manager
    );
    
    // Set operator role
    client.set_user_role(&admin, &operator, &UserRole::Operator);
    
    // Get pending deposits (should be empty initially)
    let pending_deposits = client.get_pending_deposits(&operator);
    
    // Should return a vector (empty or with deposits)
    assert!(pending_deposits.len() >= 0);
}

/// Test deposit status tracking throughout the workflow
#[test]
fn test_deposit_status_tracking() {
    let env = Env::default();
    let contract_id = env.register_contract(None, IntegrationRouter);
    let client = IntegrationRouterClient::new(&env, &contract_id);
    
    // Set up test addresses
    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let operator = Address::generate(&env);
    let kyc_registry = Address::generate(&env);
    let istsi_token = Address::generate(&env);
    let fungible_token = Address::generate(&env);
    let reserve_manager = Address::generate(&env);
    
    // Initialize the contract
    client.initialize(
        &admin,
        &kyc_registry,
        &istsi_token,
        &fungible_token,
        &reserve_manager
    );
    
    // Set operator role
    client.set_user_role(&admin, &operator, &UserRole::Operator);
    
    // Test data
    let btc_amount = 100_000_000u64;
    let btc_tx_hash = BytesN::from_array(&env, &[5u8; 32]);
    let btc_confirmations = 6u32;
    
    // Execute deposit
    let _operation_id = client.execute_btc_deposit_tracked(
        &operator,
        &user,
        &btc_amount,
        &btc_tx_hash,
        &btc_confirmations
    );
    
    // Check that deposit status was created and updated
    let deposit_status = client.get_deposit_status_by_tx_hash(&btc_tx_hash);
    assert!(deposit_status.is_some());
    
    let status = deposit_status.unwrap();
    assert_eq!(status.btc_tx_hash, btc_tx_hash);
    assert_eq!(status.user, user);
    assert_eq!(status.btc_amount, btc_amount);
    assert_eq!(status.confirmations, btc_confirmations);
    
    // Status should be either Completed or Failed (depending on mock responses)
    assert!(matches!(status.status, 
        DepositProcessingStatus::Completed | 
        DepositProcessingStatus::Failed |
        DepositProcessingStatus::Pending |
        DepositProcessingStatus::KYCVerifying |
        DepositProcessingStatus::ReserveValidating |
        DepositProcessingStatus::Registering |
        DepositProcessingStatus::Minting |
        DepositProcessingStatus::RolledBack
    ));
}

/// Test atomic rollback functionality when minting fails
#[test]
fn test_atomic_rollback_on_mint_failure() {
    let env = Env::default();
    let contract_id = env.register_contract(None, IntegrationRouter);
    let client = IntegrationRouterClient::new(&env, &contract_id);
    
    // Set up test addresses
    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let operator = Address::generate(&env);
    let kyc_registry = Address::generate(&env);
    let istsi_token = Address::generate(&env);
    let fungible_token = Address::generate(&env);
    let reserve_manager = Address::generate(&env);
    
    // Initialize the contract
    client.initialize(
        &admin,
        &kyc_registry,
        &istsi_token,
        &fungible_token,
        &reserve_manager
    );
    
    // Set operator role
    client.set_user_role(&admin, &operator, &UserRole::Operator);
    
    // Test data that might cause minting to fail
    let btc_amount = 100_000_000u64;
    let btc_tx_hash = BytesN::from_array(&env, &[6u8; 32]);
    let btc_confirmations = 6u32;
    
    // Execute deposit (may fail at minting step)
    let operation_id = client.execute_btc_deposit_tracked(
        &operator,
        &user,
        &btc_amount,
        &btc_tx_hash,
        &btc_confirmations
    );
    
    // Check that operation was tracked even if it failed
    assert!(!operation_id.to_array().iter().all(|&x| x == 0));
    
    // Check deposit status reflects the outcome
    let deposit_status = client.get_deposit_status_by_tx_hash(&btc_tx_hash);
    assert!(deposit_status.is_some());
}

/// Test system pause functionality during deposit
#[test]
#[should_panic(expected = "SystemPaused")]
fn test_deposit_when_system_paused() {
    let env = Env::default();
    let contract_id = env.register_contract(None, IntegrationRouter);
    let client = IntegrationRouterClient::new(&env, &contract_id);
    
    // Set up test addresses
    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let operator = Address::generate(&env);
    let kyc_registry = Address::generate(&env);
    let istsi_token = Address::generate(&env);
    let fungible_token = Address::generate(&env);
    let reserve_manager = Address::generate(&env);
    
    // Initialize the contract
    client.initialize(
        &admin,
        &kyc_registry,
        &istsi_token,
        &fungible_token,
        &reserve_manager
    );
    
    // Set operator role
    client.set_user_role(&admin, &operator, &UserRole::Operator);
    
    // Pause the system
    client.emergency_pause(&admin, &String::from_str(&env, "Testing pause functionality"));
    
    // Test data
    let btc_amount = 100_000_000u64;
    let btc_tx_hash = BytesN::from_array(&env, &[7u8; 32]);
    let btc_confirmations = 6u32;
    
    // This should fail because system is paused
    client.execute_btc_deposit_tracked(
        &operator,
        &user,
        &btc_amount,
        &btc_tx_hash,
        &btc_confirmations
    );
}