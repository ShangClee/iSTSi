#!/bin/bash

# Security Audit Script for Bitcoin Custody Platform
# This script performs comprehensive security checks across all components

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
AUDIT_REPORT_DIR="$PROJECT_ROOT/security-audit-reports"
TIMESTAMP=$(date +"%Y%m%d_%H%M%S")
REPORT_FILE="$AUDIT_REPORT_DIR/security_audit_$TIMESTAMP.md"

# Create audit report directory
mkdir -p "$AUDIT_REPORT_DIR"

# Initialize report
cat > "$REPORT_FILE" << EOF
# Security Audit Report

**Generated:** $(date)
**Environment:** ${ENVIRONMENT:-development}
**Auditor:** $(whoami)

## Executive Summary

This report contains the results of automated security checks performed on the Bitcoin Custody Platform.

---

EOF

echo -e "${BLUE}🔒 Starting Security Audit for Bitcoin Custody Platform${NC}"
echo -e "${BLUE}Report will be saved to: $REPORT_FILE${NC}"
echo ""

# Function to log results
log_result() {
    local status=$1
    local message=$2
    local details=$3
    
    if [ "$status" = "PASS" ]; then
        echo -e "${GREEN}✅ $message${NC}"
        echo "✅ **PASS**: $message" >> "$REPORT_FILE"
    elif [ "$status" = "FAIL" ]; then
        echo -e "${RED}❌ $message${NC}"
        echo "❌ **FAIL**: $message" >> "$REPORT_FILE"
    elif [ "$status" = "WARN" ]; then
        echo -e "${YELLOW}⚠️  $message${NC}"
        echo "⚠️ **WARNING**: $message" >> "$REPORT_FILE"
    else
        echo -e "${BLUE}ℹ️  $message${NC}"
        echo "ℹ️ **INFO**: $message" >> "$REPORT_FILE"
    fi
    
    if [ -n "$details" ]; then
        echo "   $details" >> "$REPORT_FILE"
    fi
    echo "" >> "$REPORT_FILE"
}

# Function to check file permissions
check_file_permissions() {
    echo -e "\n${BLUE}📁 Checking File Permissions${NC}"
    echo "## File Permissions" >> "$REPORT_FILE"
    
    # Check for world-writable files
    if find "$PROJECT_ROOT" -type f -perm -002 2>/dev/null | grep -q .; then
        log_result "FAIL" "World-writable files found" "$(find "$PROJECT_ROOT" -type f -perm -002 2>/dev/null | head -5)"
    else
        log_result "PASS" "No world-writable files found"
    fi
    
    # Check for executable scripts
    find "$PROJECT_ROOT" -name "*.sh" -type f ! -executable 2>/dev/null | while read -r file; do
        log_result "WARN" "Shell script not executable: $file"
    done
    
    # Check sensitive files
    sensitive_files=(".env" ".env.production" ".env.staging" "config/secrets.yaml")
    for file in "${sensitive_files[@]}"; do
        if [ -f "$PROJECT_ROOT/$file" ]; then
            perms=$(stat -c "%a" "$PROJECT_ROOT/$file" 2>/dev/null || stat -f "%A" "$PROJECT_ROOT/$file" 2>/dev/null)
            if [ "$perms" != "600" ] && [ "$perms" != "0600" ]; then
                log_result "WARN" "Sensitive file has loose permissions: $file ($perms)"
            else
                log_result "PASS" "Sensitive file has correct permissions: $file"
            fi
        fi
    done
}

# Function to check for secrets in code
check_secrets_in_code() {
    echo -e "\n${BLUE}🔐 Checking for Hardcoded Secrets${NC}"
    echo "## Hardcoded Secrets Check" >> "$REPORT_FILE"
    
    # Patterns to search for
    secret_patterns=(
        "password\s*=\s*['\"][^'\"]{3,}"
        "secret\s*=\s*['\"][^'\"]{8,}"
        "api_key\s*=\s*['\"][^'\"]{8,}"
        "private_key\s*=\s*['\"][^'\"]{20,}"
        "jwt_secret\s*=\s*['\"][^'\"]{8,}"
        "database_url\s*=\s*['\"]postgres://[^'\"]*:[^'\"]*@"
    )
    
    secrets_found=false
    for pattern in "${secret_patterns[@]}"; do
        if grep -r -i -E "$pattern" "$PROJECT_ROOT" \
           --exclude-dir=node_modules \
           --exclude-dir=target \
           --exclude-dir=.git \
           --exclude="*.md" \
           --exclude="security-audit.sh" 2>/dev/null | grep -v ".env.example"; then
            secrets_found=true
        fi
    done
    
    if [ "$secrets_found" = true ]; then
        log_result "FAIL" "Potential hardcoded secrets found in code"
    else
        log_result "PASS" "No hardcoded secrets detected"
    fi
}

