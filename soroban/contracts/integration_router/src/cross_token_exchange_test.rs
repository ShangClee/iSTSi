#[cfg(test)]
mod cross_token_exchange_tests {
    use super::*;
    use soroban_sdk::{testutils::Address as TestAddress, Address, Env, String, BytesN};

    fn setup_test_env() -> (Env, Address, Address, Address, Address, Address, Address) {
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

        // Set up oracle configuration
        let oracle_address = Address::generate(&env);
        let _ = IntegrationRouter::configure_oracle(
            env.clone(),
            admin.clone(),
            istsi_token.clone(),
            fungible_token.clone(),
            oracle_address,
            300, // 5 minutes update frequency
            500, // 5% max deviation
            10000 // 1:1 fallback rate
        );

        (env, admin, user, kyc_registry, istsi_token, fungible_token, reserve_manager)
    }

    #[test]
    fn test_execute_cross_token_exchange_success() {
        let (env, _admin, user, _kyc_registry, istsi_token, fungible_token, _reserve_manager) = setup_test_env();

        // Set user role
        IntegrationRouter::set_user_role(
            env.clone(),
            _admin.clone(),
            user.clone(),
            UserRole::User
        );

        // Execute cross-token exchange from iSTSi to fungible
        let result = IntegrationRouter::execute_cross_token_exchange(
            env.clone(),
            user.clone(),
            istsi_token.clone(),
            fungible_token.clone(),
            1000000, // 1M iSTSi tokens
            500 // 5% max slippage
        );

        // Should succeed (mocked KYC and oracle calls will return success)
        assert!(result.is_ok());
        let exchange_op = result.unwrap();
        assert_eq!(exchange_op.user, user);
        assert_eq!(exchange_op.from_token, istsi_token);
        assert_eq!(exchange_op.to_token, fungible_token);
        assert_eq!(exchange_op.from_amount, 1000000);
        assert_eq!(exchange_op.status, ExchangeStatus::Completed);
    }

    #[test]
    fn test_execute_cross_token_exchange_reverse() {
        let (env, _admin, user, _kyc_registry, istsi_token, fungible_token, _reserve_manager) = setup_test_env();

        // Set user role
        IntegrationRouter::set_user_role(
            env.clone(),
            _admin.clone(),
            user.clone(),
            UserRole::User
        );

        // Execute cross-token exchange from fungible to iSTSi
        let result = IntegrationRouter::execute_cross_token_exchange(
            env.clone(),
            user.clone(),
            fungible_token.clone(),
            istsi_token.clone(),
            500000, // 500K fungible tokens
            300 // 3% max slippage
        );

        // Should succeed
        assert!(result.is_ok());
        let exchange_op = result.unwrap();
        assert_eq!(exchange_op.from_token, fungible_token);
        assert_eq!(exchange_op.to_token, istsi_token);
        assert_eq!(exchange_op.from_amount, 500000);
        assert_eq!(exchange_op.status, ExchangeStatus::Completed);
    }

    #[test]
    fn test_execute_cross_token_exchange_system_paused() {
        let (env, admin, user, _kyc_registry, istsi_token, fungible_token, _reserve_manager) = setup_test_env();

        // Pause the system
        IntegrationRouter::emergency_pause(
            env.clone(),
            admin.clone(),
            String::from_str(&env, "Testing pause functionality")
        );

        // Set user role
        IntegrationRouter::set_user_role(
            env.clone(),
            admin.clone(),
            user.clone(),
            UserRole::User
        );

        // Try to execute exchange while paused - should panic
        let result = std::panic::catch_unwind(|| {
            IntegrationRouter::execute_cross_token_exchange(
                env.clone(),
                user.clone(),
                istsi_token.clone(),
                fungible_token.clone(),
                1000000,
                500
            )
        });

        assert!(result.is_err()); // Should panic due to system being paused
    }

