#!/bin/bash

# Integration Deployment Script
# This script deploys and initializes all integrated contracts in the correct order

set -e

# Configuration
NETWORK=${NETWORK:-"testnet"}
ADMIN_ADDRESS=${ADMIN_ADDRESS:-""}
STELLAR_RPC_URL=${STELLAR_RPC_URL:-"https://soroban-testnet.stellar.org"}

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
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

# Check prerequisites
check_prerequisites() {
    log_info "Checking prerequisites..."
    
    if ! command -v stellar &> /dev/null; then
        log_error "Stellar CLI not found. Please install it first."
        exit 1
    fi
    
    if [ -z "$ADMIN_ADDRESS" ]; then
        log_error "ADMIN_ADDRESS environment variable is required"
        exit 1
    fi
    
    log_info "Prerequisites check passed"
}

# Build all contracts
build_contracts() {
    log_info "Building all contracts..."
    
    # Build KYC Registry
    log_info "Building KYC Registry..."
    cd soroban/contracts/kyc_registry
    stellar contract build
    cd ../..
    
    # Build Reserve Manager
    log_info "Building Reserve Manager..."
    cd soroban/contracts/reserve_manager
    stellar contract build
    cd ../..
    
    # Build iSTSi Token
    log_info "Building iSTSi Token..."
    cd soroban/contracts/istsi_token
    stellar contract build
    cd ../..
    
    # Build Fungible Token
    log_info "Building Fungible Token..."
    cd soroban/contracts/fungible
    stellar contract build
    cd ../..
    
    # Build Integration Router (if exists)
    if [ -d "soroban/contracts/integration_router" ]; then
        log_info "Building Integration Router..."
        cd soroban/contracts/integration_router
        stellar contract build
        cd ../..
    fi
    
    log_info "All contracts built successfully"
}

# Deploy contracts in dependency order
deploy_contracts() {
    log_info "Deploying contracts..."
    
    # Deploy KYC Registry first (no dependencies)
    log_info "Deploying KYC Registry..."
    KYC_WASM_HASH=$(stellar contract install \
        --wasm soroban/contracts/kyc_registry/target/wasm32-unknown-unknown/release/kyc_registry.wasm \
        --network $NETWORK)
    
    KYC_CONTRACT_ID=$(stellar contract deploy \
        --wasm-hash $KYC_WASM_HASH \
        --network $NETWORK)
    
    log_info "KYC Registry deployed: $KYC_CONTRACT_ID"
    
    # Deploy Reserve Manager
    log_info "Deploying Reserve Manager..."
    RESERVE_WASM_HASH=$(stellar contract install \
        --wasm soroban/contracts/reserve_manager/target/wasm32-unknown-unknown/release/reserve_manager.wasm \
        --network $NETWORK)
    
    RESERVE_CONTRACT_ID=$(stellar contract deploy \
        --wasm-hash $RESERVE_WASM_HASH \
        --network $NETWORK)
    
    log_info "Reserve Manager deployed: $RESERVE_CONTRACT_ID"
    
    # Deploy Fungible Token
    log_info "Deploying Fungible Token..."
    FUNGIBLE_WASM_HASH=$(stellar contract install \
        --wasm soroban/contracts/fungible/target/wasm32-unknown-unknown/release/fungible.wasm \
        --network $NETWORK)
    
    FUNGIBLE_CONTRACT_ID=$(stellar contract deploy \
        --wasm-hash $FUNGIBLE_WASM_HASH \
        --network $NETWORK)
    
    log_info "Fungible Token deployed: $FUNGIBLE_CONTRACT_ID"
    
    # Deploy iSTSi Token
    log_info "Deploying iSTSi Token..."
    ISTSI_WASM_HASH=$(stellar contract install \
        --wasm contracts/istsi_token/target/wasm32-unknown-unknown/release/istsi_token.wasm \
        --network $NETWORK)
    
    ISTSI_CONTRACT_ID=$(stellar contract deploy \
        --wasm-hash $ISTSI_WASM_HASH \
        --network $NETWORK)
    
    log_info "iSTSi Token deployed: $ISTSI_CONTRACT_ID"
    
    # Deploy Integration Router if exists
    if [ -d "contracts/integration_router" ]; then
        log_info "Deploying Integration Router..."
        ROUTER_WASM_HASH=$(stellar contract install \
            --wasm contracts/integration_router/target/wasm32-unknown-unknown/release/integration_router.wasm \
            --network $NETWORK)
        
        ROUTER_CONTRACT_ID=$(stellar contract deploy \
            --wasm-hash $ROUTER_WASM_HASH \
            --network $NETWORK)
        
        log_info "Integration Router deployed: $ROUTER_CONTRACT_ID"
    fi
    
    # Save contract addresses
    save_contract_addresses
}

