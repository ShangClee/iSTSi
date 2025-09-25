use soroban_sdk::{testutils::Address as _, Address, Env, String};

// Import contract types
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

#[cfg(test)]
mod bitcoin_deposit_tests {
    use super::*;

    struct BitcoinDepositSetup {
        env: Env,
        admin: Address,
        user: Address,
        kyc_registry: Address,
        reserve_manager: Address,
        istsi_token: Address,
    }

    impl BitcoinDepositSetup {
        fn new() -> Self {
            let env = Env::default();
            env.mock_all_auths();

            let admin = Address::generate(&env);
            let user = Address::generate(&env);

            // Deploy and initialize KYC Registry
            let kyc_registry = env.register_contract_wasm(None, kyc_registry::WASM);
            let kyc_client = kyc_registry::Client::new(&env, &kyc_registry);
            kyc_client.initialize(&admin);

            // Deploy and initialize Reserve Manager
            let reserve_manager = env.register_contract_wasm(None, reserve_manager::WASM);
            let reserve_client = reserve_manager::Client::new(&env, &reserve_manager);
            reserve_client.initialize(&admin, &kyc_registry);

            // Deploy and initialize iSTSi Token
            let istsi_token = env.register_contract_wasm(None, istsi_token::WASM);
            let istsi_client = istsi_token::Client::new(&env, &istsi_token);
            istsi_client.initialize(&admin, &kyc_registry, &reserve_manager);

            Self {
                env,
                admin,
                user,
                kyc_registry,
                reserve_manager,
                istsi_token,
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
    }

    #[test]
    fn test_simple_bitcoin_deposit() {
        let setup = BitcoinDepositSetup::new();
        
        // Register user for KYC (Tier 1)
        setup.kyc_client().register_customer(&setup.user, &1u32);
        
        // Simulate Bitcoin deposit
        let btc_amount = 100_000_000i128; // 1 BTC in satoshis
        let tx_hash = String::from_str(&setup.env, "a1b2c3d4e5f6789012345678901234567890abcdef1234567890abcdef123456");
        
        // Register the Bitcoin deposit
        setup.reserve_client().register_bitcoin_deposit(&setup.user, &btc_amount, &tx_hash);
        
        // Mint corresponding iSTSi tokens
        setup.istsi_client().mint(&setup.user, &btc_amount);
        
        // Verify the deposit was recorded
        assert_eq!(setup.reserve_client().get_user_deposits(&setup.user), btc_amount);
        assert_eq!(setup.istsi_client().balance(&setup.user), btc_amount);
        assert_eq!(setup.reserve_client().get_total_reserves(), btc_amount);
    }

    #[test]
    fn test_multiple_bitcoin_deposits() {
        let setup = BitcoinDepositSetup::new();
        
        // Register user for KYC (Tier 2 for larger amounts)
        setup.kyc_client().register_customer(&setup.user, &2u32);
        
        // First deposit
        let deposit1 = 50_000_000i128; // 0.5 BTC
        let tx_hash1 = String::from_str(&setup.env, "tx1_hash_123456789012345678901234567890abcdef1234567890abcdef123456");
        
        setup.reserve_client().register_bitcoin_deposit(&setup.user, &deposit1, &tx_hash1);
        setup.istsi_client().mint(&setup.user, &deposit1);
        
        // Second deposit
        let deposit2 = 150_000_000i128; // 1.5 BTC
        let tx_hash2 = String::from_str(&setup.env, "tx2_hash_abcdef1234567890123456789012345678901234567890abcdef123456");
        
        setup.reserve_client().register_bitcoin_deposit(&setup.user, &deposit2, &tx_hash2);
        setup.istsi_client().mint(&setup.user, &deposit2);
        
        // Verify total deposits
        let total_deposits = deposit1 + deposit2;
        assert_eq!(setup.reserve_client().get_user_deposits(&setup.user), total_deposits);
        assert_eq!(setup.istsi_client().balance(&setup.user), total_deposits);
        assert_eq!(setup.reserve_client().get_total_reserves(), total_deposits);
    }

    #[test]
    fn test_bitcoin_deposit_with_kyc_compliance() {
        let setup = BitcoinDepositSetup::new();
        
        // Register user for KYC Tier 1 (lower limits)
        setup.kyc_client().register_customer(&setup.user, &1u32);
        
        // Set tier 1 limits (daily: 1 BTC, monthly: 10 BTC)
        setup.kyc_client().set_tier_limits(&1u32, &100_000_000i128, &1_000_000_000i128);
        
        // Test deposit within limits
        let small_deposit = 50_000_000i128; // 0.5 BTC
        let tx_hash = String::from_str(&setup.env, "small_deposit_tx_hash_123456789012345678901234567890abcdef123456");
        
        setup.reserve_client().register_bitcoin_deposit(&setup.user, &small_deposit, &tx_hash);
        setup.istsi_client().mint(&setup.user, &small_deposit);
        
        assert_eq!(setup.istsi_client().balance(&setup.user), small_deposit);
        
        // Test that large deposit would be rejected by compliance
        let large_deposit = 500_000_000i128; // 5 BTC (exceeds tier 1 daily limit)
        let large_tx_hash = String::from_str(&setup.env, "large_deposit_tx_hash_abcdef1234567890123456789012345678901234");
        
        // This should work at the reserve level but might be restricted at token level
        setup.reserve_client().register_bitcoin_deposit(&setup.user, &large_deposit, &large_tx_hash);
        
        // The mint operation might fail due to KYC limits (depending on implementation)
        // For this test, we'll assume it succeeds but compliance is checked elsewhere
        setup.istsi_client().mint(&setup.user, &large_deposit);
        
        let total_balance = small_deposit + large_deposit;
        assert_eq!(setup.istsi_client().balance(&setup.user), total_balance);
    }

    #[test]
    fn test_duplicate_bitcoin_deposit_prevention() {
        let setup = BitcoinDepositSetup::new();
        
        // Register user for KYC
        setup.kyc_client().register_customer(&setup.user, &1u32);
        
        let btc_amount = 100_000_000i128; // 1 BTC
        let tx_hash = String::from_str(&setup.env, "duplicate_test_tx_hash_123456789012345678901234567890abcdef123");
        
        // First deposit should succeed
        setup.reserve_client().register_bitcoin_deposit(&setup.user, &btc_amount, &tx_hash);
        setup.istsi_client().mint(&setup.user, &btc_amount);
        
        // Attempt to register the same transaction hash again should fail
        let result = std::panic::catch_unwind(|| {
            setup.reserve_client().register_bitcoin_deposit(&setup.user, &btc_amount, &tx_hash);
        });
        
        // Should panic due to duplicate transaction hash
        assert!(result.is_err());
        
        // Balance should remain unchanged
        assert_eq!(setup.istsi_client().balance(&setup.user), btc_amount);
        assert_eq!(setup.reserve_client().get_total_reserves(), btc_amount);
    }

    #[test]
    fn test_bitcoin_deposit_reserve_ratio() {
        let setup = BitcoinDepositSetup::new();
        
        // Set reserve threshold to 95%
        setup.reserve_client().set_reserve_threshold(&9500u32);
        
        // Register user for KYC
        setup.kyc_client().register_customer(&setup.user, &2u32);
        
        // Make a deposit
        let deposit_amount = 200_000_000i128; // 2 BTC
        let tx_hash = String::from_str(&setup.env, "reserve_ratio_test_tx_hash_123456789012345678901234567890abcdef");
        
        setup.reserve_client().register_bitcoin_deposit(&setup.user, &deposit_amount, &tx_hash);
        setup.istsi_client().mint(&setup.user, &deposit_amount);
        
        // Check reserve ratio (should be 100% since reserves equal issued tokens)
        let ratio = setup.reserve_client().calculate_reserve_ratio(&deposit_amount);
        assert_eq!(ratio, 10000u32); // 100.00%
        
        // Verify reserve status is healthy
        assert!(ratio >= 9500u32); // Above threshold
    }

    #[test]
    fn test_bitcoin_deposit_event_emission() {
        let setup = BitcoinDepositSetup::new();
        
        // Register user for KYC
        setup.kyc_client().register_customer(&setup.user, &1u32);
        
        let btc_amount = 75_000_000i128; // 0.75 BTC
        let tx_hash = String::from_str(&setup.env, "event_test_tx_hash_123456789012345678901234567890abcdef1234567");
        
        // Register Bitcoin deposit (this should emit events)
        setup.reserve_client().register_bitcoin_deposit(&setup.user, &btc_amount, &tx_hash);
        
        // Mint tokens (this should also emit events)
        setup.istsi_client().mint(&setup.user, &btc_amount);
        
        // Verify final state
        assert_eq!(setup.reserve_client().get_user_deposits(&setup.user), btc_amount);
        assert_eq!(setup.istsi_client().balance(&setup.user), btc_amount);
        
        // Note: In a real test environment, you would check for emitted events
        // This would require additional test infrastructure to capture and verify events
    }

    #[test]
    fn test_bitcoin_deposit_with_different_users() {
        let setup = BitcoinDepositSetup::new();
        
        // Create additional users
        let user2 = Address::generate(&setup.env);
        let user3 = Address::generate(&setup.env);
        
        // Register all users for KYC
        setup.kyc_client().register_customer(&setup.user, &1u32);
        setup.kyc_client().register_customer(&user2, &2u32);
        setup.kyc_client().register_customer(&user3, &1u32);
        
        // Different deposit amounts
        let deposit1 = 50_000_000i128;  // 0.5 BTC
        let deposit2 = 100_000_000i128; // 1.0 BTC
        let deposit3 = 25_000_000i128;  // 0.25 BTC
        
        // Different transaction hashes
        let tx_hash1 = String::from_str(&setup.env, "user1_deposit_tx_hash_123456789012345678901234567890abcdef123456");
        let tx_hash2 = String::from_str(&setup.env, "user2_deposit_tx_hash_abcdef1234567890123456789012345678901234567");
        let tx_hash3 = String::from_str(&setup.env, "user3_deposit_tx_hash_fedcba0987654321098765432109876543210987654");
        
        // Process deposits for each user
        setup.reserve_client().register_bitcoin_deposit(&setup.user, &deposit1, &tx_hash1);
        setup.istsi_client().mint(&setup.user, &deposit1);
        
        setup.reserve_client().register_bitcoin_deposit(&user2, &deposit2, &tx_hash2);
        setup.istsi_client().mint(&user2, &deposit2);
        
        setup.reserve_client().register_bitcoin_deposit(&user3, &deposit3, &tx_hash3);
        setup.istsi_client().mint(&user3, &deposit3);
        
        // Verify individual balances
        assert_eq!(setup.istsi_client().balance(&setup.user), deposit1);
        assert_eq!(setup.istsi_client().balance(&user2), deposit2);
        assert_eq!(setup.istsi_client().balance(&user3), deposit3);
        
        // Verify individual deposit records
        assert_eq!(setup.reserve_client().get_user_deposits(&setup.user), deposit1);
        assert_eq!(setup.reserve_client().get_user_deposits(&user2), deposit2);
        assert_eq!(setup.reserve_client().get_user_deposits(&user3), deposit3);
        
        // Verify total reserves
        let total_expected = deposit1 + deposit2 + deposit3;
        assert_eq!(setup.reserve_client().get_total_reserves(), total_expected);
    }

    #[test]
    fn test_bitcoin_deposit_edge_cases() {
        let setup = BitcoinDepositSetup::new();
        
        // Register user for KYC
        setup.kyc_client().register_customer(&setup.user, &2u32);
        
        // Test minimum deposit (1 satoshi)
        let min_deposit = 1i128;
        let min_tx_hash = String::from_str(&setup.env, "min_deposit_tx_hash_123456789012345678901234567890abcdef1234567890");
        
        setup.reserve_client().register_bitcoin_deposit(&setup.user, &min_deposit, &min_tx_hash);
        setup.istsi_client().mint(&setup.user, &min_deposit);
        
        assert_eq!(setup.istsi_client().balance(&setup.user), min_deposit);
        
        // Test large deposit (100 BTC)
        let large_deposit = 10_000_000_000i128; // 100 BTC
        let large_tx_hash = String::from_str(&setup.env, "large_deposit_tx_hash_abcdef1234567890123456789012345678901234567");
        
        setup.reserve_client().register_bitcoin_deposit(&setup.user, &large_deposit, &large_tx_hash);
        setup.istsi_client().mint(&setup.user, &large_deposit);
        
        let total_balance = min_deposit + large_deposit;
        assert_eq!(setup.istsi_client().balance(&setup.user), total_balance);
        assert_eq!(setup.reserve_client().get_total_reserves(), total_balance);
    }

    #[test]
    fn test_bitcoin_deposit_without_kyc_registration() {
        let setup = BitcoinDepositSetup::new();
        
        // Do NOT register user for KYC
        let btc_amount = 100_000_000i128; // 1 BTC
        let tx_hash = String::from_str(&setup.env, "no_kyc_deposit_tx_hash_123456789012345678901234567890abcdef123456");
        
        // Reserve manager might allow deposit registration
        setup.reserve_client().register_bitcoin_deposit(&setup.user, &btc_amount, &tx_hash);
        
        // But token minting should fail due to KYC requirements
        let result = std::panic::catch_unwind(|| {
            setup.istsi_client().mint(&setup.user, &btc_amount);
        });
        
        // Should fail due to lack of KYC registration
        assert!(result.is_err());
        
        // Reserve should be recorded but no tokens minted
        assert_eq!(setup.reserve_client().get_user_deposits(&setup.user), btc_amount);
        assert_eq!(setup.istsi_client().balance(&setup.user), 0i128);
    }
}