use soroban_sdk::{testutils::Address as _, Address, Env};

// Import the KYC Registry contract
mod kyc_registry {
    soroban_sdk::contractimport!(
        file = "../../contracts/kyc_registry/target/wasm32-unknown-unknown/release/kyc_registry.wasm"
    );
}

use kyc_registry::{Client as KycClient, KycTier, KycStatus};

struct KycTestSetup {
    env: Env,
    admin: Address,
    user1: Address,
    user2: Address,
    user3: Address,
    kyc_contract: Address,
    kyc_client: KycClient,
}

impl KycTestSetup {
    fn new() -> Self {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let user1 = Address::generate(&env);
        let user2 = Address::generate(&env);
        let user3 = Address::generate(&env);

        let kyc_contract = env.register_contract_wasm(None, kyc_registry::WASM);
        let kyc_client = KycClient::new(&env, &kyc_contract);

        // Initialize the contract
        kyc_client.initialize(&admin);

        Self {
            env,
            admin,
            user1,
            user2,
            user3,
            kyc_contract,
            kyc_client,
        }
    }
}

#[test]
fn test_kyc_registry_initialization() {
    let setup = KycTestSetup::new();

    // Verify admin is set correctly
    assert_eq!(setup.kyc_client.get_admin(), setup.admin);

    // Verify registry is enabled by default
    assert!(setup.kyc_client.is_registry_enabled());

    // Verify initial tier limits are set
    let tier1_limits = setup.kyc_client.get_tier_limits(&KycTier::Tier1);
    assert!(tier1_limits.daily_limit > 0);
    assert!(tier1_limits.monthly_limit > 0);
}

#[test]
fn test_customer_registration() {
    let setup = KycTestSetup::new();

    // Register a customer with tier 1
    setup.kyc_client.register_customer(&setup.user1, &KycTier::Tier1);

    // Verify registration
    assert!(setup.kyc_client.is_customer_registered(&setup.user1));
    assert_eq!(setup.kyc_client.get_customer_tier(&setup.user1), KycTier::Tier1);
    assert_eq!(setup.kyc_client.get_customer_status(&setup.user1), KycStatus::Approved);
}

#[test]
fn test_multiple_customer_registrations() {
    let setup = KycTestSetup::new();

    // Register multiple customers with different tiers
    setup.kyc_client.register_customer(&setup.user1, &KycTier::Tier1);
    setup.kyc_client.register_customer(&setup.user2, &KycTier::Tier2);
    setup.kyc_client.register_customer(&setup.user3, &KycTier::Tier3);

    // Verify all registrations
    assert_eq!(setup.kyc_client.get_customer_tier(&setup.user1), KycTier::Tier1);
    assert_eq!(setup.kyc_client.get_customer_tier(&setup.user2), KycTier::Tier2);
    assert_eq!(setup.kyc_client.get_customer_tier(&setup.user3), KycTier::Tier3);
}

#[test]
fn test_tier_updates() {
    let setup = KycTestSetup::new();

    // Register customer with tier 1
    setup.kyc_client.register_customer(&setup.user1, &KycTier::Tier1);
    assert_eq!(setup.kyc_client.get_customer_tier(&setup.user1), KycTier::Tier1);

    // Upgrade to tier 2
    setup.kyc_client.update_customer_tier(&setup.user1, &KycTier::Tier2);
    assert_eq!(setup.kyc_client.get_customer_tier(&setup.user1), KycTier::Tier2);

    // Upgrade to tier 3
    setup.kyc_client.update_customer_tier(&setup.user1, &KycTier::Tier3);
    assert_eq!(setup.kyc_client.get_customer_tier(&setup.user1), KycTier::Tier3);
}

