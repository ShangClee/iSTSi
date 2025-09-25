#!/bin/bash

# Contract Registry Update Script
# This script provides utilities for updating and managing the contract registry

set -e

# Configuration
REGISTRY_FILE=${REGISTRY_FILE:-"deployment_addresses.json"}
BACKUP_DIR="registry_backups"

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

# Show usage
show_usage() {
    echo "Usage: $0 <command> [options]"
    echo ""
    echo "Commands:"
    echo "  list                    List all registered contracts"
    echo "  get <name>             Get contract address by name"
    echo "  set <name> <address>   Set contract address"
    echo "  remove <name>          Remove contract from registry"
    echo "  validate               Validate all contract addresses"
    echo "  backup                 Create backup of current registry"
    echo "  restore <backup_file>  Restore registry from backup"
    echo "  merge <other_registry> Merge another registry into current"
    echo "  export <format>        Export registry (json|env|yaml)"
    echo ""
    echo "Options:"
    echo "  --registry <file>      Use specific registry file (default: deployment_addresses.json)"
    echo "  --network <network>    Filter by network (for multi-network registries)"
    echo ""
    echo "Examples:"
    echo "  $0 list"
    echo "  $0 get kyc_registry"
    echo "  $0 set new_contract CNEW123..."
    echo "  $0 validate"
    echo "  $0 export env > contracts.env"
}

# Check if registry file exists
check_registry() {
    if [ ! -f "$REGISTRY_FILE" ]; then
        log_error "Registry file not found: $REGISTRY_FILE"
        log_info "Run deployment script first or specify correct registry file with --registry"
        exit 1
    fi
}

# Create backup of registry
backup_registry() {
    log_step "Creating registry backup..."
    
    mkdir -p $BACKUP_DIR
    BACKUP_FILE="$BACKUP_DIR/registry_backup_$(date +%Y%m%d_%H%M%S).json"
    
    cp "$REGISTRY_FILE" "$BACKUP_FILE"
    log_info "Registry backed up to: $BACKUP_FILE"
    echo "$BACKUP_FILE"
}

# List all contracts
list_contracts() {
    check_registry
    
    log_step "Registered contracts in $REGISTRY_FILE:"
    echo ""
    
    # Extract and display contract information
    jq -r '.contracts | to_entries[] | "  \(.key): \(.value)"' "$REGISTRY_FILE"
    
    echo ""
    NETWORK=$(jq -r '.network // "unknown"' "$REGISTRY_FILE")
    DEPLOYED_AT=$(jq -r '.deployed_at // "unknown"' "$REGISTRY_FILE")
    ADMIN=$(jq -r '.admin_address // "unknown"' "$REGISTRY_FILE")
    
    echo "Network: $NETWORK"
    echo "Deployed: $DEPLOYED_AT"
    echo "Admin: $ADMIN"
}

# Get specific contract address
get_contract() {
    local contract_name="$1"
    
    if [ -z "$contract_name" ]; then
        log_error "Contract name is required"
        exit 1
    fi
    
    check_registry
    
    ADDRESS=$(jq -r ".contracts.\"$contract_name\" // empty" "$REGISTRY_FILE")
    
    if [ -z "$ADDRESS" ]; then
        log_error "Contract '$contract_name' not found in registry"
        exit 1
    fi
    
    echo "$ADDRESS"
}

# Set contract address
set_contract() {
    local contract_name="$1"
    local contract_address="$2"
    
    if [ -z "$contract_name" ] || [ -z "$contract_address" ]; then
        log_error "Both contract name and address are required"
        exit 1
    fi
    
    check_registry
    
    # Validate contract address format
    if [[ ! "$contract_address" =~ ^C[A-Z0-9]{55}$ ]]; then
        log_error "Invalid Stellar contract address format: $contract_address"
        exit 1
    fi
    
    # Create backup before modification
    BACKUP_FILE=$(backup_registry)
    
    # Update registry
    jq ".contracts.\"$contract_name\" = \"$contract_address\"" "$REGISTRY_FILE" > "${REGISTRY_FILE}.tmp"
    mv "${REGISTRY_FILE}.tmp" "$REGISTRY_FILE"
    
    log_info "Contract '$contract_name' set to: $contract_address"
    log_info "Backup created: $BACKUP_FILE"
}

# Remove contract from registry
remove_contract() {
    local contract_name="$1"
    
    if [ -z "$contract_name" ]; then
        log_error "Contract name is required"
        exit 1
    fi
    
    check_registry
    
    # Check if contract exists
    if ! jq -e ".contracts.\"$contract_name\"" "$REGISTRY_FILE" > /dev/null; then
        log_error "Contract '$contract_name' not found in registry"
        exit 1
    fi
    
    # Create backup before modification
    BACKUP_FILE=$(backup_registry)
    
    # Remove contract
    jq "del(.contracts.\"$contract_name\")" "$REGISTRY_FILE" > "${REGISTRY_FILE}.tmp"
    mv "${REGISTRY_FILE}.tmp" "$REGISTRY_FILE"
    
    log_info "Contract '$contract_name' removed from registry"
    log_info "Backup created: $BACKUP_FILE"
}

