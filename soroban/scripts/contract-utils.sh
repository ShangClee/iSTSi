#!/bin/bash

# Contract interaction utilities for development and testing

set -e

# Configuration
NETWORK=${NETWORK:-"testnet"}
CONFIG_DIR="config/$NETWORK"
CONTRACTS_CONFIG="$CONFIG_DIR/addresses.env"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

log_info() {
    echo -e "${GREEN}[UTILS]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[UTILS]${NC} $1"
}

log_error() {
    echo -e "${RED}[UTILS]${NC} $1"
}

log_step() {
    echo -e "${BLUE}[UTILS]${NC} $1"
}

# Load contract addresses
load_contracts() {
    if [ ! -f "$CONTRACTS_CONFIG" ]; then
        log_error "Contract configuration not found: $CONTRACTS_CONFIG"
        log_info "Run deployment script first: ./scripts/deploy-$NETWORK.sh"
        exit 1
    fi
    
    source "$CONTRACTS_CONFIG"
    log_info "Loaded contracts for $NETWORK network"
}

# Show contract information
show_contracts() {
    log_step "Contract Information for $NETWORK"
    echo
    
    if [ -f "$CONFIG_DIR/addresses.json" ]; then
        echo "Contract Addresses:"
        jq -r '.contracts | to_entries[] | "  \(.key): \(.value)"' "$CONFIG_DIR/addresses.json"
        echo
        
        echo "WASM Hashes:"
        jq -r '.wasm_hashes | to_entries[] | "  \(.key): \(.value)"' "$CONFIG_DIR/addresses.json"
        echo
        
        echo "Deployment Info:"
        echo "  Network: $(jq -r '.network' "$CONFIG_DIR/addresses.json")"
        echo "  Deployed: $(jq -r '.deployed_at' "$CONFIG_DIR/addresses.json")"
        echo "  Admin: $(jq -r '.admin_address' "$CONFIG_DIR/addresses.json")"
    else
        log_error "No deployment found for $NETWORK"
    fi
}

# Test contract connectivity
test_connectivity() {
    log_step "Testing contract connectivity..."
    
    load_contracts
    
    local errors=0
    
    # Test KYC Registry
    if [ -n "$KYC_REGISTRY_ID" ]; then
        log_info "Testing KYC Registry..."
        if stellar contract invoke --id "$KYC_REGISTRY_ID" --network "$NETWORK" --source admin -- get_admin &>/dev/null; then
            echo "  ✅ KYC Registry: Connected"
        else
            echo "  ❌ KYC Registry: Failed"
            errors=$((errors + 1))
        fi
    fi
    
    # Test Reserve Manager
    if [ -n "$RESERVE_MANAGER_ID" ]; then
        log_info "Testing Reserve Manager..."
        if stellar contract invoke --id "$RESERVE_MANAGER_ID" --network "$NETWORK" --source admin -- get_admin &>/dev/null; then
            echo "  ✅ Reserve Manager: Connected"
        else
            echo "  ❌ Reserve Manager: Failed"
            errors=$((errors + 1))
        fi
    fi
    
    # Test iSTSi Token
    if [ -n "$ISTSI_TOKEN_ID" ]; then
        log_info "Testing iSTSi Token..."
        if stellar contract invoke --id "$ISTSI_TOKEN_ID" --network "$NETWORK" --source admin -- get_admin &>/dev/null; then
            echo "  ✅ iSTSi Token: Connected"
        else
            echo "  ❌ iSTSi Token: Failed"
            errors=$((errors + 1))
        fi
    fi
    
    # Test Fungible Token
    if [ -n "$FUNGIBLE_TOKEN_ID" ]; then
        log_info "Testing Fungible Token..."
        if stellar contract invoke --id "$FUNGIBLE_TOKEN_ID" --network "$NETWORK" --source admin -- get_admin &>/dev/null; then
            echo "  ✅ Fungible Token: Connected"
        else
            echo "  ❌ Fungible Token: Failed"
            errors=$((errors + 1))
        fi
    fi
    
    if [ $errors -eq 0 ]; then
        log_info "All contracts are accessible"
    else
        log_error "$errors contract(s) failed connectivity test"
        exit 1
    fi
}

