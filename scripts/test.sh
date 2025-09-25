#!/bin/bash
# Unified testing script for all components
# Usage: ./scripts/test.sh [component] [test-type] [options]
# Components: frontend, backend, soroban, integration, all (default)
# Test types: unit, integration, e2e, all (default)

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Default values
COMPONENT=${1:-all}
TEST_TYPE=${2:-all}
PARALLEL=${3:-false}
COVERAGE=${4:-false}
TIMESTAMP=$(date +%Y%m%d_%H%M%S)

# Test results tracking
TESTS_DIR="test-results"
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0
SKIPPED_TESTS=0

# Logging functions
log() {
    echo -e "${BLUE}[TEST]${NC} $1"
}

error() {
    echo -e "${RED}[ERROR]${NC} $1" >&2
}

success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

warn() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

# Initialize test environment
init_test_env() {
    log "Initializing test environment..."
    
    # Create test results directory
    mkdir -p "$TESTS_DIR"
    mkdir -p "$TESTS_DIR/reports"
    mkdir -p "$TESTS_DIR/coverage"
    mkdir -p "$TESTS_DIR/logs"
    
    # Start test infrastructure if needed
    if [ "$COMPONENT" = "all" ] || [ "$COMPONENT" = "integration" ]; then
        log "Starting test infrastructure..."
        docker-compose -f docker-compose.test.yml up -d postgres soroban-rpc
        
        # Wait for services to be ready
        log "Waiting for test infrastructure..."
        sleep 15
    fi
}

# Test frontend
test_frontend() {
    log "Running frontend tests..."
    
    if [ ! -d "frontend" ]; then
        warn "Frontend directory not found, skipping frontend tests"
        return 0
    fi
    
    cd frontend
    
    local exit_code=0
    local test_output="$TESTS_DIR/logs/frontend-$TIMESTAMP.log"
    
    # Install dependencies if needed
    if [ ! -d "node_modules" ]; then
        log "Installing frontend dependencies..."
        npm ci
    fi
    
    case $TEST_TYPE in
        unit|all)
            log "Running frontend unit tests..."
            if [ "$COVERAGE" = "true" ]; then
                npm run test -- --run --coverage --reporter=json --outputFile="../$TESTS_DIR/reports/frontend-unit-$TIMESTAMP.json" 2>&1 | tee "$test_output"
            else
                npm run test -- --run --reporter=json --outputFile="../$TESTS_DIR/reports/frontend-unit-$TIMESTAMP.json" 2>&1 | tee "$test_output"
            fi
            exit_code=$?
            ;;
    esac
    
    # Type checking
    if [ "$TEST_TYPE" = "all" ] || [ "$TEST_TYPE" = "unit" ]; then
        log "Running TypeScript type checking..."
        npm run type-check 2>&1 | tee -a "$test_output"
        local type_exit_code=$?
        if [ $exit_code -eq 0 ]; then
            exit_code=$type_exit_code
        fi
    fi
    
    # Linting
    if [ "$TEST_TYPE" = "all" ] || [ "$TEST_TYPE" = "unit" ]; then
        log "Running ESLint..."
        npm run lint 2>&1 | tee -a "$test_output"
        local lint_exit_code=$?
        if [ $exit_code -eq 0 ]; then
            exit_code=$lint_exit_code
        fi
    fi
    
    cd ..
    
    if [ $exit_code -eq 0 ]; then
        success "Frontend tests passed"
        PASSED_TESTS=$((PASSED_TESTS + 1))
    else
        error "Frontend tests failed"
        FAILED_TESTS=$((FAILED_TESTS + 1))
    fi
    
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
    return $exit_code
}

