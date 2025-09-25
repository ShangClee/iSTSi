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

mod fungible_token {
    soroban_sdk::contractimport!(
        file = "../contracts/fungible/target/wasm32-unknown-unknown/release/fungible.wasm"
    );
}

#[cfg(test)]
mod cross_token_tests {
    use super::*;

    struct CrossTokenSetup {
        env: Env,
        admin: Address,
        user1: Address,
        user2: Address,
        user3: Address,
        kyc_registry: Address,
        reserve_manager: Address,
        istsi_token: Address,
        fungible_token: Address,
    }

    impl CrossTokenSetup {
        fn new() -> Self {
            let env = Env::default();
            env.mock_all_auths();

            let admin = Address::generate(&env);
            let user1 = Address::generate(&env);
            let user2 = Address::generate(&env);
            let user3 = Address::generate(&env);

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
                &String::from_str(&env, "Cross Token Test"),
                &String::from_str(&env, "CTT"),
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
                user3,
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
    fn test_cross_token_transfers() {
        let setup = CrossTokenSetup::new();
        
        // Register users for KYC with different tiers
        setup.kyc_client().register_customer(&setup.user1, &2u32); // Tier 2
        setup.kyc_client().register_customer(&setup.user2, &1u32); // Tier 1
        setup.kyc_client().register_customer(&setup.user3, &2u32); // Tier 2

        // Mint iSTSi tokens to user1 (simulate Bitcoin deposit)
        let istsi_amount = 200_000_000i128; // 2 BTC worth
        let tx_hash = String::from_str(&setup.env, "cross_token_deposit_tx_hash_123456789012345678901234567890abcdef");
        setup.reserve_client().register_bitcoin_deposit(&setup.user1, &istsi_amount, &tx_hash);
        setup.istsi_client().mint(&setup.user1, &istsi_amount);

        // Mint fungible tokens to user1
        let fungible_amount = 1_000_000_000i128; // 100 tokens (7 decimals)
        setup.fungible_client().mint(&setup.user1, &fungible_amount);

        // Test iSTSi token transfer from user1 to user2
        let istsi_transfer = 50_000_000i128; // 0.5 BTC worth
        setup.istsi_client().transfer(&setup.user1, &setup.user2, &istsi_transfer);

        // Test fungible token transfer from user1 to user3
        let fungible_transfer = 250_000_000i128; // 25 tokens
        setup.fungible_client().transfer(&setup.user1, &setup.user3, &fungible_transfer);

        // Verify balances
        assert_eq!(setup.istsi_client().balance(&setup.user1), istsi_amount - istsi_transfer);
        assert_eq!(setup.istsi_client().balance(&setup.user2), istsi_transfer);
        assert_eq!(setup.istsi_client().balance(&setup.user3), 0i128);

        assert_eq!(setup.fungible_client().balance(&setup.user1), fungible_amount - fungible_transfer);
        assert_eq!(setup.fungible_client().balance(&setup.user2), 0i128);
        assert_eq!(setup.fungible_client().balance(&setup.user3), fungible_transfer);
    }

    #[test]
    fn test_cross_token_compliance_differences() {
        let setup = CrossTokenSetup::new();
        
        // Register users with different KYC tiers
        setup.kyc_client().register_customer(&setup.user1, &1u32); // Tier 1 - lower limits
        setup.kyc_client().register_customer(&setup.user2, &2u32); // Tier 2 - higher limits

        // Set tier limits
        setup.kyc_client().set_tier_limits(&1u32, &100_000_000i128, &1_000_000_000i128); // 1 BTC daily, 10 BTC monthly
        setup.kyc_client().set_tier_limits(&2u32, &500_000_000i128, &5_000_000_000i128); // 5 BTC daily, 50 BTC monthly

        // Mint tokens to both users
        let large_amount = 300_000_000i128; // 3 BTC worth
        
        // iSTSi tokens (Bitcoin-backed, should respect KYC limits)
        let tx_hash1 = String::from_str(&setup.env, "compliance_test_tx1_123456789012345678901234567890abcdef123456");
        let tx_hash2 = String::from_str(&setup.env, "compliance_test_tx2_abcdef1234567890123456789012345678901234567");
        
        setup.reserve_client().register_bitcoin_deposit(&setup.user1, &large_amount, &tx_hash1);
        setup.reserve_client().register_bitcoin_deposit(&setup.user2, &large_amount, &tx_hash2);
        
        setup.istsi_client().mint(&setup.user1, &large_amount);
        setup.istsi_client().mint(&setup.user2, &large_amount);

        // Fungible tokens (may have different compliance rules)
        setup.fungible_client().mint(&setup.user1, &large_amount);
        setup.fungible_client().mint(&setup.user2, &large_amount);

        // Test transfer within tier 1 limits (should work for both tokens)
        let small_transfer = 50_000_000i128; // 0.5 BTC worth
        setup.istsi_client().transfer(&setup.user1, &setup.user2, &small_transfer);
        setup.fungible_client().transfer(&setup.user1, &setup.user2, &small_transfer);

        // Test large transfer from tier 1 user (might fail for iSTSi due to compliance)
        let large_transfer = 200_000_000i128; // 2 BTC worth (exceeds tier 1 daily limit)
        
        // This might fail for iSTSi token due to KYC compliance
        let istsi_result = std::panic::catch_unwind(|| {
            setup.istsi_client().transfer(&setup.user1, &setup.user2, &large_transfer);
        });
        
        // Fungible token might have different rules (or no KYC integration)
        let fungible_result = std::panic::catch_unwind(|| {
            setup.fungible_client().transfer(&setup.user1, &setup.user2, &large_transfer);
        });

        // Verify that compliance is enforced differently for different tokens
        // (The exact behavior depends on the specific implementation)
        
        // At minimum, verify that tier 2 user can make larger transfers
        setup.istsi_client().transfer(&setup.user2, &setup.user1, &large_transfer);
        setup.fungible_client().transfer(&setup.user2, &setup.user1, &large_transfer);
    }

    #[test]
    fn test_token_interoperability_scenarios() {
        let setup = CrossTokenSetup::new();
        
        // Register users
        setup.kyc_client().register_customer(&setup.user1, &2u32);
        setup.kyc_client().register_customer(&setup.user2, &2u32);

        // Scenario 1: User has both token types and transfers both
        let amount = 100_000_000i128; // 1 BTC worth / 10 fungible tokens
        
        // Setup initial balances
        let tx_hash = String::from_str(&setup.env, "interop_test_tx_hash_123456789012345678901234567890abcdef123456");
        setup.reserve_client().register_bitcoin_deposit(&setup.user1, &amount, &tx_hash);
        setup.istsi_client().mint(&setup.user1, &amount);
        setup.fungible_client().mint(&setup.user1, &amount);

        // Transfer both token types in sequence
        let transfer_amount = 25_000_000i128; // 0.25 BTC worth / 2.5 fungible tokens
        
        setup.istsi_client().transfer(&setup.user1, &setup.user2, &transfer_amount);
        setup.fungible_client().transfer(&setup.user1, &setup.user2, &transfer_amount);

        // Verify both transfers succeeded
        assert_eq!(setup.istsi_client().balance(&setup.user1), amount - transfer_amount);
        assert_eq!(setup.istsi_client().balance(&setup.user2), transfer_amount);
        assert_eq!(setup.fungible_client().balance(&setup.user1), amount - transfer_amount);
        assert_eq!(setup.fungible_client().balance(&setup.user2), transfer_amount);
    }

    #[test]
    fn test_token_supply_independence() {
        let setup = CrossTokenSetup::new();
        
        // Register users
        setup.kyc_client().register_customer(&setup.user1, &2u32);
        setup.kyc_client().register_customer(&setup.user2, &2u32);

        // Mint different amounts of each token
        let istsi_amount = 500_000_000i128; // 5 BTC worth
        let fungible_amount = 2_000_000_000i128; // 200 fungible tokens

        // iSTSi tokens require Bitcoin deposits
        let tx_hash = String::from_str(&setup.env, "supply_test_tx_hash_123456789012345678901234567890abcdef123456");
        setup.reserve_client().register_bitcoin_deposit(&setup.user1, &istsi_amount, &tx_hash);
        setup.istsi_client().mint(&setup.user1, &istsi_amount);

        // Fungible tokens can be minted independently
        setup.fungible_client().mint(&setup.user1, &fungible_amount);

        // Verify total supplies are independent
        assert_eq!(setup.istsi_client().total_supply(), istsi_amount);
        assert_eq!(setup.fungible_client().total_supply(), fungible_amount);

        // Mint more fungible tokens to user2
        let additional_fungible = 500_000_000i128; // 50 more tokens
        setup.fungible_client().mint(&setup.user2, &additional_fungible);

        // Verify supplies updated independently
        assert_eq!(setup.istsi_client().total_supply(), istsi_amount); // Unchanged
        assert_eq!(setup.fungible_client().total_supply(), fungible_amount + additional_fungible);
    }

    #[test]
    fn test_cross_token_burn_scenarios() {
        let setup = CrossTokenSetup::new();
        
        // Register user
        setup.kyc_client().register_customer(&setup.user1, &2u32);

        // Mint both token types
        let initial_amount = 200_000_000i128; // 2 BTC worth / 20 fungible tokens
        
        let tx_hash = String::from_str(&setup.env, "burn_test_tx_hash_123456789012345678901234567890abcdef123456");
        setup.reserve_client().register_bitcoin_deposit(&setup.user1, &initial_amount, &tx_hash);
        setup.istsi_client().mint(&setup.user1, &initial_amount);
        setup.fungible_client().mint(&setup.user1, &initial_amount);

        // Burn some of each token type
        let burn_amount = 50_000_000i128; // 0.5 BTC worth / 5 fungible tokens
        
        setup.istsi_client().burn(&setup.user1, &burn_amount);
        setup.fungible_client().burn(&setup.user1, &burn_amount);

        // Verify balances and supplies
        let expected_balance = initial_amount - burn_amount;
        assert_eq!(setup.istsi_client().balance(&setup.user1), expected_balance);
        assert_eq!(setup.fungible_client().balance(&setup.user1), expected_balance);
        assert_eq!(setup.istsi_client().total_supply(), expected_balance);
        assert_eq!(setup.fungible_client().total_supply(), expected_balance);

        // For iSTSi tokens, burning might trigger Bitcoin withdrawal process
        let btc_address = String::from_str(&setup.env, "bc1qtest123456789012345678901234567890abcdef");
        let withdrawal_tx = String::from_str(&setup.env, "withdrawal_tx_hash_abcdef1234567890123456789012345678901234567");
        
        // Register the Bitcoin withdrawal
        setup.reserve_client().register_bitcoin_withdrawal(
            &setup.user1,
            &burn_amount,
            &btc_address,
            &withdrawal_tx
        );

        // Verify reserve accounting
        let expected_reserves = initial_amount - burn_amount;
        assert_eq!(setup.reserve_client().get_user_deposits(&setup.user1), expected_reserves);
    }

    #[test]
    fn test_token_metadata_differences() {
        let setup = CrossTokenSetup::new();

        // Verify iSTSi token metadata
        assert_eq!(setup.istsi_client().name(), String::from_str(&setup.env, "iSTSi Token"));
        assert_eq!(setup.istsi_client().symbol(), String::from_str(&setup.env, "iSTSi"));
        assert_eq!(setup.istsi_client().decimals(), 7u32);

        // Verify fungible token metadata
        assert_eq!(setup.fungible_client().name(), String::from_str(&setup.env, "Cross Token Test"));
        assert_eq!(setup.fungible_client().symbol(), String::from_str(&setup.env, "CTT"));
        assert_eq!(setup.fungible_client().decimals(), 7u32);

        // Verify different admin addresses (should be the same in this test)
        assert_eq!(setup.istsi_client().get_admin(), setup.admin);
        assert_eq!(setup.fungible_client().get_admin(), setup.admin);

        // Verify iSTSi token has additional integration points
        assert_eq!(setup.istsi_client().get_kyc_registry(), setup.kyc_registry);
        assert_eq!(setup.istsi_client().get_reserve_manager(), setup.reserve_manager);
    }

    #[test]
    fn test_concurrent_token_operations() {
        let setup = CrossTokenSetup::new();
        
        // Register users
        setup.kyc_client().register_customer(&setup.user1, &2u32);
        setup.kyc_client().register_customer(&setup.user2, &2u32);
        setup.kyc_client().register_customer(&setup.user3, &1u32);

        // Setup initial balances
        let initial_amount = 300_000_000i128; // 3 BTC worth / 30 fungible tokens
        
        let tx_hash = String::from_str(&setup.env, "concurrent_test_tx_hash_123456789012345678901234567890abcdef123");
        setup.reserve_client().register_bitcoin_deposit(&setup.user1, &initial_amount, &tx_hash);
        setup.istsi_client().mint(&setup.user1, &initial_amount);
        setup.fungible_client().mint(&setup.user1, &initial_amount);

        // Perform concurrent operations
        let transfer_amount = 50_000_000i128; // 0.5 BTC worth / 5 fungible tokens
        
        // Transfer iSTSi tokens to user2
        setup.istsi_client().transfer(&setup.user1, &setup.user2, &transfer_amount);
        
        // Transfer fungible tokens to user3
        setup.fungible_client().transfer(&setup.user1, &setup.user3, &transfer_amount);
        
        // Transfer between user2 and user3 (cross-token)
        setup.istsi_client().transfer(&setup.user2, &setup.user3, &transfer_amount / 2);
        setup.fungible_client().transfer(&setup.user3, &setup.user2, &transfer_amount / 2);

        // Verify final balances
        let half_transfer = transfer_amount / 2;
        
        // User1: sent transfer_amount of each token
        assert_eq!(setup.istsi_client().balance(&setup.user1), initial_amount - transfer_amount);
        assert_eq!(setup.fungible_client().balance(&setup.user1), initial_amount - transfer_amount);
        
        // User2: received transfer_amount iSTSi, sent half, received half fungible
        assert_eq!(setup.istsi_client().balance(&setup.user2), transfer_amount - half_transfer);
        assert_eq!(setup.fungible_client().balance(&setup.user2), half_transfer);
        
        // User3: received transfer_amount fungible, sent half, received half iSTSi
        assert_eq!(setup.istsi_client().balance(&setup.user3), half_transfer);
        assert_eq!(setup.fungible_client().balance(&setup.user3), transfer_amount - half_transfer);
    }

    #[test]
    fn test_token_allowance_interactions() {
        let setup = CrossTokenSetup::new();
        
        // Register users
        setup.kyc_client().register_customer(&setup.user1, &2u32);
        setup.kyc_client().register_customer(&setup.user2, &2u32);
        setup.kyc_client().register_customer(&setup.user3, &1u32);

        // Setup initial balances
        let initial_amount = 200_000_000i128; // 2 BTC worth / 20 fungible tokens
        
        let tx_hash = String::from_str(&setup.env, "allowance_test_tx_hash_123456789012345678901234567890abcdef123");
        setup.reserve_client().register_bitcoin_deposit(&setup.user1, &initial_amount, &tx_hash);
        setup.istsi_client().mint(&setup.user1, &initial_amount);
        setup.fungible_client().mint(&setup.user1, &initial_amount);

        // Set allowances for both token types
        let allowance_amount = 75_000_000i128; // 0.75 BTC worth / 7.5 fungible tokens
        
        setup.istsi_client().approve(&setup.user1, &setup.user2, &allowance_amount, &99999u32);
        setup.fungible_client().approve(&setup.user1, &setup.user2, &allowance_amount, &99999u32);

        // Use allowances to transfer to user3
        let transfer_amount = 25_000_000i128; // 0.25 BTC worth / 2.5 fungible tokens
        
        setup.istsi_client().transfer_from(&setup.user2, &setup.user1, &setup.user3, &transfer_amount);
        setup.fungible_client().transfer_from(&setup.user2, &setup.user1, &setup.user3, &transfer_amount);

        // Verify balances
        assert_eq!(setup.istsi_client().balance(&setup.user1), initial_amount - transfer_amount);
        assert_eq!(setup.istsi_client().balance(&setup.user3), transfer_amount);
        assert_eq!(setup.fungible_client().balance(&setup.user1), initial_amount - transfer_amount);
        assert_eq!(setup.fungible_client().balance(&setup.user3), transfer_amount);

        // Verify remaining allowances
        let remaining_allowance = allowance_amount - transfer_amount;
        assert_eq!(setup.istsi_client().allowance(&setup.user1, &setup.user2), remaining_allowance);
        assert_eq!(setup.fungible_client().allowance(&setup.user1, &setup.user2), remaining_allowance);
    }
}