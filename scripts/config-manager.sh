#!/bin/bash

# Configuration Management Script for Bitcoin Custody Platform
# This script provides comprehensive configuration management capabilities

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
NC='\033[0m' # No Color

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
CONFIG_BACKUP_DIR="$PROJECT_ROOT/.config-backups"
TIMESTAMP=$(date +"%Y%m%d_%H%M%S")

# Default values
ENVIRONMENT="${ENVIRONMENT:-development}"
ACTION=""
BACKUP_ID=""
FORCE=false
VERBOSE=false

# Usage information
usage() {
    cat << EOF
Configuration Manager for Bitcoin Custody Platform

USAGE:
    $0 [OPTIONS] <ACTION>

ACTIONS:
    validate [ENV]          Validate configuration for environment
    backup [ENV]            Create configuration backup
    restore <BACKUP_ID>     Restore from backup
    list-backups [ENV]      List available backups
    consistency-check [ENV] Run consistency checks
    sync-environments       Sync configurations between environments
    generate-template [ENV] Generate configuration template
    security-scan [ENV]     Scan for security issues
    cleanup-backups         Clean up old backups
    help                    Show this help message

OPTIONS:
    -e, --environment ENV   Target environment (development|staging|production)
    -f, --force            Force operation without confirmation
    -v, --verbose          Enable verbose output
    -h, --help             Show this help message

EXAMPLES:
    $0 validate production
    $0 backup staging
    $0 restore config_backup_production_20240115_143022
    $0 consistency-check development
    $0 -e production -v security-scan

ENVIRONMENT VARIABLES:
    ENVIRONMENT            Default environment (default: development)
    CONFIG_BACKUP_DIR      Backup directory (default: .config-backups)

EOF
}

# Logging functions
log_info() {
    echo -e "${BLUE}ℹ️  $1${NC}"
}

log_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

log_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

log_error() {
    echo -e "${RED}❌ $1${NC}"
}

log_debug() {
    if [ "$VERBOSE" = true ]; then
        echo -e "${PURPLE}🔍 $1${NC}"
    fi
}

# Utility functions
confirm_action() {
    local message="$1"
    if [ "$FORCE" = true ]; then
        return 0
    fi
    
    echo -e "${YELLOW}$message${NC}"
    read -p "Continue? (y/N): " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        return 0
    else
        return 1
    fi
}

check_environment() {
    local env="$1"
    case "$env" in
        development|staging|production)
            return 0
            ;;
        *)
            log_error "Invalid environment: $env"
            log_info "Valid environments: development, staging, production"
            return 1
            ;;
    esac
}

ensure_backup_dir() {
    if [ ! -d "$CONFIG_BACKUP_DIR" ]; then
        log_debug "Creating backup directory: $CONFIG_BACKUP_DIR"
        mkdir -p "$CONFIG_BACKUP_DIR"
    fi
}

# Configuration validation
validate_configuration() {
    local env="${1:-$ENVIRONMENT}"
    
    log_info "Validating configuration for environment: $env"
    
    if ! check_environment "$env"; then
        return 1
    fi
    
    local errors=0
    local warnings=0
    
    # Check required files
    local backend_config="$PROJECT_ROOT/backend/config/$env.yaml"
    local frontend_env="$PROJECT_ROOT/frontend/.env.$env"
    
    log_debug "Checking backend configuration: $backend_config"
    if [ -f "$backend_config" ]; then
        log_success "Backend configuration exists: $backend_config"
        
        # Validate YAML syntax
        if command -v yq >/dev/null 2>&1; then
            if yq eval '.' "$backend_config" >/dev/null 2>&1; then
                log_success "Backend configuration syntax is valid"
            else
                log_error "Backend configuration has invalid YAML syntax"
                ((errors++))
            fi
        else
            log_warning "yq not available, skipping YAML syntax validation"
            ((warnings++))
        fi
    else
        log_error "Backend configuration missing: $backend_config"
        ((errors++))
    fi
    
    log_debug "Checking frontend environment: $frontend_env"
    if [ -f "$frontend_env" ]; then
        log_success "Frontend environment file exists: $frontend_env"
    else
        if [ "$env" = "development" ]; then
            log_warning "Frontend environment file missing: $frontend_env"
            ((warnings++))
        else
            log_error "Frontend environment file missing: $frontend_env"
            ((errors++))
        fi
    fi
    
    # Environment-specific validations
    case "$env" in
        production)
            validate_production_config
            local prod_result=$?
            ((errors += prod_result))
            ;;
        staging)
            validate_staging_config
            local staging_result=$?
            ((warnings += staging_result))
            ;;
        development)
            validate_development_config
            ;;
    esac
    
    # Summary
    echo
    log_info "Validation Summary:"
    echo "  Errors: $errors"
    echo "  Warnings: $warnings"
    
    if [ $errors -eq 0 ]; then
        log_success "Configuration validation passed"
        return 0
    else
        log_error "Configuration validation failed with $errors errors"
        return 1
    fi
}

