#!/bin/bash

# Version Manager Script for Bitcoin Custody Full-Stack Application
# Handles semantic versioning across frontend, backend, and soroban components

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
VERSION_CONFIG="$PROJECT_ROOT/version-config.json"
COMPATIBILITY_LOG="$PROJECT_ROOT/compatibility-log.md"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging function
log() {
    echo -e "${BLUE}[VERSION-MANAGER]${NC} $1"
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

# Check if jq is installed
check_dependencies() {
    if ! command -v jq &> /dev/null; then
        error "jq is required but not installed. Please install jq to continue."
        exit 1
    fi
}

# Get current version of a component
get_current_version() {
    local component=$1
    local component_path=$(jq -r ".project.components.${component}.path" "$VERSION_CONFIG")
    local component_type=$(jq -r ".project.components.${component}.type" "$VERSION_CONFIG")
    
    case $component_type in
        "npm")
            if [ -f "$PROJECT_ROOT/$component_path/package.json" ]; then
                jq -r '.version' "$PROJECT_ROOT/$component_path/package.json"
            else
                echo "0.0.0"
            fi
            ;;
        "cargo")
            if [ -f "$PROJECT_ROOT/$component_path/Cargo.toml" ]; then
                grep '^version = ' "$PROJECT_ROOT/$component_path/Cargo.toml" | head -1 | sed 's/version = "\(.*\)"/\1/'
            else
                echo "0.0.0"
            fi
            ;;
        *)
            echo "0.0.0"
            ;;
    esac
}

# Update version in component files
update_component_version() {
    local component=$1
    local new_version=$2
    local component_path=$(jq -r ".project.components.${component}.path" "$VERSION_CONFIG")
    local component_type=$(jq -r ".project.components.${component}.type" "$VERSION_CONFIG")
    
    log "Updating $component version to $new_version"
    
    case $component_type in
        "npm")
            if [ -f "$PROJECT_ROOT/$component_path/package.json" ]; then
                # Update package.json version
                jq ".version = \"$new_version\"" "$PROJECT_ROOT/$component_path/package.json" > tmp.json
                mv tmp.json "$PROJECT_ROOT/$component_path/package.json"
                success "Updated $component package.json version to $new_version"
            fi
            ;;
        "cargo")
            if [ -f "$PROJECT_ROOT/$component_path/Cargo.toml" ]; then
                # Update Cargo.toml version
                sed -i.bak "s/^version = \".*\"/version = \"$new_version\"/" "$PROJECT_ROOT/$component_path/Cargo.toml"
                rm -f "$PROJECT_ROOT/$component_path/Cargo.toml.bak"
                success "Updated $component Cargo.toml version to $new_version"
            fi
            ;;
    esac
    
    # Update version config
    jq ".project.components.${component}.version = \"$new_version\"" "$VERSION_CONFIG" > tmp.json
    mv tmp.json "$VERSION_CONFIG"
}

# Increment version based on type (major, minor, patch)
increment_version() {
    local current_version=$1
    local increment_type=$2
    
    IFS='.' read -ra VERSION_PARTS <<< "$current_version"
    local major=${VERSION_PARTS[0]}
    local minor=${VERSION_PARTS[1]}
    local patch=${VERSION_PARTS[2]}
    
    case $increment_type in
        "major")
            major=$((major + 1))
            minor=0
            patch=0
            ;;
        "minor")
            minor=$((minor + 1))
            patch=0
            ;;
        "patch")
            patch=$((patch + 1))
            ;;
        *)
            error "Invalid increment type: $increment_type. Use major, minor, or patch."
            exit 1
            ;;
    esac
    
    echo "${major}.${minor}.${patch}"
}

# Check compatibility between components
check_compatibility() {
    local frontend_version=$(get_current_version "frontend")
    local backend_version=$(get_current_version "backend")
    local soroban_version=$(get_current_version "soroban")
    
    log "Checking compatibility:"
    log "  Frontend: $frontend_version"
    log "  Backend: $backend_version"
    log "  Soroban: $soroban_version"
    
    # Extract major.minor versions for compatibility check
    local frontend_major_minor=$(echo $frontend_version | cut -d. -f1-2)
    local backend_major_minor=$(echo $backend_version | cut -d. -f1-2)
    local soroban_major_minor=$(echo $soroban_version | cut -d. -f1-2)
    
    # Check if all components have compatible major.minor versions
    if [ "$frontend_major_minor" = "$backend_major_minor" ] && [ "$backend_major_minor" = "$soroban_major_minor" ]; then
        success "All components are compatible (${frontend_major_minor}.x)"
        return 0
    else
        warn "Version compatibility issues detected:"
        warn "  Frontend: ${frontend_major_minor}.x"
        warn "  Backend: ${backend_major_minor}.x"
        warn "  Soroban: ${soroban_major_minor}.x"
        return 1
    fi
}