    #[test]
    fn test_execute_cross_token_exchange_unauthorized() {
        let (env, _admin, user, _kyc_registry, istsi_token, fungible_token, _reserve_manager) = setup_test_env();
        
        let unauthorized_user = Address::generate(&env);

        // Try to execute exchange without proper authorization - should panic
        let result = std::panic::catch_unwind(|| {
            IntegrationRouter::execute_cross_token_exchange(
                env.clone(),
                unauthorized_user.clone(),
                istsi_token.clone(),
                fungible_token.clone(),
                1000000,
                500
            )
        });

        assert!(result.is_err()); // Should panic due to lack of authorization
    }

    #[test]
    fn test_verify_cross_token_kyc_compliance_success() {
        let (env, _admin, user, _kyc_registry, istsi_token, fungible_token, _reserve_manager) = setup_test_env();

        let result = IntegrationRouter::verify_cross_token_kyc_compliance(
            &env,
            &user,
            &istsi_token,
            &fungible_token,
            1000000
        );

        assert!(result.is_ok());
        let (approved, _message) = result.unwrap();
        assert!(approved); // Mock KYC should approve
    }

    #[test]
    fn test_verify_exchange_limits_within_limits() {
        let (env, admin, user, _kyc_registry, istsi_token, fungible_token, _reserve_manager) = setup_test_env();

        // Set reasonable exchange limits
        let _ = IntegrationRouter::set_exchange_limits(
            env.clone(),
            admin.clone(),
            user.clone(),
            5000000, // 5M daily
            50000000, // 50M monthly
            10000000 // 10M enhanced verification
        );

        let result = IntegrationRouter::verify_exchange_limits(
            &env,
            &user,
            &istsi_token,
            &fungible_token,
            1000000 // 1M - within limits
        );

        assert!(result.is_ok());
        let (approved, _message) = result.unwrap();
        assert!(approved);
    }

    #[test]
    fn test_verify_exchange_limits_exceeds_daily() {
        let (env, admin, user, _kyc_registry, istsi_token, fungible_token, _reserve_manager) = setup_test_env();

        // Set low daily limit
        let _ = IntegrationRouter::set_exchange_limits(
            env.clone(),
            admin.clone(),
            user.clone(),
            500000, // 500K daily (low)
            50000000, // 50M monthly
            10000000 // 10M enhanced verification
        );

        let result = IntegrationRouter::verify_exchange_limits(
            &env,
            &user,
            &istsi_token,
            &fungible_token,
            1000000 // 1M - exceeds daily limit
        );

        assert!(result.is_ok());
        let (approved, message) = result.unwrap();
        assert!(!approved);
        assert!(message.contains("Daily exchange limit exceeded"));
    }

    #[test]
    fn test_verify_exchange_limits_exceeds_enhanced_verification() {
        let (env, admin, user, _kyc_registry, istsi_token, fungible_token, _reserve_manager) = setup_test_env();

        // Set low enhanced verification limit
        let _ = IntegrationRouter::set_exchange_limits(
            env.clone(),
            admin.clone(),
            user.clone(),
            50000000, // 50M daily
            500000000, // 500M monthly
            500000 // 500K enhanced verification (low)
        );

        let result = IntegrationRouter::verify_exchange_limits(
            &env,
            &user,
            &istsi_token,
            &fungible_token,
            1000000 // 1M - exceeds enhanced verification limit
        );

        assert!(result.is_ok());
        let (approved, message) = result.unwrap();
        assert!(!approved);
        assert!(message.contains("enhanced verification limit"));
    }

    #[test]
    fn test_execute_token_swap_atomic_istsi_to_fungible() {
        let (env, _admin, user, _kyc_registry, istsi_token, fungible_token, _reserve_manager) = setup_test_env();
        
        let correlation_id = BytesN::from_array(&env, &[1; 32]);

        let result = IntegrationRouter::execute_token_swap_atomic(
            &env,
            &user,
            &istsi_token,
            &fungible_token,
            1000000, // from amount
            950000,  // to amount (after fees/slippage)
            50000,   // fee amount
            &correlation_id
        );

        // Should succeed with mocked contract calls
        assert!(result.is_ok());
    }

