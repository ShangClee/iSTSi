#!/bin/bash

# Mainnet deployment script with enhanced security and validation

set -e

# Configuration
NETWORK="mainnet"
STELLAR_RPC_URL=${STELLAR_RPC_URL:-"https://horizon.stellar.org"}
NETWORK_PASSPHRASE="Public Global Stellar Network ; September 2015"
ADMIN_ADDRESS=${ADMIN_ADDRESS:-""}
CONFIG_DIR="config/mainnet"
BACKUP_DIR="backups/mainnet_$(date +%Y%m%d_%H%M%S)"
MULTISIG_THRESHOLD=${MULTISIG_THRESHOLD:-2}

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
NC='\033[0m'

log_info() {
    echo -e "${GREEN}[MAINNET]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[MAINNET]${NC} $1"
}

log_error() {
    echo -e "${RED}[MAINNET]${NC} $1"
}

log_step() {
    echo -e "${BLUE}[MAINNET]${NC} $1"
}

log_security() {
    echo -e "${PURPLE}[SECURITY]${NC} $1"
}

# Security confirmation
security_confirmation() {
    log_security "MAINNET DEPLOYMENT SECURITY CHECKLIST"
    echo
    log_warn "⚠️  You are about to deploy to MAINNET (production)"
    log_warn "⚠️  This will deploy contracts with real value"
    log_warn "⚠️  Ensure all security reviews are complete"
    echo
    
    read -p "$(echo -e ${YELLOW}Have you completed security audit? [y/N]:${NC}) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        log_error "Security audit required before mainnet deployment"
        exit 1
    fi
    
    read -p "$(echo -e ${YELLOW}Have you tested on testnet? [y/N]:${NC}) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        log_error "Testnet testing required before mainnet deployment"
        exit 1
    fi
    
    read -p "$(echo -e ${YELLOW}Are you authorized for mainnet deployment? [y/N]:${NC}) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        log_error "Authorization required for mainnet deployment"
        exit 1
    fi
    
    read -p "$(echo -e ${RED}FINAL CONFIRMATION - Deploy to MAINNET? [y/N]:${NC}) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        log_info "Mainnet deployment cancelled"
        exit 0
    fi
    
    log_security "Security confirmation completed"
}

# Enhanced pre-deployment checks
enhanced_pre_checks() {
    log_step "Running enhanced pre-deployment checks..."
    
    # Check stellar CLI
    if ! command -v stellar &> /dev/null; then
        log_error "Stellar CLI not found"
        exit 1
    fi
    
    # Verify admin account
    if [ -z "$ADMIN_ADDRESS" ]; then
        log_error "ADMIN_ADDRESS required for mainnet deployment"
        exit 1
    fi
    
    # Check admin account balance (minimum 1000 XLM for deployment costs)
    log_info "Checking admin account balance..."
    BALANCE=$(stellar account balance --address "$ADMIN_ADDRESS" --network mainnet 2>/dev/null || echo "0")
    BALANCE_NUM=$(echo "$BALANCE" | grep -o '[0-9]*\.[0-9]*' | head -1)
    
    if (( $(echo "$BALANCE_NUM < 1000" | bc -l) )); then
        log_error "Insufficient balance for mainnet deployment (need 1000+ XLM, have $BALANCE_NUM)"
        exit 1
    fi
    
    # Verify contract builds
    log_info "Verifying contract builds..."
    local required_contracts=("kyc_registry" "reserve_manager" "istsi_token" "fungible")
    
    for contract in "${required_contracts[@]}"; do
        wasm_file="contracts/$contract/target/wasm32-unknown-unknown/release/${contract}.wasm"
        if [ ! -f "$wasm_file" ]; then
            log_error "Missing WASM file: $wasm_file"
            exit 1
        fi
        
        # Check WASM file size (should be reasonable for mainnet)
        size=$(stat -f%z "$wasm_file" 2>/dev/null || stat -c%s "$wasm_file" 2>/dev/null)
        if [ "$size" -gt 2097152 ]; then  # 2MB limit
            log_error "WASM file too large for mainnet: $contract ($size bytes)"
            exit 1
        fi
    done
    
    # Check for testnet deployment
    if [ ! -f "config/testnet/addresses.json" ]; then
        log_error "No testnet deployment found. Deploy to testnet first."
        exit 1
    fi
    
    # Verify git state
    if ! git diff-index --quiet HEAD --; then
        log_error "Uncommitted changes detected. Commit all changes before mainnet deployment."
        exit 1
    fi
    
    log_info "Enhanced pre-deployment checks passed"
}

