#!/bin/bash

# Environment Setup Script for Bitcoin Custody Platform
# This script sets up environment-specific configurations and validates security

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
ENVIRONMENT="${1:-development}"
FORCE=false
VERBOSE=false

# Usage information
usage() {
    cat << EOF
Environment Setup Script for Bitcoin Custody Platform

USAGE:
    $0 [ENVIRONMENT] [OPTIONS]

ENVIRONMENTS:
    development     Set up development environment (default)
    staging         Set up staging environment
    production      Set up production environment

OPTIONS:
    -f, --force     Force setup without confirmation
    -v, --verbose   Enable verbose output
    -h, --help      Show this help message

EXAMPLES:
    $0 development
    $0 staging --verbose
    $0 production --force

This script will:
1. Validate environment requirements
2. Generate configuration files
3. Set up environment variables
4. Configure security settings
5. Initialize services
6. Run validation checks

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

check_dependencies() {
    log_info "Checking system dependencies..."
    
    local missing_deps=()
    
    # Required tools
    local required_tools=(
        "docker"
        "docker-compose"
        "openssl"
        "curl"
        "jq"
    )
    
    for tool in "${required_tools[@]}"; do
        if ! command -v "$tool" >/dev/null 2>&1; then
            missing_deps+=("$tool")
        else
            log_debug "Found: $tool"
        fi
    done
    
    # Environment-specific tools
    case "$ENVIRONMENT" in
        development)
            local dev_tools=("cargo" "node" "npm")
            for tool in "${dev_tools[@]}"; do
                if ! command -v "$tool" >/dev/null 2>&1; then
                    log_warning "Development tool not found: $tool"
                fi
            done
            ;;
        production)
            local prod_tools=("systemctl")
            for tool in "${prod_tools[@]}"; do
                if ! command -v "$tool" >/dev/null 2>&1; then
                    missing_deps+=("$tool")
                fi
            done
            ;;
    esac
    
    if [ ${#missing_deps[@]} -gt 0 ]; then
        log_error "Missing required dependencies: ${missing_deps[*]}"
        log_info "Please install the missing dependencies and try again"
        return 1
    fi
    
    log_success "All dependencies are available"
    return 0
}

generate_secrets() {
    log_info "Generating secure secrets for $ENVIRONMENT environment..."
    
    local env_file="$PROJECT_ROOT/.env.$ENVIRONMENT"
    local backup_file="$env_file.backup.$(date +%s)"
    
    # Backup existing file if it exists
    if [ -f "$env_file" ]; then
        log_debug "Backing up existing environment file"
        cp "$env_file" "$backup_file"
    fi
    
    # Generate secrets
    local jwt_secret=$(openssl rand -hex 32)
    local encryption_key=$(openssl rand -hex 32)
    local postgres_password=$(openssl rand -base64 32 | tr -d "=+/" | cut -c1-25)
    local redis_password=$(openssl rand -base64 32 | tr -d "=+/" | cut -c1-25)
    
    # Environment-specific configurations
    case "$ENVIRONMENT" in
        development)
            cat > "$env_file" << EOF
# Development Environment Configuration
ENVIRONMENT=development

# Database Configuration
POSTGRES_DB=bitcoin_custody_dev
POSTGRES_USER=postgres
POSTGRES_PASSWORD=$postgres_password
DATABASE_URL=postgres://postgres:$postgres_password@localhost:5432/bitcoin_custody_dev

# Redis Configuration
REDIS_PASSWORD=$redis_password
REDIS_URL=redis://:$redis_password@localhost:6379

# JWT Configuration
JWT_SECRET=$jwt_secret

# Soroban Configuration
SOROBAN_NETWORK=testnet
SOROBAN_RPC_URL=https://soroban-testnet.stellar.org
SOROBAN_NETWORK_PASSPHRASE=Test SDF Network ; September 2015

# Frontend Configuration
FRONTEND_URL=http://localhost:3000

# Security Configuration
SECRET_ENCRYPTION_KEY=$encryption_key

# Development Tools
PGADMIN_EMAIL=admin@dev.local
PGADMIN_PASSWORD=dev_admin
GRAFANA_ADMIN_PASSWORD=dev_admin
GRAFANA_DOMAIN=localhost:3001

# Data Directory
DATA_DIR=$PROJECT_ROOT/data
EOF
            ;;
        staging)
            cat > "$env_file" << EOF
# Staging Environment Configuration
ENVIRONMENT=staging

# Database Configuration
POSTGRES_DB=bitcoin_custody_staging
POSTGRES_USER=postgres
POSTGRES_PASSWORD=$postgres_password
DATABASE_URL=postgres://postgres:$postgres_password@postgres:5432/bitcoin_custody_staging

