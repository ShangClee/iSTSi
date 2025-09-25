#!/bin/bash

# CI/CD build script with comprehensive validation and artifact generation

set -e

# Configuration
CI_MODE=true
ARTIFACT_DIR="artifacts"
BUILD_NUMBER=${BUILD_NUMBER:-$(date +%Y%m%d%H%M%S)}

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

log_info() {
    echo -e "${GREEN}[CI]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[CI]${NC} $1"
}

log_error() {
    echo -e "${RED}[CI]${NC} $1"
}

log_step() {
    echo -e "${BLUE}[CI]${NC} $1"
}

# Setup CI environment
setup_ci() {
    log_step "Setting up CI environment..."
    
    # Create artifact directory
    mkdir -p "$ARTIFACT_DIR"
    
    # Install required tools
    if ! command -v stellar &> /dev/null; then
        log_info "Installing Stellar CLI..."
        cargo install --locked stellar-cli --features opt
    fi
    
    # Add wasm target
    rustup target add wasm32-unknown-unknown
    
    # Install wasm-opt for optimization
    if ! command -v wasm-opt &> /dev/null; then
        log_info "Installing binaryen for WASM optimization..."
        if [[ "$OSTYPE" == "linux-gnu"* ]]; then
            wget -q https://github.com/WebAssembly/binaryen/releases/latest/download/binaryen-version_114-x86_64-linux.tar.gz
            tar -xzf binaryen-*.tar.gz
            sudo cp binaryen-*/bin/wasm-opt /usr/local/bin/
        elif [[ "$OSTYPE" == "darwin"* ]]; then
            brew install binaryen
        fi
    fi
    
    log_info "CI environment setup completed"
}

# Run comprehensive tests
run_tests() {
    log_step "Running contract tests..."
    
    # Run unit tests for each contract
    for contract_dir in contracts/*/; do
        if [ -d "$contract_dir" ] && [ -f "$contract_dir/Cargo.toml" ]; then
            contract_name=$(basename "$contract_dir")
            log_info "Testing $contract_name..."
            
            cd "$contract_dir"
            cargo test --target wasm32-unknown-unknown
            cd - > /dev/null
        fi
    done
    
    # Run integration tests if they exist
    if [ -d "tests" ] && [ "$(ls -A tests)" ]; then
        log_info "Running integration tests..."
        cargo test --test '*'
    fi
    
    log_info "All tests passed"
}

# Security and quality checks
security_checks() {
    log_step "Running security and quality checks..."
    
    # Check for common security issues
    if command -v cargo-audit &> /dev/null; then
        log_info "Running security audit..."
        cargo audit
    else
        log_warn "cargo-audit not found, skipping security audit"
    fi
    
    # Check code formatting
    log_info "Checking code formatting..."
    cargo fmt --all -- --check
    
    # Run clippy for linting
    log_info "Running clippy lints..."
    cargo clippy --all-targets --all-features -- -D warnings
    
    log_info "Security and quality checks passed"
}

# Build contracts with full optimization
build_optimized() {
    log_step "Building optimized contracts..."
    
    # Clean build
    ./scripts/build.sh --clean
    
    # Build all contracts
    ./scripts/build.sh
    
    log_info "Optimized build completed"
}

# Generate CI artifacts
generate_artifacts() {
    log_step "Generating CI artifacts..."
    
    # Copy WASM files to artifacts directory
    for contract_dir in contracts/*/; do
        if [ -d "$contract_dir" ] && [ -f "$contract_dir/Cargo.toml" ]; then
            contract_name=$(basename "$contract_dir")
            wasm_file="$contract_dir/target/wasm32-unknown-unknown/release/${contract_name}.wasm"
            
            if [ -f "$wasm_file" ]; then
                cp "$wasm_file" "$ARTIFACT_DIR/${contract_name}.wasm"
                
                # Generate metadata
                size=$(stat -f%z "$wasm_file" 2>/dev/null || stat -c%s "$wasm_file" 2>/dev/null)
                hash=$(shasum -a 256 "$wasm_file" | cut -d' ' -f1)
                
                cat > "$ARTIFACT_DIR/${contract_name}.json" << EOF
{
  "contract_name": "$contract_name",
  "build_number": "$BUILD_NUMBER",
  "build_timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "wasm_size": $size,
  "wasm_hash": "$hash",
  "git_commit": "$(git rev-parse HEAD 2>/dev/null || echo 'unknown')",
  "git_branch": "$(git branch --show-current 2>/dev/null || echo 'unknown')"
}
EOF
            fi
        fi
    done
    
    # Create deployment package
    tar -czf "$ARTIFACT_DIR/contracts-${BUILD_NUMBER}.tar.gz" -C "$ARTIFACT_DIR" *.wasm *.json
    
    # Generate deployment manifest
    cat > "$ARTIFACT_DIR/deployment-manifest.json" << EOF
{
  "build_number": "$BUILD_NUMBER",
  "build_timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "git_commit": "$(git rev-parse HEAD 2>/dev/null || echo 'unknown')",
  "git_branch": "$(git branch --show-current 2>/dev/null || echo 'unknown')",
  "contracts": [
$(for contract_dir in contracts/*/; do
    if [ -d "$contract_dir" ] && [ -f "$contract_dir/Cargo.toml" ]; then
        contract_name=$(basename "$contract_dir")
        if [ -f "$ARTIFACT_DIR/${contract_name}.json" ]; then
            echo "    \"$contract_name\","
        fi
    fi
done | sed '$ s/,$//')
  ]
}
EOF
    
    log_info "CI artifacts generated in $ARTIFACT_DIR/"
}

# Validate deployment readiness
validate_deployment() {
    log_step "Validating deployment readiness..."
    
    local validation_errors=0
    
    # Check all expected contracts are built
    local expected_contracts=("kyc_registry" "reserve_manager" "istsi_token" "fungible" "integration_router")
    
    for contract in "${expected_contracts[@]}"; do
        if [ ! -f "$ARTIFACT_DIR/${contract}.wasm" ]; then
            log_error "Missing contract: $contract"
            validation_errors=$((validation_errors + 1))
        else
            # Check WASM file is not empty
            size=$(stat -f%z "$ARTIFACT_DIR/${contract}.wasm" 2>/dev/null || stat -c%s "$ARTIFACT_DIR/${contract}.wasm" 2>/dev/null)
            if [ "$size" -eq 0 ]; then
                log_error "Empty WASM file: $contract"
                validation_errors=$((validation_errors + 1))
            fi
        fi
    done
    
    # Check deployment package exists
    if [ ! -f "$ARTIFACT_DIR/contracts-${BUILD_NUMBER}.tar.gz" ]; then
        log_error "Deployment package not found"
        validation_errors=$((validation_errors + 1))
    fi
    
    if [ $validation_errors -gt 0 ]; then
        log_error "Deployment validation failed with $validation_errors errors"
        exit 1
    fi
    
    log_info "Deployment validation passed"
}

# Main CI function
main() {
    log_info "Starting CI build process (Build #$BUILD_NUMBER)..."
    
    setup_ci
    security_checks
    run_tests
    build_optimized
    generate_artifacts
    validate_deployment
    
    log_info "CI build completed successfully!"
    log_info "Artifacts available in: $ARTIFACT_DIR/"
    log_info "Deployment package: $ARTIFACT_DIR/contracts-${BUILD_NUMBER}.tar.gz"
}

main "$@"