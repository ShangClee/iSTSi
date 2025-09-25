use soroban_sdk::Address;
use alloc::collections::BTreeMap as HashMap;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use alloc::format;

/// Contract addresses configuration for different networks
/// 
/// This module manages contract addresses across different Soroban networks
/// (testnet, mainnet, local) and provides configuration management.
#[derive(Debug, Clone)]
pub struct ContractAddresses {
    pub integration_router: Option<Address>,
    pub kyc_registry: Option<Address>,
    pub istsi_token: Option<Address>,
    pub reserve_manager: Option<Address>,
    pub fungible_token: Option<Address>,
}

impl ContractAddresses {
    /// Create a new empty contract addresses configuration
    pub fn new() -> Self {
        Self {
            integration_router: None,
            kyc_registry: None,
            istsi_token: None,
            reserve_manager: None,
            fungible_token: None,
        }
    }

    /// Create contract addresses from a configuration map
    /// 
    /// # Arguments
    /// * `config` - Map of contract names to address strings
    /// 
    /// # Returns
    /// * `Ok(addresses)` - Contract addresses configuration
    /// * `Err(error)` - Parse error
    pub fn from_config(config: HashMap<String, String>) -> Result<Self, String> {
        let mut addresses = Self::new();

        for (contract_name, address_str) in config {
            let address = Address::from_string(&soroban_sdk::String::from_str(
                &soroban_sdk::Env::default(),
                &address_str
            ));

            match contract_name.as_str() {
                "integration_router" => addresses.integration_router = Some(address),
                "kyc_registry" => addresses.kyc_registry = Some(address),
                "istsi_token" => addresses.istsi_token = Some(address),
                "reserve_manager" => addresses.reserve_manager = Some(address),
                "fungible_token" => addresses.fungible_token = Some(address),
                _ => return Err(format!("Unknown contract name: {}", contract_name)),
            }
        }

        Ok(addresses)
    }

    /// Validate that all required addresses are present
    /// 
    /// # Returns
    /// * `Ok(())` - All required addresses present
    /// * `Err(missing)` - List of missing contract addresses
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut missing = Vec::new();

        if self.integration_router.is_none() {
            missing.push("integration_router".to_string());
        }
        if self.kyc_registry.is_none() {
            missing.push("kyc_registry".to_string());
        }
        if self.istsi_token.is_none() {
            missing.push("istsi_token".to_string());
        }
        if self.reserve_manager.is_none() {
            missing.push("reserve_manager".to_string());
        }

        if missing.is_empty() {
            Ok(())
        } else {
            Err(missing)
        }
    }

    /// Convert to a configuration map
    /// 
    /// # Returns
    /// * Map of contract names to address strings
    pub fn to_config(&self) -> HashMap<String, String> {
        let mut config = HashMap::new();

        if let Some(addr) = &self.integration_router {
            config.insert("integration_router".to_string(), format!("{:?}", addr));
        }
        if let Some(addr) = &self.kyc_registry {
            config.insert("kyc_registry".to_string(), format!("{:?}", addr));
        }
        if let Some(addr) = &self.istsi_token {
            config.insert("istsi_token".to_string(), format!("{:?}", addr));
        }
        if let Some(addr) = &self.reserve_manager {
            config.insert("reserve_manager".to_string(), format!("{:?}", addr));
        }
        if let Some(addr) = &self.fungible_token {
            config.insert("fungible_token".to_string(), format!("{:?}", addr));
        }

        config
    }
}

impl Default for ContractAddresses {
    fn default() -> Self {
        Self::new()
    }
}

/// Network configuration for Soroban interactions
#[derive(Debug, Clone)]
pub struct NetworkConfig {
    pub network_name: String,
    pub rpc_url: String,
    pub network_passphrase: String,
    pub min_confirmations: u32,
    pub timeout_seconds: u64,
    pub retry_count: u32,
    pub gas_limit: u64,
}

impl NetworkConfig {
    /// Create testnet configuration
    pub fn testnet() -> Self {
        Self {
            network_name: "testnet".to_string(),
            rpc_url: "https://soroban-testnet.stellar.org".to_string(),
            network_passphrase: "Test SDF Network ; September 2015".to_string(),
            min_confirmations: 1,
            timeout_seconds: 30,
            retry_count: 3,
            gas_limit: 1_000_000,
        }
    }

    /// Create mainnet configuration
    pub fn mainnet() -> Self {
        Self {
            network_name: "mainnet".to_string(),
            rpc_url: "https://soroban-mainnet.stellar.org".to_string(),
            network_passphrase: "Public Global Stellar Network ; September 2015".to_string(),
            min_confirmations: 6,
            timeout_seconds: 60,
            retry_count: 5,
            gas_limit: 2_000_000,
        }
    }

    /// Create local development configuration
    pub fn local() -> Self {
        Self {
            network_name: "local".to_string(),
            rpc_url: "http://localhost:8000".to_string(),
            network_passphrase: "Standalone Network ; February 2017".to_string(),
            min_confirmations: 1,
            timeout_seconds: 10,
            retry_count: 1,
            gas_limit: 500_000,
        }
    }

    /// Create custom network configuration
    /// 
    /// # Arguments
    /// * `name` - Network name
    /// * `rpc_url` - RPC endpoint URL
    /// * `passphrase` - Network passphrase
    /// 
    /// # Returns
    /// * Custom network configuration
    pub fn custom(name: String, rpc_url: String, passphrase: String) -> Self {
        Self {
            network_name: name,
            rpc_url,
            network_passphrase: passphrase,
            min_confirmations: 3,
            timeout_seconds: 30,
            retry_count: 3,
            gas_limit: 1_000_000,
        }
    }

