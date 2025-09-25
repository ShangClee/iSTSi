#![no_std]
use soroban_sdk::{
    Env, Address, String, panic_with_error
};

use crate::error_handling::{
    IntegrationError, CircuitBreaker, CircuitBreakerConfig, CircuitBreakerState,
    DataKey, get_default_circuit_breaker_config
};

/// Circuit breaker implementation for automatic failure protection
/// 
/// This module implements circuit breakers that automatically pause operations
/// when error rates exceed configured thresholds, preventing cascade failures
/// and allowing services time to recover.

pub struct CircuitBreakerManager;

impl CircuitBreakerManager {
    /// Initialize a circuit breaker for a service
    pub fn initialize_circuit_breaker(
        env: &Env,
        service_name: String,
        config: Option<CircuitBreakerConfig>
    ) {
        let cb_config = config.unwrap_or_else(|| {
            get_default_circuit_breaker_config(&service_name.to_string())
        });
        
        let circuit_breaker = CircuitBreaker {
            name: service_name.clone(),
            state: CircuitBreakerState::Closed,
            config: cb_config.clone(),
            failure_count: 0,
            success_count: 0,
            last_failure_time: 0,
            last_success_time: 0,
            state_changed_at: env.ledger().timestamp(),
            total_requests: 0,
            total_failures: 0,
        };
        
        env.storage().persistent().set(
            &DataKey::CircuitBreaker(service_name.clone()),
            &circuit_breaker
        );
        
        env.storage().persistent().set(
            &DataKey::CircuitBreakerConfig(service_name),
            &cb_config
        );
    }
    
    /// Check if a request should be allowed through the circuit breaker
    pub fn should_allow_request(
        env: &Env,
        service_name: String
    ) -> Result<(), IntegrationError> {
        let mut circuit_breaker: CircuitBreaker = env.storage().persistent()
            .get(&DataKey::CircuitBreaker(service_name.clone()))
            .unwrap_or_else(|| {
                // Initialize with default config if not found
                Self::initialize_circuit_breaker(env, service_name.clone(), None);
                env.storage().persistent()
                    .get(&DataKey::CircuitBreaker(service_name.clone()))
                    .unwrap()
            });
        
        if !circuit_breaker.config.enabled {
            return Ok(());
        }
        
        let current_time = env.ledger().timestamp();
        
        match circuit_breaker.state {
            CircuitBreakerState::Closed => {
                // Normal operation - allow request
                Ok(())
            },
            CircuitBreakerState::Open => {
                // Check if timeout has passed
                if current_time >= circuit_breaker.state_changed_at + circuit_breaker.config.timeout_ms {
                    // Transition to half-open
                    circuit_breaker.state = CircuitBreakerState::HalfOpen;
                    circuit_breaker.state_changed_at = current_time;
                    circuit_breaker.success_count = 0;
                    
                    env.storage().persistent().set(
                        &DataKey::CircuitBreaker(service_name),
                        &circuit_breaker
                    );
                    
                    Ok(())
                } else {
                    // Still in open state - reject request
                    Err(IntegrationError::CircuitBreakerOpen)
                }
            },
            CircuitBreakerState::HalfOpen => {
                // Allow limited requests to test if service has recovered
                Ok(())
            }
        }
    }
    
    /// Record a successful request
    pub fn record_success(env: &Env, service_name: String) {
        let mut circuit_breaker: CircuitBreaker = env.storage().persistent()
            .get(&DataKey::CircuitBreaker(service_name.clone()))
            .unwrap_or_else(|| {
                Self::initialize_circuit_breaker(env, service_name.clone(), None);
                env.storage().persistent()
                    .get(&DataKey::CircuitBreaker(service_name.clone()))
                    .unwrap()
            });
        
        let current_time = env.ledger().timestamp();
        
        circuit_breaker.total_requests += 1;
        circuit_breaker.last_success_time = current_time;
        
        match circuit_breaker.state {
            CircuitBreakerState::Closed => {
                // Reset failure count on success
                circuit_breaker.failure_count = 0;
            },
            CircuitBreakerState::HalfOpen => {
                circuit_breaker.success_count += 1;
                
                // Check if we have enough successes to close the circuit
                if circuit_breaker.success_count >= circuit_breaker.config.success_threshold {
                    circuit_breaker.state = CircuitBreakerState::Closed;
                    circuit_breaker.state_changed_at = current_time;
                    circuit_breaker.failure_count = 0;
                    circuit_breaker.success_count = 0;
                }
            },
            CircuitBreakerState::Open => {
                // This shouldn't happen if should_allow_request is used correctly
                // But handle it gracefully
            }
        }
        
        env.storage().persistent().set(
            &DataKey::CircuitBreaker(service_name),
            &circuit_breaker
        );
    }
    
