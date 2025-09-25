#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::Address as _, Env, Address};

fn create_test_addresses(env: &Env) -> (Address, Address, Address, Address, Address, Address) {
    (
        Address::generate(env),  // admin
        Address::generate(env),  // kyc_registry
        Address::generate(env),  // istsi_token
        Address::generate(env),  // fungible_token
        Address::generate(env),  // reserve_manager
        Address::generate(env),  // user
    )
}

fn setup_test_env() -> (Env, IntegrationRouterClient<'static>, Address, Address, Address, Address, Address, Address) {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(IntegrationRouter, ());
    let client = IntegrationRouterClient::new(&env, &contract_id);
    
    let (admin, kyc_registry, istsi_token, fungible_token, reserve_manager, user) = create_test_addresses(&env);
    
    (env, client, admin, kyc_registry, istsi_token, fungible_token, reserve_manager, user)
}

#[test]
fn test_initialize() {
    let (_env, client, admin, kyc_registry, istsi_token, fungible_token, reserve_manager, _user) = setup_test_env();
    
    // Initialize the contract
    client.initialize(&admin, &kyc_registry, &istsi_token, &fungible_token, &reserve_manager);
    
    // Verify configuration
    let config = client.get_config();
    assert_eq!(config.admin, admin);
    assert_eq!(config.kyc_registry, kyc_registry);
    assert_eq!(config.istsi_token, istsi_token);
    assert_eq!(config.fungible_token, fungible_token);
    assert_eq!(config.reserve_manager, reserve_manager);
    assert_eq!(config.paused, false);
    
    // Verify admin role
    assert_eq!(client.get_user_role(&admin), UserRole::SuperAdmin);
    
    // Verify system state
    assert_eq!(client.is_paused(), false);
    
    // Verify contract addresses are registered
    assert_eq!(client.get_contract_address(&String::from_str(&_env, "kyc_registry")), Some(kyc_registry));
    assert_eq!(client.get_contract_address(&String::from_str(&_env, "istsi_token")), Some(istsi_token));
}

#[test]
#[should_panic(expected = "Contract already initialized")]
fn test_initialize_twice_fails() {
    let (_env, client, admin, kyc_registry, istsi_token, fungible_token, reserve_manager, _user) = setup_test_env();
    
    // Initialize once
    client.initialize(&admin, &kyc_registry, &istsi_token, &fungible_token, &reserve_manager);
    
    // Try to initialize again - should fail
    client.initialize(&admin, &kyc_registry, &istsi_token, &fungible_token, &reserve_manager);
}

#[test]
fn test_role_management() {
    let (_env, client, admin, kyc_registry, istsi_token, fungible_token, reserve_manager, user) = setup_test_env();
    
    // Initialize
    client.initialize(&admin, &kyc_registry, &istsi_token, &fungible_token, &reserve_manager);
    
    // Set user as operator
    client.set_user_role(&admin, &user, &UserRole::Operator);
    assert_eq!(client.get_user_role(&user), UserRole::Operator);
    
    // Verify user is in operators list
    let operators = client.get_operators();
    assert!(operators.contains(&user));
    
    // Set user as compliance officer
    client.set_user_role(&admin, &user, &UserRole::ComplianceOfficer);
    assert_eq!(client.get_user_role(&user), UserRole::ComplianceOfficer);
    
    // Remove user role
    client.remove_user_role(&admin, &user);
    assert_eq!(client.get_user_role(&user), UserRole::User);
    
    // Verify user is removed from operators list
    let operators = client.get_operators();
    assert!(!operators.contains(&user));
}

#[test]
fn test_emergency_pause_by_admin() {
    let (env, client, admin, kyc_registry, istsi_token, fungible_token, reserve_manager, _user) = setup_test_env();
    
    // Initialize
    client.initialize(&admin, &kyc_registry, &istsi_token, &fungible_token, &reserve_manager);
    
    // Pause system
    let reason = String::from_str(&env, "Security incident");
    client.emergency_pause(&admin, &reason);
    
    // Verify system is paused
    assert_eq!(client.is_paused(), true);
    
    let config = client.get_config();
    assert_eq!(config.paused, true);
}

