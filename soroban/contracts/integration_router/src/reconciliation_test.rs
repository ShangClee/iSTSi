#[cfg(test)]
mod reconciliation_tests {
    use super::*;
    use soroban_sdk::{testutils::Address as AddressTestUtils, Address, Env};

    fn setup_test_environment() -> (Env, Address, Address, Address, Address, Address, IntegrationRouterClient<'static>) {
        let env = Env::default();
        env.mock_all_auths();
        
        let contract_id = env.register(IntegrationRouter, ());
        let client = IntegrationRouterClient::new(&env, &contract_id);
        
        let admin = Address::generate(&env);
        let kyc_registry = Address::generate(&env);
        let istsi_token = Address::generate(&env);
        let fungible_token = Address::generate(&env);
        let reserve_manager = Address::generate(&env);
        
        client.initialize(
            &admin,
            &kyc_registry,
            &istsi_token,
            &fungible_token,
            &reserve_manager
        );
        
        (env, admin, kyc_registry, istsi_token, fungible_token, reserve_manager, client)
    }

    #[test]
    fn test_reconciliation_config() {
        let (env, admin, _, _, _, _, client) = setup_test_environment();
        
        // Test default configuration
        let default_config = client.get_reconciliation_config();
        assert_eq!(default_config.tolerance_threshold, 100); // 1%
        assert_eq!(default_config.auto_reconcile_enabled, true);
        assert_eq!(default_config.emergency_halt_on_discrepancy, true);
        assert_eq!(default_config.reconciliation_frequency, 3600); // 1 hour
        assert_eq!(default_config.max_discrepancy_before_halt, 500); // 5%
        
        // Test custom configuration
        let custom_config = ReconciliationConfig {
            tolerance_threshold: 200,        // 2%
            auto_reconcile_enabled: false,
            emergency_halt_on_discrepancy: false,
            reconciliation_frequency: 7200, // 2 hours
            max_discrepancy_before_halt: 1000, // 10%
        };
        
        client.configure_reconciliation(&admin, &custom_config);
        
        let updated_config = client.get_reconciliation_config();
        assert_eq!(updated_config.tolerance_threshold, 200);
        assert_eq!(updated_config.auto_reconcile_enabled, false);
        assert_eq!(updated_config.emergency_halt_on_discrepancy, false);
        assert_eq!(updated_config.reconciliation_frequency, 7200);
        assert_eq!(updated_config.max_discrepancy_before_halt, 1000);
    }

    #[test]
    fn test_real_time_reserve_data() {
        let (env, admin, _, _, _, _, client) = setup_test_environment();
        
        // Test getting real-time data (will return zeros since we're not mocking the contract calls)
        let (btc_reserves, token_supply, actual_ratio) = client.get_real_time_reserve_data();
        
        // In a test environment without proper contract mocks, these should be 0
        assert_eq!(btc_reserves, 0);
        assert_eq!(token_supply, 0);
        assert_eq!(actual_ratio, 0);
    }

    #[test]
    fn test_reconciliation_execution() {
        let (env, admin, _, _, _, _, client) = setup_test_environment();
        
        // Set admin as operator to execute reconciliation
        client.set_user_role(&admin, &admin, &UserRole::Operator);
        
        // Execute reconciliation check
        let result = client.execute_reconciliation_check(&admin);
        
        // Verify reconciliation result structure
        assert_eq!(result.btc_reserves, 0);
        assert_eq!(result.token_supply, 0);
        assert_eq!(result.expected_ratio, 10000); // 1:1 ratio = 100%
        assert_eq!(result.actual_ratio, 0);
        assert_eq!(result.discrepancy, -10000); // Expected 100%, got 0%
        assert_eq!(result.status, ReconciliationStatus::DiscrepancyDetected);
        
        // Verify reconciliation can be retrieved
        let stored_result = client.get_reconciliation_result(&result.reconciliation_id);
        assert!(stored_result.is_some());
        assert_eq!(stored_result.unwrap().reconciliation_id, result.reconciliation_id);
    }

    #[test]
    fn test_reconciliation_history() {
        let (env, admin, _, _, _, _, client) = setup_test_environment();
        
        // Set admin as operator
        client.set_user_role(&admin, &admin, &UserRole::Operator);
        
        // Execute multiple reconciliations
        let result1 = client.execute_reconciliation_check(&admin);
        let result2 = client.execute_reconciliation_check(&admin);
        let result3 = client.execute_reconciliation_check(&admin);
        
        // Get reconciliation history
        let history = client.get_reconciliation_history(&10u32);
        assert_eq!(history.len(), 3);
        
        // Verify order (most recent first)
        assert_eq!(history.get(0).unwrap(), result1.reconciliation_id);
        assert_eq!(history.get(1).unwrap(), result2.reconciliation_id);
        assert_eq!(history.get(2).unwrap(), result3.reconciliation_id);
        
        // Test limited history
        let limited_history = client.get_reconciliation_history(&2u32);
        assert_eq!(limited_history.len(), 2);
    }

