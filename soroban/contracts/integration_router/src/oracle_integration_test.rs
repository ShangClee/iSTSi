#![cfg(test)]

use super::*;
use soroban_sdk::{
    testutils::{Address as TestAddress, Ledger, LedgerInfo},
    Address, Env, String, BytesN
};

fn create_test_env() -> Env {
    Env::default()
}

fn setup_test_addresses(env: &Env) -> (Address, Address, Address, Address, Address) {
    let admin = Address::generate(env);
    let oracle = Address::generate(env);
    let token_a = Address::generate(env);
    let token_b = Address::generate(env);
    let user = Address::generate(env);
    (admin, oracle, token_a, token_b, user)
}

fn initialize_router_with_oracle(env: &Env, admin: &Address, oracle: &Address, token_a: &Address, token_b: &Address) {
    let kyc_registry = Address::generate(env);
    let istsi_token = Address::generate(env);
    let fungible_token = Address::generate(env);
    let reserve_manager = Address::generate(env);

    // Initialize router
    IntegrationRouter::initialize(
        env.clone(),
        admin.clone(),
        kyc_registry,
        istsi_token,
        fungible_token,
        reserve_manager,
    );

    // Configure oracle
    IntegrationRouter::configure_oracle(
        env.clone(),
        admin.clone(),
        token_a.clone(),
        token_b.clone(),
        oracle.clone(),
        300, // 5 minutes update frequency
        500, // 5% max deviation
        10000, // 1:1 fallback rate
    ).unwrap();
}

#[test]
fn test_configure_oracle_success() {
    let env = create_test_env();
    let (admin, oracle, token_a, token_b, _user) = setup_test_addresses(&env);

    initialize_router_with_oracle(&env, &admin, &oracle, &token_a, &token_b);

    // Verify oracle configuration was stored
    let oracle_config: OracleConfig = env.storage().persistent()
        .get(&DataKey::OracleConfig)
        .unwrap();

    assert_eq!(oracle_config.oracle_address, oracle);
    assert_eq!(oracle_config.update_frequency, 300);
    assert_eq!(oracle_config.max_price_deviation, 500);
    assert_eq!(oracle_config.fallback_rate, 10000);
    assert_eq!(oracle_config.enabled, true);
}

#[test]
fn test_configure_oracle_unauthorized() {
    let env = create_test_env();
    let (admin, oracle, token_a, token_b, user) = setup_test_addresses(&env);

    initialize_router_with_oracle(&env, &admin, &oracle, &token_a, &token_b);

    // Try to configure oracle as non-admin user
    let result = IntegrationRouter::configure_oracle(
        env.clone(),
        user, // Non-admin user
        token_a,
        token_b,
        oracle,
        300,
        500,
        10000,
    );

    assert_eq!(result, Err(IntegrationError::Unauthorized));
}

#[test]
fn test_get_fallback_rate_when_oracle_fails() {
    let env = create_test_env();
    let (admin, oracle, token_a, token_b, _user) = setup_test_addresses(&env);

    initialize_router_with_oracle(&env, &admin, &oracle, &token_a, &token_b);

    // Get exchange rate (should use fallback since oracle is not implemented)
    let rate = IntegrationRouter::get_exchange_rate(
        env.clone(),
        token_a.clone(),
        token_b.clone(),
    ).unwrap();

    assert_eq!(rate.rate, 10000); // Fallback rate
    assert_eq!(rate.oracle_source, String::from_str(&env, "fallback"));
    assert_eq!(rate.fee_rate, 50); // Higher fee for fallback
}

#[test]
fn test_calculate_exchange_amount_basic() {
    let env = create_test_env();
    let (admin, oracle, token_a, token_b, _user) = setup_test_addresses(&env);

    initialize_router_with_oracle(&env, &admin, &oracle, &token_a, &token_b);

    let from_amount = 1000u64;
    let max_slippage = 100u64; // 1% max slippage

    let quote = IntegrationRouter::calculate_exchange_amount(
        env.clone(),
        token_a.clone(),
        token_b.clone(),
        from_amount,
        max_slippage,
    ).unwrap();

    assert_eq!(quote.from_amount, from_amount);
    assert_eq!(quote.exchange_rate, 10000); // 1:1 fallback rate
    
    // Calculate expected amounts
    let expected_fee = (from_amount * 50) / 10000; // 0.5% fee for fallback
    let expected_to_amount = ((from_amount - expected_fee) * 10000) / 10000;
    
    assert_eq!(quote.fee_amount, expected_fee);
    assert_eq!(quote.to_amount, expected_to_amount);
}

