#!/bin/bash

# Development build script with fast compilation and debugging features

set -e

# Configuration
BUILD_MODE="debug"
WATCH_MODE=${WATCH_MODE:-false}

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

log_info() {
    echo -e "${GREEN}[DEV]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[DEV]${NC} $1"
}

# Fast development build
dev_build() {
    log_info "Starting development build..."
    
    # Use debug mode for faster compilation
    export BUILD_MODE="debug"
    
    # Skip optimization for speed
    export SKIP_OPTIMIZATION=true
    
    # Run main build script
    ./scripts/build.sh --debug
    
    log_info "Development build completed"
}

# Watch mode for continuous building
watch_build() {
    log_info "Starting watch mode..."
    
    if ! command -v fswatch &> /dev/null; then
        log_warn "fswatch not found. Install it for watch mode: brew install fswatch"
        exit 1
    fi
    
    # Initial build
    dev_build
    
    # Watch for changes
    log_info "Watching for changes in contracts/ directory..."
    fswatch -o contracts/ | while read f; do
        log_info "Changes detected, rebuilding..."
        dev_build
    done
}

# Main function
main() {
    if [ "$1" = "--watch" ]; then
        watch_build
    else
        dev_build
    fi
}

main "$@"