    #[test]
    fn test_auto_reconciliation_trigger() {
        let (env, admin, _, _, _, _, client) = setup_test_environment();
        
        // Test when auto reconciliation is disabled
        let mut config = client.get_reconciliation_config();
        config.auto_reconcile_enabled = false;
        client.configure_reconciliation(&admin, &config);
        
        let auto_result = client.trigger_auto_reconciliation();
        assert!(auto_result.is_none());
        
        // Test when auto reconciliation is enabled but not due
        config.auto_reconcile_enabled = true;
        config.reconciliation_frequency = 3600; // 1 hour
        client.configure_reconciliation(&admin, &config);
        
        // Execute a manual reconciliation first (sets last reconciliation time)
        client.set_user_role(&admin, &admin, &UserRole::Operator);
        client.execute_reconciliation_check(&admin);
        
        // Try auto reconciliation immediately (should not trigger)
        let auto_result2 = client.trigger_auto_reconciliation();
        assert!(auto_result2.is_none());
    }

    #[test]
    fn test_proof_schedule_configuration() {
        let (env, admin, _, _, _, _, client) = setup_test_environment();
        
        // Test default proof schedule
        let default_schedule = client.get_proof_schedule();
        assert_eq!(default_schedule.enabled, true);
        assert_eq!(default_schedule.frequency, 86400); // Daily
        assert_eq!(default_schedule.auto_verify, true);
        assert_eq!(default_schedule.storage_enabled, true);
        
        // Test custom proof schedule
        let custom_schedule = ProofOfReservesSchedule {
            enabled: false,
            frequency: 43200, // 12 hours
            last_generated: 0,
            next_scheduled: 0,
            auto_verify: false,
            storage_enabled: false,
        };
        
        client.configure_proof_schedule(&admin, &custom_schedule);
        
        let updated_schedule = client.get_proof_schedule();
        assert_eq!(updated_schedule.enabled, false);
        assert_eq!(updated_schedule.frequency, 43200);
        assert_eq!(updated_schedule.auto_verify, false);
        assert_eq!(updated_schedule.storage_enabled, false);
    }

    #[test]
    fn test_proof_generation() {
        let (env, admin, _, _, _, _, client) = setup_test_environment();
        
        // Set admin as operator to generate proofs
        client.set_user_role(&admin, &admin, &UserRole::Operator);
        
        // Generate automated proof
        let stored_proof = client.generate_auto_proof_of_reserves(&admin);
        
        // Verify proof structure
        assert_eq!(stored_proof.total_btc_reserves, 0);
        assert_eq!(stored_proof.total_token_supply, 0);
        assert_eq!(stored_proof.reserve_ratio, 0);
        assert_eq!(stored_proof.verification_status, ProofVerificationStatus::Verified); // Auto-verified
        assert_eq!(stored_proof.generated_by, admin);
        
        // Verify proof can be retrieved
        let retrieved_proof = client.get_stored_proof(&stored_proof.proof_id);
        assert!(retrieved_proof.is_some());
        assert_eq!(retrieved_proof.unwrap().proof_id, stored_proof.proof_id);
    }

    #[test]
    fn test_proof_verification() {
        let (env, admin, _, _, _, _, client) = setup_test_environment();
        
        // Set admin as operator
        client.set_user_role(&admin, &admin, &UserRole::Operator);
        
        // Disable auto-verification to test manual verification
        let mut schedule = client.get_proof_schedule();
        schedule.auto_verify = false;
        client.configure_proof_schedule(&admin, &schedule);
        
        // Generate proof without auto-verification
        let stored_proof = client.generate_auto_proof_of_reserves(&admin);
        assert_eq!(stored_proof.verification_status, ProofVerificationStatus::Pending);
        
        // Manually verify proof
        let verification_status = client.verify_proof_of_reserves(&admin, &stored_proof.proof_id);
        assert_eq!(verification_status, ProofVerificationStatus::Verified);
        
        // Verify the stored proof was updated
        let updated_proof = client.get_stored_proof(&stored_proof.proof_id).unwrap();
        assert_eq!(updated_proof.verification_status, ProofVerificationStatus::Verified);
    }

