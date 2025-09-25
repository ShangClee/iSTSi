#![no_std]
use soroban_sdk::{
    Env, Address, BytesN, Vec, Map, String, panic_with_error
};

use crate::error_handling::{
    IntegrationError, ErrorContext, RetryConfig, RetryState, DataKey,
    get_default_retry_config
};

/// Retry logic implementation with exponential backoff
/// 
/// This module provides automatic retry functionality for transient failures
/// with configurable exponential backoff, jitter, and retry policies.

pub struct RetryManager;

impl RetryManager {
    /// Initialize retry configuration for an operation type
    pub fn initialize_retry_config(
        env: &Env,
        operation_type: String,
        config: Option<RetryConfig>
    ) {
        let retry_config = config.unwrap_or_else(|| {
            get_default_retry_config(&operation_type.to_string())
        });
        
        env.storage().persistent().set(
            &DataKey::RetryConfig(operation_type),
            &retry_config
        );
    }
    
    /// Check if an operation should be retried
    pub fn should_retry(
        env: &Env,
        operation_id: BytesN<32>,
        error: IntegrationError,
        operation_type: String
    ) -> bool {
        // Get retry configuration
        let retry_config: RetryConfig = env.storage().persistent()
            .get(&DataKey::RetryConfig(operation_type))
            .unwrap_or_else(|| get_default_retry_config("default"));
        
        // Check if error type is retryable
        if !retry_config.retry_on_errors.contains(&error) {
            return false;
        }
        
        // Get current retry state
        let retry_state: Option<RetryState> = env.storage().temporary()
            .get(&DataKey::RetryState(operation_id.clone()));
        
        match retry_state {
            Some(state) => {
                // Check if we've exceeded max retries
                state.current_attempt < retry_config.max_retries
            },
            None => {
                // First failure, can retry
                true
            }
        }
    }
    
    /// Schedule a retry for an operation
    pub fn schedule_retry(
        env: &Env,
        operation_id: BytesN<32>,
        error: IntegrationError,
        operation_type: String
    ) -> Result<u64, IntegrationError> {
        let retry_config: RetryConfig = env.storage().persistent()
            .get(&DataKey::RetryConfig(operation_type))
            .unwrap_or_else(|| get_default_retry_config("default"));
        
        let current_time = env.ledger().timestamp();
        
        // Get or create retry state
        let mut retry_state: RetryState = env.storage().temporary()
            .get(&DataKey::RetryState(operation_id.clone()))
            .unwrap_or_else(|| RetryState {
                operation_id: operation_id.clone(),
                current_attempt: 0,
                next_retry_at: current_time,
                last_error: error,
                error_history: Vec::new(env),
                created_at: current_time,
                updated_at: current_time,
            });
        
        // Check if we can still retry
        if retry_state.current_attempt >= retry_config.max_retries {
            return Err(IntegrationError::OperationTimeout);
        }
        
        // Update retry state
        retry_state.current_attempt += 1;
        retry_state.last_error = error;
        retry_state.error_history.push_back(error);
        retry_state.updated_at = current_time;
        
        // Calculate next retry time with exponential backoff
        let delay = Self::calculate_retry_delay(
            &retry_config,
            retry_state.current_attempt,
            env
        );
        
        retry_state.next_retry_at = current_time + delay;
        
        // Save retry state
        env.storage().temporary().set(
            &DataKey::RetryState(operation_id.clone()),
            &retry_state
        );
        
        // Add to pending retries list
        let mut pending_retries: Vec<BytesN<32>> = env.storage().temporary()
            .get(&DataKey::PendingRetries)
            .unwrap_or_else(|| Vec::new(env));
        
        // Check if already in pending list
        let mut already_pending = false;
        for pending_id in pending_retries.iter() {
            if pending_id == operation_id {
                already_pending = true;
                break;
            }
        }
        
        if !already_pending {
            pending_retries.push_back(operation_id);
            env.storage().temporary().set(&DataKey::PendingRetries, &pending_retries);
        }
        
        Ok(retry_state.next_retry_at)
    }
    
