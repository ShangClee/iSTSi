use loco_rs::config::Config;
use serde::{Deserialize, Serialize};

/// Extension trait for accessing JWT configuration
pub trait JwtConfig {
    fn get_jwt_secret(&self) -> Option<String>;
    fn get_jwt_expiration(&self) -> Option<u64>;
}

impl JwtConfig for Config {
    fn get_jwt_secret(&self) -> Option<String> {
        // Access the auth.jwt.secret from the config
        std::env::var("JWT_SECRET").ok()
            .or_else(|| Some("development-secret-key-change-in-production".to_string()))
    }

    fn get_jwt_expiration(&self) -> Option<u64> {
        // Default to 24 hours (86400 seconds)
        Some(86400)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthConfig {
    pub jwt: JwtSettings,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JwtSettings {
    pub secret: String,
    pub expiration: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SorobanConfig {
    pub network: String,
    pub rpc_url: String,
    pub network_passphrase: String,
    pub contracts: ContractAddresses,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ContractAddresses {
    pub integration_router: String,
    pub kyc_registry: String,
    pub istsi_token: String,
    pub reserve_manager: String,
}