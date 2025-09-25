//! Unified Query Interfaces
//! 
//! This module provides unified query interfaces for external applications and user interfaces,
//! aggregating data from all integrated contracts into standardized responses.

use soroban_sdk::{
    contractimpl, panic_with_error, Address, Env, Map, Vec, String, BytesN
};

use crate::{
    IntegrationRouter, IntegrationError, DataKey, UserRole,
    api_layer::{
        ApiResponse, ApiError, ApiLayer, ApiVersion, UserProfile, KycStatus, 
        TransactionLimits, ActivitySummary, SystemStatus, ComponentStatus, 
        SystemMetrics, SystemAlert, TransactionDetails, ComplianceCheck,
        QueryFilter, PaginatedResponse, AggregatedStats, TransactionTypeStats
    }
};

#[contractimpl]
impl IntegrationRouter {
    
    // =====================
    // User Profile Queries
    // =====================
    
    /// Get comprehensive user profile aggregating data from all contracts
    pub fn get_user_profile(
        env: Env,
        caller: Address,
        user: Address,
        api_version: String,
    ) -> ApiResponse<UserProfile> {
        let start_time = env.ledger().timestamp();
        let request_id = ApiLayer::generate_request_id(&env);
        
        // Parse API version
        let version = match ApiVersion::from_string(&env, &api_version.to_string()) {
            Ok(v) => v,
            Err(e) => {
                let error = ApiLayer::create_error(&env, e, "api_layer", Some(request_id.clone()), None);
                return ApiLayer::create_response(None, Some(error), ApiVersion::V1, request_id, 0);
            }
        };
        
        // Check permissions - users can view their own profile, admins can view any
        let caller_role = Self::get_user_role_internal(&env, &caller);
        if caller != user {
            match caller_role {
                UserRole::SuperAdmin | UserRole::SystemAdmin | UserRole::ComplianceOfficer => {
                    caller.require_auth();
                },
                _ => {
                    let error = ApiLayer::create_error(&env, IntegrationError::InsufficientPermissions, "authorization", Some(request_id.clone()), None);
                    return ApiLayer::create_response(None, Some(error), version, request_id, 0);
                }
            }
        } else {
            caller.require_auth();
        }
        
        // Aggregate user data from all contracts
        let profile = match Self::aggregate_user_profile(&env, &user) {
            Ok(profile) => profile,
            Err(e) => {
                let error = ApiLayer::create_error(&env, e, "data_aggregation", Some(request_id.clone()), None);
                return ApiLayer::create_response(None, Some(error), version, request_id, 0);
            }
        };
        
        let execution_time = env.ledger().timestamp() - start_time;
        ApiLayer::create_response(Some(profile), None, version, request_id, execution_time)
    }
    
    /// Get user transaction history with filtering and pagination
    pub fn get_user_transactions(
        env: Env,
        caller: Address,
        user: Address,
        filter: QueryFilter,
        api_version: String,
    ) -> ApiResponse<PaginatedResponse<TransactionDetails>> {
        let start_time = env.ledger().timestamp();
        let request_id = ApiLayer::generate_request_id(&env);
        
        // Parse API version
        let version = match ApiVersion::from_string(&env, &api_version.to_string()) {
            Ok(v) => v,
            Err(e) => {
                let error = ApiLayer::create_error(&env, e, "api_layer", Some(request_id.clone()), None);
                return ApiLayer::create_response(None, Some(error), ApiVersion::V1, request_id, 0);
            }
        };
        
        // Check permissions
        let caller_role = Self::get_user_role_internal(&env, &caller);
        if caller != user {
            match caller_role {
                UserRole::SuperAdmin | UserRole::SystemAdmin | UserRole::ComplianceOfficer => {
                    caller.require_auth();
                },
                _ => {
                    let error = ApiLayer::create_error(&env, IntegrationError::InsufficientPermissions, "authorization", Some(request_id.clone()), None);
                    return ApiLayer::create_response(None, Some(error), version, request_id, 0);
                }
            }
        } else {
            caller.require_auth();
        }
        
        // Get paginated transaction history
        let transactions = match Self::get_filtered_transactions(&env, &user, &filter) {
            Ok(transactions) => transactions,
            Err(e) => {
                let error = ApiLayer::create_error(&env, e, "transaction_query", Some(request_id.clone()), None);
                return ApiLayer::create_response(None, Some(error), version, request_id, 0);
            }
        };
        
        let execution_time = env.ledger().timestamp() - start_time;
        ApiLayer::create_response(Some(transactions), None, version, request_id, execution_time)
    }
    