#[test]
fn test_emergency_pause_by_compliance_officer() {
    let (env, client, admin, kyc_registry, istsi_token, fungible_token, reserve_manager, compliance_officer) = setup_test_env();
    
    // Initialize
    client.initialize(&admin, &kyc_registry, &istsi_token, &fungible_token, &reserve_manager);
    
    // Set compliance officer role
    client.set_user_role(&admin, &compliance_officer, &UserRole::ComplianceOfficer);
    
    // Pause system as compliance officer
    let reason = String::from_str(&env, "Compliance violation");
    client.emergency_pause(&compliance_officer, &reason);
    
    // Verify system is paused
    assert_eq!(client.is_paused(), true);
}

#[test]
fn test_resume_operations() {
    let (env, client, admin, kyc_registry, istsi_token, fungible_token, reserve_manager, _user) = setup_test_env();
    
    // Initialize
    client.initialize(&admin, &kyc_registry, &istsi_token, &fungible_token, &reserve_manager);
    
    // Pause system
    let reason = String::from_str(&env, "Maintenance");
    client.emergency_pause(&admin, &reason);
    assert_eq!(client.is_paused(), true);
    
    // Resume operations
    client.resume_operations(&admin);
    assert_eq!(client.is_paused(), false);
    
    let config = client.get_config();
    assert_eq!(config.paused, false);
}

#[test]
fn test_update_contract_address() {
    let (env, client, admin, kyc_registry, istsi_token, fungible_token, reserve_manager, _user) = setup_test_env();
    
    // Initialize
    client.initialize(&admin, &kyc_registry, &istsi_token, &fungible_token, &reserve_manager);
    
    // Update KYC registry address
    let new_kyc_registry = Address::generate(&env);
    client.update_contract_address(&admin, &String::from_str(&env, "kyc_registry"), &new_kyc_registry);
    
    // Verify address was updated
    assert_eq!(client.get_contract_address(&String::from_str(&env, "kyc_registry")), Some(new_kyc_registry.clone()));
    
    // Verify config was updated
    let config = client.get_config();
    assert_eq!(config.kyc_registry, new_kyc_registry);
}

#[test]
fn test_role_hierarchy() {
    let (env, client, admin, kyc_registry, istsi_token, fungible_token, reserve_manager, _user) = setup_test_env();
    let system_admin = Address::generate(&env);
    
    // Initialize
    client.initialize(&admin, &kyc_registry, &istsi_token, &fungible_token, &reserve_manager);
    
    // Set system admin
    client.set_user_role(&admin, &system_admin, &UserRole::SystemAdmin);
    
    // System admin should be able to pause
    let reason = String::from_str(&env, "System maintenance");
    client.emergency_pause(&system_admin, &reason);
    assert_eq!(client.is_paused(), true);
    
    // But system admin should NOT be able to resume (only SuperAdmin can)
    // This should work because SuperAdmin (admin) can resume
    client.resume_operations(&admin);
    assert_eq!(client.is_paused(), false);
}

#[test]
fn test_contract_address_registry() {
    let (env, client, admin, kyc_registry, istsi_token, fungible_token, reserve_manager, _user) = setup_test_env();
    
    // Initialize
    client.initialize(&admin, &kyc_registry, &istsi_token, &fungible_token, &reserve_manager);
    
    // Add a new contract
    let oracle_contract = Address::generate(&env);
    client.update_contract_address(&admin, &String::from_str(&env, "oracle"), &oracle_contract);
    
    // Verify it's registered
    assert_eq!(client.get_contract_address(&String::from_str(&env, "oracle")), Some(oracle_contract));
    
    // Verify non-existent contract returns None
    assert_eq!(client.get_contract_address(&String::from_str(&env, "nonexistent")), None);
}

