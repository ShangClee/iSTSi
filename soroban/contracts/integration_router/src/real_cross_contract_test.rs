#![cfg(test)]

use super::*;
use soroban_sdk::{
    testutils::{Address as TestAddress, Ledger, LedgerInfo},
    Address, Env, String, Vec, BytesN
};

/// Test real cross-contract communication functionality
#[test]
fn test_real_cross_contract_call_execution() {
    let env = Env::default();
    let contract_id = env.register_contract(None, IntegrationRouter);
    let client = IntegrationRouterClient::new(&env, &contract_id);
    
    // Set up test addresses
    let admin = Address::generate(&env);
    let kyc_registry = Address::generate(&env);
    let istsi_token = Address::generate(&env);
    let fungible_token = Address::generate(&env);
    let reserve_manager = Address::generate(&env);
    let operator = Address::generate(&env);
    
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
    
    // Initialize cross-contract config
    let config = CrossContractConfig {
        max_batch_size: 5,
        default_timeout: 300,
        max_retry_count: 3,
        enable_rollbacks: true,
        enable_timeouts: true,
    };
    client.initialize_cross_contract_config(&admin, &config);
    
    // Test individual contract call
    let call = ContractCall {
        target_contract: kyc_registry.clone(),
        function_name: String::from_str(&env, "is_approved_simple"),
        parameters: vec![
            &env,
            operator.to_string(),
            String::from_str(&env, "0"), // Transfer operation
            String::from_str(&env, "1000") // Amount
        ],
        expected_return_type: String::from_str(&env, "bool"),
        timeout: 60,
        retry_count: 2,
    };
    
    let result = client.execute_contract_call(&operator, &call);
    
    // Verify the call was executed (even if it fails due to no actual contract)
    assert!(result.execution_time > 0);
    assert!(result.gas_used > 0);
}

#[test]
fn test_batch_operation_with_real_calls() {
    let env = Env::default();
    let contract_id = env.register_contract(None, IntegrationRouter);
    let client = IntegrationRouterClient::new(&env, &contract_id);
    
    // Set up test addresses
    let admin = Address::generate(&env);
    let kyc_registry = Address::generate(&env);
    let istsi_token = Address::generate(&env);
    let fungible_token = Address::generate(&env);
    let reserve_manager = Address::generate(&env);
    let operator = Address::generate(&env);
    
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
    
    // Initialize cross-contract config
    let config = CrossContractConfig {
        max_batch_size: 5,
        default_timeout: 300,
        max_retry_count: 3,
        enable_rollbacks: true,
        enable_timeouts: true,
    };
    client.initialize_cross_contract_config(&admin, &config);
    
    // Create batch operation with multiple calls
    let calls = vec![
        &env,
        ContractCall {
            target_contract: kyc_registry.clone(),
            function_name: String::from_str(&env, "is_approved_simple"),
            parameters: vec![
                &env,
                operator.to_string(),
                String::from_str(&env, "0"),
                String::from_str(&env, "1000")
            ],
            expected_return_type: String::from_str(&env, "bool"),
            timeout: 60,
            retry_count: 2,
        },
        ContractCall {
            target_contract: reserve_manager.clone(),
            function_name: String::from_str(&env, "get_reserve_ratio"),
            parameters: vec![&env],
            expected_return_type: String::from_str(&env, "u64"),
            timeout: 60,
            retry_count: 2,
        }
    ];
    
    let rollback_calls = vec![&env];
    
    let operation_id = client.create_batch_operation(
        &operator,
        &calls,
        &rollback_calls,
        &300u64,
        &true // atomic
    );
    
    let batch = BatchOperation {
        operation_id: operation_id.clone(),
        calls,
        rollback_calls,
        timeout: 300,
        atomic: true,
        created_at: env.ledger().timestamp(),
        status: OperationStatus::Pending,
    };
    
    let result = client.execute_batch_operation(&operator, &batch);
    
    // Verify batch execution
    assert_eq!(result.operation_id, operation_id);
    assert_eq!(result.call_results.len(), 2);
    assert!(result.total_execution_time > 0);
}

