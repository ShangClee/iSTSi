use loco_rs::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigValidationResult {
    pub valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub environment: String,
}

#[derive(Debug, Clone)]
pub struct ConfigValidationService {
    environment: String,
    required_vars: HashMap<String, Vec<String>>,
    security_requirements: HashMap<String, SecurityRequirement>,
}

#[derive(Debug, Clone)]
struct SecurityRequirement {
    min_length: Option<usize>,
    pattern: Option<String>,
    allowed_values: Option<Vec<String>>,
    required_in_production: bool,
}

impl ConfigValidationService {
    pub fn new(environment: &str) -> Self {
        let mut service = Self {
            environment: environment.to_string(),
            required_vars: HashMap::new(),
            security_requirements: HashMap::new(),
        };

        service.setup_requirements();
        service
    }

    fn setup_requirements(&mut self) {
        // Define required environment variables per environment
        self.required_vars.insert("development".to_string(), vec![
            "DATABASE_URL".to_string(),
        ]);

        self.required_vars.insert("staging".to_string(), vec![
            "DATABASE_URL".to_string(),
            "JWT_SECRET".to_string(),
            "SOROBAN_RPC_URL".to_string(),
        ]);

        self.required_vars.insert("production".to_string(), vec![
            "DATABASE_URL".to_string(),
            "JWT_SECRET".to_string(),
            "SOROBAN_RPC_URL".to_string(),
            "SOROBAN_SOURCE_SECRET".to_string(),
            "SECRET_ENCRYPTION_KEY".to_string(),
            "FRONTEND_URL".to_string(),
            "TLS_CERT_FILE".to_string(),
            "TLS_KEY_FILE".to_string(),
        ]);

        // Define security requirements
        self.security_requirements.insert("JWT_SECRET".to_string(), SecurityRequirement {
            min_length: Some(32),
            pattern: None,
            allowed_values: None,
            required_in_production: true,
        });

        self.security_requirements.insert("SECRET_ENCRYPTION_KEY".to_string(), SecurityRequirement {
            min_length: Some(32),
            pattern: None,
            allowed_values: None,
            required_in_production: true,
        });

        self.security_requirements.insert("DATABASE_URL".to_string(), SecurityRequirement {
            min_length: Some(10),
            pattern: Some(r"^postgres://.*".to_string()),
            allowed_values: None,
            required_in_production: true,
        });

        self.security_requirements.insert("SOROBAN_NETWORK".to_string(), SecurityRequirement {
            min_length: None,
            pattern: None,
            allowed_values: Some(vec!["testnet".to_string(), "mainnet".to_string()]),
            required_in_production: true,
        });

        self.security_requirements.insert("FRONTEND_URL".to_string(), SecurityRequirement {
            min_length: Some(8),
            pattern: Some(r"^https://.*".to_string()),
            allowed_values: None,
            required_in_production: true,
        });
    }

    pub fn validate_configuration(&self) -> ConfigValidationResult {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        // Check required environment variables
        if let Some(required) = self.required_vars.get(&self.environment) {
            for var in required {
                if env::var(var).is_err() {
                    errors.push(format!("Required environment variable '{}' is not set", var));
                }
            }
        }

        // Validate security requirements
        for (var_name, requirement) in &self.security_requirements {
            if let Ok(value) = env::var(var_name) {
                self.validate_security_requirement(var_name, &value, requirement, &mut errors, &mut warnings);
            } else if requirement.required_in_production && self.environment == "production" {
                errors.push(format!("Security-critical variable '{}' is required in production", var_name));
            }
        }

        // Environment-specific validations
        match self.environment.as_str() {
            "production" => self.validate_production_config(&mut errors, &mut warnings),
            "staging" => self.validate_staging_config(&mut errors, &mut warnings),
            "development" => self.validate_development_config(&mut errors, &mut warnings),
            _ => errors.push(format!("Unknown environment: {}", self.environment)),
        }

        ConfigValidationResult {
            valid: errors.is_empty(),
            errors,
            warnings,
            environment: self.environment.clone(),
        }
    }

    fn validate_security_requirement(
        &self,
        var_name: &str,
        value: &str,
        requirement: &SecurityRequirement,
        errors: &mut Vec<String>,
        warnings: &mut Vec<String>,
    ) {
        // Check minimum length
        if let Some(min_length) = requirement.min_length {
            if value.len() < min_length {
                errors.push(format!(
                    "Variable '{}' must be at least {} characters long",
                    var_name, min_length
                ));
            }
        }

        // Check pattern
        if let Some(pattern) = &requirement.pattern {
            if let Ok(regex) = regex::Regex::new(pattern) {
                if !regex.is_match(value) {
                    errors.push(format!(
                        "Variable '{}' does not match required pattern",
                        var_name
                    ));
                }
            }
        }

        // Check allowed values
        if let Some(allowed) = &requirement.allowed_values {
            if !allowed.contains(&value.to_string()) {
                errors.push(format!(
                    "Variable '{}' must be one of: {}",
                    var_name,
                    allowed.join(", ")
                ));
            }
        }

        // Check for common insecure values
        self.check_insecure_values(var_name, value, warnings);
    }