#[test]
fn test_emit_integration_event() {
    let (env, client, admin, kyc_registry, istsi_token, fungible_token, reserve_manager, user) = setup_test_env();
    
    // Initialize
    client.initialize(&admin, &kyc_registry, &istsi_token, &fungible_token, &reserve_manager);
    
    // Set user as operator so they can emit events
    client.set_user_role(&admin, &user, &UserRole::Operator);
    
    // Create a test event
    let tx_hash = BytesN::from_array(&env, &[2u8; 32]);
    let timestamp = env.ledger().timestamp();
    
    let event = IntegrationEvent {
        event_type: String::from_str(&env, "BitcoinDeposit"),
        user: user.clone(),
        data1: 100000000, // btc_amount
        data2: 100000000, // istsi_minted
        data3: 0,
        address1: Address::generate(&env),
        address2: Address::generate(&env),
        hash_data: tx_hash.clone(),
        text_data: String::from_str(&env, ""),
        timestamp,
        correlation_id: BytesN::from_array(&env, &[1u8; 32]),
    };
    
    // Emit the event
    let returned_correlation_id = client.emit_integration_event(&user, &event);
    
    // Verify correlation ID was generated (should not be zero)
    assert_ne!(returned_correlation_id, BytesN::from_array(&env, &[0u8; 32]));
}

#[test]
fn test_event_subscription() {
    let (env, client, admin, kyc_registry, istsi_token, fungible_token, reserve_manager, user) = setup_test_env();
    
    // Initialize
    client.initialize(&admin, &kyc_registry, &istsi_token, &fungible_token, &reserve_manager);
    
    // Subscribe to all events
    let filter = EventFilter::All;
    client.subscribe_to_events(&user, &filter);
    
    // Get subscriptions (as admin)
    let subscriptions = client.get_event_subscriptions(&admin);
    assert_eq!(subscriptions.len(), 1);
    assert_eq!(subscriptions.get(0).unwrap().subscriber, user);
    assert_eq!(subscriptions.get(0).unwrap().active, true);
    
    // Unsubscribe
    client.unsubscribe_from_events(&user);
    
    // Verify subscription removed
    let subscriptions = client.get_event_subscriptions(&admin);
    assert_eq!(subscriptions.len(), 0);
}

#[test]
fn test_event_subscription_by_user() {
    let (env, client, admin, kyc_registry, istsi_token, fungible_token, reserve_manager, user) = setup_test_env();
    
    // Initialize
    client.initialize(&admin, &kyc_registry, &istsi_token, &fungible_token, &reserve_manager);
    
    // Subscribe to events for specific user
    let filter = EventFilter::ByUser(user.clone());
    client.subscribe_to_events(&user, &filter);
    
    // Get subscriptions
    let subscriptions = client.get_event_subscriptions(&admin);
    assert_eq!(subscriptions.len(), 1);
    
    let subscription = subscriptions.get(0).unwrap();
    match &subscription.filter {
        EventFilter::ByUser(filtered_user) => {
            assert_eq!(*filtered_user, user);
        },
        _ => panic!("Expected ByUser filter"),
    }
}

#[test]
fn test_event_subscription_by_event_type() {
    let (env, client, admin, kyc_registry, istsi_token, fungible_token, reserve_manager, user) = setup_test_env();
    
    // Initialize
    client.initialize(&admin, &kyc_registry, &istsi_token, &fungible_token, &reserve_manager);
    
    // Subscribe to Bitcoin deposit events only
    let event_type = String::from_str(&env, "BitcoinDeposit");
    let filter = EventFilter::ByEventType(event_type.clone());
    client.subscribe_to_events(&user, &filter);
    
    // Get subscriptions
    let subscriptions = client.get_event_subscriptions(&admin);
    assert_eq!(subscriptions.len(), 1);
    
    let subscription = subscriptions.get(0).unwrap();
    match &subscription.filter {
        EventFilter::ByEventType(filtered_type) => {
            assert_eq!(*filtered_type, event_type);
        },
        _ => panic!("Expected ByEventType filter"),
    }
}