# Get contract status
get_status() {
    log_step "Getting contract status..."
    
    load_contracts
    
    echo "=== KYC Registry Status ==="
    if [ -n "$KYC_REGISTRY_ID" ]; then
        echo "Admin: $(stellar contract invoke --id "$KYC_REGISTRY_ID" --network "$NETWORK" --source admin -- get_admin 2>/dev/null || echo 'ERROR')"
        echo "Registry Enabled: $(stellar contract invoke --id "$KYC_REGISTRY_ID" --network "$NETWORK" --source admin -- is_registry_enabled 2>/dev/null || echo 'ERROR')"
    fi
    echo
    
    echo "=== Reserve Manager Status ==="
    if [ -n "$RESERVE_MANAGER_ID" ]; then
        echo "Admin: $(stellar contract invoke --id "$RESERVE_MANAGER_ID" --network "$NETWORK" --source admin -- get_admin 2>/dev/null || echo 'ERROR')"
        echo "Reserve Threshold: $(stellar contract invoke --id "$RESERVE_MANAGER_ID" --network "$NETWORK" --source admin -- get_reserve_threshold 2>/dev/null || echo 'ERROR')"
        echo "Total Reserves: $(stellar contract invoke --id "$RESERVE_MANAGER_ID" --network "$NETWORK" --source admin -- get_total_reserves 2>/dev/null || echo 'ERROR')"
    fi
    echo
    
    echo "=== iSTSi Token Status ==="
    if [ -n "$ISTSI_TOKEN_ID" ]; then
        echo "Admin: $(stellar contract invoke --id "$ISTSI_TOKEN_ID" --network "$NETWORK" --source admin -- get_admin 2>/dev/null || echo 'ERROR')"
        echo "Name: $(stellar contract invoke --id "$ISTSI_TOKEN_ID" --network "$NETWORK" --source admin -- name 2>/dev/null || echo 'ERROR')"
        echo "Symbol: $(stellar contract invoke --id "$ISTSI_TOKEN_ID" --network "$NETWORK" --source admin -- symbol 2>/dev/null || echo 'ERROR')"
        echo "Total Supply: $(stellar contract invoke --id "$ISTSI_TOKEN_ID" --network "$NETWORK" --source admin -- total_supply 2>/dev/null || echo 'ERROR')"
    fi
    echo
}

# Register test user for KYC
register_test_user() {
    local user_address="$1"
    local tier="${2:-1}"
    
    if [ -z "$user_address" ]; then
        log_error "Usage: register_test_user <user_address> [tier]"
        return 1
    fi
    
    load_contracts
    
    log_info "Registering test user: $user_address (Tier $tier)"
    
    stellar contract invoke \
        --id "$KYC_REGISTRY_ID" \
        --network "$NETWORK" \
        --source admin \
        -- register_customer \
        --customer "$user_address" \
        --tier_code "$tier"
    
    log_info "User registered successfully"
}

# Simulate Bitcoin deposit
simulate_btc_deposit() {
    local user_address="$1"
    local btc_amount="${2:-100000000}"  # 1 BTC in satoshis
    local btc_tx_hash="${3:-$(openssl rand -hex 32)}"
    
    if [ -z "$user_address" ]; then
        log_error "Usage: simulate_btc_deposit <user_address> [btc_amount] [btc_tx_hash]"
        return 1
    fi
    
    load_contracts
    
    log_info "Simulating Bitcoin deposit..."
    log_info "User: $user_address"
    log_info "Amount: $btc_amount satoshis"
    log_info "TX Hash: $btc_tx_hash"
    
    # Register Bitcoin deposit
    stellar contract invoke \
        --id "$RESERVE_MANAGER_ID" \
        --network "$NETWORK" \
        --source admin \
        -- register_bitcoin_deposit \
        --user "$user_address" \
        --amount "$btc_amount" \
        --tx_hash "$btc_tx_hash"
    
    # Mint corresponding iSTSi tokens
    stellar contract invoke \
        --id "$ISTSI_TOKEN_ID" \
        --network "$NETWORK" \
        --source admin \
        -- mint \
        --to "$user_address" \
        --amount "$btc_amount"
    
    log_info "Bitcoin deposit simulation completed"
}

# Test token transfer
test_token_transfer() {
    local from_address="$1"
    local to_address="$2"
    local amount="${3:-1000000}"  # 0.01 tokens (7 decimals)
    
    if [ -z "$from_address" ] || [ -z "$to_address" ]; then
        log_error "Usage: test_token_transfer <from_address> <to_address> [amount]"
        return 1
    fi
    
    load_contracts
    
    log_info "Testing token transfer..."
    log_info "From: $from_address"
    log_info "To: $to_address"
    log_info "Amount: $amount"
    
    stellar contract invoke \
        --id "$ISTSI_TOKEN_ID" \
        --network "$NETWORK" \
        --source "$from_address" \
        -- transfer \
        --from "$from_address" \
        --to "$to_address" \
        --amount "$amount"
    
    log_info "Token transfer completed"
}

