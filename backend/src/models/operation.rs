use loco_rs::prelude::*;
use serde::{Deserialize, Serialize};
use sea_orm::*;
use uuid::Uuid;
use rust_decimal::Decimal;

use crate::models::_entities::{operations, prelude::*};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Operation {
    pub id: Uuid,
    pub user_id: Uuid,
    pub operation_type: String,
    pub status: String,
    pub amount: Option<Decimal>,
    pub token_type: Option<String>,
    pub btc_tx_hash: Option<String>,
    pub soroban_tx_hash: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub error_message: Option<String>,
    pub completed_at: Option<chrono::NaiveDateTime>,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateOperationRequest {
    pub user_id: Uuid,
    pub operation_type: OperationType,
    pub amount: Option<Decimal>,
    pub token_type: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateOperationRequest {
    pub status: Option<OperationStatus>,
    pub btc_tx_hash: Option<String>,
    pub soroban_tx_hash: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub error_message: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum OperationType {
    BitcoinDeposit,
    BitcoinWithdrawal,
    TokenMint,
    TokenBurn,
    TokenTransfer,
    KycVerification,
    ComplianceCheck,
}

impl ToString for OperationType {
    fn to_string(&self) -> String {
        match self {
            OperationType::BitcoinDeposit => "bitcoin_deposit".to_string(),
            OperationType::BitcoinWithdrawal => "bitcoin_withdrawal".to_string(),
            OperationType::TokenMint => "token_mint".to_string(),
            OperationType::TokenBurn => "token_burn".to_string(),
            OperationType::TokenTransfer => "token_transfer".to_string(),
            OperationType::KycVerification => "kyc_verification".to_string(),
            OperationType::ComplianceCheck => "compliance_check".to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum OperationStatus {
    Pending,
    Processing,
    Completed,
    Failed,
    Cancelled,
}

impl ToString for OperationStatus {
    fn to_string(&self) -> String {
        match self {
            OperationStatus::Pending => "pending".to_string(),
            OperationStatus::Processing => "processing".to_string(),
            OperationStatus::Completed => "completed".to_string(),
            OperationStatus::Failed => "failed".to_string(),
            OperationStatus::Cancelled => "cancelled".to_string(),
        }
    }
}

impl From<operations::Model> for Operation {
    fn from(model: operations::Model) -> Self {
        Self {
            id: model.id,
            user_id: model.user_id,
            operation_type: model.operation_type,
            status: model.status,
            amount: model.amount,
            token_type: model.token_type,
            btc_tx_hash: model.btc_tx_hash,
            soroban_tx_hash: model.soroban_tx_hash,
            metadata: model.metadata,
            error_message: model.error_message,
            completed_at: model.completed_at,
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }
}

impl Operation {
    /// Create a new operation
    pub async fn create(db: &DatabaseConnection, req: CreateOperationRequest) -> Result<Operation> {
        // Verify user exists
        let _user = Users::find_by_id(req.user_id)
            .one(db)
            .await?
            .ok_or_else(|| Error::string("User not found"))?;

        let operation = operations::ActiveModel {
            id: Set(Uuid::new_v4()),
            user_id: Set(req.user_id),
            operation_type: Set(req.operation_type.to_string()),
            status: Set(OperationStatus::Pending.to_string()),
            amount: Set(req.amount),
            token_type: Set(req.token_type),
            btc_tx_hash: Set(None),
            soroban_tx_hash: Set(None),
            metadata: Set(req.metadata),
            error_message: Set(None),
            completed_at: Set(None),
            created_at: Set(chrono::Utc::now().naive_utc()),
            updated_at: Set(chrono::Utc::now().naive_utc()),
        };

        let created_operation = operation.insert(db).await?;
        Ok(Operation::from(created_operation))
    }

    /// Find operation by ID
    pub async fn find_by_id(db: &DatabaseConnection, id: Uuid) -> Result<Option<Operation>> {
        let operation = Operations::find_by_id(id).one(db).await?;
        Ok(operation.map(Operation::from))
    }

    /// Find operations by user ID
    pub async fn find_by_user_id(db: &DatabaseConnection, user_id: Uuid) -> Result<Vec<Operation>> {
        let operations = Operations::find()
            .filter(operations::Column::UserId.eq(user_id))
            .order_by_desc(operations::Column::CreatedAt)
            .all(db)
            .await?;

        Ok(operations.into_iter().map(Operation::from).collect())
    }

    /// Find operation by Bitcoin transaction hash
    pub async fn find_by_btc_tx_hash(db: &DatabaseConnection, btc_tx_hash: &str) -> Result<Option<Operation>> {
        let operation = Operations::find()
            .filter(operations::Column::BtcTxHash.eq(btc_tx_hash))
            .one(db)
            .await?;

        Ok(operation.map(Operation::from))
    }

    /// Find operation by Soroban transaction hash
    pub async fn find_by_soroban_tx_hash(db: &DatabaseConnection, soroban_tx_hash: &str) -> Result<Option<Operation>> {
        let operation = Operations::find()
            .filter(operations::Column::SorobanTxHash.eq(soroban_tx_hash))
            .one(db)
            .await?;

        Ok(operation.map(Operation::from))
    }

    /// Update operation
    pub async fn update(db: &DatabaseConnection, id: Uuid, req: UpdateOperationRequest) -> Result<Operation> {
        let operation = Operations::find_by_id(id)
            .one(db)
            .await?
            .ok_or_else(|| Error::string("Operation not found"))?;

        let mut operation: operations::ActiveModel = operation.into();

        if let Some(status) = req.status {
            operation.status = Set(status.to_string());
            
            // Set completion timestamp if completed
            if matches!(status, OperationStatus::Completed | OperationStatus::Failed | OperationStatus::Cancelled) {
                operation.completed_at = Set(Some(chrono::Utc::now().naive_utc()));
            }
        }

        if let Some(btc_tx_hash) = req.btc_tx_hash {
            operation.btc_tx_hash = Set(Some(btc_tx_hash));
        }

        if let Some(soroban_tx_hash) = req.soroban_tx_hash {
            operation.soroban_tx_hash = Set(Some(soroban_tx_hash));
        }

        if let Some(metadata) = req.metadata {
            operation.metadata = Set(Some(metadata));
        }

        if let Some(error_message) = req.error_message {
            operation.error_message = Set(Some(error_message));
        }

        operation.updated_at = Set(chrono::Utc::now().naive_utc());

        let updated_operation = operation.update(db).await?;
        Ok(Operation::from(updated_operation))
    }

    /// List operations with pagination
    pub async fn list(db: &DatabaseConnection, page: u64, per_page: u64) -> Result<(Vec<Operation>, u64)> {
        let paginator = Operations::find()
            .order_by_desc(operations::Column::CreatedAt)
            .paginate(db, per_page);

        let total_pages = paginator.num_pages().await?;
        let operations = paginator.fetch_page(page - 1).await?;

        let operations: Vec<Operation> = operations.into_iter().map(Operation::from).collect();
        Ok((operations, total_pages))
    }

    /// List operations by status
    pub async fn list_by_status(db: &DatabaseConnection, status: OperationStatus) -> Result<Vec<Operation>> {
        let operations = Operations::find()
            .filter(operations::Column::Status.eq(status.to_string()))
            .order_by_asc(operations::Column::CreatedAt)
            .all(db)
            .await?;

        Ok(operations.into_iter().map(Operation::from).collect())
    }

    /// List operations by type
    pub async fn list_by_type(db: &DatabaseConnection, operation_type: OperationType) -> Result<Vec<Operation>> {
        let operations = Operations::find()
            .filter(operations::Column::OperationType.eq(operation_type.to_string()))
            .order_by_desc(operations::Column::CreatedAt)
            .all(db)
            .await?;

        Ok(operations.into_iter().map(Operation::from).collect())
    }

    /// Get operations statistics
    pub async fn get_statistics(db: &DatabaseConnection) -> Result<OperationStatistics> {
        let total_operations = Operations::find().count(db).await?;
        
        let pending_operations = Operations::find()
            .filter(operations::Column::Status.eq(OperationStatus::Pending.to_string()))
            .count(db)
            .await?;
        
        let completed_operations = Operations::find()
            .filter(operations::Column::Status.eq(OperationStatus::Completed.to_string()))
            .count(db)
            .await?;
        
        let failed_operations = Operations::find()
            .filter(operations::Column::Status.eq(OperationStatus::Failed.to_string()))
            .count(db)
            .await?;

        // Get operations by type
        let type_stats = Operations::find()
            .select_only()
            .column(operations::Column::OperationType)
            .column_as(operations::Column::Id.count(), "count")
            .group_by(operations::Column::OperationType)
            .into_tuple::<(String, i64)>()
            .all(db)
            .await?;

        Ok(OperationStatistics {
            total_operations,
            pending_operations,
            completed_operations,
            failed_operations,
            operations_by_type: type_stats.into_iter().collect(),
        })
    }

    /// Mark operation as processing
    pub async fn mark_processing(db: &DatabaseConnection, id: Uuid) -> Result<Operation> {
        Self::update(
            db,
            id,
            UpdateOperationRequest {
                status: Some(OperationStatus::Processing),
                btc_tx_hash: None,
                soroban_tx_hash: None,
                metadata: None,
                error_message: None,
            },
        ).await
    }

    /// Mark operation as completed
    pub async fn mark_completed(
        db: &DatabaseConnection,
        id: Uuid,
        btc_tx_hash: Option<String>,
        soroban_tx_hash: Option<String>,
    ) -> Result<Operation> {
        Self::update(
            db,
            id,
            UpdateOperationRequest {
                status: Some(OperationStatus::Completed),
                btc_tx_hash,
                soroban_tx_hash,
                metadata: None,
                error_message: None,
            },
        ).await
    }

    /// Mark operation as failed
    pub async fn mark_failed(db: &DatabaseConnection, id: Uuid, error_message: String) -> Result<Operation> {
        Self::update(
            db,
            id,
            UpdateOperationRequest {
                status: Some(OperationStatus::Failed),
                btc_tx_hash: None,
                soroban_tx_hash: None,
                metadata: None,
                error_message: Some(error_message),
            },
        ).await
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OperationStatistics {
    pub total_operations: u64,
    pub pending_operations: u64,
    pub completed_operations: u64,
    pub failed_operations: u64,
    pub operations_by_type: Vec<(String, i64)>,
}