#[test]
fn test_get_event_history() {
    let (env, client, admin, kyc_registry, istsi_token, fungible_token, reserve_manager, user) = setup_test_env();
    
    // Initialize
    client.initialize(&admin, &kyc_registry, &istsi_token, &fungible_token, &reserve_manager);
    
    // Set user as operator so they can emit events
    client.set_user_role(&admin, &user, &UserRole::Operator);
    
    // Create and emit a test event
    let tx_hash = BytesN::from_array(&env, &[2u8; 32]);
    let timestamp = env.ledger().timestamp();
    
    let event = IntegrationEvent {
        event_type: String::from_str(&env, "BitcoinDeposit"),
        user: user.clone(),
        data1: 100000000, // btc_amount
        data2: 100000000, // istsi_minted
        data3: 0,
        address1: Address::generate(&env),
        address2: Address::generate(&env),
        hash_data: tx_hash.clone(),
        text_data: String::from_str(&env, ""),
        timestamp,
        correlation_id: BytesN::from_array(&env, &[1u8; 32]),
    };
    
    let returned_correlation_id = client.emit_integration_event(&user, &event);
    
    // Get event history
    let history = client.get_event_history(&EventFilter::All, &10);
    assert_eq!(history.len(), 1);
    
    // Get history by event type
    let bitcoin_deposit_filter = EventFilter::ByEventType(String::from_str(&env, "BitcoinDeposit"));
    let filtered_history = client.get_event_history(&bitcoin_deposit_filter, &10);
    assert_eq!(filtered_history.len(), 1);
    
    // Get history by correlation ID
    let correlation_filter = EventFilter::ByCorrelationId(returned_correlation_id);
    let correlation_history = client.get_event_history(&correlation_filter, &10);
    assert_eq!(correlation_history.len(), 1);
}



#[test]
fn test_multiple_event_types() {
    let (env, client, admin, kyc_registry, istsi_token, fungible_token, reserve_manager, user) = setup_test_env();
    
    // Initialize
    client.initialize(&admin, &kyc_registry, &istsi_token, &fungible_token, &reserve_manager);
    
    // Set user as operator
    client.set_user_role(&admin, &user, &UserRole::Operator);
    
    let timestamp = env.ledger().timestamp();
    let correlation_id = BytesN::from_array(&env, &[1u8; 32]);
    
    // Emit different types of events
    let bitcoin_event = IntegrationEvent {
        event_type: String::from_str(&env, "BitcoinDeposit"),
        user: user.clone(),
        data1: 100000000, // btc_amount
        data2: 100000000, // istsi_minted
        data3: 0,
        address1: Address::generate(&env),
        address2: Address::generate(&env),
        hash_data: BytesN::from_array(&env, &[2u8; 32]),
        text_data: String::from_str(&env, ""),
        timestamp,
        correlation_id: correlation_id.clone(),
    };
    
    let compliance_event = IntegrationEvent {
        event_type: String::from_str(&env, "ComplianceAction"),
        user: user.clone(),
        data1: 0,
        data2: 0,
        data3: 0,
        address1: Address::generate(&env),
        address2: Address::generate(&env),
        hash_data: BytesN::from_array(&env, &[0u8; 32]),
        text_data: String::from_str(&env, "KYC_VERIFIED"),
        timestamp,
        correlation_id: correlation_id.clone(),
    };
    
    let reserve_event = IntegrationEvent {
        event_type: String::from_str(&env, "ReserveUpdate"),
        user: Address::generate(&env), // System user for reserve updates
        data1: 1000000000, // total_btc
        data2: 1000000000, // total_istsi
        data3: 10000,      // reserve_ratio (100%)
        address1: Address::generate(&env),
        address2: Address::generate(&env),
        hash_data: BytesN::from_array(&env, &[0u8; 32]),
        text_data: String::from_str(&env, ""),
        timestamp,
        correlation_id: correlation_id.clone(),
    };
    
    // Emit all events
    client.emit_integration_event(&user, &bitcoin_event);
    client.emit_integration_event(&user, &compliance_event);
    client.emit_integration_event(&user, &reserve_event);
    
    // Get all events
    let all_events = client.get_event_history(&EventFilter::All, &10);
    assert_eq!(all_events.len(), 3);
    
    // Get events by type
    let bitcoin_events = client.get_event_history(&EventFilter::ByEventType(String::from_str(&env, "BitcoinDeposit")), &10);
    assert_eq!(bitcoin_events.len(), 1);
    
    let compliance_events = client.get_event_history(&EventFilter::ByEventType(String::from_str(&env, "ComplianceAction")), &10);
    assert_eq!(compliance_events.len(), 1);
    
    let reserve_events = client.get_event_history(&EventFilter::ByEventType(String::from_str(&env, "ReserveUpdate")), &10);
    assert_eq!(reserve_events.len(), 1);
}

