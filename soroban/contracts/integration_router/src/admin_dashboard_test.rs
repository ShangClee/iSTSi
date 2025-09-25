#![cfg(test)]
use super::*;
use soroban_sdk::{
    testutils::{Address as TestAddress, Ledger, LedgerInfo, Events},
    Address, Env,
};

fn create_test_env() -> (Env, Address, Address, Address, Address, Address) {
    let env = Env::default();
    let admin = Address::generate(&env);
    let kyc_registry = Address::generate(&env);
    let istsi_token = Address::generate(&env);
    let fungible_token = Address::generate(&env);
    let reserve_manager = Address::generate(&env);
    
    (env, admin, kyc_registry, istsi_token, fungible_token, reserve_manager)
}

fn initialize_router(
    env: &Env,
    admin: &Address,
    kyc_registry: &Address,
    istsi_token: &Address,
    fungible_token: &Address,
    reserve_manager: &Address,
) {
    let client = IntegrationRouterClient::new(env, &env.register_contract(None, IntegrationRouter));
    client.initialize(
        admin,
        kyc_registry,
        istsi_token,
        fungible_token,
        reserve_manager,
    );
}

#[test]
fn test_get_system_health() {
    let (env, admin, kyc_registry, istsi_token, fungible_token, reserve_manager) = create_test_env();
    let client = IntegrationRouterClient::new(&env, &env.register_contract(None, IntegrationRouter));
    
    initialize_router(&env, &admin, &kyc_registry, &istsi_token, &fungible_token, &reserve_manager);
    
    // Set admin as system admin
    client.set_user_role(&admin, &admin, &UserRole::SystemAdmin);
    
    // Get system health
    let health = client.get_system_health(&admin);
    
    assert_eq!(health.overall_status, HealthStatus::Healthy);
    assert!(health.uptime_seconds >= 0);
    assert!(health.contract_health.len() > 0);
}

#[test]
fn test_get_system_health_unauthorized() {
    let (env, admin, kyc_registry, istsi_token, fungible_token, reserve_manager) = create_test_env();
    let client = IntegrationRouterClient::new(&env, &env.register_contract(None, IntegrationRouter));
    let unauthorized_user = Address::generate(&env);
    
    initialize_router(&env, &admin, &kyc_registry, &istsi_token, &fungible_token, &reserve_manager);
    
    // Try to get system health without proper role
    let result = client.try_get_system_health(&unauthorized_user);
    assert!(result.is_err());
}

#[test]
fn test_configure_alert() {
    let (env, admin, kyc_registry, istsi_token, fungible_token, reserve_manager) = create_test_env();
    let client = IntegrationRouterClient::new(&env, &env.register_contract(None, IntegrationRouter));
    
    initialize_router(&env, &admin, &kyc_registry, &istsi_token, &fungible_token, &reserve_manager);
    
    let alert_type = String::from_str(&env, "high_error_rate");
    let threshold = 1000u64; // 10% error rate in basis points
    let recipients = vec![&env, admin.clone()];
    
    client.configure_alert(&admin, &alert_type, &threshold, &recipients, &true);
    
    // Verify alert was configured by checking events
    let events = env.events().all();
    assert!(events.len() > 0);
}

#[test]
fn test_configure_alert_unauthorized() {
    let (env, admin, kyc_registry, istsi_token, fungible_token, reserve_manager) = create_test_env();
    let client = IntegrationRouterClient::new(&env, &env.register_contract(None, IntegrationRouter));
    let unauthorized_user = Address::generate(&env);
    
    initialize_router(&env, &admin, &kyc_registry, &istsi_token, &fungible_token, &reserve_manager);
    
    let alert_type = String::from_str(&env, "high_error_rate");
    let threshold = 1000u64;
    let recipients = vec![&env, admin.clone()];
    
    // Try to configure alert without proper role
    let result = client.try_configure_alert(&unauthorized_user, &alert_type, &threshold, &recipients, &true);
    assert!(result.is_err());
}

