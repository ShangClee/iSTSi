#!/bin/bash

# Testnet deployment script with comprehensive testing and validation

set -e

# Configuration
NETWORK="testnet"
STELLAR_RPC_URL=${STELLAR_RPC_URL:-"https://soroban-testnet.stellar.org"}
NETWORK_PASSPHRASE="Test SDF Network ; September 2015"
ADMIN_ADDRESS=${ADMIN_ADDRESS:-""}
CONFIG_DIR="config/testnet"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

log_info() {
    echo -e "${GREEN}[TESTNET]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[TESTNET]${NC} $1"
}

log_error() {
    echo -e "${RED}[TESTNET]${NC} $1"
}

log_step() {
    echo -e "${BLUE}[TESTNET]${NC} $1"
}

# Setup testnet environment
setup_testnet() {
    log_step "Setting up testnet environment..."
    
    # Create config directory
    mkdir -p "$CONFIG_DIR"
    
    # Check stellar CLI
    if ! command -v stellar &> /dev/null; then
        log_error "Stellar CLI not found. Please install it first."
        exit 1
    fi
    
    # Configure network
    stellar network add \
        --global testnet \
        --rpc-url "$STELLAR_RPC_URL" \
        --network-passphrase "$NETWORK_PASSPHRASE"
    
    # Check admin account
    if [ -z "$ADMIN_ADDRESS" ]; then
        log_error "ADMIN_ADDRESS environment variable is required"
        log_info "Generate a testnet account with: stellar keys generate --global admin --network testnet"
        exit 1
    fi
    
    # Fund admin account if needed
    log_info "Checking admin account balance..."
    if ! stellar account balance --address "$ADMIN_ADDRESS" --network testnet &>/dev/null; then
        log_info "Funding admin account from friendbot..."
        stellar account fund "$ADMIN_ADDRESS" --network testnet
    fi
    
    log_info "Testnet environment setup completed"
}

# Build contracts for testnet
build_for_testnet() {
    log_step "Building contracts for testnet..."
    
    # Ensure contracts are built
    ./scripts/build.sh
    
    log_info "Contracts built for testnet deployment"
}

# Deploy contracts with testnet configuration
deploy_testnet_contracts() {
    log_step "Deploying contracts to testnet..."
    
    # Deploy KYC Registry
    log_info "Deploying KYC Registry..."
    KYC_WASM_HASH=$(stellar contract install \
        --wasm contracts/kyc_registry/target/wasm32-unknown-unknown/release/kyc_registry.wasm \
        --network testnet \
        --source admin)
    
    KYC_CONTRACT_ID=$(stellar contract deploy \
        --wasm-hash "$KYC_WASM_HASH" \
        --network testnet \
        --source admin)
    
    log_info "KYC Registry deployed: $KYC_CONTRACT_ID"
    
    # Deploy Reserve Manager
    log_info "Deploying Reserve Manager..."
    RESERVE_WASM_HASH=$(stellar contract install \
        --wasm contracts/reserve_manager/target/wasm32-unknown-unknown/release/reserve_manager.wasm \
        --network testnet \
        --source admin)
    
    RESERVE_CONTRACT_ID=$(stellar contract deploy \
        --wasm-hash "$RESERVE_WASM_HASH" \
        --network testnet \
        --source admin)
    
    log_info "Reserve Manager deployed: $RESERVE_CONTRACT_ID"
    
    # Deploy Fungible Token
    log_info "Deploying Fungible Token..."
    FUNGIBLE_WASM_HASH=$(stellar contract install \
        --wasm contracts/fungible/target/wasm32-unknown-unknown/release/fungible.wasm \
        --network testnet \
        --source admin)
    
    FUNGIBLE_CONTRACT_ID=$(stellar contract deploy \
        --wasm-hash "$FUNGIBLE_WASM_HASH" \
        --network testnet \
        --source admin)
    
    log_info "Fungible Token deployed: $FUNGIBLE_CONTRACT_ID"
    
    # Deploy iSTSi Token
    log_info "Deploying iSTSi Token..."
    ISTSI_WASM_HASH=$(stellar contract install \
        --wasm contracts/istsi_token/target/wasm32-unknown-unknown/release/istsi_token.wasm \
        --network testnet \
        --source admin)
    
    ISTSI_CONTRACT_ID=$(stellar contract deploy \
        --wasm-hash "$ISTSI_WASM_HASH" \
        --network testnet \
        --source admin)
    
    log_info "iSTSi Token deployed: $ISTSI_CONTRACT_ID"
    
    # Deploy Integration Router if exists
    if [ -f "contracts/integration_router/target/wasm32-unknown-unknown/release/integration_router.wasm" ]; then
        log_info "Deploying Integration Router..."
        ROUTER_WASM_HASH=$(stellar contract install \
            --wasm contracts/integration_router/target/wasm32-unknown-unknown/release/integration_router.wasm \
            --network testnet \
            --source admin)
        
        ROUTER_CONTRACT_ID=$(stellar contract deploy \
            --wasm-hash "$ROUTER_WASM_HASH" \
            --network testnet \
            --source admin)
        
        log_info "Integration Router deployed: $ROUTER_CONTRACT_ID"
    fi
    
    # Save contract addresses
    save_testnet_addresses
}