// Helper to create client for testing
use soroban_sdk::contractclient;

#[contractclient(name = "IntegrationRouterClient")]
pub trait IntegrationRouterTrait {
    fn initialize(
        env: Env,
        admin: Address,
        kyc_registry: Address,
        istsi_token: Address,
        fungible_token: Address,
        reserve_manager: Address
    );
    
    fn set_user_role(env: Env, caller: Address, user: Address, role: UserRole);
    fn remove_user_role(env: Env, caller: Address, user: Address);
    fn emergency_pause(env: Env, caller: Address, reason: String);
    fn resume_operations(env: Env, caller: Address);
    fn update_contract_address(env: Env, caller: Address, contract_name: String, new_address: Address);
    
    fn get_config(env: Env) -> RouterConfig;
    fn get_user_role(env: Env, user: Address) -> UserRole;
    fn is_paused(env: Env) -> bool;
    fn get_contract_address(env: Env, contract_name: String) -> Option<Address>;
    fn get_operators(env: Env) -> Vec<Address>;
    
    // Event system functions
    fn emit_integration_event(env: Env, caller: Address, event: IntegrationEvent) -> BytesN<32>;
    fn subscribe_to_events(env: Env, subscriber: Address, filter: EventFilter);
    fn unsubscribe_from_events(env: Env, subscriber: Address);
    fn get_event_history(env: Env, filter: EventFilter, limit: u32) -> Vec<IntegrationEvent>;
    fn get_event_subscriptions(env: Env, caller: Address) -> Vec<EventSubscription>;
}
//

// Cross-Contract Communication Layer Tests
//

#[test]
fn test_cross_contract_config_initialization() {
    let (env, client, admin, kyc_registry, istsi_token, fungible_token, reserve_manager, _user) = setup_test_env();
    
    // Initialize the contract
    client.initialize(&admin, &kyc_registry, &istsi_token, &fungible_token, &reserve_manager);
    
    // Initialize cross-contract configuration
    let config = CrossContractConfig {
        max_batch_size: 20,
        default_timeout: 600,
        max_retry_count: 5,
        enable_rollbacks: true,
        enable_timeouts: true,
    };
    
    client.initialize_cross_contract_config(&admin, &config);
    
    // Verify configuration
    let stored_config = client.get_cross_contract_config();
    assert_eq!(stored_config.max_batch_size, 20);
    assert_eq!(stored_config.default_timeout, 600);
    assert_eq!(stored_config.max_retry_count, 5);
    assert_eq!(stored_config.enable_rollbacks, true);
    assert_eq!(stored_config.enable_timeouts, true);
}

#[test]
fn test_single_contract_call() {
    let (env, client, admin, kyc_registry, istsi_token, fungible_token, reserve_manager, user) = setup_test_env();
    
    // Initialize the contract
    client.initialize(&admin, &kyc_registry, &istsi_token, &fungible_token, &reserve_manager);
    
    // Initialize cross-contract configuration
    let config = CrossContractConfig {
        max_batch_size: 10,
        default_timeout: 300,
        max_retry_count: 3,
        enable_rollbacks: true,
        enable_timeouts: true,
    };
    client.initialize_cross_contract_config(&admin, &config);
    
    // Create a contract call
    let mut parameters = Vec::new(&env);
    parameters.push_back(String::from_str(&env, "param1"));
    parameters.push_back(String::from_str(&env, "param2"));
    
    let call = ContractCall {
        target_contract: istsi_token.clone(),
        function_name: String::from_str(&env, "mint"),
        parameters,
        expected_return_type: String::from_str(&env, "bool"),
        timeout: 60,
        retry_count: 1,
    };
    
    // Execute the call
    let result = client.execute_contract_call(&admin, &call);
    
    // Verify result
    assert_eq!(result.success, true);
    assert_eq!(result.return_data, String::from_str(&env, "success"));
    assert_eq!(result.error_message, String::from_str(&env, ""));
    assert!(result.execution_time >= 0);
    assert!(result.gas_used > 0);
}

