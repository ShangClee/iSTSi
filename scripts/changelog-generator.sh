#!/bin/bash

# Changelog Generator for Bitcoin Custody Full-Stack Application
# Generates changelogs based on git commits and version changes

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
CHANGELOG_FILE="$PROJECT_ROOT/CHANGELOG.md"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

log() {
    echo -e "${BLUE}[CHANGELOG]${NC} $1"
}

error() {
    echo -e "${RED}[ERROR]${NC} $1" >&2
}

success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

# Initialize changelog if it doesn't exist
init_changelog() {
    if [ ! -f "$CHANGELOG_FILE" ]; then
        log "Creating new CHANGELOG.md"
        cat > "$CHANGELOG_FILE" << 'EOF'
# Changelog

All notable changes to the Bitcoin Custody Full-Stack Application will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Initial project structure with frontend, backend, and soroban components

EOF
        success "Created initial CHANGELOG.md"
    fi
}

# Get git commits since last tag
get_commits_since_tag() {
    local last_tag=$(git describe --tags --abbrev=0 2>/dev/null || echo "")
    
    if [ -z "$last_tag" ]; then
        # No tags exist, get all commits
        git log --oneline --no-merges
    else
        # Get commits since last tag
        git log "${last_tag}..HEAD" --oneline --no-merges
    fi
}

# Categorize commit based on conventional commit format
categorize_commit() {
    local commit_msg="$1"
    
    case "$commit_msg" in
        feat*|feature*)
            echo "Added"
            ;;
        fix*|bugfix*)
            echo "Fixed"
            ;;
        docs*)
            echo "Documentation"
            ;;
        style*|format*)
            echo "Style"
            ;;
        refactor*)
            echo "Changed"
            ;;
        perf*|performance*)
            echo "Performance"
            ;;
        test*)
            echo "Testing"
            ;;
        build*|ci*|chore*)
            echo "Build"
            ;;
        breaking*|BREAKING*)
            echo "Breaking"
            ;;
        security*)
            echo "Security"
            ;;
        *)
            echo "Changed"
            ;;
    esac
}

# Extract component from commit message or file paths
get_component_from_commit() {
    local commit_hash="$1"
    local commit_msg="$2"
    
    # Check if commit message mentions component
    if echo "$commit_msg" | grep -qi "frontend\|react\|ui"; then
        echo "frontend"
    elif echo "$commit_msg" | grep -qi "backend\|api\|loco"; then
        echo "backend"
    elif echo "$commit_msg" | grep -qi "soroban\|contract\|stellar"; then
        echo "soroban"
    else
        # Check file paths in commit
        local files=$(git show --name-only --format="" "$commit_hash")
        if echo "$files" | grep -q "^frontend/"; then
            echo "frontend"
        elif echo "$files" | grep -q "^backend/"; then
            echo "backend"
        elif echo "$files" | grep -q "^soroban/"; then
            echo "soroban"
        else
            echo "general"
        fi
    fi
}