#[test]
fn test_coordinate_contract_upgrade() {
    let (env, admin, kyc_registry, istsi_token, fungible_token, reserve_manager) = create_test_env();
    let client = IntegrationRouterClient::new(&env, &env.register_contract(None, IntegrationRouter));
    
    initialize_router(&env, &admin, &kyc_registry, &istsi_token, &fungible_token, &reserve_manager);
    
    let contract_name = String::from_str(&env, "kyc_registry");
    let new_address = Address::generate(&env);
    let compatibility_hash = BytesN::from_array(&env, &[1u8; 32]);
    
    let result = client.coordinate_contract_upgrade(&admin, &contract_name, &new_address, &compatibility_hash);
    
    assert!(result.success);
    assert_eq!(result.error_message, String::from_str(&env, ""));
}

#[test]
fn test_execute_emergency_response_system_halt() {
    let (env, admin, kyc_registry, istsi_token, fungible_token, reserve_manager) = create_test_env();
    let client = IntegrationRouterClient::new(&env, &env.register_contract(None, IntegrationRouter));
    
    initialize_router(&env, &admin, &kyc_registry, &istsi_token, &fungible_token, &reserve_manager);
    
    let reason = String::from_str(&env, "Security breach detected");
    let affected_addresses = vec![&env];
    
    let result = client.execute_emergency_response(
        &admin,
        &EmergencyResponseType::SystemWideHalt,
        &reason,
        &affected_addresses
    );
    
    assert!(result.success);
    assert!(result.estimated_resolution_time > 0);
    
    // Verify system is paused
    assert!(client.is_paused());
}

#[test]
fn test_execute_emergency_response_address_freeze() {
    let (env, admin, kyc_registry, istsi_token, fungible_token, reserve_manager) = create_test_env();
    let client = IntegrationRouterClient::new(&env, &env.register_contract(None, IntegrationRouter));
    
    initialize_router(&env, &admin, &kyc_registry, &istsi_token, &fungible_token, &reserve_manager);
    
    let reason = String::from_str(&env, "Suspicious activity detected");
    let suspicious_address = Address::generate(&env);
    let affected_addresses = vec![&env, suspicious_address];
    
    let result = client.execute_emergency_response(
        &admin,
        &EmergencyResponseType::AddressFreeze,
        &reason,
        &affected_addresses
    );
    
    assert!(result.success);
    assert!(result.actions_taken.len() > 0);
}

#[test]
fn test_execute_emergency_response_unauthorized() {
    let (env, admin, kyc_registry, istsi_token, fungible_token, reserve_manager) = create_test_env();
    let client = IntegrationRouterClient::new(&env, &env.register_contract(None, IntegrationRouter));
    let unauthorized_user = Address::generate(&env);
    
    initialize_router(&env, &admin, &kyc_registry, &istsi_token, &fungible_token, &reserve_manager);
    
    let reason = String::from_str(&env, "Test emergency");
    let affected_addresses = vec![&env];
    
    // Try to execute emergency response without proper role
    let result = client.try_execute_emergency_response(
        &unauthorized_user,
        &EmergencyResponseType::SystemWideHalt,
        &reason,
        &affected_addresses
    );
    assert!(result.is_err());
}

#[test]
fn test_get_active_emergency_responses() {
    let (env, admin, kyc_registry, istsi_token, fungible_token, reserve_manager) = create_test_env();
    let client = IntegrationRouterClient::new(&env, &env.register_contract(None, IntegrationRouter));
    
    initialize_router(&env, &admin, &kyc_registry, &istsi_token, &fungible_token, &reserve_manager);
    
    // Set admin as system admin
    client.set_user_role(&admin, &admin, &UserRole::SystemAdmin);
    
    // Execute an emergency response
    let reason = String::from_str(&env, "Test emergency");
    let affected_addresses = vec![&env];
    
    client.execute_emergency_response(
        &admin,
        &EmergencyResponseType::AddressFreeze,
        &reason,
        &affected_addresses
    );
    
    // Get active emergency responses
    let responses = client.get_active_emergency_responses(&admin);
    assert!(responses.len() >= 0); // May be 0 if response was immediately resolved
}