    /// Calculate retry delay with exponential backoff and optional jitter
    fn calculate_retry_delay(
        config: &RetryConfig,
        attempt: u32,
        env: &Env
    ) -> u64 {
        // Calculate exponential backoff: base_delay * (multiplier ^ (attempt - 1))
        let mut delay = config.base_delay_ms;
        
        for _ in 1..attempt {
            delay = delay.saturating_mul(config.backoff_multiplier);
            if delay > config.max_delay_ms {
                delay = config.max_delay_ms;
                break;
            }
        }
        
        // Add jitter if enabled (±25% of calculated delay)
        if config.jitter_enabled {
            let jitter_range = delay / 4; // 25% of delay
            let ledger_seq = env.ledger().sequence();
            let jitter = (ledger_seq % (jitter_range * 2)) as u64;
            
            if jitter > jitter_range {
                delay = delay.saturating_add(jitter - jitter_range);
            } else {
                delay = delay.saturating_sub(jitter_range - jitter);
            }
        }
        
        delay.min(config.max_delay_ms)
    }
    
    /// Get operations ready for retry
    pub fn get_ready_retries(env: &Env) -> Vec<BytesN<32>> {
        let pending_retries: Vec<BytesN<32>> = env.storage().temporary()
            .get(&DataKey::PendingRetries)
            .unwrap_or_else(|| Vec::new(env));
        
        let current_time = env.ledger().timestamp();
        let mut ready_retries = Vec::new(env);
        let mut still_pending = Vec::new(env);
        
        for operation_id in pending_retries.iter() {
            if let Some(retry_state) = env.storage().temporary()
                .get::<DataKey, RetryState>(&DataKey::RetryState(operation_id.clone())) {
                
                if retry_state.next_retry_at <= current_time {
                    ready_retries.push_back(operation_id.clone());
                } else {
                    still_pending.push_back(operation_id.clone());
                }
            }
        }
        
        // Update pending retries list
        env.storage().temporary().set(&DataKey::PendingRetries, &still_pending);
        
        ready_retries
    }
    
    /// Mark retry as successful and clean up state
    pub fn mark_retry_success(env: &Env, operation_id: BytesN<32>) {
        // Remove from retry state
        env.storage().temporary().remove(&DataKey::RetryState(operation_id.clone()));
        
        // Remove from pending retries
        let pending_retries: Vec<BytesN<32>> = env.storage().temporary()
            .get(&DataKey::PendingRetries)
            .unwrap_or_else(|| Vec::new(env));
        
        let mut updated_pending = Vec::new(env);
        for pending_id in pending_retries.iter() {
            if pending_id != operation_id {
                updated_pending.push_back(pending_id.clone());
            }
        }
        
        env.storage().temporary().set(&DataKey::PendingRetries, &updated_pending);
    }
    
    /// Mark retry as permanently failed
    pub fn mark_retry_failed(env: &Env, operation_id: BytesN<32>) {
        // Keep retry state for audit purposes but mark as failed
        if let Some(mut retry_state) = env.storage().temporary()
            .get::<DataKey, RetryState>(&DataKey::RetryState(operation_id.clone())) {
            
            retry_state.updated_at = env.ledger().timestamp();
            env.storage().persistent().set(
                &DataKey::RetryState(operation_id.clone()),
                &retry_state
            );
        }
        
        // Remove from temporary storage and pending list
        env.storage().temporary().remove(&DataKey::RetryState(operation_id.clone()));
        
        let pending_retries: Vec<BytesN<32>> = env.storage().temporary()
            .get(&DataKey::PendingRetries)
            .unwrap_or_else(|| Vec::new(env));
        
        let mut updated_pending = Vec::new(env);
        for pending_id in pending_retries.iter() {
            if pending_id != operation_id {
                updated_pending.push_back(pending_id.clone());
            }
        }
        
        env.storage().temporary().set(&DataKey::PendingRetries, &updated_pending);
    }
    
    /// Get retry state for an operation
    pub fn get_retry_state(env: &Env, operation_id: BytesN<32>) -> Option<RetryState> {
        env.storage().temporary().get(&DataKey::RetryState(operation_id))
    }
    
    /// Get retry configuration for an operation type
    pub fn get_retry_config(env: &Env, operation_type: String) -> RetryConfig {
        env.storage().persistent()
            .get(&DataKey::RetryConfig(operation_type))
            .unwrap_or_else(|| get_default_retry_config("default"))
    }
    
    /// Update retry configuration for an operation type (admin only)
    pub fn update_retry_config(
        env: &Env,
        caller: Address,
        operation_type: String,
        config: RetryConfig
    ) -> Result<(), IntegrationError> {
        // Note: Role checking would be done in the main contract
        caller.require_auth();
        
        env.storage().persistent().set(
            &DataKey::RetryConfig(operation_type),
            &config
        );
        
        Ok(())
    }
    
