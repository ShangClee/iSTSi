#[cfg(test)]
mod config_tests {
    use super::*;
    use soroban_sdk::{
        testutils::{Address as TestAddress, Ledger, LedgerInfo},
        Address, Env, Map, String as SorobanString, BytesN,
    };

    fn create_test_env() -> Env {
        Env::default()
    }

    fn setup_test_contracts(env: &Env) -> (Address, Address, Address, Address, Address, Address) {
        let admin = Address::generate(env);
        let kyc_registry = Address::generate(env);
        let reserve_manager = Address::generate(env);
        let fungible_token = Address::generate(env);
        let istsi_token = Address::generate(env);
        let integration_router = env.register_contract(None, IntegrationRouter);

        (admin, kyc_registry, reserve_manager, fungible_token, istsi_token, integration_router)
    }

    #[test]
    fn test_system_parameter_management() {
        let env = create_test_env();
        let (admin, kyc_registry, reserve_manager, fungible_token, istsi_token, integration_router) = 
            setup_test_contracts(&env);

        let client = IntegrationRouterClient::new(&env, &integration_router);

        // Initialize
        client.initialize(
            &admin,
            &kyc_registry,
            &istsi_token,
            &fungible_token,
            &reserve_manager,
        );

        // Set system parameter
        let param_name = SorobanString::from_str(&env, "max_timeout");
        let param_value = SorobanString::from_str(&env, "300");

        client.set_system_parameter(&admin, &param_name, &param_value);

        // Get system parameter
        let retrieved_value = client.get_system_parameter(&param_name);
        assert_eq!(retrieved_value, Some(param_value));
    }

    #[test]
    fn test_contract_parameter_management() {
        let env = create_test_env();
        let (admin, kyc_registry, reserve_manager, fungible_token, istsi_token, integration_router) = 
            setup_test_contracts(&env);

        let client = IntegrationRouterClient::new(&env, &integration_router);

        // Initialize
        client.initialize(
            &admin,
            &kyc_registry,
            &istsi_token,
            &fungible_token,
            &reserve_manager,
        );

        // Set contract parameter
        let contract_name = SorobanString::from_str(&env, "kyc_registry");
        let param_name = SorobanString::from_str(&env, "max_tier");
        let param_value = SorobanString::from_str(&env, "4");

        client.set_contract_parameter(&admin, &contract_name, &param_name, &param_value);

        // Get contract parameter
        let retrieved_value = client.get_contract_parameter(&contract_name, &param_name);
        assert_eq!(retrieved_value, Some(param_value));
    }

    #[test]
    fn test_contract_limit_management() {
        let env = create_test_env();
        let (admin, kyc_registry, reserve_manager, fungible_token, istsi_token, integration_router) = 
            setup_test_contracts(&env);

        let client = IntegrationRouterClient::new(&env, &integration_router);

        // Initialize
        client.initialize(
            &admin,
            &kyc_registry,
            &istsi_token,
            &fungible_token,
            &reserve_manager,
        );

        // Set contract limit
        let contract_name = SorobanString::from_str(&env, "istsi_token");
        let limit_name = SorobanString::from_str(&env, "max_mint_per_tx");
        let limit_value = 1000000000u64;

        client.set_contract_limit(&admin, &contract_name, &limit_name, &limit_value);

        // Get contract limit
        let retrieved_limit = client.get_contract_limit(&contract_name, &limit_name);
        assert_eq!(retrieved_limit, Some(limit_value));
    }

    #[test]
    fn test_configuration_validation() {
        let env = create_test_env();
        let (admin, kyc_registry, reserve_manager, fungible_token, istsi_token, integration_router) = 
            setup_test_contracts(&env);

        let client = IntegrationRouterClient::new(&env, &integration_router);

        // Initialize
        client.initialize(
            &admin,
            &kyc_registry,
            &istsi_token,
            &fungible_token,
            &reserve_manager,
        );

        // Validate configuration (should pass with all contracts configured)
        let is_valid = client.validate_configuration(&admin);
        assert_eq!(is_valid, true);
    }