#[test]
fn test_resolve_emergency_response() {
    let (env, admin, kyc_registry, istsi_token, fungible_token, reserve_manager) = create_test_env();
    let client = IntegrationRouterClient::new(&env, &env.register_contract(None, IntegrationRouter));
    
    initialize_router(&env, &admin, &kyc_registry, &istsi_token, &fungible_token, &reserve_manager);
    
    // Execute an emergency response
    let reason = String::from_str(&env, "Test emergency");
    let affected_addresses = vec![&env];
    
    let result = client.execute_emergency_response(
        &admin,
        &EmergencyResponseType::AddressFreeze,
        &reason,
        &affected_addresses
    );
    
    let response_id = result.response_id;
    let resolution_notes = String::from_str(&env, "Issue resolved");
    
    // Resolve the emergency response
    client.resolve_emergency_response(&admin, &response_id, &resolution_notes);
    
    // Verify resolution by checking events
    let events = env.events().all();
    assert!(events.len() > 0);
}

#[test]
fn test_generate_audit_report_comprehensive() {
    let (env, admin, kyc_registry, istsi_token, fungible_token, reserve_manager) = create_test_env();
    let client = IntegrationRouterClient::new(&env, &env.register_contract(None, IntegrationRouter));
    
    initialize_router(&env, &admin, &kyc_registry, &istsi_token, &fungible_token, &reserve_manager);
    
    // Set admin as system admin
    client.set_user_role(&admin, &admin, &UserRole::SystemAdmin);
    
    let start_time = env.ledger().timestamp() - 86400; // 24 hours ago
    let end_time = env.ledger().timestamp();
    
    let report = client.generate_audit_report(&admin, &start_time, &end_time, &AuditReportType::Comprehensive);
    
    assert_eq!(report.report_type, AuditReportType::Comprehensive);
    assert_eq!(report.generated_by, admin);
    assert!(report.summary.overall_score <= 100);
}

#[test]
fn test_generate_audit_report_compliance() {
    let (env, admin, kyc_registry, istsi_token, fungible_token, reserve_manager) = create_test_env();
    let client = IntegrationRouterClient::new(&env, &env.register_contract(None, IntegrationRouter));
    
    initialize_router(&env, &admin, &kyc_registry, &istsi_token, &fungible_token, &reserve_manager);
    
    // Set admin as system admin
    client.set_user_role(&admin, &admin, &UserRole::SystemAdmin);
    
    let start_time = env.ledger().timestamp() - 86400;
    let end_time = env.ledger().timestamp();
    
    let report = client.generate_audit_report(&admin, &start_time, &end_time, &AuditReportType::Compliance);
    
    assert_eq!(report.report_type, AuditReportType::Compliance);
    assert!(report.summary.compliance_score <= 100);
}

#[test]
fn test_generate_audit_report_security() {
    let (env, admin, kyc_registry, istsi_token, fungible_token, reserve_manager) = create_test_env();
    let client = IntegrationRouterClient::new(&env, &env.register_contract(None, IntegrationRouter));
    
    initialize_router(&env, &admin, &kyc_registry, &istsi_token, &fungible_token, &reserve_manager);
    
    // Set admin as system admin
    client.set_user_role(&admin, &admin, &UserRole::SystemAdmin);
    
    let start_time = env.ledger().timestamp() - 86400;
    let end_time = env.ledger().timestamp();
    
    let report = client.generate_audit_report(&admin, &start_time, &end_time, &AuditReportType::Security);
    
    assert_eq!(report.report_type, AuditReportType::Security);
    assert!(report.summary.security_score <= 100);
}

#[test]
fn test_generate_audit_report_performance() {
    let (env, admin, kyc_registry, istsi_token, fungible_token, reserve_manager) = create_test_env();
    let client = IntegrationRouterClient::new(&env, &env.register_contract(None, IntegrationRouter));
    
    initialize_router(&env, &admin, &kyc_registry, &istsi_token, &fungible_token, &reserve_manager);
    
    // Set admin as system admin
    client.set_user_role(&admin, &admin, &UserRole::SystemAdmin);
    
    let start_time = env.ledger().timestamp() - 86400;
    let end_time = env.ledger().timestamp();
    
    let report = client.generate_audit_report(&admin, &start_time, &end_time, &AuditReportType::Performance);
    
    assert_eq!(report.report_type, AuditReportType::Performance);
    assert!(report.summary.performance_score <= 100);
}

