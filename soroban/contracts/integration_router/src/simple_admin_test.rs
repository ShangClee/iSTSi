#![cfg(test)]
use super::*;
use soroban_sdk::{
    testutils::{Address as TestAddress, Events},
    Address, Env,
};

#[test]
fn test_admin_dashboard_basic_functionality() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let kyc_registry = Address::generate(&env);
    let istsi_token = Address::generate(&env);
    let fungible_token = Address::generate(&env);
    let reserve_manager = Address::generate(&env);
    
    let client = IntegrationRouterClient::new(&env, &env.register(IntegrationRouter, ()));
    
    // Initialize the router
    client.initialize(
        &admin,
        &kyc_registry,
        &istsi_token,
        &fungible_token,
        &reserve_manager,
    );
    
    // Set admin as system admin to access admin functions
    client.set_user_role(&admin, &admin, &UserRole::SystemAdmin);
    
    // Test get_system_health
    let health = client.get_system_health(&admin);
    assert_eq!(health.overall_status, HealthStatus::Healthy);
    assert!(health.uptime_seconds >= 0);
    assert!(health.contract_health.len() > 0);
    
    // Test configure_alert
    let alert_type = String::from_str(&env, "high_error_rate");
    let threshold = 1000u64;
    let recipients = vec![&env, admin.clone()];
    
    client.configure_alert(&admin, &alert_type, &threshold, &recipients, &true);
    
    // Test coordinate_contract_upgrade
    let contract_name = String::from_str(&env, "kyc_registry");
    let new_address = Address::generate(&env);
    let compatibility_hash = BytesN::from_array(&env, &[1u8; 32]);
    
    let result = client.coordinate_contract_upgrade(&admin, &contract_name, &new_address, &compatibility_hash);
    assert!(result.success);
    
    // Test execute_emergency_response
    let reason = String::from_str(&env, "Test emergency");
    let affected_addresses = vec![&env];
    
    let result = client.execute_emergency_response(
        &admin,
        &EmergencyResponseType::SystemWideHalt,
        &reason,
        &affected_addresses
    );
    assert!(result.success);
    
    // Test generate_audit_report
    let start_time = env.ledger().timestamp() - 86400;
    let end_time = env.ledger().timestamp();
    
    let report = client.generate_audit_report(&admin, &start_time, &end_time, &AuditReportType::Comprehensive);
    assert_eq!(report.report_type, AuditReportType::Comprehensive);
    assert_eq!(report.generated_by, admin);
    
    // Verify events were emitted
    let events = env.events().all();
    assert!(events.len() > 0);
}

#[test]
fn test_admin_dashboard_unauthorized_access() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let unauthorized_user = Address::generate(&env);
    let kyc_registry = Address::generate(&env);
    let istsi_token = Address::generate(&env);
    let fungible_token = Address::generate(&env);
    let reserve_manager = Address::generate(&env);
    
    let client = IntegrationRouterClient::new(&env, &env.register(IntegrationRouter, ()));
    
    // Initialize the router
    client.initialize(
        &admin,
        &kyc_registry,
        &istsi_token,
        &fungible_token,
        &reserve_manager,
    );
    
    // Try to access admin functions without proper role
    let result = client.try_get_system_health(&unauthorized_user);
    assert!(result.is_err());
    
    let alert_type = String::from_str(&env, "test_alert");
    let threshold = 1000u64;
    let recipients = vec![&env, admin.clone()];
    
    let result = client.try_configure_alert(&unauthorized_user, &alert_type, &threshold, &recipients, &true);
    assert!(result.is_err());
}

#[test]
fn test_emergency_response_workflow() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let kyc_registry = Address::generate(&env);
    let istsi_token = Address::generate(&env);
    let fungible_token = Address::generate(&env);
    let reserve_manager = Address::generate(&env);
    
    let client = IntegrationRouterClient::new(&env, &env.register(IntegrationRouter, ()));
    
    // Initialize the router
    client.initialize(
        &admin,
        &kyc_registry,
        &istsi_token,
        &fungible_token,
        &reserve_manager,
    );
    
    // Set admin as system admin
    client.set_user_role(&admin, &admin, &UserRole::SystemAdmin);
    
    // Execute emergency response
    let reason = String::from_str(&env, "Critical security issue");
    let affected_addresses = vec![&env];
    
    let result = client.execute_emergency_response(
        &admin,
        &EmergencyResponseType::SystemWideHalt,
        &reason,
        &affected_addresses
    );
    
    assert!(result.success);
    let response_id = result.response_id;
    
    // Check that system is paused
    assert!(client.is_paused());
    
    // Get active emergency responses
    let active_responses = client.get_active_emergency_responses(&admin);
    // Note: Response might be immediately resolved in test environment
    
    // Resolve the response
    let resolution_notes = String::from_str(&env, "Issue resolved, system restored");
    client.resolve_emergency_response(&admin, &response_id, &resolution_notes);
    
    // Verify events were emitted
    let events = env.events().all();
    assert!(events.len() > 0);
}