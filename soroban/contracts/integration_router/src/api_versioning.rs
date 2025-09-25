//! API Versioning and Backward Compatibility
//! 
//! This module handles API versioning with backward compatibility for existing integrations,
//! ensuring that older API versions continue to work while new features are added.

use soroban_sdk::{
    contractimpl, Address, Env, Map, Vec, String, BytesN
};

use crate::{
    IntegrationRouter, IntegrationError,
    api_layer::{
        ApiResponse, ApiError, ApiLayer, ApiVersion, UserProfile, SystemStatus,
        TransactionDetails, PaginatedResponse, AggregatedStats, QueryFilter
    }
};

// =====================
// Version-Specific Response Types
// =====================

/// V1 API Response Types (Legacy)
pub mod v1 {
    use soroban_sdk::{contracttype, Address, Map, Vec, String, BytesN};
    
    #[contracttype]
    #[derive(Clone, Debug, Eq, PartialEq)]
    pub struct UserProfileV1 {
        pub address: Address,
        pub kyc_tier: u32,
        pub istsi_balance: u64,
        pub daily_limit: u64,
        pub daily_used: u64,
    }
    
    #[contracttype]
    #[derive(Clone, Debug, Eq, PartialEq)]
    pub struct SystemStatusV1 {
        pub status: String,
        pub paused: bool,
        pub total_operations: u64,
        pub failed_operations: u64,
    }
    
    #[contracttype]
    #[derive(Clone, Debug, Eq, PartialEq)]
    pub struct TransactionV1 {
        pub id: BytesN<32>,
        pub user: Address,
        pub amount: u64,
        pub status: String,
        pub timestamp: u64,
    }
}

/// V2 API Response Types (Enhanced)
pub mod v2 {
    use soroban_sdk::{contracttype, Address, Map, Vec, String, BytesN};
    
    #[contracttype]
    #[derive(Clone, Debug, Eq, PartialEq)]
    pub struct UserProfileV2 {
        pub address: Address,
        pub kyc_status: KycStatusV2,
        pub token_balances: Map<String, u64>,
        pub transaction_limits: TransactionLimitsV2,
        pub recent_activity: Vec<ActivityV2>,
    }
    
    #[contracttype]
    #[derive(Clone, Debug, Eq, PartialEq)]
    pub struct KycStatusV2 {
        pub tier: u32,
        pub status: String,
        pub verified_at: Option<u64>,
        pub expires_at: Option<u64>,
    }
    
    #[contracttype]
    #[derive(Clone, Debug, Eq, PartialEq)]
    pub struct TransactionLimitsV2 {
        pub daily_deposit_limit: u64,
        pub daily_deposit_used: u64,
        pub daily_withdrawal_limit: u64,
        pub daily_withdrawal_used: u64,
        pub monthly_deposit_limit: u64,
        pub monthly_deposit_used: u64,
        pub monthly_withdrawal_limit: u64,
        pub monthly_withdrawal_used: u64,
    }
    
    #[contracttype]
    #[derive(Clone, Debug, Eq, PartialEq)]
    pub struct ActivityV2 {
        pub activity_type: String,
        pub amount: u64,
        pub timestamp: u64,
        pub status: String,
    }
    
    #[contracttype]
    #[derive(Clone, Debug, Eq, PartialEq)]
    pub struct SystemStatusV2 {
        pub overall_health: String,
        pub components: Map<String, String>,
        pub active_operations: u64,
        pub pending_operations: u64,
        pub failed_operations_24h: u64,
        pub metrics: SystemMetricsV2,
    }
    
    #[contracttype]
    #[derive(Clone, Debug, Eq, PartialEq)]
    pub struct SystemMetricsV2 {
        pub total_value_locked: u64,
        pub reserve_ratio: u64,
        pub active_users_24h: u64,
        pub transaction_volume_24h: u64,
    }
}

#[contractimpl]
impl IntegrationRouter {
    
    // =====================
    // V1 API Endpoints (Legacy Support)
    // =====================
    
