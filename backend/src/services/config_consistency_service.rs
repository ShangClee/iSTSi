use loco_rs::prelude::*;
use serde::{Deserialize, Serialize};

use std::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsistencyCheckResult {
    pub environment: String,
    pub timestamp: u64,
    pub overall_status: ConsistencyStatus,
    pub checks: Vec<ConsistencyCheck>,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsistencyCheck {
    pub name: String,
    pub status: ConsistencyStatus,
    pub message: String,
    pub details: Option<String>,
    pub severity: Severity,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ConsistencyStatus {
    Pass,
    Warning,
    Fail,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Severity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone)]
pub struct ConfigConsistencyService {
    environment: String,
}

impl ConfigConsistencyService {
    pub fn new(environment: &str) -> Self {
        Self {
            environment: environment.to_string(),
        }
    }

    /// Run comprehensive consistency checks
    pub async fn run_consistency_checks(&self) -> Result<ConsistencyCheckResult> {
        let mut checks = Vec::new();
        
        // Run individual checks
        checks.extend(self.check_environment_variables().await?);
        checks.extend(self.check_configuration_files().await?);
        checks.extend(self.check_service_connectivity().await?);
        checks.extend(self.check_security_configuration().await?);
        checks.extend(self.check_database_configuration().await?);
        checks.extend(self.check_soroban_configuration().await?);
        checks.extend(self.check_cors_configuration().await?);
        checks.extend(self.check_logging_configuration().await?);

        // Determine overall status
        let overall_status = if checks.iter().any(|c| c.status == ConsistencyStatus::Fail) {
            ConsistencyStatus::Fail
        } else if checks.iter().any(|c| c.status == ConsistencyStatus::Warning) {
            ConsistencyStatus::Warning
        } else {
            ConsistencyStatus::Pass
        };

        // Generate recommendations
        let recommendations = self.generate_recommendations(&checks);

        Ok(ConsistencyCheckResult {
            environment: self.environment.clone(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            overall_status,
            checks,
            recommendations,
        })
    }

    /// Check environment variables consistency
    async fn check_environment_variables(&self) -> Result<Vec<ConsistencyCheck>> {
        let mut checks = Vec::new();

        // Required environment variables by environment
        let required_vars = match self.environment.as_str() {
            "production" => vec![
                "DATABASE_URL", "JWT_SECRET", "SOROBAN_SOURCE_SECRET",
                "FRONTEND_URL", "TLS_CERT_FILE", "TLS_KEY_FILE"
            ],
            "staging" => vec![
                "DATABASE_URL", "JWT_SECRET", "SOROBAN_RPC_URL"
            ],
            "development" => vec![
                "DATABASE_URL"
            ],
            _ => vec![],
        };

        for var in required_vars {
            match std::env::var(var) {
                Ok(value) => {
                    if value.is_empty() {
                        checks.push(ConsistencyCheck {
                            name: format!("Environment Variable: {}", var),
                            status: ConsistencyStatus::Fail,
                            message: format!("Variable {} is set but empty", var),
                            details: None,
                            severity: Severity::High,
                        });
                    } else {
                        checks.push(ConsistencyCheck {
                            name: format!("Environment Variable: {}", var),
                            status: ConsistencyStatus::Pass,
                            message: format!("Variable {} is properly set", var),
                            details: None,
                            severity: Severity::Low,
                        });
                    }
                },
                Err(_) => {
                    checks.push(ConsistencyCheck {
                        name: format!("Environment Variable: {}", var),
                        status: ConsistencyStatus::Fail,
                        message: format!("Required variable {} is not set", var),
                        details: Some(format!("Set {} environment variable for {} environment", var, self.environment)),
                        severity: Severity::Critical,
                    });
                }
            }
        }

        // Check for environment-specific security requirements
        if self.environment == "production" {
            if let Ok(jwt_secret) = std::env::var("JWT_SECRET") {
                if jwt_secret.len() < 32 {
                    checks.push(ConsistencyCheck {
                        name: "JWT Secret Strength".to_string(),
                        status: ConsistencyStatus::Fail,
                        message: "JWT secret is too short for production".to_string(),
                        details: Some("JWT secret should be at least 32 characters in production".to_string()),
                        severity: Severity::Critical,
                    });
                }
            }

            if let Ok(frontend_url) = std::env::var("FRONTEND_URL") {
                if !frontend_url.starts_with("https://") {
                    checks.push(ConsistencyCheck {
                        name: "Frontend URL Security".to_string(),
                        status: ConsistencyStatus::Fail,
                        message: "Frontend URL must use HTTPS in production".to_string(),
                        details: Some(format!("Current URL: {}", frontend_url)),
                        severity: Severity::High,
                    });
                }
            }
        }

        Ok(checks)
    }

    /// Check configuration files consistency
    async fn check_configuration_files(&self) -> Result<Vec<ConsistencyCheck>> {
        let mut checks = Vec::new();

        // Backend configuration
        let backend_config_path = format!("backend/config/{}.yaml", self.environment);
        if std::path::Path::new(&backend_config_path).exists() {
            checks.push(ConsistencyCheck {
                name: "Backend Configuration File".to_string(),
                status: ConsistencyStatus::Pass,
                message: format!("Configuration file exists: {}", backend_config_path),
                details: None,
                severity: Severity::Low,
            });

            // Parse and validate backend config
            if let Ok(content) = fs::read_to_string(&backend_config_path) {
                if let Err(e) = serde_yaml::from_str::<serde_yaml::Value>(&content) {
                    checks.push(ConsistencyCheck {
                        name: "Backend Configuration Syntax".to_string(),
                        status: ConsistencyStatus::Fail,
                        message: "Backend configuration has syntax errors".to_string(),
                        details: Some(format!("YAML parse error: {}", e)),
                        severity: Severity::High,
                    });
                } else {
                    checks.push(ConsistencyCheck {
                        name: "Backend Configuration Syntax".to_string(),
                        status: ConsistencyStatus::Pass,
                        message: "Backend configuration syntax is valid".to_string(),
                        details: None,
                        severity: Severity::Low,
                    });
                }
            }
        } else {
            checks.push(ConsistencyCheck {
                name: "Backend Configuration File".to_string(),
                status: ConsistencyStatus::Fail,
                message: format!("Configuration file missing: {}", backend_config_path),
                details: Some("Create the required configuration file".to_string()),
                severity: Severity::Critical,
            });
        }

        // Frontend environment file
        let frontend_env_path = format!("frontend/.env.{}", self.environment);
        if std::path::Path::new(&frontend_env_path).exists() {
            checks.push(ConsistencyCheck {
                name: "Frontend Environment File".to_string(),
                status: ConsistencyStatus::Pass,
                message: format!("Environment file exists: {}", frontend_env_path),
                details: None,
                severity: Severity::Low,
            });
        } else {
            checks.push(ConsistencyCheck {
                name: "Frontend Environment File".to_string(),
                status: ConsistencyStatus::Warning,
                message: format!("Environment file missing: {}", frontend_env_path),
                details: Some("Consider creating environment-specific frontend configuration".to_string()),
                severity: Severity::Medium,
            });
        }

        Ok(checks)
    }

    /// Check service connectivity
    async fn check_service_connectivity(&self) -> Result<Vec<ConsistencyCheck>> {
        let mut checks = Vec::new();

        // Database connectivity
        if let Ok(database_url) = std::env::var("DATABASE_URL") {
            // In a real implementation, you would test the actual database connection
            // For now, we'll just validate the URL format
            if database_url.starts_with("postgres://") || database_url.starts_with("postgresql://") {
                checks.push(ConsistencyCheck {
                    name: "Database URL Format".to_string(),
                    status: ConsistencyStatus::Pass,
                    message: "Database URL format is valid".to_string(),
                    details: None,
                    severity: Severity::Low,
                });
            } else {
                checks.push(ConsistencyCheck {
                    name: "Database URL Format".to_string(),
                    status: ConsistencyStatus::Fail,
                    message: "Database URL format is invalid".to_string(),
                    details: Some("URL should start with postgres:// or postgresql://".to_string()),
                    severity: Severity::High,
                });
            }
        }

        // Soroban RPC connectivity
        if let Ok(soroban_rpc_url) = std::env::var("SOROBAN_RPC_URL") {
            if soroban_rpc_url.starts_with("https://") {
                checks.push(ConsistencyCheck {
                    name: "Soroban RPC URL".to_string(),
                    status: ConsistencyStatus::Pass,
                    message: "Soroban RPC URL uses HTTPS".to_string(),
                    details: None,
                    severity: Severity::Low,
                });
            } else {
                let status = if self.environment == "production" {
                    ConsistencyStatus::Fail
                } else {
                    ConsistencyStatus::Warning
                };

                checks.push(ConsistencyCheck {
                    name: "Soroban RPC URL".to_string(),
                    status,
                    message: "Soroban RPC URL should use HTTPS".to_string(),
                    details: Some(format!("Current URL: {}", soroban_rpc_url)),
                    severity: if self.environment == "production" { Severity::High } else { Severity::Medium },
                });
            }
        }

        Ok(checks)
    }

    /// Check security configuration
    async fn check_security_configuration(&self) -> Result<Vec<ConsistencyCheck>> {
        let mut checks = Vec::new();

        // TLS configuration for production
        if self.environment == "production" {
            if let Ok(cert_file) = std::env::var("TLS_CERT_FILE") {
                if std::path::Path::new(&cert_file).exists() {
                    checks.push(ConsistencyCheck {
                        name: "TLS Certificate".to_string(),
                        status: ConsistencyStatus::Pass,
                        message: "TLS certificate file exists".to_string(),
                        details: None,
                        severity: Severity::Low,
                    });
                } else {
                    checks.push(ConsistencyCheck {
                        name: "TLS Certificate".to_string(),
                        status: ConsistencyStatus::Fail,
                        message: "TLS certificate file not found".to_string(),
                        details: Some(format!("File not found: {}", cert_file)),
                        severity: Severity::Critical,
                    });
                }
            }

            if let Ok(key_file) = std::env::var("TLS_KEY_FILE") {
                if std::path::Path::new(&key_file).exists() {
                    checks.push(ConsistencyCheck {
                        name: "TLS Private Key".to_string(),
                        status: ConsistencyStatus::Pass,
                        message: "TLS private key file exists".to_string(),
                        details: None,
                        severity: Severity::Low,
                    });
                } else {
                    checks.push(ConsistencyCheck {
                        name: "TLS Private Key".to_string(),
                        status: ConsistencyStatus::Fail,
                        message: "TLS private key file not found".to_string(),
                        details: Some(format!("File not found: {}", key_file)),
                        severity: Severity::Critical,
                    });
                }
            }
        }

        Ok(checks)
    }

    /// Check database configuration
    async fn check_database_configuration(&self) -> Result<Vec<ConsistencyCheck>> {
        let mut checks = Vec::new();

        if let Ok(database_url) = std::env::var("DATABASE_URL") {
            // Check SSL mode for production
            if self.environment == "production" {
                if database_url.contains("sslmode=require") || database_url.contains("ssl=true") {
                    checks.push(ConsistencyCheck {
                        name: "Database SSL".to_string(),
                        status: ConsistencyStatus::Pass,
                        message: "Database connection uses SSL".to_string(),
                        details: None,
                        severity: Severity::Low,
                    });
                } else {
                    checks.push(ConsistencyCheck {
                        name: "Database SSL".to_string(),
                        status: ConsistencyStatus::Warning,
                        message: "Database connection should use SSL in production".to_string(),
                        details: Some("Add sslmode=require to DATABASE_URL".to_string()),
                        severity: Severity::Medium,
                    });
                }
            }

            // Check for localhost in production
            if self.environment == "production" && (database_url.contains("localhost") || database_url.contains("127.0.0.1")) {
                checks.push(ConsistencyCheck {
                    name: "Database Host".to_string(),
                    status: ConsistencyStatus::Warning,
                    message: "Database appears to be localhost in production".to_string(),
                    details: Some("Consider using a dedicated database server".to_string()),
                    severity: Severity::Medium,
                });
            }
        }

        Ok(checks)
    }

    /// Check Soroban configuration
    async fn check_soroban_configuration(&self) -> Result<Vec<ConsistencyCheck>> {
        let mut checks = Vec::new();

        // Check network configuration
        if let Ok(network) = std::env::var("SOROBAN_NETWORK") {
            let expected_network = match self.environment.as_str() {
                "production" => "mainnet",
                "staging" => "testnet",
                "development" => "testnet",
                _ => "testnet",
            };

            if network == expected_network {
                checks.push(ConsistencyCheck {
                    name: "Soroban Network".to_string(),
                    status: ConsistencyStatus::Pass,
                    message: format!("Soroban network correctly set to {}", network),
                    details: None,
                    severity: Severity::Low,
                });
            } else {
                checks.push(ConsistencyCheck {
                    name: "Soroban Network".to_string(),
                    status: ConsistencyStatus::Warning,
                    message: format!("Soroban network is {} but expected {} for {}", network, expected_network, self.environment),
                    details: Some(format!("Consider setting SOROBAN_NETWORK to {}", expected_network)),
                    severity: Severity::Medium,
                });
            }
        }

        // Check contract addresses are set for production
        if self.environment == "production" {
            let contract_vars = vec![
                "INTEGRATION_ROUTER_ADDRESS",
                "KYC_REGISTRY_ADDRESS", 
                "ISTSI_TOKEN_ADDRESS",
                "RESERVE_MANAGER_ADDRESS"
            ];

            for var in contract_vars {
                if std::env::var(var).is_ok() {
                    checks.push(ConsistencyCheck {
                        name: format!("Contract Address: {}", var),
                        status: ConsistencyStatus::Pass,
                        message: format!("Contract address {} is set", var),
                        details: None,
                        severity: Severity::Low,
                    });
                } else {
                    checks.push(ConsistencyCheck {
                        name: format!("Contract Address: {}", var),
                        status: ConsistencyStatus::Warning,
                        message: format!("Contract address {} not set for production", var),
                        details: Some("Set contract addresses after deployment".to_string()),
                        severity: Severity::Medium,
                    });
                }
            }
        }

        Ok(checks)
    }

    /// Check CORS configuration
    async fn check_cors_configuration(&self) -> Result<Vec<ConsistencyCheck>> {
        let mut checks = Vec::new();

        // Load backend configuration to check CORS settings
        let backend_config_path = format!("backend/config/{}.yaml", self.environment);
        if let Ok(content) = fs::read_to_string(&backend_config_path) {
            if let Ok(config) = serde_yaml::from_str::<serde_yaml::Value>(&content) {
                // Check CORS allow_origins
                if let Some(cors) = config.get("cors") {
                    if let Some(allow_origins) = cors.get("allow_origins") {
                        if let Some(origins) = allow_origins.as_sequence() {
                            let has_wildcard = origins.iter().any(|o| o.as_str() == Some("*"));
                            
                            if has_wildcard && self.environment == "production" {
                                checks.push(ConsistencyCheck {
                                    name: "CORS Configuration".to_string(),
                                    status: ConsistencyStatus::Fail,
                                    message: "CORS allows all origins (*) in production".to_string(),
                                    details: Some("Restrict CORS to specific domains in production".to_string()),
                                    severity: Severity::High,
                                });
                            } else if has_wildcard {
                                checks.push(ConsistencyCheck {
                                    name: "CORS Configuration".to_string(),
                                    status: ConsistencyStatus::Warning,
                                    message: "CORS allows all origins (*)".to_string(),
                                    details: Some("Consider restricting CORS origins".to_string()),
                                    severity: Severity::Medium,
                                });
                            } else {
                                checks.push(ConsistencyCheck {
                                    name: "CORS Configuration".to_string(),
                                    status: ConsistencyStatus::Pass,
                                    message: "CORS origins are properly restricted".to_string(),
                                    details: None,
                                    severity: Severity::Low,
                                });
                            }
                        }
                    }
                }
            }
        }

        Ok(checks)
    }

    /// Check logging configuration
    async fn check_logging_configuration(&self) -> Result<Vec<ConsistencyCheck>> {
        let mut checks = Vec::new();

        let backend_config_path = format!("backend/config/{}.yaml", self.environment);
        if let Ok(content) = fs::read_to_string(&backend_config_path) {
            if let Ok(config) = serde_yaml::from_str::<serde_yaml::Value>(&content) {
                // Check audit logging
                if let Some(audit) = config.get("audit") {
                    if let Some(enabled) = audit.get("enabled") {
                        if enabled.as_bool() == Some(true) {
                            checks.push(ConsistencyCheck {
                                name: "Audit Logging".to_string(),
                                status: ConsistencyStatus::Pass,
                                message: "Audit logging is enabled".to_string(),
                                details: None,
                                severity: Severity::Low,
                            });
                        } else {
                            let status = if self.environment == "production" {
                                ConsistencyStatus::Warning
                            } else {
                                ConsistencyStatus::Pass
                            };

                            checks.push(ConsistencyCheck {
                                name: "Audit Logging".to_string(),
                                status,
                                message: "Audit logging is disabled".to_string(),
                                details: Some("Consider enabling audit logging for compliance".to_string()),
                                severity: Severity::Medium,
                            });
                        }
                    }
                }

                // Check log level
                if let Some(logger) = config.get("logger") {
                    if let Some(level) = logger.get("level") {
                        if let Some(level_str) = level.as_str() {
                            let appropriate_level = match self.environment.as_str() {
                                "production" => level_str == "info" || level_str == "warn" || level_str == "error",
                                "staging" => level_str == "debug" || level_str == "info",
                                "development" => true, // Any level is fine for development
                                _ => true,
                            };

                            if appropriate_level {
                                checks.push(ConsistencyCheck {
                                    name: "Log Level".to_string(),
                                    status: ConsistencyStatus::Pass,
                                    message: format!("Log level '{}' is appropriate for {}", level_str, self.environment),
                                    details: None,
                                    severity: Severity::Low,
                                });
                            } else {
                                checks.push(ConsistencyCheck {
                                    name: "Log Level".to_string(),
                                    status: ConsistencyStatus::Warning,
                                    message: format!("Log level '{}' may be too verbose for {}", level_str, self.environment),
                                    details: Some("Consider using 'info' or 'warn' level in production".to_string()),
                                    severity: Severity::Medium,
                                });
                            }
                        }
                    }
                }
            }
        }

        Ok(checks)
    }

    /// Generate recommendations based on check results
    fn generate_recommendations(&self, checks: &[ConsistencyCheck]) -> Vec<String> {
        let mut recommendations = Vec::new();

        let failed_checks: Vec<_> = checks.iter().filter(|c| c.status == ConsistencyStatus::Fail).collect();
        let warning_checks: Vec<_> = checks.iter().filter(|c| c.status == ConsistencyStatus::Warning).collect();

        if !failed_checks.is_empty() {
            recommendations.push("🚨 Address all FAILED checks immediately before deployment".to_string());
            
            for check in failed_checks {
                if let Some(details) = &check.details {
                    recommendations.push(format!("   • {}: {}", check.name, details));
                }
            }
        }

        if !warning_checks.is_empty() {
            recommendations.push("⚠️ Review and address WARNING items for improved security and reliability".to_string());
        }

        if self.environment == "production" {
            recommendations.push("🔒 Ensure all security configurations are production-ready".to_string());
            recommendations.push("📊 Set up monitoring and alerting for configuration drift".to_string());
        }

        recommendations.push("🔄 Run consistency checks regularly to catch configuration issues early".to_string());
        recommendations.push("📋 Document any intentional configuration deviations".to_string());

        recommendations
    }

    /// Generate a human-readable report
    pub fn generate_report(&self, result: &ConsistencyCheckResult) -> String {
        let mut report = String::new();
        
        report.push_str(&format!("# Configuration Consistency Report\n\n"));
        report.push_str(&format!("**Environment:** {}\n", result.environment));
        report.push_str(&format!("**Timestamp:** {}\n", 
            chrono::DateTime::from_timestamp(result.timestamp as i64, 0)
                .map(|dt| dt.format("%Y-%m-%d %H:%M:%S UTC").to_string())
                .unwrap_or_else(|| "Unknown".to_string())
        ));
        report.push_str(&format!("**Overall Status:** {:?}\n\n", result.overall_status));

        // Summary
        let pass_count = result.checks.iter().filter(|c| c.status == ConsistencyStatus::Pass).count();
        let warn_count = result.checks.iter().filter(|c| c.status == ConsistencyStatus::Warning).count();
        let fail_count = result.checks.iter().filter(|c| c.status == ConsistencyStatus::Fail).count();

        report.push_str("## Summary\n\n");
        report.push_str(&format!("- ✅ **Passed:** {}\n", pass_count));
        report.push_str(&format!("- ⚠️ **Warnings:** {}\n", warn_count));
        report.push_str(&format!("- ❌ **Failed:** {}\n\n", fail_count));

        // Detailed results
        report.push_str("## Detailed Results\n\n");
        
        for check in &result.checks {
            let icon = match check.status {
                ConsistencyStatus::Pass => "✅",
                ConsistencyStatus::Warning => "⚠️",
                ConsistencyStatus::Fail => "❌",
            };
            
            report.push_str(&format!("{} **{}**: {}\n", icon, check.name, check.message));
            
            if let Some(details) = &check.details {
                report.push_str(&format!("   *{}*\n", details));
            }
            
            report.push('\n');
        }

        // Recommendations
        if !result.recommendations.is_empty() {
            report.push_str("## Recommendations\n\n");
            for recommendation in &result.recommendations {
                report.push_str(&format!("- {}\n", recommendation));
            }
        }

        report
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_consistency_service_creation() {
        let service = ConfigConsistencyService::new("development");
        assert_eq!(service.environment, "development");
    }

    #[tokio::test]
    async fn test_environment_variable_checks() {
        let service = ConfigConsistencyService::new("development");
        let checks = service.check_environment_variables().await.unwrap();
        
        // Should have at least one check for DATABASE_URL
        assert!(!checks.is_empty());
    }
}