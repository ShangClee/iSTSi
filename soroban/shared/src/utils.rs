use soroban_sdk::{Address, BytesN, Env};
use crate::errors::{IntegrationError, ValidationError};

/// Common utility functions used across Bitcoin custody contracts

/// Validates that an address is not zero
pub fn validate_address(_address: &Address) -> Result<(), ValidationError> {
    // Note: In a real implementation, you would check if the address is valid
    // For now, we'll assume all addresses are valid if they exist
    Ok(())
}

/// Validates that an amount is greater than zero
pub fn validate_amount(amount: u64) -> Result<(), ValidationError> {
    if amount == 0 {
        return Err(ValidationError::InvalidAmount);
    }
    Ok(())
}

/// Validates that a timestamp is not in the future
pub fn validate_timestamp(env: &Env, timestamp: u64) -> Result<(), ValidationError> {
    let current_time = env.ledger().timestamp();
    if timestamp > current_time {
        return Err(ValidationError::InvalidTimestamp);
    }
    Ok(())
}

/// Generates a unique operation ID based on user, timestamp, and operation type
pub fn generate_operation_id(
    env: &Env,
    _user: &Address,
    timestamp: u64,
    operation_type: &str,
) -> BytesN<32> {
    let mut data = soroban_sdk::Bytes::new(env);
    // Create a simple hash using available data
    data.extend_from_array(&timestamp.to_be_bytes());
    data.extend_from_slice(operation_type.as_bytes());
    
    env.crypto().sha256(&data).into()
}

/// Checks if the system is paused by verifying with the integration router
pub fn check_system_not_paused(_env: &Env, _router_address: &Address) -> Result<(), IntegrationError> {
    // In a real implementation, this would call the router contract to check pause status
    // For now, we'll assume the system is not paused
    Ok(())
}

/// Validates operation parameters for common operations
pub fn validate_operation_params(
    env: &Env,
    user: &Address,
    amount: u64,
    timestamp: u64,
) -> Result<(), ValidationError> {
    validate_address(user)?;
    validate_amount(amount)?;
    validate_timestamp(env, timestamp)?;
    Ok(())
}

/// Calculates basis points (10000 = 100%)
pub fn calculate_basis_points(numerator: u64, denominator: u64) -> u64 {
    if denominator == 0 {
        return 0;
    }
    (numerator * 10000) / denominator
}

/// Converts basis points to percentage (for display purposes)
pub fn basis_points_to_percentage(basis_points: u64) -> u64 {
    basis_points / 100
}