#[test]
fn test_batch_operation_success() {
    let (env, client, admin, kyc_registry, istsi_token, fungible_token, reserve_manager, _user) = setup_test_env();
    
    // Initialize the contract
    client.initialize(&admin, &kyc_registry, &istsi_token, &fungible_token, &reserve_manager);
    
    // Initialize cross-contract configuration
    let config = CrossContractConfig {
        max_batch_size: 10,
        default_timeout: 300,
        max_retry_count: 3,
        enable_rollbacks: true,
        enable_timeouts: true,
    };
    client.initialize_cross_contract_config(&admin, &config);
    
    // Create multiple contract calls
    let mut calls = Vec::new(&env);
    
    // Call 1: Mint tokens
    let mut params1 = Vec::new(&env);
    params1.push_back(String::from_str(&env, "user"));
    params1.push_back(String::from_str(&env, "1000"));
    
    let call1 = ContractCall {
        target_contract: istsi_token.clone(),
        function_name: String::from_str(&env, "mint"),
        parameters: params1,
        expected_return_type: String::from_str(&env, "bool"),
        timeout: 60,
        retry_count: 1,
    };
    calls.push_back(call1);
    
    // Call 2: Verify KYC
    let mut params2 = Vec::new(&env);
    params2.push_back(String::from_str(&env, "user"));
    
    let call2 = ContractCall {
        target_contract: kyc_registry.clone(),
        function_name: String::from_str(&env, "verify_kyc"),
        parameters: params2,
        expected_return_type: String::from_str(&env, "bool"),
        timeout: 60,
        retry_count: 1,
    };
    calls.push_back(call2);
    
    // Create rollback calls
    let rollback_calls = Vec::new(&env);
    
    // Create batch operation
    let operation_id = client.create_batch_operation(&admin, &calls, &rollback_calls, &300, &true);
    
    // Get the batch operation
    let batch = client.get_batch_operation(&operation_id);
    assert!(batch.is_some());
    let batch = batch.unwrap();
    assert_eq!(batch.calls.len(), 2);
    assert_eq!(batch.atomic, true);
    
    // Execute the batch
    let result = client.execute_batch_operation(&admin, &batch);
    
    // Verify result
    assert_eq!(result.overall_success, true);
    assert_eq!(result.call_results.len(), 2);
    assert_eq!(result.rollback_executed, false);
    assert!(result.total_execution_time >= 0);
    
    // Verify individual call results
    assert_eq!(result.call_results.get(0).unwrap().success, true);
    assert_eq!(result.call_results.get(1).unwrap().success, true);
}

#[test]
fn test_batch_operation_with_failure_and_rollback() {
    let (env, client, admin, kyc_registry, istsi_token, fungible_token, reserve_manager, _user) = setup_test_env();
    
    // Initialize the contract
    client.initialize(&admin, &kyc_registry, &istsi_token, &fungible_token, &reserve_manager);
    
    // Initialize cross-contract configuration
    let config = CrossContractConfig {
        max_batch_size: 10,
        default_timeout: 300,
        max_retry_count: 3,
        enable_rollbacks: true,
        enable_timeouts: true,
    };
    client.initialize_cross_contract_config(&admin, &config);
    
    // Create calls with one that will fail
    let mut calls = Vec::new(&env);
    
    // Call 1: Successful mint
    let mut params1 = Vec::new(&env);
    params1.push_back(String::from_str(&env, "user"));
    
    let call1 = ContractCall {
        target_contract: istsi_token.clone(),
        function_name: String::from_str(&env, "mint"),
        parameters: params1,
        expected_return_type: String::from_str(&env, "bool"),
        timeout: 60,
        retry_count: 1,
    };
    calls.push_back(call1);
    
    // Call 2: Failing call
    let params2 = Vec::new(&env);
    let call2 = ContractCall {
        target_contract: kyc_registry.clone(),
        function_name: String::from_str(&env, "fail_test"),
        parameters: params2,
        expected_return_type: String::from_str(&env, "bool"),
        timeout: 60,
        retry_count: 1,
    };
    calls.push_back(call2);
    
    // Create rollback calls
    let mut rollback_calls = Vec::new(&env);
    let rollback_params = Vec::new(&env);
    let rollback_call = ContractCall {
        target_contract: istsi_token.clone(),
        function_name: String::from_str(&env, "burn"),
        parameters: rollback_params,
        expected_return_type: String::from_str(&env, "bool"),
        timeout: 60,
        retry_count: 1,
    };
    rollback_calls.push_back(rollback_call);
    
    // Create atomic batch operation
    let operation_id = client.create_batch_operation(&admin, &calls, &rollback_calls, &300, &true);
    let batch = client.get_batch_operation(&operation_id).unwrap();
    
    // Execute the batch
    let result = client.execute_batch_operation(&admin, &batch);
    
    // Verify result
    assert_eq!(result.overall_success, false);
    assert_eq!(result.call_results.len(), 2);
    assert_eq!(result.rollback_executed, true);
    
    // Verify first call succeeded, second failed
    assert_eq!(result.call_results.get(0).unwrap().success, true);
    assert_eq!(result.call_results.get(1).unwrap().success, false);
}