    /// Validate network configuration
    /// 
    /// # Returns
    /// * `Ok(())` - Configuration is valid
    /// * `Err(error)` - Validation error
    pub fn validate(&self) -> Result<(), String> {
        if self.network_name.is_empty() {
            return Err("Network name cannot be empty".to_string());
        }

        if self.rpc_url.is_empty() {
            return Err("RPC URL cannot be empty".to_string());
        }

        if self.network_passphrase.is_empty() {
            return Err("Network passphrase cannot be empty".to_string());
        }

        if self.timeout_seconds == 0 {
            return Err("Timeout must be greater than 0".to_string());
        }

        if self.gas_limit == 0 {
            return Err("Gas limit must be greater than 0".to_string());
        }

        Ok(())
    }
}

/// Contract deployment configuration
#[derive(Debug, Clone)]
pub struct DeploymentConfig {
    pub network: NetworkConfig,
    pub deployer_secret: String,
    pub contracts_to_deploy: Vec<String>,
    pub initialization_params: HashMap<String, serde_json::Value>,
}

impl DeploymentConfig {
    /// Create a new deployment configuration
    pub fn new(network: NetworkConfig, deployer_secret: String) -> Self {
        Self {
            network,
            deployer_secret,
            contracts_to_deploy: Vec::new(),
            initialization_params: HashMap::new(),
        }
    }

    /// Add a contract to deploy
    /// 
    /// # Arguments
    /// * `contract_name` - Name of the contract to deploy
    /// * `init_params` - Initialization parameters
    pub fn add_contract(&mut self, contract_name: String, init_params: serde_json::Value) {
        self.contracts_to_deploy.push(contract_name.clone());
        self.initialization_params.insert(contract_name, init_params);
    }

    /// Get initialization parameters for a contract
    /// 
    /// # Arguments
    /// * `contract_name` - Name of the contract
    /// 
    /// # Returns
    /// * `Some(params)` - Initialization parameters if found
    /// * `None` - Contract not found
    pub fn get_init_params(&self, contract_name: &str) -> Option<&serde_json::Value> {
        self.initialization_params.get(contract_name)
    }
}

/// Address registry for managing contract addresses across environments
#[derive(Debug, Clone)]
pub struct AddressRegistry {
    environments: HashMap<String, ContractAddresses>,
}

impl AddressRegistry {
    /// Create a new address registry
    pub fn new() -> Self {
        Self {
            environments: HashMap::new(),
        }
    }

    /// Add addresses for an environment
    /// 
    /// # Arguments
    /// * `environment` - Environment name (e.g., "testnet", "mainnet")
    /// * `addresses` - Contract addresses for the environment
    pub fn add_environment(&mut self, environment: String, addresses: ContractAddresses) {
        self.environments.insert(environment, addresses);
    }

    /// Get addresses for an environment
    /// 
    /// # Arguments
    /// * `environment` - Environment name
    /// 
    /// # Returns
    /// * `Some(addresses)` - Contract addresses if found
    /// * `None` - Environment not found
    pub fn get_addresses(&self, environment: &str) -> Option<&ContractAddresses> {
        self.environments.get(environment)
    }

    /// List all available environments
    /// 
    /// # Returns
    /// * Vector of environment names
    pub fn list_environments(&self) -> Vec<String> {
        self.environments.keys().cloned().collect()
    }

    /// Load registry from JSON configuration
    /// 
    /// # Arguments
    /// * `json_config` - JSON configuration string
    /// 
    /// # Returns
    /// * `Ok(registry)` - Loaded address registry
    /// * `Err(error)` - Parse error
    pub fn from_json(json_config: &str) -> Result<Self, String> {
        let config: serde_json::Value = serde_json::from_str(json_config)
            .map_err(|e| format!("Failed to parse JSON: {}", e))?;

        let mut registry = Self::new();

        if let Some(environments) = config.as_object() {
            for (env_name, env_config) in environments {
                if let Some(contracts) = env_config.as_object() {
                    let mut contract_map = HashMap::new();
                    for (contract_name, address) in contracts {
                        if let Some(addr_str) = address.as_str() {
                            contract_map.insert(contract_name.clone(), addr_str.to_string());
                        }
                    }
                    
                    let addresses = ContractAddresses::from_config(contract_map)
                        .map_err(|e| format!("Failed to parse addresses for {}: {}", env_name, e))?;
                    
                    registry.add_environment(env_name.clone(), addresses);
                }
            }
        }

        Ok(registry)
    }

    /// Save registry to JSON configuration
    /// 
    /// # Returns
    /// * `Ok(json)` - JSON configuration string
    /// * `Err(error)` - Serialization error
    pub fn to_json(&self) -> Result<String, String> {
        let mut config = serde_json::Map::new();

        for (env_name, addresses) in &self.environments {
            let address_map = addresses.to_config();
            let env_config = serde_json::Value::Object(
                address_map.into_iter()
                    .map(|(k, v)| (k, serde_json::Value::String(v)))
                    .collect()
            );
            config.insert(env_name.clone(), env_config);
        }

        serde_json::to_string_pretty(&serde_json::Value::Object(config))
            .map_err(|e| format!("Failed to serialize JSON: {}", e))
    }
}

impl Default for AddressRegistry {
    fn default() -> Self {
        Self::new()
    }
}