    /// Get user profile (V1 API - simplified response)
    pub fn get_user_profile_v1(
        env: Env,
        caller: Address,
        user: Address,
    ) -> ApiResponse<v1::UserProfileV1> {
        let start_time = env.ledger().timestamp();
        let request_id = ApiLayer::generate_request_id(&env);
        
        // Check permissions
        if caller != user {
            let caller_role = Self::get_user_role_internal(&env, &caller);
            match caller_role {
                crate::UserRole::SuperAdmin | crate::UserRole::SystemAdmin | crate::UserRole::ComplianceOfficer => {
                    caller.require_auth();
                },
                _ => {
                    let error = ApiLayer::create_error(&env, IntegrationError::InsufficientPermissions, "authorization", Some(request_id.clone()), None);
                    return ApiLayer::create_response(None, Some(error), ApiVersion::V1, request_id, 0);
                }
            }
        } else {
            caller.require_auth();
        }
        
        // Get full profile and convert to V1 format
        let full_profile = match Self::aggregate_user_profile(&env, &user) {
            Ok(profile) => profile,
            Err(e) => {
                let error = ApiLayer::create_error(&env, e, "data_aggregation", Some(request_id.clone()), None);
                return ApiLayer::create_response(None, Some(error), ApiVersion::V1, request_id, 0);
            }
        };
        
        // Convert to V1 format (simplified)
        let v1_profile = v1::UserProfileV1 {
            address: full_profile.address,
            kyc_tier: full_profile.kyc_status.tier,
            istsi_balance: full_profile.token_balances.get(String::from_str(&env, "iSTSi")).unwrap_or(0),
            daily_limit: full_profile.transaction_limits.daily_deposit_limit,
            daily_used: full_profile.transaction_limits.daily_deposit_used,
        };
        
        let execution_time = env.ledger().timestamp() - start_time;
        ApiLayer::create_response(Some(v1_profile), None, ApiVersion::V1, request_id, execution_time)
    }
    
    /// Get system status (V1 API - simplified response)
    pub fn get_system_status_v1(
        env: Env,
        caller: Address,
    ) -> ApiResponse<v1::SystemStatusV1> {
        let start_time = env.ledger().timestamp();
        let request_id = ApiLayer::generate_request_id(&env);
        
        // Check permissions
        let caller_role = Self::get_user_role_internal(&env, &caller);
        match caller_role {
            crate::UserRole::SuperAdmin | crate::UserRole::SystemAdmin | crate::UserRole::ComplianceOfficer => {
                caller.require_auth();
            },
            _ => {
                let error = ApiLayer::create_error(&env, IntegrationError::InsufficientPermissions, "authorization", Some(request_id.clone()), None);
                return ApiLayer::create_response(None, Some(error), ApiVersion::V1, request_id, 0);
            }
        }
        
        // Get full status and convert to V1 format
        let full_status = match Self::aggregate_system_status(&env) {
            Ok(status) => status,
            Err(e) => {
                let error = ApiLayer::create_error(&env, e, "system_monitoring", Some(request_id.clone()), None);
                return ApiLayer::create_response(None, Some(error), ApiVersion::V1, request_id, 0);
            }
        };
        
        // Convert to V1 format (simplified)
        let v1_status = v1::SystemStatusV1 {
            status: full_status.overall_health,
            paused: Self::is_paused(env.clone()),
            total_operations: full_status.active_operations + full_status.pending_operations,
            failed_operations: full_status.failed_operations_24h,
        };
        
        let execution_time = env.ledger().timestamp() - start_time;
        ApiLayer::create_response(Some(v1_status), None, ApiVersion::V1, request_id, execution_time)
    }
    
    /// Get user transactions (V1 API - simplified response)
    pub fn get_user_transactions_v1(
        env: Env,
        caller: Address,
        user: Address,
        limit: u32,
    ) -> ApiResponse<Vec<v1::TransactionV1>> {
        let start_time = env.ledger().timestamp();
        let request_id = ApiLayer::generate_request_id(&env);
        
        // Check permissions
        if caller != user {
            let caller_role = Self::get_user_role_internal(&env, &caller);
            match caller_role {
                crate::UserRole::SuperAdmin | crate::UserRole::SystemAdmin | crate::UserRole::ComplianceOfficer => {
                    caller.require_auth();
                },
                _ => {
                    let error = ApiLayer::create_error(&env, IntegrationError::InsufficientPermissions, "authorization", Some(request_id.clone()), None);
                    return ApiLayer::create_response(None, Some(error), ApiVersion::V1, request_id, 0);
                }
            }
        } else {
            caller.require_auth();
        }
        
        // Create filter for V1 API
        let filter = QueryFilter {
            user: Some(user.clone()),
            transaction_type: None,
            status: None,
            date_from: None,
            date_to: None,
            amount_min: None,
            amount_max: None,
            limit: Some(limit),
            offset: Some(0),
        };
        
        // Get full transactions and convert to V1 format
        let full_transactions = match Self::get_filtered_transactions(&env, &user, &filter) {
            Ok(transactions) => transactions,
            Err(e) => {
                let error = ApiLayer::create_error(&env, e, "transaction_query", Some(request_id.clone()), None);
                return ApiLayer::create_response(None, Some(error), ApiVersion::V1, request_id, 0);
            }
        };
        
        // Convert to V1 format (simplified)
        let mut v1_transactions = Vec::new(&env);
        for transaction in full_transactions.items.iter() {
            v1_transactions.push_back(v1::TransactionV1 {
                id: transaction.transaction_id.clone(),
                user: transaction.user.clone(),
                amount: transaction.amount,
                status: transaction.status.clone(),
                timestamp: transaction.created_at,
            });
        }
        
        let execution_time = env.ledger().timestamp() - start_time;
        ApiLayer::create_response(Some(v1_transactions), None, ApiVersion::V1, request_id, execution_time)
    }
    