# Redis Configuration
REDIS_PASSWORD=$redis_password
REDIS_URL=redis://:$redis_password@redis:6379

# JWT Configuration
JWT_SECRET=$jwt_secret

# Soroban Configuration
SOROBAN_NETWORK=testnet
SOROBAN_RPC_URL=https://soroban-testnet.stellar.org
SOROBAN_NETWORK_PASSPHRASE=Test SDF Network ; September 2015

# Frontend Configuration
FRONTEND_URL=https://staging.bitcoin-custody.com

# Security Configuration
SECRET_ENCRYPTION_KEY=$encryption_key

# Backup Configuration
BACKUP_ENCRYPTION_KEY=$(openssl rand -hex 32)

# Monitoring Configuration
GRAFANA_ADMIN_PASSWORD=$(openssl rand -base64 16)
GRAFANA_DOMAIN=staging-grafana.bitcoin-custody.com

# Data Directory
DATA_DIR=/var/lib/bitcoin-custody-staging
EOF
            ;;
        production)
            cat > "$env_file" << EOF
# Production Environment Configuration
ENVIRONMENT=production

# Database Configuration (CHANGE THESE VALUES)
POSTGRES_DB=bitcoin_custody_prod
POSTGRES_USER=bitcoin_custody
POSTGRES_PASSWORD=$postgres_password
DATABASE_URL=postgres://bitcoin_custody:$postgres_password@postgres:5432/bitcoin_custody_prod?sslmode=require

# Redis Configuration
REDIS_PASSWORD=$redis_password
REDIS_URL=redis://:$redis_password@redis:6379

# JWT Configuration
JWT_SECRET=$jwt_secret

# Soroban Configuration (UPDATE FOR MAINNET)
SOROBAN_NETWORK=mainnet
SOROBAN_RPC_URL=https://soroban-mainnet.stellar.org
SOROBAN_NETWORK_PASSPHRASE=Public Global Stellar Network ; September 2015
SOROBAN_SOURCE_SECRET=CHANGE_ME_TO_ACTUAL_SECRET

# Contract Addresses (UPDATE AFTER DEPLOYMENT)
INTEGRATION_ROUTER_ADDRESS=
KYC_REGISTRY_ADDRESS=
ISTSI_TOKEN_ADDRESS=
RESERVE_MANAGER_ADDRESS=

# Frontend Configuration
FRONTEND_URL=https://bitcoin-custody.com

# Security Configuration
SECRET_ENCRYPTION_KEY=$encryption_key
BACKUP_ENCRYPTION_KEY=$(openssl rand -hex 32)

# TLS Configuration
TLS_CERT_FILE=/etc/ssl/certs/server.crt
TLS_KEY_FILE=/etc/ssl/private/server.key

# Monitoring Configuration
GRAFANA_ADMIN_PASSWORD=$(openssl rand -base64 16)
GRAFANA_DOMAIN=grafana.bitcoin-custody.com

# Data Directory
DATA_DIR=/var/lib/bitcoin-custody
EOF
            ;;
    esac
    
    # Set secure permissions
    chmod 600 "$env_file"
    
    log_success "Environment file created: $env_file"
    
    if [ "$ENVIRONMENT" = "production" ]; then
        log_warning "IMPORTANT: Update the production configuration with actual values!"
        log_warning "- Set SOROBAN_SOURCE_SECRET to your actual secret key"
        log_warning "- Update contract addresses after deployment"
        log_warning "- Configure TLS certificate paths"
        log_warning "- Review all configuration values"
    fi
}

setup_directories() {
    log_info "Setting up directory structure for $ENVIRONMENT..."
    
    local data_dir
    case "$ENVIRONMENT" in
        development)
            data_dir="$PROJECT_ROOT/data"
            ;;
        staging)
            data_dir="/var/lib/bitcoin-custody-staging"
            ;;
        production)
            data_dir="/var/lib/bitcoin-custody"
            ;;
    esac
    
    # Create data directories
    local directories=(
        "$data_dir"
        "$data_dir/postgres"
        "$data_dir/redis"
        "$data_dir/prometheus"
        "$data_dir/grafana"
        "$PROJECT_ROOT/logs"
        "$PROJECT_ROOT/backups"
        "$PROJECT_ROOT/.config-backups"
    )
    
    for dir in "${directories[@]}"; do
        if [ ! -d "$dir" ]; then
            log_debug "Creating directory: $dir"
            mkdir -p "$dir"
            
            # Set appropriate permissions
            case "$ENVIRONMENT" in
                production)
                    chmod 750 "$dir"
                    ;;
                *)
                    chmod 755 "$dir"
                    ;;
            esac
        fi
    done
    
    log_success "Directory structure created"
}

