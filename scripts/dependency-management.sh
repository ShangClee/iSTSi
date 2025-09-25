#!/bin/bash
# Dependency Management and Security Scanning Script
# This script provides comprehensive dependency management across all components

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Function to check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Function to audit frontend dependencies
audit_frontend() {
    print_status "Auditing frontend dependencies..."
    
    if [ ! -d "frontend" ]; then
        print_error "Frontend directory not found"
        return 1
    fi
    
    cd frontend
    
    # Check if npm is available
    if ! command_exists npm; then
        print_error "npm not found. Please install Node.js and npm"
        return 1
    fi
    
    # Run npm audit
    print_status "Running npm audit..."
    if npm audit --audit-level=moderate; then
        print_success "No moderate or high severity vulnerabilities found in frontend"
    else
        print_warning "Vulnerabilities found in frontend dependencies"
        print_status "Attempting to fix automatically..."
        npm audit fix --audit-level=moderate
    fi
    
    # Check for outdated packages
    print_status "Checking for outdated frontend packages..."
    npm outdated || true
    
    cd ..
}

# Function to audit backend dependencies
audit_backend() {
    print_status "Auditing backend dependencies..."
    
    if [ ! -d "backend" ]; then
        print_error "Backend directory not found"
        return 1
    fi
    
    cd backend
    
    # Check if cargo is available
    if ! command_exists cargo; then
        print_error "cargo not found. Please install Rust"
        return 1
    fi
    
    # Install cargo-audit if not present
    if ! command_exists cargo-audit; then
        print_status "Installing cargo-audit..."
        cargo install cargo-audit
    fi
    
    # Run cargo audit
    print_status "Running cargo audit..."
    if cargo audit; then
        print_success "No known vulnerabilities found in backend dependencies"
    else
        print_warning "Vulnerabilities found in backend dependencies"
        print_status "Please review and update affected dependencies"
    fi
    
    # Check for outdated packages
    if ! command_exists cargo-outdated; then
        print_status "Installing cargo-outdated..."
        cargo install cargo-outdated
    fi
    
    print_status "Checking for outdated backend packages..."
    cargo outdated || true
    
    cd ..
}

# Function to audit soroban dependencies
audit_soroban() {
    print_status "Auditing soroban dependencies..."
    
    if [ ! -d "soroban" ]; then
        print_error "Soroban directory not found"
        return 1
    fi
    
    cd soroban
    
    # Check if cargo is available
    if ! command_exists cargo; then
        print_error "cargo not found. Please install Rust"
        return 1
    fi
    
    # Install cargo-audit if not present
    if ! command_exists cargo-audit; then
        print_status "Installing cargo-audit..."
        cargo install cargo-audit
    fi
    
    # Run cargo audit
    print_status "Running cargo audit for soroban workspace..."
    if cargo audit; then
        print_success "No known vulnerabilities found in soroban dependencies"
    else
        print_warning "Vulnerabilities found in soroban dependencies"
        print_status "Please review and update affected dependencies"
    fi
    
    # Check for outdated packages
    if ! command_exists cargo-outdated; then
        print_status "Installing cargo-outdated..."
        cargo install cargo-outdated
    fi
    
    print_status "Checking for outdated soroban packages..."
    cargo outdated || true
    
    cd ..
}

# Function to update all dependencies
update_dependencies() {
    print_status "Updating all dependencies..."
    
    # Update frontend dependencies
    if [ -d "frontend" ]; then
        print_status "Updating frontend dependencies..."
        cd frontend
        npm update
        cd ..
        print_success "Frontend dependencies updated"
    fi
    
    # Update backend dependencies
    if [ -d "backend" ]; then
        print_status "Updating backend dependencies..."
        cd backend
        cargo update
        cd ..
        print_success "Backend dependencies updated"
    fi
    
    # Update soroban dependencies
    if [ -d "soroban" ]; then
        print_status "Updating soroban dependencies..."
        cd soroban
        cargo update
        cd ..
        print_success "Soroban dependencies updated"
    fi
}

