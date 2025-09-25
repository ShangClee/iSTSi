#!/bin/bash

# Enhanced build script for all Soroban contracts with optimization and validation

set -e

# Configuration
BUILD_MODE=${BUILD_MODE:-"release"}
TARGET_DIR="target/wasm32-unknown-unknown"
CONTRACTS_DIR="contracts"

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

# Check prerequisites
check_prerequisites() {
    log_step "Checking build prerequisites..."
    
    # Check if stellar CLI is available
    if ! command -v stellar &> /dev/null; then
        log_warn "Stellar CLI not found. Installing via cargo..."
        cargo install --locked stellar-cli --features opt
    fi
    
    # Check if wasm32-unknown-unknown target is installed
    if ! rustup target list --installed | grep -q "wasm32-unknown-unknown"; then
        log_info "Installing wasm32-unknown-unknown target..."
        rustup target add wasm32-unknown-unknown
    fi
    
    # Check if wasm-opt is available for optimization
    if ! command -v wasm-opt &> /dev/null; then
        log_warn "wasm-opt not found. Install binaryen for WASM optimization."
    fi
    
    log_info "Prerequisites check completed"
}

# Clean previous builds
clean_build() {
    log_step "Cleaning previous builds..."
    
    if [ -d "$TARGET_DIR" ]; then
        rm -rf "$TARGET_DIR"
        log_info "Cleaned target directory"
    fi
    
    # Clean individual contract targets
    for contract_dir in $CONTRACTS_DIR/*/; do
        if [ -d "$contract_dir/target" ]; then
            rm -rf "$contract_dir/target"
            log_info "Cleaned $(basename "$contract_dir") target"
        fi
    done
}

# Build individual contract
build_contract() {
    local contract_name=$1
    local contract_path="$CONTRACTS_DIR/$contract_name"
    
    if [ ! -d "$contract_path" ]; then
        log_error "Contract directory not found: $contract_path"
        return 1
    fi
    
    log_info "Building $contract_name..."
    
    cd "$contract_path"
    
    # Build with stellar CLI for better optimization
    if command -v stellar &> /dev/null; then
        stellar contract build
    else
        # Fallback to cargo build
        cargo build --target wasm32-unknown-unknown --release
    fi
    
    cd - > /dev/null
    
    # Verify WASM file was created
    local wasm_file="$contract_path/target/wasm32-unknown-unknown/release/${contract_name}.wasm"
    if [ -f "$wasm_file" ]; then
        local size=$(stat -f%z "$wasm_file" 2>/dev/null || stat -c%s "$wasm_file" 2>/dev/null)
        log_info "$contract_name built successfully (${size} bytes)"
        
        # Optimize WASM if wasm-opt is available
        if command -v wasm-opt &> /dev/null; then
            log_info "Optimizing $contract_name WASM..."
            wasm-opt -Oz "$wasm_file" -o "${wasm_file}.opt"
            mv "${wasm_file}.opt" "$wasm_file"
            local opt_size=$(stat -f%z "$wasm_file" 2>/dev/null || stat -c%s "$wasm_file" 2>/dev/null)
            log_info "$contract_name optimized (${opt_size} bytes)"
        fi
    else
        log_error "Failed to build $contract_name - WASM file not found"
        return 1
    fi
}

# Build all contracts
build_all_contracts() {
    log_step "Building all contracts..."
    
    local contracts=()
    
    # Discover all contract directories
    for contract_dir in $CONTRACTS_DIR/*/; do
        if [ -d "$contract_dir" ] && [ -f "$contract_dir/Cargo.toml" ]; then
            contract_name=$(basename "$contract_dir")
            contracts+=("$contract_name")
        fi
    done
    
    if [ ${#contracts[@]} -eq 0 ]; then
        log_error "No contracts found in $CONTRACTS_DIR"
        exit 1
    fi
    
    log_info "Found ${#contracts[@]} contracts: ${contracts[*]}"
    
    # Build contracts in dependency order
    local build_order=("kyc_registry" "reserve_manager" "fungible" "istsi_token" "integration_router")
    local built_contracts=()
    
    # Build contracts in specified order if they exist
    for contract in "${build_order[@]}"; do
        if [[ " ${contracts[*]} " =~ " ${contract} " ]]; then
            build_contract "$contract"
            built_contracts+=("$contract")
        fi
    done
    
    # Build any remaining contracts
    for contract in "${contracts[@]}"; do
        if [[ ! " ${built_contracts[*]} " =~ " ${contract} " ]]; then
            build_contract "$contract"
            built_contracts+=("$contract")
        fi
    done
    
    log_info "All contracts built successfully"
}

# Validate built contracts
validate_contracts() {
    log_step "Validating built contracts..."
    
    local validation_errors=0
    
    for contract_dir in $CONTRACTS_DIR/*/; do
        if [ -d "$contract_dir" ] && [ -f "$contract_dir/Cargo.toml" ]; then
            contract_name=$(basename "$contract_dir")
            wasm_file="$contract_dir/target/wasm32-unknown-unknown/release/${contract_name}.wasm"
            
            if [ -f "$wasm_file" ]; then
                # Check WASM file size (should be reasonable)
                size=$(stat -f%z "$wasm_file" 2>/dev/null || stat -c%s "$wasm_file" 2>/dev/null)
                if [ "$size" -gt 1048576 ]; then  # 1MB
                    log_warn "$contract_name WASM is large (${size} bytes) - consider optimization"
                fi
                
                # Validate WASM format using stellar CLI if available
                if command -v stellar &> /dev/null; then
                    if stellar contract install --wasm "$wasm_file" --dry-run &>/dev/null; then
                        log_info "$contract_name WASM validation passed"
                    else
                        log_error "$contract_name WASM validation failed"
                        validation_errors=$((validation_errors + 1))
                    fi
                fi
            else
                log_error "$contract_name WASM file not found"
                validation_errors=$((validation_errors + 1))
            fi
        fi
    done
    
    if [ $validation_errors -gt 0 ]; then
        log_error "Validation failed with $validation_errors errors"
        exit 1
    fi
    
    log_info "All contracts validated successfully"
}

# Generate build report
generate_build_report() {
    log_step "Generating build report..."
    
    local report_file="build_report_$(date +%Y%m%d_%H%M%S).json"
    
    echo "{" > "$report_file"
    echo "  \"build_timestamp\": \"$(date -u +%Y-%m-%dT%H:%M:%SZ)\"," >> "$report_file"
    echo "  \"build_mode\": \"$BUILD_MODE\"," >> "$report_file"
    echo "  \"contracts\": {" >> "$report_file"
    
    local first=true
    for contract_dir in $CONTRACTS_DIR/*/; do
        if [ -d "$contract_dir" ] && [ -f "$contract_dir/Cargo.toml" ]; then
            contract_name=$(basename "$contract_dir")
            wasm_file="$contract_dir/target/wasm32-unknown-unknown/release/${contract_name}.wasm"
            
            if [ -f "$wasm_file" ]; then
                if [ "$first" = false ]; then
                    echo "," >> "$report_file"
                fi
                first=false
                
                size=$(stat -f%z "$wasm_file" 2>/dev/null || stat -c%s "$wasm_file" 2>/dev/null)
                hash=$(shasum -a 256 "$wasm_file" | cut -d' ' -f1)
                
                echo "    \"$contract_name\": {" >> "$report_file"
                echo "      \"wasm_path\": \"$wasm_file\"," >> "$report_file"
                echo "      \"size_bytes\": $size," >> "$report_file"
                echo "      \"sha256\": \"$hash\"" >> "$report_file"
                echo -n "    }" >> "$report_file"
            fi
        fi
    done
    
    echo "" >> "$report_file"
    echo "  }" >> "$report_file"
    echo "}" >> "$report_file"
    
    log_info "Build report generated: $report_file"
}

# List built contracts with details
list_contracts() {
    log_step "Built contracts summary:"
    
    printf "%-20s %-15s %-64s\n" "Contract" "Size" "SHA256"
    printf "%-20s %-15s %-64s\n" "--------" "----" "------"
    
    for contract_dir in $CONTRACTS_DIR/*/; do
        if [ -d "$contract_dir" ] && [ -f "$contract_dir/Cargo.toml" ]; then
            contract_name=$(basename "$contract_dir")
            wasm_file="$contract_dir/target/wasm32-unknown-unknown/release/${contract_name}.wasm"
            
            if [ -f "$wasm_file" ]; then
                size=$(stat -f%z "$wasm_file" 2>/dev/null || stat -c%s "$wasm_file" 2>/dev/null)
                hash=$(shasum -a 256 "$wasm_file" | cut -d' ' -f1)
                size_kb=$((size / 1024))
                
                printf "%-20s %-15s %-64s\n" "$contract_name" "${size_kb}KB" "$hash"
            fi
        fi
    done
}

# Main build function
main() {
    log_info "Starting Soroban contracts build process..."
    
    # Parse command line arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            --clean)
                CLEAN_BUILD=true
                shift
                ;;
            --debug)
                BUILD_MODE="debug"
                shift
                ;;
            --contract)
                SINGLE_CONTRACT="$2"
                shift 2
                ;;
            --help)
                echo "Usage: $0 [OPTIONS]"
                echo "Options:"
                echo "  --clean     Clean previous builds"
                echo "  --debug     Build in debug mode"
                echo "  --contract  Build single contract"
                echo "  --help      Show this help"
                exit 0
                ;;
            *)
                log_error "Unknown option: $1"
                exit 1
                ;;
        esac
    done
    
    check_prerequisites
    
    if [ "$CLEAN_BUILD" = true ]; then
        clean_build
    fi
    
    if [ -n "$SINGLE_CONTRACT" ]; then
        build_contract "$SINGLE_CONTRACT"
    else
        build_all_contracts
    fi
    
    validate_contracts
    generate_build_report
    list_contracts
    
    log_info "Build process completed successfully!"
}

# Run main function with all arguments
main "$@"