    #[test]
    fn test_execute_token_swap_atomic_fungible_to_istsi() {
        let (env, _admin, user, _kyc_registry, istsi_token, fungible_token, _reserve_manager) = setup_test_env();
        
        let correlation_id = BytesN::from_array(&env, &[2; 32]);

        let result = IntegrationRouter::execute_token_swap_atomic(
            &env,
            &user,
            &fungible_token,
            &istsi_token,
            500000, // from amount
            475000, // to amount (after fees/slippage)
            25000,  // fee amount
            &correlation_id
        );

        // Should succeed with mocked contract calls
        assert!(result.is_ok());
    }

    #[test]
    fn test_burn_istsi_tokens_for_exchange() {
        let (env, _admin, user, _kyc_registry, _istsi_token, _fungible_token, _reserve_manager) = setup_test_env();
        
        let correlation_id = BytesN::from_array(&env, &[3; 32]);

        let (success, _message) = IntegrationRouter::burn_istsi_tokens_for_exchange(
            &env,
            &user,
            1000000,
            &correlation_id
        );

        // Should succeed with mocked contract calls
        assert!(success);
    }

    #[test]
    fn test_mint_istsi_tokens_for_exchange() {
        let (env, _admin, user, _kyc_registry, _istsi_token, _fungible_token, _reserve_manager) = setup_test_env();
        
        let correlation_id = BytesN::from_array(&env, &[4; 32]);

        let (success, _message) = IntegrationRouter::mint_istsi_tokens_for_exchange(
            &env,
            &user,
            1000000,
            &correlation_id
        );

        // Should succeed with mocked contract calls
        assert!(success);
    }

    #[test]
    fn test_transfer_fungible_tokens_from_user() {
        let (env, _admin, user, _kyc_registry, _istsi_token, _fungible_token, _reserve_manager) = setup_test_env();
        
        let correlation_id = BytesN::from_array(&env, &[5; 32]);

        let (success, _message) = IntegrationRouter::transfer_fungible_tokens_from_user(
            &env,
            &user,
            500000,
            &correlation_id
        );

        // Should succeed with mocked contract calls
        assert!(success);
    }

    #[test]
    fn test_transfer_fungible_tokens_to_user() {
        let (env, _admin, user, _kyc_registry, _istsi_token, _fungible_token, _reserve_manager) = setup_test_env();
        
        let correlation_id = BytesN::from_array(&env, &[6; 32]);

        let (success, _message) = IntegrationRouter::transfer_fungible_tokens_to_user(
            &env,
            &user,
            500000,
            &correlation_id
        );

        // Should succeed with mocked contract calls
        assert!(success);
    }

    #[test]
    fn test_collect_exchange_fee() {
        let (env, _admin, user, _kyc_registry, istsi_token, _fungible_token, _reserve_manager) = setup_test_env();
        
        let correlation_id = BytesN::from_array(&env, &[7; 32]);

        let (success, _message) = IntegrationRouter::collect_exchange_fee(
            &env,
            &user,
            &istsi_token,
            50000,
            &correlation_id
        );

        // Should succeed with mocked contract calls
        assert!(success);
    }

    #[test]
    fn test_get_exchange_limits() {
        let (env, _admin, user, _kyc_registry, _istsi_token, _fungible_token, _reserve_manager) = setup_test_env();

        let limits = IntegrationRouter::get_exchange_limits(env.clone(), user.clone());

        // Should return default limits
        assert_eq!(limits.user, user);
        assert_eq!(limits.kyc_tier, 1);
        assert_eq!(limits.daily_limit, 1000000);
        assert_eq!(limits.monthly_limit, 10000000);
        assert_eq!(limits.enhanced_verification_limit, 5000000);
        assert_eq!(limits.daily_used, 0);
        assert_eq!(limits.monthly_used, 0);
    }

