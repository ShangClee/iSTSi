use loco_rs::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityEvent {
    pub id: String,
    pub event_type: SecurityEventType,
    pub severity: SecuritySeverity,
    pub timestamp: u64,
    pub source_ip: Option<String>,
    pub user_id: Option<String>,
    pub details: HashMap<String, String>,
    pub resolved: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityEventType {
    AuthenticationFailure,
    AuthorizationFailure,
    SuspiciousActivity,
    RateLimitExceeded,
    InvalidTokenUsage,
    UnauthorizedAccess,
    DataBreach,
    SystemVulnerability,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum SecuritySeverity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone)]
pub struct SecurityService {
    events: Vec<SecurityEvent>,
    rate_limits: HashMap<String, RateLimitTracker>,
    blocked_ips: HashMap<String, BlockedIpInfo>,
}

#[derive(Debug, Clone)]
struct RateLimitTracker {
    requests: Vec<u64>,
    window_size: Duration,
    max_requests: usize,
}

#[derive(Debug, Clone)]
struct BlockedIpInfo {
    blocked_at: u64,
    reason: String,
    expires_at: Option<u64>,
}

impl SecurityService {
    pub fn new() -> Self {
        Self {
            events: Vec::new(),
            rate_limits: HashMap::new(),
            blocked_ips: HashMap::new(),
        }
    }

    /// Log a security event
    pub fn log_security_event(
        &mut self,
        event_type: SecurityEventType,
        severity: SecuritySeverity,
        source_ip: Option<String>,
        user_id: Option<String>,
        details: HashMap<String, String>,
    ) -> String {
        let event_id = Uuid::new_v4().to_string();
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let event = SecurityEvent {
            id: event_id.clone(),
            event_type: event_type.clone(),
            severity: severity.clone(),
            timestamp,
            source_ip: source_ip.clone(),
            user_id,
            details,
            resolved: false,
        };

        self.events.push(event);

        // Auto-respond to critical events
        if severity == SecuritySeverity::Critical {
            self.handle_critical_event(&event_type, source_ip.as_deref());
        }

        // Log to system logger
        match severity {
            SecuritySeverity::Critical => tracing::error!("SECURITY CRITICAL: {:?} - {}", event_type, event_id),
            SecuritySeverity::High => tracing::warn!("SECURITY HIGH: {:?} - {}", event_type, event_id),
            SecuritySeverity::Medium => tracing::warn!("SECURITY MEDIUM: {:?} - {}", event_type, event_id),
            SecuritySeverity::Low => tracing::info!("SECURITY LOW: {:?} - {}", event_type, event_id),
        }

        event_id
    }

    /// Handle critical security events with automatic responses
    fn handle_critical_event(&mut self, event_type: &SecurityEventType, source_ip: Option<&str>) {
        match event_type {
            SecurityEventType::DataBreach | SecurityEventType::UnauthorizedAccess => {
                if let Some(ip) = source_ip {
                    self.block_ip(ip, "Critical security event", Some(3600)); // Block for 1 hour
                }
                // TODO: Send alerts to security team
                // TODO: Trigger incident response procedures
            },
            SecurityEventType::SystemVulnerability => {
                // TODO: Trigger automated security patching if available
                // TODO: Notify system administrators
            },
            _ => {}
        }
    }

    /// Check rate limits for IP address
    pub fn check_rate_limit(&mut self, ip: &str, endpoint: &str, max_requests: usize, window_minutes: u64) -> bool {
        let key = format!("{}:{}", ip, endpoint);
        let window_size = Duration::from_secs(window_minutes * 60);
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let tracker = self.rate_limits.entry(key.clone()).or_insert_with(|| RateLimitTracker {
            requests: Vec::new(),
            window_size,
            max_requests,
        });

        // Remove old requests outside the window
        tracker.requests.retain(|&timestamp| {
            current_time - timestamp < window_size.as_secs()
        });

        // Check if limit exceeded
        let requests_count = tracker.requests.len();
        if requests_count >= max_requests {
            // Release the mutable borrow before calling log_security_event
            drop(tracker);
            
            self.log_security_event(
                SecurityEventType::RateLimitExceeded,
                SecuritySeverity::Medium,
                Some(ip.to_string()),
                None,
                HashMap::from([
                    ("endpoint".to_string(), endpoint.to_string()),
                    ("requests_count".to_string(), requests_count.to_string()),
                ]),
            );
            return false;
        }

        // Add current request
        tracker.requests.push(current_time);
        true
    }

    /// Block IP address
    pub fn block_ip(&mut self, ip: &str, reason: &str, duration_seconds: Option<u64>) {
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let expires_at = duration_seconds.map(|duration| current_time + duration);

        self.blocked_ips.insert(ip.to_string(), BlockedIpInfo {
            blocked_at: current_time,
            reason: reason.to_string(),
            expires_at,
        });

        tracing::warn!("IP {} blocked: {}", ip, reason);
    }

    /// Check if IP is blocked
    pub fn is_ip_blocked(&mut self, ip: &str) -> bool {
        if let Some(block_info) = self.blocked_ips.get(ip) {
            // Check if block has expired
            if let Some(expires_at) = block_info.expires_at {
                let current_time = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs();

                if current_time > expires_at {
                    self.blocked_ips.remove(ip);
                    return false;
                }
            }
            return true;
        }
        false
    }