    // =====================
    // V2 API Endpoints (Enhanced)
    // =====================
    
    /// Get user profile (V2 API - enhanced response)
    pub fn get_user_profile_v2(
        env: Env,
        caller: Address,
        user: Address,
    ) -> ApiResponse<v2::UserProfileV2> {
        let start_time = env.ledger().timestamp();
        let request_id = ApiLayer::generate_request_id(&env);
        
        // Check permissions
        if caller != user {
            let caller_role = Self::get_user_role_internal(&env, &caller);
            match caller_role {
                crate::UserRole::SuperAdmin | crate::UserRole::SystemAdmin | crate::UserRole::ComplianceOfficer => {
                    caller.require_auth();
                },
                _ => {
                    let error = ApiLayer::create_error(&env, IntegrationError::InsufficientPermissions, "authorization", Some(request_id.clone()), None);
                    return ApiLayer::create_response(None, Some(error), ApiVersion::V2, request_id, 0);
                }
            }
        } else {
            caller.require_auth();
        }
        
        // Get full profile and convert to V2 format
        let full_profile = match Self::aggregate_user_profile(&env, &user) {
            Ok(profile) => profile,
            Err(e) => {
                let error = ApiLayer::create_error(&env, e, "data_aggregation", Some(request_id.clone()), None);
                return ApiLayer::create_response(None, Some(error), ApiVersion::V2, request_id, 0);
            }
        };
        
        // Convert to V2 format (enhanced but not full V3)
        let kyc_status_v2 = v2::KycStatusV2 {
            tier: full_profile.kyc_status.tier,
            status: full_profile.kyc_status.status,
            verified_at: full_profile.kyc_status.verified_at,
            expires_at: full_profile.kyc_status.expires_at,
        };
        
        let transaction_limits_v2 = v2::TransactionLimitsV2 {
            daily_deposit_limit: full_profile.transaction_limits.daily_deposit_limit,
            daily_deposit_used: full_profile.transaction_limits.daily_deposit_used,
            daily_withdrawal_limit: full_profile.transaction_limits.daily_withdrawal_limit,
            daily_withdrawal_used: full_profile.transaction_limits.daily_withdrawal_used,
            monthly_deposit_limit: full_profile.transaction_limits.monthly_deposit_limit,
            monthly_deposit_used: full_profile.transaction_limits.monthly_deposit_used,
            monthly_withdrawal_limit: full_profile.transaction_limits.monthly_withdrawal_limit,
            monthly_withdrawal_used: full_profile.transaction_limits.monthly_withdrawal_used,
        };
        
        let mut recent_activity_v2 = Vec::new(&env);
        for activity in full_profile.recent_activity.iter() {
            recent_activity_v2.push_back(v2::ActivityV2 {
                activity_type: activity.activity_type.clone(),
                amount: activity.amount,
                timestamp: activity.timestamp,
                status: activity.status.clone(),
            });
        }
        
        let v2_profile = v2::UserProfileV2 {
            address: full_profile.address,
            kyc_status: kyc_status_v2,
            token_balances: full_profile.token_balances,
            transaction_limits: transaction_limits_v2,
            recent_activity: recent_activity_v2,
        };
        
        let execution_time = env.ledger().timestamp() - start_time;
        ApiLayer::create_response(Some(v2_profile), None, ApiVersion::V2, request_id, execution_time)
    }
    