#[test]
fn test_compliance_verification() {
    let setup = KycTestSetup::new();

    // Register customer with tier 1
    setup.kyc_client.register_customer(&setup.user1, &KycTier::Tier1);

    // Test amounts within tier 1 limits
    let small_amount = 1_000_000i128; // 0.01 BTC
    assert!(setup.kyc_client.is_approved(&setup.user1, &small_amount));

    // Test amounts exceeding tier 1 limits
    let large_amount = 1_000_000_000i128; // 10 BTC (likely exceeds tier 1)
    let tier1_limits = setup.kyc_client.get_tier_limits(&KycTier::Tier1);
    if large_amount > tier1_limits.daily_limit {
        assert!(!setup.kyc_client.is_approved(&setup.user1, &large_amount));
    }
}

#[test]
fn test_tier_limit_configuration() {
    let setup = KycTestSetup::new();

    // Set custom tier limits
    let daily_limit = 500_000_000i128; // 5 BTC
    let monthly_limit = 10_000_000_000i128; // 100 BTC
    
    setup.kyc_client.set_tier_limits(&KycTier::Tier2, &daily_limit, &monthly_limit);

    // Verify limits are set
    let limits = setup.kyc_client.get_tier_limits(&KycTier::Tier2);
    assert_eq!(limits.daily_limit, daily_limit);
    assert_eq!(limits.monthly_limit, monthly_limit);
}

#[test]
fn test_customer_status_management() {
    let setup = KycTestSetup::new();

    // Register customer
    setup.kyc_client.register_customer(&setup.user1, &KycTier::Tier1);
    assert_eq!(setup.kyc_client.get_customer_status(&setup.user1), KycStatus::Approved);

    // Suspend customer
    setup.kyc_client.update_customer_status(&setup.user1, &KycStatus::Suspended);
    assert_eq!(setup.kyc_client.get_customer_status(&setup.user1), KycStatus::Suspended);

    // Suspended customer should not be approved for operations
    let amount = 1_000_000i128;
    assert!(!setup.kyc_client.is_approved(&setup.user1, &amount));

    // Reactivate customer
    setup.kyc_client.update_customer_status(&setup.user1, &KycStatus::Approved);
    assert_eq!(setup.kyc_client.get_customer_status(&setup.user1), KycStatus::Approved);
    assert!(setup.kyc_client.is_approved(&setup.user1, &amount));
}

#[test]
fn test_registry_enable_disable() {
    let setup = KycTestSetup::new();

    // Registry should be enabled by default
    assert!(setup.kyc_client.is_registry_enabled());

    // Disable registry
    setup.kyc_client.toggle_registry_enabled();
    assert!(!setup.kyc_client.is_registry_enabled());

    // When disabled, all operations should be approved (bypass mode)
    let unregistered_user = Address::generate(&setup.env);
    let amount = 1_000_000_000i128;
    assert!(setup.kyc_client.is_approved(&unregistered_user, &amount));

    // Re-enable registry
    setup.kyc_client.toggle_registry_enabled();
    assert!(setup.kyc_client.is_registry_enabled());

    // Now unregistered user should not be approved
    assert!(!setup.kyc_client.is_approved(&unregistered_user, &amount));
}

#[test]
fn test_batch_operations() {
    let setup = KycTestSetup::new();

    // Create multiple users
    let users = vec![
        Address::generate(&setup.env),
        Address::generate(&setup.env),
        Address::generate(&setup.env),
    ];

    // Batch register customers
    for (i, user) in users.iter().enumerate() {
        let tier = match i {
            0 => KycTier::Tier1,
            1 => KycTier::Tier2,
            _ => KycTier::Tier3,
        };
        setup.kyc_client.register_customer(user, &tier);
    }

    // Verify all registrations
    for (i, user) in users.iter().enumerate() {
        assert!(setup.kyc_client.is_customer_registered(user));
        let expected_tier = match i {
            0 => KycTier::Tier1,
            1 => KycTier::Tier2,
            _ => KycTier::Tier3,
        };
        assert_eq!(setup.kyc_client.get_customer_tier(user), expected_tier);
    }
}