# Function to check dependencies for vulnerabilities
check_dependencies() {
    echo -e "\n${BLUE}📦 Checking Dependencies for Vulnerabilities${NC}"
    echo "## Dependency Security Check" >> "$REPORT_FILE"
    
    # Check Rust dependencies
    if [ -f "$PROJECT_ROOT/backend/Cargo.toml" ]; then
        cd "$PROJECT_ROOT/backend"
        if command -v cargo-audit >/dev/null 2>&1; then
            if cargo audit 2>/dev/null; then
                log_result "PASS" "Rust dependencies: No known vulnerabilities"
            else
                log_result "FAIL" "Rust dependencies: Vulnerabilities found"
            fi
        else
            log_result "WARN" "cargo-audit not installed, skipping Rust dependency check"
        fi
        cd "$PROJECT_ROOT"
    fi
    
    # Check Node.js dependencies
    if [ -f "$PROJECT_ROOT/frontend/package.json" ]; then
        cd "$PROJECT_ROOT/frontend"
        if command -v npm >/dev/null 2>&1; then
            if npm audit --audit-level=moderate 2>/dev/null; then
                log_result "PASS" "Node.js dependencies: No moderate+ vulnerabilities"
            else
                log_result "FAIL" "Node.js dependencies: Vulnerabilities found"
            fi
        else
            log_result "WARN" "npm not available, skipping Node.js dependency check"
        fi
        cd "$PROJECT_ROOT"
    fi
    
    # Check Soroban dependencies
    if [ -f "$PROJECT_ROOT/soroban/Cargo.toml" ]; then
        cd "$PROJECT_ROOT/soroban"
        if command -v cargo-audit >/dev/null 2>&1; then
            if cargo audit 2>/dev/null; then
                log_result "PASS" "Soroban dependencies: No known vulnerabilities"
            else
                log_result "FAIL" "Soroban dependencies: Vulnerabilities found"
            fi
        fi
        cd "$PROJECT_ROOT"
    fi
}

# Function to check configuration security
check_configuration() {
    echo -e "\n${BLUE}⚙️  Checking Configuration Security${NC}"
    echo "## Configuration Security" >> "$REPORT_FILE"
    
    # Check for default passwords/secrets
    default_secrets=(
        "password"
        "admin"
        "secret"
        "123456"
        "development-secret-key-change-in-production"
    )
    
    config_files=(
        "backend/config/development.yaml"
        "backend/config/staging.yaml"
        "backend/config/production.yaml"
        "frontend/.env.development"
        "frontend/.env.staging"
        "frontend/.env.production"
    )
    
    for config_file in "${config_files[@]}"; do
        if [ -f "$PROJECT_ROOT/$config_file" ]; then
            for secret in "${default_secrets[@]}"; do
                if grep -q "$secret" "$PROJECT_ROOT/$config_file" 2>/dev/null; then
                    log_result "WARN" "Default/weak secret found in $config_file"
                fi
            done
        fi
    done
    
    # Check CORS configuration
    if [ -f "$PROJECT_ROOT/backend/config/production.yaml" ]; then
        if grep -q "allow_origins:.*\*" "$PROJECT_ROOT/backend/config/production.yaml" 2>/dev/null; then
            log_result "FAIL" "CORS allows all origins (*) in production config"
        else
            log_result "PASS" "CORS configuration appears secure"
        fi
    fi
    
    # Check TLS configuration
    if [ -f "$PROJECT_ROOT/backend/config/production.yaml" ]; then
        if grep -q "min_version.*1\.[01]" "$PROJECT_ROOT/backend/config/production.yaml" 2>/dev/null; then
            log_result "WARN" "TLS minimum version may be too low"
        else
            log_result "PASS" "TLS configuration appears secure"
        fi
    fi
}