    #[test]
    fn test_proof_history() {
        let (env, admin, _, _, _, _, client) = setup_test_environment();
        
        // Set admin as operator
        client.set_user_role(&admin, &admin, &UserRole::Operator);
        
        // Generate multiple proofs
        let proof1 = client.generate_auto_proof_of_reserves(&admin);
        let proof2 = client.generate_auto_proof_of_reserves(&admin);
        let proof3 = client.generate_auto_proof_of_reserves(&admin);
        
        // Get proof history
        let history = client.get_proof_history(&10u32);
        assert_eq!(history.len(), 3);
        
        // Verify order
        assert_eq!(history.get(0).unwrap(), proof1.proof_id);
        assert_eq!(history.get(1).unwrap(), proof2.proof_id);
        assert_eq!(history.get(2).unwrap(), proof3.proof_id);
        
        // Test limited history
        let limited_history = client.get_proof_history(&2u32);
        assert_eq!(limited_history.len(), 2);
    }

    #[test]
    fn test_reconciliation_report_generation() {
        let (env, admin, _, _, _, _, client) = setup_test_environment();
        
        // Set admin as operator
        client.set_user_role(&admin, &admin, &UserRole::Operator);
        
        // Execute some reconciliations
        client.execute_reconciliation_check(&admin);
        client.execute_reconciliation_check(&admin);
        
        // Generate reconciliation report
        let period_start = env.ledger().timestamp() - 3600; // 1 hour ago
        let period_end = env.ledger().timestamp();
        
        let report = client.generate_reconciliation_report(&admin, &period_start, &period_end);
        
        // Verify report structure
        assert_eq!(report.period_start, period_start);
        assert_eq!(report.period_end, period_end);
        assert_eq!(report.total_reconciliations, 2);
        assert_eq!(report.discrepancies_detected, 2); // Both should detect discrepancies due to 0 reserves/supply
        assert_eq!(report.emergency_halts, 0);
        assert_eq!(report.generated_by, admin);
    }

    #[test]
    fn test_discrepancy_alert_management() {
        let (env, admin, _, _, _, _, client) = setup_test_environment();
        
        // Set admin as operator and compliance officer
        client.set_user_role(&admin, &admin, &UserRole::Operator);
        
        // Execute reconciliation that will create discrepancy alerts
        let result = client.execute_reconciliation_check(&admin);
        
        // Get active discrepancy alerts
        let active_alerts = client.get_active_discrepancy_alerts();
        assert!(active_alerts.len() > 0);
        
        // Verify alert properties
        let alert = &active_alerts.get(0).unwrap();
        assert_eq!(alert.reconciliation_id, result.reconciliation_id);
        assert_eq!(alert.acknowledged, false);
        assert!(alert.acknowledged_by.is_none());
        
        // Set admin as compliance officer to acknowledge alerts
        client.set_user_role(&admin, &admin, &UserRole::ComplianceOfficer);
        
        // Acknowledge the alert
        client.acknowledge_discrepancy_alert(&admin, &alert.alert_id);
        
        // Verify alert was acknowledged
        let updated_alerts = client.get_active_discrepancy_alerts();
        // Should be empty now since acknowledged alerts are filtered out
        assert_eq!(updated_alerts.len(), 0);
    }

    #[test]
    fn test_emergency_halt_for_discrepancy() {
        let (env, admin, _, _, _, _, client) = setup_test_environment();
        
        // Set admin as compliance officer
        client.set_user_role(&admin, &admin, &UserRole::ComplianceOfficer);
        
        // Execute reconciliation to get a reconciliation ID
        client.set_user_role(&admin, &admin, &UserRole::Operator);
        let result = client.execute_reconciliation_check(&admin);
        
        // Reset role to compliance officer
        client.set_user_role(&admin, &admin, &UserRole::ComplianceOfficer);
        
        // Trigger emergency halt
        let reason = String::from_str(&env, "Critical discrepancy detected");
        client.trigger_emrg_halt_discrepancy(&admin, &result.reconciliation_id, &reason);
        
        // Verify system is paused
        let config = client.get_config();
        assert_eq!(config.paused, true);
        
        // Verify reconciliation result was updated
        let updated_result = client.get_reconciliation_result(&result.reconciliation_id).unwrap();
        assert_eq!(updated_result.status, ReconciliationStatus::EmergencyHalt);
        assert_eq!(updated_result.protective_measures_triggered, true);
    }
}