#!/bin/bash

# Release Coordinator for Bitcoin Custody Full-Stack Application
# Manages coordinated releases across frontend, backend, and soroban components

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
VERSION_CONFIG="$PROJECT_ROOT/version-config.json"
RELEASE_CONFIG="$PROJECT_ROOT/release-config.json"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
NC='\033[0m' # No Color

log() {
    echo -e "${BLUE}[RELEASE-COORDINATOR]${NC} $1"
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

# Initialize release configuration
init_release_config() {
    if [ ! -f "$RELEASE_CONFIG" ]; then
        log "Creating release configuration..."
        cat > "$RELEASE_CONFIG" << 'EOF'
{
  "release_process": {
    "pre_release_checks": [
      "version_compatibility",
      "dependency_validation", 
      "security_audit",
      "test_suite",
      "build_validation"
    ],
    "deployment_order": ["soroban", "backend", "frontend"],
    "rollback_timeout": 300,
    "validation_timeout": 180
  },
  "environments": {
    "staging": {
      "enabled": true,
      "auto_deploy": true,
      "validation_required": true
    },
    "production": {
      "enabled": true,
      "auto_deploy": false,
      "validation_required": true,
      "approval_required": true
    }
  },
  "notifications": {
    "slack_webhook": "",
    "email_recipients": [],
    "github_releases": true
  },
  "monitoring": {
    "health_check_interval": 30,
    "error_threshold": 5,
    "performance_baseline": true
  }
}
EOF
        success "Created release configuration at $RELEASE_CONFIG"
    fi
}

# Check if all tools are available
check_dependencies() {
    local missing_tools=()
    
    for tool in jq git docker-compose curl; do
        if ! command -v "$tool" &> /dev/null; then
            missing_tools+=("$tool")
        fi
    done
    
    if [ ${#missing_tools[@]} -gt 0 ]; then
        error "Missing required tools: ${missing_tools[*]}"
        exit 1
    fi
}

# Run pre-release validation checks
run_pre_release_checks() {
    log "Running pre-release validation checks..."
    
    local checks_passed=true
    
    # Version compatibility check
    info "Checking version compatibility..."
    if ./scripts/version-manager.sh compatibility > /dev/null 2>&1; then
        success "✓ Version compatibility check passed"
    else
        error "✗ Version compatibility check failed"
        checks_passed=false
    fi
    
    # Dependency validation
    info "Validating dependencies..."
    if ./scripts/dependency-validator.sh all > /dev/null 2>&1; then
        success "✓ Dependency validation passed"
    else
        error "✗ Dependency validation failed"
        checks_passed=false
    fi
    
    # Security audit
    info "Running security audit..."
    if ./scripts/security-audit.sh > /dev/null 2>&1; then
        success "✓ Security audit passed"
    else
        warn "⚠ Security audit found issues (check logs)"
    fi
    
    # Test suite
    info "Running test suite..."
    if [ -f "$PROJECT_ROOT/scripts/test.sh" ]; then
        if ./scripts/test.sh > /dev/null 2>&1; then
            success "✓ Test suite passed"
        else
            error "✗ Test suite failed"
            checks_passed=false
        fi
    else
        warn "⚠ Test suite script not found"
    fi
    
    # Build validation
    info "Validating builds..."
    local build_failed=false
    
    # Frontend build
    if [ -d "$PROJECT_ROOT/frontend" ]; then
        cd "$PROJECT_ROOT/frontend"
        if npm run build > /dev/null 2>&1; then
            success "✓ Frontend build successful"
        else
            error "✗ Frontend build failed"
            build_failed=true
        fi
        cd "$PROJECT_ROOT"
    fi
    
    # Backend build
    if [ -d "$PROJECT_ROOT/backend" ]; then
        cd "$PROJECT_ROOT/backend"
        if cargo build --release > /dev/null 2>&1; then
            success "✓ Backend build successful"
        else
            error "✗ Backend build failed"
            build_failed=true
        fi
        cd "$PROJECT_ROOT"
    fi
    
    # Soroban build
    if [ -d "$PROJECT_ROOT/soroban" ]; then
        cd "$PROJECT_ROOT/soroban"
        if cargo build --target wasm32-unknown-unknown --release > /dev/null 2>&1; then
            success "✓ Soroban build successful"
        else
            error "✗ Soroban build failed"
            build_failed=true
        fi
        cd "$PROJECT_ROOT"
    fi
    
    if [ "$build_failed" = true ]; then
        checks_passed=false
    fi
    
    if [ "$checks_passed" = false ]; then
        error "Pre-release checks failed. Please fix issues before proceeding."
        exit 1
    fi
    
    success "All pre-release checks passed!"
}

# Create release branch and prepare for deployment
prepare_release() {
    local version="$1"
    local release_type="${2:-minor}"
    
    if [ -z "$version" ]; then
        error "Version is required for release preparation"
        exit 1
    fi
    
    log "Preparing release $version..."
    
    # Create release branch
    local release_branch="release/v$version"
    
    if git rev-parse --verify "$release_branch" > /dev/null 2>&1; then
        warn "Release branch $release_branch already exists"
        git checkout "$release_branch"
    else
        git checkout -b "$release_branch"
        success "Created release branch: $release_branch"
    fi
    
    # Update all component versions
    log "Updating component versions to $version..."
    ./scripts/version-manager.sh sync "$version"
    
    # Generate changelog entries
    log "Generating changelog entries..."
    for component in frontend backend soroban; do
        ./scripts/changelog-generator.sh generate "$version" "$component"
    done
    
    # Commit version updates
    git add -A
    git commit -m "chore: prepare release v$version

- Update all component versions to $version
- Generate changelog entries
- Prepare for coordinated deployment"
    
    success "Release $version prepared on branch $release_branch"
    
    # Run pre-release checks
    run_pre_release_checks
}

# Deploy to staging environment
deploy_staging() {
    local version="$1"
    
    log "Deploying version $version to staging..."
    
    # Check if staging environment is enabled
    local staging_enabled=$(jq -r '.environments.staging.enabled' "$RELEASE_CONFIG")
    if [ "$staging_enabled" != "true" ]; then
        warn "Staging deployment is disabled in configuration"
        return 0
    fi
    
    # Deploy components in order
    local deployment_order=$(jq -r '.release_process.deployment_order[]' "$RELEASE_CONFIG")
    
    while IFS= read -r component; do
        info "Deploying $component to staging..."
        
        case "$component" in
            "soroban")
                if [ -f "$PROJECT_ROOT/soroban/scripts/deploy-testnet.sh" ]; then
                    cd "$PROJECT_ROOT/soroban"
                    ./scripts/deploy-testnet.sh
                    cd "$PROJECT_ROOT"
                    success "✓ Soroban deployed to testnet"
                else
                    warn "⚠ Soroban deployment script not found"
                fi
                ;;
            "backend")
                # Deploy backend to staging
                if [ -f "$PROJECT_ROOT/docker-compose.staging.yml" ]; then
                    docker-compose -f docker-compose.staging.yml up -d backend
                    success "✓ Backend deployed to staging"
                else
                    warn "⚠ Staging docker-compose file not found"
                fi
                ;;
            "frontend")
                # Deploy frontend to staging
                if [ -f "$PROJECT_ROOT/docker-compose.staging.yml" ]; then
                    docker-compose -f docker-compose.staging.yml up -d frontend
                    success "✓ Frontend deployed to staging"
                else
                    warn "⚠ Staging docker-compose file not found"
                fi
                ;;
        esac
        
        # Wait between deployments
        sleep 10
    done <<< "$deployment_order"
    
    # Validate staging deployment
    validate_deployment "staging"
    
    success "Staging deployment completed successfully"
}

