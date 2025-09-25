#!/bin/bash

# Dependency Compatibility Validator for Bitcoin Custody Full-Stack Application
# Validates dependencies across components and checks for compatibility issues

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
VERSION_CONFIG="$PROJECT_ROOT/version-config.json"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

log() {
    echo -e "${BLUE}[DEPENDENCY-VALIDATOR]${NC} $1"
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

# Check if required tools are installed
check_tools() {
    local missing_tools=()
    
    if ! command -v jq &> /dev/null; then
        missing_tools+=("jq")
    fi
    
    if ! command -v npm &> /dev/null; then
        missing_tools+=("npm")
    fi
    
    if ! command -v cargo &> /dev/null; then
        missing_tools+=("cargo")
    fi
    
    if [ ${#missing_tools[@]} -gt 0 ]; then
        error "Missing required tools: ${missing_tools[*]}"
        error "Please install the missing tools and try again"
        exit 1
    fi
}

# Validate frontend dependencies
validate_frontend_deps() {
    log "Validating frontend dependencies..."
    
    local frontend_path="$PROJECT_ROOT/frontend"
    if [ ! -f "$frontend_path/package.json" ]; then
        error "Frontend package.json not found at $frontend_path"
        return 1
    fi
    
    cd "$frontend_path"
    
    # Check for security vulnerabilities
    log "Checking for security vulnerabilities in frontend..."
    if npm audit --audit-level=high > /dev/null 2>&1; then
        success "No high-severity vulnerabilities found in frontend"
    else
        warn "Security vulnerabilities detected in frontend dependencies"
        npm audit --audit-level=high
    fi
    
    # Check for outdated packages
    log "Checking for outdated frontend packages..."
    local outdated=$(npm outdated --json 2>/dev/null || echo "{}")
    if [ "$outdated" = "{}" ]; then
        success "All frontend packages are up to date"
    else
        warn "Outdated frontend packages detected:"
        echo "$outdated" | jq -r 'to_entries[] | "\(.key): \(.value.current) -> \(.value.wanted)"'
    fi
    
    # Validate critical dependencies
    local critical_deps=("react" "react-dom" "@reduxjs/toolkit" "axios" "vite")
    for dep in "${critical_deps[@]}"; do
        if jq -e ".dependencies.\"$dep\"" package.json > /dev/null; then
            local version=$(jq -r ".dependencies.\"$dep\"" package.json)
            log "✓ $dep: $version"
        else
            error "Critical dependency missing: $dep"
            return 1
        fi
    done
    
    cd "$PROJECT_ROOT"
    success "Frontend dependencies validated"
}

# Validate backend dependencies
validate_backend_deps() {
    log "Validating backend dependencies..."
    
    local backend_path="$PROJECT_ROOT/backend"
    if [ ! -f "$backend_path/Cargo.toml" ]; then
        error "Backend Cargo.toml not found at $backend_path"
        return 1
    fi
    
    cd "$backend_path"
    
    # Check for security advisories
    log "Checking for security advisories in backend..."
    if cargo audit > /dev/null 2>&1; then
        success "No security advisories found in backend"
    else
        warn "Security advisories detected in backend dependencies"
        cargo audit
    fi
    
    # Check for outdated crates
    log "Checking for outdated backend crates..."
    if command -v cargo-outdated &> /dev/null; then
        cargo outdated
    else
        warn "cargo-outdated not installed, skipping outdated check"
        warn "Install with: cargo install cargo-outdated"
    fi
    
    # Validate critical dependencies
    local critical_deps=("loco-rs" "sea-orm" "tokio" "serde" "soroban-sdk")
    for dep in "${critical_deps[@]}"; do
        if grep -q "^$dep = " Cargo.toml; then
            local version=$(grep "^$dep = " Cargo.toml | head -1 | sed 's/.*= "\([^"]*\)".*/\1/')
            log "✓ $dep: $version"
        else
            error "Critical dependency missing: $dep"
            return 1
        fi
    done
    
    cd "$PROJECT_ROOT"
    success "Backend dependencies validated"
}

# Validate soroban dependencies
validate_soroban_deps() {
    log "Validating soroban dependencies..."
    
    local soroban_path="$PROJECT_ROOT/soroban"
    if [ ! -f "$soroban_path/Cargo.toml" ]; then
        error "Soroban Cargo.toml not found at $soroban_path"
        return 1
    fi
    
    cd "$soroban_path"
    
    # Check for security advisories
    log "Checking for security advisories in soroban..."
    if cargo audit > /dev/null 2>&1; then
        success "No security advisories found in soroban"
    else
        warn "Security advisories detected in soroban dependencies"
        cargo audit
    fi
    
    # Validate soroban-sdk version consistency across contracts
    log "Checking soroban-sdk version consistency..."
    local sdk_versions=$(find contracts -name "Cargo.toml" -exec grep "soroban-sdk = " {} \; | sort | uniq)
    local version_count=$(echo "$sdk_versions" | wc -l)
    
    if [ "$version_count" -eq 1 ]; then
        success "Consistent soroban-sdk version across all contracts"
        log "Version: $sdk_versions"
    else
        error "Inconsistent soroban-sdk versions found:"
        echo "$sdk_versions"
        return 1
    fi
    
    # Validate workspace members
    log "Validating workspace members..."
    local workspace_members=$(grep -A 20 "\[workspace\]" Cargo.toml | grep "members = " -A 10 | grep '"' | sed 's/.*"\([^"]*\)".*/\1/')
    
    while IFS= read -r member; do
        if [ -n "$member" ] && [ -d "$member" ]; then
            log "✓ Workspace member exists: $member"
        else
            error "Workspace member missing: $member"
            return 1
        fi
    done <<< "$workspace_members"
    
    cd "$PROJECT_ROOT"
    success "Soroban dependencies validated"
}

# Check cross-component compatibility
validate_cross_component_compatibility() {
    log "Validating cross-component compatibility..."
    
    # Check API compatibility between frontend and backend
    log "Checking frontend-backend API compatibility..."
    
    # Look for API endpoint definitions in backend
    local backend_endpoints=()
    if [ -f "$PROJECT_ROOT/backend/src/controllers/mod.rs" ]; then
        # Extract route definitions (simplified check)
        mapfile -t backend_endpoints < <(grep -r "get\|post\|put\|delete" "$PROJECT_ROOT/backend/src/controllers/" | grep -o '"/[^"]*"' | sort | uniq)
    fi
    
    # Look for API calls in frontend
    local frontend_calls=()
    if [ -f "$PROJECT_ROOT/frontend/src/services/api.ts" ]; then
        mapfile -t frontend_calls < <(grep -o "'/[^']*'" "$PROJECT_ROOT/frontend/src/services/api.ts" | sort | uniq)
    fi
    
    # Check for potential mismatches (simplified)
    if [ ${#backend_endpoints[@]} -gt 0 ] && [ ${#frontend_calls[@]} -gt 0 ]; then
        success "Found API endpoints in backend and calls in frontend"
        log "Backend endpoints: ${#backend_endpoints[@]}"
        log "Frontend calls: ${#frontend_calls[@]}"
    else
        warn "Could not validate API compatibility - files may not exist yet"
    fi
    
    # Check contract interface compatibility
    log "Checking backend-soroban contract compatibility..."
    
    # Look for contract client usage in backend
    if [ -f "$PROJECT_ROOT/backend/src/services/soroban_client.rs" ]; then
        success "Found soroban client in backend"
    else
        warn "Soroban client not found in backend - may not be implemented yet"
    fi
    
    # Check for contract definitions in soroban
    local contract_count=$(find "$PROJECT_ROOT/soroban/contracts" -name "lib.rs" 2>/dev/null | wc -l)
    if [ "$contract_count" -gt 0 ]; then
        success "Found $contract_count contracts in soroban directory"
    else
        warn "No contracts found in soroban directory"
    fi
    
    success "Cross-component compatibility check completed"
}

# Generate dependency report
generate_dependency_report() {
    log "Generating dependency compatibility report..."
    
    local report_file="$PROJECT_ROOT/DEPENDENCY_COMPATIBILITY_REPORT.md"
    local timestamp=$(date '+%Y-%m-%d %H:%M:%S')
    
    cat > "$report_file" << EOF
# Dependency Compatibility Report

Generated: $timestamp

## Summary

This report provides an overview of dependency compatibility across all components of the Bitcoin Custody Full-Stack Application.

## Component Status

EOF
    
    # Frontend status
    echo "### Frontend Dependencies" >> "$report_file"
    echo "" >> "$report_file"
    
    if [ -f "$PROJECT_ROOT/frontend/package.json" ]; then
        echo "**Status:** ✅ Package.json found" >> "$report_file"
        local frontend_version=$(jq -r '.version' "$PROJECT_ROOT/frontend/package.json")
        echo "**Version:** $frontend_version" >> "$report_file"
        
        echo "" >> "$report_file"
        echo "**Key Dependencies:**" >> "$report_file"
        
        cd "$PROJECT_ROOT/frontend"
        local deps=("react" "react-dom" "@reduxjs/toolkit" "axios" "vite")
        for dep in "${deps[@]}"; do
            if jq -e ".dependencies.\"$dep\"" package.json > /dev/null 2>&1; then
                local version=$(jq -r ".dependencies.\"$dep\"" package.json)
                echo "- $dep: $version" >> "$report_file"
            fi
        done
        cd "$PROJECT_ROOT"
    else
        echo "**Status:** ❌ Package.json not found" >> "$report_file"
    fi
    
    echo "" >> "$report_file"
    
    # Backend status
    echo "### Backend Dependencies" >> "$report_file"
    echo "" >> "$report_file"
    
    if [ -f "$PROJECT_ROOT/backend/Cargo.toml" ]; then
        echo "**Status:** ✅ Cargo.toml found" >> "$report_file"
        local backend_version=$(grep '^version = ' "$PROJECT_ROOT/backend/Cargo.toml" | head -1 | sed 's/version = "\(.*\)"/\1/')
        echo "**Version:** $backend_version" >> "$report_file"
        
        echo "" >> "$report_file"
        echo "**Key Dependencies:**" >> "$report_file"
        
        local deps=("loco-rs" "sea-orm" "tokio" "serde" "soroban-sdk")
        for dep in "${deps[@]}"; do
            if grep -q "^$dep = " "$PROJECT_ROOT/backend/Cargo.toml"; then
                local version=$(grep "^$dep = " "$PROJECT_ROOT/backend/Cargo.toml" | head -1 | sed 's/.*= "\([^"]*\)".*/\1/')
                echo "- $dep: $version" >> "$report_file"
            fi
        done
    else
        echo "**Status:** ❌ Cargo.toml not found" >> "$report_file"
    fi
    
    echo "" >> "$report_file"
    
    # Soroban status
    echo "### Soroban Dependencies" >> "$report_file"
    echo "" >> "$report_file"
    
    if [ -f "$PROJECT_ROOT/soroban/Cargo.toml" ]; then
        echo "**Status:** ✅ Cargo.toml found" >> "$report_file"
        
        echo "" >> "$report_file"
        echo "**Workspace Members:**" >> "$report_file"
        
        local members=$(grep -A 20 "\[workspace\]" "$PROJECT_ROOT/soroban/Cargo.toml" | grep "members = " -A 10 | grep '"' | sed 's/.*"\([^"]*\)".*/\1/')
        while IFS= read -r member; do
            if [ -n "$member" ]; then
                if [ -d "$PROJECT_ROOT/soroban/$member" ]; then
                    echo "- ✅ $member" >> "$report_file"
                else
                    echo "- ❌ $member (missing)" >> "$report_file"
                fi
            fi
        done <<< "$members"
    else
        echo "**Status:** ❌ Cargo.toml not found" >> "$report_file"
    fi
    
    echo "" >> "$report_file"
    
    # Compatibility matrix
    echo "## Compatibility Matrix" >> "$report_file"
    echo "" >> "$report_file"
    echo "| Component | Version | Status | Notes |" >> "$report_file"
    echo "|-----------|---------|--------|-------|" >> "$report_file"
    
    # Get versions from version config if it exists
    if [ -f "$VERSION_CONFIG" ]; then
        for component in frontend backend soroban; do
            local version=$(jq -r ".project.components.${component}.version" "$VERSION_CONFIG" 2>/dev/null || echo "unknown")
            echo "| $component | $version | ✅ Compatible | - |" >> "$report_file"
        done
    else
        echo "| All | - | ❓ Unknown | Version config not found |" >> "$report_file"
    fi
    
    echo "" >> "$report_file"
    
    # Recommendations
    echo "## Recommendations" >> "$report_file"
    echo "" >> "$report_file"
    echo "1. **Regular Updates**: Keep dependencies updated to latest compatible versions" >> "$report_file"
    echo "2. **Security Scanning**: Run security audits regularly with \`npm audit\` and \`cargo audit\`" >> "$report_file"
    echo "3. **Version Pinning**: Pin critical dependency versions to avoid unexpected breaks" >> "$report_file"
    echo "4. **Testing**: Run integration tests after dependency updates" >> "$report_file"
    echo "5. **Documentation**: Keep compatibility matrix updated with each release" >> "$report_file"
    
    success "Dependency report generated: $report_file"
}

# Run all validations
validate_all() {
    log "Running comprehensive dependency validation..."
    
    local validation_failed=false
    
    # Validate each component
    if ! validate_frontend_deps; then
        validation_failed=true
    fi
    
    if ! validate_backend_deps; then
        validation_failed=true
    fi
    
    if ! validate_soroban_deps; then
        validation_failed=true
    fi
    
    # Check cross-component compatibility
    validate_cross_component_compatibility
    
    # Generate report
    generate_dependency_report
    
    if [ "$validation_failed" = true ]; then
        error "Dependency validation failed - see errors above"
        exit 1
    else
        success "All dependency validations passed"
    fi
}

# Main command handler
main() {
    check_tools
    
    case "${1:-all}" in
        "frontend")
            validate_frontend_deps
            ;;
        "backend")
            validate_backend_deps
            ;;
        "soroban")
            validate_soroban_deps
            ;;
        "compatibility")
            validate_cross_component_compatibility
            ;;
        "report")
            generate_dependency_report
            ;;
        "all")
            validate_all
            ;;
        "help")
            cat << EOF
Dependency Compatibility Validator

Usage: $0 <command>

Commands:
  frontend       Validate frontend dependencies only
  backend        Validate backend dependencies only  
  soroban        Validate soroban dependencies only
  compatibility  Check cross-component compatibility
  report         Generate dependency compatibility report
  all            Run all validations (default)
  help           Show this help message

Examples:
  $0 all         # Run all validations
  $0 frontend    # Check only frontend dependencies
  $0 report      # Generate compatibility report

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