# Test backend
test_backend() {
    log "Running backend tests..."
    
    if [ ! -d "backend" ]; then
        warn "Backend directory not found, skipping backend tests"
        return 0
    fi
    
    cd backend
    
    local exit_code=0
    local test_output="$TESTS_DIR/logs/backend-$TIMESTAMP.log"
    
    # Set test environment
    export DATABASE_URL="postgres://postgres:password@localhost:5432/bitcoin_custody_test"
    export LOCO_ENV="test"
    
    case $TEST_TYPE in
        unit|all)
            log "Running backend unit tests..."
            if [ "$COVERAGE" = "true" ]; then
                cargo test --lib -- --test-threads=1 --format=json > "../$TESTS_DIR/reports/backend-unit-$TIMESTAMP.json" 2>&1
            else
                cargo test --lib -- --test-threads=1 2>&1 | tee "$test_output"
            fi
            exit_code=$?
            ;;
    esac
    
    if [ "$TEST_TYPE" = "integration" ] || [ "$TEST_TYPE" = "all" ]; then
        log "Running backend integration tests..."
        cargo test --test '*' -- --test-threads=1 2>&1 | tee -a "$test_output"
        local integration_exit_code=$?
        if [ $exit_code -eq 0 ]; then
            exit_code=$integration_exit_code
        fi
    fi
    
    # Clippy linting
    if [ "$TEST_TYPE" = "all" ] || [ "$TEST_TYPE" = "unit" ]; then
        log "Running Clippy..."
        cargo clippy -- -D warnings 2>&1 | tee -a "$test_output"
        local clippy_exit_code=$?
        if [ $exit_code -eq 0 ]; then
            exit_code=$clippy_exit_code
        fi
    fi
    
    # Format checking
    if [ "$TEST_TYPE" = "all" ] || [ "$TEST_TYPE" = "unit" ]; then
        log "Checking code formatting..."
        cargo fmt -- --check 2>&1 | tee -a "$test_output"
        local fmt_exit_code=$?
        if [ $exit_code -eq 0 ]; then
            exit_code=$fmt_exit_code
        fi
    fi
    
    cd ..
    
    if [ $exit_code -eq 0 ]; then
        success "Backend tests passed"
        PASSED_TESTS=$((PASSED_TESTS + 1))
    else
        error "Backend tests failed"
        FAILED_TESTS=$((FAILED_TESTS + 1))
    fi
    
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
    return $exit_code
}

# Test Soroban contracts
test_soroban() {
    log "Running Soroban contract tests..."
    
    if [ ! -d "soroban" ]; then
        warn "Soroban directory not found, skipping contract tests"
        return 0
    fi
    
    cd soroban
    
    local exit_code=0
    local test_output="$TESTS_DIR/logs/soroban-$TIMESTAMP.log"
    
    case $TEST_TYPE in
        unit|all)
            log "Running contract unit tests..."
            cargo test --lib 2>&1 | tee "$test_output"
            exit_code=$?
            ;;
    esac
    
    if [ "$TEST_TYPE" = "integration" ] || [ "$TEST_TYPE" = "all" ]; then
        log "Running contract integration tests..."
        cargo test --test '*' 2>&1 | tee -a "$test_output"
        local integration_exit_code=$?
        if [ $exit_code -eq 0 ]; then
            exit_code=$integration_exit_code
        fi
    fi
    
    # Contract compilation test
    if [ "$TEST_TYPE" = "all" ] || [ "$TEST_TYPE" = "unit" ]; then
        log "Testing contract compilation..."
        cargo build --target wasm32-unknown-unknown --release 2>&1 | tee -a "$test_output"
        local build_exit_code=$?
        if [ $exit_code -eq 0 ]; then
            exit_code=$build_exit_code
        fi
    fi
    
    # Clippy for contracts
    if [ "$TEST_TYPE" = "all" ] || [ "$TEST_TYPE" = "unit" ]; then
        log "Running Clippy on contracts..."
        cargo clippy -- -D warnings 2>&1 | tee -a "$test_output"
        local clippy_exit_code=$?
        if [ $exit_code -eq 0 ]; then
            exit_code=$clippy_exit_code
        fi
    fi
    
    cd ..
    
    if [ $exit_code -eq 0 ]; then
        success "Soroban tests passed"
        PASSED_TESTS=$((PASSED_TESTS + 1))
    else
        error "Soroban tests failed"
        FAILED_TESTS=$((FAILED_TESTS + 1))
    fi
    
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
    return $exit_code
}

# Run integration tests
test_integration() {
    log "Running cross-component integration tests..."
    
    local exit_code=0
    local test_output="$TESTS_DIR/logs/integration-$TIMESTAMP.log"
    
    # Ensure all services are running
    log "Starting integration test environment..."
    docker-compose -f docker-compose.test.yml up -d
    
    # Wait for services
    log "Waiting for services to be ready..."
    sleep 30
    
    # Run database migrations
    log "Running test database migrations..."
    docker-compose -f docker-compose.test.yml exec -T backend cargo loco db migrate --environment test
    
    # Test API endpoints
    log "Testing API endpoints..."
    if command -v newman >/dev/null 2>&1; then
        # Run Postman/Newman tests if available
        if [ -f "tests/api/postman-collection.json" ]; then
            newman run tests/api/postman-collection.json \
                --environment tests/api/test-environment.json \
                --reporters cli,json \
                --reporter-json-export "$TESTS_DIR/reports/api-tests-$TIMESTAMP.json" 2>&1 | tee "$test_output"
            exit_code=$?
        fi
    else
        # Basic curl tests
        log "Running basic API health checks..."
        
        # Test backend health
        if curl -f http://localhost:8080/health >/dev/null 2>&1; then
            log "Backend health check passed"
        else
            error "Backend health check failed"
            exit_code=1
        fi
        
        # Test frontend
        if curl -f http://localhost:3000 >/dev/null 2>&1; then
            log "Frontend health check passed"
        else
            error "Frontend health check failed"
            exit_code=1
        fi
    fi
    
    # Test contract interactions
    if [ -d "soroban" ]; then
        log "Testing contract interactions..."
        cd soroban
        
        # Run integration tests that interact with deployed contracts
        cargo test --test integration_test 2>&1 | tee -a "../$test_output"
        local contract_exit_code=$?
        if [ $exit_code -eq 0 ]; then
            exit_code=$contract_exit_code
        fi
        
        cd ..
    fi
    
    if [ $exit_code -eq 0 ]; then
        success "Integration tests passed"
        PASSED_TESTS=$((PASSED_TESTS + 1))
    else
        error "Integration tests failed"
        FAILED_TESTS=$((FAILED_TESTS + 1))
    fi
    
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
    return $exit_code
}