# Function to check Docker security
check_docker_security() {
    echo -e "\n${BLUE}🐳 Checking Docker Security${NC}"
    echo "## Docker Security" >> "$REPORT_FILE"
    
    # Check Dockerfile security
    dockerfiles=(
        "backend/Dockerfile.dev"
        "frontend/Dockerfile.dev"
        "Dockerfile.test-runner"
    )
    
    for dockerfile in "${dockerfiles[@]}"; do
        if [ -f "$PROJECT_ROOT/$dockerfile" ]; then
            # Check for running as root
            if ! grep -q "USER " "$PROJECT_ROOT/$dockerfile" 2>/dev/null; then
                log_result "WARN" "$dockerfile: No USER instruction found (may run as root)"
            else
                log_result "PASS" "$dockerfile: USER instruction found"
            fi
            
            # Check for COPY --chown
            if grep -q "COPY.*--chown" "$PROJECT_ROOT/$dockerfile" 2>/dev/null; then
                log_result "PASS" "$dockerfile: Uses --chown for secure file copying"
            fi
            
            # Check for secrets in build args
            if grep -q "ARG.*SECRET\|ARG.*PASSWORD\|ARG.*KEY" "$PROJECT_ROOT/$dockerfile" 2>/dev/null; then
                log_result "WARN" "$dockerfile: May expose secrets in build args"
            fi
        fi
    done
    
    # Check docker-compose security
    if [ -f "$PROJECT_ROOT/docker-compose.yml" ]; then
        # Check for exposed ports
        if grep -q "ports:" "$PROJECT_ROOT/docker-compose.yml" 2>/dev/null; then
            log_result "INFO" "docker-compose.yml exposes ports (review for production)"
        fi
        
        # Check for privileged containers
        if grep -q "privileged: true" "$PROJECT_ROOT/docker-compose.yml" 2>/dev/null; then
            log_result "FAIL" "docker-compose.yml contains privileged containers"
        else
            log_result "PASS" "No privileged containers in docker-compose.yml"
        fi
    fi
}

# Function to check network security
check_network_security() {
    echo -e "\n${BLUE}🌐 Checking Network Security${NC}"
    echo "## Network Security" >> "$REPORT_FILE"
    
    # Check for HTTP URLs in production configs
    if [ -f "$PROJECT_ROOT/frontend/.env.production" ]; then
        if grep -q "http://" "$PROJECT_ROOT/frontend/.env.production" 2>/dev/null; then
            log_result "FAIL" "HTTP URLs found in production frontend config"
        else
            log_result "PASS" "No HTTP URLs in production frontend config"
        fi
    fi
    
    # Check API endpoints for HTTPS
    api_configs=(
        "frontend/src/services/api.ts"
        "backend/config/production.yaml"
    )
    
    for config in "${api_configs[@]}"; do
        if [ -f "$PROJECT_ROOT/$config" ]; then
            if grep -q "http://.*api\|http://.*backend" "$PROJECT_ROOT/$config" 2>/dev/null; then
                log_result "WARN" "$config: Contains HTTP API endpoints"
            fi
        fi
    done
}

# Function to check authentication security
check_authentication() {
    echo -e "\n${BLUE}🔑 Checking Authentication Security${NC}"
    echo "## Authentication Security" >> "$REPORT_FILE"
    
    # Check JWT configuration
    if [ -f "$PROJECT_ROOT/backend/src/middleware/auth.rs" ]; then
        if grep -q "validate_exp.*false" "$PROJECT_ROOT/backend/src/middleware/auth.rs" 2>/dev/null; then
            log_result "FAIL" "JWT expiration validation is disabled"
        else
            log_result "PASS" "JWT expiration validation appears enabled"
        fi
    fi
    
    # Check for password hashing
    if [ -f "$PROJECT_ROOT/backend/src/controllers/auth.rs" ]; then
        if grep -q "bcrypt\|argon2\|scrypt" "$PROJECT_ROOT/backend/src/controllers/auth.rs" 2>/dev/null; then
            log_result "PASS" "Password hashing implementation found"
        else
            log_result "WARN" "No obvious password hashing found in auth controller"
        fi
    fi
    
    # Check session configuration
    config_files=(
        "backend/config/production.yaml"
        "backend/config/staging.yaml"
    )
    
    for config in "${config_files[@]}"; do
        if [ -f "$PROJECT_ROOT/$config" ]; then
            if grep -q "secure: true" "$PROJECT_ROOT/$config" 2>/dev/null; then
                log_result "PASS" "$config: Secure session configuration"
            else
                log_result "WARN" "$config: Session security settings not found"
            fi
        fi
    done
}