# Function to generate dependency report
generate_report() {
    print_status "Generating dependency report..."
    
    REPORT_FILE="dependency-report-$(date +%Y%m%d-%H%M%S).md"
    
    cat > "$REPORT_FILE" << EOF
# Dependency Report - $(date)

## Frontend Dependencies

### Production Dependencies
EOF
    
    if [ -d "frontend" ]; then
        cd frontend
        echo '```json' >> "../$REPORT_FILE"
        npm list --depth=0 --prod --json | jq '.dependencies' >> "../$REPORT_FILE" 2>/dev/null || echo "Error generating frontend dependency list" >> "../$REPORT_FILE"
        echo '```' >> "../$REPORT_FILE"
        cd ..
    fi
    
    cat >> "$REPORT_FILE" << EOF

## Backend Dependencies

### Cargo Dependencies
EOF
    
    if [ -d "backend" ]; then
        cd backend
        echo '```toml' >> "../$REPORT_FILE"
        cargo tree --format "{p} {f}" >> "../$REPORT_FILE" 2>/dev/null || echo "Error generating backend dependency tree" >> "../$REPORT_FILE"
        echo '```' >> "../$REPORT_FILE"
        cd ..
    fi
    
    cat >> "$REPORT_FILE" << EOF

## Soroban Dependencies

### Workspace Dependencies
EOF
    
    if [ -d "soroban" ]; then
        cd soroban
        echo '```toml' >> "../$REPORT_FILE"
        cargo tree --format "{p} {f}" >> "../$REPORT_FILE" 2>/dev/null || echo "Error generating soroban dependency tree" >> "../$REPORT_FILE"
        echo '```' >> "../$REPORT_FILE"
        cd ..
    fi
    
    print_success "Dependency report generated: $REPORT_FILE"
}

# Function to clean dependencies
clean_dependencies() {
    print_status "Cleaning dependency caches..."
    
    # Clean frontend
    if [ -d "frontend" ]; then
        cd frontend
        rm -rf node_modules package-lock.json
        npm install
        cd ..
        print_success "Frontend dependencies cleaned and reinstalled"
    fi
    
    # Clean backend
    if [ -d "backend" ]; then
        cd backend
        cargo clean
        cd ..
        print_success "Backend build cache cleaned"
    fi
    
    # Clean soroban
    if [ -d "soroban" ]; then
        cd soroban
        cargo clean
        cd ..
        print_success "Soroban build cache cleaned"
    fi
}

# Function to check dependency licenses
check_licenses() {
    print_status "Checking dependency licenses..."
    
    # Frontend license check
    if [ -d "frontend" ]; then
        print_status "Checking frontend licenses..."
        cd frontend
        if command_exists license-checker; then
            license-checker --summary
        else
            print_warning "license-checker not installed. Run: npm install -g license-checker"
        fi
        cd ..
    fi
    
    # Backend license check
    if [ -d "backend" ]; then
        print_status "Checking backend licenses..."
        cd backend
        if command_exists cargo-license; then
            cargo license
        else
            print_warning "cargo-license not installed. Run: cargo install cargo-license"
        fi
        cd ..
    fi
    
    # Soroban license check
    if [ -d "soroban" ]; then
        print_status "Checking soroban licenses..."
        cd soroban
        if command_exists cargo-license; then
            cargo license
        else
            print_warning "cargo-license not installed. Run: cargo install cargo-license"
        fi
        cd ..
    fi
}

# Main function
main() {
    case "${1:-audit}" in
        "audit")
            print_status "Running security audit for all components..."
            audit_frontend
            audit_backend
            audit_soroban
            ;;
        "update")
            update_dependencies
            ;;
        "report")
            generate_report
            ;;
        "clean")
            clean_dependencies
            ;;
        "licenses")
            check_licenses
            ;;
        "all")
            audit_frontend
            audit_backend
            audit_soroban
            generate_report
            check_licenses
            ;;
        *)
            echo "Usage: $0 {audit|update|report|clean|licenses|all}"
            echo ""
            echo "Commands:"
            echo "  audit     - Run security audit for all components (default)"
            echo "  update    - Update all dependencies"
            echo "  report    - Generate dependency report"
            echo "  clean     - Clean and reinstall dependencies"
            echo "  licenses  - Check dependency licenses"
            echo "  all       - Run audit, generate report, and check licenses"
            exit 1
            ;;
    esac
}

# Run main function with all arguments
main "$@"