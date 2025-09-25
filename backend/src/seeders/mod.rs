use loco_rs::prelude::*;
use sea_orm::*;
use uuid::Uuid;
use bcrypt::{hash, DEFAULT_COST};

use crate::models::_entities::{users, kyc_records, token_balances};

pub struct DatabaseSeeder;

impl DatabaseSeeder {
    /// Seed the database with initial development data
    pub async fn seed_development(db: &DatabaseConnection) -> Result<()> {
        tracing::info!("Starting development database seeding...");

        // Create admin user
        let admin_user = Self::create_admin_user(db).await?;
        tracing::info!("Created admin user: {}", admin_user.email);

        // Create test users
        let test_users = Self::create_test_users(db).await?;
        tracing::info!("Created {} test users", test_users.len());

        // Create initial token balances for test users
        Self::create_initial_token_balances(db, &test_users).await?;
        tracing::info!("Created initial token balances");

        // Create sample KYC records
        Self::create_sample_kyc_records(db, &test_users).await?;
        tracing::info!("Created sample KYC records");

        tracing::info!("Development database seeding completed successfully");
        Ok(())
    }

    /// Create admin user for development
    async fn create_admin_user(db: &DatabaseConnection) -> Result<users::Model> {
        let password_hash = hash("admin123", DEFAULT_COST)
            .map_err(|e| Error::string(&format!("Failed to hash password: {}", e)))?;

        let admin_user = users::ActiveModel {
            id: Set(Uuid::new_v4()),
            email: Set("admin@bitcoincustody.dev".to_string()),
            password: Set(password_hash),
            first_name: Set(Some("Admin".to_string())),
            last_name: Set(Some("User".to_string())),
            role: Set("admin".to_string()),
            is_active: Set(true),
            email_verified: Set(true),
            created_at: Set(chrono::Utc::now().naive_utc()),
            updated_at: Set(chrono::Utc::now().naive_utc()),
        };

        let user = admin_user.insert(db).await?;
        Ok(user)
    }

    /// Create test users for development
    async fn create_test_users(db: &DatabaseConnection) -> Result<Vec<users::Model>> {
        let password_hash = hash("password123", DEFAULT_COST)
            .map_err(|e| Error::string(&format!("Failed to hash password: {}", e)))?;

        let test_users_data = vec![
            ("alice@example.com", "Alice", "Johnson", "user"),
            ("bob@example.com", "Bob", "Smith", "user"),
            ("charlie@example.com", "Charlie", "Brown", "user"),
            ("diana@example.com", "Diana", "Wilson", "compliance_officer"),
        ];

        let mut created_users = Vec::new();

        for (email, first_name, last_name, role) in test_users_data {
            let user = users::ActiveModel {
                id: Set(Uuid::new_v4()),
                email: Set(email.to_string()),
                password: Set(password_hash.clone()),
                first_name: Set(Some(first_name.to_string())),
                last_name: Set(Some(last_name.to_string())),
                role: Set(role.to_string()),
                is_active: Set(true),
                email_verified: Set(true),
                created_at: Set(chrono::Utc::now().naive_utc()),
                updated_at: Set(chrono::Utc::now().naive_utc()),
            };

            let created_user = user.insert(db).await?;
            created_users.push(created_user);
        }

        Ok(created_users)
    }

    /// Create initial token balances for test users
    async fn create_initial_token_balances(db: &DatabaseConnection, users: &[users::Model]) -> Result<()> {
        use rust_decimal::Decimal;
        use std::str::FromStr;

        for user in users {
            // Create iSTSi token balance
            let istsi_balance = token_balances::ActiveModel {
                id: Set(Uuid::new_v4()),
                user_id: Set(user.id),
                token_type: Set("iSTSi".to_string()),
                balance: Set(Decimal::from_str("1000.0").unwrap()),
                locked_balance: Set(Decimal::from_str("0.0").unwrap()),
                last_updated: Set(chrono::Utc::now().naive_utc()),
                created_at: Set(chrono::Utc::now().naive_utc()),
                updated_at: Set(chrono::Utc::now().naive_utc()),
            };

            istsi_balance.insert(db).await?;

            // Create BTC balance
            let btc_balance = token_balances::ActiveModel {
                id: Set(Uuid::new_v4()),
                user_id: Set(user.id),
                token_type: Set("BTC".to_string()),
                balance: Set(Decimal::from_str("0.1").unwrap()),
                locked_balance: Set(Decimal::from_str("0.0").unwrap()),
                last_updated: Set(chrono::Utc::now().naive_utc()),
                created_at: Set(chrono::Utc::now().naive_utc()),
                updated_at: Set(chrono::Utc::now().naive_utc()),
            };

            btc_balance.insert(db).await?;
        }

        Ok(())
    }

    /// Create sample KYC records for test users
    async fn create_sample_kyc_records(db: &DatabaseConnection, users: &[users::Model]) -> Result<()> {
        for (i, user) in users.iter().enumerate() {
            let (status, tier_level) = match i {
                0 => ("approved", 2), // Alice - fully verified
                1 => ("approved", 1), // Bob - basic verification
                2 => ("pending", 0),  // Charlie - pending verification
                _ => ("rejected", 0), // Diana - rejected (for testing)
            };

            let kyc_record = kyc_records::ActiveModel {
                id: Set(Uuid::new_v4()),
                user_id: Set(user.id),
                status: Set(status.to_string()),
                tier_level: Set(tier_level),
                document_type: Set(Some("passport".to_string())),
                document_number: Set(Some(format!("DOC{:06}", i + 1))),
                verification_data: Set(Some(serde_json::json!({
                    "document_verified": status == "approved",
                    "address_verified": status == "approved" && tier_level > 1,
                    "verification_method": "automated"
                }))),
                approved_by: Set(if status == "approved" { Some(users[0].id) } else { None }),
                approved_at: Set(if status == "approved" { Some(chrono::Utc::now().naive_utc()) } else { None }),
                expires_at: Set(if status == "approved" { 
                    Some(chrono::Utc::now().naive_utc() + chrono::Duration::days(365)) 
                } else { 
                    None 
                }),
                created_at: Set(chrono::Utc::now().naive_utc()),
                updated_at: Set(chrono::Utc::now().naive_utc()),
            };

            kyc_record.insert(db).await?;
        }

        Ok(())
    }

    /// Check if database has been seeded
    pub async fn is_seeded(db: &DatabaseConnection) -> Result<bool> {
        let user_count = users::Entity::find().count(db).await?;
        Ok(user_count > 0)
    }

    /// Clear all seeded data (for testing)
    pub async fn clear_all(db: &DatabaseConnection) -> Result<()> {
        tracing::warn!("Clearing all database data...");
        
        // Delete in reverse order of dependencies
        kyc_records::Entity::delete_many().exec(db).await?;
        token_balances::Entity::delete_many().exec(db).await?;
        users::Entity::delete_many().exec(db).await?;
        
        tracing::info!("All database data cleared");
        Ok(())
    }
}