    /// Get all pending retries with their states
    pub fn get_all_pending_retries(env: &Env) -> Vec<RetryState> {
        let pending_retries: Vec<BytesN<32>> = env.storage().temporary()
            .get(&DataKey::PendingRetries)
            .unwrap_or_else(|| Vec::new(env));
        
        let mut retry_states = Vec::new(env);
        
        for operation_id in pending_retries.iter() {
            if let Some(retry_state) = env.storage().temporary()
                .get::<DataKey, RetryState>(&DataKey::RetryState(operation_id.clone())) {
                retry_states.push_back(retry_state);
            }
        }
        
        retry_states
    }
    
    /// Clean up expired retry states
    pub fn cleanup_expired_retries(env: &Env, max_age_seconds: u64) {
        let current_time = env.ledger().timestamp();
        let cutoff_time = current_time.saturating_sub(max_age_seconds);
        
        let pending_retries: Vec<BytesN<32>> = env.storage().temporary()
            .get(&DataKey::PendingRetries)
            .unwrap_or_else(|| Vec::new(env));
        
        let mut active_retries = Vec::new(env);
        
        for operation_id in pending_retries.iter() {
            if let Some(retry_state) = env.storage().temporary()
                .get::<DataKey, RetryState>(&DataKey::RetryState(operation_id.clone())) {
                
                if retry_state.created_at >= cutoff_time {
                    active_retries.push_back(operation_id.clone());
                } else {
                    // Remove expired retry state
                    env.storage().temporary().remove(&DataKey::RetryState(operation_id.clone()));
                }
            }
        }
        
        env.storage().temporary().set(&DataKey::PendingRetries, &active_retries);
    }
}

/// Utility functions for retry operations
pub mod retry_utils {
    use super::*;
    
    /// Execute an operation with automatic retry logic
    pub fn execute_with_retry<F, R>(
        env: &Env,
        operation_id: BytesN<32>,
        operation_type: String,
        operation: F
    ) -> Result<R, IntegrationError>
    where
        F: Fn() -> Result<R, IntegrationError>,
    {
        let mut last_error = IntegrationError::InvalidOperationState;
        
        loop {
            match operation() {
                Ok(result) => {
                    // Success - clean up retry state if it exists
                    RetryManager::mark_retry_success(env, operation_id.clone());
                    return Ok(result);
                },
                Err(error) => {
                    last_error = error;
                    
                    // Check if we should retry
                    if !RetryManager::should_retry(
                        env,
                        operation_id.clone(),
                        error,
                        operation_type.clone()
                    ) {
                        // No more retries - mark as failed
                        RetryManager::mark_retry_failed(env, operation_id);
                        return Err(error);
                    }
                    
                    // Schedule retry
                    match RetryManager::schedule_retry(
                        env,
                        operation_id.clone(),
                        error,
                        operation_type.clone()
                    ) {
                        Ok(_next_retry_time) => {
                            // For now, we return the error and let the caller handle scheduling
                            // In a real implementation, this might involve async scheduling
                            return Err(IntegrationError::OperationTimeout);
                        },
                        Err(retry_error) => {
                            return Err(retry_error);
                        }
                    }
                }
            }
        }
    }
    
    /// Check if an error should trigger immediate failure (no retry)
    pub fn is_permanent_failure(error: IntegrationError) -> bool {
        matches!(
            error,
            IntegrationError::Unauthorized |
            IntegrationError::InsufficientPermissions |
            IntegrationError::InvalidSignature |
            IntegrationError::AddressBlacklisted |
            IntegrationError::InsufficientKYCTier |
            IntegrationError::DuplicateOperation |
            IntegrationError::InvalidInput |
            IntegrationError::ValidationFailed |
            IntegrationError::DataCorruption
        )
    }
    
    /// Get human-readable retry status
    pub fn get_retry_status_message(retry_state: &RetryState) -> String {
        let env = Env::default(); // This would need to be passed in real implementation
        
        if retry_state.current_attempt == 0 {
            String::from_str(&env, "No retries attempted")
        } else {
            // In a real implementation, we'd format this properly
            String::from_str(&env, "Retry in progress")
        }
    }
}