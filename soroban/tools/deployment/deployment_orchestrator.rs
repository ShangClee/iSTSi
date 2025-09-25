#!/usr/bin/env rust-script

//! Deployment Orchestrator for iSTSi Integration System
//! 
//! This script manages the deployment of all integrated contracts with proper
//! initialization sequencing, address registry management, and health verification.

use std::collections::HashMap;
use std::fs;
use std::process::Command;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentConfig {
    pub network: String,
    pub rpc_url: String,
    pub deployer_secret: String,
    pub contracts: Vec<ContractConfig>,
    pub initialization_sequence: Vec<String>,
    pub verification_checks: Vec<VerificationCheck>,
    pub deployment_timeout: u64,
    pub confirmation_blocks: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractConfig {
    pub name: String,
    pub path: String,
    pub dependencies: Vec<String>,
    pub initialization_params: HashMap<String, String>,
    pub required_roles: Vec<String>,
    pub health_check_endpoint: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationCheck {
    pub name: String,
    pub contract: String,
    pub function: String,
    pub expected_result: String,
    pub timeout: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentResult {
    pub contract_name: String,
    pub address: String,
    pub tx_hash: String,
    pub deployment_time: u64,
    pub gas_used: u64,
    pub status: DeploymentStatus,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeploymentStatus {
    Pending,
    Deploying,
    Deployed,
    Initializing,
    Initialized,
    Verified,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractRegistry {
    pub network: String,
    pub deployment_id: String,
    pub deployed_at: u64,
    pub contracts: HashMap<String, ContractInfo>,
    pub integration_router: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractInfo {
    pub address: String,
    pub tx_hash: String,
    pub deployed_at: u64,
    pub version: String,
    pub abi_hash: String,
    pub verified: bool,
}

pub struct DeploymentOrchestrator {
    config: DeploymentConfig,
    registry: ContractRegistry,
    deployment_results: Vec<DeploymentResult>,
}

impl DeploymentOrchestrator {
    pub fn new(config_path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let config_content = fs::read_to_string(config_path)?;
        let config: DeploymentConfig = serde_json::from_str(&config_content)?;
        
        let registry = ContractRegistry {
            network: config.network.clone(),
            deployment_id: generate_deployment_id(),
            deployed_at: current_timestamp(),
            contracts: HashMap::new(),
            integration_router: None,
        };
        
        Ok(Self {
            config,
            registry,
            deployment_results: Vec::new(),
        })
    }
    
    /// Execute the complete deployment process
    pub fn deploy_all(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("🚀 Starting deployment orchestration for network: {}", self.config.network);
        
        // Step 1: Validate deployment configuration
        self.validate_configuration()?;
        
        // Step 2: Build all contracts
        self.build_contracts()?;
        
        // Step 3: Deploy contracts in dependency order
        self.deploy_contracts_sequenced()?;
        
        // Step 4: Initialize contracts
        self.initialize_contracts()?;
        
        // Step 5: Configure integration router
        self.configure_integration_router()?;
        
        // Step 6: Run verification checks
        self.run_verification_checks()?;
        
        // Step 7: Save deployment registry
        self.save_deployment_registry()?;
        
        println!("✅ Deployment completed successfully!");
        self.print_deployment_summary();
        
        Ok(())
    }
    
    fn validate_configuration(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("🔍 Validating deployment configuration...");
        
        // Check network connectivity
        let output = Command::new("soroban")
            .args(&["network", "ls"])
            .output()?;
        
        if !output.status.success() {
            return Err("Failed to list Soroban networks".into());
        }
        
        let networks = String::from_utf8(output.stdout)?;
        if !networks.contains(&self.config.network) {
            return Err(format!("Network '{}' not configured in Soroban CLI", self.config.network).into());
        }
        
        // Validate contract paths
        for contract in &self.config.contracts {
            if !std::path::Path::new(&contract.path).exists() {
                return Err(format!("Contract path does not exist: {}", contract.path).into());
            }
        }
        
        // Validate dependency graph
        self.validate_dependency_graph()?;
        
        println!("✅ Configuration validation passed");
        Ok(())
    }
    
    fn validate_dependency_graph(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut contract_names: std::collections::HashSet<String> = std::collections::HashSet::new();
        
        // Collect all contract names
        for contract in &self.config.contracts {
            contract_names.insert(contract.name.clone());
        }
        
        // Check that all dependencies exist
        for contract in &self.config.contracts {
            for dep in &contract.dependencies {
                if !contract_names.contains(dep) {
                    return Err(format!("Contract '{}' depends on '{}' which is not defined", contract.name, dep).into());
                }
            }
        }
        
        // Check for circular dependencies (simple check)
        // TODO: Implement proper topological sort for complex dependency graphs
        
        Ok(())
    }
    
    fn build_contracts(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("🔨 Building all contracts...");
        
        for contract in &self.config.contracts {
            println!("  Building {}...", contract.name);
            
            let output = Command::new("soroban")
                .args(&["contract", "build"])
                .current_dir(&contract.path)
                .output()?;
            
            if !output.status.success() {
                let error = String::from_utf8_lossy(&output.stderr);
                return Err(format!("Failed to build contract '{}': {}", contract.name, error).into());
            }
        }
        
        println!("✅ All contracts built successfully");
        Ok(())
    }
    
    fn deploy_contracts_sequenced(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("📦 Deploying contracts in dependency order...");
        
        // Sort contracts by dependency order
        let deployment_order = self.calculate_deployment_order()?;
        
        for contract_name in deployment_order {
            let contract = self.config.contracts.iter()
                .find(|c| c.name == contract_name)
                .ok_or(format!("Contract '{}' not found", contract_name))?;
            
            self.deploy_single_contract(contract)?;
        }
        
        Ok(())
    }
    
    fn calculate_deployment_order(&self) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        // Simple dependency resolution - deploy contracts with no dependencies first
        let mut deployed = std::collections::HashSet::new();
        let mut order = Vec::new();
        let mut remaining: Vec<_> = self.config.contracts.iter().collect();
        
        while !remaining.is_empty() {
            let mut progress = false;
            
            remaining.retain(|contract| {
                let can_deploy = contract.dependencies.iter()
                    .all(|dep| deployed.contains(dep));
                
                if can_deploy {
                    deployed.insert(contract.name.clone());
                    order.push(contract.name.clone());
                    progress = true;
                    false // Remove from remaining
                } else {
                    true // Keep in remaining
                }
            });
            
            if !progress {
                return Err("Circular dependency detected in contract deployment order".into());
            }
        }
        
        Ok(order)
    }
    
    fn deploy_single_contract(&mut self, contract: &ContractConfig) -> Result<(), Box<dyn std::error::Error>> {
        println!("  Deploying {}...", contract.name);
        
        let mut result = DeploymentResult {
            contract_name: contract.name.clone(),
            address: String::new(),
            tx_hash: String::new(),
            deployment_time: current_timestamp(),
            gas_used: 0,
            status: DeploymentStatus::Deploying,
            error_message: None,
        };
        
        // Deploy contract using Soroban CLI
        let wasm_path = format!("{}/target/wasm32-unknown-unknown/release/{}.wasm", 
                               contract.path, contract.name.replace("-", "_"));
        
        let output = Command::new("soroban")
            .args(&[
                "contract", "deploy",
                "--wasm", &wasm_path,
                "--network", &self.config.network,
                "--source", "deployer"
            ])
            .output()?;
        
        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            result.status = DeploymentStatus::Failed;
            result.error_message = Some(error.to_string());
            self.deployment_results.push(result);
            return Err(format!("Failed to deploy contract '{}': {}", contract.name, error).into());
        }
        
        let contract_address = String::from_utf8(output.stdout)?.trim().to_string();
        result.address = contract_address.clone();
        result.status = DeploymentStatus::Deployed;
        
        // Add to registry
        let contract_info = ContractInfo {
            address: contract_address.clone(),
            tx_hash: result.tx_hash.clone(),
            deployed_at: result.deployment_time,
            version: "1.0.0".to_string(), // TODO: Get from Cargo.toml
            abi_hash: calculate_abi_hash(&contract.path)?,
            verified: false,
        };
        
        self.registry.contracts.insert(contract.name.clone(), contract_info);
        self.deployment_results.push(result);
        
        println!("    ✅ Deployed at: {}", contract_address);
        Ok(())
    }
    
    fn initialize_contracts(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("⚙️  Initializing contracts...");
        
        for contract_name in &self.config.initialization_sequence {
            let contract = self.config.contracts.iter()
                .find(|c| c.name == *contract_name)
                .ok_or(format!("Contract '{}' not found in initialization sequence", contract_name))?;
            
            self.initialize_single_contract(contract)?;
        }
        
        Ok(())
    }
    
    fn initialize_single_contract(&mut self, contract: &ContractConfig) -> Result<(), Box<dyn std::error::Error>> {
        println!("  Initializing {}...", contract.name);
        
        let contract_info = self.registry.contracts.get(&contract.name)
            .ok_or(format!("Contract '{}' not found in registry", contract.name))?;
        
        // Build initialization parameters
        let mut init_args = vec![
            "contract".to_string(),
            "invoke".to_string(),
            "--id".to_string(),
            contract_info.address.clone(),
            "--network".to_string(),
            self.config.network.clone(),
            "--source".to_string(),
            "deployer".to_string(),
            "--".to_string(),
            "initialize".to_string(),
        ];
        
        // Add contract-specific parameters
        for (key, value) in &contract.initialization_params {
            // Replace placeholders with actual contract addresses
            let resolved_value = self.resolve_parameter_value(value)?;
            init_args.push(format!("--{}", key));
            init_args.push(resolved_value);
        }
        
        let output = Command::new("soroban")
            .args(&init_args)
            .output()?;
        
        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Failed to initialize contract '{}': {}", contract.name, error).into());
        }
        
        // Update deployment result status
        if let Some(result) = self.deployment_results.iter_mut()
            .find(|r| r.contract_name == contract.name) {
            result.status = DeploymentStatus::Initialized;
        }
        
        println!("    ✅ Initialized successfully");
        Ok(())
    }
    
    fn resolve_parameter_value(&self, value: &str) -> Result<String, Box<dyn std::error::Error>> {
        // Replace contract name placeholders with actual addresses
        if value.starts_with("${") && value.ends_with("}") {
            let contract_name = &value[2..value.len()-1];
            if let Some(contract_info) = self.registry.contracts.get(contract_name) {
                Ok(contract_info.address.clone())
            } else {
                Err(format!("Contract '{}' not found for parameter resolution", contract_name).into())
            }
        } else {
            Ok(value.to_string())
        }
    }
    
    fn configure_integration_router(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("🔗 Configuring integration router...");
        
        let router_info = self.registry.contracts.get("integration_router")
            .ok_or("Integration router not found in registry")?;
        
        self.registry.integration_router = Some(router_info.address.clone());
        
        // Update contract addresses in integration router
        for (contract_name, contract_info) in &self.registry.contracts {
            if contract_name == "integration_router" {
                continue;
            }
            
            let output = Command::new("soroban")
                .args(&[
                    "contract", "invoke",
                    "--id", &router_info.address,
                    "--network", &self.config.network,
                    "--source", "deployer",
                    "--",
                    "update_contract_address",
                    "--name", contract_name,
                    "--address", &contract_info.address,
                ])
                .output()?;
            
            if !output.status.success() {
                let error = String::from_utf8_lossy(&output.stderr);
                return Err(format!("Failed to update contract address for '{}': {}", contract_name, error).into());
            }
        }
        
        println!("✅ Integration router configured");
        Ok(())
    }
    
    fn run_verification_checks(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("🔍 Running verification checks...");
        
        for check in &self.config.verification_checks {
            self.run_single_verification(check)?;
        }
        
        // Mark all contracts as verified
        for result in &mut self.deployment_results {
            if result.status == DeploymentStatus::Initialized {
                result.status = DeploymentStatus::Verified;
            }
        }
        
        println!("✅ All verification checks passed");
        Ok(())
    }
    
    fn run_single_verification(&self, check: &VerificationCheck) -> Result<(), Box<dyn std::error::Error>> {
        println!("  Running check: {}", check.name);
        
        let contract_info = self.registry.contracts.get(&check.contract)
            .ok_or(format!("Contract '{}' not found for verification", check.contract))?;
        
        let output = Command::new("soroban")
            .args(&[
                "contract", "invoke",
                "--id", &contract_info.address,
                "--network", &self.config.network,
                "--source", "deployer",
                "--",
                &check.function,
            ])
            .output()?;
        
        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Verification check '{}' failed: {}", check.name, error).into());
        }
        
        let result = String::from_utf8(output.stdout)?.trim().to_string();
        if result != check.expected_result {
            return Err(format!("Verification check '{}' failed: expected '{}', got '{}'", 
                             check.name, check.expected_result, result).into());
        }
        
        println!("    ✅ Check passed");
        Ok(())
    }
    
    fn save_deployment_registry(&self) -> Result<(), Box<dyn std::error::Error>> {
        let registry_path = format!("deployment_registry_{}.json", self.config.network);
        let registry_json = serde_json::to_string_pretty(&self.registry)?;
        fs::write(&registry_path, registry_json)?;
        
        println!("📝 Deployment registry saved to: {}", registry_path);
        Ok(())
    }
    
    fn print_deployment_summary(&self) {
        println!("\n📊 Deployment Summary");
        println!("====================");
        println!("Network: {}", self.config.network);
        println!("Deployment ID: {}", self.registry.deployment_id);
        println!("Total Contracts: {}", self.deployment_results.len());
        
        for result in &self.deployment_results {
            let status_emoji = match result.status {
                DeploymentStatus::Verified => "✅",
                DeploymentStatus::Failed => "❌",
                _ => "⚠️",
            };
            
            println!("{} {}: {} ({})", 
                    status_emoji, 
                    result.contract_name, 
                    result.address,
                    format!("{:?}", result.status));
        }
        
        if let Some(router_address) = &self.registry.integration_router {
            println!("\n🔗 Integration Router: {}", router_address);
        }
    }
}

// Utility functions
fn generate_deployment_id() -> String {
    format!("deploy_{}", current_timestamp())
}

fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

fn calculate_abi_hash(contract_path: &str) -> Result<String, Box<dyn std::error::Error>> {
    // Simple hash of the contract source for ABI versioning
    // In production, this should be a proper ABI hash
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    
    let mut hasher = DefaultHasher::new();
    contract_path.hash(&mut hasher);
    Ok(format!("{:x}", hasher.finish()))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <config_file>", args[0]);
        std::process::exit(1);
    }
    
    let mut orchestrator = DeploymentOrchestrator::new(&args[1])?;
    orchestrator.deploy_all()?;
    
    Ok(())
}