# Create comprehensive backup
create_mainnet_backup() {
    log_step "Creating mainnet deployment backup..."
    
    mkdir -p "$BACKUP_DIR"
    
    # Backup source code
    git archive HEAD | tar -x -C "$BACKUP_DIR"
    
    # Backup existing mainnet config if it exists
    if [ -d "config/mainnet" ]; then
        cp -r config/mainnet "$BACKUP_DIR/"
    fi
    
    # Create deployment metadata
    cat > "$BACKUP_DIR/deployment_metadata.json" << EOF
{
  "backup_timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "git_commit": "$(git rev-parse HEAD)",
  "git_branch": "$(git branch --show-current)",
  "admin_address": "$ADMIN_ADDRESS",
  "deployer": "$(whoami)",
  "hostname": "$(hostname)"
}
EOF
    
    log_info "Backup created: $BACKUP_DIR"
}

# Setup mainnet environment with security
setup_mainnet_secure() {
    log_step "Setting up secure mainnet environment..."
    
    mkdir -p "$CONFIG_DIR"
    
    # Configure network
    stellar network add \
        --global mainnet \
        --rpc-url "$STELLAR_RPC_URL" \
        --network-passphrase "$NETWORK_PASSPHRASE"
    
    # Create secure deployment log
    DEPLOYMENT_LOG="$CONFIG_DIR/deployment_$(date +%Y%m%d_%H%M%S).log"
    exec 1> >(tee -a "$DEPLOYMENT_LOG")
    exec 2> >(tee -a "$DEPLOYMENT_LOG" >&2)
    
    log_info "Mainnet environment configured securely"
}

# Deploy with production-grade settings
deploy_mainnet_contracts() {
    log_step "Deploying contracts to mainnet..."
    
    # Deploy KYC Registry with production settings
    log_info "Deploying KYC Registry to mainnet..."
    KYC_WASM_HASH=$(stellar contract install \
        --wasm contracts/kyc_registry/target/wasm32-unknown-unknown/release/kyc_registry.wasm \
        --network mainnet \
        --source admin \
        --fee 10000)
    
    KYC_CONTRACT_ID=$(stellar contract deploy \
        --wasm-hash "$KYC_WASM_HASH" \
        --network mainnet \
        --source admin \
        --fee 10000)
    
    log_info "KYC Registry deployed: $KYC_CONTRACT_ID"
    
    # Deploy Reserve Manager
    log_info "Deploying Reserve Manager to mainnet..."
    RESERVE_WASM_HASH=$(stellar contract install \
        --wasm contracts/reserve_manager/target/wasm32-unknown-unknown/release/reserve_manager.wasm \
        --network mainnet \
        --source admin \
        --fee 10000)
    
    RESERVE_CONTRACT_ID=$(stellar contract deploy \
        --wasm-hash "$RESERVE_WASM_HASH" \
        --network mainnet \
        --source admin \
        --fee 10000)
    
    log_info "Reserve Manager deployed: $RESERVE_CONTRACT_ID"
    
    # Deploy Fungible Token
    log_info "Deploying Fungible Token to mainnet..."
    FUNGIBLE_WASM_HASH=$(stellar contract install \
        --wasm contracts/fungible/target/wasm32-unknown-unknown/release/fungible.wasm \
        --network mainnet \
        --source admin \
        --fee 10000)
    
    FUNGIBLE_CONTRACT_ID=$(stellar contract deploy \
        --wasm-hash "$FUNGIBLE_WASM_HASH" \
        --network mainnet \
        --source admin \
        --fee 10000)
    
    log_info "Fungible Token deployed: $FUNGIBLE_CONTRACT_ID"
    
    # Deploy iSTSi Token
    log_info "Deploying iSTSi Token to mainnet..."
    ISTSI_WASM_HASH=$(stellar contract install \
        --wasm contracts/istsi_token/target/wasm32-unknown-unknown/release/istsi_token.wasm \
        --network mainnet \
        --source admin \
        --fee 10000)
    
    ISTSI_CONTRACT_ID=$(stellar contract deploy \
        --wasm-hash "$ISTSI_WASM_HASH" \
        --network mainnet \
        --source admin \
        --fee 10000)
    
    log_info "iSTSi Token deployed: $ISTSI_CONTRACT_ID"
    
    # Deploy Integration Router if exists
    if [ -f "contracts/integration_router/target/wasm32-unknown-unknown/release/integration_router.wasm" ]; then
        log_info "Deploying Integration Router to mainnet..."
        ROUTER_WASM_HASH=$(stellar contract install \
            --wasm contracts/integration_router/target/wasm32-unknown-unknown/release/integration_router.wasm \
            --network mainnet \
            --source admin \
            --fee 10000)
        
        ROUTER_CONTRACT_ID=$(stellar contract deploy \
            --wasm-hash "$ROUTER_WASM_HASH" \
            --network mainnet \
            --source admin \
            --fee 10000)
        
        log_info "Integration Router deployed: $ROUTER_CONTRACT_ID"
    fi
    
    save_mainnet_addresses
}