# Save contract addresses for testnet
save_testnet_addresses() {
    log_info "Saving testnet contract addresses..."
    
    cat > "$CONFIG_DIR/addresses.json" << EOF
{
  "network": "testnet",
  "deployed_at": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "admin_address": "$ADMIN_ADDRESS",
  "rpc_url": "$STELLAR_RPC_URL",
  "network_passphrase": "$NETWORK_PASSPHRASE",
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
    
    # Also create a simple addresses file for easy sourcing
    cat > "$CONFIG_DIR/addresses.env" << EOF
# Testnet Contract Addresses
export KYC_REGISTRY_ID="$KYC_CONTRACT_ID"
export RESERVE_MANAGER_ID="$RESERVE_CONTRACT_ID"
export FUNGIBLE_TOKEN_ID="$FUNGIBLE_CONTRACT_ID"
export ISTSI_TOKEN_ID="$ISTSI_CONTRACT_ID"
$([ -n "$ROUTER_CONTRACT_ID" ] && echo "export INTEGRATION_ROUTER_ID=\"$ROUTER_CONTRACT_ID\"")
export ADMIN_ADDRESS="$ADMIN_ADDRESS"
export NETWORK="testnet"
EOF
    
    log_info "Testnet addresses saved to $CONFIG_DIR/"
}

# Initialize contracts with testnet configuration
initialize_testnet_contracts() {
    log_step "Initializing contracts for testnet..."
    
    # Load addresses
    source "$CONFIG_DIR/addresses.env"
    
    # Initialize KYC Registry
    log_info "Initializing KYC Registry..."
    stellar contract invoke \
        --id "$KYC_REGISTRY_ID" \
        --network testnet \
        --source admin \
        -- initialize \
        --admin "$ADMIN_ADDRESS"
    
    # Initialize Reserve Manager
    log_info "Initializing Reserve Manager..."
    stellar contract invoke \
        --id "$RESERVE_MANAGER_ID" \
        --network testnet \
        --source admin \
        -- initialize \
        --admin "$ADMIN_ADDRESS" \
        --kyc_registry "$KYC_REGISTRY_ID"
    
    # Initialize Fungible Token
    log_info "Initializing Fungible Token..."
    stellar contract invoke \
        --id "$FUNGIBLE_TOKEN_ID" \
        --network testnet \
        --source admin \
        -- initialize \
        --admin "$ADMIN_ADDRESS" \
        --name "Testnet Fungible Token" \
        --symbol "TFT" \
        --decimals 7
    
    # Initialize iSTSi Token
    log_info "Initializing iSTSi Token..."
    stellar contract invoke \
        --id "$ISTSI_TOKEN_ID" \
        --network testnet \
        --source admin \
        -- initialize \
        --admin "$ADMIN_ADDRESS" \
        --kyc_registry "$KYC_REGISTRY_ID" \
        --reserve_manager "$RESERVE_MANAGER_ID"
    
    # Initialize Integration Router if exists
    if [ -n "$INTEGRATION_ROUTER_ID" ]; then
        log_info "Initializing Integration Router..."
        stellar contract invoke \
            --id "$INTEGRATION_ROUTER_ID" \
            --network testnet \
            --source admin \
            -- initialize \
            --admin "$ADMIN_ADDRESS" \
            --kyc_registry "$KYC_REGISTRY_ID" \
            --reserve_manager "$RESERVE_MANAGER_ID" \
            --istsi_token "$ISTSI_TOKEN_ID" \
            --fungible_token "$FUNGIBLE_TOKEN_ID"
    fi
    
    log_info "All contracts initialized for testnet"
}

# Run testnet-specific tests
test_testnet_deployment() {
    log_step "Testing testnet deployment..."
    
    source "$CONFIG_DIR/addresses.env"
    
    # Test KYC Registry
    log_info "Testing KYC Registry..."
    KYC_ADMIN=$(stellar contract invoke \
        --id "$KYC_REGISTRY_ID" \
        --network testnet \
        --source admin \
        -- get_admin)
    
    if [ "$KYC_ADMIN" != "$ADMIN_ADDRESS" ]; then
        log_error "KYC Registry admin test failed"
        exit 1
    fi
    
    # Test Reserve Manager
    log_info "Testing Reserve Manager..."
    RESERVE_ADMIN=$(stellar contract invoke \
        --id "$RESERVE_MANAGER_ID" \
        --network testnet \
        --source admin \
        -- get_admin)
    
    if [ "$RESERVE_ADMIN" != "$ADMIN_ADDRESS" ]; then
        log_error "Reserve Manager admin test failed"
        exit 1
    fi
    
    # Test iSTSi Token
    log_info "Testing iSTSi Token..."
    ISTSI_ADMIN=$(stellar contract invoke \
        --id "$ISTSI_TOKEN_ID" \
        --network testnet \
        --source admin \
        -- get_admin)
    
    if [ "$ISTSI_ADMIN" != "$ADMIN_ADDRESS" ]; then
        log_error "iSTSi Token admin test failed"
        exit 1
    fi
    
    # Test contract integrations
    log_info "Testing contract integrations..."
    ISTSI_KYC=$(stellar contract invoke \
        --id "$ISTSI_TOKEN_ID" \
        --network testnet \
        --source admin \
        -- get_kyc_registry)
    
    if [ "$ISTSI_KYC" != "$KYC_REGISTRY_ID" ]; then
        log_error "iSTSi Token KYC integration test failed"
        exit 1
    fi
    
    log_info "All testnet tests passed"
}

# Generate testnet deployment report
generate_testnet_report() {
    log_step "Generating testnet deployment report..."
    
    local report_file="$CONFIG_DIR/deployment_report_$(date +%Y%m%d_%H%M%S).md"
    
    cat > "$report_file" << EOF
# Testnet Deployment Report

**Deployment Date:** $(date -u +%Y-%m-%dT%H:%M:%SZ)
**Network:** testnet
**RPC URL:** $STELLAR_RPC_URL
**Admin Address:** $ADMIN_ADDRESS

## Deployed Contracts

$(cat "$CONFIG_DIR/addresses.json" | jq -r '.contracts | to_entries[] | "- **\(.key):** \(.value)"')

## WASM Hashes

$(cat "$CONFIG_DIR/addresses.json" | jq -r '.wasm_hashes | to_entries[] | "- **\(.key):** \(.value)"')

## Deployment Status

- [x] All contracts deployed successfully
- [x] All contracts initialized
- [x] Contract integrations verified
- [x] Basic functionality tested

## Usage Examples

\`\`\`bash
# Source contract addresses
source $CONFIG_DIR/addresses.env

# Test KYC Registry
stellar contract invoke --id \$KYC_REGISTRY_ID --network testnet --source admin -- get_admin

# Test Reserve Manager
stellar contract invoke --id \$RESERVE_MANAGER_ID --network testnet --source admin -- get_admin

# Test iSTSi Token
stellar contract invoke --id \$ISTSI_TOKEN_ID --network testnet --source admin -- get_admin
\`\`\`

## Next Steps

1. Run integration tests: \`./scripts/test-integration.sh --network testnet\`
2. Test frontend integration with testnet contracts
3. Validate end-to-end workflows
4. Monitor contract performance and gas usage

## Troubleshooting

- Contract addresses: $CONFIG_DIR/addresses.json
- Environment variables: $CONFIG_DIR/addresses.env
- Deployment logs: Check console output above

EOF
    
    log_info "Testnet deployment report: $report_file"
}

# Main testnet deployment function
main() {
    log_info "Starting testnet deployment..."
    
    setup_testnet
    build_for_testnet
    deploy_testnet_contracts
    initialize_testnet_contracts
    test_testnet_deployment
    generate_testnet_report
    
    log_info "🎉 Testnet deployment completed successfully!"
    log_info "📋 Configuration: $CONFIG_DIR/"
    log_info "📄 Addresses: $CONFIG_DIR/addresses.json"
    log_info "🔧 Environment: source $CONFIG_DIR/addresses.env"
    
    log_warn "Next steps:"
    log_warn "1. Run integration tests"
    log_warn "2. Test with frontend"
    log_warn "3. Validate workflows"
}

main "$@"