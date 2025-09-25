#!/bin/bash

# Configuration management utility for Soroban contracts

set -e

# Configuration
CONFIG_FILE="config/networks.yaml"
NETWORK=${NETWORK:-"testnet"}

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

log_info() {
    echo -e "${GREEN}[CONFIG]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[CONFIG]${NC} $1"
}

log_error() {
    echo -e "${RED}[CONFIG]${NC} $1"
}

log_step() {
    echo -e "${BLUE}[CONFIG]${NC} $1"
}

# Check if yq is available for YAML processing
check_yq() {
    if ! command -v yq &> /dev/null; then
        log_warn "yq not found. Installing..."
        
        # Install yq based on OS
        if [[ "$OSTYPE" == "darwin"* ]]; then
            brew install yq
        elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
            # Try to install via package manager
            if command -v apt-get &> /dev/null; then
                sudo apt-get update && sudo apt-get install -y yq
            elif command -v yum &> /dev/null; then
                sudo yum install -y yq
            else
                # Install binary directly
                wget -qO /usr/local/bin/yq https://github.com/mikefarah/yq/releases/latest/download/yq_linux_amd64
                chmod +x /usr/local/bin/yq
            fi
        else
            log_error "Unsupported OS. Please install yq manually."
            exit 1
        fi
    fi
}

# Load configuration for a specific network
load_network_config() {
    local network="$1"
    
    if [ ! -f "$CONFIG_FILE" ]; then
        log_error "Configuration file not found: $CONFIG_FILE"
        exit 1
    fi
    
    if ! yq eval ".networks.$network" "$CONFIG_FILE" | grep -q "name:"; then
        log_error "Network configuration not found: $network"
        exit 1
    fi
    
    log_info "Loading configuration for $network network"
}

# Get configuration value
get_config() {
    local network="$1"
    local key="$2"
    
    load_network_config "$network"
    
    local value=$(yq eval ".networks.$network.$key" "$CONFIG_FILE")
    
    if [ "$value" = "null" ]; then
        log_error "Configuration key not found: $key"
        exit 1
    fi
    
    echo "$value"
}

# Set configuration value
set_config() {
    local network="$1"
    local key="$2"
    local value="$3"
    
    load_network_config "$network"
    
    # Create backup
    cp "$CONFIG_FILE" "${CONFIG_FILE}.backup"
    
    # Update configuration
    yq eval ".networks.$network.$key = \"$value\"" -i "$CONFIG_FILE"
    
    log_info "Updated $network.$key = $value"
}