    /// Validate request for suspicious patterns
    pub fn validate_request_security(&mut self, 
        ip: &str, 
        user_agent: Option<&str>, 
        path: &str,
        _method: &str,
    ) -> Result<()> {
        // Check if IP is blocked
        if self.is_ip_blocked(ip) {
            return Err(Error::string("IP address is blocked"));
        }

        // Check for suspicious patterns
        if self.is_suspicious_user_agent(user_agent) {
            self.log_security_event(
                SecurityEventType::SuspiciousActivity,
                SecuritySeverity::Medium,
                Some(ip.to_string()),
                None,
                HashMap::from([
                    ("user_agent".to_string(), user_agent.unwrap_or("").to_string()),
                    ("path".to_string(), path.to_string()),
                ]),
            );
        }

        // Check for path traversal attempts
        if path.contains("..") || path.contains("%2e%2e") {
            self.log_security_event(
                SecurityEventType::UnauthorizedAccess,
                SecuritySeverity::High,
                Some(ip.to_string()),
                None,
                HashMap::from([
                    ("attack_type".to_string(), "path_traversal".to_string()),
                    ("path".to_string(), path.to_string()),
                ]),
            );
            return Err(Error::string("Path traversal attempt detected"));
        }

        // Check for SQL injection patterns
        if self.contains_sql_injection_patterns(path) {
            self.log_security_event(
                SecurityEventType::UnauthorizedAccess,
                SecuritySeverity::High,
                Some(ip.to_string()),
                None,
                HashMap::from([
                    ("attack_type".to_string(), "sql_injection".to_string()),
                    ("path".to_string(), path.to_string()),
                ]),
            );
            return Err(Error::string("SQL injection attempt detected"));
        }

        Ok(())
    }

    /// Check for suspicious user agent patterns
    fn is_suspicious_user_agent(&self, user_agent: Option<&str>) -> bool {
        if let Some(ua) = user_agent {
            let suspicious_patterns = vec![
                "sqlmap", "nikto", "nmap", "masscan", "zap", "burp",
                "python-requests", "curl", "wget", "scanner",
            ];

            let ua_lower = ua.to_lowercase();
            return suspicious_patterns.iter().any(|pattern| ua_lower.contains(pattern));
        }
        false
    }

    /// Check for SQL injection patterns
    fn contains_sql_injection_patterns(&self, input: &str) -> bool {
        let sql_patterns = vec![
            "union select", "drop table", "insert into", "delete from",
            "update set", "exec(", "execute(", "sp_", "xp_",
            "' or '1'='1", "' or 1=1", "admin'--", "' union",
        ];

        let input_lower = input.to_lowercase();
        sql_patterns.iter().any(|pattern| input_lower.contains(pattern))
    }

    /// Get security events by severity
    pub fn get_events_by_severity(&self, severity: SecuritySeverity) -> Vec<&SecurityEvent> {
        self.events.iter()
            .filter(|event| event.severity == severity)
            .collect()
    }

    /// Get unresolved security events
    pub fn get_unresolved_events(&self) -> Vec<&SecurityEvent> {
        self.events.iter()
            .filter(|event| !event.resolved)
            .collect()
    }

    /// Mark security event as resolved
    pub fn resolve_event(&mut self, event_id: &str) -> Result<()> {
        if let Some(event) = self.events.iter_mut().find(|e| e.id == event_id) {
            event.resolved = true;
            tracing::info!("Security event {} marked as resolved", event_id);
            Ok(())
        } else {
            Err(Error::string(&format!("Security event {} not found", event_id)))
        }
    }

    /// Generate security report
    pub fn generate_security_report(&self) -> SecurityReport {
        let total_events = self.events.len();
        let critical_events = self.get_events_by_severity(SecuritySeverity::Critical).len();
        let high_events = self.get_events_by_severity(SecuritySeverity::High).len();
        let unresolved_events = self.get_unresolved_events().len();
        let blocked_ips_count = self.blocked_ips.len();

        SecurityReport {
            total_events,
            critical_events,
            high_events,
            unresolved_events,
            blocked_ips_count,
            report_generated_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct SecurityReport {
    pub total_events: usize,
    pub critical_events: usize,
    pub high_events: usize,
    pub unresolved_events: usize,
    pub blocked_ips_count: usize,
    pub report_generated_at: u64,
}

#[derive(Debug, Clone)]
pub enum SecurityViolation {
    BlockedIp,
    PathTraversal,
    SqlInjection,
    RateLimitExceeded,
}

impl std::fmt::Display for SecurityViolation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SecurityViolation::BlockedIp => write!(f, "IP address is blocked"),
            SecurityViolation::PathTraversal => write!(f, "Path traversal attempt detected"),
            SecurityViolation::SqlInjection => write!(f, "SQL injection attempt detected"),
            SecurityViolation::RateLimitExceeded => write!(f, "Rate limit exceeded"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limiting() {
        let mut service = SecurityService::new();
        
        // Should allow first few requests
        assert!(service.check_rate_limit("127.0.0.1", "/api/test", 5, 1));
        assert!(service.check_rate_limit("127.0.0.1", "/api/test", 5, 1));
        
        // Should block after limit
        for _ in 0..5 {
            service.check_rate_limit("127.0.0.1", "/api/test", 5, 1);
        }
        assert!(!service.check_rate_limit("127.0.0.1", "/api/test", 5, 1));
    }

    #[test]
    fn test_ip_blocking() {
        let mut service = SecurityService::new();
        
        service.block_ip("192.168.1.1", "Test block", Some(3600));
        assert!(service.is_ip_blocked("192.168.1.1"));
        assert!(!service.is_ip_blocked("192.168.1.2"));
    }

    #[test]
    fn test_sql_injection_detection() {
        let service = SecurityService::new();
        
        assert!(service.contains_sql_injection_patterns("' union select * from users"));
        assert!(service.contains_sql_injection_patterns("admin'--"));
        assert!(!service.contains_sql_injection_patterns("normal query"));
    }
}