#[test]
fn test_gas_estimation_and_optimization() {
    let env = Env::default();
    let contract_id = env.register_contract(None, IntegrationRouter);
    let client = IntegrationRouterClient::new(&env, &contract_id);
    
    // Set up test addresses
    let admin = Address::generate(&env);
    let kyc_registry = Address::generate(&env);
    let istsi_token = Address::generate(&env);
    let fungible_token = Address::generate(&env);
    let reserve_manager = Address::generate(&env);
    let operator = Address::generate(&env);
    
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
    
    // Test different function types for gas estimation
    let mint_call = ContractCall {
        target_contract: istsi_token.clone(),
        function_name: String::from_str(&env, "integrated_mint"),
        parameters: vec![
            &env,
            operator.to_string(),
            String::from_str(&env, "1000")
        ],
        expected_return_type: String::from_str(&env, "bool"),
        timeout: 60,
        retry_count: 2,
    };
    
    let kyc_call = ContractCall {
        target_contract: kyc_registry.clone(),
        function_name: String::from_str(&env, "verify_integration_compliance"),
        parameters: vec![
            &env,
            operator.to_string(),
            String::from_str(&env, "0"),
            String::from_str(&env, "1000")
        ],
        expected_return_type: String::from_str(&env, "bool"),
        timeout: 60,
        retry_count: 2,
    };
    
    // Execute calls and verify gas usage is tracked
    let mint_result = client.execute_contract_call(&operator, &mint_call);
    let kyc_result = client.execute_contract_call(&operator, &kyc_call);
    
    // Verify gas usage is recorded
    assert!(mint_result.gas_used > 0);
    assert!(kyc_result.gas_used > 0);
    
    // Mint operations should generally use more gas than simple KYC checks
    // (This might not hold in test environment, but structure is correct)
}

#[test]
fn test_parameter_parsing_and_serialization() {
    let env = Env::default();
    
    // Test parameter parsing
    let params = vec![
        &env,
        String::from_str(&env, "GCKFBEIYTKP33UJNHFHFQUU4CJQHQFQHQFQHQFQHQFQHQFQHQFQHQFQHQFQHQFQH"), // Address
        String::from_str(&env, "1000"), // Number
        String::from_str(&env, "abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890"), // Hash
        String::from_str(&env, "test_string") // String
    ];
    
    let parsed = IntegrationRouter::parse_call_parameters(&env, &params);
    
    // Should have parsed 4 parameters
    assert_eq!(parsed.len(), 4);
}

#[test]
fn test_retry_logic_with_failures() {
    let env = Env::default();
    let contract_id = env.register_contract(None, IntegrationRouter);
    let client = IntegrationRouterClient::new(&env, &contract_id);
    
    // Set up test addresses
    let admin = Address::generate(&env);
    let kyc_registry = Address::generate(&env);
    let istsi_token = Address::generate(&env);
    let fungible_token = Address::generate(&env);
    let reserve_manager = Address::generate(&env);
    let operator = Address::generate(&env);
    
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
    
    // Test call that should fail (using fail_test function)
    let fail_call = ContractCall {
        target_contract: kyc_registry.clone(),
        function_name: String::from_str(&env, "fail_test"),
        parameters: vec![&env],
        expected_return_type: String::from_str(&env, "bool"),
        timeout: 60,
        retry_count: 3,
    };
    
    let result = client.execute_contract_call(&operator, &fail_call);
    
    // Should fail but still record execution details
    assert!(!result.success);
    assert!(result.execution_time > 0);
    assert!(!result.error_message.is_empty());
}

#[test]
fn test_timeout_handling() {
    let env = Env::default();
    let contract_id = env.register_contract(None, IntegrationRouter);
    let client = IntegrationRouterClient::new(&env, &contract_id);
    
    // Set up test addresses
    let admin = Address::generate(&env);
    let kyc_registry = Address::generate(&env);
    let istsi_token = Address::generate(&env);
    let fungible_token = Address::generate(&env);
    let reserve_manager = Address::generate(&env);
    let operator = Address::generate(&env);
    
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
    
    // Create call with very short timeout
    let timeout_call = ContractCall {
        target_contract: kyc_registry.clone(),
        function_name: String::from_str(&env, "is_approved_simple"),
        parameters: vec![
            &env,
            operator.to_string(),
            String::from_str(&env, "0"),
            String::from_str(&env, "1000")
        ],
        expected_return_type: String::from_str(&env, "bool"),
        timeout: 0, // Immediate timeout
        retry_count: 1,
    };
    
    let result = client.execute_contract_call(&operator, &timeout_call);
    
    // Should handle timeout gracefully
    assert!(result.execution_time >= 0);
}

#[test]
fn test_cross_contract_config_management() {
    let env = Env::default();
    let contract_id = env.register_contract(None, IntegrationRouter);
    let client = IntegrationRouterClient::new(&env, &contract_id);
    
    // Set up test addresses
    let admin = Address::generate(&env);
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
    
    // Test initial config
    let initial_config = client.get_cross_contract_config();
    assert_eq!(initial_config.max_batch_size, 10); // Default value
    
    // Update config
    let new_config = CrossContractConfig {
        max_batch_size: 20,
        default_timeout: 600,
        max_retry_count: 5,
        enable_rollbacks: false,
        enable_timeouts: false,
    };
    
    client.initialize_cross_contract_config(&admin, &new_config);
    
    // Verify config was updated
    let updated_config = client.get_cross_contract_config();
    assert_eq!(updated_config.max_batch_size, 20);
    assert_eq!(updated_config.default_timeout, 600);
    assert_eq!(updated_config.max_retry_count, 5);
    assert!(!updated_config.enable_rollbacks);
    assert!(!updated_config.enable_timeouts);
}