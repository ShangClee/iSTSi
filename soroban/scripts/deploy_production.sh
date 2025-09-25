#!/bin/bash

# Production Integration Deployment Script
# This script includes additional safety checks and confirmation steps for production deployment

set -e

# Configuration
NETWORK="mainnet"
ADMIN_ADDRESS=${ADMIN_ADDRESS:-""}
STELLAR_RPC_URL=${STELLAR_RPC_URL:-"https://horizon.stellar.org"}
BACKUP_DIR="deployment_backups/$(date +%Y%m%d_%H%M%S)"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging functions
log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

log_step() {
    echo -e "${BLUE}[STEP]${NC} $1"
}

# Confirmation prompt
confirm() {
    read -p "$(echo -e ${YELLOW}$1${NC}) [y/N]: " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        log_info "Operation cancelled by user"
        exit 0
    fi
}

# Pre-deployment checks
pre_deployment_checks() {
    log_step "Running pre-deployment checks..."
    
    # Check if this is really production
    log_warn "You are about to deploy to PRODUCTION (mainnet)"
    confirm "Are you absolutely sure you want to continue?"
    
    # Check prerequisites
    if ! command -v stellar &> /dev/null; then
        log_error "Stellar CLI not found. Please install it first."
        exit 1
    fi
    
    if [ -z "$ADMIN_ADDRESS" ]; then
        log_error "ADMIN_ADDRESS environment variable is required"
        exit 1
    fi
    
    # Check admin account has sufficient funds
    log_info "Checking admin account balance..."
    BALANCE=$(stellar account balance --address $ADMIN_ADDRESS --network $NETWORK || echo "0")
    if [ "$BALANCE" = "0" ]; then
        log_error "Admin account has insufficient balance or doesn't exist"
        exit 1
    fi
    
    # Verify all contract builds are up to date
    log_info "Verifying contract builds..."
    if [ ! -f "contracts/kyc_registry/target/wasm32-unknown-unknown/release/kyc_registry.wasm" ]; then
        log_error "KYC Registry WASM not found. Run 'stellar contract build' first."
        exit 1
    fi
    
    # Check for existing deployment
    if [ -f "production_addresses.json" ]; then
        log_warn "Existing production deployment found!"
        confirm "Do you want to overwrite the existing deployment?"
        
        # Backup existing deployment
        mkdir -p $BACKUP_DIR
        cp production_addresses.json $BACKUP_DIR/
        log_info "Existing deployment backed up to $BACKUP_DIR"
    fi
    
    log_info "Pre-deployment checks passed"
}

# Create deployment backup
create_backup() {
    log_step "Creating deployment backup..."
    
    mkdir -p $BACKUP_DIR
    
    # Backup contract source code
    tar -czf $BACKUP_DIR/contracts_source.tar.gz contracts/
    
    # Backup deployment scripts
    cp scripts/deploy_*.sh $BACKUP_DIR/
    
    # Backup any existing configuration
    if [ -f "deployment_addresses.json" ]; then
        cp deployment_addresses.json $BACKUP_DIR/
    fi
    
    log_info "Backup created in $BACKUP_DIR"
}

# Deploy with production settings
deploy_production() {
    log_step "Deploying contracts to production..."
    
    # Set production environment
    export NETWORK="mainnet"
    export STELLAR_RPC_URL="https://horizon.stellar.org"
    
    # Run the main deployment script
    ./soroban/scripts/deploy_integration.sh
    
    # Rename output file for production
    if [ -f "deployment_addresses.json" ]; then
        mv deployment_addresses.json production_addresses.json
        log_info "Production addresses saved to production_addresses.json"
    fi
}

# Production-specific configuration
configure_production() {
    log_step "Applying production configuration..."
    
    # Load contract addresses
    if [ ! -f "production_addresses.json" ]; then
        log_error "Production addresses file not found"
        exit 1
    fi
    
    KYC_CONTRACT_ID=$(jq -r '.contracts.kyc_registry' production_addresses.json)
    RESERVE_CONTRACT_ID=$(jq -r '.contracts.reserve_manager' production_addresses.json)
    ISTSI_CONTRACT_ID=$(jq -r '.contracts.istsi_token' production_addresses.json)
    
    # Set production parameters
    log_info "Setting production KYC parameters..."
    stellar contract invoke \
        --id $KYC_CONTRACT_ID \
        --network $NETWORK \
        -- set_tier_limits \
        --tier 1 \
        --daily_limit 1000000000 \
        --monthly_limit 10000000000
    
    stellar contract invoke \
        --id $KYC_CONTRACT_ID \
        --network $NETWORK \
        -- set_tier_limits \
        --tier 2 \
        --daily_limit 10000000000 \
        --monthly_limit 100000000000
    
    # Set production reserve thresholds
    log_info "Setting production reserve thresholds..."
    stellar contract invoke \
        --id $RESERVE_CONTRACT_ID \
        --network $NETWORK \
        -- set_reserve_threshold \
        --threshold 9500  # 95% minimum reserve ratio
    
    log_info "Production configuration applied"
}