# Generate changelog entry
generate_changelog_entry() {
    local component=$1
    local old_version=$2
    local new_version=$3
    local change_type=$4
    local description=$5
    
    local timestamp=$(date '+%Y-%m-%d %H:%M:%S')
    
    cat >> "$PROJECT_ROOT/CHANGELOG.md" << EOF

## [$component v$new_version] - $timestamp

### $change_type
- $description

**Compatibility:** 
- Previous version: $old_version
- Breaking changes: $([ "$change_type" = "Major" ] && echo "Yes" || echo "No")

EOF
}

# Bump version for a specific component
bump_component_version() {
    local component=$1
    local increment_type=$2
    local description=$3
    
    if [ -z "$component" ] || [ -z "$increment_type" ]; then
        error "Usage: bump_component_version <component> <major|minor|patch> [description]"
        exit 1
    fi
    
    # Validate component exists
    if ! jq -e ".project.components.${component}" "$VERSION_CONFIG" > /dev/null; then
        error "Component '$component' not found in version config"
        exit 1
    fi
    
    local current_version=$(get_current_version "$component")
    local new_version=$(increment_version "$current_version" "$increment_type")
    
    log "Bumping $component version from $current_version to $new_version ($increment_type)"
    
    # Update component version
    update_component_version "$component" "$new_version"
    
    # Generate changelog entry
    local change_type_title=$(echo "$increment_type" | sed 's/.*/\u&/')
    generate_changelog_entry "$component" "$current_version" "$new_version" "$change_type_title" "${description:-$increment_type version bump}"
    
    success "$component version bumped to $new_version"
    
    # Check compatibility after version bump
    if ! check_compatibility; then
        warn "Consider updating other components to maintain compatibility"
    fi
}

# Sync all component versions to the same major.minor
sync_versions() {
    local target_version=$1
    
    if [ -z "$target_version" ]; then
        error "Usage: sync_versions <target_version>"
        exit 1
    fi
    
    log "Syncing all components to version $target_version"
    
    for component in frontend backend soroban; do
        local current_version=$(get_current_version "$component")
        if [ "$current_version" != "$target_version" ]; then
            update_component_version "$component" "$target_version"
            generate_changelog_entry "$component" "$current_version" "$target_version" "Sync" "Version sync to $target_version"
        fi
    done
    
    success "All components synced to version $target_version"
}

# Display current versions
show_versions() {
    log "Current component versions:"
    echo
    printf "%-12s %-10s %-15s\n" "Component" "Version" "Path"
    printf "%-12s %-10s %-15s\n" "---------" "-------" "----"
    
    for component in frontend backend soroban; do
        local version=$(get_current_version "$component")
        local path=$(jq -r ".project.components.${component}.path" "$VERSION_CONFIG")
        printf "%-12s %-10s %-15s\n" "$component" "$version" "$path"
    done
    
    echo
    check_compatibility
}

# Validate dependencies and compatibility
validate_dependencies() {
    log "Validating component dependencies and compatibility..."
    
    # Check if all required files exist
    local all_valid=true
    
    for component in frontend backend soroban; do
        local component_path=$(jq -r ".project.components.${component}.path" "$VERSION_CONFIG")
        local component_type=$(jq -r ".project.components.${component}.type" "$VERSION_CONFIG")
        
        case $component_type in
            "npm")
                if [ ! -f "$PROJECT_ROOT/$component_path/package.json" ]; then
                    error "$component: package.json not found at $component_path"
                    all_valid=false
                fi
                ;;
            "cargo")
                if [ ! -f "$PROJECT_ROOT/$component_path/Cargo.toml" ]; then
                    error "$component: Cargo.toml not found at $component_path"
                    all_valid=false
                fi
                ;;
        esac
    done
    
    if [ "$all_valid" = true ]; then
        success "All component files are valid"
        check_compatibility
    else
        error "Validation failed - some component files are missing"
        exit 1
    fi
}

# Main command handler
main() {
    check_dependencies
    
    case "${1:-help}" in
        "show"|"status")
            show_versions
            ;;
        "bump")
            bump_component_version "$2" "$3" "$4"
            ;;
        "sync")
            sync_versions "$2"
            ;;
        "check"|"validate")
            validate_dependencies
            ;;
        "compatibility")
            check_compatibility
            ;;
        "help"|*)
            cat << EOF
Bitcoin Custody Version Manager

Usage: $0 <command> [options]

Commands:
  show                          Show current versions of all components
  bump <component> <type> [msg] Bump version for specific component
                               component: frontend, backend, soroban
                               type: major, minor, patch
  sync <version>               Sync all components to same version
  check                        Validate dependencies and compatibility
  compatibility                Check version compatibility between components
  help                         Show this help message

Examples:
  $0 show                      # Show current versions
  $0 bump frontend minor       # Bump frontend minor version
  $0 sync 1.2.0               # Sync all to version 1.2.0
  $0 check                     # Validate all components

EOF
            ;;
    esac
}

main "$@"