#[test]
fn test_calculate_exchange_amount_with_price_impact() {
    let env = create_test_env();
    let (admin, oracle, token_a, token_b, _user) = setup_test_addresses(&env);

    initialize_router_with_oracle(&env, &admin, &oracle, &token_a, &token_b);

    // Large amount to trigger price impact
    let from_amount = 5_000_000u64; // 5M units
    let max_slippage = 1000u64; // 10% max slippage

    let quote = IntegrationRouter::calculate_exchange_amount(
        env.clone(),
        token_a.clone(),
        token_b.clone(),
        from_amount,
        max_slippage,
    ).unwrap();

    assert_eq!(quote.from_amount, from_amount);
    
    // Should have price impact for large trades
    let expected_impact = ((from_amount - 1_000_000) / 1_000_000) * 10; // 0.1% per 1M excess
    assert_eq!(quote.price_impact, expected_impact.min(500)); // Capped at 5%
}

#[test]
fn test_update_oracle_config() {
    let env = create_test_env();
    let (admin, oracle, token_a, token_b, _user) = setup_test_addresses(&env);

    initialize_router_with_oracle(&env, &admin, &oracle, &token_a, &token_b);

    let new_oracle = Address::generate(&env);

    // Update oracle configuration
    IntegrationRouter::update_oracle_config(
        env.clone(),
        admin,
        Some(new_oracle.clone()),
        Some(600), // 10 minutes
        Some(1000), // 10% max deviation
        Some(9500), // 0.95:1 fallback rate
        Some(false), // Disable oracle
    ).unwrap();

    // Verify updates
    let oracle_config: OracleConfig = env.storage().persistent()
        .get(&DataKey::OracleConfig)
        .unwrap();

    assert_eq!(oracle_config.oracle_address, new_oracle);
    assert_eq!(oracle_config.update_frequency, 600);
    assert_eq!(oracle_config.max_price_deviation, 1000);
    assert_eq!(oracle_config.fallback_rate, 9500);
    assert_eq!(oracle_config.enabled, false);
}

#[test]
fn test_get_oracle_status_disabled() {
    let env = create_test_env();
    let (admin, oracle, token_a, token_b, _user) = setup_test_addresses(&env);

    initialize_router_with_oracle(&env, &admin, &oracle, &token_a, &token_b);

    // Disable oracle
    IntegrationRouter::update_oracle_config(
        env.clone(),
        admin,
        None,
        None,
        None,
        None,
        Some(false),
    ).unwrap();

    let status = IntegrationRouter::get_oracle_status(env.clone()).unwrap();

    assert_eq!(status.enabled, false);
    assert_eq!(status.health_status, OracleHealthStatus::Offline);
    assert_eq!(status.uptime_percentage, 0);
}

#[test]
fn test_get_oracle_status_enabled() {
    let env = create_test_env();
    let (admin, oracle, token_a, token_b, _user) = setup_test_addresses(&env);

    initialize_router_with_oracle(&env, &admin, &oracle, &token_a, &token_b);

    let status = IntegrationRouter::get_oracle_status(env.clone()).unwrap();

    assert_eq!(status.enabled, true);
    assert_eq!(status.oracle_address, oracle);
    // Health status will be Degraded since we don't have a real oracle implementation
    assert_eq!(status.health_status, OracleHealthStatus::Degraded);
}

#[test]
fn test_token_pair_key_generation() {
    let env = create_test_env();
    let token_a = Address::generate(&env);
    let token_b = Address::generate(&env);

    // Keys should be the same regardless of order
    let key1 = IntegrationRouter::get_token_pair_key(&env, &token_a, &token_b);
    let key2 = IntegrationRouter::get_token_pair_key(&env, &token_b, &token_a);

    assert_eq!(key1, key2);
}

