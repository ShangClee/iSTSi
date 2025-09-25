#[cfg(test)]
mod simple_reconciliation_tests {
    use super::*;
    use soroban_sdk::{testutils::Address as AddressTestUtils, Address, Env};

    fn setup_simple_test() -> (Env, Address, IntegrationRouterClient<'static>) {
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
        
        (env, admin, client)
    }

    #[test]
    fn test_basic_reconciliation_config() {
        let (env, admin, client) = setup_simple_test();
        
        // Test getting default configuration
        let config = client.get_reconciliation_config();
        assert_eq!(config.tolerance_threshold, 100);
        assert_eq!(config.auto_reconcile_enabled, true);
        
        // Test updating configuration
        let new_config = ReconciliationConfig {
            tolerance_threshold: 200,
            auto_reconcile_enabled: false,
            emergency_halt_on_discrepancy: false,
            reconciliation_frequency: 7200,
            max_discrepancy_before_halt: 1000,
        };
        
        client.configure_reconciliation(&admin, &new_config);
        
        let updated_config = client.get_reconciliation_config();
        assert_eq!(updated_config.tolerance_threshold, 200);
        assert_eq!(updated_config.auto_reconcile_enabled, false);
    }

    #[test]
    fn test_basic_reconciliation_execution() {
        let (env, admin, client) = setup_simple_test();
        
        // Set admin as operator
        client.set_user_role(&admin, &admin, &UserRole::Operator);
        
        // Execute reconciliation
        let result = client.execute_reconciliation_check(&admin);
        
        // Basic checks
        assert_eq!(result.expected_ratio, 10000);
        assert!(result.reconciliation_id != BytesN::from_array(&env, &[0u8; 32]));
        assert!(result.timestamp > 0);
    }

    #[test]
    fn test_basic_proof_generation() {
        let (env, admin, client) = setup_simple_test();
        
        // Set admin as operator
        client.set_user_role(&admin, &admin, &UserRole::Operator);
        
        // Generate proof
        let proof = client.generate_auto_proof_of_reserves(&admin);
        
        // Basic checks
        assert!(proof.proof_id != BytesN::from_array(&env, &[0u8; 32]));
        assert_eq!(proof.generated_by, admin);
        assert!(proof.timestamp > 0);
    }

    #[test]
    fn test_basic_real_time_data() {
        let (env, admin, client) = setup_simple_test();
        
        // Get real-time data
        let (reserves, supply, ratio) = client.get_real_time_reserve_data();
        
        // In test environment, these should be 0
        assert_eq!(reserves, 0);
        assert_eq!(supply, 0);
        assert_eq!(ratio, 0);
    }

    #[test]
    fn test_basic_history_functions() {
        let (env, admin, client) = setup_simple_test();
        
        // Set admin as operator
        client.set_user_role(&admin, &admin, &UserRole::Operator);
        
        // Execute reconciliation to create history
        client.execute_reconciliation_check(&admin);
        
        // Check reconciliation history
        let recon_history = client.get_reconciliation_history(&10u32);
        assert_eq!(recon_history.len(), 1);
        
        // Generate proof to create proof history
        client.generate_auto_proof_of_reserves(&admin);
        
        // Check proof history
        let proof_history = client.get_proof_history(&10u32);
        assert_eq!(proof_history.len(), 1);
    }
}