# Comprehensive production verification
verify_production() {
    log_step "Running production verification..."
    
    # Load contract addresses
    KYC_CONTRACT_ID=$(jq -r '.contracts.kyc_registry' production_addresses.json)
    RESERVE_CONTRACT_ID=$(jq -r '.contracts.reserve_manager' production_addresses.json)
    ISTSI_CONTRACT_ID=$(jq -r '.contracts.istsi_token' production_addresses.json)
    FUNGIBLE_CONTRACT_ID=$(jq -r '.contracts.fungible_token' production_addresses.json)
    
    # Verify contract initialization
    log_info "Verifying contract initialization..."
    
    # Check KYC Registry
    KYC_ADMIN=$(stellar contract invoke --id $KYC_CONTRACT_ID --network $NETWORK -- get_admin)
    if [ "$KYC_ADMIN" != "$ADMIN_ADDRESS" ]; then
        log_error "KYC Registry admin verification failed"
        exit 1
    fi
    
    # Check Reserve Manager
    RESERVE_ADMIN=$(stellar contract invoke --id $RESERVE_CONTRACT_ID --network $NETWORK -- get_admin)
    if [ "$RESERVE_ADMIN" != "$ADMIN_ADDRESS" ]; then
        log_error "Reserve Manager admin verification failed"
        exit 1
    fi
    
    # Check iSTSi Token
    ISTSI_ADMIN=$(stellar contract invoke --id $ISTSI_TOKEN_ID --network $NETWORK -- get_admin)
    if [ "$ISTSI_ADMIN" != "$ADMIN_ADDRESS" ]; then
        log_error "iSTSi Token admin verification failed"
        exit 1
    fi
    
    # Verify contract integrations
    log_info "Verifying contract integrations..."
    
    # Check if iSTSi token knows about KYC registry
    ISTSI_KYC=$(stellar contract invoke --id $ISTSI_CONTRACT_ID --network $NETWORK -- get_kyc_registry)
    if [ "$ISTSI_KYC" != "$KYC_CONTRACT_ID" ]; then
        log_error "iSTSi Token KYC registry integration verification failed"
        exit 1
    fi
    
    # Check if iSTSi token knows about reserve manager
    ISTSI_RESERVE=$(stellar contract invoke --id $ISTSI_CONTRACT_ID --network $NETWORK -- get_reserve_manager)
    if [ "$ISTSI_RESERVE" != "$RESERVE_CONTRACT_ID" ]; then
        log_error "iSTSi Token reserve manager integration verification failed"
        exit 1
    fi
    
    log_info "Production verification passed"
}

# Generate production deployment report
generate_report() {
    log_step "Generating deployment report..."
    
    REPORT_FILE="production_deployment_report_$(date +%Y%m%d_%H%M%S).md"
    
    cat > $REPORT_FILE << EOF
# Production Deployment Report

**Deployment Date:** $(date -u +%Y-%m-%dT%H:%M:%SZ)
**Network:** mainnet
**Admin Address:** $ADMIN_ADDRESS
**Backup Location:** $BACKUP_DIR

## Deployed Contracts

$(cat production_addresses.json | jq -r '.contracts | to_entries[] | "- **\(.key):** \(.value)"')

## Deployment Verification

- [x] All contracts deployed successfully
- [x] All contracts initialized with correct admin
- [x] Contract integrations verified
- [x] Production configuration applied
- [x] Backup created

## Next Steps

1. Update monitoring systems with new contract addresses
2. Configure alerting for production contracts
3. Update documentation with production addresses
4. Notify stakeholders of successful deployment

## Emergency Contacts

- Admin Address: $ADMIN_ADDRESS
- Backup Location: $BACKUP_DIR
- Deployment Script: scripts/deploy_production.sh

## Contract Functions Test

Run the following commands to test basic functionality:

\`\`\`bash
# Test KYC Registry
stellar contract invoke --id $(jq -r '.contracts.kyc_registry' production_addresses.json) --network mainnet -- get_admin

# Test Reserve Manager
stellar contract invoke --id $(jq -r '.contracts.reserve_manager' production_addresses.json) --network mainnet -- get_admin

# Test iSTSi Token
stellar contract invoke --id $(jq -r '.contracts.istsi_token' production_addresses.json) --network mainnet -- get_admin
\`\`\`
EOF
    
    log_info "Deployment report generated: $REPORT_FILE"
}

# Main production deployment flow
main() {
    log_info "Starting PRODUCTION deployment process..."
    
    pre_deployment_checks
    create_backup
    deploy_production
    configure_production
    verify_production
    generate_report
    
    log_info "🎉 Production deployment completed successfully!"
    log_info "📋 Deployment report: $(ls production_deployment_report_*.md | tail -1)"
    log_info "💾 Backup location: $BACKUP_DIR"
    log_info "📄 Contract addresses: production_addresses.json"
    
    log_warn "Remember to:"
    log_warn "1. Update monitoring systems"
    log_warn "2. Configure production alerting"
    log_warn "3. Update documentation"
    log_warn "4. Notify stakeholders"
}

# Run main function
main "$@"