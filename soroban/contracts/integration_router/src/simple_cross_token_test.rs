#[cfg(test)]
mod simple_cross_token_tests {
    use super::*;
    use soroban_sdk::{testutils::Address as TestAddress, Address, Env, String, BytesN};

    fn setup_simple_test() -> (Env, Address, Address, Address, Address, Address, Address) {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let user = Address::generate(&env);
        let kyc_registry = Address::generate(&env);
        let istsi_token = Address::generate(&env);
        let fungible_token = Address::generate(&env);
        let reserve_manager = Address::generate(&env);

        // Initialize the contract
        IntegrationRouter::initialize(
            env.clone(),
            admin.clone(),
            kyc_registry.clone(),
            istsi_token.clone(),
            fungible_token.clone(),
            reserve_manager.clone()
        );

        (env, admin, user, kyc_registry, istsi_token, fungible_token, reserve_manager)
    }

    #[test]
    fn test_basic_cross_token_exchange_setup() {
        let (env, admin, user, _kyc_registry, istsi_token, fungible_token, _reserve_manager) = setup_simple_test();

        // Set up oracle configuration
        let oracle_address = Address::generate(&env);
        let oracle_result = IntegrationRouter::configure_oracle(
            env.clone(),
            admin.clone(),
            istsi_token.clone(),
            fungible_token.clone(),
            oracle_address,
            300, // 5 minutes
            500, // 5% deviation
            10000 // 1:1 fallback rate
        );

        assert!(oracle_result.is_ok());

        // Set user role
        IntegrationRouter::set_user_role(
            env.clone(),
            admin.clone(),
            user.clone(),
            UserRole::User
        );

        // Check that we can get exchange limits
        let limits = IntegrationRouter::get_exchange_limits(env.clone(), user.clone());
        assert_eq!(limits.user, user);
        assert!(limits.daily_limit > 0);
        assert!(limits.monthly_limit > 0);
    }

    #[test]
    fn test_exchange_limits_management() {
        let (env, admin, user, _kyc_registry, _istsi_token, _fungible_token, _reserve_manager) = setup_simple_test();

        // Set custom exchange limits
        let result = IntegrationRouter::set_exchange_limits(
            env.clone(),
            admin.clone(),
            user.clone(),
            5000000, // 5M daily
            50000000, // 50M monthly
            10000000 // 10M enhanced verification
        );

        assert!(result.is_ok());

        // Verify limits were set correctly
        let limits = IntegrationRouter::get_exchange_limits(env.clone(), user.clone());
        assert_eq!(limits.daily_limit, 5000000);
        assert_eq!(limits.monthly_limit, 50000000);
        assert_eq!(limits.enhanced_verification_limit, 10000000);
        assert_eq!(limits.daily_used, 0);
        assert_eq!(limits.monthly_used, 0);
    }

    #[test]
    fn test_exchange_operation_creation() {
        let (env, admin, user, _kyc_registry, istsi_token, fungible_token, _reserve_manager) = setup_simple_test();

        // Set up oracle
        let oracle_address = Address::generate(&env);
        let _ = IntegrationRouter::configure_oracle(
            env.clone(),
            admin.clone(),
            istsi_token.clone(),
            fungible_token.clone(),
            oracle_address,
            300,
            500,
            10000
        );

        // Set user role
        IntegrationRouter::set_user_role(
            env.clone(),
            admin.clone(),
            user.clone(),
            UserRole::User
        );

        // Execute exchange
        let result = IntegrationRouter::execute_cross_token_exchange(
            env.clone(),
            user.clone(),
            istsi_token.clone(),
            fungible_token.clone(),
            1000000, // 1M tokens
            500 // 5% slippage
        );

        // Should succeed with mocked calls
        assert!(result.is_ok());
        
        let exchange_op = result.unwrap();
        assert_eq!(exchange_op.user, user);
        assert_eq!(exchange_op.from_token, istsi_token);
        assert_eq!(exchange_op.to_token, fungible_token);
        assert_eq!(exchange_op.from_amount, 1000000);
        assert!(exchange_op.to_amount > 0);
        assert_eq!(exchange_op.status, ExchangeStatus::Completed);
    }

    #[test]
    fn test_kyc_compliance_verification() {
        let (env, _admin, user, _kyc_registry, istsi_token, fungible_token, _reserve_manager) = setup_simple_test();

        let result = IntegrationRouter::verify_cross_token_kyc_compliance(
            &env,
            &user,
            &istsi_token,
            &fungible_token,
            1000000
        );

        assert!(result.is_ok());
        let (approved, _message) = result.unwrap();
        // Mock KYC should approve by default
        assert!(approved);
    }

    #[test]
    fn test_token_swap_functions() {
        let (env, _admin, user, _kyc_registry, _istsi_token, _fungible_token, _reserve_manager) = setup_simple_test();
        
        let correlation_id = BytesN::from_array(&env, &[1; 32]);

        // Test iSTSi token operations
        let (burn_success, _) = IntegrationRouter::burn_istsi_tokens_for_exchange(
            &env,
            &user,
            1000000,
            &correlation_id
        );
        assert!(burn_success);

        let (mint_success, _) = IntegrationRouter::mint_istsi_tokens_for_exchange(
            &env,
            &user,
            1000000,
            &correlation_id
        );
        assert!(mint_success);

        // Test fungible token operations
        let (transfer_from_success, _) = IntegrationRouter::transfer_fungible_tokens_from_user(
            &env,
            &user,
            500000,
            &correlation_id
        );
        assert!(transfer_from_success);

        let (transfer_to_success, _) = IntegrationRouter::transfer_fungible_tokens_to_user(
            &env,
            &user,
            500000,
            &correlation_id
        );
        assert!(transfer_to_success);
    }

    #[test]
    fn test_exchange_event_creation() {
        let (env, _admin, user, _kyc_registry, istsi_token, fungible_token, _reserve_manager) = setup_simple_test();
        
        let correlation_id = BytesN::from_array(&env, &[2; 32]);

        let event = IntegrationRouter::create_cross_token_exchange_event(
            &env,
            &user,
            &istsi_token,
            &fungible_token,
            1000000, // from amount
            950000,  // to amount
            50000,   // fee amount
            &correlation_id
        );

        assert_eq!(event.event_type, String::from_str(&env, "CrossTokenExchange"));
        assert_eq!(event.user, user);
        assert_eq!(event.data1, 1000000);
        assert_eq!(event.data2, 950000);
        assert_eq!(event.data3, 50000);
        assert_eq!(event.address1, istsi_token);
        assert_eq!(event.address2, fungible_token);
        assert_eq!(event.correlation_id, correlation_id);
    }
}