# Save mainnet addresses with security
save_mainnet_addresses() {
    log_info "Saving mainnet contract addresses..."
    
    cat > "$CONFIG_DIR/addresses.json" << EOF
{
  "network": "mainnet",
  "deployed_at": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "admin_address": "$ADMIN_ADDRESS",
  "rpc_url": "$STELLAR_RPC_URL",
  "network_passphrase": "$NETWORK_PASSPHRASE",
  "git_commit": "$(git rev-parse HEAD)",
  "deployment_backup": "$BACKUP_DIR",
  "contracts": {
    "kyc_registry": "$KYC_CONTRACT_ID",
    "reserve_manager": "$RESERVE_CONTRACT_ID",
    "fungible_token": "$FUNGIBLE_CONTRACT_ID",
    "istsi_token": "$ISTSI_CONTRACT_ID"$([ -n "$ROUTER_CONTRACT_ID" ] && echo ",
    \"integration_router\": \"$ROUTER_CONTRACT_ID\"")
  },
  "wasm_hashes": {
    "kyc_registry": "$KYC_WASM_HASH",
    "reserve_manager": "$RESERVE_WASM_HASH",
    "fungible_token": "$FUNGIBLE_WASM_HASH",
    "istsi_token": "$ISTSI_WASM_HASH"$([ -n "$ROUTER_WASM_HASH" ] && echo ",
    \"integration_router\": \"$ROUTER_WASM_HASH\"")
  }
}
EOF
    
    # Create production environment file
    cat > "$CONFIG_DIR/addresses.env" << EOF
# Mainnet Contract Addresses - PRODUCTION
export KYC_REGISTRY_ID="$KYC_CONTRACT_ID"
export RESERVE_MANAGER_ID="$RESERVE_CONTRACT_ID"
export FUNGIBLE_TOKEN_ID="$FUNGIBLE_CONTRACT_ID"
export ISTSI_TOKEN_ID="$ISTSI_CONTRACT_ID"
$([ -n "$ROUTER_CONTRACT_ID" ] && echo "export INTEGRATION_ROUTER_ID=\"$ROUTER_CONTRACT_ID\"")
export ADMIN_ADDRESS="$ADMIN_ADDRESS"
export NETWORK="mainnet"
export STELLAR_RPC_URL="$STELLAR_RPC_URL"
EOF
    
    # Set secure permissions
    chmod 600 "$CONFIG_DIR/addresses.json"
    chmod 600 "$CONFIG_DIR/addresses.env"
    
    log_info "Mainnet addresses saved securely"
}