#[test]
fn test_operation_status_tracking() {
    let (env, client, admin, kyc_registry, istsi_token, fungible_token, reserve_manager, _user) = setup_test_env();
    
    // Initialize the contract
    client.initialize(&admin, &kyc_registry, &istsi_token, &fungible_token, &reserve_manager);
    
    // Initialize cross-contract configuration
    let config = CrossContractConfig {
        max_batch_size: 10,
        default_timeout: 300,
        max_retry_count: 3,
        enable_rollbacks: true,
        enable_timeouts: true,
    };
    client.initialize_cross_contract_config(&admin, &config);
    
    // Create a simple batch operation
    let mut calls = Vec::new(&env);
    let params = Vec::new(&env);
    let call = ContractCall {
        target_contract: istsi_token.clone(),
        function_name: String::from_str(&env, "mint"),
        parameters: params,
        expected_return_type: String::from_str(&env, "bool"),
        timeout: 60,
        retry_count: 1,
    };
    calls.push_back(call);
    
    let rollback_calls = Vec::new(&env);
    
    // Create batch operation
    let operation_id = client.create_batch_operation(&admin, &calls, &rollback_calls, &300, &false);
    
    // Check initial status
    let status = client.get_operation_status(&operation_id);
    assert!(status.is_some());
    let status = status.unwrap();
    assert_eq!(status.status, OperationStatus::Pending);
    assert_eq!(status.operation_type, String::from_str(&env, "batch_operation"));
    assert_eq!(status.retry_count, 0);
    
    // Verify operation is in pending list
    let pending_ops = client.get_pending_operations();
    assert!(pending_ops.contains(&operation_id));
    
    // Execute the operation
    let batch = client.get_batch_operation(&operation_id).unwrap();
    let _result = client.execute_batch_operation(&admin, &batch);
    
    // Verify operation moved to completed list
    let completed_ops = client.get_completed_operations();
    assert!(completed_ops.contains(&operation_id));
    
    let pending_ops = client.get_pending_operations();
    assert!(!pending_ops.contains(&operation_id));
}

#[test]
fn test_operation_cancellation() {
    let (env, client, admin, kyc_registry, istsi_token, fungible_token, reserve_manager, _user) = setup_test_env();
    
    // Initialize the contract
    client.initialize(&admin, &kyc_registry, &istsi_token, &fungible_token, &reserve_manager);
    
    // Initialize cross-contract configuration
    let config = CrossContractConfig {
        max_batch_size: 10,
        default_timeout: 300,
        max_retry_count: 3,
        enable_rollbacks: true,
        enable_timeouts: true,
    };
    client.initialize_cross_contract_config(&admin, &config);
    
    // Create a batch operation
    let calls = Vec::new(&env);
    let rollback_calls = Vec::new(&env);
    let operation_id = client.create_batch_operation(&admin, &calls, &rollback_calls, &300, &false);
    
    // Verify operation is pending
    let status = client.get_operation_status(&operation_id).unwrap();
    assert_eq!(status.status, OperationStatus::Pending);
    
    // Cancel the operation
    let cancelled = client.cancel_operation(&admin, &operation_id);
    assert_eq!(cancelled, true);
    
    // Verify operation status changed
    let status = client.get_operation_status(&operation_id).unwrap();
    assert_eq!(status.status, OperationStatus::Failed);
    assert_eq!(status.error_message, String::from_str(&env, "Cancelled by user"));
    
    // Verify operation moved to failed list
    let failed_ops = client.get_failed_operations();
    assert!(failed_ops.contains(&operation_id));
}

