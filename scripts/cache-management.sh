#!/bin/bash

# Build Cache Management Script
# Manages build caches across all components for optimal performance

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
ACTION=${1:-status}
COMPONENT=${2:-all}

log() {
    echo -e "${BLUE}[CACHE]${NC} $1"
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

# Function to format bytes
format_bytes() {
    local bytes=$1
    local units=("B" "KB" "MB" "GB")
    local unit=0
    
    while [ $bytes -gt 1024 ] && [ $unit -lt 3 ]; do
        bytes=$((bytes / 1024))
        unit=$((unit + 1))
    done
    
    echo "${bytes}${units[$unit]}"
}

# Get directory size
get_dir_size() {
    local dir=$1
    if [ -d "$dir" ]; then
        du -sb "$dir" 2>/dev/null | cut -f1 || echo 0
    else
        echo 0
    fi
}

# Frontend cache management
manage_frontend_cache() {
    local action=$1
    
    case $action in
        "status")
            log "Frontend Cache Status:"
            
            local node_modules_size=$(get_dir_size "frontend/node_modules")
            local vite_cache_size=$(get_dir_size "frontend/node_modules/.vite")
            local dist_size=$(get_dir_size "frontend/dist")
            
            echo "  Node Modules: $(format_bytes $node_modules_size)"
            echo "  Vite Cache: $(format_bytes $vite_cache_size)"
            echo "  Build Output: $(format_bytes $dist_size)"
            echo "  Total: $(format_bytes $((node_modules_size + vite_cache_size + dist_size)))"
            ;;
            
        "clean")
            log "Cleaning frontend cache..."
            
            if [ -d "frontend/node_modules/.vite" ]; then
                rm -rf frontend/node_modules/.vite
                success "Vite cache cleaned"
            fi
            
            if [ -d "frontend/dist" ]; then
                rm -rf frontend/dist
                success "Build output cleaned"
            fi
            ;;
            
        "deep-clean")
            log "Deep cleaning frontend cache..."
            
            if [ -d "frontend/node_modules" ]; then
                rm -rf frontend/node_modules
                success "Node modules removed"
            fi
            
            if [ -f "frontend/package-lock.json" ]; then
                rm -f frontend/package-lock.json
                success "Package lock removed"
            fi
            
            if [ -d "frontend/dist" ]; then
                rm -rf frontend/dist
                success "Build output cleaned"
            fi
            ;;
            
        "optimize")
            log "Optimizing frontend cache..."
            
            cd frontend
            
            # Clean old cache
            if [ -d "node_modules/.vite" ]; then
                find node_modules/.vite -type f -mtime +7 -delete 2>/dev/null || true
                success "Old Vite cache files removed"
            fi
            
            # Prune unused dependencies
            if command -v npm &> /dev/null; then
                npm prune
                success "Unused dependencies pruned"
            fi
            
            cd ..
            ;;
    esac
}

# Backend cache management
manage_backend_cache() {
    local action=$1
    
    case $action in
        "status")
            log "Backend Cache Status:"
            
            local target_size=$(get_dir_size "backend/target")
            local cargo_registry_size=$(get_dir_size "$HOME/.cargo/registry")
            local cargo_git_size=$(get_dir_size "$HOME/.cargo/git")
            
            echo "  Target Directory: $(format_bytes $target_size)"
            echo "  Cargo Registry: $(format_bytes $cargo_registry_size)"
            echo "  Cargo Git: $(format_bytes $cargo_git_size)"
            echo "  Total: $(format_bytes $((target_size + cargo_registry_size + cargo_git_size)))"
            ;;
            
        "clean")
            log "Cleaning backend cache..."
            
            cd backend
            cargo clean
            success "Cargo target directory cleaned"
            cd ..
            ;;
            
        "deep-clean")
            log "Deep cleaning backend cache..."
            
            cd backend
            cargo clean
            
            # Clean incremental compilation cache
            if [ -d "target" ]; then
                rm -rf target
                success "Target directory removed"
            fi
            
            cd ..
            ;;
            
        "optimize")
            log "Optimizing backend cache..."
            
            cd backend
            
            # Clean old build artifacts
            if [ -d "target" ]; then
                find target -name "*.rlib" -mtime +7 -delete 2>/dev/null || true
                find target -name "*.rmeta" -mtime +7 -delete 2>/dev/null || true
                success "Old build artifacts removed"
            fi
            
            cd ..
            
            # Clean old cargo cache
            if command -v cargo &> /dev/null; then
                cargo cache --autoclean 2>/dev/null || true
                success "Cargo cache optimized"
            fi
            ;;
    esac
}