# Initialize with production parameters
initialize_mainnet_contracts() {
    log_step "Initializing contracts with production parameters..."
    
    source "$CONFIG_DIR/addresses.env"
    
    # Initialize KYC Registry with production settings
    log_info "Initializing KYC Registry..."
    stellar contract invoke \
        --id "$KYC_REGISTRY_ID" \
        --network mainnet \
        --source admin \
        --fee 10000 \
        -- initialize \
        --admin "$ADMIN_ADDRESS"
    
    # Set production KYC limits
    stellar contract invoke \
        --id "$KYC_REGISTRY_ID" \
        --network mainnet \
        --source admin \
        --fee 10000 \
        -- set_tier_limits \
        --tier 1 \
        --daily_limit 100000000 \
        --monthly_limit 1000000000
    
    # Initialize Reserve Manager
    log_info "Initializing Reserve Manager..."
    stellar contract invoke \
        --id "$RESERVE_MANAGER_ID" \
        --network mainnet \
        --source admin \
        --fee 10000 \
        -- initialize \
        --admin "$ADMIN_ADDRESS" \
        --kyc_registry "$KYC_REGISTRY_ID"
    
    # Set production reserve threshold (98%)
    stellar contract invoke \
        --id "$RESERVE_MANAGER_ID" \
        --network mainnet \
        --source admin \
        --fee 10000 \
        -- set_reserve_threshold \
        --threshold 9800
    
    # Initialize Fungible Token
    log_info "Initializing Fungible Token..."
    stellar contract invoke \
        --id "$FUNGIBLE_TOKEN_ID" \
        --network mainnet \
        --source admin \
        --fee 10000 \
        -- initialize \
        --admin "$ADMIN_ADDRESS" \
        --name "Bitcoin Custody Fungible Token" \
        --symbol "BCFT" \
        --decimals 7
    
    # Initialize iSTSi Token
    log_info "Initializing iSTSi Token..."
    stellar contract invoke \
        --id "$ISTSI_TOKEN_ID" \
        --network mainnet \
        --source admin \
        --fee 10000 \
        -- initialize \
        --admin "$ADMIN_ADDRESS" \
        --kyc_registry "$KYC_REGISTRY_ID" \
        --reserve_manager "$RESERVE_MANAGER_ID"
    
    # Initialize Integration Router if exists
    if [ -n "$INTEGRATION_ROUTER_ID" ]; then
        log_info "Initializing Integration Router..."
        stellar contract invoke \
            --id "$INTEGRATION_ROUTER_ID" \
            --network mainnet \
            --source admin \
            --fee 10000 \
            -- initialize \
            --admin "$ADMIN_ADDRESS" \
            --kyc_registry "$KYC_REGISTRY_ID" \
            --reserve_manager "$RESERVE_MANAGER_ID" \
            --istsi_token "$ISTSI_TOKEN_ID" \
            --fungible_token "$FUNGIBLE_TOKEN_ID"
    fi
    
    log_info "All contracts initialized with production parameters"
}

# Comprehensive mainnet validation
validate_mainnet_deployment() {
    log_step "Running comprehensive mainnet validation..."
    
    source "$CONFIG_DIR/addresses.env"
    
    local validation_errors=0
    
    # Test all contract admin functions
    log_info "Validating contract admins..."
    
    contracts=("KYC_REGISTRY_ID" "RESERVE_MANAGER_ID" "FUNGIBLE_TOKEN_ID" "ISTSI_TOKEN_ID")
    for contract_var in "${contracts[@]}"; do
        contract_id="${!contract_var}"
        if [ -n "$contract_id" ]; then
            admin=$(stellar contract invoke \
                --id "$contract_id" \
                --network mainnet \
                --source admin \
                -- get_admin 2>/dev/null || echo "ERROR")
            
            if [ "$admin" != "$ADMIN_ADDRESS" ]; then
                log_error "Admin validation failed for $contract_var"
                validation_errors=$((validation_errors + 1))
            fi
        fi
    done
    
    # Test contract integrations
    log_info "Validating contract integrations..."
    
    # Check iSTSi Token integrations
    istsi_kyc=$(stellar contract invoke \
        --id "$ISTSI_TOKEN_ID" \
        --network mainnet \
        --source admin \
        -- get_kyc_registry 2>/dev/null || echo "ERROR")
    
    if [ "$istsi_kyc" != "$KYC_REGISTRY_ID" ]; then
        log_error "iSTSi Token KYC integration validation failed"
        validation_errors=$((validation_errors + 1))
    fi
    
    # Test production parameters
    log_info "Validating production parameters..."
    
    # Check reserve threshold
    reserve_threshold=$(stellar contract invoke \
        --id "$RESERVE_MANAGER_ID" \
        --network mainnet \
        --source admin \
        -- get_reserve_threshold 2>/dev/null || echo "0")
    
    if [ "$reserve_threshold" != "9800" ]; then
        log_error "Reserve threshold validation failed (expected 9800, got $reserve_threshold)"
        validation_errors=$((validation_errors + 1))
    fi
    
    if [ $validation_errors -gt 0 ]; then
        log_error "Mainnet validation failed with $validation_errors errors"
        exit 1
    fi
    
    log_info "Mainnet validation passed successfully"
}