# Deploy to production environment
deploy_production() {
    local version="$1"
    local skip_approval="${2:-false}"
    
    log "Deploying version $version to production..."
    
    # Check if production deployment is enabled
    local prod_enabled=$(jq -r '.environments.production.enabled' "$RELEASE_CONFIG")
    if [ "$prod_enabled" != "true" ]; then
        error "Production deployment is disabled in configuration"
        exit 1
    fi
    
    # Check if approval is required
    local approval_required=$(jq -r '.environments.production.approval_required' "$RELEASE_CONFIG")
    if [ "$approval_required" = "true" ] && [ "$skip_approval" != "true" ]; then
        echo
        warn "Production deployment requires approval."
        read -p "Do you want to proceed with production deployment? (yes/no): " approval
        if [ "$approval" != "yes" ]; then
            log "Production deployment cancelled by user"
            exit 0
        fi
    fi
    
    # Create production deployment tag
    local prod_tag="v$version"
    git tag -a "$prod_tag" -m "Production release $version"
    
    # Deploy components in order
    local deployment_order=$(jq -r '.release_process.deployment_order[]' "$RELEASE_CONFIG")
    
    while IFS= read -r component; do
        info "Deploying $component to production..."
        
        case "$component" in
            "soroban")
                if [ -f "$PROJECT_ROOT/soroban/scripts/deploy-mainnet.sh" ]; then
                    cd "$PROJECT_ROOT/soroban"
                    ./scripts/deploy-mainnet.sh
                    cd "$PROJECT_ROOT"
                    success "✓ Soroban deployed to mainnet"
                else
                    warn "⚠ Soroban mainnet deployment script not found"
                fi
                ;;
            "backend")
                # Deploy backend to production
                if [ -f "$PROJECT_ROOT/docker-compose.production.yml" ]; then
                    docker-compose -f docker-compose.production.yml up -d backend
                    success "✓ Backend deployed to production"
                else
                    warn "⚠ Production docker-compose file not found"
                fi
                ;;
            "frontend")
                # Deploy frontend to production
                if [ -f "$PROJECT_ROOT/docker-compose.production.yml" ]; then
                    docker-compose -f docker-compose.production.yml up -d frontend
                    success "✓ Frontend deployed to production"
                else
                    warn "⚠ Production docker-compose file not found"
                fi
                ;;
        esac
        
        # Wait between deployments
        sleep 15
    done <<< "$deployment_order"
    
    # Validate production deployment
    validate_deployment "production"
    
    # Send notifications
    send_release_notifications "$version" "production"
    
    success "Production deployment completed successfully"
}