# Generate environment variables from configuration
generate_env() {
    local network="$1"
    local output_file="${2:-config/$network/config.env}"
    
    load_network_config "$network"
    
    # Create output directory
    mkdir -p "$(dirname "$output_file")"
    
    log_info "Generating environment variables for $network..."
    
    cat > "$output_file" << EOF
# Generated configuration for $network network
# Generated at: $(date -u +%Y-%m-%dT%H:%M:%SZ)

# Network settings
export NETWORK="$network"
export STELLAR_RPC_URL="$(yq eval ".networks.$network.rpc_url" "$CONFIG_FILE")"
export NETWORK_PASSPHRASE="$(yq eval ".networks.$network.network_passphrase" "$CONFIG_FILE")"
export HORIZON_URL="$(yq eval ".networks.$network.horizon_url" "$CONFIG_FILE")"

# Deployment settings
export DEPLOYMENT_FEE="$(yq eval ".networks.$network.deployment.fee_per_operation" "$CONFIG_FILE")"
export DEPLOYMENT_TIMEOUT="$(yq eval ".networks.$network.deployment.timeout" "$CONFIG_FILE")"
export DEPLOYMENT_RETRIES="$(yq eval ".networks.$network.deployment.retry_attempts" "$CONFIG_FILE")"

# KYC Registry settings
export KYC_TIER1_DAILY="$(yq eval ".networks.$network.kyc_registry.tier_limits.tier_1.daily_limit" "$CONFIG_FILE")"
export KYC_TIER1_MONTHLY="$(yq eval ".networks.$network.kyc_registry.tier_limits.tier_1.monthly_limit" "$CONFIG_FILE")"
export KYC_TIER2_DAILY="$(yq eval ".networks.$network.kyc_registry.tier_limits.tier_2.daily_limit" "$CONFIG_FILE")"
export KYC_TIER2_MONTHLY="$(yq eval ".networks.$network.kyc_registry.tier_limits.tier_2.monthly_limit" "$CONFIG_FILE")"

# Reserve Manager settings
export RESERVE_THRESHOLD="$(yq eval ".networks.$network.reserve_manager.reserve_threshold" "$CONFIG_FILE")"
export WITHDRAWAL_FEE="$(yq eval ".networks.$network.reserve_manager.withdrawal_fee" "$CONFIG_FILE")"

# Token settings
export ISTSI_TOKEN_NAME="$(yq eval ".networks.$network.tokens.istsi.name" "$CONFIG_FILE")"
export ISTSI_TOKEN_SYMBOL="$(yq eval ".networks.$network.tokens.istsi.symbol" "$CONFIG_FILE")"
export ISTSI_TOKEN_DECIMALS="$(yq eval ".networks.$network.tokens.istsi.decimals" "$CONFIG_FILE")"

export FUNGIBLE_TOKEN_NAME="$(yq eval ".networks.$network.tokens.fungible.name" "$CONFIG_FILE")"
export FUNGIBLE_TOKEN_SYMBOL="$(yq eval ".networks.$network.tokens.fungible.symbol" "$CONFIG_FILE")"
export FUNGIBLE_TOKEN_DECIMALS="$(yq eval ".networks.$network.tokens.fungible.decimals" "$CONFIG_FILE")"
EOF
    
    log_info "Environment variables generated: $output_file"
}

# Validate configuration
validate_config() {
    local network="$1"
    
    log_step "Validating configuration for $network..."
    
    load_network_config "$network"
    
    local errors=0
    
    # Check required fields
    local required_fields=(
        "name"
        "rpc_url"
        "network_passphrase"
        "deployment.fee_per_operation"
        "kyc_registry.tier_limits.tier_1.daily_limit"
        "reserve_manager.reserve_threshold"
        "tokens.istsi.name"
    )
    
    for field in "${required_fields[@]}"; do
        local value=$(yq eval ".networks.$network.$field" "$CONFIG_FILE")
        if [ "$value" = "null" ] || [ -z "$value" ]; then
            log_error "Missing required field: $field"
            errors=$((errors + 1))
        fi
    done
    
    # Validate numeric values
    local reserve_threshold=$(yq eval ".networks.$network.reserve_manager.reserve_threshold" "$CONFIG_FILE")
    if [ "$reserve_threshold" -lt 5000 ] || [ "$reserve_threshold" -gt 10000 ]; then
        log_error "Invalid reserve threshold: $reserve_threshold (should be 50-100%)"
        errors=$((errors + 1))
    fi
    
    # Validate tier limits
    local tier1_daily=$(yq eval ".networks.$network.kyc_registry.tier_limits.tier_1.daily_limit" "$CONFIG_FILE")
    local tier1_monthly=$(yq eval ".networks.$network.kyc_registry.tier_limits.tier_1.monthly_limit" "$CONFIG_FILE")
    
    if [ "$tier1_monthly" -lt "$tier1_daily" ]; then
        log_error "Tier 1 monthly limit should be >= daily limit"
        errors=$((errors + 1))
    fi
    
    if [ $errors -eq 0 ]; then
        log_info "Configuration validation passed"
    else
        log_error "Configuration validation failed with $errors errors"
        exit 1
    fi
}

# Show configuration for a network
show_config() {
    local network="$1"
    
    load_network_config "$network"
    
    log_step "Configuration for $network network:"
    echo
    
    yq eval ".networks.$network" "$CONFIG_FILE"
}

