#!/bin/bash

# Comprehensive test runner for Soroban contracts

set -e

# Configuration
TEST_MODE=${TEST_MODE:-"all"}
NETWORK=${NETWORK:-"testnet"}
COVERAGE=${COVERAGE:-false}
PARALLEL=${PARALLEL:-true}

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

log_info() {
    echo -e "${GREEN}[TEST]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[TEST]${NC} $1"
}

log_error() {
    echo -e "${RED}[TEST]${NC} $1"
}

log_step() {
    echo -e "${BLUE}[TEST]${NC} $1"
}

# Check prerequisites
check_prerequisites() {
    log_step "Checking test prerequisites..."
    
    # Check if contracts are built
    if [ ! -d "contracts" ]; then
        log_error "Contracts directory not found"
        exit 1
    fi
    
    # Check if stellar CLI is available
    if ! command -v stellar &> /dev/null; then
        log_error "Stellar CLI not found. Please install it first."
        exit 1
    fi
    
    # Ensure wasm32-unknown-unknown target is installed
    if ! rustup target list --installed | grep -q "wasm32-unknown-unknown"; then
        log_info "Installing wasm32-unknown-unknown target..."
        rustup target add wasm32-unknown-unknown
    fi
    
    log_info "Prerequisites check passed"
}

# Build contracts for testing
build_contracts_for_test() {
    log_step "Building contracts for testing..."
    
    # Build all contracts
    ./scripts/build.sh
    
    log_info "Contracts built for testing"
}

# Run unit tests for individual contracts
run_unit_tests() {
    log_step "Running unit tests for individual contracts..."
    
    local test_errors=0
    
    for contract_dir in contracts/*/; do
        if [ -d "$contract_dir" ] && [ -f "$contract_dir/Cargo.toml" ]; then
            contract_name=$(basename "$contract_dir")
            log_info "Testing $contract_name..."
            
            cd "$contract_dir"
            
            if [ "$PARALLEL" = true ]; then
                if cargo test --target wasm32-unknown-unknown --release; then
                    log_info "$contract_name unit tests passed"
                else
                    log_error "$contract_name unit tests failed"
                    test_errors=$((test_errors + 1))
                fi
            else
                if cargo test --target wasm32-unknown-unknown --release -- --test-threads=1; then
                    log_info "$contract_name unit tests passed"
                else
                    log_error "$contract_name unit tests failed"
                    test_errors=$((test_errors + 1))
                fi
            fi
            
            cd - > /dev/null
        fi
    done
    
    if [ $test_errors -gt 0 ]; then
        log_error "$test_errors contract(s) failed unit tests"
        return 1
    fi
    
    log_info "All unit tests passed"
}

# Run integration tests
run_integration_tests() {
    log_step "Running integration tests..."
    
    if [ ! -d "tests" ] || [ -z "$(ls -A tests)" ]; then
        log_warn "No integration tests found in tests/ directory"
        return 0
    fi
    
    log_info "Running integration test suite..."
    
    if [ "$PARALLEL" = true ]; then
        if cargo test --test '*' --release; then
            log_info "Integration tests passed"
        else
            log_error "Integration tests failed"
            return 1
        fi
    else
        if cargo test --test '*' --release -- --test-threads=1; then
            log_info "Integration tests passed"
        else
            log_error "Integration tests failed"
            return 1
        fi
    fi
}

# Run specific test file
run_specific_test() {
    local test_file="$1"
    
    if [ -z "$test_file" ]; then
        log_error "Test file not specified"
        return 1
    fi
    
    log_step "Running specific test: $test_file"
    
    if [ -f "tests/${test_file}.rs" ]; then
        cargo test --test "$test_file" --release
    elif [ -f "tests/${test_file}" ]; then
        cargo test --test "$(basename "$test_file" .rs)" --release
    else
        log_error "Test file not found: $test_file"
        return 1
    fi
}

# Run contract deployment tests
run_deployment_tests() {
    log_step "Running deployment tests on $NETWORK..."
    
    # Check if deployment script exists
    if [ ! -f "scripts/deploy-$NETWORK.sh" ]; then
        log_error "Deployment script not found for $NETWORK"
        return 1
    fi
    
    # Create temporary test environment
    local test_env_dir="test_env_$(date +%s)"
    mkdir -p "$test_env_dir"
    
    # Set test environment variables
    export ADMIN_ADDRESS=${TEST_ADMIN_ADDRESS:-""}
    
    if [ -z "$ADMIN_ADDRESS" ]; then
        log_info "Generating test admin account..."
        stellar keys generate --global test_admin --network "$NETWORK"
        export ADMIN_ADDRESS=$(stellar keys address test_admin)
        
        if [ "$NETWORK" = "testnet" ]; then
            log_info "Funding test admin account..."
            stellar account fund "$ADMIN_ADDRESS" --network testnet
        fi
    fi
    
    # Run deployment test
    log_info "Testing deployment to $NETWORK..."
    if ./scripts/deploy-$NETWORK.sh; then
        log_info "Deployment test passed"
        
        # Run basic contract interaction tests
        if [ -f "scripts/contract-utils.sh" ]; then
            log_info "Testing contract interactions..."
            NETWORK="$NETWORK" ./scripts/contract-utils.sh test
        fi
    else
        log_error "Deployment test failed"
        return 1
    fi
    
    # Cleanup
    rm -rf "$test_env_dir"
    
    if [ -n "$TEST_ADMIN_ADDRESS" ]; then
        stellar keys remove --global test_admin 2>/dev/null || true
    fi
}

# Run performance tests
run_performance_tests() {
    log_step "Running performance tests..."
    
    log_info "Testing contract build times..."
    local start_time=$(date +%s)
    ./scripts/build.sh --clean
    local end_time=$(date +%s)
    local build_time=$((end_time - start_time))
    
    log_info "Build time: ${build_time}s"
    
    # Test WASM file sizes
    log_info "Checking WASM file sizes..."
    for contract_dir in contracts/*/; do
        if [ -d "$contract_dir" ] && [ -f "$contract_dir/Cargo.toml" ]; then
            contract_name=$(basename "$contract_dir")
            wasm_file="$contract_dir/target/wasm32-unknown-unknown/release/${contract_name}.wasm"
            
            if [ -f "$wasm_file" ]; then
                size=$(stat -f%z "$wasm_file" 2>/dev/null || stat -c%s "$wasm_file" 2>/dev/null)
                size_kb=$((size / 1024))
                
                if [ $size_kb -gt 1024 ]; then  # Warn if > 1MB
                    log_warn "$contract_name WASM is large: ${size_kb}KB"
                else
                    log_info "$contract_name WASM size: ${size_kb}KB"
                fi
            fi
        fi
    done
}

