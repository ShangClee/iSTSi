//! Unified API Layer for Integration Router
//! 
//! This module provides standardized API responses that aggregate data from all integrated contracts,
//! unified query interfaces for external applications, API versioning with backward compatibility,
//! and comprehensive error reporting with component-specific context.

use soroban_sdk::{
    contracttype, Address, Env, Map, Vec, String, BytesN
};

use crate::{IntegrationError, UserRole, RouterConfig, IntegrationEvent, OperationStatus};

// =====================
// API Version Management
// =====================

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ApiVersion {
    V1,
    V2,
    V3,
}

impl ApiVersion {
    pub fn from_string(env: &Env, version_str: &str) -> Result<ApiVersion, IntegrationError> {
        match version_str {
            "v1" | "1" | "1.0" => Ok(ApiVersion::V1),
            "v2" | "2" | "2.0" => Ok(ApiVersion::V2),
            "v3" | "3" | "3.0" => Ok(ApiVersion::V3),
            _ => Err(IntegrationError::InvalidContractResponse),
        }
    }
    
    pub fn to_string(&self, env: &Env) -> String {
        match self {
            ApiVersion::V1 => String::from_str(env, "v1"),
            ApiVersion::V2 => String::from_str(env, "v2"),
            ApiVersion::V3 => String::from_str(env, "v3"),
        }
    }
    
    pub fn is_compatible_with(&self, other: &ApiVersion) -> bool {
        match (self, other) {
            // V1 is compatible with V1 only
            (ApiVersion::V1, ApiVersion::V1) => true,
            // V2 is compatible with V1 and V2
            (ApiVersion::V2, ApiVersion::V1) | (ApiVersion::V2, ApiVersion::V2) => true,
            // V3 is compatible with all versions
            (ApiVersion::V3, _) => true,
            _ => false,
        }
    }
}

