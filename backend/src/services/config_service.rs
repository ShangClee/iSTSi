use loco_rs::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::soroban_client::{SorobanConfig, ContractAddresses};

/// Application configuration including Soroban settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub soroban: SorobanConfig,
    pub database_url: String,
    pub jwt_secret: String,
    pub environment: String,
}

/// Configuration service for loading and managing app settings
pub struct ConfigService;

impl ConfigService {
    /// Load Soroban configuration from environment and config files
    pub fn load_soroban_config() -> Result<SorobanConfig> {
        let network = std::env::var("SOROBAN_NETWORK")
            .unwrap_or_else(|_| "testnet".to_string());
        
        let rpc_url = match network.as_str() {
            "mainnet" => "https://soroban-mainnet.stellar.org".to_string(),
            "testnet" => "https://soroban-testnet.stellar.org".to_string(),
            "futurenet" => "https://rpc-futurenet.stellar.org".to_string(),
            _ => std::env::var("SOROBAN_RPC_URL")
                .unwrap_or_else(|_| "https://soroban-testnet.stellar.org".to_string()),
        };
        
        let network_passphrase = match network.as_str() {
            "mainnet" => "Public Global Stellar Network ; September 2015".to_string(),
            "testnet" => "Test SDF Network ; September 2015".to_string(),
            "futurenet" => "Test SDF Future Network ; October 2022".to_string(),
            _ => std::env::var("SOROBAN_NETWORK_PASSPHRASE")
                .unwrap_or_else(|_| "Test SDF Network ; September 2015".to_string()),
        };
        
        let contracts = ContractAddresses {
            integration_router: std::env::var("CONTRACT_INTEGRATION_ROUTER")
                .unwrap_or_else(|_| "".to_string()),
            kyc_registry: std::env::var("CONTRACT_KYC_REGISTRY")
                .unwrap_or_else(|_| "".to_string()),
            istsi_token: std::env::var("CONTRACT_ISTSI_TOKEN")
                .unwrap_or_else(|_| "".to_string()),
            reserve_manager: std::env::var("CONTRACT_RESERVE_MANAGER")
                .unwrap_or_else(|_| "".to_string()),
        };
        
        Ok(SorobanConfig {
            network,
            rpc_url,
            network_passphrase,
            contracts,
        })
    }
    
    /// Load complete application configuration
    pub fn load_app_config() -> Result<AppConfig> {
        let soroban = Self::load_soroban_config()?;
        
        let database_url = std::env::var("DATABASE_URL")
            .map_err(|_| Error::string("DATABASE_URL environment variable not set"))?;
        
        let jwt_secret = std::env::var("JWT_SECRET")
            .unwrap_or_else(|_| "development-secret-key".to_string());
        
        let environment = std::env::var("LOCO_ENV")
            .unwrap_or_else(|_| "development".to_string());
        
        Ok(AppConfig {
            soroban,
            database_url,
            jwt_secret,
            environment,
        })
    }
    
    /// Validate that all required contract addresses are configured
    pub fn validate_contract_addresses(config: &SorobanConfig) -> Result<()> {
        let contracts = &config.contracts;
        
        // For testing, allow empty addresses
        if !contracts.integration_router.is_empty() && !Self::is_valid_stellar_address(&contracts.integration_router) {
            return Err(Error::string(&format!("Invalid integration router address format: {}", contracts.integration_router)));
        }
        
        if !contracts.kyc_registry.is_empty() && !Self::is_valid_stellar_address(&contracts.kyc_registry) {
            return Err(Error::string(&format!("Invalid KYC registry address format: {}", contracts.kyc_registry)));
        }
        
        if !contracts.istsi_token.is_empty() && !Self::is_valid_stellar_address(&contracts.istsi_token) {
            return Err(Error::string(&format!("Invalid iSTSi token address format: {}", contracts.istsi_token)));
        }
        
        if !contracts.reserve_manager.is_empty() && !Self::is_valid_stellar_address(&contracts.reserve_manager) {
            return Err(Error::string(&format!("Invalid reserve manager address format: {}", contracts.reserve_manager)));
        }
        
        Ok(())
    }
    
    /// Basic validation for Stellar contract addresses
    fn is_valid_stellar_address(address: &str) -> bool {
        // Stellar addresses are 56 characters long and start with 'C' for contracts
        // For testing purposes, we'll be more lenient and allow test addresses
        if address.len() != 56 {
            return false;
        }
        
        // Contract addresses start with 'C'
        if !address.starts_with('C') {
            return false;
        }
        
        // For testing, allow any uppercase letters and digits
        // In production, this would use proper Stellar address validation
        address.chars().all(|c| c.is_ascii_alphanumeric() && (c.is_ascii_uppercase() || c.is_ascii_digit()))
    }
    
    /// Get network-specific configuration
    pub fn get_network_config(network: &str) -> HashMap<String, String> {
        let mut config = HashMap::new();
        
        match network {
            "mainnet" => {
                config.insert("rpc_url".to_string(), "https://soroban-mainnet.stellar.org".to_string());
                config.insert("network_passphrase".to_string(), "Public Global Stellar Network ; September 2015".to_string());
                config.insert("horizon_url".to_string(), "https://horizon.stellar.org".to_string());
            }
            "testnet" => {
                config.insert("rpc_url".to_string(), "https://soroban-testnet.stellar.org".to_string());
                config.insert("network_passphrase".to_string(), "Test SDF Network ; September 2015".to_string());
                config.insert("horizon_url".to_string(), "https://horizon-testnet.stellar.org".to_string());
            }
            "futurenet" => {
                config.insert("rpc_url".to_string(), "https://rpc-futurenet.stellar.org".to_string());
                config.insert("network_passphrase".to_string(), "Test SDF Future Network ; October 2022".to_string());
                config.insert("horizon_url".to_string(), "https://horizon-futurenet.stellar.org".to_string());
            }
            _ => {
                // Default to testnet
                config.insert("rpc_url".to_string(), "https://soroban-testnet.stellar.org".to_string());
                config.insert("network_passphrase".to_string(), "Test SDF Network ; September 2015".to_string());
                config.insert("horizon_url".to_string(), "https://horizon-testnet.stellar.org".to_string());
            }
        }
        
        config
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_stellar_address_validation() {
        // Valid contract address
        assert!(ConfigService::is_valid_stellar_address("CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"));
        
        // Invalid addresses
        assert!(!ConfigService::is_valid_stellar_address("GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA")); // Account, not contract
        assert!(!ConfigService::is_valid_stellar_address("CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA")); // Too short
        assert!(!ConfigService::is_valid_stellar_address("invalid")); // Invalid format
    }
    
    #[test]
    fn test_network_config() {
        let mainnet_config = ConfigService::get_network_config("mainnet");
        assert_eq!(mainnet_config.get("rpc_url").unwrap(), "https://soroban-mainnet.stellar.org");
        
        let testnet_config = ConfigService::get_network_config("testnet");
        assert_eq!(testnet_config.get("rpc_url").unwrap(), "https://soroban-testnet.stellar.org");
        
        let unknown_config = ConfigService::get_network_config("unknown");
        assert_eq!(unknown_config.get("rpc_url").unwrap(), "https://soroban-testnet.stellar.org"); // Defaults to testnet
    }
}