# Function to check logging and monitoring
check_logging_monitoring() {
    echo -e "\n${BLUE}📊 Checking Logging and Monitoring${NC}"
    echo "## Logging and Monitoring" >> "$REPORT_FILE"
    
    # Check for audit logging
    if [ -f "$PROJECT_ROOT/backend/config/production.yaml" ]; then
        if grep -q "audit.*enabled: true" "$PROJECT_ROOT/backend/config/production.yaml" 2>/dev/null; then
            log_result "PASS" "Audit logging is enabled in production"
        else
            log_result "WARN" "Audit logging not explicitly enabled in production"
        fi
    fi
    
    # Check for security event logging
    if [ -f "$PROJECT_ROOT/backend/src/services/security_service.rs" ]; then
        if grep -q "log_security_event" "$PROJECT_ROOT/backend/src/services/security_service.rs" 2>/dev/null; then
            log_result "PASS" "Security event logging implementation found"
        else
            log_result "WARN" "Security event logging not found"
        fi
    fi
    
    # Check for sensitive data in logs
    log_patterns=(
        "password"
        "secret"
        "token"
        "private_key"
    )
    
    for pattern in "${log_patterns[@]}"; do
        if grep -r -i "log.*$pattern\|println.*$pattern" "$PROJECT_ROOT/backend/src" \
           --exclude-dir=target 2>/dev/null | grep -v "mask\|redact\|sanitize"; then
            log_result "WARN" "Potential sensitive data logging: $pattern"
        fi
    done
}

# Function to generate recommendations
generate_recommendations() {
    echo -e "\n${BLUE}💡 Generating Security Recommendations${NC}"
    echo "## Security Recommendations" >> "$REPORT_FILE"
    
    cat >> "$REPORT_FILE" << 'EOF'

### Immediate Actions Required
- [ ] Review and fix all FAIL items above
- [ ] Address high-priority WARNING items
- [ ] Ensure all secrets are properly managed and rotated
- [ ] Verify TLS/SSL configuration in production

### Security Best Practices
- [ ] Implement regular security audits (monthly)
- [ ] Set up automated dependency vulnerability scanning
- [ ] Configure security monitoring and alerting
- [ ] Establish incident response procedures
- [ ] Conduct penetration testing before production deployment

### Monitoring and Maintenance
- [ ] Set up log monitoring for security events
- [ ] Implement automated backup verification
- [ ] Configure health checks for all services
- [ ] Establish security metrics and KPIs

### Documentation
- [ ] Document security architecture and controls
- [ ] Create security runbooks for common scenarios
- [ ] Maintain security contact information
- [ ] Document data classification and handling procedures

EOF
}

# Main execution
main() {
    check_file_permissions
    check_secrets_in_code
    check_dependencies
    check_configuration
    check_docker_security
    check_network_security
    check_authentication
    check_logging_monitoring
    generate_recommendations
    
    echo -e "\n${GREEN}🎉 Security audit completed!${NC}"
    echo -e "${BLUE}📄 Full report saved to: $REPORT_FILE${NC}"
    
    # Summary
    echo -e "\n${BLUE}📋 Audit Summary:${NC}"
    echo "   PASS: $(grep -c "✅ \*\*PASS\*\*" "$REPORT_FILE" || echo 0)"
    echo "   WARN: $(grep -c "⚠️ \*\*WARNING\*\*" "$REPORT_FILE" || echo 0)"
    echo "   FAIL: $(grep -c "❌ \*\*FAIL\*\*" "$REPORT_FILE" || echo 0)"
    echo "   INFO: $(grep -c "ℹ️ \*\*INFO\*\*" "$REPORT_FILE" || echo 0)"
}

# Run the audit
main "$@"