# Validate deployment health and functionality
validate_deployment() {
    local environment="$1"
    local validation_timeout=$(jq -r '.release_process.validation_timeout' "$RELEASE_CONFIG")
    
    log "Validating $environment deployment..."
    
    # Determine base URLs based on environment
    local backend_url="http://localhost:8080"
    local frontend_url="http://localhost:3000"
    
    if [ "$environment" = "production" ]; then
        backend_url="https://api.bitcoin-custody.com"
        frontend_url="https://bitcoin-custody.com"
    elif [ "$environment" = "staging" ]; then
        backend_url="https://staging-api.bitcoin-custody.com"
        frontend_url="https://staging.bitcoin-custody.com"
    fi
    
    # Health check with timeout
    local start_time=$(date +%s)
    local health_check_passed=false
    
    while [ $(($(date +%s) - start_time)) -lt "$validation_timeout" ]; do
        info "Checking health endpoints..."
        
        # Check backend health
        if curl -s "$backend_url/health" > /dev/null 2>&1; then
            success "✓ Backend health check passed"
            
            # Check frontend (if accessible)
            if curl -s "$frontend_url" > /dev/null 2>&1; then
                success "✓ Frontend health check passed"
                health_check_passed=true
                break
            fi
        fi
        
        info "Waiting for services to be ready..."
        sleep 10
    done
    
    if [ "$health_check_passed" = false ]; then
        error "Health checks failed within timeout period"
        return 1
    fi
    
    # Run additional validation tests
    info "Running integration validation..."
    
    # Test API endpoints
    if curl -s "$backend_url/api/system/status" | jq -e '.status == "healthy"' > /dev/null 2>&1; then
        success "✓ API endpoints responding correctly"
    else
        warn "⚠ API validation failed"
    fi
    
    # Test database connectivity (if endpoint exists)
    if curl -s "$backend_url/api/system/db-status" > /dev/null 2>&1; then
        success "✓ Database connectivity verified"
    else
        warn "⚠ Database connectivity check not available"
    fi
    
    success "$environment deployment validation completed"
}