#[test]
fn test_quote_id_generation() {
    let env = create_test_env();

    // Set different ledger states to ensure different quote IDs
    env.ledger().set(LedgerInfo {
        timestamp: 1000,
        protocol_version: 1,
        sequence_number: 100,
        network_id: [0; 32],
        base_reserve: 10,
        min_temp_entry_ttl: 10,
        min_persistent_entry_ttl: 10,
        max_entry_ttl: 6312000,
    });

    let quote_id1 = IntegrationRouter::generate_quote_id(&env);

    env.ledger().set(LedgerInfo {
        timestamp: 2000,
        protocol_version: 1,
        sequence_number: 200,
        network_id: [0; 32],
        base_reserve: 10,
        min_temp_entry_ttl: 10,
        min_persistent_entry_ttl: 10,
        max_entry_ttl: 6312000,
    });

    let quote_id2 = IntegrationRouter::generate_quote_id(&env);

    // Quote IDs should be different
    assert_ne!(quote_id1, quote_id2);
}

#[test]
fn test_price_impact_calculation() {
    let env = create_test_env();
    let token_a = Address::generate(&env);
    let token_b = Address::generate(&env);

    // Small amount - no price impact
    let impact1 = IntegrationRouter::calculate_price_impact(&env, &token_a, &token_b, 500_000).unwrap();
    assert_eq!(impact1, 0);

    // Exactly at threshold - no price impact
    let impact2 = IntegrationRouter::calculate_price_impact(&env, &token_a, &token_b, 1_000_000).unwrap();
    assert_eq!(impact2, 0);

    // Above threshold - should have price impact
    let impact3 = IntegrationRouter::calculate_price_impact(&env, &token_a, &token_b, 2_000_000).unwrap();
    assert_eq!(impact3, 10); // 0.1% for 1M excess

    // Very large amount - should be capped at 5%
    let impact4 = IntegrationRouter::calculate_price_impact(&env, &token_a, &token_b, 100_000_000).unwrap();
    assert_eq!(impact4, 500); // Capped at 5%
}

#[test]
fn test_slippage_protection() {
    let env = create_test_env();
    let (admin, oracle, token_a, token_b, _user) = setup_test_addresses(&env);

    initialize_router_with_oracle(&env, &admin, &oracle, &token_a, &token_b);

    let from_amount = 1000u64;
    let max_slippage = 10u64; // Very low slippage tolerance (0.1%)

    // This should succeed since we're using fallback rate with no slippage
    let result = IntegrationRouter::calculate_exchange_amount(
        env.clone(),
        token_a.clone(),
        token_b.clone(),
        from_amount,
        max_slippage,
    );

    assert!(result.is_ok());
}

#[test]
fn test_oracle_rate_validation_deviation() {
    let env = create_test_env();
    
    let oracle_config = OracleConfig {
        oracle_address: Address::generate(&env),
        update_frequency: 300,
        max_price_deviation: 500, // 5% max deviation
        fallback_rate: 10000, // 1:1 rate
        enabled: true,
    };

    // Rate within acceptable deviation
    let good_rate = OracleRateData {
        rate: 10400, // 4% higher than fallback
        timestamp: env.ledger().timestamp(),
        confidence: 10000,
    };

    let result1 = IntegrationRouter::validate_oracle_rate(&env, &good_rate, &oracle_config);
    assert!(result1.is_ok());

    // Rate with excessive deviation
    let bad_rate = OracleRateData {
        rate: 11000, // 10% higher than fallback (exceeds 5% limit)
        timestamp: env.ledger().timestamp(),
        confidence: 10000,
    };

    let result2 = IntegrationRouter::validate_oracle_rate(&env, &bad_rate, &oracle_config);
    assert_eq!(result2, Err(IntegrationError::ContractCallFailed));
}

#[test]
fn test_oracle_rate_validation_staleness() {
    let env = create_test_env();
    
    let oracle_config = OracleConfig {
        oracle_address: Address::generate(&env),
        update_frequency: 300, // 5 minutes
        max_price_deviation: 500,
        fallback_rate: 10000,
        enabled: true,
    };

    let current_time = env.ledger().timestamp();

    // Fresh rate
    let fresh_rate = OracleRateData {
        rate: 10000,
        timestamp: current_time - 100, // 100 seconds ago
        confidence: 10000,
    };

    let result1 = IntegrationRouter::validate_oracle_rate(&env, &fresh_rate, &oracle_config);
    assert!(result1.is_ok());

    // Stale rate (older than 2x update frequency)
    let stale_rate = OracleRateData {
        rate: 10000,
        timestamp: current_time - 700, // 700 seconds ago (exceeds 2x300 = 600)
        confidence: 10000,
    };

    let result2 = IntegrationRouter::validate_oracle_rate(&env, &stale_rate, &oracle_config);
    assert_eq!(result2, Err(IntegrationError::ContractCallFailed));
}