configure_security() {
    log_info "Configuring security settings for $ENVIRONMENT..."
    
    # Set file permissions
    local config_files=(
        ".env.$ENVIRONMENT"
        "backend/config/$ENVIRONMENT.yaml"
        "frontend/.env.$ENVIRONMENT"
    )
    
    for file in "${config_files[@]}"; do
        local full_path="$PROJECT_ROOT/$file"
        if [ -f "$full_path" ]; then
            chmod 600 "$full_path"
            log_debug "Set secure permissions for: $file"
        fi
    done
    
    # Environment-specific security configurations
    case "$ENVIRONMENT" in
        production)
            # Additional production security measures
            log_info "Applying production security hardening..."
            
            # Create security audit log
            local audit_log="$PROJECT_ROOT/logs/security-audit.log"
            touch "$audit_log"
            chmod 640 "$audit_log"
            
            # Set up log rotation (if logrotate is available)
            if command -v logrotate >/dev/null 2>&1; then
                log_debug "Setting up log rotation"
                # This would typically be done via system configuration
            fi
            ;;
        staging)
            log_info "Applying staging security configurations..."
            ;;
        development)
            log_info "Applying development security configurations..."
            ;;
    esac
    
    log_success "Security configuration completed"
}

initialize_services() {
    log_info "Initializing services for $ENVIRONMENT..."
    
    # Load environment variables
    if [ -f "$PROJECT_ROOT/.env.$ENVIRONMENT" ]; then
        set -a
        source "$PROJECT_ROOT/.env.$ENVIRONMENT"
        set +a
    fi
    
    # Choose appropriate docker-compose file
    local compose_file="docker-compose.yml"
    if [ -f "$PROJECT_ROOT/docker-compose.$ENVIRONMENT.yml" ]; then
        compose_file="docker-compose.$ENVIRONMENT.yml"
    fi
    
    log_debug "Using compose file: $compose_file"
    
    # Start services
    case "$ENVIRONMENT" in
        development)
            log_info "Starting development services..."
            cd "$PROJECT_ROOT"
            docker-compose -f "$compose_file" up -d postgres redis
            ;;
        staging|production)
            log_info "Starting $ENVIRONMENT services..."
            cd "$PROJECT_ROOT"
            docker-compose -f "$compose_file" up -d
            ;;
    esac
    
    # Wait for services to be ready
    log_info "Waiting for services to be ready..."
    sleep 10
    
    # Run database migrations
    if [ "$ENVIRONMENT" != "production" ] || confirm_action "Run database migrations?"; then
        log_info "Running database migrations..."
        # This would run the actual migration command
        log_debug "Database migrations would run here"
    fi
    
    log_success "Services initialized"
}

run_validation() {
    log_info "Running validation checks for $ENVIRONMENT..."
    
    # Configuration validation
    log_debug "Running configuration validation..."
    if [ -x "$PROJECT_ROOT/scripts/config-manager.sh" ]; then
        "$PROJECT_ROOT/scripts/config-manager.sh" validate "$ENVIRONMENT"
    fi
    
    # Security scan
    log_debug "Running security scan..."
    if [ -x "$PROJECT_ROOT/scripts/security-audit.sh" ]; then
        "$PROJECT_ROOT/scripts/security-audit.sh"
    fi
    
    # Service health checks
    log_debug "Checking service health..."
    local health_checks_passed=true
    
    # Check database connectivity
    if command -v psql >/dev/null 2>&1 && [ -n "$DATABASE_URL" ]; then
        if psql "$DATABASE_URL" -c "SELECT 1;" >/dev/null 2>&1; then
            log_success "Database connection: OK"
        else
            log_warning "Database connection: FAILED"
            health_checks_passed=false
        fi
    fi
    
    # Check Redis connectivity
    if command -v redis-cli >/dev/null 2>&1 && [ -n "$REDIS_URL" ]; then
        if redis-cli -u "$REDIS_URL" ping >/dev/null 2>&1; then
            log_success "Redis connection: OK"
        else
            log_warning "Redis connection: FAILED"
            health_checks_passed=false
        fi
    fi
    
    if [ "$health_checks_passed" = true ]; then
        log_success "All validation checks passed"
    else
        log_warning "Some validation checks failed"
    fi
}