# Validate all contract addresses
validate_registry() {
    check_registry
    
    log_step "Validating contract addresses..."
    
    VALID=true
    
    # Check each contract address
    while IFS= read -r line; do
        CONTRACT_NAME=$(echo "$line" | jq -r '.key')
        CONTRACT_ADDRESS=$(echo "$line" | jq -r '.value')
        
        if [[ ! "$CONTRACT_ADDRESS" =~ ^C[A-Z0-9]{55}$ ]]; then
            log_error "Invalid address format for '$CONTRACT_NAME': $CONTRACT_ADDRESS"
            VALID=false
        else
            log_info "✓ $CONTRACT_NAME: $CONTRACT_ADDRESS"
        fi
    done < <(jq -c '.contracts | to_entries[]' "$REGISTRY_FILE")
    
    if [ "$VALID" = true ]; then
        log_info "All contract addresses are valid"
    else
        log_error "Registry validation failed"
        exit 1
    fi
}

# Export registry in different formats
export_registry() {
    local format="$1"
    
    if [ -z "$format" ]; then
        format="json"
    fi
    
    check_registry
    
    case "$format" in
        "json")
            cat "$REGISTRY_FILE"
            ;;
        "env")
            echo "# Contract Registry Environment Variables"
            echo "# Generated on $(date)"
            echo ""
            jq -r '.contracts | to_entries[] | "export \(.key | ascii_upcase)_CONTRACT=\(.value)"' "$REGISTRY_FILE"
            ;;
        "yaml")
            echo "# Contract Registry YAML"
            echo "# Generated on $(date)"
            echo ""
            echo "contracts:"
            jq -r '.contracts | to_entries[] | "  \(.key): \(.value)"' "$REGISTRY_FILE"
            ;;
        *)
            log_error "Unsupported export format: $format"
            log_info "Supported formats: json, env, yaml"
            exit 1
            ;;
    esac
}

# Merge another registry
merge_registry() {
    local other_registry="$1"
    
    if [ -z "$other_registry" ]; then
        log_error "Other registry file is required"
        exit 1
    fi
    
    if [ ! -f "$other_registry" ]; then
        log_error "Other registry file not found: $other_registry"
        exit 1
    fi
    
    check_registry
    
    # Create backup before merge
    BACKUP_FILE=$(backup_registry)
    
    # Merge registries
    jq -s '.[0] * .[1] | .contracts = (.[0].contracts * .[1].contracts)' "$REGISTRY_FILE" "$other_registry" > "${REGISTRY_FILE}.tmp"
    mv "${REGISTRY_FILE}.tmp" "$REGISTRY_FILE"
    
    log_info "Registry merged with: $other_registry"
    log_info "Backup created: $BACKUP_FILE"
}

# Restore from backup
restore_registry() {
    local backup_file="$1"
    
    if [ -z "$backup_file" ]; then
        log_error "Backup file is required"
        exit 1
    fi
    
    if [ ! -f "$backup_file" ]; then
        log_error "Backup file not found: $backup_file"
        exit 1
    fi
    
    # Validate backup file
    if ! jq empty "$backup_file" 2>/dev/null; then
        log_error "Invalid JSON in backup file: $backup_file"
        exit 1
    fi
    
    # Create backup of current registry if it exists
    if [ -f "$REGISTRY_FILE" ]; then
        CURRENT_BACKUP=$(backup_registry)
        log_info "Current registry backed up to: $CURRENT_BACKUP"
    fi
    
    # Restore from backup
    cp "$backup_file" "$REGISTRY_FILE"
    log_info "Registry restored from: $backup_file"
}

# Main function
main() {
    # Parse options
    while [[ $# -gt 0 ]]; do
        case $1 in
            --registry)
                REGISTRY_FILE="$2"
                shift 2
                ;;
            --network)
                NETWORK_FILTER="$2"
                shift 2
                ;;
            -h|--help)
                show_usage
                exit 0
                ;;
            *)
                break
                ;;
        esac
    done
    
    # Parse command
    COMMAND="$1"
    shift || true
    
    case "$COMMAND" in
        "list")
            list_contracts
            ;;
        "get")
            get_contract "$1"
            ;;
        "set")
            set_contract "$1" "$2"
            ;;
        "remove")
            remove_contract "$1"
            ;;
        "validate")
            validate_registry
            ;;
        "backup")
            backup_registry
            ;;
        "restore")
            restore_registry "$1"
            ;;
        "merge")
            merge_registry "$1"
            ;;
        "export")
            export_registry "$1"
            ;;
        "")
            log_error "Command is required"
            show_usage
            exit 1
            ;;
        *)
            log_error "Unknown command: $COMMAND"
            show_usage
            exit 1
            ;;
    esac
}

# Run main function
main "$@"