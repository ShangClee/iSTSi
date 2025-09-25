#!/bin/bash
# Unified build script for all components
# Usage: ./scripts/build.sh [component] [environment]
# Components: frontend, backend, soroban, all (default)
# Environments: development, staging, production (default)

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Default values
COMPONENT=${1:-all}
ENVIRONMENT=${2:-production}
BUILD_DIR="build"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)

# Build options
CLEAN_CACHE=${CLEAN_CACHE:-false}
MONITOR_PERFORMANCE=${MONITOR_PERFORMANCE:-false}
PARALLEL_BUILD=${PARALLEL_BUILD:-false}

# Logging function
log() {
    echo -e "${BLUE}[BUILD]${NC} $1"
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

# Create build directory
create_build_dir() {
    log "Creating build directory: $BUILD_DIR"
    mkdir -p "$BUILD_DIR"
    mkdir -p "$BUILD_DIR/artifacts"
    mkdir -p "$BUILD_DIR/logs"
}

# Build frontend
build_frontend() {
    log "Building frontend for $ENVIRONMENT environment..."
    
    if [ ! -d "frontend" ]; then
        error "Frontend directory not found. Please ensure project is properly restructured."
        return 1
    fi
    
    cd frontend
    
    # Check for cache and clean if requested
    if [ "$CLEAN_CACHE" = "true" ]; then
        log "Cleaning frontend cache..."
        rm -rf node_modules/.vite dist
    fi
    
    # Install dependencies if node_modules doesn't exist
    if [ ! -d "node_modules" ]; then
        log "Installing frontend dependencies..."
        npm ci --prefer-offline --no-audit
    fi
    
    # Set environment variables
    case $ENVIRONMENT in
        development)
            export NODE_ENV=development
            export VITE_API_URL=http://localhost:8080
            ;;
        staging)
            export NODE_ENV=production
            export VITE_API_URL=https://api-staging.bitcoincustody.com
            ;;
        production)
            export NODE_ENV=production
            export VITE_API_URL=https://api.bitcoincustody.com
            ;;
    esac
    
    # Run build performance monitoring
    if [ "$MONITOR_PERFORMANCE" = "true" ]; then
        log "Running build performance monitoring..."
        npm run build:performance
    fi
    
    # Type check
    log "Running TypeScript type check..."
    npm run type-check
    
    # Lint
    log "Running ESLint..."
    npm run lint
    
    # Build with optimization
    case $ENVIRONMENT in
        development)
            log "Building frontend application (development)..."
            npm run build:fast
            ;;
        *)
            log "Building frontend application (optimized)..."
            npm run build
            
            # Generate bundle analysis
            log "Generating bundle analysis..."
            npm run build:analyze
            ;;
    esac
    
    # Create artifact
    log "Creating frontend build artifact..."
    tar -czf "../$BUILD_DIR/artifacts/frontend-$ENVIRONMENT-$TIMESTAMP.tar.gz" dist/
    
    # Copy performance reports if they exist
    if [ -f "build-performance-report.json" ]; then
        cp build-performance-report.json "../$BUILD_DIR/artifacts/"
    fi
    
    if [ -f "dist/bundle-analysis.html" ]; then
        cp dist/bundle-analysis.html "../$BUILD_DIR/artifacts/"
    fi
    
    cd ..
    success "Frontend build completed successfully"
}

# Build backend
build_backend() {
    log "Building backend for $ENVIRONMENT environment..."
    
    if [ ! -d "backend" ]; then
        error "Backend directory not found. Please ensure project is properly restructured."
        return 1
    fi
    
    cd backend
    
    # Clean cache if requested
    if [ "$CLEAN_CACHE" = "true" ]; then
        log "Cleaning backend cache..."
        cargo clean
    fi
    
    # Set build profile based on environment
    case $ENVIRONMENT in
        development)
            CARGO_PROFILE="dev-fast"
            ;;
        staging)
            CARGO_PROFILE="release"
            ;;
        production)
            CARGO_PROFILE="release"
            ;;
    esac
    
    # Run build performance monitoring
    if [ "$MONITOR_PERFORMANCE" = "true" ]; then
        log "Running build performance monitoring..."
        ./scripts/build-performance.sh "$CARGO_PROFILE"
    fi
    
    # Build backend with optimizations
    log "Building backend with profile: $CARGO_PROFILE..."
    case $CARGO_PROFILE in
        "dev-fast")
            CARGO_INCREMENTAL=1 cargo build --profile dev-fast
            ;;
        "release")
            CARGO_INCREMENTAL=0 cargo build --release
            ;;
        *)
            cargo build
            ;;
    esac
    
    # Run tests
    log "Running backend tests..."
    cargo test --profile test
    
    # Create artifact
    log "Creating backend build artifact..."
    case $CARGO_PROFILE in
        "dev-fast")
            cp target/dev-fast/bitcoin-custody-backend "../$BUILD_DIR/artifacts/backend-$ENVIRONMENT-$TIMESTAMP" 2>/dev/null || \
            cp target/debug/bitcoin-custody-backend "../$BUILD_DIR/artifacts/backend-$ENVIRONMENT-$TIMESTAMP"
            ;;
        "release")
            cp target/release/bitcoin-custody-backend "../$BUILD_DIR/artifacts/backend-$ENVIRONMENT-$TIMESTAMP"
            ;;
    esac
    
    # Copy configuration files
    cp -r config "../$BUILD_DIR/artifacts/backend-config-$ENVIRONMENT-$TIMESTAMP"
    
    # Copy performance reports if they exist
    if [ -f "build-performance-report.json" ]; then
        cp build-performance-report.json "../$BUILD_DIR/artifacts/"
    fi
    
    cd ..
    success "Backend build completed successfully"
}

