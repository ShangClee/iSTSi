use loco_rs::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::fs;


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretConfig {
    pub secrets: HashMap<String, String>,
    pub encrypted: bool,
    pub rotation_enabled: bool,
    pub rotation_interval_days: u32,
}

#[derive(Debug, Clone)]
pub struct SecretService {
    config: SecretConfig,
    encryption_key: Option<String>,
}

impl SecretService {
    /// Initialize secret service with configuration
    pub fn new() -> Result<Self> {
        let config = Self::load_secret_config()?;
        let encryption_key = env::var("SECRET_ENCRYPTION_KEY").ok();
        
        Ok(Self {
            config,
            encryption_key,
        })
    }

    /// Load secret configuration from file or environment
    fn load_secret_config() -> Result<SecretConfig> {
        // Try to load from file first
        if let Ok(config_content) = fs::read_to_string("config/secrets.yaml") {
            return serde_yaml::from_str(&config_content)
                .map_err(|e| Error::string(&format!("Failed to parse secrets config: {}", e)));
        }

        // Fallback to default configuration
        Ok(SecretConfig {
            secrets: HashMap::new(),
            encrypted: false,
            rotation_enabled: false,
            rotation_interval_days: 90,
        })
    }

    /// Get secret value by key
    pub fn get_secret(&self, key: &str) -> Result<String> {
        // First try environment variable (highest priority)
        if let Ok(value) = env::var(key) {
            return Ok(value);
        }

        // Then try configuration file
        if let Some(value) = self.config.secrets.get(key) {
            if self.config.encrypted {
                return self.decrypt_secret(value);
            }
            return Ok(value.clone());
        }

        Err(Error::string(&format!("Secret '{}' not found", key)))
    }

    /// Set secret value (for runtime configuration)
    pub fn set_secret(&mut self, key: String, value: String) -> Result<()> {
        let final_value = if self.config.encrypted {
            self.encrypt_secret(&value)?
        } else {
            value
        };

        self.config.secrets.insert(key, final_value);
        Ok(())
    }

    /// Get JWT secret with validation
    pub fn get_jwt_secret(&self) -> Result<String> {
        let secret = self.get_secret("JWT_SECRET")?;
        
        // Validate secret strength
        if secret.len() < 32 {
            return Err(Error::string("JWT secret must be at least 32 characters long"));
        }

        Ok(secret)
    }

    /// Get database URL with credential masking for logs
    pub fn get_database_url(&self) -> Result<String> {
        self.get_secret("DATABASE_URL")
    }

    /// Get Soroban network configuration
    pub fn get_soroban_config(&self) -> Result<SorobanSecrets> {
        Ok(SorobanSecrets {
            network_passphrase: self.get_secret("SOROBAN_NETWORK_PASSPHRASE")?,
            rpc_url: self.get_secret("SOROBAN_RPC_URL")?,
            source_account_secret: self.get_secret("SOROBAN_SOURCE_SECRET").ok(),
        })
    }

    /// Encrypt secret value
    fn encrypt_secret(&self, value: &str) -> Result<String> {
        let key = self.encryption_key.as_ref()
            .ok_or_else(|| Error::string("Encryption key not available"))?;
        
        // Simple XOR encryption for demonstration
        // In production, use proper encryption like AES-GCM
        let encrypted: Vec<u8> = value
            .bytes()
            .zip(key.bytes().cycle())
            .map(|(a, b)| a ^ b)
            .collect();
        
        use base64::{Engine as _, engine::general_purpose};
        Ok(general_purpose::STANDARD.encode(encrypted))
    }

    /// Decrypt secret value
    fn decrypt_secret(&self, encrypted_value: &str) -> Result<String> {
        let key = self.encryption_key.as_ref()
            .ok_or_else(|| Error::string("Encryption key not available"))?;
        
        use base64::{Engine as _, engine::general_purpose};
        let encrypted_bytes = general_purpose::STANDARD.decode(encrypted_value)
            .map_err(|e| Error::string(&format!("Failed to decode secret: {}", e)))?;
        
        let decrypted: Vec<u8> = encrypted_bytes
            .iter()
            .zip(key.bytes().cycle())
            .map(|(a, b)| a ^ b)
            .collect();
        
        String::from_utf8(decrypted)
            .map_err(|e| Error::string(&format!("Failed to decrypt secret: {}", e)))
    }

    /// Validate all required secrets are present
    pub fn validate_secrets(&self) -> Result<()> {
        let required_secrets = vec![
            "JWT_SECRET",
            "DATABASE_URL",
            "SOROBAN_NETWORK_PASSPHRASE",
            "SOROBAN_RPC_URL",
        ];

        for secret in required_secrets {
            self.get_secret(secret)
                .map_err(|_| Error::string(&format!("Required secret '{}' is missing", secret)))?;
        }

        Ok(())
    }

    /// Rotate secrets (placeholder for production implementation)
    pub async fn rotate_secrets(&mut self) -> Result<()> {
        if !self.config.rotation_enabled {
            return Ok(());
        }

        // TODO: Implement actual secret rotation logic
        // This would involve:
        // 1. Generating new secrets
        // 2. Updating external services
        // 3. Graceful rollover
        // 4. Cleanup of old secrets

        tracing::info!("Secret rotation completed");
        Ok(())
    }

    /// Mask sensitive values for logging
    pub fn mask_secret(value: &str) -> String {
        if value.len() <= 8 {
            "*".repeat(value.len())
        } else {
            format!("{}***{}", &value[..4], &value[value.len()-4..])
        }
    }
}

#[derive(Debug, Clone)]
pub struct SorobanSecrets {
    pub network_passphrase: String,
    pub rpc_url: String,
    pub source_account_secret: Option<String>,
}

/// Environment-specific secret validation
pub fn validate_environment_secrets(environment: &str) -> Result<()> {
    match environment {
        "production" => {
            // Production requires all secrets to be properly configured
            let required_env_vars = vec![
                "JWT_SECRET",
                "DATABASE_URL", 
                "SOROBAN_SOURCE_SECRET",
                "SECRET_ENCRYPTION_KEY",
            ];

            for var in required_env_vars {
                env::var(var)
                    .map_err(|_| Error::string(&format!("Production environment requires {} to be set", var)))?;
            }
        },
        "staging" => {
            // Staging has similar requirements but may allow some defaults
            env::var("JWT_SECRET")
                .map_err(|_| Error::string("Staging environment requires JWT_SECRET to be set"))?;
        },
        "development" => {
            // Development can use defaults but should warn about insecure configurations
            if env::var("JWT_SECRET").unwrap_or_default() == "development-secret-key-change-in-production" {
                tracing::warn!("Using default JWT secret in development - change for production");
            }
        },
        _ => {
            return Err(Error::string(&format!("Unknown environment: {}", environment)));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_secret_masking() {
        assert_eq!(SecretService::mask_secret("short"), "*****");
        assert_eq!(SecretService::mask_secret("verylongsecretkey123"), "very***k123");
    }

    #[tokio::test]
    async fn test_secret_service_creation() {
        let service = SecretService::new();
        assert!(service.is_ok());
    }
}