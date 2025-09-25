#![cfg(test)]

use super::*;
use soroban_sdk::{
    testutils::{Address as TestAddress, Ledger, LedgerInfo},
    Address, Env
};

#[test]
fn test_oracle_basic_functionality() {
    let env = Env::default();
    let contract_id = env.register_contract(None, IntegrationRouter);
    let client = IntegrationRouterClient::new(&env, &contract_id);

    // Setup test addresses
    let admin = Address::generate(&env);
    let oracle = Address::generate(&env);
    let token_a = Address::generate(&env);
    let token_b = Address::generate(&env);
    let kyc_registry = Address::generate(&env);
    let istsi_token = Address::generate(&env);
    let fungible_token = Address::generate(&env);
    let reserve_manager = Address::generate(&env);

    // Initialize router
    client.initialize(
        &admin,
        &kyc_registry,
        &istsi_token,
        &fungible_token,
        &reserve_manager,
    );

    // Configure oracle
    client.configure_oracle(
        &admin,
        &token_a,
        &token_b,
        &oracle,
        &300u64, // 5 minutes update frequency
        &500u64, // 5% max deviation
        &10000u64, // 1:1 fallback rate
    );
}

#[test]
fn test_get_exchange_rate() {
    let env = Env::default();
    let contract_id = env.register_contract(None, IntegrationRouter);
    let client = IntegrationRouterClient::new(&env, &contract_id);

    // Setup test addresses
    let admin = Address::generate(&env);
    let oracle = Address::generate(&env);
    let token_a = Address::generate(&env);
    let token_b = Address::generate(&env);
    let kyc_registry = Address::generate(&env);
    let istsi_token = Address::generate(&env);
    let fungible_token = Address::generate(&env);
    let reserve_manager = Address::generate(&env);

    // Initialize router
    client.initialize(
        &admin,
        &kyc_registry,
        &istsi_token,
        &fungible_token,
        &reserve_manager,
    );

    // Configure oracle
    client.configure_oracle(
        &admin,
        &token_a,
        &token_b,
        &oracle,
        &300u64,
        &500u64,
        &10000u64,
    );

    // Get exchange rate (should use fallback since oracle is simulated)
    let rate = client.get_exchange_rate(&token_a, &token_b);

    assert_eq!(rate.rate, 10000); // Should be fallback rate
    assert_eq!(rate.fee_rate, 50); // Higher fee for fallback
}

#[test]
fn test_calculate_exchange_amount() {
    let env = Env::default();
    let contract_id = env.register_contract(None, IntegrationRouter);
    let client = IntegrationRouterClient::new(&env, &contract_id);

    // Setup test addresses
    let admin = Address::generate(&env);
    let oracle = Address::generate(&env);
    let token_a = Address::generate(&env);
    let token_b = Address::generate(&env);
    let kyc_registry = Address::generate(&env);
    let istsi_token = Address::generate(&env);
    let fungible_token = Address::generate(&env);
    let reserve_manager = Address::generate(&env);

    // Initialize router
    client.initialize(
        &admin,
        &kyc_registry,
        &istsi_token,
        &fungible_token,
        &reserve_manager,
    );

    // Configure oracle
    client.configure_oracle(
        &admin,
        &token_a,
        &token_b,
        &oracle,
        &300u64,
        &500u64,
        &10000u64,
    );

    let from_amount = 1000u64;
    let max_slippage = 100u64; // 1% max slippage

    let quote = client.calculate_exchange_amount(
        &token_a,
        &token_b,
        &from_amount,
        &max_slippage,
    );

    assert_eq!(quote.from_amount, from_amount);
    assert_eq!(quote.exchange_rate, 10000); // 1:1 fallback rate
    
    // Calculate expected amounts
    let expected_fee = (from_amount * 50) / 10000; // 0.5% fee for fallback
    let expected_to_amount = ((from_amount - expected_fee) * 10000) / 10000;
    
    assert_eq!(quote.fee_amount, expected_fee);
    assert_eq!(quote.to_amount, expected_to_amount);
}

#[test]
fn test_oracle_status() {
    let env = Env::default();
    let contract_id = env.register_contract(None, IntegrationRouter);
    let client = IntegrationRouterClient::new(&env, &contract_id);

    // Setup test addresses
    let admin = Address::generate(&env);
    let oracle = Address::generate(&env);
    let token_a = Address::generate(&env);
    let token_b = Address::generate(&env);
    let kyc_registry = Address::generate(&env);
    let istsi_token = Address::generate(&env);
    let fungible_token = Address::generate(&env);
    let reserve_manager = Address::generate(&env);

    // Initialize router
    client.initialize(
        &admin,
        &kyc_registry,
        &istsi_token,
        &fungible_token,
        &reserve_manager,
    );

    // Configure oracle
    client.configure_oracle(
        &admin,
        &token_a,
        &token_b,
        &oracle,
        &300u64,
        &500u64,
        &10000u64,
    );

    let status = client.get_oracle_status();

    assert_eq!(status.enabled, true);
    assert_eq!(status.oracle_address, oracle);
    // Health status will be Degraded since we simulate oracle failures
    assert_eq!(status.health_status, OracleHealthStatus::Degraded);
}

#[test]
fn test_update_oracle_config() {
    let env = Env::default();
    let contract_id = env.register_contract(None, IntegrationRouter);
    let client = IntegrationRouterClient::new(&env, &contract_id);

    // Setup test addresses
    let admin = Address::generate(&env);
    let oracle = Address::generate(&env);
    let token_a = Address::generate(&env);
    let token_b = Address::generate(&env);
    let kyc_registry = Address::generate(&env);
    let istsi_token = Address::generate(&env);
    let fungible_token = Address::generate(&env);
    let reserve_manager = Address::generate(&env);

    // Initialize router
    client.initialize(
        &admin,
        &kyc_registry,
        &istsi_token,
        &fungible_token,
        &reserve_manager,
    );

    // Configure oracle
    client.configure_oracle(
        &admin,
        &token_a,
        &token_b,
        &oracle,
        &300u64,
        &500u64,
        &10000u64,
    );

    let new_oracle = Address::generate(&env);

    // Update oracle configuration
    client.update_oracle_config(
        &admin,
        &Some(new_oracle.clone()),
        &Some(600u64), // 10 minutes
        &Some(1000u64), // 10% max deviation
        &Some(9500u64), // 0.95:1 fallback rate
        &Some(false), // Disable oracle
    );

    // Verify the oracle is disabled
    let status = client.get_oracle_status();
    assert_eq!(status.enabled, false);
    assert_eq!(status.health_status, OracleHealthStatus::Offline);
}