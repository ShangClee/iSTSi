#!/bin/bash

# Production Deployment Monitor for Bitcoin Custody Full-Stack Application
# Monitors production deployments and provides real-time health checks

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
MONITOR_CONFIG="$PROJECT_ROOT/monitor-config.json"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
NC='\033[0m' # No Color

log() {
    echo -e "${BLUE}[PRODUCTION-MONITOR]${NC} $1"
}

error() {
    echo -e "${RED}[ERROR]${NC} $1" >&2
}

success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

warn() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

info() {
    echo -e "${PURPLE}[INFO]${NC} $1"
}

# Initialize monitoring configuration
init_monitor_config() {
    if [ ! -f "$MONITOR_CONFIG" ]; then
        log "Creating monitoring configuration..."
        cat > "$MONITOR_CONFIG" << 'EOF'
{
  "endpoints": {
    "production": {
      "frontend": "https://bitcoin-custody.com",
      "backend": "https://api.bitcoin-custody.com",
      "health_check": "https://api.bitcoin-custody.com/health",
      "status_check": "https://api.bitcoin-custody.com/api/system/status"
    },
    "staging": {
      "frontend": "https://staging.bitcoin-custody.com",
      "backend": "https://staging-api.bitcoin-custody.com", 
      "health_check": "https://staging-api.bitcoin-custody.com/health",
      "status_check": "https://staging-api.bitcoin-custody.com/api/system/status"
    }
  },
  "monitoring": {
    "check_interval": 30,
    "timeout": 10,
    "retry_count": 3,
    "alert_threshold": 3,
    "performance_baseline": {
      "response_time_ms": 500,
      "error_rate_percent": 1
    }
  },
  "alerts": {
    "slack_webhook": "",
    "email_recipients": [],
    "pagerduty_key": ""
  },
  "metrics": {
    "collect_response_times": true,
    "collect_error_rates": true,
    "collect_availability": true,
    "retention_hours": 24
  }
}
EOF
        success "Created monitoring configuration at $MONITOR_CONFIG"
    fi
}

# Check endpoint health
check_endpoint_health() {
    local url="$1"
    local timeout="${2:-10}"
    local expected_status="${3:-200}"
    
    local response_code
    local response_time
    
    # Measure response time and get status code
    local start_time=$(date +%s%N)
    response_code=$(curl -s -o /dev/null -w "%{http_code}" --max-time "$timeout" "$url" 2>/dev/null || echo "000")
    local end_time=$(date +%s%N)
    
    response_time=$(( (end_time - start_time) / 1000000 )) # Convert to milliseconds
    
    if [ "$response_code" = "$expected_status" ]; then
        echo "healthy,$response_time"
        return 0
    else
        echo "unhealthy,$response_time,$response_code"
        return 1
    fi
}