    #[test]
    fn test_configuration_summary() {
        let env = create_test_env();
        let (admin, kyc_registry, reserve_manager, fungible_token, istsi_token, integration_router) = 
            setup_test_contracts(&env);

        let client = IntegrationRouterClient::new(&env, &integration_router);

        // Initialize
        client.initialize(
            &admin,
            &kyc_registry,
            &istsi_token,
            &fungible_token,
            &reserve_manager,
        );

        // Get configuration summary
        let summary = client.get_configuration_summary(&admin);

        // Verify summary contains expected information
        assert!(summary.len() > 0);
        
        let kyc_name = SorobanString::from_str(&env, "kyc_registry");
        let istsi_name = SorobanString::from_str(&env, "istsi_token");
        let paused_key = SorobanString::from_str(&env, "paused");
        let admin_key = SorobanString::from_str(&env, "admin");

        assert!(summary.contains_key(kyc_name));
        assert!(summary.contains_key(istsi_name));
        assert!(summary.contains_key(paused_key));
        assert!(summary.contains_key(admin_key));
    }

    #[test]
    fn test_configuration_batch_update() {
        let env = create_test_env();
        let (admin, kyc_registry, reserve_manager, fungible_token, istsi_token, integration_router) = 
            setup_test_contracts(&env);

        let client = IntegrationRouterClient::new(&env, &integration_router);

        // Initialize
        client.initialize(
            &admin,
            &kyc_registry,
            &istsi_token,
            &fungible_token,
            &reserve_manager,
        );

        // Prepare batch update
        let mut parameters = Map::new(&env);
        parameters.set(
            SorobanString::from_str(&env, "timeout"),
            SorobanString::from_str(&env, "600")
        );
        parameters.set(
            SorobanString::from_str(&env, "gas_limit"),
            SorobanString::from_str(&env, "2000000")
        );

        let mut limits = Map::new(&env);
        limits.set(
            SorobanString::from_str(&env, "istsi_token.max_mint"),
            1000000000u64
        );
        limits.set(
            SorobanString::from_str(&env, "kyc_registry.max_registrations"),
            10000u64
        );

        // Apply batch update
        client.apply_configuration_batch(&admin, &parameters, &limits);

        // Verify parameters were set
        let timeout_value = client.get_system_parameter(&SorobanString::from_str(&env, "timeout"));
        assert_eq!(timeout_value, Some(SorobanString::from_str(&env, "600")));

        // Verify limits were set
        let mint_limit = client.get_contract_limit(
            &SorobanString::from_str(&env, "istsi_token"),
            &SorobanString::from_str(&env, "max_mint")
        );
        assert_eq!(mint_limit, Some(1000000000u64));
    }

    #[test]
    fn test_configuration_backup_and_restore() {
        let env = create_test_env();
        let (admin, kyc_registry, reserve_manager, fungible_token, istsi_token, integration_router) = 
            setup_test_contracts(&env);

        let client = IntegrationRouterClient::new(&env, &integration_router);

        // Initialize
        client.initialize(
            &admin,
            &kyc_registry,
            &istsi_token,
            &fungible_token,
            &reserve_manager,
        );

        // Create configuration backup
        let backup_id = client.create_configuration_backup(&admin);
        assert_ne!(backup_id, BytesN::from_array(&env, &[0u8; 32]));

        // Restore configuration backup
        let restore_success = client.restore_configuration_backup(&admin, &backup_id);
        assert_eq!(restore_success, true);
    }

    #[test]
    fn test_environment_info() {
        let env = create_test_env();
        let (admin, kyc_registry, reserve_manager, fungible_token, istsi_token, integration_router) = 
            setup_test_contracts(&env);

        let client = IntegrationRouterClient::new(&env, &integration_router);

        // Initialize
        client.initialize(
            &admin,
            &kyc_registry,
            &istsi_token,
            &fungible_token,
            &reserve_manager,
        );

        // Get environment info
        let env_info = client.get_environment_info();

        // Verify environment info contains expected fields
        assert!(env_info.len() > 0);
        
        let contract_addr_key = SorobanString::from_str(&env, "contract_address");
        let timestamp_key = SorobanString::from_str(&env, "timestamp");
        let paused_key = SorobanString::from_str(&env, "paused");

        assert!(env_info.contains_key(contract_addr_key));
        assert!(env_info.contains_key(timestamp_key));
        assert!(env_info.contains_key(paused_key));
    }

