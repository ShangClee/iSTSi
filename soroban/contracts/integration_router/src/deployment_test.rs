#[cfg(test)]
mod deployment_tests {
    use super::*;
    use soroban_sdk::{
        testutils::{Address as TestAddress, Ledger, LedgerInfo},
        Address, Env, Map, String as SorobanString, Vec as SorobanVec,
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
    fn test_deployment_initialization() {
        let env = create_test_env();
        let (admin, kyc_registry, reserve_manager, fungible_token, istsi_token, integration_router) = 
            setup_test_contracts(&env);

        let client = IntegrationRouterClient::new(&env, &integration_router);

        // Test initialization
        client.initialize(
            &admin,
            &kyc_registry,
            &istsi_token,
            &fungible_token,
            &reserve_manager,
        );

        // Verify configuration
        let config = client.get_config();
        assert_eq!(config.admin, admin);
        assert_eq!(config.kyc_registry, kyc_registry);
        assert_eq!(config.istsi_token, istsi_token);
        assert_eq!(config.fungible_token, fungible_token);
        assert_eq!(config.reserve_manager, reserve_manager);
        assert_eq!(config.paused, false);

        // Verify admin role
        let admin_role = client.get_user_role(&admin);
        assert_eq!(admin_role, UserRole::SuperAdmin);

        // Verify system is not paused
        assert_eq!(client.is_paused(), false);
    }

    #[test]
    fn test_contract_address_registry() {
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

        // Test getting contract addresses
        let kyc_name = SorobanString::from_str(&env, "kyc_registry");
        let retrieved_kyc = client.get_contract_address(&kyc_name);
        assert_eq!(retrieved_kyc, Some(kyc_registry));

        let istsi_name = SorobanString::from_str(&env, "istsi_token");
        let retrieved_istsi = client.get_contract_address(&istsi_name);
        assert_eq!(retrieved_istsi, Some(istsi_token));

        // Test getting all contract addresses
        let all_contracts = client.get_all_contract_addresses();
        assert_eq!(all_contracts.get(kyc_name.clone()), Some(kyc_registry));
        assert_eq!(all_contracts.get(istsi_name.clone()), Some(istsi_token));
    }

    #[test]
    fn test_batch_contract_address_update() {
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

        // Create new addresses for update
        let new_kyc = Address::generate(&env);
        let new_reserve = Address::generate(&env);

        // Create batch update map
        let mut contracts = Map::new(&env);
        contracts.set(SorobanString::from_str(&env, "kyc_registry"), new_kyc.clone());
        contracts.set(SorobanString::from_str(&env, "reserve_manager"), new_reserve.clone());

        // Perform batch update
        client.batch_update_contract_addresses(&admin, &contracts);

        // Verify updates
        let kyc_name = SorobanString::from_str(&env, "kyc_registry");
        let reserve_name = SorobanString::from_str(&env, "reserve_manager");
        
        assert_eq!(client.get_contract_address(&kyc_name), Some(new_kyc));
        assert_eq!(client.get_contract_address(&reserve_name), Some(new_reserve));

        // Verify configuration was updated
        let config = client.get_config();
        assert_eq!(config.kyc_registry, new_kyc);
        assert_eq!(config.reserve_manager, new_reserve);
    }

    #[test]
    fn test_deployment_configuration_validation() {
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

        // Test valid configuration
        let mut valid_contracts = Map::new(&env);
        valid_contracts.set(SorobanString::from_str(&env, "kyc_registry"), kyc_registry);
        valid_contracts.set(SorobanString::from_str(&env, "istsi_token"), istsi_token);
        valid_contracts.set(SorobanString::from_str(&env, "fungible_token"), fungible_token);
        valid_contracts.set(SorobanString::from_str(&env, "reserve_manager"), reserve_manager);

        let is_valid = client.validate_deployment_config(&admin, &valid_contracts);
        assert_eq!(is_valid, true);

        // Test invalid configuration (missing required contract)
        let mut invalid_contracts = Map::new(&env);
        invalid_contracts.set(SorobanString::from_str(&env, "kyc_registry"), kyc_registry);
        invalid_contracts.set(SorobanString::from_str(&env, "istsi_token"), istsi_token);
        // Missing fungible_token and reserve_manager

        let is_invalid = client.validate_deployment_config(&admin, &invalid_contracts);
        assert_eq!(is_invalid, false);
    }

    #[test]
    fn test_deployment_health_check() {
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

        // Perform health check
        let health_status = client.deployment_health_check(&admin);

        // Verify health check results
        // Note: In a real test environment, these would check actual contract responses
        // For now, we verify the structure is correct
        assert!(health_status.len() > 0);
        
        // Check that all required contracts are included in health check
        let kyc_name = SorobanString::from_str(&env, "kyc_registry");
        let istsi_name = SorobanString::from_str(&env, "istsi_token");
        let fungible_name = SorobanString::from_str(&env, "fungible_token");
        let reserve_name = SorobanString::from_str(&env, "reserve_manager");

        assert!(health_status.contains_key(kyc_name));
        assert!(health_status.contains_key(istsi_name));
        assert!(health_status.contains_key(fungible_name));
        assert!(health_status.contains_key(reserve_name));
    }

    #[test]
    fn test_deployment_status_reporting() {
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

        // Get deployment status
        let deployment_status = client.get_deployment_status(&admin);

        // Verify status structure
        assert!(deployment_status.len() > 0);
        
        // Check for overall status
        let overall_key = SorobanString::from_str(&env, "overall");
        assert!(deployment_status.contains_key(overall_key));
        
        // Verify individual contract statuses
        let kyc_name = SorobanString::from_str(&env, "kyc_registry");
        let istsi_name = SorobanString::from_str(&env, "istsi_token");
        
        assert!(deployment_status.contains_key(kyc_name));
        assert!(deployment_status.contains_key(istsi_name));
    }

    #[test]
    fn test_unauthorized_deployment_operations() {
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

        // Test unauthorized batch update
        let mut contracts = Map::new(&env);
        contracts.set(SorobanString::from_str(&env, "kyc_registry"), Address::generate(&env));

        // This should panic due to insufficient permissions
        let result = std::panic::catch_unwind(|| {
            client.batch_update_contract_addresses(&unauthorized_user, &contracts);
        });
        assert!(result.is_err());

        // Test unauthorized validation
        let result = std::panic::catch_unwind(|| {
            client.validate_deployment_config(&unauthorized_user, &contracts);
        });
        assert!(result.is_err());

        // Test unauthorized health check
        let result = std::panic::catch_unwind(|| {
            client.deployment_health_check(&unauthorized_user);
        });
        assert!(result.is_err());
    }

    #[test]
    fn test_contract_address_update_events() {
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

        // Update a single contract address
        let new_kyc = Address::generate(&env);
        let kyc_name = SorobanString::from_str(&env, "kyc_registry");
        
        client.update_contract_address(&admin, &kyc_name, &new_kyc);

        // Verify the update
        assert_eq!(client.get_contract_address(&kyc_name), Some(new_kyc));

        // Check that events were emitted (in a real test, we'd verify event content)
        let events = env.events().all();
        assert!(events.len() > 0);
    }

    #[test]
    fn test_deployment_configuration_persistence() {
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

        // Simulate ledger advancement (contract state persistence)
        env.ledger().with_mut(|li| {
            li.sequence_number += 100;
            li.timestamp += 3600; // 1 hour later
        });

        // Verify configuration persists
        let config = client.get_config();
        assert_eq!(config.admin, admin);
        assert_eq!(config.kyc_registry, kyc_registry);
        assert_eq!(config.istsi_token, istsi_token);

        // Verify contract addresses persist
        let kyc_name = SorobanString::from_str(&env, "kyc_registry");
        assert_eq!(client.get_contract_address(&kyc_name), Some(kyc_registry));
    }

    #[test]
    fn test_deployment_rollback_scenario() {
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

        // Store original configuration
        let original_config = client.get_config();

        // Perform an update
        let new_kyc = Address::generate(&env);
        let kyc_name = SorobanString::from_str(&env, "kyc_registry");
        client.update_contract_address(&admin, &kyc_name, &new_kyc);

        // Verify update
        assert_eq!(client.get_contract_address(&kyc_name), Some(new_kyc));

        // Rollback by updating back to original
        client.update_contract_address(&admin, &kyc_name, &original_config.kyc_registry);

        // Verify rollback
        assert_eq!(client.get_contract_address(&kyc_name), Some(original_config.kyc_registry));
        
        let current_config = client.get_config();
        assert_eq!(current_config.kyc_registry, original_config.kyc_registry);
    }
}