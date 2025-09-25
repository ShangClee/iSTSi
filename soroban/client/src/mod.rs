//! Contract client interfaces for backend integration
//! 
//! This module provides Rust client interfaces for interacting with
//! the Soroban smart contracts from the backend service.

pub mod integration_router_client;
pub mod kyc_registry_client;
pub mod istsi_token_client;
pub mod reserve_manager_client;
pub mod contract_manager;
pub mod event_monitor;
pub mod address_config;

// Re-export commonly used items
pub use integration_router_client::IntegrationRouterClient;
pub use kyc_registry_client::KycRegistryClient;
pub use istsi_token_client::IstsiTokenClient;
pub use reserve_manager_client::ReserveManagerClient;
pub use contract_manager::ContractManager;
pub use event_monitor::{EventMonitor, ContractEvent};
pub use address_config::{ContractAddresses, NetworkConfig};

use soroban_sdk::{Address, Env};
use shared::{IntegrationError, ValidationError};

/// Common result type for contract operations
pub type ContractResult<T> = Result<T, ContractError>;

/// Contract operation errors
#[derive(Debug, Clone, PartialEq)]
pub enum ContractError {
    Integration(IntegrationError),
    Validation(ValidationError),
    NetworkError(String),
    ParseError(String),
    Timeout(String),
    ContractNotFound(String),
}

impl From<IntegrationError> for ContractError {
    fn from(err: IntegrationError) -> Self {
        ContractError::Integration(err)
    }
}

impl From<ValidationError> for ContractError {
    fn from(err: ValidationError) -> Self {
        ContractError::Validation(err)
    }
}

/// Common trait for all contract clients
pub trait ContractClient {
    /// Get the contract address
    fn contract_address(&self) -> &Address;
    
    /// Check if the contract is available
    fn is_available(&self) -> bool;
    
    /// Get contract version (if supported)
    fn version(&self) -> ContractResult<String>;
}

/// Contract operation context
#[derive(Debug, Clone)]
pub struct OperationContext {
    pub caller: Address,
    pub operation_id: String,
    pub timeout_seconds: u64,
    pub retry_count: u32,
}

impl Default for OperationContext {
    fn default() -> Self {
        Self {
            caller: Address::from_string(&soroban_sdk::String::from_str(&Env::default(), "PLACEHOLDER")),
            operation_id: String::new(),
            timeout_seconds: 30,
            retry_count: 3,
        }
    }
}