# Rollback deployment if issues are detected
rollback_deployment() {
    local environment="$1"
    local target_version="$2"
    
    warn "Initiating rollback for $environment environment..."
    
    if [ -z "$target_version" ]; then
        # Get previous version from git tags
        target_version=$(git tag --sort=-version:refname | grep -v "$(git describe --tags --abbrev=0)" | head -1 | sed 's/^v//')
    fi
    
    if [ -z "$target_version" ]; then
        error "Cannot determine rollback target version"
        exit 1
    fi
    
    log "Rolling back to version $target_version..."
    
    # Checkout previous version
    git checkout "v$target_version"
    
    # Rollback components in reverse order
    local deployment_order=$(jq -r '.release_process.deployment_order[]' "$RELEASE_CONFIG" | tac)
    
    while IFS= read -r component; do
        warn "Rolling back $component..."
        
        case "$component" in
            "frontend")
                if [ "$environment" = "production" ]; then
                    docker-compose -f docker-compose.production.yml up -d frontend
                else
                    docker-compose -f docker-compose.staging.yml up -d frontend
                fi
                ;;
            "backend")
                if [ "$environment" = "production" ]; then
                    docker-compose -f docker-compose.production.yml up -d backend
                else
                    docker-compose -f docker-compose.staging.yml up -d backend
                fi
                ;;
            "soroban")
                # Contract rollback is complex and may not be possible
                warn "⚠ Contract rollback may require manual intervention"
                ;;
        esac
        
        sleep 10
    done <<< "$deployment_order"
    
    # Validate rollback
    if validate_deployment "$environment"; then
        success "Rollback to $target_version completed successfully"
    else
        error "Rollback validation failed - manual intervention required"
        exit 1
    fi
}

# Send release notifications
send_release_notifications() {
    local version="$1"
    local environment="$2"
    
    log "Sending release notifications for $version ($environment)..."
    
    # GitHub release (if enabled)
    local github_releases=$(jq -r '.notifications.github_releases' "$RELEASE_CONFIG")
    if [ "$github_releases" = "true" ]; then
        create_github_release "$version"
    fi
    
    # Slack notification (if configured)
    local slack_webhook=$(jq -r '.notifications.slack_webhook' "$RELEASE_CONFIG")
    if [ "$slack_webhook" != "null" ] && [ -n "$slack_webhook" ]; then
        send_slack_notification "$version" "$environment" "$slack_webhook"
    fi
    
    success "Release notifications sent"
}

# Create GitHub release
create_github_release() {
    local version="$1"
    
    log "Creating GitHub release for v$version..."
    
    # Extract changelog for this version
    local changelog_section=""
    if [ -f "$PROJECT_ROOT/CHANGELOG.md" ]; then
        changelog_section=$(sed -n "/## \[.*v$version\]/,/## \[/p" "$PROJECT_ROOT/CHANGELOG.md" | head -n -1)
    fi
    
    # Create release (requires gh CLI)
    if command -v gh &> /dev/null; then
        echo "$changelog_section" | gh release create "v$version" --title "Release v$version" --notes-file -
        success "✓ GitHub release created"
    else
        warn "⚠ GitHub CLI not available, skipping GitHub release"
    fi
}