# Monitor single environment
monitor_environment() {
    local environment="$1"
    local duration="${2:-300}" # 5 minutes default
    
    log "Starting monitoring for $environment environment (duration: ${duration}s)..."
    
    # Get endpoint configuration
    local frontend_url=$(jq -r ".endpoints.${environment}.frontend" "$MONITOR_CONFIG")
    local backend_url=$(jq -r ".endpoints.${environment}.backend" "$MONITOR_CONFIG")
    local health_url=$(jq -r ".endpoints.${environment}.health_check" "$MONITOR_CONFIG")
    local status_url=$(jq -r ".endpoints.${environment}.status_check" "$MONITOR_CONFIG")
    
    # Get monitoring configuration
    local check_interval=$(jq -r '.monitoring.check_interval' "$MONITOR_CONFIG")
    local timeout=$(jq -r '.monitoring.timeout' "$MONITOR_CONFIG")
    local alert_threshold=$(jq -r '.monitoring.alert_threshold' "$MONITOR_CONFIG")
    
    local start_time=$(date +%s)
    local consecutive_failures=0
    local total_checks=0
    local failed_checks=0
    local response_times=()
    
    # Create monitoring log file
    local log_file="$PROJECT_ROOT/monitoring_${environment}_$(date +%Y%m%d_%H%M%S).log"
    echo "timestamp,endpoint,status,response_time_ms,http_code" > "$log_file"
    
    info "Monitoring log: $log_file"
    
    while [ $(($(date +%s) - start_time)) -lt "$duration" ]; do
        local timestamp=$(date '+%Y-%m-%d %H:%M:%S')
        local check_failed=false
        
        info "[$timestamp] Running health checks..."
        
        # Check frontend
        local frontend_result=$(check_endpoint_health "$frontend_url" "$timeout")
        local frontend_status=$(echo "$frontend_result" | cut -d',' -f1)
        local frontend_time=$(echo "$frontend_result" | cut -d',' -f2)
        
        echo "$timestamp,frontend,$frontend_result" >> "$log_file"
        
        if [ "$frontend_status" = "healthy" ]; then
            success "✓ Frontend: ${frontend_time}ms"
        else
            error "✗ Frontend: Failed"
            check_failed=true
        fi
        
        # Check backend health endpoint
        local health_result=$(check_endpoint_health "$health_url" "$timeout")
        local health_status=$(echo "$health_result" | cut -d',' -f1)
        local health_time=$(echo "$health_result" | cut -d',' -f2)
        
        echo "$timestamp,health,$health_result" >> "$log_file"
        
        if [ "$health_status" = "healthy" ]; then
            success "✓ Health endpoint: ${health_time}ms"
        else
            error "✗ Health endpoint: Failed"
            check_failed=true
        fi
        
        # Check backend status endpoint
        local status_result=$(check_endpoint_health "$status_url" "$timeout")
        local status_status=$(echo "$status_result" | cut -d',' -f1)
        local status_time=$(echo "$status_result" | cut -d',' -f2)
        
        echo "$timestamp,status,$status_result" >> "$log_file"
        
        if [ "$status_status" = "healthy" ]; then
            success "✓ Status endpoint: ${status_time}ms"
        else
            error "✗ Status endpoint: Failed"
            check_failed=true
        fi
        
        # Update counters
        total_checks=$((total_checks + 1))
        
        if [ "$check_failed" = true ]; then
            failed_checks=$((failed_checks + 1))
            consecutive_failures=$((consecutive_failures + 1))
            
            # Check if we need to send alerts
            if [ "$consecutive_failures" -ge "$alert_threshold" ]; then
                send_alert "$environment" "Health check failures exceeded threshold ($consecutive_failures/$alert_threshold)"
            fi
        else
            consecutive_failures=0
        fi
        
        # Collect response times for analysis
        response_times+=("$frontend_time" "$health_time" "$status_time")
        
        # Wait for next check
        sleep "$check_interval"
    done
    
    # Generate monitoring summary
    generate_monitoring_summary "$environment" "$total_checks" "$failed_checks" "$log_file" "${response_times[@]}"
}

