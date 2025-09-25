#![allow(unused)]
use soroban_sdk::{
    contracttype, contracterror, Address, Env, Map, Vec, String, BytesN
};

use crate::{IntegrationError, DataKey, IntegrationEvent, UserRole};

/// Monitoring and Alerting System for Integration Router
/// 
/// This module provides real-time monitoring of cross-contract operations,
/// suspicious activity detection, risk threshold monitoring, and comprehensive
/// incident reporting for compliance violations.

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MonitoringConfig {
    pub enabled: bool,
    pub alert_threshold_high: u64,      // High priority alert threshold
    pub alert_threshold_medium: u64,    // Medium priority alert threshold
    pub suspicious_activity_threshold: u64, // Threshold for flagging suspicious activity
    pub max_failed_operations_per_hour: u32, // Max failed ops before alert
    pub max_operations_per_minute: u32,  // Rate limiting threshold
    pub compliance_violation_auto_pause: bool, // Auto-pause on compliance violations
    pub incident_retention_days: u32,    // How long to keep incident records
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AlertSeverity {
    Low,      // Informational
    Medium,   // Warning - requires attention
    High,     // Critical - immediate action required
    Critical, // Emergency - system-wide impact
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AlertType {
    SuspiciousActivity,
    ComplianceViolation,
    SystemError,
    RiskThresholdExceeded,
    OperationFailure,
    UnauthorizedAccess,
    RateLimitExceeded,
    ReserveThreshold,
    ContractFailure,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Alert {
    pub alert_id: BytesN<32>,
    pub alert_type: AlertType,
    pub severity: AlertSeverity,
    pub user: Option<Address>,
    pub contract: Option<Address>,
    pub operation_id: Option<BytesN<32>>,
    pub correlation_id: BytesN<32>,
    pub message: String,
    pub metadata: Map<String, String>,
    pub created_at: u64,
    pub acknowledged: bool,
    pub acknowledged_by: Option<Address>,
    pub acknowledged_at: Option<u64>,
    pub resolved: bool,
    pub resolved_by: Option<Address>,
    pub resolved_at: Option<u64>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SuspiciousActivityPattern {
    pub pattern_id: String,
    pub description: String,
    pub detection_rules: Vec<DetectionRule>,
    pub severity: AlertSeverity,
    pub auto_flag: bool,
    pub enabled: bool,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DetectionRule {
    pub rule_type: RuleType,
    pub threshold: u64,
    pub time_window_seconds: u64,
    pub conditions: Map<String, String>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RuleType {
    FrequencyThreshold,    // Too many operations in time window
    AmountThreshold,       // Single operation amount too high
    VelocityThreshold,     // Rate of operations increasing too fast
    PatternAnomaly,        // Unusual pattern detected
    GeographicAnomaly,     // Unusual geographic pattern (if available)
    TimeAnomaly,           // Operations at unusual times
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UserRiskProfile {
    pub user: Address,
    pub risk_score: u64,           // 0-1000 risk score
    pub flagged: bool,
    pub flag_reason: String,
    pub flag_timestamp: Option<u64>,
    pub operations_count_24h: u32,
    pub total_volume_24h: u64,
    pub failed_operations_24h: u32,
    pub last_operation_timestamp: u64,
    pub kyc_tier: u32,
    pub compliance_violations: u32,
    pub last_risk_assessment: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SystemMetrics {
    pub total_operations: u64,
    pub successful_operations: u64,
    pub failed_operations: u64,
    pub operations_last_hour: u32,
    pub operations_last_minute: u32,
    pub average_processing_time_ms: u64,
    pub current_reserve_ratio: u64,
    pub active_users_24h: u32,
    pub flagged_users: u32,
    pub pending_alerts: u32,
    pub critical_alerts: u32,
    pub last_updated: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IncidentReport {
    pub incident_id: BytesN<32>,
    pub incident_type: String,
    pub severity: AlertSeverity,
    pub user: Option<Address>,
    pub operation_id: Option<BytesN<32>>,
    pub correlation_id: BytesN<32>,
    pub description: String,
    pub compliance_impact: bool,
    pub regulatory_notification_required: bool,
    pub affected_contracts: Vec<Address>,
    pub timeline: Vec<IncidentTimelineEntry>,
    pub resolution_actions: Vec<String>,
    pub created_at: u64,
    pub resolved_at: Option<u64>,
    pub created_by: Address,
    pub assigned_to: Option<Address>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IncidentTimelineEntry {
    pub timestamp: u64,
    pub event_type: String,
    pub description: String,
    pub user: Address,
    pub metadata: Map<String, String>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RiskThreshold {
    pub threshold_id: String,
    pub description: String,
    pub metric_type: String,      // e.g., "user_volume_24h", "system_error_rate"
    pub threshold_value: u64,
    pub comparison: ThresholdComparison,
    pub severity: AlertSeverity,
    pub auto_action: Option<AutoAction>,
    pub enabled: bool,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ThresholdComparison {
    GreaterThan,
    LessThan,
    Equals,
    GreaterThanOrEqual,
    LessThanOrEqual,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AutoAction {
    FlagUser,
    PauseUserOperations,
    EmergencyPause,
    NotifyCompliance,
    EscalateToAdmin,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OperationMonitoringData {
    pub operation_id: BytesN<32>,
    pub correlation_id: BytesN<32>,
    pub user: Address,
    pub operation_type: String,
    pub start_time: u64,
    pub end_time: Option<u64>,
    pub status: String,
    pub contracts_involved: Vec<Address>,
    pub gas_used: u64,
    pub error_message: Option<String>,
    pub compliance_checks: Vec<String>,
    pub risk_factors: Map<String, u64>,
}

// Additional data keys for monitoring system
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MonitoringDataKey {
    // Configuration
    MonitoringConfig,
    
    // Alerts
    Alert(BytesN<32>),              // Alert ID -> Alert
    AlertsByUser(Address),          // User -> Vec<BytesN<32>> (alert IDs)
    AlertsByType(AlertType),        // Alert type -> Vec<BytesN<32>> (alert IDs)
    AlertsBySeverity(AlertSeverity), // Severity -> Vec<BytesN<32>> (alert IDs)
    PendingAlerts,                  // Vec<BytesN<32>> (pending alert IDs)
    AlertNonce,                     // u64 - alert counter
    
    // Suspicious Activity Detection
    SuspiciousPattern(String),      // Pattern ID -> SuspiciousActivityPattern
    ActivePatterns,                 // Vec<String> (active pattern IDs)
    
    // User Risk Profiles
    UserRiskProfile(Address),       // User -> UserRiskProfile
    FlaggedUsers,                   // Vec<Address> (flagged user addresses)
    HighRiskUsers,                  // Vec<Address> (high risk user addresses)
    
    // System Metrics
    SystemMetrics,                  // Current system metrics
    MetricsHistory(u64),           // Timestamp -> SystemMetrics (hourly snapshots)
    
    // Incident Management
    IncidentReport(BytesN<32>),    // Incident ID -> IncidentReport
    IncidentsByUser(Address),      // User -> Vec<BytesN<32>> (incident IDs)
    ActiveIncidents,               // Vec<BytesN<32>> (active incident IDs)
    IncidentNonce,                 // u64 - incident counter
    
    // Risk Thresholds
    RiskThreshold(String),         // Threshold ID -> RiskThreshold
    ActiveThresholds,              // Vec<String> (active threshold IDs)
    
    // Operation Monitoring
    OperationMonitoring(BytesN<32>), // Operation ID -> OperationMonitoringData
    RecentOperations,              // Vec<BytesN<32>> (recent operation IDs)
    
    // Time-based tracking
    HourlyOperationCount(u64),     // Hour timestamp -> u32 (operation count)
    DailyUserOperations(Address, u64), // (User, Day timestamp) -> u32 (operation count)
    DailyUserVolume(Address, u64), // (User, Day timestamp) -> u64 (total volume)
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            alert_threshold_high: 1000,
            alert_threshold_medium: 500,
            suspicious_activity_threshold: 100,
            max_failed_operations_per_hour: 10,
            max_operations_per_minute: 60,
            compliance_violation_auto_pause: true,
            incident_retention_days: 90,
        }
    }
}

impl Default for SystemMetrics {
    fn default() -> Self {
        Self {
            total_operations: 0,
            successful_operations: 0,
            failed_operations: 0,
            operations_last_hour: 0,
            operations_last_minute: 0,
            average_processing_time_ms: 0,
            current_reserve_ratio: 10000, // 100% in basis points
            active_users_24h: 0,
            flagged_users: 0,
            pending_alerts: 0,
            critical_alerts: 0,
            last_updated: 0,
        }
    }
}