# List available networks
list_networks() {
    log_step "Available networks:"
    echo
    
    yq eval '.networks | keys | .[]' "$CONFIG_FILE" | while read -r network; do
        local name=$(yq eval ".networks.$network.name" "$CONFIG_FILE")
        local rpc_url=$(yq eval ".networks.$network.rpc_url" "$CONFIG_FILE")
        echo "  $network: $name ($rpc_url)"
    done
}

# Create new network configuration
create_network() {
    local network="$1"
    local name="$2"
    local rpc_url="$3"
    local passphrase="$4"
    
    if [ -z "$network" ] || [ -z "$name" ] || [ -z "$rpc_url" ] || [ -z "$passphrase" ]; then
        log_error "Usage: create_network <network> <name> <rpc_url> <passphrase>"
        exit 1
    fi
    
    log_info "Creating new network configuration: $network"
    
    # Create backup
    cp "$CONFIG_FILE" "${CONFIG_FILE}.backup"
    
    # Add new network configuration
    yq eval ".networks.$network = {}" -i "$CONFIG_FILE"
    yq eval ".networks.$network.name = \"$name\"" -i "$CONFIG_FILE"
    yq eval ".networks.$network.rpc_url = \"$rpc_url\"" -i "$CONFIG_FILE"
    yq eval ".networks.$network.network_passphrase = \"$passphrase\"" -i "$CONFIG_FILE"
    
    # Add default deployment settings
    yq eval ".networks.$network.deployment.fee_per_operation = 10000" -i "$CONFIG_FILE"
    yq eval ".networks.$network.deployment.timeout = 300" -i "$CONFIG_FILE"
    yq eval ".networks.$network.deployment.retry_attempts = 3" -i "$CONFIG_FILE"
    
    log_info "Network configuration created. Please customize the settings."
}

# Export configuration to different formats
export_config() {
    local network="$1"
    local format="${2:-json}"
    local output_file="${3:-config/$network/config.$format}"
    
    load_network_config "$network"
    
    mkdir -p "$(dirname "$output_file")"
    
    case "$format" in
        "json")
            yq eval ".networks.$network" "$CONFIG_FILE" -o json > "$output_file"
            ;;
        "yaml")
            yq eval ".networks.$network" "$CONFIG_FILE" > "$output_file"
            ;;
        "toml")
            yq eval ".networks.$network" "$CONFIG_FILE" -o toml > "$output_file"
            ;;
        *)
            log_error "Unsupported format: $format"
            exit 1
            ;;
    esac
    
    log_info "Configuration exported to $output_file"
}

# Show usage information
show_usage() {
    echo "Configuration Manager for Soroban Contracts"
    echo
    echo "Usage: $0 [COMMAND] [OPTIONS]"
    echo
    echo "Commands:"
    echo "  get <network> <key>              Get configuration value"
    echo "  set <network> <key> <value>      Set configuration value"
    echo "  show <network>                   Show network configuration"
    echo "  list                             List available networks"
    echo "  validate <network>               Validate network configuration"
    echo "  generate-env <network> [file]    Generate environment variables"
    echo "  create <network> <name> <rpc> <passphrase>  Create new network"
    echo "  export <network> [format] [file] Export configuration"
    echo "  help                             Show this help"
    echo
    echo "Examples:"
    echo "  $0 show testnet"
    echo "  $0 get testnet rpc_url"
    echo "  $0 set testnet reserve_manager.reserve_threshold 9500"
    echo "  $0 generate-env mainnet"
    echo "  $0 validate testnet"
    echo "  $0 export testnet json config/testnet.json"
}

# Main function
main() {
    local command="$1"
    shift || true
    
    check_yq
    
    case "$command" in
        "get")
            get_config "$1" "$2"
            ;;
        "set")
            set_config "$1" "$2" "$3"
            ;;
        "show")
            show_config "$1"
            ;;
        "list")
            list_networks
            ;;
        "validate")
            validate_config "$1"
            ;;
        "generate-env")
            generate_env "$1" "$2"
            ;;
        "create")
            create_network "$1" "$2" "$3" "$4"
            ;;
        "export")
            export_config "$1" "$2" "$3"
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