# Generate monitoring summary
generate_monitoring_summary() {
    local environment="$1"
    local total_checks="$2"
    local failed_checks="$3"
    local log_file="$4"
    shift 4
    local response_times=("$@")
    
    log "Generating monitoring summary for $environment..."
    
    # Calculate metrics
    local success_rate=$(( (total_checks - failed_checks) * 100 / total_checks ))
    local failure_rate=$(( failed_checks * 100 / total_checks ))
    
    # Calculate average response time
    local total_time=0
    local count=0
    for time in "${response_times[@]}"; do
        if [[ "$time" =~ ^[0-9]+$ ]]; then
            total_time=$((total_time + time))
            count=$((count + 1))
        fi
    done
    
    local avg_response_time=0
    if [ "$count" -gt 0 ]; then
        avg_response_time=$((total_time / count))
    fi
    
    # Create summary report
    local summary_file="$PROJECT_ROOT/monitoring_summary_${environment}_$(date +%Y%m%d_%H%M%S).md"
    
    cat > "$summary_file" << EOF
# Production Monitoring Summary

**Environment:** $environment  
**Monitoring Period:** $(date '+%Y-%m-%d %H:%M:%S')  
**Duration:** $(( $(date +%s) - $(stat -f %B "$log_file" 2>/dev/null || stat -c %Y "$log_file") )) seconds

## Health Check Results

| Metric | Value |
|--------|-------|
| Total Checks | $total_checks |
| Successful Checks | $((total_checks - failed_checks)) |
| Failed Checks | $failed_checks |
| Success Rate | ${success_rate}% |
| Failure Rate | ${failure_rate}% |
| Average Response Time | ${avg_response_time}ms |

## Status Assessment

EOF
    
    if [ "$failure_rate" -eq 0 ]; then
        echo "✅ **HEALTHY** - All health checks passed" >> "$summary_file"
    elif [ "$failure_rate" -lt 5 ]; then
        echo "⚠️ **WARNING** - Minor issues detected (${failure_rate}% failure rate)" >> "$summary_file"
    elif [ "$failure_rate" -lt 20 ]; then
        echo "🔶 **DEGRADED** - Significant issues detected (${failure_rate}% failure rate)" >> "$summary_file"
    else
        echo "❌ **CRITICAL** - Major issues detected (${failure_rate}% failure rate)" >> "$summary_file"
    fi
    
    cat >> "$summary_file" << EOF

## Performance Analysis

- **Response Time Baseline:** $(jq -r '.monitoring.performance_baseline.response_time_ms' "$MONITOR_CONFIG")ms
- **Actual Average:** ${avg_response_time}ms
- **Performance Status:** $([ "$avg_response_time" -le "$(jq -r '.monitoring.performance_baseline.response_time_ms' "$MONITOR_CONFIG")" ] && echo "✅ Within baseline" || echo "⚠️ Above baseline")

## Recommendations

EOF
    
    if [ "$failure_rate" -gt 0 ]; then
        echo "1. **Investigate Failures:** Review detailed logs in $log_file" >> "$summary_file"
        echo "2. **Check Infrastructure:** Verify server resources and network connectivity" >> "$summary_file"
        echo "3. **Review Recent Changes:** Check if recent deployments caused issues" >> "$summary_file"
    fi
    
    if [ "$avg_response_time" -gt "$(jq -r '.monitoring.performance_baseline.response_time_ms' "$MONITOR_CONFIG")" ]; then
        echo "4. **Performance Optimization:** Response times are above baseline" >> "$summary_file"
        echo "5. **Resource Scaling:** Consider scaling up resources if needed" >> "$summary_file"
    fi
    
    echo "" >> "$summary_file"
    echo "## Detailed Logs" >> "$summary_file"
    echo "" >> "$summary_file"
    echo "Full monitoring data available in: \`$log_file\`" >> "$summary_file"
    
    success "Monitoring summary generated: $summary_file"
    
    # Display summary to console
    echo
    log "=== MONITORING SUMMARY ==="
    log "Environment: $environment"
    log "Success Rate: ${success_rate}%"
    log "Average Response Time: ${avg_response_time}ms"
    
    if [ "$failure_rate" -eq 0 ]; then
        success "Status: HEALTHY"
    elif [ "$failure_rate" -lt 5 ]; then
        warn "Status: WARNING"
    else
        error "Status: CRITICAL"
    fi
}

# Send alert notifications
send_alert() {
    local environment="$1"
    local message="$2"
    
    warn "ALERT: $message"
    
    # Slack notification
    local slack_webhook=$(jq -r '.alerts.slack_webhook' "$MONITOR_CONFIG")
    if [ "$slack_webhook" != "null" ] && [ -n "$slack_webhook" ]; then
        local payload="{\"text\": \"🚨 Production Alert [$environment]: $message\"}"
        curl -s -X POST -H 'Content-type: application/json' --data "$payload" "$slack_webhook" > /dev/null
        log "Alert sent to Slack"
    fi
    
    # Log alert to file
    local alert_log="$PROJECT_ROOT/alerts.log"
    echo "$(date '+%Y-%m-%d %H:%M:%S') [$environment] $message" >> "$alert_log"
}

