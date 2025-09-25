use bitcoin_custody_backend::models::{
    user::{User, UserRole, KycStatus},
    operation::{Operation, OperationType, OperationStatus},
    kyc_record::{KycRecord, KycTier},
};
use chrono::{DateTime, Utc};
use uuid::Uuid;

pub struct UserFixture;

impl UserFixture {
    pub fn create_test_user() -> User {
        User {
            id: Uuid::new_v4(),
            email: "test@example.com".to_string(),
            password_hash: bcrypt::hash("password123", bcrypt::DEFAULT_COST).unwrap(),
            stellar_address: "GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA".to_string(),
            role: UserRole::User,
            kyc_status: KycStatus::Approved,
            tier: 1,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    pub fn create_admin_user() -> User {
        User {
            id: Uuid::new_v4(),
            email: "admin@example.com".to_string(),
            password_hash: bcrypt::hash("admin123", bcrypt::DEFAULT_COST).unwrap(),
            stellar_address: "GBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB".to_string(),
            role: UserRole::Admin,
            kyc_status: KycStatus::Approved,
            tier: 3,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    pub fn create_pending_kyc_user() -> User {
        User {
            id: Uuid::new_v4(),
            email: "pending@example.com".to_string(),
            password_hash: bcrypt::hash("password123", bcrypt::DEFAULT_COST).unwrap(),
            stellar_address: "GCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCC".to_string(),
            role: UserRole::User,
            kyc_status: KycStatus::Pending,
            tier: 0,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    pub fn create_rejected_kyc_user() -> User {
        User {
            id: Uuid::new_v4(),
            email: "rejected@example.com".to_string(),
            password_hash: bcrypt::hash("password123", bcrypt::DEFAULT_COST).unwrap(),
            stellar_address: "GDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDD".to_string(),
            role: UserRole::User,
            kyc_status: KycStatus::Rejected,
            tier: 0,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
}

pub struct OperationFixture;

impl OperationFixture {
    pub fn create_bitcoin_deposit(user_id: Uuid) -> Operation {
        Operation {
            id: Uuid::new_v4(),
            user_id,
            operation_type: OperationType::BitcoinDeposit,
            status: OperationStatus::Completed,
            amount: 100000000, // 1 BTC
            btc_tx_hash: Some("abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890".to_string()),
            stellar_tx_hash: Some("stellar_tx_hash_123".to_string()),
            btc_address: None,
            confirmations: Some(6),
            error_message: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            completed_at: Some(Utc::now()),
        }
    }

    pub fn create_token_withdrawal(user_id: Uuid) -> Operation {
        Operation {
            id: Uuid::new_v4(),
            user_id,
            operation_type: OperationType::TokenWithdrawal,
            status: OperationStatus::Pending,
            amount: 50000000, // 0.5 BTC worth
            btc_tx_hash: None,
            stellar_tx_hash: Some("stellar_burn_tx_456".to_string()),
            btc_address: Some("bc1qtest123456789abcdef".to_string()),
            confirmations: None,
            error_message: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            completed_at: None,
        }
    }

    pub fn create_failed_operation(user_id: Uuid) -> Operation {
        Operation {
            id: Uuid::new_v4(),
            user_id,
            operation_type: OperationType::BitcoinDeposit,
            status: OperationStatus::Failed,
            amount: 25000000, // 0.25 BTC
            btc_tx_hash: Some("failed_tx_hash_789".to_string()),
            stellar_tx_hash: None,
            btc_address: None,
            confirmations: Some(2), // Insufficient confirmations
            error_message: Some("Insufficient confirmations".to_string()),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            completed_at: None,
        }
    }
}

pub struct KycFixture;

impl KycFixture {
    pub fn create_approved_kyc(user_id: Uuid) -> KycRecord {
        KycRecord {
            id: Uuid::new_v4(),
            user_id,
            tier: KycTier::Tier2,
            status: KycStatus::Approved,
            submitted_documents: serde_json::json!({
                "identity_document": "passport_123.pdf",
                "proof_of_address": "utility_bill_456.pdf"
            }),
            verification_notes: Some("All documents verified successfully".to_string()),
            reviewed_by: Some(Uuid::new_v4()), // Admin user ID
            reviewed_at: Some(Utc::now()),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    pub fn create_pending_kyc(user_id: Uuid) -> KycRecord {
        KycRecord {
            id: Uuid::new_v4(),
            user_id,
            tier: KycTier::Tier1,
            status: KycStatus::Pending,
            submitted_documents: serde_json::json!({
                "identity_document": "drivers_license_789.pdf"
            }),
            verification_notes: None,
            reviewed_by: None,
            reviewed_at: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    pub fn create_rejected_kyc(user_id: Uuid) -> KycRecord {
        KycRecord {
            id: Uuid::new_v4(),
            user_id,
            tier: KycTier::Tier1,
            status: KycStatus::Rejected,
            submitted_documents: serde_json::json!({
                "identity_document": "blurry_photo.jpg"
            }),
            verification_notes: Some("Document quality insufficient for verification".to_string()),
            reviewed_by: Some(Uuid::new_v4()),
            reviewed_at: Some(Utc::now()),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
}

// Test data builders for more complex scenarios
pub struct TestDataBuilder {
    users: Vec<User>,
    operations: Vec<Operation>,
    kyc_records: Vec<KycRecord>,
}

impl TestDataBuilder {
    pub fn new() -> Self {
        Self {
            users: Vec::new(),
            operations: Vec::new(),
            kyc_records: Vec::new(),
        }
    }

    pub fn with_user(mut self, user: User) -> Self {
        self.users.push(user);
        self
    }

    pub fn with_operation(mut self, operation: Operation) -> Self {
        self.operations.push(operation);
        self
    }

    pub fn with_kyc_record(mut self, kyc_record: KycRecord) -> Self {
        self.kyc_records.push(kyc_record);
        self
    }

    pub fn with_complete_user_flow(mut self) -> (Self, Uuid) {
        let user = UserFixture::create_test_user();
        let user_id = user.id;
        
        let kyc_record = KycFixture::create_approved_kyc(user_id);
        let deposit_operation = OperationFixture::create_bitcoin_deposit(user_id);
        let withdrawal_operation = OperationFixture::create_token_withdrawal(user_id);

        self.users.push(user);
        self.kyc_records.push(kyc_record);
        self.operations.push(deposit_operation);
        self.operations.push(withdrawal_operation);

        (self, user_id)
    }

    pub fn build(self) -> TestData {
        TestData {
            users: self.users,
            operations: self.operations,
            kyc_records: self.kyc_records,
        }
    }
}

pub struct TestData {
    pub users: Vec<User>,
    pub operations: Vec<Operation>,
    pub kyc_records: Vec<KycRecord>,
}

impl Default for TestDataBuilder {
    fn default() -> Self {
        Self::new()
    }
}