# Save contract addresses to registry
save_contract_addresses() {
    log_info "Saving contract addresses to registry..."
    
    cat > deployment_addresses.json << EOF
{
  "network": "$NETWORK",
  "deployed_at": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "admin_address": "$ADMIN_ADDRESS",
  "contracts": {
    "kyc_registry": "$KYC_CONTRACT_ID",
    "reserve_manager": "$RESERVE_CONTRACT_ID",
    "fungible_token": "$FUNGIBLE_CONTRACT_ID",
    "istsi_token": "$ISTSI_CONTRACT_ID"$([ -n "$ROUTER_CONTRACT_ID" ] && echo ",
    \"integration_router\": \"$ROUTER_CONTRACT_ID\"")
  }
}
EOF
    
    log_info "Contract addresses saved to deployment_addresses.json"
}

# Initialize contracts with proper configuration
initialize_contracts() {
    log_info "Initializing contracts..."
    
    # Initialize KYC Registry
    log_info "Initializing KYC Registry..."
    stellar contract invoke \
        --id $KYC_CONTRACT_ID \
        --network $NETWORK \
        -- initialize \
        --admin $ADMIN_ADDRESS
    
    # Initialize Reserve Manager
    log_info "Initializing Reserve Manager..."
    stellar contract invoke \
        --id $RESERVE_CONTRACT_ID \
        --network $NETWORK \
        -- initialize \
        --admin $ADMIN_ADDRESS \
        --kyc_registry $KYC_CONTRACT_ID
    
    # Initialize Fungible Token
    log_info "Initializing Fungible Token..."
    stellar contract invoke \
        --id $FUNGIBLE_CONTRACT_ID \
        --network $NETWORK \
        -- initialize \
        --admin $ADMIN_ADDRESS \
        --name "Test Fungible Token" \
        --symbol "TFT" \
        --decimals 7
    
    # Initialize iSTSi Token
    log_info "Initializing iSTSi Token..."
    stellar contract invoke \
        --id $ISTSI_CONTRACT_ID \
        --network $NETWORK \
        -- initialize \
        --admin $ADMIN_ADDRESS \
        --kyc_registry $KYC_CONTRACT_ID \
        --reserve_manager $RESERVE_CONTRACT_ID
    
    # Initialize Integration Router if exists
    if [ -n "$ROUTER_CONTRACT_ID" ]; then
        log_info "Initializing Integration Router..."
        stellar contract invoke \
            --id $ROUTER_CONTRACT_ID \
            --network $NETWORK \
            -- initialize \
            --admin $ADMIN_ADDRESS \
            --kyc_registry $KYC_CONTRACT_ID \
            --reserve_manager $RESERVE_CONTRACT_ID \
            --istsi_token $ISTSI_CONTRACT_ID \
            --fungible_token $FUNGIBLE_CONTRACT_ID
    fi
    
    log_info "All contracts initialized successfully"
}

# Verify deployment
verify_deployment() {
    log_info "Verifying deployment..."
    
    # Check KYC Registry
    KYC_ADMIN=$(stellar contract invoke \
        --id $KYC_CONTRACT_ID \
        --network $NETWORK \
        -- get_admin)
    
    if [ "$KYC_ADMIN" != "$ADMIN_ADDRESS" ]; then
        log_error "KYC Registry admin verification failed"
        exit 1
    fi
    
    # Check Reserve Manager
    RESERVE_ADMIN=$(stellar contract invoke \
        --id $RESERVE_CONTRACT_ID \
        --network $NETWORK \
        -- get_admin)
    
    if [ "$RESERVE_ADMIN" != "$ADMIN_ADDRESS" ]; then
        log_error "Reserve Manager admin verification failed"
        exit 1
    fi
    
    # Check iSTSi Token
    ISTSI_ADMIN=$(stellar contract invoke \
        --id $ISTSI_CONTRACT_ID \
        --network $NETWORK \
        -- get_admin)
    
    if [ "$ISTSI_ADMIN" != "$ADMIN_ADDRESS" ]; then
        log_error "iSTSi Token admin verification failed"
        exit 1
    fi
    
    log_info "Deployment verification passed"
}

# Main deployment flow
main() {
    log_info "Starting integration deployment for network: $NETWORK"
    
    check_prerequisites
    build_contracts
    deploy_contracts
    initialize_contracts
    verify_deployment
    
    log_info "Integration deployment completed successfully!"
    log_info "Contract addresses saved in deployment_addresses.json"
    log_info "Run 'scripts/test_deployment.sh' to verify the deployment"
}

# Run main function
main "$@"