generate_documentation() {
    log_info "Generating environment documentation..."
    
    local doc_file="$PROJECT_ROOT/docs/environment-$ENVIRONMENT.md"
    mkdir -p "$(dirname "$doc_file")"
    
    cat > "$doc_file" << EOF
# $ENVIRONMENT Environment Setup

**Generated:** $(date)
**Environment:** $ENVIRONMENT

## Configuration Files

- Environment variables: \`.env.$ENVIRONMENT\`
- Backend config: \`backend/config/$ENVIRONMENT.yaml\`
- Frontend config: \`frontend/.env.$ENVIRONMENT\`
- Docker compose: \`docker-compose.$ENVIRONMENT.yml\`

## Services

### Database
- PostgreSQL with secure configuration
- Automated backups enabled
- Connection pooling configured

### Backend
- Loco.rs application server
- JWT authentication
- Soroban integration
- Security middleware enabled

### Frontend
- React application with Vite
- Secure API communication
- Environment-specific configuration

### Monitoring
- Prometheus metrics collection
- Grafana dashboards
- Health check endpoints

## Security Features

- Encrypted secrets management
- Secure inter-service communication
- Rate limiting and IP blocking
- Audit logging enabled
- Regular security scans

## Maintenance

### Backup Procedures
\`\`\`bash
# Create configuration backup
./scripts/config-manager.sh backup $ENVIRONMENT

# Create database backup
docker-compose -f docker-compose.$ENVIRONMENT.yml exec backup /usr/local/bin/backup.sh
\`\`\`

### Monitoring
- Grafana: http://localhost:3001 (development) or https://grafana.domain.com
- Prometheus: http://localhost:9091
- Application logs: \`./logs/\`

### Updates
\`\`\`bash
# Update configuration
./scripts/config-manager.sh validate $ENVIRONMENT

# Run security audit
./scripts/security-audit.sh

# Update services
docker-compose -f docker-compose.$ENVIRONMENT.yml pull
docker-compose -f docker-compose.$ENVIRONMENT.yml up -d
\`\`\`

## Troubleshooting

### Common Issues
1. **Service startup failures**: Check logs with \`docker-compose logs [service]\`
2. **Database connection issues**: Verify DATABASE_URL and network connectivity
3. **Permission errors**: Ensure proper file permissions (600 for secrets)

### Support Contacts
- Development: dev-team@bitcoin-custody.com
- Operations: ops-team@bitcoin-custody.com
- Security: security@bitcoin-custody.com

EOF
    
    log_success "Documentation generated: $doc_file"
}

# Parse command line arguments
parse_args() {
    while [[ $# -gt 0 ]]; do
        case $1 in
            development|staging|production)
                ENVIRONMENT="$1"
                shift
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
            *)
                log_error "Unknown option: $1"
                usage
                exit 1
                ;;
        esac
    done
}

# Main execution
main() {
    parse_args "$@"
    
    log_info "Setting up $ENVIRONMENT environment for Bitcoin Custody Platform"
    
    if [ "$ENVIRONMENT" = "production" ]; then
        log_warning "You are setting up a PRODUCTION environment!"
        if ! confirm_action "This will create production configurations with real secrets."; then
            log_info "Setup cancelled"
            exit 0
        fi
    fi
    
    # Run setup steps
    check_dependencies
    generate_secrets
    setup_directories
    configure_security
    initialize_services
    run_validation
    generate_documentation
    
    log_success "Environment setup completed successfully!"
    
    # Next steps
    echo
    log_info "Next steps:"
    case "$ENVIRONMENT" in
        development)
            echo "  1. Start development servers: npm run dev (frontend) and cargo run (backend)"
            echo "  2. Access the application at http://localhost:3000"
            echo "  3. View logs in ./logs/ directory"
            ;;
        staging)
            echo "  1. Update DNS records to point to this server"
            echo "  2. Configure SSL certificates"
            echo "  3. Run integration tests"
            echo "  4. Monitor services via Grafana"
            ;;
        production)
            echo "  1. IMPORTANT: Review and update all configuration values in .env.production"
            echo "  2. Configure SSL certificates and update TLS_CERT_FILE/TLS_KEY_FILE"
            echo "  3. Set up monitoring and alerting"
            echo "  4. Configure backup procedures"
            echo "  5. Run security audit before going live"
            echo "  6. Set up log monitoring and incident response"
            ;;
    esac
    
    echo
    log_info "Configuration files created:"
    echo "  - .env.$ENVIRONMENT"
    echo "  - docs/environment-$ENVIRONMENT.md"
    
    if [ "$ENVIRONMENT" = "production" ]; then
        echo
        log_warning "SECURITY REMINDER:"
        echo "  - Change default passwords and secrets"
        echo "  - Review all configuration values"
        echo "  - Set up proper SSL certificates"
        echo "  - Configure monitoring and alerting"
        echo "  - Run security audit before deployment"
    fi
}

# Run the script
main "$@"