    /// Get system status (V2 API - enhanced response)
    pub fn get_system_status_v2(
        env: Env,
        caller: Address,
    ) -> ApiResponse<v2::SystemStatusV2> {
        let start_time = env.ledger().timestamp();
        let request_id = ApiLayer::generate_request_id(&env);
        
        // Check permissions
        let caller_role = Self::get_user_role_internal(&env, &caller);
        match caller_role {
            crate::UserRole::SuperAdmin | crate::UserRole::SystemAdmin | crate::UserRole::ComplianceOfficer => {
                caller.require_auth();
            },
            _ => {
                let error = ApiLayer::create_error(&env, IntegrationError::InsufficientPermissions, "authorization", Some(request_id.clone()), None);
                return ApiLayer::create_response(None, Some(error), ApiVersion::V2, request_id, 0);
            }
        }
        
        // Get full status and convert to V2 format
        let full_status = match Self::aggregate_system_status(&env) {
            Ok(status) => status,
            Err(e) => {
                let error = ApiLayer::create_error(&env, e, "system_monitoring", Some(request_id.clone()), None);
                return ApiLayer::create_response(None, Some(error), ApiVersion::V2, request_id, 0);
            }
        };
        
        // Convert component status to simple map
        let mut components = Map::new(&env);
        for (name, status) in full_status.component_status.iter() {
            components.set(name, status.status);
        }
        
        let metrics_v2 = v2::SystemMetricsV2 {
            total_value_locked: full_status.system_metrics.total_value_locked,
            reserve_ratio: full_status.system_metrics.reserve_ratio,
            active_users_24h: full_status.system_metrics.active_users_24h,
            transaction_volume_24h: full_status.system_metrics.transaction_volume_24h,
        };
        
        let v2_status = v2::SystemStatusV2 {
            overall_health: full_status.overall_health,
            components,
            active_operations: full_status.active_operations,
            pending_operations: full_status.pending_operations,
            failed_operations_24h: full_status.failed_operations_24h,
            metrics: metrics_v2,
        };
        
        let execution_time = env.ledger().timestamp() - start_time;
        ApiLayer::create_response(Some(v2_status), None, ApiVersion::V2, request_id, execution_time)
    }
    
    // =====================
    // Version Compatibility Helpers
    // =====================
    
    /// Check if a requested API version is supported
    pub fn is_api_version_supported(env: Env, version: String) -> bool {
        match ApiVersion::from_string(&env, &version.to_string()) {
            Ok(_) => true,
            Err(_) => false,
        }
    }
    
    /// Get list of supported API versions
    pub fn get_supported_api_versions(env: Env) -> Vec<String> {
        let mut versions = Vec::new(&env);
        versions.push_back(String::from_str(&env, "v1"));
        versions.push_back(String::from_str(&env, "v2"));
        versions.push_back(String::from_str(&env, "v3"));
        versions
    }
    
    /// Get API version compatibility matrix
    pub fn get_api_compatibility_info(env: Env) -> Map<String, Vec<String>> {
        let mut compatibility = Map::new(&env);
        
        let mut v1_compatible = Vec::new(&env);
        v1_compatible.push_back(String::from_str(&env, "v1"));
        compatibility.set(String::from_str(&env, "v1"), v1_compatible);
        
        let mut v2_compatible = Vec::new(&env);
        v2_compatible.push_back(String::from_str(&env, "v1"));
        v2_compatible.push_back(String::from_str(&env, "v2"));
        compatibility.set(String::from_str(&env, "v2"), v2_compatible);
        
        let mut v3_compatible = Vec::new(&env);
        v3_compatible.push_back(String::from_str(&env, "v1"));
        v3_compatible.push_back(String::from_str(&env, "v2"));
        v3_compatible.push_back(String::from_str(&env, "v3"));
        compatibility.set(String::from_str(&env, "v3"), v3_compatible);
        
        compatibility
    }
    
    /// Migrate response from one version to another
    pub fn migrate_api_response(
        env: Env,
        from_version: String,
        to_version: String,
        response_data: String, // Serialized response data
    ) -> Result<String, IntegrationError> {
        let from_ver = ApiVersion::from_string(&env, &from_version.to_string())?;
        let to_ver = ApiVersion::from_string(&env, &to_version.to_string())?;
        
        // Check compatibility
        if !to_ver.is_compatible_with(&from_ver) {
            return Err(IntegrationError::InvalidContractResponse);
        }
        
        // For now, return the same data (in a real implementation, this would
        // perform actual data transformation between versions)
        Ok(response_data)
    }
}