# Run end-to-end tests
test_e2e() {
    log "Running end-to-end tests..."
    
    local exit_code=0
    local test_output="$TESTS_DIR/logs/e2e-$TIMESTAMP.log"
    
    # Check if Playwright is available
    if [ -d "frontend" ] && [ -f "frontend/playwright.config.ts" ]; then
        log "Running Playwright E2E tests..."
        cd frontend
        
        # Install Playwright if needed
        if [ ! -d "node_modules/@playwright" ]; then
            log "Installing Playwright..."
            npx playwright install
        fi
        
        # Run E2E tests
        npx playwright test --reporter=json --output-dir="../$TESTS_DIR/reports/e2e-$TIMESTAMP" 2>&1 | tee "$test_output"
        exit_code=$?
        
        cd ..
    else
        warn "Playwright not configured, skipping E2E tests"
        SKIPPED_TESTS=$((SKIPPED_TESTS + 1))
        return 0
    fi
    
    if [ $exit_code -eq 0 ]; then
        success "E2E tests passed"
        PASSED_TESTS=$((PASSED_TESTS + 1))
    else
        error "E2E tests failed"
        FAILED_TESTS=$((FAILED_TESTS + 1))
    fi
    
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
    return $exit_code
}

# Generate test report
generate_test_report() {
    log "Generating test report..."
    
    local report_file="$TESTS_DIR/test-report-$TIMESTAMP.json"
    local success_rate=0
    
    if [ $TOTAL_TESTS -gt 0 ]; then
        success_rate=$(echo "scale=2; $PASSED_TESTS * 100 / $TOTAL_TESTS" | bc -l 2>/dev/null || echo "0")
    fi
    
    cat > "$report_file" << EOF
{
  "test_run_id": "$TIMESTAMP",
  "component": "$COMPONENT",
  "test_type": "$TEST_TYPE",
  "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "git_commit": "$(git rev-parse HEAD 2>/dev/null || echo 'unknown')",
  "git_branch": "$(git rev-parse --abbrev-ref HEAD 2>/dev/null || echo 'unknown')",
  "summary": {
    "total_tests": $TOTAL_TESTS,
    "passed": $PASSED_TESTS,
    "failed": $FAILED_TESTS,
    "skipped": $SKIPPED_TESTS,
    "success_rate": "$success_rate%"
  },
  "coverage_enabled": $COVERAGE,
  "parallel_execution": $PARALLEL,
  "reports_directory": "$TESTS_DIR/reports",
  "logs_directory": "$TESTS_DIR/logs"
}
EOF
    
    success "Test report generated: $report_file"
    
    # Print summary
    echo ""
    echo "========================================="
    echo "           TEST SUMMARY"
    echo "========================================="
    echo "Total Tests:    $TOTAL_TESTS"
    echo "Passed:         $PASSED_TESTS"
    echo "Failed:         $FAILED_TESTS"
    echo "Skipped:        $SKIPPED_TESTS"
    echo "Success Rate:   $success_rate%"
    echo "========================================="
}

# Cleanup test environment
cleanup_test_env() {
    log "Cleaning up test environment..."
    
    # Stop test infrastructure
    if [ -f "docker-compose.test.yml" ]; then
        docker-compose -f docker-compose.test.yml down
    fi
    
    # Archive test results if they exist
    if [ -d "$TESTS_DIR" ] && [ "$(ls -A $TESTS_DIR)" ]; then
        log "Archiving test results..."
        tar -czf "test-results-$TIMESTAMP.tar.gz" "$TESTS_DIR"
        success "Test results archived: test-results-$TIMESTAMP.tar.gz"
    fi
}