    // =====================
    // System Status Queries
    // =====================
    
    /// Get comprehensive system status from all integrated contracts
    pub fn get_system_status(
        env: Env,
        caller: Address,
        api_version: String,
    ) -> ApiResponse<SystemStatus> {
        let start_time = env.ledger().timestamp();
        let request_id = ApiLayer::generate_request_id(&env);
        
        // Parse API version
        let version = match ApiVersion::from_string(&env, &api_version.to_string()) {
            Ok(v) => v,
            Err(e) => {
                let error = ApiLayer::create_error(&env, e, "api_layer", Some(request_id.clone()), None);
                return ApiLayer::create_response(None, Some(error), ApiVersion::V1, request_id, 0);
            }
        };
        
        // Check permissions - only admins can view system status
        let caller_role = Self::get_user_role_internal(&env, &caller);
        match caller_role {
            UserRole::SuperAdmin | UserRole::SystemAdmin | UserRole::ComplianceOfficer => {
                caller.require_auth();
            },
            _ => {
                let error = ApiLayer::create_error(&env, IntegrationError::InsufficientPermissions, "authorization", Some(request_id.clone()), None);
                return ApiLayer::create_response(None, Some(error), version, request_id, 0);
            }
        }
        
        // Aggregate system status from all contracts
        let status = match Self::aggregate_system_status(&env) {
            Ok(status) => status,
            Err(e) => {
                let error = ApiLayer::create_error(&env, e, "system_monitoring", Some(request_id.clone()), None);
                return ApiLayer::create_response(None, Some(error), version, request_id, 0);
            }
        };
        
        let execution_time = env.ledger().timestamp() - start_time;
        ApiLayer::create_response(Some(status), None, version, request_id, execution_time)
    }
    
    /// Get aggregated statistics for a given time period
    pub fn get_aggregated_stats(
        env: Env,
        caller: Address,
        period: String,
        api_version: String,
    ) -> ApiResponse<AggregatedStats> {
        let start_time = env.ledger().timestamp();
        let request_id = ApiLayer::generate_request_id(&env);
        
        // Parse API version
        let version = match ApiVersion::from_string(&env, &api_version.to_string()) {
            Ok(v) => v,
            Err(e) => {
                let error = ApiLayer::create_error(&env, e, "api_layer", Some(request_id.clone()), None);
                return ApiLayer::create_response(None, Some(error), ApiVersion::V1, request_id, 0);
            }
        };
        
        // Check permissions - only admins can view aggregated stats
        let caller_role = Self::get_user_role_internal(&env, &caller);
        match caller_role {
            UserRole::SuperAdmin | UserRole::SystemAdmin | UserRole::ComplianceOfficer => {
                caller.require_auth();
            },
            _ => {
                let error = ApiLayer::create_error(&env, IntegrationError::InsufficientPermissions, "authorization", Some(request_id.clone()), None);
                return ApiLayer::create_response(None, Some(error), version, request_id, 0);
            }
        }
        
        // Calculate aggregated statistics
        let stats = match Self::calculate_aggregated_stats(&env, &period) {
            Ok(stats) => stats,
            Err(e) => {
                let error = ApiLayer::create_error(&env, e, "statistics", Some(request_id.clone()), None);
                return ApiLayer::create_response(None, Some(error), version, request_id, 0);
            }
        };
        
        let execution_time = env.ledger().timestamp() - start_time;
        ApiLayer::create_response(Some(stats), None, version, request_id, execution_time)
    }
    
    // =====================
    // Transaction Queries
    // =====================
    
    /// Get detailed transaction information by ID
    pub fn get_transaction_details(
        env: Env,
        caller: Address,
        transaction_id: BytesN<32>,
        api_version: String,
    ) -> ApiResponse<TransactionDetails> {
        let start_time = env.ledger().timestamp();
        let request_id = ApiLayer::generate_request_id(&env);
        
        // Parse API version
        let version = match ApiVersion::from_string(&env, &api_version.to_string()) {
            Ok(v) => v,
            Err(e) => {
                let error = ApiLayer::create_error(&env, e, "api_layer", Some(request_id.clone()), None);
                return ApiLayer::create_response(None, Some(error), ApiVersion::V1, request_id, 0);
            }
        };
        
        caller.require_auth();
        
        // Get transaction details from all relevant contracts
        let details = match Self::get_transaction_details_internal(&env, &caller, &transaction_id) {
            Ok(details) => details,
            Err(e) => {
                let error = ApiLayer::create_error(&env, e, "transaction_lookup", Some(request_id.clone()), None);
                return ApiLayer::create_response(None, Some(error), version, request_id, 0);
            }
        };
        
        let execution_time = env.ledger().timestamp() - start_time;
        ApiLayer::create_response(Some(details), None, version, request_id, execution_time)
    }
    