# Build Soroban contracts
build_soroban() {
    log "Building Soroban contracts for $ENVIRONMENT environment..."
    
    if [ ! -d "soroban" ]; then
        error "Soroban directory not found. Please ensure project is properly restructured."
        return 1
    fi
    
    cd soroban
    
    # Check if soroban CLI is installed
    if ! command -v soroban &> /dev/null; then
        error "Soroban CLI not found. Please install it first."
        return 1
    fi
    
    # Clean cache if requested
    if [ "$CLEAN_CACHE" = "true" ]; then
        log "Cleaning Soroban cache..."
        cargo clean
    fi
    
    # Set build profile based on environment
    case $ENVIRONMENT in
        development)
            CARGO_PROFILE="dev-fast"
            ;;
        staging)
            CARGO_PROFILE="release"
            ;;
        production)
            CARGO_PROFILE="release-size"
            ;;
    esac
    
    # Run build performance monitoring
    if [ "$MONITOR_PERFORMANCE" = "true" ]; then
        log "Running contract build performance monitoring..."
        ./scripts/build-performance.sh "$CARGO_PROFILE"
    fi
    
    # Build contracts with optimization
    log "Building all Soroban contracts with profile: $CARGO_PROFILE..."
    case $CARGO_PROFILE in
        "dev-fast")
            cargo build --target wasm32-unknown-unknown --profile dev-fast
            ;;
        "release")
            cargo build --target wasm32-unknown-unknown --release
            ;;
        "release-size")
            cargo build --target wasm32-unknown-unknown --profile release-size
            ;;
    esac
    
    # Run contract tests
    log "Running contract tests..."
    cargo test --profile test
    
    # Optimize contracts for deployment
    log "Optimizing contracts..."
    mkdir -p "../$BUILD_DIR/artifacts/contracts"
    
    # Determine target directory based on profile
    case $CARGO_PROFILE in
        "dev-fast")
            TARGET_DIR="target/wasm32-unknown-unknown/debug"
            ;;
        "release"|"release-size")
            TARGET_DIR="target/wasm32-unknown-unknown/release"
            ;;
    esac
    
    # Find and copy all contract WASM files
    find "$TARGET_DIR" -name "*.wasm" -type f | while read -r wasm_file; do
        contract_name=$(basename "$wasm_file" .wasm)
        log "Optimizing contract: $contract_name"
        
        # Copy original WASM
        cp "$wasm_file" "../$BUILD_DIR/artifacts/contracts/${contract_name}-$ENVIRONMENT-$TIMESTAMP-original.wasm"
        
        # Optimize the WASM file
        soroban contract optimize --wasm "$wasm_file" --out "../$BUILD_DIR/artifacts/contracts/${contract_name}-$ENVIRONMENT-$TIMESTAMP.wasm"
    done
    
    # Copy performance reports if they exist
    if [ -f "contract-build-performance.json" ]; then
        cp contract-build-performance.json "../$BUILD_DIR/artifacts/"
    fi
    
    cd ..
    success "Soroban contracts build completed successfully"
}

