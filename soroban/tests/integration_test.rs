use soroban_sdk::{testutils::Address as _, Address, Env};

// Import contract types from the new directory structure
mod kyc_registry {
    soroban_sdk::contractimport!(
        file = "../contracts/kyc_registry/target/wasm32-unknown-unknown/release/kyc_registry.wasm"
    );
}

mod reserve_manager {
    soroban_sdk::contractimport!(
        file = "../contracts/reserve_manager/target/wasm32-unknown-unknown/release/reserve_manager.wasm"
    );
}

mod istsi_token {
    soroban_sdk::contractimport!(
        file = "../contracts/istsi_token/target/wasm32-unknown-unknown/release/istsi_token.wasm"
    );
}

mod fungible_token {
    soroban_sdk::contractimport!(
        file = "../contracts/fungible/target/wasm32-unknown-unknown/release/fungible.wasm"
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestSetup {
        env: Env,
        admin: Address,
        user1: Address,
        user2: Address,
        kyc_registry: Address,
        reserve_manager: Address,
        istsi_token: Address,
        fungible_token: Address,
    }

    impl TestSetup {
        fn new() -> Self {
            let env = Env::default();
            env.mock_all_auths();

            let admin = Address::generate(&env);
            let user1 = Address::generate(&env);
            let user2 = Address::generate(&env);

            // Deploy KYC Registry
            let kyc_registry = env.register_contract_wasm(None, kyc_registry::WASM);
            let kyc_client = kyc_registry::Client::new(&env, &kyc_registry);
            kyc_client.initialize(&admin);

            // Deploy Reserve Manager
            let reserve_manager = env.register_contract_wasm(None, reserve_manager::WASM);
            let reserve_client = reserve_manager::Client::new(&env, &reserve_manager);
            reserve_client.initialize(&admin, &kyc_registry);

            // Deploy Fungible Token
            let fungible_token = env.register_contract_wasm(None, fungible_token::WASM);
            let fungible_client = fungible_token::Client::new(&env, &fungible_token);
            fungible_client.initialize(
                &admin,
                &7u32,
                &"Test Fungible Token".into_val(&env),
                &"TFT".into_val(&env),
            );

            // Deploy iSTSi Token
            let istsi_token = env.register_contract_wasm(None, istsi_token::WASM);
            let istsi_client = istsi_token::Client::new(&env, &istsi_token);
            istsi_client.initialize(&admin, &kyc_registry, &reserve_manager);

            Self {
                env,
                admin,
                user1,
                user2,
                kyc_registry,
                reserve_manager,
                istsi_token,
                fungible_token,
            }
        }

        fn kyc_client(&self) -> kyc_registry::Client {
            kyc_registry::Client::new(&self.env, &self.kyc_registry)
        }

        fn reserve_client(&self) -> reserve_manager::Client {
            reserve_manager::Client::new(&self.env, &self.reserve_manager)
        }

        fn istsi_client(&self) -> istsi_token::Client {
            istsi_token::Client::new(&self.env, &self.istsi_token)
        }

        fn fungible_client(&self) -> fungible_token::Client {
            fungible_token::Client::new(&self.env, &self.fungible_token)
        }
    }

    #[test]
    fn test_contract_initialization() {
        let setup = TestSetup::new();

        // Test KYC Registry initialization
        assert_eq!(setup.kyc_client().get_admin(), setup.admin);

        // Test Reserve Manager initialization
        assert_eq!(setup.reserve_client().get_admin(), setup.admin);

        // Test iSTSi Token initialization
        assert_eq!(setup.istsi_client().get_admin(), setup.admin);
        assert_eq!(setup.istsi_client().get_kyc_registry(), setup.kyc_registry);
        assert_eq!(setup.istsi_client().get_reserve_manager(), setup.reserve_manager);

        // Test Fungible Token initialization
        assert_eq!(setup.fungible_client().get_admin(), setup.admin);
        assert_eq!(setup.fungible_client().name(), "Test Fungible Token".into_val(&setup.env));
        assert_eq!(setup.fungible_client().symbol(), "TFT".into_val(&setup.env));
        assert_eq!(setup.fungible_client().decimals(), 7u32);
    }

    #[test]
    fn test_kyc_registration_and_compliance() {
        let setup = TestSetup::new();
        let kyc_client = setup.kyc_client();

        // Register user1 with tier 1
        kyc_client.register_customer(&setup.user1, &1u32);

        // Verify registration
        assert!(kyc_client.is_customer_registered(&setup.user1));
        assert_eq!(kyc_client.get_customer_tier(&setup.user1), 1u32);

        // Test compliance check
        assert!(kyc_client.is_approved(&setup.user1, &10_000_000i128)); // 0.1 BTC
    }

    #[test]
    fn test_bitcoin_deposit_flow() {
        let setup = TestSetup::new();
        let kyc_client = setup.kyc_client();
        let reserve_client = setup.reserve_client();
        let istsi_client = setup.istsi_client();

        // Register user for KYC
        kyc_client.register_customer(&setup.user1, &1u32);

        // Simulate Bitcoin deposit
        let btc_amount = 100_000_000i128; // 1 BTC
        let tx_hash = "test_tx_hash_123".into_val(&setup.env);

        reserve_client.register_bitcoin_deposit(&setup.user1, &btc_amount, &tx_hash);

        // Mint corresponding iSTSi tokens
        istsi_client.mint(&setup.user1, &btc_amount);

        // Verify balances
        assert_eq!(istsi_client.balance(&setup.user1), btc_amount);
        assert_eq!(reserve_client.get_user_deposits(&setup.user1), btc_amount);
    }

    #[test]
    fn test_token_transfer_with_compliance() {
        let setup = TestSetup::new();
        let kyc_client = setup.kyc_client();
        let istsi_client = setup.istsi_client();

        // Register both users for KYC
        kyc_client.register_customer(&setup.user1, &1u32);
        kyc_client.register_customer(&setup.user2, &1u32);

        // Mint tokens to user1
        let initial_amount = 100_000_000i128; // 1 BTC worth
        istsi_client.mint(&setup.user1, &initial_amount);

        // Transfer tokens from user1 to user2
        let transfer_amount = 50_000_000i128; // 0.5 BTC worth
        istsi_client.transfer(&setup.user1, &setup.user2, &transfer_amount);

        // Verify balances
        assert_eq!(istsi_client.balance(&setup.user1), initial_amount - transfer_amount);
        assert_eq!(istsi_client.balance(&setup.user2), transfer_amount);
    }

    #[test]
    fn test_reserve_management() {
        let setup = TestSetup::new();
        let reserve_client = setup.reserve_client();

        // Set reserve threshold
        let threshold = 9500u32; // 95%
        reserve_client.set_reserve_threshold(&threshold);
        assert_eq!(reserve_client.get_reserve_threshold(), threshold);

        // Register Bitcoin deposits
        let deposit1 = 100_000_000i128; // 1 BTC
        let deposit2 = 200_000_000i128; // 2 BTC
        
        reserve_client.register_bitcoin_deposit(
            &setup.user1, 
            &deposit1, 
            &"tx1".into_val(&setup.env)
        );
        reserve_client.register_bitcoin_deposit(
            &setup.user2, 
            &deposit2, 
            &"tx2".into_val(&setup.env)
        );

        // Verify total reserves
        let expected_total = deposit1 + deposit2;
        assert_eq!(reserve_client.get_total_reserves(), expected_total);

        // Test reserve ratio calculation
        let ratio = reserve_client.calculate_reserve_ratio(&expected_total);
        assert_eq!(ratio, 10000u32); // 100% when reserves equal issued tokens
    }

    #[test]
    fn test_withdrawal_flow() {
        let setup = TestSetup::new();
        let kyc_client = setup.kyc_client();
        let reserve_client = setup.reserve_client();
        let istsi_client = setup.istsi_client();

        // Setup: Register user and create deposit
        kyc_client.register_customer(&setup.user1, &1u32);
        
        let deposit_amount = 100_000_000i128; // 1 BTC
        reserve_client.register_bitcoin_deposit(
            &setup.user1, 
            &deposit_amount, 
            &"deposit_tx".into_val(&setup.env)
        );
        istsi_client.mint(&setup.user1, &deposit_amount);

        // Initiate withdrawal
        let withdrawal_amount = 50_000_000i128; // 0.5 BTC
        let btc_address = "bc1qtest123".into_val(&setup.env);
        
        // Burn tokens for withdrawal
        istsi_client.burn(&setup.user1, &withdrawal_amount);
        
        // Register withdrawal in reserve manager
        reserve_client.register_bitcoin_withdrawal(
            &setup.user1,
            &withdrawal_amount,
            &btc_address,
            &"withdrawal_tx".into_val(&setup.env)
        );

        // Verify final state
        assert_eq!(istsi_client.balance(&setup.user1), deposit_amount - withdrawal_amount);
        assert_eq!(
            reserve_client.get_user_deposits(&setup.user1), 
            deposit_amount - withdrawal_amount
        );
    }

    #[test]
    fn test_cross_contract_integration() {
        let setup = TestSetup::new();
        let kyc_client = setup.kyc_client();
        let reserve_client = setup.reserve_client();
        let istsi_client = setup.istsi_client();

        // Test that iSTSi token properly integrates with KYC registry
        assert_eq!(istsi_client.get_kyc_registry(), setup.kyc_registry);
        assert_eq!(istsi_client.get_reserve_manager(), setup.reserve_manager);

        // Test that reserve manager integrates with KYC registry
        // (This would depend on the specific implementation)

        // Register user and test compliance integration
        kyc_client.register_customer(&setup.user1, &2u32); // Tier 2
        
        // Test that token operations respect KYC compliance
        let large_amount = 1_000_000_000i128; // 10 BTC
        
        // This should work for tier 2 user
        istsi_client.mint(&setup.user1, &large_amount);
        assert_eq!(istsi_client.balance(&setup.user1), large_amount);

        // Test transfer compliance
        kyc_client.register_customer(&setup.user2, &1u32); // Tier 1
        
        // Transfer within tier 1 limits should work
        let small_transfer = 10_000_000i128; // 0.1 BTC
        istsi_client.transfer(&setup.user1, &setup.user2, &small_transfer);
        
        assert_eq!(istsi_client.balance(&setup.user2), small_transfer);
    }

    #[test]
    fn test_error_conditions() {
        let setup = TestSetup::new();
        let kyc_client = setup.kyc_client();
        let istsi_client = setup.istsi_client();

        // Test transfer without KYC registration should fail
        let amount = 1_000_000i128;
        istsi_client.mint(&setup.user1, &amount);

        // This should fail because user2 is not KYC registered
        let result = std::panic::catch_unwind(|| {
            istsi_client.transfer(&setup.user1, &setup.user2, &amount);
        });
        assert!(result.is_err());

        // Test exceeding tier limits
        kyc_client.register_customer(&setup.user1, &1u32); // Tier 1
        kyc_client.register_customer(&setup.user2, &1u32); // Tier 1
        
        // Try to transfer amount exceeding tier 1 limits
        let large_amount = 1_000_000_000i128; // 10 BTC (exceeds tier 1 limits)
        istsi_client.mint(&setup.user1, &large_amount);
        
        let result = std::panic::catch_unwind(|| {
            istsi_client.transfer(&setup.user1, &setup.user2, &large_amount);
        });
        assert!(result.is_err());
    }

    #[test]
    fn test_admin_functions() {
        let setup = TestSetup::new();
        let kyc_client = setup.kyc_client();
        let reserve_client = setup.reserve_client();
        let istsi_client = setup.istsi_client();

        // Test admin-only functions
        assert_eq!(kyc_client.get_admin(), setup.admin);
        assert_eq!(reserve_client.get_admin(), setup.admin);
        assert_eq!(istsi_client.get_admin(), setup.admin);

        // Test setting tier limits (admin only)
        kyc_client.set_tier_limits(&1u32, &100_000_000i128, &1_000_000_000i128);
        
        // Test setting reserve threshold (admin only)
        reserve_client.set_reserve_threshold(&9000u32); // 90%
        assert_eq!(reserve_client.get_reserve_threshold(), 9000u32);

        // Test pausing/unpausing contracts (if implemented)
        // This would depend on the specific contract implementations
    }
}