    // =====================
    // Internal Helper Functions
    // =====================
    
    /// Aggregate user profile data from all contracts
    fn aggregate_user_profile(env: &Env, user: &Address) -> Result<UserProfile, IntegrationError> {
        // Get KYC status (mock implementation - would call actual KYC contract)
        let kyc_status = KycStatus {
            tier: 2,
            status: String::from_str(env, "verified"),
            verified_at: Some(env.ledger().timestamp() - 86400), // 1 day ago
            expires_at: Some(env.ledger().timestamp() + 31536000), // 1 year from now
            required_actions: Vec::new(env),
        };
        
        // Get token balances (mock implementation - would call token contracts)
        let mut token_balances = Map::new(env);
        token_balances.set(String::from_str(env, "iSTSi"), 1000000u64);
        token_balances.set(String::from_str(env, "USDC"), 5000000u64);
        
        // Get transaction limits based on KYC tier
        let transaction_limits = TransactionLimits {
            daily_deposit_limit: 100000000u64, // $1000 equivalent
            daily_deposit_used: 25000000u64,   // $250 used
            daily_withdrawal_limit: 50000000u64, // $500 equivalent
            daily_withdrawal_used: 10000000u64,  // $100 used
            monthly_deposit_limit: 1000000000u64, // $10,000 equivalent
            monthly_deposit_used: 250000000u64,   // $2,500 used
            monthly_withdrawal_limit: 500000000u64, // $5,000 equivalent
            monthly_withdrawal_used: 100000000u64,  // $1,000 used
            single_transaction_limit: 50000000u64,  // $500 equivalent
        };
        
        // Get recent activity (mock implementation - would aggregate from event history)
        let mut recent_activity = Vec::new(env);
        recent_activity.push_back(ActivitySummary {
            activity_type: String::from_str(env, "bitcoin_deposit"),
            amount: 10000000u64,
            timestamp: env.ledger().timestamp() - 3600, // 1 hour ago
            status: String::from_str(env, "completed"),
            transaction_hash: Some(BytesN::from_array(env, &[1u8; 32])),
        });
        
        // Get compliance flags
        let compliance_flags = Vec::new(env);
        
        Ok(UserProfile {
            address: user.clone(),
            kyc_status,
            token_balances,
            transaction_limits,
            recent_activity,
            compliance_flags,
        })
    }
    
    /// Get filtered transactions for a user
    fn get_filtered_transactions(
        env: &Env,
        user: &Address,
        filter: &QueryFilter,
    ) -> Result<PaginatedResponse<TransactionDetails>, IntegrationError> {
        // Mock implementation - would query actual transaction history
        let mut transactions = Vec::new(env);
        
        // Add sample transactions
        transactions.push_back(TransactionDetails {
            transaction_id: BytesN::from_array(env, &[1u8; 32]),
            transaction_type: String::from_str(env, "bitcoin_deposit"),
            user: user.clone(),
            amount: 10000000u64,
            status: String::from_str(env, "completed"),
            created_at: env.ledger().timestamp() - 3600,
            updated_at: env.ledger().timestamp() - 3500,
            completion_time: Some(env.ledger().timestamp() - 3500),
            fees: 50000u64,
            exchange_rate: Some(10000u64), // 1:1 rate in basis points
            compliance_checks: Vec::new(env),
            blockchain_confirmations: Some(6),
            error_details: None,
        });
        
        let page_size = filter.limit.unwrap_or(10);
        let current_page = (filter.offset.unwrap_or(0) / page_size) + 1;
        
        Ok(ApiLayer::create_paginated_response(
            env,
            transactions,
            1u64, // total count
            page_size,
            current_page,
        ))
    }
    