# Build Docker images
build_docker() {
    log "Building Docker images for $ENVIRONMENT environment..."
    
    # Build backend Docker image
    if [ -f "backend/Dockerfile" ] || [ -f "backend/Dockerfile.$ENVIRONMENT" ]; then
        log "Building backend Docker image..."
        if [ -f "backend/Dockerfile.$ENVIRONMENT" ]; then
            docker build -f "backend/Dockerfile.$ENVIRONMENT" -t "bitcoin-custody-backend:$ENVIRONMENT-$TIMESTAMP" backend/
        else
            docker build -t "bitcoin-custody-backend:$ENVIRONMENT-$TIMESTAMP" backend/
        fi
        
        # Tag as latest for environment
        docker tag "bitcoin-custody-backend:$ENVIRONMENT-$TIMESTAMP" "bitcoin-custody-backend:$ENVIRONMENT-latest"
    fi
    
    # Build frontend Docker image
    if [ -f "frontend/Dockerfile" ] || [ -f "frontend/Dockerfile.$ENVIRONMENT" ]; then
        log "Building frontend Docker image..."
        if [ -f "frontend/Dockerfile.$ENVIRONMENT" ]; then
            docker build -f "frontend/Dockerfile.$ENVIRONMENT" -t "bitcoin-custody-frontend:$ENVIRONMENT-$TIMESTAMP" frontend/
        else
            docker build -t "bitcoin-custody-frontend:$ENVIRONMENT-$TIMESTAMP" frontend/
        fi
        
        # Tag as latest for environment
        docker tag "bitcoin-custody-frontend:$ENVIRONMENT-$TIMESTAMP" "bitcoin-custody-frontend:$ENVIRONMENT-latest"
    fi
    
    success "Docker images built successfully"
}

# Generate build manifest
generate_manifest() {
    log "Generating build manifest..."
    
    cat > "$BUILD_DIR/build-manifest-$TIMESTAMP.json" << EOF
{
  "build_id": "$TIMESTAMP",
  "environment": "$ENVIRONMENT",
  "component": "$COMPONENT",
  "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "git_commit": "$(git rev-parse HEAD 2>/dev/null || echo 'unknown')",
  "git_branch": "$(git rev-parse --abbrev-ref HEAD 2>/dev/null || echo 'unknown')",
  "artifacts": {
    "frontend": "frontend-$ENVIRONMENT-$TIMESTAMP.tar.gz",
    "backend": "backend-$ENVIRONMENT-$TIMESTAMP",
    "backend_config": "backend-config-$ENVIRONMENT-$TIMESTAMP",
    "contracts": "contracts/"
  },
  "docker_images": {
    "backend": "bitcoin-custody-backend:$ENVIRONMENT-$TIMESTAMP",
    "frontend": "bitcoin-custody-frontend:$ENVIRONMENT-$TIMESTAMP"
  }
}
EOF
    
    success "Build manifest generated: $BUILD_DIR/build-manifest-$TIMESTAMP.json"
}

# Main build function
main() {
    log "Starting build process..."
    log "Component: $COMPONENT"
    log "Environment: $ENVIRONMENT"
    log "Build ID: $TIMESTAMP"
    
    create_build_dir
    
    case $COMPONENT in
        frontend)
            build_frontend
            ;;
        backend)
            build_backend
            ;;
        soroban)
            build_soroban
            ;;
        docker)
            build_docker
            ;;
        all)
            build_frontend
            build_backend
            build_soroban
            build_docker
            ;;
        *)
            error "Unknown component: $COMPONENT"
            echo "Available components: frontend, backend, soroban, docker, all"
            exit 1
            ;;
    esac
    
    generate_manifest
    
    success "Build process completed successfully!"
    log "Build artifacts available in: $BUILD_DIR/artifacts/"
    log "Build manifest: $BUILD_DIR/build-manifest-$TIMESTAMP.json"
}

# Show usage if help requested
if [ "$1" = "--help" ] || [ "$1" = "-h" ]; then
    echo "Usage: $0 [component] [environment]"
    echo ""
    echo "Components:"
    echo "  frontend  - Build React frontend only"
    echo "  backend   - Build Loco.rs backend only"
    echo "  soroban   - Build Soroban contracts only"
    echo "  docker    - Build Docker images only"
    echo "  all       - Build all components (default)"
    echo ""
    echo "Environments:"
    echo "  development - Development build with fast compilation"
    echo "  staging     - Staging build with optimizations"
    echo "  production  - Production build with full optimizations (default)"
    echo ""
    echo "Environment Variables:"
    echo "  CLEAN_CACHE=true        - Clean build caches before building"
    echo "  MONITOR_PERFORMANCE=true - Enable build performance monitoring"
    echo "  PARALLEL_BUILD=true     - Enable parallel component builds"
    echo ""
    echo "Examples:"
    echo "  $0                              # Build all components for production"
    echo "  $0 frontend                     # Build frontend for production"
    echo "  $0 backend development          # Build backend for development"
    echo "  CLEAN_CACHE=true $0 all         # Clean build all components"
    echo "  MONITOR_PERFORMANCE=true $0     # Build with performance monitoring"
    exit 0
fi

# Run main function
main "$@"