validate_production_config() {
    local errors=0
    
    log_info "Running production-specific validations..."
    
    # Check required environment variables
    local required_vars=(
        "DATABASE_URL"
        "JWT_SECRET"
        "SOROBAN_SOURCE_SECRET"
        "FRONTEND_URL"
        "TLS_CERT_FILE"
        "TLS_KEY_FILE"
    )
    
    for var in "${required_vars[@]}"; do
        if [ -z "${!var}" ]; then
            log_error "Required environment variable not set: $var"
            ((errors++))
        else
            log_debug "Environment variable set: $var"
        fi
    done
    
    # Check TLS files exist
    if [ -n "$TLS_CERT_FILE" ] && [ ! -f "$TLS_CERT_FILE" ]; then
        log_error "TLS certificate file not found: $TLS_CERT_FILE"
        ((errors++))
    fi
    
    if [ -n "$TLS_KEY_FILE" ] && [ ! -f "$TLS_KEY_FILE" ]; then
        log_error "TLS private key file not found: $TLS_KEY_FILE"
        ((errors++))
    fi
    
    # Check HTTPS URLs
    if [ -n "$FRONTEND_URL" ] && [[ ! "$FRONTEND_URL" =~ ^https:// ]]; then
        log_error "Frontend URL must use HTTPS in production: $FRONTEND_URL"
        ((errors++))
    fi
    
    return $errors
}

validate_staging_config() {
    local warnings=0
    
    log_info "Running staging-specific validations..."
    
    # Check for HTTPS (warning only)
    if [ -n "$FRONTEND_URL" ] && [[ ! "$FRONTEND_URL" =~ ^https:// ]]; then
        log_warning "Frontend URL should use HTTPS in staging: $FRONTEND_URL"
        ((warnings++))
    fi
    
    return $warnings
}

validate_development_config() {
    log_info "Running development-specific validations..."
    
    # Check for default secrets
    if [ "$JWT_SECRET" = "development-secret-key-change-in-production" ]; then
        log_warning "Using default JWT secret in development"
    fi
    
    return 0
}

# Configuration backup
backup_configuration() {
    local env="${1:-$ENVIRONMENT}"
    
    log_info "Creating configuration backup for environment: $env"
    
    if ! check_environment "$env"; then
        return 1
    fi
    
    ensure_backup_dir
    
    local backup_id="config_backup_${env}_${TIMESTAMP}"
    local backup_path="$CONFIG_BACKUP_DIR/$backup_id"
    
    log_debug "Creating backup directory: $backup_path"
    mkdir -p "$backup_path"
    
    # Files to backup
    local files_to_backup=(
        "backend/config/$env.yaml"
        "frontend/.env.$env"
        "docker-compose.yml"
        "soroban/config/network_config.toml"
    )
    
    # Add environment-specific docker-compose file
    if [ "$env" != "development" ] && [ -f "$PROJECT_ROOT/docker-compose.$env.yml" ]; then
        files_to_backup+=("docker-compose.$env.yml")
    fi
    
    local backed_up_count=0
    
    for file in "${files_to_backup[@]}"; do
        local source_path="$PROJECT_ROOT/$file"
        local backup_file_path="$backup_path/$(basename "$file")"
        
        if [ -f "$source_path" ]; then
            log_debug "Backing up: $file"
            cp "$source_path" "$backup_file_path"
            ((backed_up_count++))
        else
            log_warning "File not found, skipping: $file"
        fi
    done
    
    # Create backup metadata
    cat > "$backup_path/backup_metadata.json" << EOF
{
    "id": "$backup_id",
    "timestamp": "$(date -u +"%Y-%m-%dT%H:%M:%SZ")",
    "environment": "$env",
    "created_by": "$(whoami)",
    "file_count": $backed_up_count,
    "description": "Configuration backup for $env environment"
}
EOF
    
    log_success "Configuration backup created: $backup_id"
    log_info "Backup location: $backup_path"
    log_info "Files backed up: $backed_up_count"
    
    return 0
}

# List backups
list_backups() {
    local env="$1"
    
    ensure_backup_dir
    
    log_info "Available configuration backups:"
    
    if [ ! -d "$CONFIG_BACKUP_DIR" ] || [ -z "$(ls -A "$CONFIG_BACKUP_DIR" 2>/dev/null)" ]; then
        log_warning "No backups found"
        return 0
    fi
    
    echo
    printf "%-40s %-15s %-20s %-10s\n" "BACKUP ID" "ENVIRONMENT" "TIMESTAMP" "FILES"
    printf "%-40s %-15s %-20s %-10s\n" "$(printf '%*s' 40 '' | tr ' ' '-')" "$(printf '%*s' 15 '' | tr ' ' '-')" "$(printf '%*s' 20 '' | tr ' ' '-')" "$(printf '%*s' 10 '' | tr ' ' '-')"
    
    for backup_dir in "$CONFIG_BACKUP_DIR"/config_backup_*; do
        if [ -d "$backup_dir" ]; then
            local metadata_file="$backup_dir/backup_metadata.json"
            if [ -f "$metadata_file" ]; then
                local backup_id=$(basename "$backup_dir")
                local backup_env=$(jq -r '.environment // "unknown"' "$metadata_file" 2>/dev/null || echo "unknown")
                local timestamp=$(jq -r '.timestamp // "unknown"' "$metadata_file" 2>/dev/null || echo "unknown")
                local file_count=$(jq -r '.file_count // "0"' "$metadata_file" 2>/dev/null || echo "0")
                
                # Filter by environment if specified
                if [ -z "$env" ] || [ "$backup_env" = "$env" ]; then
                    printf "%-40s %-15s %-20s %-10s\n" "$backup_id" "$backup_env" "$timestamp" "$file_count"
                fi
            fi
        fi
    done
    
    echo
}

# Restore configuration
restore_configuration() {
    local backup_id="$1"
    
    if [ -z "$backup_id" ]; then
        log_error "Backup ID is required for restore operation"
        return 1
    fi
    
    local backup_path="$CONFIG_BACKUP_DIR/$backup_id"
    
    if [ ! -d "$backup_path" ]; then
        log_error "Backup not found: $backup_id"
        return 1
    fi
    
    local metadata_file="$backup_path/backup_metadata.json"
    if [ ! -f "$metadata_file" ]; then
        log_error "Backup metadata not found: $backup_id"
        return 1
    fi
    
    local backup_env=$(jq -r '.environment' "$metadata_file" 2>/dev/null)
    local backup_timestamp=$(jq -r '.timestamp' "$metadata_file" 2>/dev/null)
    
    log_info "Restoring configuration from backup:"
    log_info "  Backup ID: $backup_id"
    log_info "  Environment: $backup_env"
    log_info "  Created: $backup_timestamp"
    
    if ! confirm_action "This will overwrite current configuration files."; then
        log_info "Restore cancelled"
        return 0
    fi
    
    # Create emergency backup before restore
    log_info "Creating emergency backup before restore..."
    backup_configuration "$backup_env"
    
    # Restore files
    local restored_count=0
    
    for backup_file in "$backup_path"/*; do
        if [ -f "$backup_file" ] && [ "$(basename "$backup_file")" != "backup_metadata.json" ]; then
            local filename=$(basename "$backup_file")
            local target_path=""
            
            case "$filename" in
                "$backup_env.yaml")
                    target_path="$PROJECT_ROOT/backend/config/$filename"
                    ;;
                ".env.$backup_env")
                    target_path="$PROJECT_ROOT/frontend/$filename"
                    ;;
                "docker-compose.yml")
                    target_path="$PROJECT_ROOT/$filename"
                    ;;
                "docker-compose.$backup_env.yml")
                    target_path="$PROJECT_ROOT/$filename"
                    ;;
                "network_config.toml")
                    target_path="$PROJECT_ROOT/soroban/config/$filename"
                    ;;
                *)
                    log_warning "Unknown file in backup, skipping: $filename"
                    continue
                    ;;
            esac
            
            if [ -n "$target_path" ]; then
                log_debug "Restoring: $filename -> $target_path"
                
                # Create parent directory if needed
                mkdir -p "$(dirname "$target_path")"
                
                # Copy file
                cp "$backup_file" "$target_path"
                ((restored_count++))
            fi
        fi
    done
    
    log_success "Configuration restore completed"
    log_info "Files restored: $restored_count"
    
    # Validate restored configuration
    log_info "Validating restored configuration..."
    if validate_configuration "$backup_env"; then
        log_success "Restored configuration is valid"
    else
        log_warning "Restored configuration has validation issues"
    fi
    
    return 0
}

# Consistency check
consistency_check() {
    local env="${1:-$ENVIRONMENT}"
    
    log_info "Running consistency check for environment: $env"
    
    if ! check_environment "$env"; then
        return 1
    fi
    
    # This would integrate with the Rust consistency checker
    # For now, we'll run basic checks
    
    local issues=0
    
    # Check file consistency
    log_info "Checking file consistency..."
    
    local backend_config="$PROJECT_ROOT/backend/config/$env.yaml"
    local frontend_env="$PROJECT_ROOT/frontend/.env.$env"
    
    if [ -f "$backend_config" ] && [ -f "$frontend_env" ]; then
        # Check if API URLs match
        local backend_port=$(yq eval '.server.port // 8080' "$backend_config" 2>/dev/null)
        local frontend_api_url=$(grep "VITE_API_URL" "$frontend_env" 2>/dev/null | cut -d'=' -f2 | tr -d '"' || echo "")
        
        if [ -n "$frontend_api_url" ] && [[ "$frontend_api_url" =~ :([0-9]+) ]]; then
            local frontend_port="${BASH_REMATCH[1]}"
            if [ "$backend_port" != "$frontend_port" ]; then
                log_warning "Port mismatch: Backend ($backend_port) vs Frontend ($frontend_port)"
                ((issues++))
            else
                log_success "API ports are consistent"
            fi
        fi
    fi
    
    # Check environment variable consistency
    log_info "Checking environment variable consistency..."
    
    # This is a simplified check - the full implementation would be in Rust
    if [ "$env" = "production" ]; then
        local required_vars=("DATABASE_URL" "JWT_SECRET" "FRONTEND_URL")
        for var in "${required_vars[@]}"; do
            if [ -z "${!var}" ]; then
                log_error "Required variable not set: $var"
                ((issues++))
            fi
        done
    fi
    
    # Summary
    echo
    if [ $issues -eq 0 ]; then
        log_success "Consistency check passed"
        return 0
    else
        log_warning "Consistency check found $issues issues"
        return 1
    fi
}

# Security scan
security_scan() {
    local env="${1:-$ENVIRONMENT}"
    
    log_info "Running security scan for environment: $env"
    
    if ! check_environment "$env"; then
        return 1
    fi
    
    local security_issues=0
    
    # Check for default secrets
    log_info "Checking for default/weak secrets..."
    
    local config_files=(
        "backend/config/$env.yaml"
        "frontend/.env.$env"
    )
    
    local weak_patterns=(
        "password"
        "admin"
        "secret"
        "123456"
        "development-secret-key-change-in-production"
    )
    
    for config_file in "${config_files[@]}"; do
        local full_path="$PROJECT_ROOT/$config_file"
        if [ -f "$full_path" ]; then
            for pattern in "${weak_patterns[@]}"; do
                if grep -q "$pattern" "$full_path" 2>/dev/null; then
                    log_warning "Potential weak secret in $config_file: $pattern"
                    ((security_issues++))
                fi
            done
        fi
    done
    
    # Check file permissions
    log_info "Checking file permissions..."
    
    for config_file in "${config_files[@]}"; do
        local full_path="$PROJECT_ROOT/$config_file"
        if [ -f "$full_path" ]; then
            local perms=$(stat -c "%a" "$full_path" 2>/dev/null || stat -f "%A" "$full_path" 2>/dev/null)
            if [ "$perms" != "600" ] && [ "$perms" != "0600" ]; then
                log_warning "Insecure file permissions for $config_file: $perms"
                ((security_issues++))
            fi
        fi
    done
    
    # Environment-specific security checks
    if [ "$env" = "production" ]; then
        log_info "Running production security checks..."
        
        # Check HTTPS usage
        if [ -n "$FRONTEND_URL" ] && [[ ! "$FRONTEND_URL" =~ ^https:// ]]; then
            log_error "Frontend URL must use HTTPS in production"
            ((security_issues++))
        fi
        
        # Check database SSL
        if [ -n "$DATABASE_URL" ] && [[ ! "$DATABASE_URL" =~ sslmode=require ]] && [[ ! "$DATABASE_URL" =~ ssl=true ]]; then
            log_warning "Database connection should use SSL in production"
            ((security_issues++))
        fi
    fi
    
    # Summary
    echo
    if [ $security_issues -eq 0 ]; then
        log_success "Security scan passed"
        return 0
    else
        log_warning "Security scan found $security_issues potential issues"
        return 1
    fi
}

# Cleanup old backups
cleanup_backups() {
    local retention_days="${1:-30}"
    
    log_info "Cleaning up backups older than $retention_days days..."
    
    ensure_backup_dir
    
    if [ ! -d "$CONFIG_BACKUP_DIR" ]; then
        log_info "No backup directory found"
        return 0
    fi
    
    local cleaned_count=0
    local cutoff_date=$(date -d "$retention_days days ago" +%s 2>/dev/null || date -v-${retention_days}d +%s 2>/dev/null)
    
    for backup_dir in "$CONFIG_BACKUP_DIR"/config_backup_*; do
        if [ -d "$backup_dir" ]; then
            local backup_date=$(stat -c %Y "$backup_dir" 2>/dev/null || stat -f %m "$backup_dir" 2>/dev/null)
            
            if [ "$backup_date" -lt "$cutoff_date" ]; then
                log_debug "Removing old backup: $(basename "$backup_dir")"
                rm -rf "$backup_dir"
                ((cleaned_count++))
            fi
        fi
    done
    
    log_success "Cleaned up $cleaned_count old backups"
    return 0
}

# Generate configuration template
generate_template() {
    local env="${1:-development}"
    
    log_info "Generating configuration template for environment: $env"
    
    if ! check_environment "$env"; then
        return 1
    fi
    
    local template_dir="$PROJECT_ROOT/config-templates"
    mkdir -p "$template_dir"
    
    # Generate backend config template
    local backend_template="$template_dir/backend-$env.yaml.template"
    
    cat > "$backend_template" << EOF
# Backend Configuration Template for $env Environment
# Copy this file to backend/config/$env.yaml and customize

server:
  port: 8080
  host: 0.0.0.0
  ident: bitcoin-custody-backend-$env

database:
  uri: \${DATABASE_URL}
  enable_logging: $([ "$env" = "production" ] && echo "false" || echo "true")
  auto_migrate: $([ "$env" = "production" ] && echo "false" || echo "true")
  connect_timeout: $([ "$env" = "production" ] && echo "5000" || echo "3000")
  idle_timeout: 300
  min_connections: $([ "$env" = "production" ] && echo "5" || echo "2")
  max_connections: $([ "$env" = "production" ] && echo "50" || echo "20")

auth:
  jwt:
    secret: \${JWT_SECRET}
    expiration: $([ "$env" = "production" ] && echo "3600" || echo "7200")

soroban:
  network: $([ "$env" = "production" ] && echo "mainnet" || echo "testnet")
  rpc_url: \${SOROBAN_RPC_URL}
  network_passphrase: \${SOROBAN_NETWORK_PASSPHRASE}

cors:
  enable: true
  allow_origins: 
    - \${FRONTEND_URL}
  allow_methods: 
    - "GET"
    - "POST" 
    - "PUT"
    - "DELETE"
    - "OPTIONS"
  allow_headers:
    - "Content-Type"
    - "Authorization"

logger:
  enable: true
  level: $([ "$env" = "production" ] && echo "info" || echo "debug")
  format: json
EOF
    
    # Generate frontend env template
    local frontend_template="$template_dir/frontend-$env.env.template"
    
    cat > "$frontend_template" << EOF
# Frontend Environment Template for $env Environment
# Copy this file to frontend/.env.$env and customize

VITE_APP_ENV=$env
VITE_APP_VERSION=1.0.0

# API Configuration
VITE_API_URL=\${API_URL}
VITE_WS_URL=\${WS_URL}

# Security Configuration
VITE_ENABLE_CSRF_PROTECTION=true
VITE_ENABLE_REQUEST_SIGNING=$([ "$env" = "production" ] && echo "true" || echo "false")

# Feature Flags
VITE_ENABLE_DEBUG_MODE=$([ "$env" = "development" ] && echo "true" || echo "false")
VITE_ENABLE_MOCK_DATA=false

# Soroban Network Configuration
VITE_SOROBAN_NETWORK=$([ "$env" = "production" ] && echo "mainnet" || echo "testnet")
VITE_SOROBAN_RPC_URL=\${SOROBAN_RPC_URL}
EOF
    
    log_success "Configuration templates generated:"
    log_info "  Backend: $backend_template"
    log_info "  Frontend: $frontend_template"
    
    return 0
}

# Parse command line arguments
parse_args() {
    while [[ $# -gt 0 ]]; do
        case $1 in
            -e|--environment)
                ENVIRONMENT="$2"
                shift 2
                ;;
            -f|--force)
                FORCE=true
                shift
                ;;
            -v|--verbose)
                VERBOSE=true
                shift
                ;;
            -h|--help)
                usage
                exit 0
                ;;
            validate|backup|restore|list-backups|consistency-check|sync-environments|generate-template|security-scan|cleanup-backups|help)
                ACTION="$1"
                shift
                break
                ;;
            *)
                log_error "Unknown option: $1"
                usage
                exit 1
                ;;
        esac
    done
    
    # Remaining arguments are action-specific
    case "$ACTION" in
        validate|backup|consistency-check|security-scan|generate-template)
            if [ $# -gt 0 ]; then
                ENVIRONMENT="$1"
            fi
            ;;
        restore)
            if [ $# -eq 0 ]; then
                log_error "Backup ID is required for restore action"
                exit 1
            fi
            BACKUP_ID="$1"
            ;;
        list-backups)
            if [ $# -gt 0 ]; then
                ENVIRONMENT="$1"
            else
                ENVIRONMENT=""  # List all environments
            fi
            ;;
        cleanup-backups)
            # Optional retention days argument
            ;;
    esac
}

# Main execution
main() {
    if [ $# -eq 0 ]; then
        usage
        exit 1
    fi
    
    parse_args "$@"
    
    if [ -z "$ACTION" ]; then
        log_error "No action specified"
        usage
        exit 1
    fi
    
    log_debug "Action: $ACTION"
    log_debug "Environment: $ENVIRONMENT"
    log_debug "Force: $FORCE"
    log_debug "Verbose: $VERBOSE"
    
    case "$ACTION" in
        validate)
            validate_configuration "$ENVIRONMENT"
            ;;
        backup)
            backup_configuration "$ENVIRONMENT"
            ;;
        restore)
            restore_configuration "$BACKUP_ID"
            ;;
        list-backups)
            list_backups "$ENVIRONMENT"
            ;;
        consistency-check)
            consistency_check "$ENVIRONMENT"
            ;;
        security-scan)
            security_scan "$ENVIRONMENT"
            ;;
        cleanup-backups)
            cleanup_backups
            ;;
        generate-template)
            generate_template "$ENVIRONMENT"
            ;;
        help)
            usage
            ;;
        *)
            log_error "Unknown action: $ACTION"
            usage
            exit 1
            ;;
    esac
}

# Run the script
main "$@"