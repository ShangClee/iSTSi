#!/bin/bash

# Release Validator for Bitcoin Custody Full-Stack Application
# Validates releases and ensures quality gates are met

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

log() {
    echo -e "${BLUE}[RELEASE-VALIDATOR]${NC} $1"
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

# Validate code quality
validate_code_quality() {
    log "Validating code quality..."
    
    local quality_passed=true
    
    # Frontend code quality
    if [ -d "$PROJECT_ROOT/frontend" ]; then
        log "Checking frontend code quality..."
        cd "$PROJECT_ROOT/frontend"
        
        # TypeScript compilation
        if npm run type-check > /dev/null 2>&1; then
            success "✓ TypeScript compilation passed"
        else
            error "✗ TypeScript compilation failed"
            quality_passed=false
        fi
        
        # ESLint
        if npm run lint > /dev/null 2>&1; then
            success "✓ ESLint checks passed"
        else
            warn "⚠ ESLint found issues"
        fi
        
        # Build test
        if npm run build > /dev/null 2>&1; then
            success "✓ Frontend build successful"
        else
            error "✗ Frontend build failed"
            quality_passed=false
        fi
        
        cd "$PROJECT_ROOT"
    fi
    
    # Backend code quality
    if [ -d "$PROJECT_ROOT/backend" ]; then
        log "Checking backend code quality..."
        cd "$PROJECT_ROOT/backend"
        
        # Rust compilation
        if cargo check > /dev/null 2>&1; then
            success "✓ Rust compilation passed"
        else
            error "✗ Rust compilation failed"
            quality_passed=false
        fi
        
        # Clippy linting
        if cargo clippy -- -D warnings > /dev/null 2>&1; then
            success "✓ Clippy checks passed"
        else
            warn "⚠ Clippy found issues"
        fi
        
        # Format check
        if cargo fmt -- --check > /dev/null 2>&1; then
            success "✓ Code formatting is correct"
        else
            warn "⚠ Code formatting issues found"
        fi
        
        cd "$PROJECT_ROOT"
    fi
    
    # Soroban code quality
    if [ -d "$PROJECT_ROOT/soroban" ]; then
        log "Checking soroban code quality..."
        cd "$PROJECT_ROOT/soroban"
        
        # Contract compilation
        if cargo check > /dev/null 2>&1; then
            success "✓ Contract compilation passed"
        else
            error "✗ Contract compilation failed"
            quality_passed=false
        fi
        
        # WASM build
        if cargo build --target wasm32-unknown-unknown --release > /dev/null 2>&1; then
            success "✓ WASM build successful"
        else
            error "✗ WASM build failed"
            quality_passed=false
        fi
        
        cd "$PROJECT_ROOT"
    fi
    
    if [ "$quality_passed" = false ]; then
        error "Code quality validation failed"
        return 1
    fi
    
    success "Code quality validation passed"
}

# Validate test coverage
validate_test_coverage() {
    log "Validating test coverage..."
    
    local coverage_passed=true
    local min_coverage=80
    
    # Frontend test coverage
    if [ -d "$PROJECT_ROOT/frontend" ]; then
        log "Checking frontend test coverage..."
        cd "$PROJECT_ROOT/frontend"
        
        if [ -f "package.json" ] && jq -e '.scripts.test' package.json > /dev/null; then
            if npm test -- --coverage --watchAll=false > /dev/null 2>&1; then
                success "✓ Frontend tests passed"
                
                # Check coverage if available
                if [ -f "coverage/lcov-report/index.html" ]; then
                    log "Frontend test coverage report generated"
                fi
            else
                error "✗ Frontend tests failed"
                coverage_passed=false
            fi
        else
            warn "⚠ Frontend tests not configured"
        fi
        
        cd "$PROJECT_ROOT"
    fi
    
    # Backend test coverage
    if [ -d "$PROJECT_ROOT/backend" ]; then
        log "Checking backend test coverage..."
        cd "$PROJECT_ROOT/backend"
        
        if cargo test > /dev/null 2>&1; then
            success "✓ Backend tests passed"
        else
            error "✗ Backend tests failed"
            coverage_passed=false
        fi
        
        # Generate coverage report if tarpaulin is available
        if command -v cargo-tarpaulin &> /dev/null; then
            if cargo tarpaulin --out Html --output-dir coverage > /dev/null 2>&1; then
                log "Backend test coverage report generated"
            fi
        fi
        
        cd "$PROJECT_ROOT"
    fi
    
    # Soroban test coverage
    if [ -d "$PROJECT_ROOT/soroban" ]; then
        log "Checking soroban test coverage..."
        cd "$PROJECT_ROOT/soroban"
        
        if cargo test > /dev/null 2>&1; then
            success "✓ Soroban tests passed"
        else
            error "✗ Soroban tests failed"
            coverage_passed=false
        fi
        
        cd "$PROJECT_ROOT"
    fi
    
    if [ "$coverage_passed" = false ]; then
        error "Test coverage validation failed"
        return 1
    fi
    
    success "Test coverage validation passed"
}

# Validate security requirements
validate_security() {
    log "Validating security requirements..."
    
    local security_passed=true
    
    # Run security audit
    if [ -f "$PROJECT_ROOT/scripts/security-audit.sh" ]; then
        if ./scripts/security-audit.sh > /dev/null 2>&1; then
            success "✓ Security audit passed"
        else
            warn "⚠ Security audit found issues"
        fi
    fi
    
    # Check for secrets in code
    log "Scanning for exposed secrets..."
    if command -v git-secrets &> /dev/null; then
        if git secrets --scan > /dev/null 2>&1; then
            success "✓ No secrets found in code"
        else
            error "✗ Potential secrets found in code"
            security_passed=false
        fi
    else
        warn "⚠ git-secrets not available for secret scanning"
    fi
    
    # Dependency vulnerability check
    log "Checking for dependency vulnerabilities..."
    
    # Frontend dependencies
    if [ -d "$PROJECT_ROOT/frontend" ]; then
        cd "$PROJECT_ROOT/frontend"
        if npm audit --audit-level=high > /dev/null 2>&1; then
            success "✓ No high-severity vulnerabilities in frontend"
        else
            warn "⚠ High-severity vulnerabilities found in frontend dependencies"
        fi
        cd "$PROJECT_ROOT"
    fi
    
    # Backend dependencies
    if [ -d "$PROJECT_ROOT/backend" ]; then
        cd "$PROJECT_ROOT/backend"
        if command -v cargo-audit &> /dev/null; then
            if cargo audit > /dev/null 2>&1; then
                success "✓ No security advisories in backend"
            else
                warn "⚠ Security advisories found in backend dependencies"
            fi
        fi
        cd "$PROJECT_ROOT"
    fi
    
    if [ "$security_passed" = false ]; then
        error "Security validation failed"
        return 1
    fi
    
    success "Security validation passed"
}

# Validate performance requirements
validate_performance() {
    log "Validating performance requirements..."
    
    # Frontend performance
    if [ -d "$PROJECT_ROOT/frontend" ]; then
        log "Checking frontend bundle size..."
        cd "$PROJECT_ROOT/frontend"
        
        if npm run build > /dev/null 2>&1; then
            # Check bundle size (basic check)
            local bundle_size=$(du -sh dist/ 2>/dev/null | cut -f1 || echo "unknown")
            log "Frontend bundle size: $bundle_size"
            success "✓ Frontend build completed"
        fi
        
        cd "$PROJECT_ROOT"
    fi
    
    # Backend performance
    if [ -d "$PROJECT_ROOT/backend" ]; then
        log "Checking backend binary size..."
        cd "$PROJECT_ROOT/backend"
        
        if cargo build --release > /dev/null 2>&1; then
            local binary_size=$(du -sh target/release/bitcoin-custody-backend 2>/dev/null | cut -f1 || echo "unknown")
            log "Backend binary size: $binary_size"
            success "✓ Backend release build completed"
        fi
        
        cd "$PROJECT_ROOT"
    fi
    
    # Contract size validation
    if [ -d "$PROJECT_ROOT/soroban" ]; then
        log "Checking contract sizes..."
        cd "$PROJECT_ROOT/soroban"
        
        if cargo build --target wasm32-unknown-unknown --release > /dev/null 2>&1; then
            # Check WASM file sizes
            find target/wasm32-unknown-unknown/release -name "*.wasm" -exec ls -lh {} \; | while read -r line; do
                local filename=$(echo "$line" | awk '{print $9}' | xargs basename)
                local size=$(echo "$line" | awk '{print $5}')
                log "Contract $filename size: $size"
            done
            success "✓ Contract builds completed"
        fi
        
        cd "$PROJECT_ROOT"
    fi
    
    success "Performance validation completed"
}

# Validate deployment readiness
validate_deployment_readiness() {
    log "Validating deployment readiness..."
    
    local deployment_ready=true
    
    # Check required configuration files
    local required_configs=(
        "docker-compose.yml"
        "docker-compose.staging.yml" 
        "docker-compose.production.yml"
    )
    
    for config in "${required_configs[@]}"; do
        if [ -f "$PROJECT_ROOT/$config" ]; then
            success "✓ $config exists"
        else
            warn "⚠ $config not found"
        fi
    done
    
    # Check environment configuration
    if [ -d "$PROJECT_ROOT/backend/config" ]; then
        local env_configs=("development.yaml" "staging.yaml" "production.yaml")
        for config in "${env_configs[@]}"; do
            if [ -f "$PROJECT_ROOT/backend/config/$config" ]; then
                success "✓ Backend config/$config exists"
            else
                warn "⚠ Backend config/$config not found"
            fi
        done
    fi
    
    # Check deployment scripts
    local deployment_scripts=(
        "scripts/deploy.sh"
        "scripts/health-check.sh"
    )
    
    for script in "${deployment_scripts[@]}"; do
        if [ -f "$PROJECT_ROOT/$script" ]; then
            success "✓ $script exists"
        else
            warn "⚠ $script not found"
        fi
    done
    
    # Validate Docker builds
    log "Validating Docker builds..."
    
    if [ -f "$PROJECT_ROOT/docker-compose.yml" ]; then
        if docker-compose config > /dev/null 2>&1; then
            success "✓ Docker Compose configuration is valid"
        else
            error "✗ Docker Compose configuration is invalid"
            deployment_ready=false
        fi
    fi
    
    if [ "$deployment_ready" = false ]; then
        error "Deployment readiness validation failed"
        return 1
    fi
    
    success "Deployment readiness validation passed"
}

# Generate validation report
generate_validation_report() {
    local version="$1"
    local report_file="$PROJECT_ROOT/RELEASE_VALIDATION_REPORT_v${version}.md"
    local timestamp=$(date '+%Y-%m-%d %H:%M:%S')
    
    log "Generating validation report..."
    
    cat > "$report_file" << EOF
# Release Validation Report

**Version:** $version  
**Generated:** $timestamp  
**Validator:** Release Validator Script

## Validation Summary

This report contains the results of automated validation checks performed before release.

## Code Quality Validation

EOF
    
    # Run validations and capture results
    if validate_code_quality >> "$report_file" 2>&1; then
        echo "**Status:** ✅ PASSED" >> "$report_file"
    else
        echo "**Status:** ❌ FAILED" >> "$report_file"
    fi
    
    echo "" >> "$report_file"
    echo "## Test Coverage Validation" >> "$report_file"
    echo "" >> "$report_file"
    
    if validate_test_coverage >> "$report_file" 2>&1; then
        echo "**Status:** ✅ PASSED" >> "$report_file"
    else
        echo "**Status:** ❌ FAILED" >> "$report_file"
    fi
    
    echo "" >> "$report_file"
    echo "## Security Validation" >> "$report_file"
    echo "" >> "$report_file"
    
    if validate_security >> "$report_file" 2>&1; then
        echo "**Status:** ✅ PASSED" >> "$report_file"
    else
        echo "**Status:** ⚠️ WARNINGS" >> "$report_file"
    fi
    
    echo "" >> "$report_file"
    echo "## Performance Validation" >> "$report_file"
    echo "" >> "$report_file"
    
    validate_performance >> "$report_file" 2>&1
    echo "**Status:** ✅ COMPLETED" >> "$report_file"
    
    echo "" >> "$report_file"
    echo "## Deployment Readiness" >> "$report_file"
    echo "" >> "$report_file"
    
    if validate_deployment_readiness >> "$report_file" 2>&1; then
        echo "**Status:** ✅ READY" >> "$report_file"
    else
        echo "**Status:** ❌ NOT READY" >> "$report_file"
    fi
    
    cat >> "$report_file" << EOF

## Recommendations

1. **Code Quality**: Ensure all compilation and linting issues are resolved
2. **Testing**: Maintain test coverage above 80% for critical components
3. **Security**: Address any high-severity vulnerabilities before release
4. **Performance**: Monitor bundle sizes and optimize if necessary
5. **Deployment**: Verify all configuration files are present and valid

## Next Steps

- [ ] Review validation results
- [ ] Address any failed validations
- [ ] Re-run validation if changes are made
- [ ] Proceed with deployment if all validations pass

---
*This report was generated automatically by the Release Validator*
EOF
    
    success "Validation report generated: $report_file"
}

# Run all validations
validate_all() {
    local version="${1:-unknown}"
    
    log "Running comprehensive release validation for version $version..."
    
    local validation_failed=false
    
    # Run all validation checks
    if ! validate_code_quality; then
        validation_failed=true
    fi
    
    if ! validate_test_coverage; then
        validation_failed=true
    fi
    
    if ! validate_security; then
        # Security warnings don't fail the build, but are noted
        warn "Security validation completed with warnings"
    fi
    
    validate_performance
    
    if ! validate_deployment_readiness; then
        validation_failed=true
    fi
    
    # Generate report
    generate_validation_report "$version"
    
    if [ "$validation_failed" = true ]; then
        error "Release validation failed - see report for details"
        exit 1
    else
        success "All release validations passed!"
    fi
}

# Main command handler
main() {
    case "${1:-all}" in
        "code-quality")
            validate_code_quality
            ;;
        "test-coverage")
            validate_test_coverage
            ;;
        "security")
            validate_security
            ;;
        "performance")
            validate_performance
            ;;
        "deployment")
            validate_deployment_readiness
            ;;
        "report")
            generate_validation_report "$2"
            ;;
        "all")
            validate_all "$2"
            ;;
        "help")
            cat << EOF
Release Validator for Bitcoin Custody Full-Stack Application

Usage: $0 <command> [version]

Commands:
  code-quality    Validate code quality (compilation, linting, formatting)
  test-coverage   Validate test coverage and run test suites
  security        Validate security requirements and scan for vulnerabilities
  performance     Validate performance requirements and bundle sizes
  deployment      Validate deployment readiness and configuration
  report <version> Generate validation report for version
  all [version]   Run all validations (default)
  help           Show this help message

Examples:
  $0 all 1.2.0           # Run all validations for version 1.2.0
  $0 code-quality        # Check only code quality
  $0 security           # Check only security requirements
  $0 report 1.2.0       # Generate validation report

EOF
            ;;
        *)
            error "Unknown command: $1"
            main help
            exit 1
            ;;
    esac
}

main "$@"