    /// Record a failed request
    pub fn record_failure(env: &Env, service_name: String, error: IntegrationError) {
        let mut circuit_breaker: CircuitBreaker = env.storage().persistent()
            .get(&DataKey::CircuitBreaker(service_name.clone()))
            .unwrap_or_else(|| {
                Self::initialize_circuit_breaker(env, service_name.clone(), None);
                env.storage().persistent()
                    .get(&DataKey::CircuitBreaker(service_name.clone()))
                    .unwrap()
            });
        
        // Only count failures that should trigger circuit breaker
        if !error.should_trigger_circuit_breaker() {
            return;
        }
        
        let current_time = env.ledger().timestamp();
        
        circuit_breaker.total_requests += 1;
        circuit_breaker.total_failures += 1;
        circuit_breaker.last_failure_time = current_time;
        
        match circuit_breaker.state {
            CircuitBreakerState::Closed => {
                circuit_breaker.failure_count += 1;
                
                // Check if we should open the circuit
                if circuit_breaker.failure_count >= circuit_breaker.config.failure_threshold {
                    circuit_breaker.state = CircuitBreakerState::Open;
                    circuit_breaker.state_changed_at = current_time;
                }
            },
            CircuitBreakerState::HalfOpen => {
                // Any failure in half-open state immediately opens the circuit
                circuit_breaker.state = CircuitBreakerState::Open;
                circuit_breaker.state_changed_at = current_time;
                circuit_breaker.failure_count += 1;
                circuit_breaker.success_count = 0;
            },
            CircuitBreakerState::Open => {
                // Already open, just increment counters
                circuit_breaker.failure_count += 1;
            }
        }
        
        env.storage().persistent().set(
            &DataKey::CircuitBreaker(service_name),
            &circuit_breaker
        );
    }
    
    /// Get circuit breaker state
    pub fn get_circuit_breaker_state(env: &Env, service_name: String) -> Option<CircuitBreaker> {
        env.storage().persistent().get(&DataKey::CircuitBreaker(service_name))
    }
    
    /// Manually open a circuit breaker (admin function)
    pub fn manual_open(
        env: &Env,
        caller: Address,
        service_name: String
    ) -> Result<(), IntegrationError> {
        caller.require_auth();
        
        let mut circuit_breaker: CircuitBreaker = env.storage().persistent()
            .get(&DataKey::CircuitBreaker(service_name.clone()))
            .unwrap_or_else(|| {
                Self::initialize_circuit_breaker(env, service_name.clone(), None);
                env.storage().persistent()
                    .get(&DataKey::CircuitBreaker(service_name.clone()))
                    .unwrap()
            });
        
        circuit_breaker.state = CircuitBreakerState::Open;
        circuit_breaker.state_changed_at = env.ledger().timestamp();
        
        env.storage().persistent().set(
            &DataKey::CircuitBreaker(service_name),
            &circuit_breaker
        );
        
        Ok(())
    }
    
    /// Manually close a circuit breaker (admin function)
    pub fn manual_close(
        env: &Env,
        caller: Address,
        service_name: String
    ) -> Result<(), IntegrationError> {
        caller.require_auth();
        
        let mut circuit_breaker: CircuitBreaker = env.storage().persistent()
            .get(&DataKey::CircuitBreaker(service_name.clone()))
            .unwrap_or_else(|| {
                Self::initialize_circuit_breaker(env, service_name.clone(), None);
                env.storage().persistent()
                    .get(&DataKey::CircuitBreaker(service_name.clone()))
                    .unwrap()
            });
        
        circuit_breaker.state = CircuitBreakerState::Closed;
        circuit_breaker.state_changed_at = env.ledger().timestamp();
        circuit_breaker.failure_count = 0;
        circuit_breaker.success_count = 0;
        
        env.storage().persistent().set(
            &DataKey::CircuitBreaker(service_name),
            &circuit_breaker
        );
        
        Ok(())
    }
    
    /// Update circuit breaker configuration (admin function)
    pub fn update_config(
        env: &Env,
        caller: Address,
        service_name: String,
        config: CircuitBreakerConfig
    ) -> Result<(), IntegrationError> {
        caller.require_auth();
        
        env.storage().persistent().set(
            &DataKey::CircuitBreakerConfig(service_name.clone()),
            &config
        );
        
        // Update the circuit breaker with new config
        let mut circuit_breaker: CircuitBreaker = env.storage().persistent()
            .get(&DataKey::CircuitBreaker(service_name.clone()))
            .unwrap_or_else(|| {
                Self::initialize_circuit_breaker(env, service_name.clone(), Some(config.clone()));
                env.storage().persistent()
                    .get(&DataKey::CircuitBreaker(service_name.clone()))
                    .unwrap()
            });
        
        circuit_breaker.config = config;
        
        env.storage().persistent().set(
            &DataKey::CircuitBreaker(service_name),
            &circuit_breaker
        );
        
        Ok(())
    }
    
