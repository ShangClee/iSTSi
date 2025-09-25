#![cfg(test)]

use crate::{IntegrationRouter, IntegrationRouterClient, CrossContractConfig, ContractCall, BatchOperation, OperationStatus};
use soroban_sdk::{testutils::Address as _, Env, Address, String, Vec, BytesN};

#[test]
fn test_cross_contract_basic_functionality() {
    let env = Env::default();
    env.mock_all_auths();
    
    let contract_id = env.register_contract(None, IntegrationRouter);
    let client = IntegrationRouterClient::new(&env, &contract_id);
    
    // Create test addresses
    let admin = Address::generate(&env);
    let kyc_registry = Address::generate(&env);
    let istsi_token = Address::generate(&env);
    let fungible_token = Address::generate(&env);
    let reserve_manager = Address::generate(&env);
    
    // Initialize the contract
    client.initialize(&admin, &kyc_registry, &istsi_token, &fungible_token, &reserve_manager);
    
    // Test cross-contract configuration
    let config = CrossContractConfig {
        max_batch_size: 10,
        default_timeout: 300,
        max_retry_count: 3,
        enable_rollbacks: true,
        enable_timeouts: true,
    };
    
    client.initialize_cross_contract_config(&admin, &config);
    
    // Verify configuration
    let stored_config = client.get_cross_contract_config();
    assert_eq!(stored_config.max_batch_size, 10);
    assert_eq!(stored_config.default_timeout, 300);
    assert_eq!(stored_config.max_retry_count, 3);
    assert_eq!(stored_config.enable_rollbacks, true);
    assert_eq!(stored_config.enable_timeouts, true);
    
    // Test single contract call
    let call = ContractCall {
        target_contract: istsi_token.clone(),
        function_name: String::from_str(&env, "mint"),
        parameters: Vec::new(&env),
        expected_return_type: String::from_str(&env, "void"),
        timeout: 60,
        retry_count: 1,
    };
    
    let result = client.execute_contract_call(&admin, &call);
    assert!(result.success);
    assert_eq!(result.return_data, String::from_str(&env, "success"));
    
    // Test batch operation creation
    let calls = Vec::from_array(&env, [call.clone()]);
    let rollback_calls = Vec::new(&env);
    
    let operation_id = client.create_batch_operation(
        &admin,
        &calls,
        &rollback_calls,
        300,
        true
    );
    
    // Verify batch operation was created
    let batch = client.get_batch_operation(&operation_id).unwrap();
    assert_eq!(batch.calls.len(), 1);
    assert_eq!(batch.atomic, true);
    assert_eq!(batch.status, OperationStatus::Pending);
    
    // Execute batch operation
    let batch_result = client.execute_batch_operation(&admin, &batch);
    assert!(batch_result.overall_success);
    assert_eq!(batch_result.call_results.len(), 1);
    assert!(batch_result.call_results.get(0).unwrap().success);
    
    // Test operation tracking
    let status = client.get_operation_status(&operation_id).unwrap();
    assert_eq!(status.operation_id, operation_id);
    
    // Test operation lists
    let completed_ops = client.get_completed_operations();
    assert!(completed_ops.len() > 0);
    
    let pending_ops = client.get_pending_operations();
    // Should be empty since we executed the operation
    assert_eq!(pending_ops.len(), 0);
    
    let failed_ops = client.get_failed_operations();
    assert_eq!(failed_ops.len(), 0);
}

#[test]
fn test_cross_contract_failure_handling() {
    let env = Env::default();
    env.mock_all_auths();
    
    let contract_id = env.register_contract(None, IntegrationRouter);
    let client = IntegrationRouterClient::new(&env, &contract_id);
    
    // Create test addresses
    let admin = Address::generate(&env);
    let kyc_registry = Address::generate(&env);
    let istsi_token = Address::generate(&env);
    let fungible_token = Address::generate(&env);
    let reserve_manager = Address::generate(&env);
    
    // Initialize the contract
    client.initialize(&admin, &kyc_registry, &istsi_token, &fungible_token, &reserve_manager);
    
    let config = CrossContractConfig {
        max_batch_size: 10,
        default_timeout: 300,
        max_retry_count: 3,
        enable_rollbacks: true,
        enable_timeouts: true,
    };
    
    client.initialize_cross_contract_config(&admin, &config);
    
    // Test failing contract call
    let failing_call = ContractCall {
        target_contract: istsi_token.clone(),
        function_name: String::from_str(&env, "fail_test"),
        parameters: Vec::new(&env),
        expected_return_type: String::from_str(&env, "void"),
        timeout: 60,
        retry_count: 1,
    };
    
    let result = client.execute_contract_call(&admin, &failing_call);
    assert!(!result.success);
    assert_eq!(result.error_message, String::from_str(&env, "Simulated failure"));
    
    // Test atomic batch with failure
    let calls = Vec::from_array(&env, [failing_call.clone()]);
    let rollback_calls = Vec::new(&env);
    
    let operation_id = client.create_batch_operation(
        &admin,
        &calls,
        &rollback_calls,
        300,
        true // atomic
    );
    
    let batch = client.get_batch_operation(&operation_id).unwrap();
    let batch_result = client.execute_batch_operation(&admin, &batch);
    
    assert!(!batch_result.overall_success);
    assert_eq!(batch_result.call_results.len(), 1);
    assert!(!batch_result.call_results.get(0).unwrap().success);
    
    // Check that operation is in failed list
    let failed_ops = client.get_failed_operations();
    assert!(failed_ops.len() > 0);
}

#[test]
fn test_operation_cancellation() {
    let env = Env::default();
    env.mock_all_auths();
    
    let contract_id = env.register_contract(None, IntegrationRouter);
    let client = IntegrationRouterClient::new(&env, &contract_id);
    
    // Create test addresses
    let admin = Address::generate(&env);
    let kyc_registry = Address::generate(&env);
    let istsi_token = Address::generate(&env);
    let fungible_token = Address::generate(&env);
    let reserve_manager = Address::generate(&env);
    
    // Initialize the contract
    client.initialize(&admin, &kyc_registry, &istsi_token, &fungible_token, &reserve_manager);
    
    let config = CrossContractConfig {
        max_batch_size: 10,
        default_timeout: 300,
        max_retry_count: 3,
        enable_rollbacks: true,
        enable_timeouts: true,
    };
    
    client.initialize_cross_contract_config(&admin, &config);
    
    // Create a batch operation but don't execute it
    let call = ContractCall {
        target_contract: istsi_token.clone(),
        function_name: String::from_str(&env, "mint"),
        parameters: Vec::new(&env),
        expected_return_type: String::from_str(&env, "void"),
        timeout: 60,
        retry_count: 1,
    };
    
    let calls = Vec::from_array(&env, [call]);
    let rollback_calls = Vec::new(&env);
    
    let operation_id = client.create_batch_operation(
        &admin,
        &calls,
        &rollback_calls,
        300,
        true
    );
    
    // Verify it's pending
    let status = client.get_operation_status(&operation_id).unwrap();
    assert_eq!(status.status, OperationStatus::Pending);
    
    // Cancel the operation
    let cancelled = client.cancel_operation(&admin, &operation_id);
    assert!(cancelled);
    
    // Verify it's now failed
    let status = client.get_operation_status(&operation_id).unwrap();
    assert_eq!(status.status, OperationStatus::Failed);
    assert_eq!(status.error_message, String::from_str(&env, "Cancelled by user"));
    
    // Check that it's in the failed operations list
    let failed_ops = client.get_failed_operations();
    assert!(failed_ops.contains(&operation_id));
}