    /// Aggregate system status from all contracts
    fn aggregate_system_status(env: &Env) -> Result<SystemStatus, IntegrationError> {
        let mut component_status = Map::new(env);
        
        // Check integration router status
        component_status.set(
            String::from_str(env, "integration_router"),
            ComponentStatus {
                name: String::from_str(env, "Integration Router"),
                status: String::from_str(env, "healthy"),
                last_check: env.ledger().timestamp(),
                response_time_ms: 50,
                error_rate_24h: 0,
                uptime_percentage: 9999, // 99.99%
            }
        );
        
        // Check KYC registry status (mock)
        component_status.set(
            String::from_str(env, "kyc_registry"),
            ComponentStatus {
                name: String::from_str(env, "KYC Registry"),
                status: String::from_str(env, "healthy"),
                last_check: env.ledger().timestamp(),
                response_time_ms: 75,
                error_rate_24h: 1,
                uptime_percentage: 9995, // 99.95%
            }
        );
        
        // Check iSTSi token status (mock)
        component_status.set(
            String::from_str(env, "istsi_token"),
            ComponentStatus {
                name: String::from_str(env, "iSTSi Token"),
                status: String::from_str(env, "healthy"),
                last_check: env.ledger().timestamp(),
                response_time_ms: 30,
                error_rate_24h: 0,
                uptime_percentage: 10000, // 100%
            }
        );
        
        // Check reserve manager status (mock)
        component_status.set(
            String::from_str(env, "reserve_manager"),
            ComponentStatus {
                name: String::from_str(env, "Reserve Manager"),
                status: String::from_str(env, "healthy"),
                last_check: env.ledger().timestamp(),
                response_time_ms: 100,
                error_rate_24h: 2,
                uptime_percentage: 9990, // 99.90%
            }
        );
        
        let system_metrics = SystemMetrics {
            total_value_locked: 1000000000000u64, // $10M equivalent
            reserve_ratio: 10500u64, // 105% in basis points
            active_users_24h: 150,
            transaction_volume_24h: 50000000000u64, // $500K equivalent
            average_processing_time: 2500, // 2.5 seconds
        };
        
        let alerts = Vec::new(env); // No active alerts
        
        Ok(SystemStatus {
            overall_health: String::from_str(env, "healthy"),
            component_status,
            active_operations: 5,
            pending_operations: 2,
            failed_operations_24h: 1,
            system_metrics,
            alerts,
        })
    }
    
    /// Calculate aggregated statistics for a time period
    fn calculate_aggregated_stats(
        env: &Env,
        period: &String,
    ) -> Result<AggregatedStats, IntegrationError> {
        // Mock implementation - would calculate from actual transaction data
        let mut by_transaction_type = Map::new(env);
        
        by_transaction_type.set(
            String::from_str(env, "bitcoin_deposit"),
            TransactionTypeStats {
                count: 25,
                volume: 250000000u64,
                success_rate: 9600, // 96%
                average_processing_time: 3000, // 3 seconds
            }
        );
        
        by_transaction_type.set(
            String::from_str(env, "token_withdrawal"),
            TransactionTypeStats {
                count: 15,
                volume: 150000000u64,
                success_rate: 9333, // 93.33%
                average_processing_time: 4500, // 4.5 seconds
            }
        );
        
        by_transaction_type.set(
            String::from_str(env, "cross_token_exchange"),
            TransactionTypeStats {
                count: 10,
                volume: 100000000u64,
                success_rate: 10000, // 100%
                average_processing_time: 1500, // 1.5 seconds
            }
        );
        
        Ok(AggregatedStats {
            period: period.clone(),
            total_transactions: 50,
            total_volume: 500000000u64,
            successful_transactions: 48,
            failed_transactions: 2,
            average_transaction_size: 10000000u64,
            unique_users: 35,
            by_transaction_type,
        })
    }
    
    /// Get detailed transaction information
    fn get_transaction_details_internal(
        env: &Env,
        caller: &Address,
        transaction_id: &BytesN<32>,
    ) -> Result<TransactionDetails, IntegrationError> {
        // Mock implementation - would look up actual transaction
        let mut compliance_checks = Vec::new(env);
        compliance_checks.push_back(ComplianceCheck {
            check_type: String::from_str(env, "kyc_verification"),
            status: String::from_str(env, "passed"),
            checked_at: env.ledger().timestamp() - 3600,
            details: String::from_str(env, "KYC tier 2 verified"),
        });
        
        compliance_checks.push_back(ComplianceCheck {
            check_type: String::from_str(env, "aml_screening"),
            status: String::from_str(env, "passed"),
            checked_at: env.ledger().timestamp() - 3590,
            details: String::from_str(env, "No AML flags detected"),
        });
        
        Ok(TransactionDetails {
            transaction_id: transaction_id.clone(),
            transaction_type: String::from_str(env, "bitcoin_deposit"),
            user: caller.clone(),
            amount: 10000000u64,
            status: String::from_str(env, "completed"),
            created_at: env.ledger().timestamp() - 3600,
            updated_at: env.ledger().timestamp() - 3500,
            completion_time: Some(env.ledger().timestamp() - 3500),
            fees: 50000u64,
            exchange_rate: Some(10000u64),
            compliance_checks,
            blockchain_confirmations: Some(6),
            error_details: None,
        })
    }
}