    /// Get circuit breaker configuration
    pub fn get_config(env: &Env, service_name: String) -> Option<CircuitBreakerConfig> {
        env.storage().persistent().get(&DataKey::CircuitBreakerConfig(service_name))
    }
    
    /// Reset circuit breaker statistics (admin function)
    pub fn reset_statistics(
        env: &Env,
        caller: Address,
        service_name: String
    ) -> Result<(), IntegrationError> {
        caller.require_auth();
        
        let mut circuit_breaker: CircuitBreaker = env.storage().persistent()
            .get(&DataKey::CircuitBreaker(service_name.clone()))
            .unwrap_or_else(|| {
                Self::initialize_circuit_breaker(env, service_name.clone(), None);
                env.storage().persistent()
                    .get(&DataKey::CircuitBreaker(service_name.clone()))
                    .unwrap()
            });
        
        circuit_breaker.failure_count = 0;
        circuit_breaker.success_count = 0;
        circuit_breaker.total_requests = 0;
        circuit_breaker.total_failures = 0;
        circuit_breaker.last_failure_time = 0;
        circuit_breaker.last_success_time = 0;
        
        // Reset to closed state
        circuit_breaker.state = CircuitBreakerState::Closed;
        circuit_breaker.state_changed_at = env.ledger().timestamp();
        
        env.storage().persistent().set(
            &DataKey::CircuitBreaker(service_name),
            &circuit_breaker
        );
        
        Ok(())
    }
    
    /// Check if any circuit breakers are open (system health check)
    pub fn get_open_circuit_breakers(env: &Env) -> Vec<String> {
        let mut open_breakers = Vec::new(env);
        
        // In a real implementation, we'd maintain a list of all circuit breakers
        // For now, check the common ones
        let service_names = vec![
            &env,
            String::from_str(env, "kyc_registry"),
            String::from_str(env, "reserve_manager"),
            String::from_str(env, "bitcoin_network"),
            String::from_str(env, "oracle_service"),
        ];
        
        for service_name in service_names.iter() {
            if let Some(circuit_breaker) = env.storage().persistent()
                .get::<DataKey, CircuitBreaker>(&DataKey::CircuitBreaker(service_name.clone())) {
                
                if circuit_breaker.state == CircuitBreakerState::Open {
                    open_breakers.push_back(service_name.clone());
                }
            }
        }
        
        open_breakers
    }
    
    /// Get failure rate for a service
    pub fn get_failure_rate(env: &Env, service_name: String) -> u64 {
        if let Some(circuit_breaker) = env.storage().persistent()
            .get::<DataKey, CircuitBreaker>(&DataKey::CircuitBreaker(service_name)) {
            
            if circuit_breaker.total_requests == 0 {
                return 0;
            }
            
            // Return failure rate as percentage (0-100)
            (circuit_breaker.total_failures * 100) / circuit_breaker.total_requests
        } else {
            0
        }
    }
}

/// Utility functions for circuit breaker operations
pub mod circuit_breaker_utils {
    use super::*;
    
    /// Execute an operation with circuit breaker protection
    pub fn execute_with_circuit_breaker<F, R>(
        env: &Env,
        service_name: String,
        operation: F
    ) -> Result<R, IntegrationError>
    where
        F: Fn() -> Result<R, IntegrationError>,
    {
        // Check if request should be allowed
        CircuitBreakerManager::should_allow_request(env, service_name.clone())?;
        
        // Execute the operation
        match operation() {
            Ok(result) => {
                // Record success
                CircuitBreakerManager::record_success(env, service_name);
                Ok(result)
            },
            Err(error) => {
                // Record failure
                CircuitBreakerManager::record_failure(env, service_name, error);
                Err(error)
            }
        }
    }
    
    /// Check if system is healthy (no open circuit breakers)
    pub fn is_system_healthy(env: &Env) -> bool {
        let open_breakers = CircuitBreakerManager::get_open_circuit_breakers(env);
        open_breakers.is_empty()
    }
    
    /// Get system health summary
    pub fn get_system_health_summary(env: &Env) -> String {
        let open_breakers = CircuitBreakerManager::get_open_circuit_breakers(env);
        
        if open_breakers.is_empty() {
            String::from_str(env, "All services healthy")
        } else {
            // In a real implementation, we'd format this properly
            String::from_str(env, "Some services degraded")
        }
    }
}