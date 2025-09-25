// Integration Testing Suite Module
// This module contains comprehensive end-to-end tests for all integrated workflows

pub mod end_to_end_tests;
pub mod failure_scenario_tests;
pub mod performance_tests;
pub mod security_tests;
pub mod load_tests;
pub mod test_utils;

// Re-export test utilities for easy access
pub use test_utils::*;

#[cfg(test)]
mod integration_test_runner {
    use super::*;
    use soroban_sdk::{Env, testutils::Address as _};
    
    /// Integration test environment setup
    pub struct IntegrationTestEnv {
        pub env: Env,
        pub admin: soroban_sdk::Address,
        pub user1: soroban_sdk::Address,
        pub user2: soroban_sdk::Address,
        pub operator: soroban_sdk::Address,
        pub compliance_officer: soroban_sdk::Address,
    }
    
    impl IntegrationTestEnv {
        pub fn new() -> Self {
            let env = Env::default();
            env.mock_all_auths();
            
            Self {
                admin: soroban_sdk::Address::generate(&env),
                user1: soroban_sdk::Address::generate(&env),
                user2: soroban_sdk::Address::generate(&env),
                operator: soroban_sdk::Address::generate(&env),
                compliance_officer: soroban_sdk::Address::generate(&env),
                env,
            }
        }
    }
}