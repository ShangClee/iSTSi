use loco_rs::prelude::*;
use serde::{Deserialize, Serialize};
use sea_orm::*;
use uuid::Uuid;
use bcrypt::{hash, verify, DEFAULT_COST};

use crate::models::_entities::{users, kyc_records, token_balances, operations, prelude::*};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub role: String,
    pub is_active: bool,
    pub email_verified: bool,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateUserRequest {
    pub email: String,
    pub password: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub role: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateUserRequest {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub is_active: Option<bool>,
    pub email_verified: Option<bool>,
}

impl From<users::Model> for User {
    fn from(model: users::Model) -> Self {
        Self {
            id: model.id,
            email: model.email,
            first_name: model.first_name,
            last_name: model.last_name,
            role: model.role,
            is_active: model.is_active,
            email_verified: model.email_verified,
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }
}

impl User {
    /// Create a new user
    pub async fn create(db: &DatabaseConnection, req: CreateUserRequest) -> Result<User> {
        // Check if email already exists
        if Self::find_by_email(db, &req.email).await?.is_some() {
            return Err(Error::string("Email already exists"));
        }

        // Hash password
        let password_hash = hash(&req.password, DEFAULT_COST)
            .map_err(|e| Error::string(&format!("Failed to hash password: {}", e)))?;

        let user = users::ActiveModel {
            id: Set(Uuid::new_v4()),
            email: Set(req.email),
            password: Set(password_hash),
            first_name: Set(req.first_name),
            last_name: Set(req.last_name),
            role: Set(req.role.unwrap_or_else(|| "user".to_string())),
            is_active: Set(true),
            email_verified: Set(false),
            created_at: Set(chrono::Utc::now().naive_utc()),
            updated_at: Set(chrono::Utc::now().naive_utc()),
        };

        let created_user = user.insert(db).await?;
        Ok(User::from(created_user))
    }

    /// Find user by email
    pub async fn find_by_email(db: &DatabaseConnection, email: &str) -> Result<Option<User>> {
        let user = Users::find()
            .filter(users::Column::Email.eq(email))
            .one(db)
            .await?;

        Ok(user.map(User::from))
    }

    /// Find user by ID
    pub async fn find_by_id(db: &DatabaseConnection, id: Uuid) -> Result<Option<User>> {
        let user = Users::find_by_id(id).one(db).await?;
        Ok(user.map(User::from))
    }

    /// Find user with password for authentication
    pub async fn find_by_email_with_password(db: &DatabaseConnection, email: &str) -> Result<Option<users::Model>> {
        let user = Users::find()
            .filter(users::Column::Email.eq(email))
            .one(db)
            .await?;

        Ok(user)
    }

    /// Verify password
    pub fn verify_password(&self, password: &str, hashed_password: &str) -> Result<bool> {
        verify(password, hashed_password)
            .map_err(|e| Error::string(&format!("Password verification failed: {}", e)))
    }

    /// Update user
    pub async fn update(db: &DatabaseConnection, id: Uuid, req: UpdateUserRequest) -> Result<User> {
        let user = Users::find_by_id(id)
            .one(db)
            .await?
            .ok_or_else(|| Error::string("User not found"))?;

        let mut user: users::ActiveModel = user.into();

        if let Some(first_name) = req.first_name {
            user.first_name = Set(Some(first_name));
        }
        if let Some(last_name) = req.last_name {
            user.last_name = Set(Some(last_name));
        }
        if let Some(is_active) = req.is_active {
            user.is_active = Set(is_active);
        }
        if let Some(email_verified) = req.email_verified {
            user.email_verified = Set(email_verified);
        }

        user.updated_at = Set(chrono::Utc::now().naive_utc());

        let updated_user = user.update(db).await?;
        Ok(User::from(updated_user))
    }

    /// Get user with KYC records
    pub async fn find_with_kyc(db: &DatabaseConnection, id: Uuid) -> Result<Option<(User, Vec<kyc_records::Model>)>> {
        let user_with_kyc = Users::find_by_id(id)
            .find_with_related(KycRecords)
            .all(db)
            .await?;

        if let Some((user, kyc_records)) = user_with_kyc.into_iter().next() {
            Ok(Some((User::from(user), kyc_records)))
        } else {
            Ok(None)
        }
    }

    /// Get user with token balances
    pub async fn find_with_balances(db: &DatabaseConnection, id: Uuid) -> Result<Option<(User, Vec<token_balances::Model>)>> {
        let user_with_balances = Users::find_by_id(id)
            .find_with_related(TokenBalances)
            .all(db)
            .await?;

        if let Some((user, balances)) = user_with_balances.into_iter().next() {
            Ok(Some((User::from(user), balances)))
        } else {
            Ok(None)
        }
    }

    /// Get user with operations
    pub async fn find_with_operations(db: &DatabaseConnection, id: Uuid) -> Result<Option<(User, Vec<operations::Model>)>> {
        let user_with_operations = Users::find_by_id(id)
            .find_with_related(Operations)
            .all(db)
            .await?;

        if let Some((user, operations)) = user_with_operations.into_iter().next() {
            Ok(Some((User::from(user), operations)))
        } else {
            Ok(None)
        }
    }

    /// List all users with pagination
    pub async fn list(db: &DatabaseConnection, page: u64, per_page: u64) -> Result<(Vec<User>, u64)> {
        let paginator = Users::find()
            .order_by_desc(users::Column::CreatedAt)
            .paginate(db, per_page);

        let total_pages = paginator.num_pages().await?;
        let users = paginator.fetch_page(page - 1).await?;

        let users: Vec<User> = users.into_iter().map(User::from).collect();
        Ok((users, total_pages))
    }

    /// Delete user (soft delete by setting is_active to false)
    pub async fn soft_delete(db: &DatabaseConnection, id: Uuid) -> Result<()> {
        let user = Users::find_by_id(id)
            .one(db)
            .await?
            .ok_or_else(|| Error::string("User not found"))?;

        let mut user: users::ActiveModel = user.into();
        user.is_active = Set(false);
        user.updated_at = Set(chrono::Utc::now().naive_utc());

        user.update(db).await?;
        Ok(())
    }
}