# Soroban cache management
manage_soroban_cache() {
    local action=$1
    
    case $action in
        "status")
            log "Soroban Cache Status:"
            
            local target_size=$(get_dir_size "soroban/target")
            local wasm_size=$(get_dir_size "soroban/target/wasm32-unknown-unknown")
            
            echo "  Target Directory: $(format_bytes $target_size)"
            echo "  WASM Artifacts: $(format_bytes $wasm_size)"
            echo "  Total: $(format_bytes $target_size)"
            ;;
            
        "clean")
            log "Cleaning Soroban cache..."
            
            cd soroban
            cargo clean
            success "Soroban target directory cleaned"
            cd ..
            ;;
            
        "deep-clean")
            log "Deep cleaning Soroban cache..."
            
            cd soroban
            cargo clean
            
            if [ -d "target" ]; then
                rm -rf target
                success "Target directory removed"
            fi
            
            cd ..
            ;;
            
        "optimize")
            log "Optimizing Soroban cache..."
            
            cd soroban
            
            # Clean old WASM files
            if [ -d "target/wasm32-unknown-unknown" ]; then
                find target/wasm32-unknown-unknown -name "*.wasm" -mtime +7 -delete 2>/dev/null || true
                success "Old WASM files removed"
            fi
            
            cd ..
            ;;
    esac
}

# Docker cache management
manage_docker_cache() {
    local action=$1
    
    case $action in
        "status")
            log "Docker Cache Status:"
            
            if command -v docker &> /dev/null; then
                local images_size=$(docker system df --format "table {{.Size}}" | tail -n +2 | head -n 1 || echo "0B")
                local containers_size=$(docker system df --format "table {{.Size}}" | tail -n +2 | head -n 2 | tail -n 1 || echo "0B")
                local volumes_size=$(docker system df --format "table {{.Size}}" | tail -n +2 | tail -n 1 || echo "0B")
                
                echo "  Images: $images_size"
                echo "  Containers: $containers_size"
                echo "  Volumes: $volumes_size"
            else
                warn "Docker not available"
            fi
            ;;
            
        "clean")
            log "Cleaning Docker cache..."
            
            if command -v docker &> /dev/null; then
                docker system prune -f
                success "Docker cache cleaned"
            else
                warn "Docker not available"
            fi
            ;;
            
        "deep-clean")
            log "Deep cleaning Docker cache..."
            
            if command -v docker &> /dev/null; then
                docker system prune -af --volumes
                success "Docker cache deep cleaned"
            else
                warn "Docker not available"
            fi
            ;;
            
        "optimize")
            log "Optimizing Docker cache..."
            
            if command -v docker &> /dev/null; then
                # Remove dangling images
                docker image prune -f
                
                # Remove unused volumes
                docker volume prune -f
                
                success "Docker cache optimized"
            else
                warn "Docker not available"
            fi
            ;;
    esac
}

# Global cache management
manage_global_cache() {
    local action=$1
    
    case $action in
        "status")
            log "Global Cache Status:"
            echo "================================"
            
            manage_frontend_cache "status"
            echo
            manage_backend_cache "status"
            echo
            manage_soroban_cache "status"
            echo
            manage_docker_cache "status"
            ;;
            
        "clean")
            log "Cleaning all caches..."
            
            manage_frontend_cache "clean"
            manage_backend_cache "clean"
            manage_soroban_cache "clean"
            manage_docker_cache "clean"
            
            success "All caches cleaned"
            ;;
            
        "deep-clean")
            log "Deep cleaning all caches..."
            
            manage_frontend_cache "deep-clean"
            manage_backend_cache "deep-clean"
            manage_soroban_cache "deep-clean"
            manage_docker_cache "deep-clean"
            
            success "All caches deep cleaned"
            ;;
            
        "optimize")
            log "Optimizing all caches..."
            
            manage_frontend_cache "optimize"
            manage_backend_cache "optimize"
            manage_soroban_cache "optimize"
            manage_docker_cache "optimize"
            
            success "All caches optimized"
            ;;
    esac
}

# Main function
main() {
    case $COMPONENT in
        "frontend")
            manage_frontend_cache "$ACTION"
            ;;
        "backend")
            manage_backend_cache "$ACTION"
            ;;
        "soroban")
            manage_soroban_cache "$ACTION"
            ;;
        "docker")
            manage_docker_cache "$ACTION"
            ;;
        "all")
            manage_global_cache "$ACTION"
            ;;
        *)
            error "Unknown component: $COMPONENT"
            exit 1
            ;;
    esac
}

# Show help
if [ "$1" = "--help" ] || [ "$1" = "-h" ]; then
    echo "Usage: $0 [action] [component]"
    echo ""
    echo "Actions:"
    echo "  status      - Show cache status and sizes (default)"
    echo "  clean       - Clean build caches"
    echo "  deep-clean  - Deep clean all caches and dependencies"
    echo "  optimize    - Optimize caches by removing old files"
    echo ""
    echo "Components:"
    echo "  frontend    - Frontend (Node.js/Vite) caches"
    echo "  backend     - Backend (Rust/Cargo) caches"
    echo "  soroban     - Soroban contracts caches"
    echo "  docker      - Docker build caches"
    echo "  all         - All components (default)"
    echo ""
    echo "Examples:"
    echo "  $0                      # Show status of all caches"
    echo "  $0 clean frontend       # Clean frontend caches"
    echo "  $0 deep-clean all       # Deep clean all caches"
    echo "  $0 optimize backend     # Optimize backend caches"
    exit 0
fi

# Run main function
main