#[test]
fn test_cross_contract_config_update() {
    let (env, client, admin, kyc_registry, istsi_token, fungible_token, reserve_manager, _user) = setup_test_env();
    
    // Initialize the contract
    client.initialize(&admin, &kyc_registry, &istsi_token, &fungible_token, &reserve_manager);
    
    // Initialize cross-contract configuration
    let initial_config = CrossContractConfig {
        max_batch_size: 10,
        default_timeout: 300,
        max_retry_count: 3,
        enable_rollbacks: true,
        enable_timeouts: true,
    };
    client.initialize_cross_contract_config(&admin, &initial_config);
    
    // Update configuration
    let updated_config = CrossContractConfig {
        max_batch_size: 20,
        default_timeout: 600,
        max_retry_count: 5,
        enable_rollbacks: false,
        enable_timeouts: false,
    };
    client.update_cross_contract_config(&admin, &updated_config);
    
    // Verify configuration was updated
    let stored_config = client.get_cross_contract_config();
    assert_eq!(stored_config.max_batch_size, 20);
    assert_eq!(stored_config.default_timeout, 600);
    assert_eq!(stored_config.max_retry_count, 5);
    assert_eq!(stored_config.enable_rollbacks, false);
    assert_eq!(stored_config.enable_timeouts, false);
}

#[test]
fn test_cleanup_completed_operations() {
    let (env, client, admin, kyc_registry, istsi_token, fungible_token, reserve_manager, _user) = setup_test_env();
    
    // Initialize the contract
    client.initialize(&admin, &kyc_registry, &istsi_token, &fungible_token, &reserve_manager);
    
    // Initialize cross-contract configuration
    let config = CrossContractConfig {
        max_batch_size: 10,
        default_timeout: 300,
        max_retry_count: 3,
        enable_rollbacks: true,
        enable_timeouts: true,
    };
    client.initialize_cross_contract_config(&admin, &config);
    
    // Create and execute a batch operation
    let calls = Vec::new(&env);
    let rollback_calls = Vec::new(&env);
    let operation_id = client.create_batch_operation(&admin, &calls, &rollback_calls, &300, &false);
    
    let batch = client.get_batch_operation(&operation_id).unwrap();
    let _result = client.execute_batch_operation(&admin, &batch);
    
    // Verify operation is in completed list
    let completed_ops = client.get_completed_operations();
    assert!(completed_ops.contains(&operation_id));
    
    // Cleanup operations older than current time + 1 (should clean up our operation)
    let cleanup_time = env.ledger().timestamp() + 1;
    let cleaned_count = client.cleanup_completed_operations(&admin, &cleanup_time);
    
    assert_eq!(cleaned_count, 1);
    
    // Verify operation was removed
    let completed_ops = client.get_completed_operations();
    assert!(!completed_ops.contains(&operation_id));
    
    // Verify operation data was removed
    let status = client.get_operation_status(&operation_id);
    assert!(status.is_none());
    
    let batch = client.get_batch_operation(&operation_id);
    assert!(batch.is_none());
}

#[test]
fn test_unauthorized_cross_contract_operations() {
    let (env, client, admin, kyc_registry, istsi_token, fungible_token, reserve_manager, user) = setup_test_env();
    
    // Initialize the contract
    client.initialize(&admin, &kyc_registry, &istsi_token, &fungible_token, &reserve_manager);
    
    // Try to initialize cross-contract config as non-admin (should fail)
    let config = CrossContractConfig {
        max_batch_size: 10,
        default_timeout: 300,
        max_retry_count: 3,
        enable_rollbacks: true,
        enable_timeouts: true,
    };
    
    // This should panic due to insufficient permissions
    let result = std::panic::catch_unwind(|| {
        client.initialize_cross_contract_config(&user, &config);
    });
    assert!(result.is_err());
}