    #[test]
    fn test_unauthorized_configuration_operations() {
        let env = create_test_env();
        let (admin, kyc_registry, reserve_manager, fungible_token, istsi_token, integration_router) = 
            setup_test_contracts(&env);

        let client = IntegrationRouterClient::new(&env, &integration_router);
        let unauthorized_user = Address::generate(&env);

        // Initialize
        client.initialize(
            &admin,
            &kyc_registry,
            &istsi_token,
            &fungible_token,
            &reserve_manager,
        );

        // Test unauthorized system parameter setting
        let result = std::panic::catch_unwind(|| {
            client.set_system_parameter(
                &unauthorized_user,
                &SorobanString::from_str(&env, "timeout"),
                &SorobanString::from_str(&env, "300")
            );
        });
        assert!(result.is_err());

        // Test unauthorized contract parameter setting
        let result = std::panic::catch_unwind(|| {
            client.set_contract_parameter(
                &unauthorized_user,
                &SorobanString::from_str(&env, "kyc_registry"),
                &SorobanString::from_str(&env, "max_tier"),
                &SorobanString::from_str(&env, "4")
            );
        });
        assert!(result.is_err());

        // Test unauthorized limit setting
        let result = std::panic::catch_unwind(|| {
            client.set_contract_limit(
                &unauthorized_user,
                &SorobanString::from_str(&env, "istsi_token"),
                &SorobanString::from_str(&env, "max_mint"),
                1000000000u64
            );
        });
        assert!(result.is_err());
    }

    #[test]
    fn test_configuration_events() {
        let env = create_test_env();
        let (admin, kyc_registry, reserve_manager, fungible_token, istsi_token, integration_router) = 
            setup_test_contracts(&env);

        let client = IntegrationRouterClient::new(&env, &integration_router);

        // Initialize
        client.initialize(
            &admin,
            &kyc_registry,
            &istsi_token,
            &fungible_token,
            &reserve_manager,
        );

        // Set a parameter and check for events
        client.set_system_parameter(
            &admin,
            &SorobanString::from_str(&env, "timeout"),
            &SorobanString::from_str(&env, "300")
        );

        // Check that events were emitted
        let events = env.events().all();
        assert!(events.len() > 0);
    }

    #[test]
    fn test_configuration_persistence() {
        let env = create_test_env();
        let (admin, kyc_registry, reserve_manager, fungible_token, istsi_token, integration_router) = 
            setup_test_contracts(&env);

        let client = IntegrationRouterClient::new(&env, &integration_router);

        // Initialize
        client.initialize(
            &admin,
            &kyc_registry,
            &istsi_token,
            &fungible_token,
            &reserve_manager,
        );

        // Set configuration values
        let param_name = SorobanString::from_str(&env, "test_param");
        let param_value = SorobanString::from_str(&env, "test_value");
        client.set_system_parameter(&admin, &param_name, &param_value);

        // Simulate ledger advancement
        env.ledger().with_mut(|li| {
            li.sequence_number += 100;
            li.timestamp += 3600; // 1 hour later
        });

        // Verify configuration persists
        let retrieved_value = client.get_system_parameter(&param_name);
        assert_eq!(retrieved_value, Some(param_value));
    }

    #[test]
    fn test_invalid_contract_parameter_setting() {
        let env = create_test_env();
        let (admin, kyc_registry, reserve_manager, fungible_token, istsi_token, integration_router) = 
            setup_test_contracts(&env);

        let client = IntegrationRouterClient::new(&env, &integration_router);

        // Initialize
        client.initialize(
            &admin,
            &kyc_registry,
            &istsi_token,
            &fungible_token,
            &reserve_manager,
        );

        // Try to set parameter for non-existent contract
        let result = std::panic::catch_unwind(|| {
            client.set_contract_parameter(
                &admin,
                &SorobanString::from_str(&env, "non_existent_contract"),
                &SorobanString::from_str(&env, "param"),
                &SorobanString::from_str(&env, "value")
            );
        });
        assert!(result.is_err());
    }
}