# Generate changelog entry for version
generate_version_changelog() {
    local version="$1"
    local component="$2"
    local date="${3:-$(date '+%Y-%m-%d')}"
    
    log "Generating changelog for $component v$version"
    
    # Create temporary file for new changelog content
    local temp_file=$(mktemp)
    
    # Add new version header
    cat > "$temp_file" << EOF
## [$component v$version] - $date

EOF
    
    # Get commits for this component since last version
    local commits=$(get_commits_since_tag)
    
    # Categorize commits
    declare -A categories
    categories["Added"]=""
    categories["Fixed"]=""
    categories["Changed"]=""
    categories["Security"]=""
    categories["Performance"]=""
    categories["Breaking"]=""
    categories["Documentation"]=""
    
    while IFS= read -r line; do
        if [ -n "$line" ]; then
            local commit_hash=$(echo "$line" | cut -d' ' -f1)
            local commit_msg=$(echo "$line" | cut -d' ' -f2-)
            local commit_component=$(get_component_from_commit "$commit_hash" "$commit_msg")
            
            # Only include commits for this component or general commits
            if [ "$commit_component" = "$component" ] || [ "$commit_component" = "general" ]; then
                local category=$(categorize_commit "$commit_msg")
                if [ -n "${categories[$category]+x}" ]; then
                    categories["$category"]+="- $commit_msg"$'\n'
                fi
            fi
        fi
    done <<< "$commits"
    
    # Add categories with content to changelog
    for category in "Breaking" "Added" "Changed" "Fixed" "Security" "Performance" "Documentation"; do
        if [ -n "${categories[$category]}" ]; then
            echo "### $category" >> "$temp_file"
            echo "${categories[$category]}" >> "$temp_file"
        fi
    done
    
    # Add compatibility info
    cat >> "$temp_file" << EOF

**Compatibility Notes:**
- Requires all components to be on compatible versions (see COMPATIBILITY_MATRIX.md)
- Database migrations: $([ "$component" = "backend" ] && echo "Required" || echo "None")
- Contract deployments: $([ "$component" = "soroban" ] && echo "Required" || echo "None")

EOF
    
    # Prepend to existing changelog (after the header)
    if [ -f "$CHANGELOG_FILE" ]; then
        # Find the line number after the header
        local header_end=$(grep -n "^## \[Unreleased\]" "$CHANGELOG_FILE" | cut -d: -f1)
        if [ -n "$header_end" ]; then
            # Insert new content after unreleased section
            head -n "$((header_end + 2))" "$CHANGELOG_FILE" > "${temp_file}.full"
            cat "$temp_file" >> "${temp_file}.full"
            tail -n "+$((header_end + 3))" "$CHANGELOG_FILE" >> "${temp_file}.full"
            mv "${temp_file}.full" "$CHANGELOG_FILE"
        else
            # Append to end of file
            cat "$temp_file" >> "$CHANGELOG_FILE"
        fi
    else
        init_changelog
        cat "$temp_file" >> "$CHANGELOG_FILE"
    fi
    
    rm -f "$temp_file"
    success "Added changelog entry for $component v$version"
}

# Generate full changelog from git history
generate_full_changelog() {
    log "Generating full changelog from git history"
    
    # Backup existing changelog
    if [ -f "$CHANGELOG_FILE" ]; then
        cp "$CHANGELOG_FILE" "${CHANGELOG_FILE}.backup"
        log "Backed up existing changelog to ${CHANGELOG_FILE}.backup"
    fi
    
    # Initialize new changelog
    init_changelog
    
    # Get all tags in reverse chronological order
    local tags=$(git tag --sort=-version:refname)
    
    if [ -z "$tags" ]; then
        log "No git tags found, generating from all commits"
        generate_version_changelog "1.0.0" "general"
        return
    fi
    
    # Process each tag
    local prev_tag=""
    while IFS= read -r tag; do
        if [ -n "$tag" ]; then
            local tag_date=$(git log -1 --format=%ai "$tag" | cut -d' ' -f1)
            
            # Determine component from tag name or use general
            local component="general"
            if echo "$tag" | grep -q "frontend"; then
                component="frontend"
            elif echo "$tag" | grep -q "backend"; then
                component="backend"
            elif echo "$tag" | grep -q "soroban"; then
                component="soroban"
            fi
            
            generate_version_changelog "$tag" "$component" "$tag_date"
            prev_tag="$tag"
        fi
    done <<< "$tags"
    
    success "Generated full changelog from git history"
}

