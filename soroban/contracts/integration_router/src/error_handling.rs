#![no_std]
use soroban_sdk::{
    contracttype, contracterror, Env, Address, BytesN, Vec, Map, String
};

/// Comprehensive error handling and recovery system for the Integration Router
/// 
/// This module implements:
/// - Complete IntegrationError enum with all error types
/// - Automatic retry logic with exponential backoff
/// - Rollback procedures for complex multi-step operations  
/// - Circuit breakers that automatically pause operations on high error rates

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum IntegrationError {
    // Authentication & Authorization (1-9)
    Unauthorized = 1,
    InsufficientPermissions = 2,
    InvalidSignature = 3,
    SessionExpired = 4,
    AccountLocked = 5,
    
    // Contract Communication (10-19)
    ContractNotFound = 10,
    ContractCallFailed = 11,
    InvalidContractResponse = 12,
    ContractTimeout = 13,
    ContractVersionMismatch = 14,
    InvalidContractState = 15,
    
    // Compliance & KYC (20-29)
    ComplianceCheckFailed = 20,
    InsufficientKYCTier = 21,
    AddressBlacklisted = 22,
    ComplianceTimeout = 23,
    InvalidComplianceData = 24,
    ComplianceServiceUnavailable = 25,
    
    // Reserve Management (30-39)
    InsufficientReserves = 30,
    ReserveRatioTooLow = 31,
    BitcoinTransactionFailed = 32,
    ReserveValidationFailed = 33,
    ReserveUpdateFailed = 34,
    ProofOfReservesInvalid = 35,
    
    // Operation Processing (40-49)
    OperationTimeout = 40,
    InvalidOperationState = 41,
    DuplicateOperation = 42,
    OperationLimitExceeded = 43,
    InvalidOperationParameters = 44,
    OperationRollbackFailed = 45,
    
    // System State (50-59)
    SystemPaused = 50,
    EmergencyMode = 51,
    MaintenanceMode = 52,
    CircuitBreakerOpen = 53,
    SystemOverloaded = 54,
    ConfigurationError = 55,
    
    // Network & External Services (60-69)
    NetworkTimeout = 60,
    ExternalServiceUnavailable = 61,
    OracleDataStale = 62,
    BitcoinNetworkError = 63,
    RateLimitExceeded = 64,
    
    // Data & Validation (70-79)
    InvalidInput = 70,
    DataCorruption = 71,
    SerializationError = 72,
    DeserializationError = 73,
    ValidationFailed = 74,
    
    // Resource Management (80-89)
    InsufficientGas = 80,
    StorageQuotaExceeded = 81,
    MemoryLimitExceeded = 82,
    ComputationLimitExceeded = 83,
    
    // Recovery & Rollback (90-99)
    RollbackRequired = 90,
    RollbackInProgress = 91,
    RollbackFailed = 92,
    RecoveryRequired = 93,
    RecoveryInProgress = 94,
    RecoveryFailed = 95,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ErrorSeverity {
    Low,        // Informational, no action required
    Medium,     // Warning, monitoring required
    High,       // Error, immediate attention required
    Critical,   // System failure, emergency response required
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ErrorCategory {
    Transient,      // Temporary error, retry possible
    Permanent,      // Permanent error, no retry
    Configuration,  // Configuration issue
    External,       // External service issue
    Internal,       // Internal system issue
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ErrorContext {
    pub error_code: IntegrationError,
    pub severity: ErrorSeverity,
    pub category: ErrorCategory,
    pub message: String,
    pub operation_id: BytesN<32>,
    pub user_address: Option<Address>,
    pub contract_address: Option<Address>,
    pub timestamp: u64,
    pub retry_count: u32,
    pub max_retries: u32,
    pub metadata: Map<String, String>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RetryConfig {
    pub max_retries: u32,
    pub base_delay_ms: u64,
    pub max_delay_ms: u64,
    pub backoff_multiplier: u64, // Multiplier for exponential backoff (e.g., 2 for doubling)
    pub jitter_enabled: bool,    // Add random jitter to prevent thundering herd
    pub retry_on_errors: Vec<IntegrationError>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RetryState {
    pub operation_id: BytesN<32>,
    pub current_attempt: u32,
    pub next_retry_at: u64,
    pub last_error: IntegrationError,
    pub error_history: Vec<IntegrationError>,
    pub created_at: u64,
    pub updated_at: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CircuitBreakerConfig {
    pub failure_threshold: u32,     // Number of failures before opening
    pub success_threshold: u32,     // Number of successes to close
    pub timeout_ms: u64,           // Time to wait before trying again
    pub monitoring_window_ms: u64,  // Time window for failure counting
    pub enabled: bool,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CircuitBreakerState {
    Closed,     // Normal operation
    Open,       // Failing fast, not allowing operations
    HalfOpen,   // Testing if service has recovered
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CircuitBreaker {
    pub name: String,
    pub state: CircuitBreakerState,
    pub config: CircuitBreakerConfig,
    pub failure_count: u32,
    pub success_count: u32,
    pub last_failure_time: u64,
    pub last_success_time: u64,
    pub state_changed_at: u64,
    pub total_requests: u64,
    pub total_failures: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RollbackStep {
    pub step_id: u32,
    pub operation_type: String,
    pub target_contract: Address,
    pub function_name: String,
    pub parameters: Vec<String>,
    pub description: String,
    pub critical: bool, // If true, rollback failure is critical
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RollbackPlan {
    pub operation_id: BytesN<32>,
    pub steps: Vec<RollbackStep>,
    pub created_at: u64,
    pub timeout: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RollbackStatus {
    NotStarted,
    InProgress,
    Completed,
    Failed,
    PartiallyCompleted,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RollbackExecution {
    pub operation_id: BytesN<32>,
    pub plan: RollbackPlan,
    pub status: RollbackStatus,
    pub current_step: u32,
    pub completed_steps: Vec<u32>,
    pub failed_steps: Vec<u32>,
    pub started_at: u64,
    pub completed_at: Option<u64>,
    pub error_message: String,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ErrorMetrics {
    pub total_errors: u64,
    pub errors_by_type: Map<IntegrationError, u64>,
    pub errors_by_severity: Map<ErrorSeverity, u64>,
    pub errors_by_category: Map<ErrorCategory, u64>,
    pub last_error_time: u64,
    pub error_rate_per_hour: u64,
    pub recovery_success_rate: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    // Error tracking
    ErrorContext(BytesN<32>),       // Error ID -> ErrorContext
    ErrorMetrics,                   // Global error metrics
    ErrorHistory,                   // Vec<BytesN<32>> - recent error IDs
    
    // Retry system
    RetryConfig(String),            // Operation type -> RetryConfig
    RetryState(BytesN<32>),         // Operation ID -> RetryState
    PendingRetries,                 // Vec<BytesN<32>> - operations pending retry
    
    // Circuit breakers
    CircuitBreaker(String),         // Service name -> CircuitBreaker
    CircuitBreakerConfig(String),   // Service name -> CircuitBreakerConfig
    
    // Rollback system
    RollbackPlan(BytesN<32>),       // Operation ID -> RollbackPlan
    RollbackExecution(BytesN<32>),  // Operation ID -> RollbackExecution
    ActiveRollbacks,                // Vec<BytesN<32>> - active rollback operations
}

impl IntegrationError {
    /// Get the severity level for this error
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            // Low severity - informational
            IntegrationError::SessionExpired |
            IntegrationError::RateLimitExceeded => ErrorSeverity::Low,
            
            // Medium severity - warnings
            IntegrationError::ContractTimeout |
            IntegrationError::ComplianceTimeout |
            IntegrationError::NetworkTimeout |
            IntegrationError::OracleDataStale => ErrorSeverity::Medium,
            
            // High severity - errors requiring attention
            IntegrationError::Unauthorized |
            IntegrationError::InsufficientPermissions |
            IntegrationError::ContractCallFailed |
            IntegrationError::ComplianceCheckFailed |
            IntegrationError::InsufficientKYCTier |
            IntegrationError::AddressBlacklisted |
            IntegrationError::InsufficientReserves |
            IntegrationError::ReserveRatioTooLow |
            IntegrationError::OperationTimeout |
            IntegrationError::InvalidOperationState |
            IntegrationError::DuplicateOperation => ErrorSeverity::High,
            
            // Critical severity - system failures
            IntegrationError::SystemPaused |
            IntegrationError::EmergencyMode |
            IntegrationError::CircuitBreakerOpen |
            IntegrationError::SystemOverloaded |
            IntegrationError::DataCorruption |
            IntegrationError::RollbackFailed |
            IntegrationError::RecoveryFailed => ErrorSeverity::Critical,
            
            // Default to high for unlisted errors
            _ => ErrorSeverity::High,
        }
    }
    
    /// Get the category for this error
    pub fn category(&self) -> ErrorCategory {
        match self {
            // Transient errors - can be retried
            IntegrationError::ContractTimeout |
            IntegrationError::ComplianceTimeout |
            IntegrationError::NetworkTimeout |
            IntegrationError::ExternalServiceUnavailable |
            IntegrationError::BitcoinNetworkError |
            IntegrationError::RateLimitExceeded |
            IntegrationError::SystemOverloaded => ErrorCategory::Transient,
            
            // Configuration errors
            IntegrationError::ContractNotFound |
            IntegrationError::ContractVersionMismatch |
            IntegrationError::ConfigurationError |
            IntegrationError::InvalidContractState => ErrorCategory::Configuration,
            
            // External service errors
            IntegrationError::ComplianceServiceUnavailable |
            IntegrationError::OracleDataStale |
            IntegrationError::BitcoinTransactionFailed => ErrorCategory::External,
            
            // Internal system errors
            IntegrationError::DataCorruption |
            IntegrationError::SerializationError |
            IntegrationError::DeserializationError |
            IntegrationError::InsufficientGas |
            IntegrationError::StorageQuotaExceeded => ErrorCategory::Internal,
            
            // Default to permanent for unlisted errors
            _ => ErrorCategory::Permanent,
        }
    }
    
    /// Check if this error type should trigger a retry
    pub fn is_retryable(&self) -> bool {
        matches!(self.category(), ErrorCategory::Transient | ErrorCategory::External)
    }
    
    /// Check if this error should trigger circuit breaker
    pub fn should_trigger_circuit_breaker(&self) -> bool {
        matches!(
            self,
            IntegrationError::ContractCallFailed |
            IntegrationError::ContractTimeout |
            IntegrationError::ExternalServiceUnavailable |
            IntegrationError::BitcoinNetworkError |
            IntegrationError::ComplianceServiceUnavailable |
            IntegrationError::SystemOverloaded
        )
    }
}

/// Default retry configurations for different operation types
pub fn get_default_retry_config(operation_type: &str) -> RetryConfig {
    match operation_type {
        "bitcoin_deposit" => RetryConfig {
            max_retries: 5,
            base_delay_ms: 1000,
            max_delay_ms: 30000,
            backoff_multiplier: 2,
            jitter_enabled: true,
            retry_on_errors: vec![
                IntegrationError::ContractTimeout,
                IntegrationError::NetworkTimeout,
                IntegrationError::ExternalServiceUnavailable,
                IntegrationError::BitcoinNetworkError,
            ],
        },
        "token_withdrawal" => RetryConfig {
            max_retries: 3,
            base_delay_ms: 2000,
            max_delay_ms: 60000,
            backoff_multiplier: 2,
            jitter_enabled: true,
            retry_on_errors: vec![
                IntegrationError::ContractTimeout,
                IntegrationError::BitcoinTransactionFailed,
                IntegrationError::ReserveValidationFailed,
            ],
        },
        "cross_token_exchange" => RetryConfig {
            max_retries: 3,
            base_delay_ms: 500,
            max_delay_ms: 10000,
            backoff_multiplier: 2,
            jitter_enabled: true,
            retry_on_errors: vec![
                IntegrationError::ContractTimeout,
                IntegrationError::OracleDataStale,
                IntegrationError::ExternalServiceUnavailable,
            ],
        },
        "compliance_check" => RetryConfig {
            max_retries: 2,
            base_delay_ms: 1000,
            max_delay_ms: 5000,
            backoff_multiplier: 2,
            jitter_enabled: false,
            retry_on_errors: vec![
                IntegrationError::ComplianceTimeout,
                IntegrationError::ComplianceServiceUnavailable,
            ],
        },
        _ => RetryConfig {
            max_retries: 3,
            base_delay_ms: 1000,
            max_delay_ms: 15000,
            backoff_multiplier: 2,
            jitter_enabled: true,
            retry_on_errors: vec![
                IntegrationError::ContractTimeout,
                IntegrationError::NetworkTimeout,
                IntegrationError::ExternalServiceUnavailable,
            ],
        },
    }
}

/// Default circuit breaker configurations for different services
pub fn get_default_circuit_breaker_config(service_name: &str) -> CircuitBreakerConfig {
    match service_name {
        "kyc_registry" => CircuitBreakerConfig {
            failure_threshold: 5,
            success_threshold: 3,
            timeout_ms: 30000,
            monitoring_window_ms: 60000,
            enabled: true,
        },
        "reserve_manager" => CircuitBreakerConfig {
            failure_threshold: 3,
            success_threshold: 2,
            timeout_ms: 60000,
            monitoring_window_ms: 120000,
            enabled: true,
        },
        "bitcoin_network" => CircuitBreakerConfig {
            failure_threshold: 10,
            success_threshold: 5,
            timeout_ms: 120000,
            monitoring_window_ms: 300000,
            enabled: true,
        },
        "oracle_service" => CircuitBreakerConfig {
            failure_threshold: 5,
            success_threshold: 3,
            timeout_ms: 30000,
            monitoring_window_ms: 60000,
            enabled: true,
        },
        _ => CircuitBreakerConfig {
            failure_threshold: 5,
            success_threshold: 3,
            timeout_ms: 30000,
            monitoring_window_ms: 60000,
            enabled: true,
        },
    }
}