# Monitor contract events
monitor_events() {
    local contract_name="$1"
    local duration="${2:-60}"  # seconds
    
    if [ -z "$contract_name" ]; then
        log_error "Usage: monitor_events <contract_name> [duration_seconds]"
        log_info "Available contracts: kyc_registry, reserve_manager, istsi_token, fungible_token"
        return 1
    fi
    
    load_contracts
    
    # Get contract ID based on name
    local contract_id=""
    case "$contract_name" in
        "kyc_registry")
            contract_id="$KYC_REGISTRY_ID"
            ;;
        "reserve_manager")
            contract_id="$RESERVE_MANAGER_ID"
            ;;
        "istsi_token")
            contract_id="$ISTSI_TOKEN_ID"
            ;;
        "fungible_token")
            contract_id="$FUNGIBLE_TOKEN_ID"
            ;;
        *)
            log_error "Unknown contract: $contract_name"
            return 1
            ;;
    esac
    
    if [ -z "$contract_id" ]; then
        log_error "Contract ID not found for $contract_name"
        return 1
    fi
    
    log_info "Monitoring events for $contract_name ($contract_id) for ${duration}s..."
    
    timeout "$duration" stellar contract events \
        --id "$contract_id" \
        --network "$NETWORK" \
        --start-ledger recent \
        --count 100 \
        || log_info "Monitoring completed"
}

# Generate test data
generate_test_data() {
    log_step "Generating test data for $NETWORK..."
    
    load_contracts
    
    # Generate test users
    local test_users=()
    for i in {1..3}; do
        user_key="test_user_$i"
        if ! stellar keys show "$user_key" &>/dev/null; then
            stellar keys generate --global "$user_key" --network "$NETWORK"
        fi
        user_address=$(stellar keys address "$user_key")
        test_users+=("$user_address")
        
        # Fund user account
        if [ "$NETWORK" = "testnet" ]; then
            stellar account fund "$user_address" --network testnet
        fi
        
        # Register for KYC
        register_test_user "$user_address" "$i"
        
        # Simulate Bitcoin deposit
        simulate_btc_deposit "$user_address" "$((i * 50000000))"  # 0.5, 1.0, 1.5 BTC
    done
    
    log_info "Test data generated for ${#test_users[@]} users"
    
    # Test transfers between users
    if [ ${#test_users[@]} -ge 2 ]; then
        log_info "Testing transfers between users..."
        test_token_transfer "${test_users[0]}" "${test_users[1]}" "5000000"
        test_token_transfer "${test_users[1]}" "${test_users[2]}" "3000000"
    fi
    
    log_info "Test data generation completed"
}

# Cleanup test data
cleanup_test_data() {
    log_step "Cleaning up test data..."
    
    # Remove test user keys
    for i in {1..3}; do
        user_key="test_user_$i"
        if stellar keys show "$user_key" &>/dev/null; then
            stellar keys remove --global "$user_key"
            log_info "Removed test user: $user_key"
        fi
    done
    
    log_info "Test data cleanup completed"
}

# Show usage information
show_usage() {
    echo "Contract Utilities for Soroban Contracts"
    echo
    echo "Usage: $0 [COMMAND] [OPTIONS]"
    echo
    echo "Commands:"
    echo "  info                    Show contract information"
    echo "  test                    Test contract connectivity"
    echo "  status                  Get contract status"
    echo "  register <address> [tier]  Register test user for KYC"
    echo "  deposit <address> [amount] [tx_hash]  Simulate Bitcoin deposit"
    echo "  transfer <from> <to> [amount]  Test token transfer"
    echo "  monitor <contract> [duration]  Monitor contract events"
    echo "  generate-test-data      Generate test data"
    echo "  cleanup-test-data       Cleanup test data"
    echo "  help                    Show this help"
    echo
    echo "Environment Variables:"
    echo "  NETWORK                 Network to use (testnet/mainnet, default: testnet)"
    echo
    echo "Examples:"
    echo "  $0 info"
    echo "  $0 test"
    echo "  NETWORK=mainnet $0 status"
    echo "  $0 register GXXXXXXX 2"
    echo "  $0 deposit GXXXXXXX 100000000"
    echo "  $0 monitor istsi_token 120"
}

# Main function
main() {
    local command="$1"
    shift || true
    
    case "$command" in
        "info")
            show_contracts
            ;;
        "test")
            test_connectivity
            ;;
        "status")
            get_status
            ;;
        "register")
            register_test_user "$@"
            ;;
        "deposit")
            simulate_btc_deposit "$@"
            ;;
        "transfer")
            test_token_transfer "$@"
            ;;
        "monitor")
            monitor_events "$@"
            ;;
        "generate-test-data")
            generate_test_data
            ;;
        "cleanup-test-data")
            cleanup_test_data
            ;;
        "help"|"--help"|"-h"|"")
            show_usage
            ;;
        *)
            log_error "Unknown command: $command"
            show_usage
            exit 1
            ;;
    esac
}

main "$@"