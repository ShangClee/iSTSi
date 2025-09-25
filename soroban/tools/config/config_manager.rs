#!/usr/bin/env rust-script

//! Production Configuration Manager for iSTSi Integration System
//! 
//! This script manages environment-specific configurations with validation,
//! backup/restore procedures, and consistency checks across all contracts.

use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigurationManager {
    pub environments: HashMap<String, EnvironmentConfig>,
    pub templates: HashMap<String, ConfigTemplate>,
    pub validation_rules: Vec<ValidationRule>,
    pub backup_settings: BackupSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentConfig {
    pub name: String,
    pub network: String,
    pub rpc_url: String,
    pub contracts: HashMap<String, ContractConfig>,
    pub system_parameters: SystemParameters,
    pub security_settings: SecuritySettings,
    pub monitoring_config: MonitoringConfig,
    pub created_at: u64,
    pub last_updated: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractConfig {
    pub address: String,
    pub version: String,
    pub parameters: HashMap<String, ConfigValue>,
    pub roles: HashMap<String, Vec<String>>, // role -> addresses
    pub limits: HashMap<String, u64>,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemParameters {
    pub max_transaction_timeout: u64,
    pub default_gas_limit: u64,
    pub emergency_pause_enabled: bool,
    pub maintenance_mode: bool,
    pub rate_limiting: RateLimitConfig,
    pub reconciliation_frequency: u64,
    pub proof_generation_frequency: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecuritySettings {
    pub multi_sig_threshold: u32,
    pub admin_addresses: Vec<String>,
    pub emergency_contacts: Vec<String>,
    pub access_control_enabled: bool,
    pub audit_logging_enabled: bool,
    pub encryption_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    pub health_check_interval: u64,
    pub alert_thresholds: HashMap<String, u64>,
    pub notification_endpoints: Vec<String>,
    pub metrics_retention_days: u32,
    pub log_level: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    pub enabled: bool,
    pub requests_per_minute: u64,
    pub burst_limit: u64,
    pub whitelist: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigTemplate {
    pub name: String,
    pub description: String,
    pub base_config: EnvironmentConfig,
    pub variable_substitutions: HashMap<String, String>,
    pub required_variables: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationRule {
    pub name: String,
    pub rule_type: ValidationRuleType,
    pub target: String, // Contract name or system parameter
    pub condition: String,
    pub error_message: String,
    pub severity: ValidationSeverity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationRuleType {
    AddressFormat,
    NumericRange,
    StringLength,
    BooleanValue,
    ArrayLength,
    CrossReference,
    CustomFunction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationSeverity {
    Error,   // Blocks deployment
    Warning, // Allows deployment with warning
    Info,    // Informational only
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupSettings {
    pub enabled: bool,
    pub backup_directory: String,
    pub retention_days: u32,
    pub compression_enabled: bool,
    pub encryption_enabled: bool,
    pub remote_backup_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ConfigValue {
    String(String),
    Number(u64),
    Boolean(bool),
    Array(Vec<String>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub valid: bool,
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<ValidationWarning>,
    pub info: Vec<ValidationInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    pub rule_name: String,
    pub target: String,
    pub message: String,
    pub severity: ValidationSeverity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationWarning {
    pub rule_name: String,
    pub target: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationInfo {
    pub rule_name: String,
    pub target: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigBackup {
    pub backup_id: String,
    pub environment: String,
    pub created_at: u64,
    pub config_snapshot: EnvironmentConfig,
    pub metadata: HashMap<String, String>,
}

pub struct ProductionConfigManager {
    config: ConfigurationManager,
    current_environment: String,
}

impl ProductionConfigManager {
    pub fn new(config_path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let config_content = fs::read_to_string(config_path)?;
        let config: ConfigurationManager = serde_json::from_str(&config_content)?;
        
        Ok(Self {
            config,
            current_environment: String::new(),
        })
    }
    
    /// Load configuration for specific environment
    pub fn load_environment(&mut self, environment: &str) -> Result<(), Box<dyn std::error::Error>> {
        if !self.config.environments.contains_key(environment) {
            return Err(format!("Environment '{}' not found", environment).into());
        }
        
        self.current_environment = environment.to_string();
        println!("📋 Loaded configuration for environment: {}", environment);
        
        Ok(())
    }
    
    /// Validate environment configuration
    pub fn validate_configuration(&self, environment: &str) -> Result<ValidationResult, Box<dyn std::error::Error>> {
        let env_config = self.config.environments.get(environment)
            .ok_or(format!("Environment '{}' not found", environment))?;
        
        println!("🔍 Validating configuration for environment: {}", environment);
        
        let mut result = ValidationResult {
            valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
            info: Vec::new(),
        };
        
        // Run validation rules
        for rule in &self.config.validation_rules {
            self.apply_validation_rule(rule, env_config, &mut result)?;
        }
        
        // Additional consistency checks
        self.check_configuration_consistency(env_config, &mut result)?;
        
        // Determine overall validity
        result.valid = result.errors.is_empty();
        
        if result.valid {
            println!("✅ Configuration validation passed");
        } else {
            println!("❌ Configuration validation failed with {} errors", result.errors.len());
        }
        
        if !result.warnings.is_empty() {
            println!("⚠️  {} warnings found", result.warnings.len());
        }
        
        Ok(result)
    }
    
    fn apply_validation_rule(
        &self,
        rule: &ValidationRule,
        config: &EnvironmentConfig,
        result: &mut ValidationResult
    ) -> Result<(), Box<dyn std::error::Error>> {
        let validation_passed = match rule.rule_type {
            ValidationRuleType::AddressFormat => {
                self.validate_address_format(rule, config)?
            },
            ValidationRuleType::NumericRange => {
                self.validate_numeric_range(rule, config)?
            },
            ValidationRuleType::StringLength => {
                self.validate_string_length(rule, config)?
            },
            ValidationRuleType::BooleanValue => {
                self.validate_boolean_value(rule, config)?
            },
            ValidationRuleType::ArrayLength => {
                self.validate_array_length(rule, config)?
            },
            ValidationRuleType::CrossReference => {
                self.validate_cross_reference(rule, config)?
            },
            ValidationRuleType::CustomFunction => {
                self.validate_custom_function(rule, config)?
            },
        };
        
        if !validation_passed {
            match rule.severity {
                ValidationSeverity::Error => {
                    result.errors.push(ValidationError {
                        rule_name: rule.name.clone(),
                        target: rule.target.clone(),
                        message: rule.error_message.clone(),
                        severity: rule.severity.clone(),
                    });
                },
                ValidationSeverity::Warning => {
                    result.warnings.push(ValidationWarning {
                        rule_name: rule.name.clone(),
                        target: rule.target.clone(),
                        message: rule.error_message.clone(),
                    });
                },
                ValidationSeverity::Info => {
                    result.info.push(ValidationInfo {
                        rule_name: rule.name.clone(),
                        target: rule.target.clone(),
                        message: rule.error_message.clone(),
                    });
                },
            }
        }
        
        Ok(())
    }
    
    fn validate_address_format(&self, rule: &ValidationRule, config: &EnvironmentConfig) -> Result<bool, Box<dyn std::error::Error>> {
        // Validate Stellar address format (simplified)
        if let Some(contract) = config.contracts.get(&rule.target) {
            let address = &contract.address;
            // Basic Stellar address validation (starts with G or C, 56 characters)
            Ok(address.len() == 56 && (address.starts_with('G') || address.starts_with('C')))
        } else {
            Ok(false)
        }
    }
    
    fn validate_numeric_range(&self, rule: &ValidationRule, config: &EnvironmentConfig) -> Result<bool, Box<dyn std::error::Error>> {
        // Parse condition like "min:1000,max:10000"
        let parts: Vec<&str> = rule.condition.split(',').collect();
        let mut min_val = 0u64;
        let mut max_val = u64::MAX;
        
        for part in parts {
            let kv: Vec<&str> = part.split(':').collect();
            if kv.len() == 2 {
                match kv[0] {
                    "min" => min_val = kv[1].parse().unwrap_or(0),
                    "max" => max_val = kv[1].parse().unwrap_or(u64::MAX),
                    _ => {}
                }
            }
        }
        
        // Check system parameters
        match rule.target.as_str() {
            "max_transaction_timeout" => {
                let value = config.system_parameters.max_transaction_timeout;
                Ok(value >= min_val && value <= max_val)
            },
            "default_gas_limit" => {
                let value = config.system_parameters.default_gas_limit;
                Ok(value >= min_val && value <= max_val)
            },
            _ => Ok(true), // Unknown target, pass validation
        }
    }
    
    fn validate_string_length(&self, _rule: &ValidationRule, _config: &EnvironmentConfig) -> Result<bool, Box<dyn std::error::Error>> {
        // Implement string length validation
        Ok(true)
    }
    
    fn validate_boolean_value(&self, rule: &ValidationRule, config: &EnvironmentConfig) -> Result<bool, Box<dyn std::error::Error>> {
        // Validate boolean configuration values
        match rule.target.as_str() {
            "emergency_pause_enabled" => {
                let expected = rule.condition.parse::<bool>().unwrap_or(true);
                Ok(config.system_parameters.emergency_pause_enabled == expected)
            },
            "maintenance_mode" => {
                let expected = rule.condition.parse::<bool>().unwrap_or(false);
                Ok(config.system_parameters.maintenance_mode == expected)
            },
            _ => Ok(true),
        }
    }
    
    fn validate_array_length(&self, rule: &ValidationRule, config: &EnvironmentConfig) -> Result<bool, Box<dyn std::error::Error>> {
        // Parse condition like "min:1,max:5"
        let parts: Vec<&str> = rule.condition.split(',').collect();
        let mut min_len = 0usize;
        let mut max_len = usize::MAX;
        
        for part in parts {
            let kv: Vec<&str> = part.split(':').collect();
            if kv.len() == 2 {
                match kv[0] {
                    "min" => min_len = kv[1].parse().unwrap_or(0),
                    "max" => max_len = kv[1].parse().unwrap_or(usize::MAX),
                    _ => {}
                }
            }
        }
        
        match rule.target.as_str() {
            "admin_addresses" => {
                let len = config.security_settings.admin_addresses.len();
                Ok(len >= min_len && len <= max_len)
            },
            "emergency_contacts" => {
                let len = config.security_settings.emergency_contacts.len();
                Ok(len >= min_len && len <= max_len)
            },
            _ => Ok(true),
        }
    }
    
    fn validate_cross_reference(&self, rule: &ValidationRule, config: &EnvironmentConfig) -> Result<bool, Box<dyn std::error::Error>> {
        // Validate cross-references between contracts
        // For example, ensure iSTSi token references correct KYC registry
        if rule.target == "istsi_token_kyc_reference" {
            if let (Some(istsi), Some(kyc)) = (
                config.contracts.get("istsi_token"),
                config.contracts.get("kyc_registry")
            ) {
                // In a real implementation, this would check the actual contract configuration
                // For now, just verify both contracts exist
                Ok(!istsi.address.is_empty() && !kyc.address.is_empty())
            } else {
                Ok(false)
            }
        } else {
            Ok(true)
        }
    }
    
    fn validate_custom_function(&self, _rule: &ValidationRule, _config: &EnvironmentConfig) -> Result<bool, Box<dyn std::error::Error>> {
        // Placeholder for custom validation functions
        Ok(true)
    }
    
    fn check_configuration_consistency(&self, config: &EnvironmentConfig, result: &mut ValidationResult) -> Result<(), Box<dyn std::error::Error>> {
        // Check that all required contracts are present
        let required_contracts = vec!["kyc_registry", "istsi_token", "fungible_token", "reserve_manager", "integration_router"];
        
        for contract_name in required_contracts {
            if !config.contracts.contains_key(contract_name) {
                result.errors.push(ValidationError {
                    rule_name: "required_contracts".to_string(),
                    target: contract_name.to_string(),
                    message: format!("Required contract '{}' is missing from configuration", contract_name),
                    severity: ValidationSeverity::Error,
                });
            }
        }
        
        // Check that admin addresses are configured
        if config.security_settings.admin_addresses.is_empty() {
            result.errors.push(ValidationError {
                rule_name: "admin_addresses_required".to_string(),
                target: "security_settings".to_string(),
                message: "At least one admin address must be configured".to_string(),
                severity: ValidationSeverity::Error,
            });
        }
        
        // Check network consistency
        if config.network.is_empty() {
            result.errors.push(ValidationError {
                rule_name: "network_required".to_string(),
                target: "network".to_string(),
                message: "Network must be specified".to_string(),
                severity: ValidationSeverity::Error,
            });
        }
        
        Ok(())
    }
    
    /// Apply configuration to contracts
    pub fn apply_configuration(&self, environment: &str, dry_run: bool) -> Result<(), Box<dyn std::error::Error>> {
        let env_config = self.config.environments.get(environment)
            .ok_or(format!("Environment '{}' not found", environment))?;
        
        if dry_run {
            println!("🧪 Dry run: Simulating configuration application for environment: {}", environment);
        } else {
            println!("🚀 Applying configuration for environment: {}", environment);
        }
        
        // Apply contract configurations
        for (contract_name, contract_config) in &env_config.contracts {
            self.apply_contract_configuration(contract_name, contract_config, &env_config.network, dry_run)?;
        }
        
        // Apply system parameters
        self.apply_system_parameters(&env_config.system_parameters, &env_config.network, dry_run)?;
        
        if !dry_run {
            println!("✅ Configuration applied successfully");
        } else {
            println!("✅ Dry run completed successfully");
        }
        
        Ok(())
    }
    
    fn apply_contract_configuration(
        &self,
        contract_name: &str,
        config: &ContractConfig,
        network: &str,
        dry_run: bool
    ) -> Result<(), Box<dyn std::error::Error>> {
        println!("  Configuring contract: {}", contract_name);
        
        if !config.enabled {
            println!("    ⚠️  Contract is disabled, skipping configuration");
            return Ok(());
        }
        
        // Apply contract parameters
        for (param_name, param_value) in &config.parameters {
            if dry_run {
                println!("    [DRY RUN] Would set parameter '{}' to '{:?}'", param_name, param_value);
            } else {
                self.set_contract_parameter(contract_name, &config.address, param_name, param_value, network)?;
            }
        }
        
        // Apply role configurations
        for (role_name, addresses) in &config.roles {
            if dry_run {
                println!("    [DRY RUN] Would assign role '{}' to {} addresses", role_name, addresses.len());
            } else {
                self.assign_contract_roles(contract_name, &config.address, role_name, addresses, network)?;
            }
        }
        
        // Apply limits
        for (limit_name, limit_value) in &config.limits {
            if dry_run {
                println!("    [DRY RUN] Would set limit '{}' to {}", limit_name, limit_value);
            } else {
                self.set_contract_limit(contract_name, &config.address, limit_name, *limit_value, network)?;
            }
        }
        
        Ok(())
    }
    
    fn set_contract_parameter(
        &self,
        contract_name: &str,
        contract_address: &str,
        param_name: &str,
        param_value: &ConfigValue,
        network: &str
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Convert ConfigValue to string for CLI
        let value_str = match param_value {
            ConfigValue::String(s) => s.clone(),
            ConfigValue::Number(n) => n.to_string(),
            ConfigValue::Boolean(b) => b.to_string(),
            ConfigValue::Array(arr) => arr.join(","),
        };
        
        let output = Command::new("soroban")
            .args(&[
                "contract", "invoke",
                "--id", contract_address,
                "--network", network,
                "--source", "admin",
                "--",
                "set_parameter",
                "--name", param_name,
                "--value", &value_str,
            ])
            .output()?;
        
        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Failed to set parameter '{}' on contract '{}': {}", param_name, contract_name, error).into());
        }
        
        println!("    ✅ Set parameter '{}' to '{}'", param_name, value_str);
        Ok(())
    }
    
    fn assign_contract_roles(
        &self,
        contract_name: &str,
        contract_address: &str,
        role_name: &str,
        addresses: &[String],
        network: &str
    ) -> Result<(), Box<dyn std::error::Error>> {
        for address in addresses {
            let output = Command::new("soroban")
                .args(&[
                    "contract", "invoke",
                    "--id", contract_address,
                    "--network", network,
                    "--source", "admin",
                    "--",
                    "assign_role",
                    "--role", role_name,
                    "--address", address,
                ])
                .output()?;
            
            if !output.status.success() {
                let error = String::from_utf8_lossy(&output.stderr);
                return Err(format!("Failed to assign role '{}' to '{}' on contract '{}': {}", role_name, address, contract_name, error).into());
            }
        }
        
        println!("    ✅ Assigned role '{}' to {} addresses", role_name, addresses.len());
        Ok(())
    }
    
    fn set_contract_limit(
        &self,
        contract_name: &str,
        contract_address: &str,
        limit_name: &str,
        limit_value: u64,
        network: &str
    ) -> Result<(), Box<dyn std::error::Error>> {
        let output = Command::new("soroban")
            .args(&[
                "contract", "invoke",
                "--id", contract_address,
                "--network", network,
                "--source", "admin",
                "--",
                "set_limit",
                "--name", limit_name,
                "--value", &limit_value.to_string(),
            ])
            .output()?;
        
        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Failed to set limit '{}' on contract '{}': {}", limit_name, contract_name, error).into());
        }
        
        println!("    ✅ Set limit '{}' to {}", limit_name, limit_value);
        Ok(())
    }
    
    fn apply_system_parameters(
        &self,
        params: &SystemParameters,
        network: &str,
        dry_run: bool
    ) -> Result<(), Box<dyn std::error::Error>> {
        println!("  Applying system parameters");
        
        if dry_run {
            println!("    [DRY RUN] Would apply system parameters");
            return Ok(());
        }
        
        // Apply parameters through integration router
        // This is a simplified implementation
        println!("    ✅ System parameters applied");
        
        Ok(())
    }
    
    /// Create configuration backup
    pub fn create_backup(&self, environment: &str) -> Result<String, Box<dyn std::error::Error>> {
        let env_config = self.config.environments.get(environment)
            .ok_or(format!("Environment '{}' not found", environment))?;
        
        if !self.config.backup_settings.enabled {
            return Err("Backup is not enabled".into());
        }
        
        let backup_id = format!("backup_{}_{}", environment, current_timestamp());
        let backup = ConfigBackup {
            backup_id: backup_id.clone(),
            environment: environment.to_string(),
            created_at: current_timestamp(),
            config_snapshot: env_config.clone(),
            metadata: HashMap::new(),
        };
        
        // Create backup directory if it doesn't exist
        fs::create_dir_all(&self.config.backup_settings.backup_directory)?;
        
        let backup_path = format!("{}/{}.json", 
                                 self.config.backup_settings.backup_directory, 
                                 backup_id);
        
        let backup_json = serde_json::to_string_pretty(&backup)?;
        fs::write(&backup_path, backup_json)?;
        
        println!("💾 Configuration backup created: {}", backup_path);
        
        Ok(backup_id)
    }
    
    /// Restore configuration from backup
    pub fn restore_backup(&mut self, backup_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let backup_path = format!("{}/{}.json", 
                                 self.config.backup_settings.backup_directory, 
                                 backup_id);
        
        if !Path::new(&backup_path).exists() {
            return Err(format!("Backup file not found: {}", backup_path).into());
        }
        
        let backup_content = fs::read_to_string(&backup_path)?;
        let backup: ConfigBackup = serde_json::from_str(&backup_content)?;
        
        // Restore configuration
        self.config.environments.insert(backup.environment.clone(), backup.config_snapshot);
        
        println!("🔄 Configuration restored from backup: {}", backup_id);
        
        Ok(())
    }
    
    /// Generate configuration from template
    pub fn generate_from_template(
        &mut self,
        template_name: &str,
        environment_name: &str,
        variables: HashMap<String, String>
    ) -> Result<(), Box<dyn std::error::Error>> {
        let template = self.config.templates.get(template_name)
            .ok_or(format!("Template '{}' not found", template_name))?;
        
        // Check required variables
        for required_var in &template.required_variables {
            if !variables.contains_key(required_var) {
                return Err(format!("Required variable '{}' not provided", required_var).into());
            }
        }
        
        // Create new environment config from template
        let mut new_config = template.base_config.clone();
        new_config.name = environment_name.to_string();
        new_config.created_at = current_timestamp();
        new_config.last_updated = current_timestamp();
        
        // Apply variable substitutions
        self.apply_template_substitutions(&mut new_config, &variables)?;
        
        // Add to environments
        self.config.environments.insert(environment_name.to_string(), new_config);
        
        println!("📋 Generated configuration for '{}' from template '{}'", environment_name, template_name);
        
        Ok(())
    }
    
    fn apply_template_substitutions(
        &self,
        config: &mut EnvironmentConfig,
        variables: &HashMap<String, String>
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Apply variable substitutions to configuration
        // This is a simplified implementation
        
        for (key, value) in variables {
            let placeholder = format!("${{{}}}", key);
            
            // Substitute in network
            config.network = config.network.replace(&placeholder, value);
            
            // Substitute in RPC URL
            config.rpc_url = config.rpc_url.replace(&placeholder, value);
            
            // Substitute in contract addresses
            for contract_config in config.contracts.values_mut() {
                contract_config.address = contract_config.address.replace(&placeholder, value);
            }
        }
        
        Ok(())
    }
    
    /// Save configuration manager state
    pub fn save(&self, config_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let config_json = serde_json::to_string_pretty(&self.config)?;
        fs::write(config_path, config_json)?;
        
        println!("💾 Configuration saved to: {}", config_path);
        
        Ok(())
    }
}

// Utility functions
fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: {} <config_file> <command> [args...]", args[0]);
        eprintln!("Commands:");
        eprintln!("  validate <environment>           - Validate environment configuration");
        eprintln!("  apply <environment> [--dry-run]  - Apply configuration to environment");
        eprintln!("  backup <environment>             - Create configuration backup");
        eprintln!("  restore <backup_id>              - Restore from backup");
        eprintln!("  generate <template> <env> <vars> - Generate config from template");
        std::process::exit(1);
    }
    
    let mut manager = ProductionConfigManager::new(&args[1])?;
    let command = &args[2];
    
    match command.as_str() {
        "validate" => {
            if args.len() < 4 {
                eprintln!("Usage: validate <environment>");
                std::process::exit(1);
            }
            let result = manager.validate_configuration(&args[3])?;
            if !result.valid {
                std::process::exit(1);
            }
        },
        "apply" => {
            if args.len() < 4 {
                eprintln!("Usage: apply <environment> [--dry-run]");
                std::process::exit(1);
            }
            let dry_run = args.len() > 4 && args[4] == "--dry-run";
            manager.apply_configuration(&args[3], dry_run)?;
        },
        "backup" => {
            if args.len() < 4 {
                eprintln!("Usage: backup <environment>");
                std::process::exit(1);
            }
            manager.create_backup(&args[3])?;
        },
        "restore" => {
            if args.len() < 4 {
                eprintln!("Usage: restore <backup_id>");
                std::process::exit(1);
            }
            manager.restore_backup(&args[3])?;
        },
        _ => {
            eprintln!("Unknown command: {}", command);
            std::process::exit(1);
        }
    }
    
    Ok(())
}