# Update unreleased section
update_unreleased() {
    log "Updating unreleased section"
    
    init_changelog
    
    # Get commits since last tag
    local commits=$(get_commits_since_tag)
    
    if [ -z "$commits" ]; then
        log "No new commits since last release"
        return
    fi
    
    # Create temporary file for unreleased content
    local temp_file=$(mktemp)
    
    # Categorize commits
    declare -A categories
    categories["Added"]=""
    categories["Fixed"]=""
    categories["Changed"]=""
    categories["Security"]=""
    categories["Performance"]=""
    categories["Breaking"]=""
    
    while IFS= read -r line; do
        if [ -n "$line" ]; then
            local commit_hash=$(echo "$line" | cut -d' ' -f1)
            local commit_msg=$(echo "$line" | cut -d' ' -f2-)
            local category=$(categorize_commit "$commit_msg")
            
            if [ -n "${categories[$category]+x}" ]; then
                categories["$category"]+="- $commit_msg"$'\n'
            fi
        fi
    done <<< "$commits"
    
    # Build unreleased section
    echo "## [Unreleased]" > "$temp_file"
    echo "" >> "$temp_file"
    
    for category in "Breaking" "Added" "Changed" "Fixed" "Security" "Performance"; do
        if [ -n "${categories[$category]}" ]; then
            echo "### $category" >> "$temp_file"
            echo "${categories[$category]}" >> "$temp_file"
        fi
    done
    
    # Replace unreleased section in changelog
    if [ -f "$CHANGELOG_FILE" ]; then
        # Find unreleased section and replace it
        local start_line=$(grep -n "^## \[Unreleased\]" "$CHANGELOG_FILE" | cut -d: -f1)
        local next_release=$(grep -n "^## \[" "$CHANGELOG_FILE" | grep -v "Unreleased" | head -1 | cut -d: -f1)
        
        if [ -n "$start_line" ]; then
            if [ -n "$next_release" ]; then
                # Replace between unreleased and next release
                head -n "$((start_line - 1))" "$CHANGELOG_FILE" > "${temp_file}.full"
                cat "$temp_file" >> "${temp_file}.full"
                echo "" >> "${temp_file}.full"
                tail -n "+$next_release" "$CHANGELOG_FILE" >> "${temp_file}.full"
            else
                # Replace from unreleased to end
                head -n "$((start_line - 1))" "$CHANGELOG_FILE" > "${temp_file}.full"
                cat "$temp_file" >> "${temp_file}.full"
            fi
            mv "${temp_file}.full" "$CHANGELOG_FILE"
        fi
    fi
    
    rm -f "$temp_file"
    success "Updated unreleased section"
}

# Validate changelog format
validate_changelog() {
    if [ ! -f "$CHANGELOG_FILE" ]; then
        error "CHANGELOG.md not found"
        return 1
    fi
    
    log "Validating changelog format"
    
    # Check for required sections
    if ! grep -q "^# Changelog" "$CHANGELOG_FILE"; then
        error "Missing main changelog header"
        return 1
    fi
    
    if ! grep -q "^## \[Unreleased\]" "$CHANGELOG_FILE"; then
        warn "Missing unreleased section"
    fi
    
    # Check for proper version format
    local invalid_versions=$(grep "^## \[" "$CHANGELOG_FILE" | grep -v "Unreleased" | grep -v "\[.*v[0-9]\+\.[0-9]\+\.[0-9]\+\]")
    if [ -n "$invalid_versions" ]; then
        error "Invalid version format found:"
        echo "$invalid_versions"
        return 1
    fi
    
    success "Changelog format is valid"
}

# Main command handler
main() {
    case "${1:-help}" in
        "init")
            init_changelog
            ;;
        "generate")
            generate_version_changelog "$2" "$3" "$4"
            ;;
        "full")
            generate_full_changelog
            ;;
        "unreleased")
            update_unreleased
            ;;
        "validate")
            validate_changelog
            ;;
        "help"|*)
            cat << EOF
Changelog Generator for Bitcoin Custody Full-Stack Application

Usage: $0 <command> [options]

Commands:
  init                                Initialize new CHANGELOG.md
  generate <version> <component> [date] Generate changelog entry for version
  full                               Generate full changelog from git history
  unreleased                         Update unreleased section with recent commits
  validate                           Validate changelog format
  help                              Show this help message

Examples:
  $0 init                           # Create new changelog
  $0 generate 1.1.0 frontend       # Generate entry for frontend v1.1.0
  $0 unreleased                     # Update unreleased section
  $0 validate                       # Check changelog format

EOF
            ;;
    esac
}

main "$@"