# Main test function
main() {
    log "Starting test execution..."
    log "Component: $COMPONENT"
    log "Test Type: $TEST_TYPE"
    log "Parallel: $PARALLEL"
    log "Coverage: $COVERAGE"
    log "Test Run ID: $TIMESTAMP"
    
    init_test_env
    
    local overall_exit_code=0
    
    if [ "$PARALLEL" = "true" ] && [ "$COMPONENT" = "all" ]; then
        log "Running tests in parallel..."
        
        # Run tests in parallel
        test_frontend &
        local frontend_pid=$!
        
        test_backend &
        local backend_pid=$!
        
        test_soroban &
        local soroban_pid=$!
        
        # Wait for all tests to complete
        wait $frontend_pid
        local frontend_exit=$?
        
        wait $backend_pid
        local backend_exit=$?
        
        wait $soroban_pid
        local soroban_exit=$?
        
        # Check if any tests failed
        if [ $frontend_exit -ne 0 ] || [ $backend_exit -ne 0 ] || [ $soroban_exit -ne 0 ]; then
            overall_exit_code=1
        fi
        
        # Run integration tests after component tests
        if [ "$TEST_TYPE" = "integration" ] || [ "$TEST_TYPE" = "all" ]; then
            test_integration
            if [ $? -ne 0 ]; then
                overall_exit_code=1
            fi
        fi
        
        # Run E2E tests
        if [ "$TEST_TYPE" = "e2e" ] || [ "$TEST_TYPE" = "all" ]; then
            test_e2e
            if [ $? -ne 0 ]; then
                overall_exit_code=1
            fi
        fi
    else
        # Run tests sequentially
        case $COMPONENT in
            frontend)
                test_frontend
                overall_exit_code=$?
                ;;
            backend)
                test_backend
                overall_exit_code=$?
                ;;
            soroban)
                test_soroban
                overall_exit_code=$?
                ;;
            integration)
                test_integration
                overall_exit_code=$?
                ;;
            e2e)
                test_e2e
                overall_exit_code=$?
                ;;
            all)
                test_frontend
                if [ $? -ne 0 ]; then overall_exit_code=1; fi
                
                test_backend
                if [ $? -ne 0 ]; then overall_exit_code=1; fi
                
                test_soroban
                if [ $? -ne 0 ]; then overall_exit_code=1; fi
                
                if [ "$TEST_TYPE" = "integration" ] || [ "$TEST_TYPE" = "all" ]; then
                    test_integration
                    if [ $? -ne 0 ]; then overall_exit_code=1; fi
                fi
                
                if [ "$TEST_TYPE" = "e2e" ] || [ "$TEST_TYPE" = "all" ]; then
                    test_e2e
                    if [ $? -ne 0 ]; then overall_exit_code=1; fi
                fi
                ;;
            *)
                error "Unknown component: $COMPONENT"
                echo "Available components: frontend, backend, soroban, integration, e2e, all"
                exit 1
                ;;
        esac
    fi
    
    generate_test_report
    cleanup_test_env
    
    if [ $overall_exit_code -eq 0 ]; then
        success "All tests completed successfully!"
    else
        error "Some tests failed. Check the test report for details."
    fi
    
    exit $overall_exit_code
}

# Show usage if help requested
if [ "$1" = "--help" ] || [ "$1" = "-h" ]; then
    echo "Usage: $0 [component] [test-type] [--parallel] [--coverage]"
    echo ""
    echo "Components:"
    echo "  frontend     - Test React frontend only"
    echo "  backend      - Test Loco.rs backend only"
    echo "  soroban      - Test Soroban contracts only"
    echo "  integration  - Test cross-component integration"
    echo "  e2e          - Test end-to-end user workflows"
    echo "  all          - Test all components (default)"
    echo ""
    echo "Test Types:"
    echo "  unit         - Unit tests only"
    echo "  integration  - Integration tests only"
    echo "  e2e          - End-to-end tests only"
    echo "  all          - All test types (default)"
    echo ""
    echo "Options:"
    echo "  --parallel   - Run component tests in parallel"
    echo "  --coverage   - Generate code coverage reports"
    echo ""
    echo "Examples:"
    echo "  $0                           # Run all tests"
    echo "  $0 frontend unit             # Run frontend unit tests"
    echo "  $0 all integration --parallel # Run integration tests in parallel"
    echo "  $0 backend all --coverage    # Run backend tests with coverage"
    exit 0
fi

# Parse options
for arg in "$@"; do
    case $arg in
        --parallel)
            PARALLEL=true
            ;;
        --coverage)
            COVERAGE=true
            ;;
    esac
done

# Run main function
main "$@"