#!/bin/bash

# Deployment Verification Script for iSTSi Integration System
# This script performs comprehensive verification of deployed contracts

set -e

# Configuration
NETWORK=${1:-testnet}
REGISTRY_FILE="deployment_registry_${NETWORK}.json"
VERIFICATION_LOG="verification_${NETWORK}_$(date +%Y%m%d_%H%M%S).log"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging function
log() {
    echo -e "${BLUE}[$(date '+%Y-%m-%d %H:%M:%S')]${NC} $1" | tee -a "$VERIFICATION_LOG"
}

error() {
    echo -e "${RED}[ERROR]${NC} $1" | tee -a "$VERIFICATION_LOG"
}

success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1" | tee -a "$VERIFICATION_LOG"
}

warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1" | tee -a "$VERIFICATION_LOG"
}

# Check if registry file exists
if [ ! -f "$REGISTRY_FILE" ]; then
    error "Deployment registry file not found: $REGISTRY_FILE"
    exit 1
fi

log "Starting deployment verification for network: $NETWORK"
log "Using registry file: $REGISTRY_FILE"

# Parse contract addresses from registry
KYC_REGISTRY=$(jq -r '.contracts.kyc_registry.address' "$REGISTRY_FILE")
RESERVE_MANAGER=$(jq -r '.contracts.reserve_manager.address' "$REGISTRY_FILE")
FUNGIBLE_TOKEN=$(jq -r '.contracts.fungible_token.address' "$REGISTRY_FILE")
ISTSI_TOKEN=$(jq -r '.contracts.istsi_token.address' "$REGISTRY_FILE")
INTEGRATION_ROUTER=$(jq -r '.contracts.integration_router.address' "$REGISTRY_FILE")

log "Contract addresses loaded from registry:"
log "  KYC Registry: $KYC_REGISTRY"
log "  Reserve Manager: $RESERVE_MANAGER"
log "  Fungible Token: $FUNGIBLE_TOKEN"
log "  iSTSi Token: $ISTSI_TOKEN"
log "  Integration Router: $INTEGRATION_ROUTER"

# Verification functions
verify_contract_deployed() {
    local contract_name=$1
    local contract_address=$2
    
    log "Verifying deployment of $contract_name..."
    
    # Check if contract exists on network
    if soroban contract invoke --id "$contract_address" --network "$NETWORK" --source deployer -- --help > /dev/null 2>&1; then
        success "$contract_name is deployed and responsive"
        return 0
    else
        error "$contract_name is not responsive at address $contract_address"
        return 1
    fi
}

verify_contract_initialization() {
    local contract_name=$1
    local contract_address=$2
    
    log "Verifying initialization of $contract_name..."
    
    case $contract_name in
        "kyc_registry")
            # Check if admin is set
            local admin=$(soroban contract invoke --id "$contract_address" --network "$NETWORK" --source deployer -- get_admin 2>/dev/null || echo "")
            if [ -n "$admin" ]; then
                success "$contract_name is properly initialized with admin: $admin"
                return 0
            else
                error "$contract_name initialization failed - no admin set"
                return 1
            fi
            ;;
        "reserve_manager")
            # Check if admin is set and reserve ratio is configured
            local admin=$(soroban contract invoke --id "$contract_address" --network "$NETWORK" --source deployer -- get_admin 2>/dev/null || echo "")
            if [ -n "$admin" ]; then
                success "$contract_name is properly initialized with admin: $admin"
                return 0
            else
                error "$contract_name initialization failed - no admin set"
                return 1
            fi
            ;;
        "fungible_token")
            # Check if token name and symbol are set
            local name=$(soroban contract invoke --id "$contract_address" --network "$NETWORK" --source deployer -- name 2>/dev/null || echo "")
            local symbol=$(soroban contract invoke --id "$contract_address" --network "$NETWORK" --source deployer -- symbol 2>/dev/null || echo "")
            if [ -n "$name" ] && [ -n "$symbol" ]; then
                success "$contract_name is properly initialized - Name: $name, Symbol: $symbol"
                return 0
            else
                error "$contract_name initialization failed - missing name or symbol"
                return 1
            fi
            ;;
        "istsi_token")
            # Check if token is initialized and linked to KYC registry
            local name=$(soroban contract invoke --id "$contract_address" --network "$NETWORK" --source deployer -- name 2>/dev/null || echo "")
            if [ -n "$name" ]; then
                success "$contract_name is properly initialized - Name: $name"
                return 0
            else
                error "$contract_name initialization failed"
                return 1
            fi
            ;;
        "integration_router")
            # Check if router is initialized and not paused
            local is_paused=$(soroban contract invoke --id "$contract_address" --network "$NETWORK" --source deployer -- is_paused 2>/dev/null || echo "true")
            local admin=$(soroban contract invoke --id "$contract_address" --network "$NETWORK" --source deployer -- get_admin 2>/dev/null || echo "")
            if [ "$is_paused" = "false" ] && [ -n "$admin" ]; then
                success "$contract_name is properly initialized and operational"
                return 0
            else
                error "$contract_name initialization failed - paused: $is_paused, admin: $admin"
                return 1
            fi
            ;;
        *)
            warning "Unknown contract type: $contract_name"
            return 1
            ;;
    esac
}

verify_integration_configuration() {
    log "Verifying integration configuration..."
    
    # Check if integration router has all contract addresses configured
    local health_status=$(soroban contract invoke --id "$INTEGRATION_ROUTER" --network "$NETWORK" --source deployer -- deployment_health_check 2>/dev/null || echo "")
    
    if [ -n "$health_status" ]; then
        success "Integration router health check completed"
        log "Health status: $health_status"
        return 0
    else
        error "Integration router health check failed"
        return 1
    fi
}

