pub mod fixtures;
pub mod integration;
pub mod unit;

// Test utilities and helpers
pub mod helpers {
    use bitcoin_custody_backend::models::user::User;
    use jsonwebtoken::{encode, EncodingKey, Header};
    use serde::{Deserialize, Serialize};
    use std::time::{SystemTime, UNIX_EPOCH};

    #[derive(Debug, Serialize, Deserialize)]
    pub struct Claims {
        pub sub: String,
        pub email: String,
        pub role: String,
        pub exp: usize,
    }

    pub fn generate_test_jwt(user: &User) -> Result<String, jsonwebtoken::errors::Error> {
        let expiration = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() + 3600; // 1 hour

        let claims = Claims {
            sub: user.id.to_string(),
            email: user.email.clone(),
            role: user.role.to_string(),
            exp: expiration as usize,
        };

        let secret = "test-secret-key";
        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(secret.as_ref()),
        )
    }

    pub async fn setup_test_database() -> sea_orm::DatabaseConnection {
        // This would set up an in-memory or test database
        // Implementation depends on your specific database setup
        todo!("Implement test database setup")
    }

    pub fn create_test_config() -> bitcoin_custody_backend::services::soroban_client::SorobanConfig {
        bitcoin_custody_backend::services::soroban_client::SorobanConfig {
            network: "testnet".to_string(),
            rpc_url: "https://soroban-testnet.stellar.org".to_string(),
            network_passphrase: "Test SDF Network ; September 2015".to_string(),
            contracts: bitcoin_custody_backend::services::soroban_client::ContractAddresses {
                integration_router: "CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA1".to_string(),
                kyc_registry: "CBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB2".to_string(),
                istsi_token: "CCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCC3".to_string(),
                reserve_manager: "CDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDD4".to_string(),
            },
        }
    }
}