    #[test]
    fn test_set_exchange_limits() {
        let (env, admin, user, _kyc_registry, _istsi_token, _fungible_token, _reserve_manager) = setup_test_env();

        let result = IntegrationRouter::set_exchange_limits(
            env.clone(),
            admin.clone(),
            user.clone(),
            2000000, // 2M daily
            20000000, // 20M monthly
            8000000 // 8M enhanced verification
        );

        assert!(result.is_ok());

        // Verify limits were set
        let limits = IntegrationRouter::get_exchange_limits(env.clone(), user.clone());
        assert_eq!(limits.daily_limit, 2000000);
        assert_eq!(limits.monthly_limit, 20000000);
        assert_eq!(limits.enhanced_verification_limit, 8000000);
    }

    #[test]
    fn test_set_exchange_limits_unauthorized() {
        let (env, _admin, user, _kyc_registry, _istsi_token, _fungible_token, _reserve_manager) = setup_test_env();
        
        let unauthorized_user = Address::generate(&env);

        let result = IntegrationRouter::set_exchange_limits(
            env.clone(),
            unauthorized_user.clone(),
            user.clone(),
            2000000,
            20000000,
            8000000
        );

        // Should fail due to insufficient permissions
        assert!(result.is_err());
    }

    #[test]
    fn test_get_exchange_operation() {
        let (env, _admin, user, _kyc_registry, istsi_token, fungible_token, _reserve_manager) = setup_test_env();

        // Set user role
        IntegrationRouter::set_user_role(
            env.clone(),
            _admin.clone(),
            user.clone(),
            UserRole::User
        );

        // Execute an exchange to create an operation
        let result = IntegrationRouter::execute_cross_token_exchange(
            env.clone(),
            user.clone(),
            istsi_token.clone(),
            fungible_token.clone(),
            1000000,
            500
        );

        assert!(result.is_ok());
        let exchange_op = result.unwrap();

        // Retrieve the operation by ID
        let retrieved_op = IntegrationRouter::get_exchange_operation(
            env.clone(),
            exchange_op.operation_id.clone()
        );

        assert!(retrieved_op.is_some());
        let retrieved = retrieved_op.unwrap();
        assert_eq!(retrieved.operation_id, exchange_op.operation_id);
        assert_eq!(retrieved.user, user);
        assert_eq!(retrieved.status, ExchangeStatus::Completed);
    }

    #[test]
    fn test_create_cross_token_exchange_event() {
        let (env, _admin, user, _kyc_registry, istsi_token, fungible_token, _reserve_manager) = setup_test_env();
        
        let correlation_id = BytesN::from_array(&env, &[8; 32]);

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
        assert_eq!(event.data1, 1000000); // from amount
        assert_eq!(event.data2, 950000);  // to amount
        assert_eq!(event.data3, 50000);   // fee amount
        assert_eq!(event.address1, istsi_token); // from token
        assert_eq!(event.address2, fungible_token); // to token
        assert_eq!(event.hash_data, correlation_id);
        assert_eq!(event.text_data, String::from_str(&env, "atomic_swap_completed"));
    }

    #[test]
    fn test_rollback_from_token_transfer_istsi() {
        let (env, _admin, user, _kyc_registry, istsi_token, _fungible_token, _reserve_manager) = setup_test_env();
        
        let correlation_id = BytesN::from_array(&env, &[9; 32]);

        let (success, _message) = IntegrationRouter::rollback_from_token_transfer(
            &env,
            &user,
            &istsi_token,
            1000000,
            &correlation_id
        );

        // Should succeed (re-mint tokens)
        assert!(success);
    }

    #[test]
    fn test_rollback_to_token_transfer_fungible() {
        let (env, _admin, user, _kyc_registry, _istsi_token, fungible_token, _reserve_manager) = setup_test_env();
        
        let correlation_id = BytesN::from_array(&env, &[10; 32]);

        let (success, _message) = IntegrationRouter::rollback_to_token_transfer(
            &env,
            &user,
            &fungible_token,
            500000,
            &correlation_id
        );

        // Should succeed (transfer tokens back)
        assert!(success);
    }
}