# Send Slack notification
send_slack_notification() {
    local version="$1"
    local environment="$2"
    local webhook_url="$3"
    
    local message="🚀 Bitcoin Custody v$version deployed to $environment"
    local payload="{\"text\": \"$message\"}"
    
    if curl -s -X POST -H 'Content-type: application/json' --data "$payload" "$webhook_url" > /dev/null; then
        success "✓ Slack notification sent"
    else
        warn "⚠ Failed to send Slack notification"
    fi
}

# Monitor deployment and auto-rollback if needed
monitor_deployment() {
    local environment="$1"
    local version="$2"
    local monitor_duration="${3:-300}" # 5 minutes default
    
    log "Monitoring $environment deployment for $monitor_duration seconds..."
    
    local error_threshold=$(jq -r '.monitoring.error_threshold' "$RELEASE_CONFIG")
    local check_interval=$(jq -r '.monitoring.health_check_interval' "$RELEASE_CONFIG")
    local error_count=0
    local start_time=$(date +%s)
    
    while [ $(($(date +%s) - start_time)) -lt "$monitor_duration" ]; do
        if ! validate_deployment "$environment" > /dev/null 2>&1; then
            error_count=$((error_count + 1))
            warn "Health check failed (attempt $error_count/$error_threshold)"
            
            if [ "$error_count" -ge "$error_threshold" ]; then
                error "Error threshold exceeded, initiating automatic rollback"
                rollback_deployment "$environment"
                return 1
            fi
        else
            # Reset error count on successful check
            error_count=0
        fi
        
        sleep "$check_interval"
    done
    
    success "Deployment monitoring completed - no issues detected"
}

# Main command handler
main() {
    check_dependencies
    init_release_config
    
    case "${1:-help}" in
        "prepare")
            prepare_release "$2" "$3"
            ;;
        "deploy-staging")
            deploy_staging "$2"
            ;;
        "deploy-production")
            deploy_production "$2" "$3"
            ;;
        "validate")
            validate_deployment "$2"
            ;;
        "rollback")
            rollback_deployment "$2" "$3"
            ;;
        "monitor")
            monitor_deployment "$2" "$3" "$4"
            ;;
        "full-release")
            # Complete release workflow
            local version="$2"
            if [ -z "$version" ]; then
                error "Version required for full release"
                exit 1
            fi
            
            log "Starting full release workflow for version $version"
            prepare_release "$version"
            deploy_staging "$version"
            
            echo
            warn "Staging deployment completed. Please validate manually before proceeding."
            read -p "Proceed with production deployment? (yes/no): " proceed
            
            if [ "$proceed" = "yes" ]; then
                deploy_production "$version"
                monitor_deployment "production" "$version"
            else
                log "Production deployment cancelled"
            fi
            ;;
        "help"|*)
            cat << EOF
Release Coordinator for Bitcoin Custody Full-Stack Application

Usage: $0 <command> [options]

Commands:
  prepare <version> [type]           Prepare release branch and update versions
  deploy-staging <version>           Deploy to staging environment
  deploy-production <version> [skip] Deploy to production (skip=true to skip approval)
  validate <environment>             Validate deployment health
  rollback <environment> [version]   Rollback to previous version
  monitor <env> <version> [duration] Monitor deployment with auto-rollback
  full-release <version>             Complete release workflow (staging -> production)
  help                              Show this help message

Examples:
  $0 prepare 1.2.0                 # Prepare release v1.2.0
  $0 deploy-staging 1.2.0          # Deploy to staging
  $0 deploy-production 1.2.0       # Deploy to production with approval
  $0 validate production           # Validate production deployment
  $0 rollback production 1.1.0     # Rollback production to v1.1.0
  $0 full-release 1.2.0            # Complete release workflow

EOF
            ;;
    esac
}

main "$@"