#[test]
fn test_operation_approval_with_limits() {
    let setup = KycTestSetup::new();

    // Set specific limits for tier 1
    let daily_limit = 100_000_000i128; // 1 BTC
    let monthly_limit = 1_000_000_000i128; // 10 BTC
    
    setup.kyc_client.set_tier_limits(&KycTier::Tier1, &daily_limit, &monthly_limit);
    setup.kyc_client.register_customer(&setup.user1, &KycTier::Tier1);

    // Test amount within daily limit
    let within_limit = 50_000_000i128; // 0.5 BTC
    assert!(setup.kyc_client.is_approved(&setup.user1, &within_limit));

    // Test amount equal to daily limit
    assert!(setup.kyc_client.is_approved(&setup.user1, &daily_limit));

    // Test amount exceeding daily limit
    let exceeding_limit = daily_limit + 1;
    assert!(!setup.kyc_client.is_approved(&setup.user1, &exceeding_limit));
}

#[test]
#[should_panic(expected = "unauthorized")]
fn test_unauthorized_admin_operations() {
    let setup = KycTestSetup::new();
    let unauthorized_user = Address::generate(&setup.env);

    // Try to register customer as non-admin (should panic)
    setup.env.mock_all_auths_allowing_non_root_auth();
    setup.kyc_client.mock_auths(&[soroban_sdk::testutils::MockAuth {
        address: &unauthorized_user,
        invoke: &soroban_sdk::testutils::MockAuthInvoke {
            contract: &setup.kyc_contract,
            fn_name: "register_customer",
            args: (&setup.user1, &KycTier::Tier1).into_val(&setup.env),
            sub_invokes: &[],
        },
    }]);

    setup.kyc_client.register_customer(&setup.user1, &KycTier::Tier1);
}

#[test]
fn test_customer_data_retrieval() {
    let setup = KycTestSetup::new();

    // Register customer
    setup.kyc_client.register_customer(&setup.user1, &KycTier::Tier2);

    // Get customer info
    let customer_info = setup.kyc_client.get_customer_info(&setup.user1);
    assert_eq!(customer_info.tier, KycTier::Tier2);
    assert_eq!(customer_info.status, KycStatus::Approved);
    assert!(customer_info.registered_at > 0);

    // Test non-existent customer
    let non_existent = Address::generate(&setup.env);
    let result = std::panic::catch_unwind(|| {
        setup.kyc_client.get_customer_info(&non_existent);
    });
    assert!(result.is_err());
}

#[test]
fn test_event_emission() {
    let setup = KycTestSetup::new();

    // Register customer and check events
    setup.kyc_client.register_customer(&setup.user1, &KycTier::Tier1);

    // In a real test, we would check for emitted events
    // This is a placeholder for event verification logic
    let events = setup.env.events().all();
    assert!(!events.is_empty());

    // Look for CustomerRegistered event
    let registration_events: Vec<_> = events
        .iter()
        .filter(|event| {
            event.topics.len() > 0 && 
            event.topics[0] == soroban_sdk::symbol_short!("reg")
        })
        .collect();
    
    assert!(!registration_events.is_empty());
}

#[test]
fn test_integration_compliance_verification() {
    let setup = KycTestSetup::new();

    // Register customers with different tiers
    setup.kyc_client.register_customer(&setup.user1, &KycTier::Tier1);
    setup.kyc_client.register_customer(&setup.user2, &KycTier::Tier2);

    // Test batch compliance check
    let users = vec![setup.user1.clone(), setup.user2.clone()];
    let amounts = vec![10_000_000i128, 100_000_000i128]; // 0.1 BTC, 1 BTC

    for (user, amount) in users.iter().zip(amounts.iter()) {
        let is_approved = setup.kyc_client.is_approved(user, amount);
        // Tier 1 and Tier 2 should both approve these amounts
        assert!(is_approved);
    }
}