# Generate production deployment report
generate_mainnet_report() {
    log_step "Generating mainnet deployment report..."
    
    local report_file="$CONFIG_DIR/MAINNET_DEPLOYMENT_REPORT_$(date +%Y%m%d_%H%M%S).md"
    
    cat > "$report_file" << EOF
# 🚀 MAINNET DEPLOYMENT REPORT

**⚠️ PRODUCTION DEPLOYMENT - HANDLE WITH CARE ⚠️**

## Deployment Information

- **Network:** Mainnet (Production)
- **Deployment Date:** $(date -u +%Y-%m-%dT%H:%M:%SZ)
- **Admin Address:** $ADMIN_ADDRESS
- **Git Commit:** $(git rev-parse HEAD)
- **Deployer:** $(whoami)@$(hostname)
- **Backup Location:** $BACKUP_DIR

## Contract Addresses

$(cat "$CONFIG_DIR/addresses.json" | jq -r '.contracts | to_entries[] | "- **\(.key):** \(.value)"')

## WASM Hashes

$(cat "$CONFIG_DIR/addresses.json" | jq -r '.wasm_hashes | to_entries[] | "- **\(.key):** \(.value)"')

## Production Parameters

- **KYC Tier 1 Limits:** 1 BTC daily, 10 BTC monthly
- **Reserve Threshold:** 98% minimum
- **Token Decimals:** 7
- **Network Fees:** 10,000 stroops per operation

## Security Checklist

- [x] Security audit completed
- [x] Testnet deployment tested
- [x] Authorization confirmed
- [x] Source code backed up
- [x] All contracts deployed successfully
- [x] All contracts initialized
- [x] Production parameters set
- [x] Contract integrations verified
- [x] Comprehensive validation passed

## Emergency Procedures

### Contract Admin Functions
\`\`\`bash
# Source mainnet addresses
source $CONFIG_DIR/addresses.env

# Emergency pause (if supported)
stellar contract invoke --id \$ISTSI_TOKEN_ID --network mainnet --source admin -- pause

# Check contract status
stellar contract invoke --id \$KYC_REGISTRY_ID --network mainnet --source admin -- get_admin
\`\`\`

### Monitoring Commands
\`\`\`bash
# Monitor contract events
stellar contract events --id \$ISTSI_TOKEN_ID --network mainnet

# Check reserve status
stellar contract invoke --id \$RESERVE_MANAGER_ID --network mainnet --source admin -- get_reserve_status
\`\`\`

## Post-Deployment Actions Required

1. **Update Monitoring Systems**
   - Add contract addresses to monitoring dashboards
   - Configure alerting for critical events
   - Set up automated health checks

2. **Update Documentation**
   - Update API documentation with mainnet addresses
   - Notify integration partners of mainnet deployment
   - Update user-facing documentation

3. **Security Monitoring**
   - Enable 24/7 monitoring for all contracts
   - Set up automated security scanning
   - Configure incident response procedures

4. **Backup and Recovery**
   - Verify backup integrity: $BACKUP_DIR
   - Test recovery procedures
   - Document rollback processes

## Support Information

- **Configuration:** $CONFIG_DIR/
- **Deployment Log:** $DEPLOYMENT_LOG
- **Emergency Contacts:** [Add emergency contact information]
- **Incident Response:** [Add incident response procedures]

---

**⚠️ CRITICAL: This is a production deployment. All changes must follow change management procedures.**
EOF
    
    log_info "Mainnet deployment report: $report_file"
}

# Main mainnet deployment function
main() {
    log_security "🚨 MAINNET DEPLOYMENT INITIATED 🚨"
    
    security_confirmation
    enhanced_pre_checks
    create_mainnet_backup
    setup_mainnet_secure
    deploy_mainnet_contracts
    initialize_mainnet_contracts
    validate_mainnet_deployment
    generate_mainnet_report
    
    log_info "🎉 MAINNET DEPLOYMENT COMPLETED SUCCESSFULLY! 🎉"
    log_security "📋 Report: $(ls $CONFIG_DIR/MAINNET_DEPLOYMENT_REPORT_*.md | tail -1)"
    log_security "💾 Backup: $BACKUP_DIR"
    log_security "🔧 Config: $CONFIG_DIR/"
    
    log_warn "🚨 CRITICAL POST-DEPLOYMENT ACTIONS:"
    log_warn "1. Update monitoring systems immediately"
    log_warn "2. Configure production alerting"
    log_warn "3. Notify stakeholders of successful deployment"
    log_warn "4. Begin 24/7 monitoring procedures"
}

main "$@"