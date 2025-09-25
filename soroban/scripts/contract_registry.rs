use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Contract registry for managing deployed contract addresses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractRegistry {
    pub network: String,
    pub deployed_at: String,
    pub admin_address: String,
    pub contracts: HashMap<String, String>,
    pub metadata: HashMap<String, serde_json::Value>,
}

impl ContractRegistry {
    /// Load registry from file
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let registry: ContractRegistry = serde_json::from_str(&content)?;
        Ok(registry)
    }

    /// Save registry to file
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn std::error::Error>> {
        let content = serde_json::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }

    /// Get contract address by name
    pub fn get_contract(&self, name: &str) -> Option<&String> {
        self.contracts.get(name)
    }

    /// Add or update contract address
    pub fn set_contract(&mut self, name: String, address: String) {
        self.contracts.insert(name, address);
    }

    /// Remove contract from registry
    pub fn remove_contract(&mut self, name: &str) -> Option<String> {
        self.contracts.remove(name)
    }

    /// Get all contract names
    pub fn get_contract_names(&self) -> Vec<&String> {
        self.contracts.keys().collect()
    }

    /// Validate all contract addresses are valid Stellar contract IDs
    pub fn validate_addresses(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        
        for (name, address) in &self.contracts {
            if !is_valid_stellar_contract_id(address) {
                errors.push(format!("Invalid contract ID for {}: {}", name, address));
            }
        }
        
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Create a new registry
    pub fn new(network: String, admin_address: String) -> Self {
        Self {
            network,
            deployed_at: chrono::Utc::now().to_rfc3339(),
            admin_address,
            contracts: HashMap::new(),
            metadata: HashMap::new(),
        }
    }

    /// Update metadata
    pub fn set_metadata(&mut self, key: String, value: serde_json::Value) {
        self.metadata.insert(key, value);
    }

    /// Get metadata
    pub fn get_metadata(&self, key: &str) -> Option<&serde_json::Value> {
        self.metadata.get(key)
    }
}

/// Validate if a string is a valid Stellar contract ID
fn is_valid_stellar_contract_id(address: &str) -> bool {
    // Stellar contract IDs are 56 characters long and start with 'C'
    address.len() == 56 && address.starts_with('C') && address.chars().all(|c| c.is_alphanumeric())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_contract_registry_creation() {
        let registry = ContractRegistry::new(
            "testnet".to_string(),
            "GADMIN123456789".to_string(),
        );
        
        assert_eq!(registry.network, "testnet");
        assert_eq!(registry.admin_address, "GADMIN123456789");
        assert!(registry.contracts.is_empty());
    }

    #[test]
    fn test_contract_operations() {
        let mut registry = ContractRegistry::new(
            "testnet".to_string(),
            "GADMIN123456789".to_string(),
        );
        
        // Add contract
        registry.set_contract(
            "kyc_registry".to_string(),
            "CKYC123456789012345678901234567890123456789012345678901234".to_string(),
        );
        
        // Get contract
        assert_eq!(
            registry.get_contract("kyc_registry"),
            Some(&"CKYC123456789012345678901234567890123456789012345678901234".to_string())
        );
        
        // Remove contract
        let removed = registry.remove_contract("kyc_registry");
        assert!(removed.is_some());
        assert!(registry.get_contract("kyc_registry").is_none());
    }

    #[test]
    fn test_file_operations() {
        let mut registry = ContractRegistry::new(
            "testnet".to_string(),
            "GADMIN123456789".to_string(),
        );
        
        registry.set_contract(
            "test_contract".to_string(),
            "CTEST123456789012345678901234567890123456789012345678901234".to_string(),
        );
        
        // Save to file
        let temp_file = NamedTempFile::new().unwrap();
        registry.save_to_file(temp_file.path()).unwrap();
        
        // Load from file
        let loaded_registry = ContractRegistry::load_from_file(temp_file.path()).unwrap();
        
        assert_eq!(loaded_registry.network, registry.network);
        assert_eq!(loaded_registry.admin_address, registry.admin_address);
        assert_eq!(loaded_registry.contracts, registry.contracts);
    }

    #[test]
    fn test_address_validation() {
        let mut registry = ContractRegistry::new(
            "testnet".to_string(),
            "GADMIN123456789".to_string(),
        );
        
        // Valid address
        registry.set_contract(
            "valid_contract".to_string(),
            "CVALID123456789012345678901234567890123456789012345678901234".to_string(),
        );
        
        // Invalid address (too short)
        registry.set_contract(
            "invalid_contract".to_string(),
            "CINVALID123".to_string(),
        );
        
        let validation_result = registry.validate_addresses();
        assert!(validation_result.is_err());
        
        let errors = validation_result.unwrap_err();
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("invalid_contract"));
    }
}