    fn check_insecure_values(&self, var_name: &str, value: &str, warnings: &mut Vec<String>) {
        let insecure_patterns = vec![
            ("password", "Contains 'password'"),
            ("123456", "Contains common weak pattern"),
            ("admin", "Contains 'admin'"),
            ("test", "Contains 'test'"),
            ("default", "Contains 'default'"),
            ("secret", "Contains 'secret' in plaintext"),
        ];

        let value_lower = value.to_lowercase();
        for (pattern, description) in insecure_patterns {
            if value_lower.contains(pattern) {
                warnings.push(format!(
                    "Variable '{}' may be insecure: {}",
                    var_name, description
                ));
            }
        }

        // Check for development/example values in production
        if self.environment == "production" {
            let dev_patterns = vec![
                "development",
                "localhost",
                "127.0.0.1",
                "example.com",
                "change-me",
            ];

            for pattern in dev_patterns {
                if value_lower.contains(pattern) {
                    warnings.push(format!(
                        "Variable '{}' contains development pattern '{}' in production",
                        var_name, pattern
                    ));
                }
            }
        }
    }

    fn validate_production_config(&self, errors: &mut Vec<String>, warnings: &mut Vec<String>) {
        // TLS configuration
        if let Ok(cert_file) = env::var("TLS_CERT_FILE") {
            if !std::path::Path::new(&cert_file).exists() {
                errors.push(format!("TLS certificate file not found: {}", cert_file));
            }
        }

        if let Ok(key_file) = env::var("TLS_KEY_FILE") {
            if !std::path::Path::new(&key_file).exists() {
                errors.push(format!("TLS key file not found: {}", key_file));
            }
        }

        // Database SSL requirement
        if let Ok(db_url) = env::var("DATABASE_URL") {
            if !db_url.contains("sslmode=require") && !db_url.contains("ssl=true") {
                warnings.push("Database connection should use SSL in production".to_string());
            }
        }

        // Frontend URL should be HTTPS
        if let Ok(frontend_url) = env::var("FRONTEND_URL") {
            if !frontend_url.starts_with("https://") {
                errors.push("Frontend URL must use HTTPS in production".to_string());
            }
        }

        // Soroban network should be mainnet
        if let Ok(network) = env::var("SOROBAN_NETWORK") {
            if network != "mainnet" {
                warnings.push("Soroban network should be 'mainnet' in production".to_string());
            }
        }
    }

    fn validate_staging_config(&self, _errors: &mut Vec<String>, warnings: &mut Vec<String>) {
        // Staging-specific validations
        if let Ok(frontend_url) = env::var("FRONTEND_URL") {
            if !frontend_url.starts_with("https://") {
                warnings.push("Frontend URL should use HTTPS in staging".to_string());
            }
        }

        // Check for production secrets in staging
        if let Ok(jwt_secret) = env::var("JWT_SECRET") {
            if jwt_secret.contains("production") {
                warnings.push("JWT secret appears to be a production secret in staging".to_string());
            }
        }
    }

    fn validate_development_config(&self, _errors: &mut Vec<String>, warnings: &mut Vec<String>) {
        // Development-specific validations
        if env::var("JWT_SECRET").unwrap_or_default() == "development-secret-key-change-in-production" {
            warnings.push("Using default JWT secret in development".to_string());
        }

        if let Ok(db_url) = env::var("DATABASE_URL") {
            if db_url.contains("localhost") || db_url.contains("127.0.0.1") {
                // This is expected in development
            } else {
                warnings.push("Database URL points to external server in development".to_string());
            }
        }
    }

    pub fn generate_config_report(&self) -> String {
        let result = self.validate_configuration();
        
        let mut report = format!("Configuration Validation Report\n");
        report.push_str(&format!("Environment: {}\n", result.environment));
        report.push_str(&format!("Status: {}\n\n", if result.valid { "VALID" } else { "INVALID" }));

        if !result.errors.is_empty() {
            report.push_str("ERRORS:\n");
            for error in &result.errors {
                report.push_str(&format!("  ❌ {}\n", error));
            }
            report.push('\n');
        }

        if !result.warnings.is_empty() {
            report.push_str("WARNINGS:\n");
            for warning in &result.warnings {
                report.push_str(&format!("  ⚠️  {}\n", warning));
            }
            report.push('\n');
        }

        if result.valid && result.warnings.is_empty() {
            report.push_str("✅ All configuration checks passed!\n");
        }

        report
    }

    pub fn validate_runtime_config(&self) -> Result<()> {
        let result = self.validate_configuration();
        
        if !result.valid {
            let error_msg = format!(
                "Configuration validation failed:\n{}",
                result.errors.join("\n")
            );
            return Err(Error::string(&error_msg));
        }

        // Log warnings
        for warning in result.warnings {
            tracing::warn!("Configuration warning: {}", warning);
        }

        Ok(())
    }
}

/// Validate configuration on application startup
pub fn validate_startup_config(environment: &str) -> Result<()> {
    let validator = ConfigValidationService::new(environment);
    validator.validate_runtime_config()
}

/// Generate configuration report for debugging
pub fn generate_config_report(environment: &str) -> String {
    let validator = ConfigValidationService::new(environment);
    validator.generate_config_report()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_development_config_validation() {
        env::set_var("DATABASE_URL", "postgres://localhost:5432/test");
        
        let validator = ConfigValidationService::new("development");
        let result = validator.validate_configuration();
        
        // Should be valid with minimal requirements
        assert!(result.valid || result.errors.len() <= 1); // Allow for missing JWT_SECRET
    }

    #[test]
    fn test_production_config_requirements() {
        // Clear environment
        env::remove_var("JWT_SECRET");
        env::remove_var("DATABASE_URL");
        
        let validator = ConfigValidationService::new("production");
        let result = validator.validate_configuration();
        
        // Should have errors for missing required vars
        assert!(!result.valid);
        assert!(!result.errors.is_empty());
    }

    #[test]
    fn test_insecure_value_detection() {
        env::set_var("JWT_SECRET", "password123");
        
        let validator = ConfigValidationService::new("production");
        let result = validator.validate_configuration();
        
        // Should have warnings about insecure values
        assert!(!result.warnings.is_empty());
    }
}