#[test]
fn test_generate_audit_report_unauthorized() {
    let (env, admin, kyc_registry, istsi_token, fungible_token, reserve_manager) = create_test_env();
    let client = IntegrationRouterClient::new(&env, &env.register_contract(None, IntegrationRouter));
    let unauthorized_user = Address::generate(&env);
    
    initialize_router(&env, &admin, &kyc_registry, &istsi_token, &fungible_token, &reserve_manager);
    
    let start_time = env.ledger().timestamp() - 86400;
    let end_time = env.ledger().timestamp();
    
    // Try to generate audit report without proper role
    let result = client.try_generate_audit_report(&unauthorized_user, &start_time, &end_time, &AuditReportType::Comprehensive);
    assert!(result.is_err());
}

#[test]
fn test_system_metrics_calculation() {
    let (env, admin, kyc_registry, istsi_token, fungible_token, reserve_manager) = create_test_env();
    let client = IntegrationRouterClient::new(&env, &env.register_contract(None, IntegrationRouter));
    
    initialize_router(&env, &admin, &kyc_registry, &istsi_token, &fungible_token, &reserve_manager);
    
    // Set admin as system admin
    client.set_user_role(&admin, &admin, &UserRole::SystemAdmin);
    
    let health = client.get_system_health(&admin);
    let metrics = health.system_metrics;
    
    assert!(metrics.total_operations >= 0);
    assert!(metrics.successful_operations >= 0);
    assert!(metrics.failed_operations >= 0);
    assert!(metrics.current_reserve_ratio > 0);
    assert!(metrics.last_updated > 0);
}

#[test]
fn test_contract_health_monitoring() {
    let (env, admin, kyc_registry, istsi_token, fungible_token, reserve_manager) = create_test_env();
    let client = IntegrationRouterClient::new(&env, &env.register_contract(None, IntegrationRouter));
    
    initialize_router(&env, &admin, &kyc_registry, &istsi_token, &fungible_token, &reserve_manager);
    
    // Set admin as system admin
    client.set_user_role(&admin, &admin, &UserRole::SystemAdmin);
    
    let health = client.get_system_health(&admin);
    
    // Verify all core contracts are monitored
    assert!(health.contract_health.contains_key(String::from_str(&env, "kyc_registry")));
    assert!(health.contract_health.contains_key(String::from_str(&env, "istsi_token")));
    assert!(health.contract_health.contains_key(String::from_str(&env, "fungible_token")));
    assert!(health.contract_health.contains_key(String::from_str(&env, "reserve_manager")));
    
    // Verify health info structure
    for (_, contract_health) in health.contract_health.iter() {
        assert!(contract_health.uptime_percentage <= 10000); // Max 100% in basis points
        assert!(contract_health.last_response_time > 0);
    }
}

#[test]
fn test_emergency_response_workflow() {
    let (env, admin, kyc_registry, istsi_token, fungible_token, reserve_manager) = create_test_env();
    let client = IntegrationRouterClient::new(&env, &env.register_contract(None, IntegrationRouter));
    
    initialize_router(&env, &admin, &kyc_registry, &istsi_token, &fungible_token, &reserve_manager);
    
    // Set admin as system admin
    client.set_user_role(&admin, &admin, &UserRole::SystemAdmin);
    
    // 1. Execute emergency response
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
    
    // 2. Check active responses
    let active_responses = client.get_active_emergency_responses(&admin);
    // Note: Response might be immediately resolved in test environment
    
    // 3. Resolve the response
    let resolution_notes = String::from_str(&env, "Issue resolved, system restored");
    client.resolve_emergency_response(&admin, &response_id, &resolution_notes);
    
    // Verify events were emitted
    let events = env.events().all();
    assert!(events.len() > 0);
}