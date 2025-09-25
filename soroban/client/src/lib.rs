//! Soroban Contract Client Library
//! 
//! This library provides high-level client interfaces for interacting with
//! the Bitcoin custody system's Soroban smart contracts from backend services.
//! 
//! # Features
//! 
//! - **Contract Clients**: High-level interfaces for all contracts
//! - **Event Monitoring**: Real-time event processing and filtering
//! - **Address Management**: Configuration management for different networks
//! - **Error Handling**: Comprehensive error types and retry logic
//! - **Integration Workflows**: End-to-end operation orchestration
//! 
//! # Quick Start
//! 
//! ```rust
//! use soroban_client::{ContractManager, ContractAddresses, NetworkConfig};
//! use soroban_sdk::Env;
//! 
//! // Initialize contract manager
//! let env = Env::default();
//! let addresses = ContractAddresses::from_config(config_map)?;
//! let network = NetworkConfig::testnet();
//! let manager = ContractManager::new(env, addresses, network)?;
//! 
//! // Execute Bitcoin deposit
//! let operation_id = manager.execute_bitcoin_deposit_workflow(
//!     &ctx,
//!     &user_address,
//!     100_000_000, // 1 BTC in satoshis
//!     &btc_tx_hash,
//!     6, // confirmations
//!     800000, // block height
//! )?;
//! ```
//! 
//! # Architecture
//! 
//! The library is organized into several modules:
//! 
//! - `integration_router_client`: Client for the Integration Router contract
//! - `kyc_registry_client`: Client for the KYC Registry contract
//! - `istsi_token_client`: Client for the iSTSi Token contract
//! - `reserve_manager_client`: Client for the Reserve Manager contract
//! - `contract_manager`: Unified manager for all contract interactions
//! - `event_monitor`: Event monitoring and processing utilities
//! - `address_config`: Contract address and network configuration management

#![no_std]

extern crate alloc;

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
pub use contract_manager::{ContractManager, SystemHealth, SystemStatus};
pub use event_monitor::{EventMonitor, ContractEvent, EventData, EventFilter};
pub use address_config::{ContractAddresses, NetworkConfig, AddressRegistry};

use soroban_sdk::Address;

/// Common result type for contract operations
pub type ContractResult<T> = Result<T, ContractError>;

/// Contract operation errors
#[derive(Debug, Clone, PartialEq)]
pub enum ContractError {
    Integration(shared::IntegrationError),
    Validation(shared::ValidationError),
    NetworkError(alloc::string::String),
    ParseError(alloc::string::String),
    Timeout(alloc::string::String),
    ContractNotFound(alloc::string::String),
}

impl From<shared::IntegrationError> for ContractError {
    fn from(err: shared::IntegrationError) -> Self {
        ContractError::Integration(err)
    }
}

impl From<shared::ValidationError> for ContractError {
    fn from(err: shared::ValidationError) -> Self {
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
    fn version(&self) -> ContractResult<alloc::string::String>;
}

/// Contract operation context
#[derive(Debug, Clone)]
pub struct OperationContext {
    pub caller: Address,
    pub operation_id: alloc::string::String,
    pub timeout_seconds: u64,
    pub retry_count: u32,
}

impl Default for OperationContext {
    fn default() -> Self {
        Self {
            caller: Address::from_string(&soroban_sdk::String::from_str(
                &soroban_sdk::Env::default(), 
                "GXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX"
            )),
            operation_id: alloc::string::String::new(),
            timeout_seconds: 30,
            retry_count: 3,
        }
    }
}

/// Library version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Get library version
pub fn version() -> &'static str {
    VERSION
}

/// Initialize the client library with default configuration
pub fn init() -> ContractResult<()> {
    // Perform any necessary initialization
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::Env;

    #[test]
    fn test_version() {
        assert!(!version().is_empty());
    }

    #[test]
    fn test_init() {
        assert!(init().is_ok());
    }

    #[test]
    fn test_operation_context_default() {
        let ctx = OperationContext::default();
        assert_eq!(ctx.timeout_seconds, 30);
        assert_eq!(ctx.retry_count, 3);
    }
}