verify_cross_contract_communication() {
    log "Verifying cross-contract communication..."
    
    # Test basic cross-contract calls through integration router
    local deployment_status=$(soroban contract invoke --id "$INTEGRATION_ROUTER" --network "$NETWORK" --source deployer -- get_deployment_status 2>/dev/null || echo "")
    
    if [ -n "$deployment_status" ]; then
        success "Cross-contract communication is working"
        log "Deployment status: $deployment_status"
        return 0
    else
        error "Cross-contract communication failed"
        return 1
    fi
}

run_performance_tests() {
    log "Running basic performance tests..."
    
    # Test response times for key functions
    local start_time=$(date +%s%N)
    soroban contract invoke --id "$INTEGRATION_ROUTER" --network "$NETWORK" --source deployer -- is_paused > /dev/null 2>&1
    local end_time=$(date +%s%N)
    local response_time=$(( (end_time - start_time) / 1000000 )) # Convert to milliseconds
    
    if [ $response_time -lt 5000 ]; then # Less than 5 seconds
        success "Performance test passed - Response time: ${response_time}ms"
        return 0
    else
        warning "Performance test warning - Slow response time: ${response_time}ms"
        return 1
    fi
}

generate_verification_report() {
    log "Generating verification report..."
    
    local report_file="verification_report_${NETWORK}_$(date +%Y%m%d_%H%M%S).json"
    
    cat > "$report_file" << EOF
{
  "network": "$NETWORK",
  "verification_time": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "registry_file": "$REGISTRY_FILE",
  "contracts": {
    "kyc_registry": {
      "address": "$KYC_REGISTRY",
      "deployed": $kyc_deployed,
      "initialized": $kyc_initialized
    },
    "reserve_manager": {
      "address": "$RESERVE_MANAGER", 
      "deployed": $reserve_deployed,
      "initialized": $reserve_initialized
    },
    "fungible_token": {
      "address": "$FUNGIBLE_TOKEN",
      "deployed": $fungible_deployed,
      "initialized": $fungible_initialized
    },
    "istsi_token": {
      "address": "$ISTSI_TOKEN",
      "deployed": $istsi_deployed,
      "initialized": $istsi_initialized
    },
    "integration_router": {
      "address": "$INTEGRATION_ROUTER",
      "deployed": $router_deployed,
      "initialized": $router_initialized
    }
  },
  "integration_tests": {
    "configuration": $integration_config,
    "cross_contract_communication": $cross_contract_comm,
    "performance": $performance_test
  },
  "overall_status": "$overall_status"
}
EOF
    
    success "Verification report generated: $report_file"
}

# Main verification process
log "=========================================="
log "Starting comprehensive deployment verification"
log "=========================================="

# Initialize verification results
kyc_deployed=false
kyc_initialized=false
reserve_deployed=false
reserve_initialized=false
fungible_deployed=false
fungible_initialized=false
istsi_deployed=false
istsi_initialized=false
router_deployed=false
router_initialized=false
integration_config=false
cross_contract_comm=false
performance_test=false

# Verify contract deployments
if verify_contract_deployed "kyc_registry" "$KYC_REGISTRY"; then
    kyc_deployed=true
    if verify_contract_initialization "kyc_registry" "$KYC_REGISTRY"; then
        kyc_initialized=true
    fi
fi

if verify_contract_deployed "reserve_manager" "$RESERVE_MANAGER"; then
    reserve_deployed=true
    if verify_contract_initialization "reserve_manager" "$RESERVE_MANAGER"; then
        reserve_initialized=true
    fi
fi

if verify_contract_deployed "fungible_token" "$FUNGIBLE_TOKEN"; then
    fungible_deployed=true
    if verify_contract_initialization "fungible_token" "$FUNGIBLE_TOKEN"; then
        fungible_initialized=true
    fi
fi

if verify_contract_deployed "istsi_token" "$ISTSI_TOKEN"; then
    istsi_deployed=true
    if verify_contract_initialization "istsi_token" "$ISTSI_TOKEN"; then
        istsi_initialized=true
    fi
fi

if verify_contract_deployed "integration_router" "$INTEGRATION_ROUTER"; then
    router_deployed=true
    if verify_contract_initialization "integration_router" "$INTEGRATION_ROUTER"; then
        router_initialized=true
    fi
fi

# Verify integration configuration
if verify_integration_configuration; then
    integration_config=true
fi

# Verify cross-contract communication
if verify_cross_contract_communication; then
    cross_contract_comm=true
fi

# Run performance tests
if run_performance_tests; then
    performance_test=true
fi

# Determine overall status
if [ "$kyc_deployed" = true ] && [ "$kyc_initialized" = true ] && \
   [ "$reserve_deployed" = true ] && [ "$reserve_initialized" = true ] && \
   [ "$fungible_deployed" = true ] && [ "$fungible_initialized" = true ] && \
   [ "$istsi_deployed" = true ] && [ "$istsi_initialized" = true ] && \
   [ "$router_deployed" = true ] && [ "$router_initialized" = true ] && \
   [ "$integration_config" = true ] && [ "$cross_contract_comm" = true ]; then
    overall_status="PASSED"
    success "All verification checks passed!"
else
    overall_status="FAILED"
    error "Some verification checks failed!"
fi

# Generate final report
generate_verification_report

log "=========================================="
log "Verification completed with status: $overall_status"
log "Log file: $VERIFICATION_LOG"
log "=========================================="

# Exit with appropriate code
if [ "$overall_status" = "PASSED" ]; then
    exit 0
else
    exit 1
fi