# Validate deployment before monitoring
validate_deployment() {
    local environment="$1"
    
    log "Validating $environment deployment before monitoring..."
    
    # Get endpoint URLs
    local health_url=$(jq -r ".endpoints.${environment}.health_check" "$MONITOR_CONFIG")
    local status_url=$(jq -r ".endpoints.${environment}.status_check" "$MONITOR_CONFIG")
    
    # Basic connectivity test
    if ! curl -s --max-time 10 "$health_url" > /dev/null; then
        error "Cannot connect to health endpoint: $health_url"
        return 1
    fi
    
    # Check if status endpoint returns valid JSON
    local status_response=$(curl -s --max-time 10 "$status_url" 2>/dev/null)
    if ! echo "$status_response" | jq . > /dev/null 2>&1; then
        error "Status endpoint not returning valid JSON: $status_url"
        return 1
    fi
    
    success "Deployment validation passed"
}

# Continuous monitoring mode
continuous_monitor() {
    local environment="$1"
    
    log "Starting continuous monitoring for $environment..."
    log "Press Ctrl+C to stop monitoring"
    
    # Trap interrupt signal
    trap 'log "Monitoring stopped by user"; exit 0' INT
    
    while true; do
        monitor_environment "$environment" 300 # 5-minute cycles
        log "Monitoring cycle completed, starting next cycle..."
        sleep 60 # 1-minute break between cycles
    done
}

# Compare environments
compare_environments() {
    log "Comparing staging and production environments..."
    
    local staging_health=$(check_endpoint_health "$(jq -r '.endpoints.staging.health_check' "$MONITOR_CONFIG")" 10)
    local production_health=$(check_endpoint_health "$(jq -r '.endpoints.production.health_check' "$MONITOR_CONFIG")" 10)
    
    local staging_status=$(echo "$staging_health" | cut -d',' -f1)
    local staging_time=$(echo "$staging_health" | cut -d',' -f2)
    local production_status=$(echo "$production_health" | cut -d',' -f1)
    local production_time=$(echo "$production_health" | cut -d',' -f2)
    
    echo
    log "=== ENVIRONMENT COMPARISON ==="
    
    printf "%-12s %-10s %-15s\n" "Environment" "Status" "Response Time"
    printf "%-12s %-10s %-15s\n" "-----------" "------" "-------------"
    printf "%-12s %-10s %-15s\n" "Staging" "$staging_status" "${staging_time}ms"
    printf "%-12s %-10s %-15s\n" "Production" "$production_status" "${production_time}ms"
    
    echo
    
    if [ "$staging_status" = "healthy" ] && [ "$production_status" = "healthy" ]; then
        success "Both environments are healthy"
        
        # Compare response times
        if [ "$staging_time" -lt "$production_time" ]; then
            info "Staging is faster by $((production_time - staging_time))ms"
        elif [ "$production_time" -lt "$staging_time" ]; then
            info "Production is faster by $((staging_time - production_time))ms"
        else
            info "Response times are similar"
        fi
    else
        warn "One or both environments have issues"
    fi
}

# Main command handler
main() {
    init_monitor_config
    
    case "${1:-help}" in
        "monitor")
            if ! validate_deployment "$2"; then
                exit 1
            fi
            monitor_environment "$2" "$3"
            ;;
        "continuous")
            if ! validate_deployment "$2"; then
                exit 1
            fi
            continuous_monitor "$2"
            ;;
        "validate")
            validate_deployment "$2"
            ;;
        "compare")
            compare_environments
            ;;
        "check")
            local url="$2"
            local result=$(check_endpoint_health "$url" 10)
            echo "Result: $result"
            ;;
        "help"|*)
            cat << EOF
Production Deployment Monitor

Usage: $0 <command> [options]

Commands:
  monitor <env> [duration]    Monitor environment for specified duration (seconds)
  continuous <env>           Start continuous monitoring (Ctrl+C to stop)
  validate <env>             Validate deployment before monitoring
  compare                    Compare staging and production environments
  check <url>               Check single endpoint health
  help                      Show this help message

Environments:
  staging                   Staging environment
  production               Production environment

Examples:
  $0 monitor production 600    # Monitor production for 10 minutes
  $0 continuous staging        # Continuous staging monitoring
  $0 validate production       # Validate production deployment
  $0 compare                   # Compare environments
  $0 check https://api.example.com/health

EOF
            ;;
    esac
}

main "$@"