# Generate test coverage report
generate_coverage() {
    log_step "Generating test coverage report..."
    
    if ! command -v cargo-tarpaulin &> /dev/null; then
        log_warn "cargo-tarpaulin not found. Installing..."
        cargo install cargo-tarpaulin
    fi
    
    log_info "Running coverage analysis..."
    cargo tarpaulin --target wasm32-unknown-unknown --out Html --output-dir coverage/
    
    log_info "Coverage report generated in coverage/ directory"
}

# Run security tests
run_security_tests() {
    log_step "Running security tests..."
    
    # Check for common security issues
    if command -v cargo-audit &> /dev/null; then
        log_info "Running security audit..."
        cargo audit
    else
        log_warn "cargo-audit not found. Install with: cargo install cargo-audit"
    fi
    
    # Check for unsafe code
    log_info "Checking for unsafe code..."
    if grep -r "unsafe" contracts/*/src/ 2>/dev/null; then
        log_warn "Unsafe code found in contracts"
    else
        log_info "No unsafe code found"
    fi
    
    # Check for TODO/FIXME comments
    log_info "Checking for TODO/FIXME comments..."
    local todos=$(grep -r "TODO\|FIXME" contracts/*/src/ 2>/dev/null | wc -l)
    if [ "$todos" -gt 0 ]; then
        log_warn "Found $todos TODO/FIXME comments"
        grep -r "TODO\|FIXME" contracts/*/src/ 2>/dev/null || true
    else
        log_info "No TODO/FIXME comments found"
    fi
}

# Clean test artifacts
clean_test_artifacts() {
    log_step "Cleaning test artifacts..."
    
    # Remove test coverage files
    rm -rf coverage/
    
    # Remove test deployment configs
    rm -rf config/test_*/
    
    # Remove temporary test files
    find . -name "test_*" -type d -exec rm -rf {} + 2>/dev/null || true
    
    log_info "Test artifacts cleaned"
}

# Show test results summary
show_test_summary() {
    log_step "Test Summary"
    echo
    
    if [ -f "test_results.json" ]; then
        echo "Detailed results available in test_results.json"
    fi
    
    echo "Test run completed at $(date)"
    echo "Network: $NETWORK"
    echo "Mode: $TEST_MODE"
    echo "Coverage: $COVERAGE"
    echo "Parallel: $PARALLEL"
}

# Show usage information
show_usage() {
    echo "Soroban Contract Test Runner"
    echo
    echo "Usage: $0 [COMMAND] [OPTIONS]"
    echo
    echo "Commands:"
    echo "  unit                    Run unit tests only"
    echo "  integration             Run integration tests only"
    echo "  deployment              Run deployment tests"
    echo "  performance             Run performance tests"
    echo "  security                Run security tests"
    echo "  coverage                Generate coverage report"
    echo "  specific <test_file>    Run specific test file"
    echo "  all                     Run all tests (default)"
    echo "  clean                   Clean test artifacts"
    echo "  help                    Show this help"
    echo
    echo "Environment Variables:"
    echo "  TEST_MODE               Test mode (unit/integration/deployment/all)"
    echo "  NETWORK                 Network for deployment tests (testnet/mainnet)"
    echo "  COVERAGE                Generate coverage report (true/false)"
    echo "  PARALLEL                Run tests in parallel (true/false)"
    echo "  TEST_ADMIN_ADDRESS      Admin address for deployment tests"
    echo
    echo "Examples:"
    echo "  $0 unit"
    echo "  $0 integration"
    echo "  NETWORK=testnet $0 deployment"
    echo "  COVERAGE=true $0 all"
    echo "  $0 specific integration_test"
}

# Main test function
main() {
    local command="${1:-all}"
    shift || true
    
    case "$command" in
        "unit")
            check_prerequisites
            build_contracts_for_test
            run_unit_tests
            ;;
        "integration")
            check_prerequisites
            build_contracts_for_test
            run_integration_tests
            ;;
        "deployment")
            check_prerequisites
            build_contracts_for_test
            run_deployment_tests
            ;;
        "performance")
            check_prerequisites
            run_performance_tests
            ;;
        "security")
            check_prerequisites
            run_security_tests
            ;;
        "coverage")
            check_prerequisites
            build_contracts_for_test
            generate_coverage
            ;;
        "specific")
            check_prerequisites
            build_contracts_for_test
            run_specific_test "$1"
            ;;
        "all")
            check_prerequisites
            build_contracts_for_test
            run_unit_tests
            run_integration_tests
            run_security_tests
            
            if [ "$COVERAGE" = true ]; then
                generate_coverage
            fi
            
            show_test_summary
            ;;
        "clean")
            clean_test_artifacts
            ;;
        "help"|"--help"|"-h")
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