use loco_rs::prelude::*;
use serde::{Deserialize, Serialize};
use sea_orm::*;
use uuid::Uuid;

use crate::models::_entities::{kyc_records, prelude::*};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KycRecord {
    pub id: Uuid,
    pub user_id: Uuid,
    pub status: String,
    pub tier_level: i32,
    pub document_type: Option<String>,
    pub document_number: Option<String>,
    pub verification_data: Option<serde_json::Value>,
    pub approved_by: Option<Uuid>,
    pub approved_at: Option<chrono::NaiveDateTime>,
    pub expires_at: Option<chrono::NaiveDateTime>,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateKycRequest {
    pub user_id: Uuid,
    pub document_type: String,
    pub document_number: String,
    pub verification_data: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateKycRequest {
    pub status: Option<String>,
    pub tier_level: Option<i32>,
    pub verification_data: Option<serde_json::Value>,
    pub approved_by: Option<Uuid>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum KycStatus {
    Pending,
    UnderReview,
    Approved,
    Rejected,
    Expired,
}

impl ToString for KycStatus {
    fn to_string(&self) -> String {
        match self {
            KycStatus::Pending => "pending".to_string(),
            KycStatus::UnderReview => "under_review".to_string(),
            KycStatus::Approved => "approved".to_string(),
            KycStatus::Rejected => "rejected".to_string(),
            KycStatus::Expired => "expired".to_string(),
        }
    }
}

impl From<kyc_records::Model> for KycRecord {
    fn from(model: kyc_records::Model) -> Self {
        Self {
            id: model.id,
            user_id: model.user_id,
            status: model.status,
            tier_level: model.tier_level,
            document_type: model.document_type,
            document_number: model.document_number,
            verification_data: model.verification_data,
            approved_by: model.approved_by,
            approved_at: model.approved_at,
            expires_at: model.expires_at,
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }
}

impl KycRecord {
    /// Create a new KYC record
    pub async fn create(db: &DatabaseConnection, req: CreateKycRequest) -> Result<KycRecord> {
        // Verify user exists
        let _user = Users::find_by_id(req.user_id)
            .one(db)
            .await?
            .ok_or_else(|| Error::string("User not found"))?;

        let kyc_record = kyc_records::ActiveModel {
            id: Set(Uuid::new_v4()),
            user_id: Set(req.user_id),
            status: Set(KycStatus::Pending.to_string()),
            tier_level: Set(0),
            document_type: Set(Some(req.document_type)),
            document_number: Set(Some(req.document_number)),
            verification_data: Set(req.verification_data),
            approved_by: Set(None),
            approved_at: Set(None),
            expires_at: Set(None),
            created_at: Set(chrono::Utc::now().naive_utc()),
            updated_at: Set(chrono::Utc::now().naive_utc()),
        };

        let created_record = kyc_record.insert(db).await?;
        Ok(KycRecord::from(created_record))
    }

    /// Find KYC record by ID
    pub async fn find_by_id(db: &DatabaseConnection, id: Uuid) -> Result<Option<KycRecord>> {
        let record = KycRecords::find_by_id(id).one(db).await?;
        Ok(record.map(KycRecord::from))
    }

    /// Find KYC records by user ID
    pub async fn find_by_user_id(db: &DatabaseConnection, user_id: Uuid) -> Result<Vec<KycRecord>> {
        let records = KycRecords::find()
            .filter(kyc_records::Column::UserId.eq(user_id))
            .order_by_desc(kyc_records::Column::CreatedAt)
            .all(db)
            .await?;

        Ok(records.into_iter().map(KycRecord::from).collect())
    }

    /// Get latest KYC record for user
    pub async fn find_latest_by_user_id(db: &DatabaseConnection, user_id: Uuid) -> Result<Option<KycRecord>> {
        let record = KycRecords::find()
            .filter(kyc_records::Column::UserId.eq(user_id))
            .order_by_desc(kyc_records::Column::CreatedAt)
            .one(db)
            .await?;

        Ok(record.map(KycRecord::from))
    }

    /// Update KYC record
    pub async fn update(db: &DatabaseConnection, id: Uuid, req: UpdateKycRequest) -> Result<KycRecord> {
        let record = KycRecords::find_by_id(id)
            .one(db)
            .await?
            .ok_or_else(|| Error::string("KYC record not found"))?;

        let mut record: kyc_records::ActiveModel = record.into();

        if let Some(status) = req.status {
            record.status = Set(status.clone());
            
            // Set approval timestamp if approved
            if status == KycStatus::Approved.to_string() {
                record.approved_at = Set(Some(chrono::Utc::now().naive_utc()));
                record.expires_at = Set(Some(chrono::Utc::now().naive_utc() + chrono::Duration::days(365)));
            }
        }

        if let Some(tier_level) = req.tier_level {
            record.tier_level = Set(tier_level);
        }

        if let Some(verification_data) = req.verification_data {
            record.verification_data = Set(Some(verification_data));
        }

        if let Some(approved_by) = req.approved_by {
            record.approved_by = Set(Some(approved_by));
        }

        record.updated_at = Set(chrono::Utc::now().naive_utc());

        let updated_record = record.update(db).await?;
        Ok(KycRecord::from(updated_record))
    }

    /// Check if user is KYC approved
    pub async fn is_user_approved(db: &DatabaseConnection, user_id: Uuid) -> Result<bool> {
        let record = Self::find_latest_by_user_id(db, user_id).await?;
        
        if let Some(record) = record {
            Ok(record.status == KycStatus::Approved.to_string() && 
               record.expires_at.map_or(false, |exp| exp > chrono::Utc::now().naive_utc()))
        } else {
            Ok(false)
        }
    }

    /// Get user's KYC tier level
    pub async fn get_user_tier_level(db: &DatabaseConnection, user_id: Uuid) -> Result<i32> {
        let record = Self::find_latest_by_user_id(db, user_id).await?;
        
        if let Some(record) = record {
            if record.status == KycStatus::Approved.to_string() {
                Ok(record.tier_level)
            } else {
                Ok(0)
            }
        } else {
            Ok(0)
        }
    }

    /// List KYC records with pagination
    pub async fn list(db: &DatabaseConnection, page: u64, per_page: u64) -> Result<(Vec<KycRecord>, u64)> {
        let paginator = KycRecords::find()
            .order_by_desc(kyc_records::Column::CreatedAt)
            .paginate(db, per_page);

        let total_pages = paginator.num_pages().await?;
        let records = paginator.fetch_page(page - 1).await?;

        let records: Vec<KycRecord> = records.into_iter().map(KycRecord::from).collect();
        Ok((records, total_pages))
    }

    /// List pending KYC records for review
    pub async fn list_pending(db: &DatabaseConnection) -> Result<Vec<KycRecord>> {
        let records = KycRecords::find()
            .filter(kyc_records::Column::Status.eq(KycStatus::Pending.to_string()))
            .order_by_asc(kyc_records::Column::CreatedAt)
            .all(db)
            .await?;

        Ok(records.into_iter().map(KycRecord::from).collect())
    }

    /// Check for expired KYC records and update status
    pub async fn update_expired_records(db: &DatabaseConnection) -> Result<u64> {
        let now = chrono::Utc::now().naive_utc();
        
        let expired_records = KycRecords::find()
            .filter(kyc_records::Column::Status.eq(KycStatus::Approved.to_string()))
            .filter(kyc_records::Column::ExpiresAt.lt(now))
            .all(db)
            .await?;

        let mut updated_count = 0;

        for record in expired_records {
            let mut record: kyc_records::ActiveModel = record.into();
            record.status = Set(KycStatus::Expired.to_string());
            record.updated_at = Set(now);
            record.update(db).await?;
            updated_count += 1;
        }

        Ok(updated_count)
    }
}