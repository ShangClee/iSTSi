use loco_rs::prelude::*;
use serde::{Deserialize, Serialize};
use sea_orm::*;
use uuid::Uuid;
use rust_decimal::Decimal;
use std::str::FromStr;

use crate::models::_entities::{token_balances, prelude::*};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenBalance {
    pub id: Uuid,
    pub user_id: Uuid,
    pub token_type: String,
    pub balance: Decimal,
    pub locked_balance: Decimal,
    pub last_updated: chrono::NaiveDateTime,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateTokenBalanceRequest {
    pub user_id: Uuid,
    pub token_type: String,
    pub initial_balance: Option<Decimal>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateBalanceRequest {
    pub amount: Decimal,
    pub operation_type: BalanceOperation,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum BalanceOperation {
    Credit,
    Debit,
    Lock,
    Unlock,
}

impl From<token_balances::Model> for TokenBalance {
    fn from(model: token_balances::Model) -> Self {
        Self {
            id: model.id,
            user_id: model.user_id,
            token_type: model.token_type,
            balance: model.balance,
            locked_balance: model.locked_balance,
            last_updated: model.last_updated,
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }
}

impl TokenBalance {
    /// Create a new token balance
    pub async fn create(db: &DatabaseConnection, req: CreateTokenBalanceRequest) -> Result<TokenBalance> {
        // Verify user exists
        let _user = Users::find_by_id(req.user_id)
            .one(db)
            .await?
            .ok_or_else(|| Error::string("User not found"))?;

        // Check if balance already exists for this user and token type
        let existing = Self::find_by_user_and_token(db, req.user_id, &req.token_type).await?;
        if existing.is_some() {
            return Err(Error::string("Token balance already exists for this user and token type"));
        }

        let balance = token_balances::ActiveModel {
            id: Set(Uuid::new_v4()),
            user_id: Set(req.user_id),
            token_type: Set(req.token_type),
            balance: Set(req.initial_balance.unwrap_or_else(|| Decimal::from_str("0").unwrap())),
            locked_balance: Set(Decimal::from_str("0").unwrap()),
            last_updated: Set(chrono::Utc::now().naive_utc()),
            created_at: Set(chrono::Utc::now().naive_utc()),
            updated_at: Set(chrono::Utc::now().naive_utc()),
        };

        let created_balance = balance.insert(db).await?;
        Ok(TokenBalance::from(created_balance))
    }

    /// Find token balance by ID
    pub async fn find_by_id(db: &DatabaseConnection, id: Uuid) -> Result<Option<TokenBalance>> {
        let balance = TokenBalances::find_by_id(id).one(db).await?;
        Ok(balance.map(TokenBalance::from))
    }

    /// Find token balance by user and token type
    pub async fn find_by_user_and_token(
        db: &DatabaseConnection, 
        user_id: Uuid, 
        token_type: &str
    ) -> Result<Option<TokenBalance>> {
        let balance = TokenBalances::find()
            .filter(token_balances::Column::UserId.eq(user_id))
            .filter(token_balances::Column::TokenType.eq(token_type))
            .one(db)
            .await?;

        Ok(balance.map(TokenBalance::from))
    }

    /// Find all token balances for a user
    pub async fn find_by_user_id(db: &DatabaseConnection, user_id: Uuid) -> Result<Vec<TokenBalance>> {
        let balances = TokenBalances::find()
            .filter(token_balances::Column::UserId.eq(user_id))
            .order_by_asc(token_balances::Column::TokenType)
            .all(db)
            .await?;

        Ok(balances.into_iter().map(TokenBalance::from).collect())
    }

    /// Update balance (credit/debit)
    pub async fn update_balance(
        db: &DatabaseConnection,
        user_id: Uuid,
        token_type: &str,
        req: UpdateBalanceRequest,
    ) -> Result<TokenBalance> {
        let balance = TokenBalances::find()
            .filter(token_balances::Column::UserId.eq(user_id))
            .filter(token_balances::Column::TokenType.eq(token_type))
            .one(db)
            .await?
            .ok_or_else(|| Error::string("Token balance not found"))?;

        let mut balance: token_balances::ActiveModel = balance.into();

        match req.operation_type {
            BalanceOperation::Credit => {
                let current_balance = balance.balance.as_ref().clone();
                balance.balance = Set(current_balance + req.amount);
            }
            BalanceOperation::Debit => {
                let current_balance = balance.balance.as_ref().clone();
                let new_balance = current_balance - req.amount;
                
                if new_balance < Decimal::from_str("0").unwrap() {
                    return Err(Error::string("Insufficient balance"));
                }
                
                balance.balance = Set(new_balance);
            }
            BalanceOperation::Lock => {
                let current_balance = balance.balance.as_ref().clone();
                let current_locked = balance.locked_balance.as_ref().clone();
                let available_balance = current_balance - current_locked;
                
                if req.amount > available_balance {
                    return Err(Error::string("Insufficient available balance to lock"));
                }
                
                balance.locked_balance = Set(current_locked + req.amount);
            }
            BalanceOperation::Unlock => {
                let current_locked = balance.locked_balance.as_ref().clone();
                
                if req.amount > current_locked {
                    return Err(Error::string("Cannot unlock more than locked balance"));
                }
                
                balance.locked_balance = Set(current_locked - req.amount);
            }
        }

        balance.last_updated = Set(chrono::Utc::now().naive_utc());
        balance.updated_at = Set(chrono::Utc::now().naive_utc());

        let updated_balance = balance.update(db).await?;
        Ok(TokenBalance::from(updated_balance))
    }

    /// Get available balance (total - locked)
    pub fn available_balance(&self) -> Decimal {
        self.balance - self.locked_balance
    }

    /// Transfer tokens between users
    pub async fn transfer(
        db: &DatabaseConnection,
        from_user_id: Uuid,
        to_user_id: Uuid,
        token_type: &str,
        amount: Decimal,
    ) -> Result<(TokenBalance, TokenBalance)> {
        // For now, implement without transaction to avoid type issues
        // TODO: Implement proper transaction handling
        
        // Debit from sender
        let from_balance = Self::update_balance(
            db,
            from_user_id,
            token_type,
            UpdateBalanceRequest {
                amount,
                operation_type: BalanceOperation::Debit,
            },
        ).await?;

        // Credit to receiver (create balance if doesn't exist)
        let to_balance = match Self::find_by_user_and_token(db, to_user_id, token_type).await? {
            Some(_) => {
                Self::update_balance(
                    db,
                    to_user_id,
                    token_type,
                    UpdateBalanceRequest {
                        amount,
                        operation_type: BalanceOperation::Credit,
                    },
                ).await?
            }
            None => {
                Self::create(
                    db,
                    CreateTokenBalanceRequest {
                        user_id: to_user_id,
                        token_type: token_type.to_string(),
                        initial_balance: Some(amount),
                    },
                ).await?
            }
        };

        Ok((from_balance, to_balance))
    }

    /// Get total balance across all users for a token type
    pub async fn get_total_supply(db: &DatabaseConnection, token_type: &str) -> Result<Decimal> {
        let result = TokenBalances::find()
            .filter(token_balances::Column::TokenType.eq(token_type))
            .select_only()
            .column_as(token_balances::Column::Balance.sum(), "total")
            .into_tuple::<Option<Decimal>>()
            .one(db)
            .await?;

        Ok(result.flatten().unwrap_or_else(|| Decimal::from_str("0").unwrap()))
    }

    /// List all token balances with pagination
    pub async fn list(db: &DatabaseConnection, page: u64, per_page: u64) -> Result<(Vec<TokenBalance>, u64)> {
        let paginator = TokenBalances::find()
            .order_by_desc(token_balances::Column::LastUpdated)
            .paginate(db, per_page);

        let total_pages = paginator.num_pages().await?;
        let balances = paginator.fetch_page(page - 1).await?;

        let balances: Vec<TokenBalance> = balances.into_iter().map(TokenBalance::from).collect();
        Ok((balances, total_pages))
    }

    /// Get balances summary by token type
    pub async fn get_balances_summary(db: &DatabaseConnection) -> Result<Vec<(String, Decimal, u64)>> {
        let results = TokenBalances::find()
            .select_only()
            .column(token_balances::Column::TokenType)
            .column_as(token_balances::Column::Balance.sum(), "total_balance")
            .column_as(token_balances::Column::UserId.count(), "user_count")
            .group_by(token_balances::Column::TokenType)
            .into_tuple::<(String, Option<Decimal>, i64)>()
            .all(db)
            .await?;

        Ok(results
            .into_iter()
            .map(|(token_type, total, count)| {
                (
                    token_type,
                    total.unwrap_or_else(|| Decimal::from_str("0").unwrap()),
                    count as u64,
                )
            })
            .collect())
    }
}