// =====================
// Standardized API Response Types
// =====================

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<ApiError>,
    pub metadata: ApiMetadata,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApiError {
    pub code: u32,
    pub message: String,
    pub component: String,
    pub details: Map<String, String>,
    pub correlation_id: Option<BytesN<32>>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApiMetadata {
    pub version: String,
    pub timestamp: u64,
    pub request_id: BytesN<32>,
    pub execution_time_ms: u64,
    pub contract_version: String,
}

// =====================
// Aggregated Data Types
// =====================

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UserProfile {
    pub address: Address,
    pub kyc_status: KycStatus,
    pub token_balances: Map<String, u64>,
    pub transaction_limits: TransactionLimits,
    pub recent_activity: Vec<ActivitySummary>,
    pub compliance_flags: Vec<String>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KycStatus {
    pub tier: u32,
    pub status: String,
    pub verified_at: Option<u64>,
    pub expires_at: Option<u64>,
    pub required_actions: Vec<String>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TransactionLimits {
    pub daily_deposit_limit: u64,
    pub daily_deposit_used: u64,
    pub daily_withdrawal_limit: u64,
    pub daily_withdrawal_used: u64,
    pub monthly_deposit_limit: u64,
    pub monthly_deposit_used: u64,
    pub monthly_withdrawal_limit: u64,
    pub monthly_withdrawal_used: u64,
    pub single_transaction_limit: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActivitySummary {
    pub activity_type: String,
    pub amount: u64,
    pub timestamp: u64,
    pub status: String,
    pub transaction_hash: Option<BytesN<32>>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SystemStatus {
    pub overall_health: String,
    pub component_status: Map<String, ComponentStatus>,
    pub active_operations: u64,
    pub pending_operations: u64,
    pub failed_operations_24h: u64,
    pub system_metrics: SystemMetrics,
    pub alerts: Vec<SystemAlert>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ComponentStatus {
    pub name: String,
    pub status: String, // "healthy", "degraded", "down"
    pub last_check: u64,
    pub response_time_ms: u64,
    pub error_rate_24h: u64,
    pub uptime_percentage: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SystemMetrics {
    pub total_value_locked: u64,
    pub reserve_ratio: u64,
    pub active_users_24h: u64,
    pub transaction_volume_24h: u64,
    pub average_processing_time: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SystemAlert {
    pub alert_id: BytesN<32>,
    pub severity: String, // "info", "warning", "error", "critical"
    pub component: String,
    pub message: String,
    pub created_at: u64,
    pub acknowledged: bool,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TransactionDetails {
    pub transaction_id: BytesN<32>,
    pub transaction_type: String,
    pub user: Address,
    pub amount: u64,
    pub status: String,
    pub created_at: u64,
    pub updated_at: u64,
    pub completion_time: Option<u64>,
    pub fees: u64,
    pub exchange_rate: Option<u64>,
    pub compliance_checks: Vec<ComplianceCheck>,
    pub blockchain_confirmations: Option<u32>,
    pub error_details: Option<String>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ComplianceCheck {
    pub check_type: String,
    pub status: String,
    pub checked_at: u64,
    pub details: String,
}

// =====================
// Query Interface Types
// =====================

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QueryFilter {
    pub user: Option<Address>,
    pub transaction_type: Option<String>,
    pub status: Option<String>,
    pub date_from: Option<u64>,
    pub date_to: Option<u64>,
    pub amount_min: Option<u64>,
    pub amount_max: Option<u64>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PaginatedResponse<T> {
    pub items: Vec<T>,
    pub total_count: u64,
    pub page_size: u32,
    pub current_page: u32,
    pub total_pages: u32,
    pub has_next: bool,
    pub has_previous: bool,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AggregatedStats {
    pub period: String, // "24h", "7d", "30d", "all"
    pub total_transactions: u64,
    pub total_volume: u64,
    pub successful_transactions: u64,
    pub failed_transactions: u64,
    pub average_transaction_size: u64,
    pub unique_users: u64,
    pub by_transaction_type: Map<String, TransactionTypeStats>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TransactionTypeStats {
    pub count: u64,
    pub volume: u64,
    pub success_rate: u64, // percentage
    pub average_processing_time: u64,
}

// =====================
// API Layer Implementation
// =====================

pub struct ApiLayer;

impl ApiLayer {
    /// Create a standardized API response
    pub fn create_response<T>(
        env: &Env,
        data: Option<T>,
        error: Option<ApiError>,
        version: ApiVersion,
        request_id: BytesN<32>,
        execution_time_ms: u64,
    ) -> ApiResponse<T> {
        ApiResponse {
            success: error.is_none(),
            data,
            error,
            metadata: ApiMetadata {
                version: version.to_string(env),
                timestamp: env.ledger().timestamp(),
                request_id,
                execution_time_ms,
                contract_version: String::from_str(env, "1.0.0"),
            },
        }
    }
    
    /// Create an API error with component context
    pub fn create_error(
        env: &Env,
        integration_error: IntegrationError,
        component: &str,
        correlation_id: Option<BytesN<32>>,
        additional_details: Option<Map<String, String>>,
    ) -> ApiError {
        let (code, message) = Self::map_integration_error_to_api(env, integration_error);
        
        let details = additional_details.unwrap_or_else(|| Map::new(env));
        
        ApiError {
            code,
            message,
            component: String::from_str(env, component),
            details,
            correlation_id,
        }
    }
    
    /// Map IntegrationError to API error code and message
    fn map_integration_error_to_api(env: &Env, error: IntegrationError) -> (u32, String) {
        match error {
            IntegrationError::Unauthorized => (401, String::from_str(env, "Unauthorized access")),
            IntegrationError::InsufficientPermissions => (403, String::from_str(env, "Insufficient permissions")),
            IntegrationError::ContractNotFound => (404, String::from_str(env, "Contract not found")),
            IntegrationError::ContractCallFailed => (500, String::from_str(env, "Contract call failed")),
            IntegrationError::InvalidContractResponse => (502, String::from_str(env, "Invalid contract response")),
            IntegrationError::ComplianceCheckFailed => (422, String::from_str(env, "Compliance check failed")),
            IntegrationError::InsufficientKYCTier => (403, String::from_str(env, "Insufficient KYC tier")),
            IntegrationError::AddressBlacklisted => (403, String::from_str(env, "Address is blacklisted")),
            IntegrationError::InsufficientReserves => (422, String::from_str(env, "Insufficient reserves")),
            IntegrationError::ReserveRatioTooLow => (422, String::from_str(env, "Reserve ratio too low")),
            IntegrationError::BitcoinTransactionFailed => (502, String::from_str(env, "Bitcoin transaction failed")),
            IntegrationError::OperationTimeout => (408, String::from_str(env, "Operation timeout")),
            IntegrationError::InvalidOperationState => (422, String::from_str(env, "Invalid operation state")),
            IntegrationError::DuplicateOperation => (409, String::from_str(env, "Duplicate operation")),
            IntegrationError::SystemPaused => (503, String::from_str(env, "System is paused")),
            IntegrationError::EmergencyMode => (503, String::from_str(env, "System in emergency mode")),
            IntegrationError::MaintenanceMode => (503, String::from_str(env, "System in maintenance mode")),
        }
    }
    
    /// Check API version compatibility
    pub fn check_version_compatibility(
        requested_version: &ApiVersion,
        supported_version: &ApiVersion,
    ) -> bool {
        supported_version.is_compatible_with(requested_version)
    }
    
    /// Create paginated response
    pub fn create_paginated_response<T>(
        env: &Env,
        items: Vec<T>,
        total_count: u64,
        page_size: u32,
        current_page: u32,
    ) -> PaginatedResponse<T> {
        let total_pages = if total_count == 0 {
            0
        } else {
            ((total_count - 1) / page_size as u64) + 1
        } as u32;
        
        PaginatedResponse {
            items,
            total_count,
            page_size,
            current_page,
            total_pages,
            has_next: current_page < total_pages,
            has_previous: current_page > 1,
        }
    }
    
    /// Generate request ID for tracking
    pub fn generate_request_id(env: &Env) -> BytesN<32> {
        let timestamp = env.ledger().timestamp();
        let sequence = env.ledger().sequence();
        
        // Create a simple hash from timestamp and sequence
        let mut data = [0u8; 32];
        let timestamp_bytes = timestamp.to_be_bytes();
        let sequence_bytes = sequence.to_be_bytes();
        
        // Fill first 16 bytes with timestamp and sequence data
        data[0..8].copy_from_slice(&timestamp_bytes);
        data[8..12].copy_from_slice(&sequence_bytes);
        
        // Fill remaining bytes with a simple pattern
        for i in 12..32 {
            data[i] = ((i * 7) % 256) as u8;
        }
        
        BytesN::from_array(env, &data)
    }
}