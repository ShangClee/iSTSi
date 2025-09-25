#!/bin/bash
# Unified deployment script for all environments
# Usage: ./scripts/deploy.sh [environment] [component] [options]
# Environments: development, staging, production
# Components: frontend, backend, soroban, all (default)

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Default values
ENVIRONMENT=${1:-development}
COMPONENT=${2:-all}
DRY_RUN=${3:-false}
TIMESTAMP=$(date +%Y%m%d_%H%M%S)

# Configuration
DEPLOY_DIR="deployments"
BUILD_DIR="build"

# Logging functions
log() {
    echo -e "${BLUE}[DEPLOY]${NC} $1"
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

# Validate environment
validate_environment() {
    case $ENVIRONMENT in
        development|staging|production)
            log "Deploying to $ENVIRONMENT environment"
            ;;
        *)
            error "Invalid environment: $ENVIRONMENT"
            echo "Valid environments: development, staging, production"
            exit 1
            ;;
    esac
}

# Load environment configuration
load_config() {
    local config_file="deployments/config/$ENVIRONMENT.env"
    
    if [ -f "$config_file" ]; then
        log "Loading configuration from $config_file"
        source "$config_file"
    else
        warn "Configuration file not found: $config_file"
        warn "Using default configuration"
    fi
    
    # Set default values if not provided
    export DEPLOY_HOST=${DEPLOY_HOST:-localhost}
    export DEPLOY_USER=${DEPLOY_USER:-deploy}
    export DEPLOY_PATH=${DEPLOY_PATH:-/opt/bitcoin-custody}
    export DATABASE_URL=${DATABASE_URL:-postgres://postgres:password@localhost:5432/bitcoin_custody_$ENVIRONMENT}
    export FRONTEND_URL=${FRONTEND_URL:-http://localhost:3000}
    export BACKEND_URL=${BACKEND_URL:-http://localhost:8080}
    export SOROBAN_NETWORK=${SOROBAN_NETWORK:-testnet}
}

# Pre-deployment checks
pre_deployment_checks() {
    log "Running pre-deployment checks..."
    
    # Check if build artifacts exist
    if [ ! -d "$BUILD_DIR/artifacts" ]; then
        error "Build artifacts not found. Please run ./scripts/build.sh first"
        exit 1
    fi
    
    # Check Docker daemon for container deployments
    if [ "$ENVIRONMENT" != "development" ]; then
        if ! docker info >/dev/null 2>&1; then
            error "Docker daemon not running. Required for $ENVIRONMENT deployment"
            exit 1
        fi
    fi
    
    # Check deployment target connectivity
    if [ "$ENVIRONMENT" != "development" ] && [ "$DEPLOY_HOST" != "localhost" ]; then
        log "Testing connectivity to deployment target: $DEPLOY_HOST"
        if ! ssh -o ConnectTimeout=10 "$DEPLOY_USER@$DEPLOY_HOST" "echo 'Connection test successful'" >/dev/null 2>&1; then
            error "Cannot connect to deployment target: $DEPLOY_USER@$DEPLOY_HOST"
            exit 1
        fi
    fi
    
    success "Pre-deployment checks passed"
}

# Deploy frontend
deploy_frontend() {
    log "Deploying frontend to $ENVIRONMENT..."
    
    case $ENVIRONMENT in
        development)
            deploy_frontend_development
            ;;
        staging|production)
            deploy_frontend_production
            ;;
    esac
    
    success "Frontend deployment completed"
}

# Deploy frontend for development
deploy_frontend_development() {
    log "Starting frontend development server..."
    
    if [ "$DRY_RUN" = "true" ]; then
        log "[DRY RUN] Would start frontend development server"
        return
    fi
    
    cd frontend
    
    # Install dependencies if needed
    if [ ! -d "node_modules" ]; then
        log "Installing frontend dependencies..."
        npm ci
    fi
    
    # Start development server in background
    log "Starting Vite development server..."
    npm run dev &
    echo $! > "../$DEPLOY_DIR/frontend.pid"
    
    cd ..
}

# Deploy frontend for production/staging
deploy_frontend_production() {
    log "Deploying frontend static files..."
    
    local latest_build=$(ls -t "$BUILD_DIR/artifacts"/frontend-$ENVIRONMENT-*.tar.gz | head -n1)
    
    if [ -z "$latest_build" ]; then
        error "No frontend build artifact found for $ENVIRONMENT"
        exit 1
    fi
    
    if [ "$DRY_RUN" = "true" ]; then
        log "[DRY RUN] Would deploy frontend from: $latest_build"
        return
    fi
    
    # Extract build artifact
    local temp_dir=$(mktemp -d)
    tar -xzf "$latest_build" -C "$temp_dir"
    
    if [ "$DEPLOY_HOST" = "localhost" ]; then
        # Local deployment
        log "Deploying frontend locally..."
        mkdir -p "$DEPLOY_PATH/frontend"
        cp -r "$temp_dir/dist/"* "$DEPLOY_PATH/frontend/"
        
        # Start nginx or serve static files
        if command -v nginx >/dev/null 2>&1; then
            log "Configuring nginx for frontend..."
            # Generate nginx config
            cat > "/tmp/nginx-frontend-$ENVIRONMENT.conf" << EOF
server {
    listen 80;
    server_name ${FRONTEND_DOMAIN:-localhost};
    root $DEPLOY_PATH/frontend;
    index index.html;
    
    location / {
        try_files \$uri \$uri/ /index.html;
    }
    
    location /api {
        proxy_pass $BACKEND_URL;
        proxy_set_header Host \$host;
        proxy_set_header X-Real-IP \$remote_addr;
    }
}
EOF
        fi
    else
        # Remote deployment
        log "Deploying frontend to remote server: $DEPLOY_HOST"
        rsync -avz --delete "$temp_dir/dist/" "$DEPLOY_USER@$DEPLOY_HOST:$DEPLOY_PATH/frontend/"
    fi
    
    rm -rf "$temp_dir"
}

# Deploy backend
deploy_backend() {
    log "Deploying backend to $ENVIRONMENT..."
    
    case $ENVIRONMENT in
        development)
            deploy_backend_development
            ;;
        staging|production)
            deploy_backend_production
            ;;
    esac
    
    success "Backend deployment completed"
}

# Deploy backend for development
deploy_backend_development() {
    log "Starting backend development server..."
    
    if [ "$DRY_RUN" = "true" ]; then
        log "[DRY RUN] Would start backend development server"
        return
    fi
    
    cd backend
    
    # Run database migrations
    log "Running database migrations..."
    cargo loco db migrate --environment development
    
    # Start development server
    log "Starting Loco.rs development server..."
    cargo loco start --environment development &
    echo $! > "../$DEPLOY_DIR/backend.pid"
    
    cd ..
}

# Deploy backend for production/staging
deploy_backend_production() {
    log "Deploying backend service..."
    
    if [ "$DRY_RUN" = "true" ]; then
        log "[DRY RUN] Would deploy backend service"
        return
    fi
    
    # Deploy using Docker
    log "Deploying backend with Docker..."
    
    # Create docker-compose override for environment
    cat > "docker-compose.$ENVIRONMENT.yml" << EOF
version: '3.8'

services:
  backend:
    image: bitcoin-custody-backend:$ENVIRONMENT-latest
    environment:
      - DATABASE_URL=$DATABASE_URL
      - RUST_LOG=info
      - LOCO_ENV=$ENVIRONMENT
    ports:
      - "8080:8080"
    restart: unless-stopped
    
  postgres:
    image: postgres:15
    environment:
      - POSTGRES_DB=bitcoin_custody_$ENVIRONMENT
      - POSTGRES_USER=postgres
      - POSTGRES_PASSWORD=$DB_PASSWORD
    volumes:
      - postgres_data_$ENVIRONMENT:/var/lib/postgresql/data
    restart: unless-stopped

volumes:
  postgres_data_$ENVIRONMENT:
EOF
    
    # Deploy with docker-compose
    docker-compose -f docker-compose.yml -f "docker-compose.$ENVIRONMENT.yml" up -d backend postgres
    
    # Wait for services to be ready
    log "Waiting for services to be ready..."
    sleep 30
    
    # Run database migrations
    log "Running database migrations..."
    docker-compose -f docker-compose.yml -f "docker-compose.$ENVIRONMENT.yml" exec -T backend cargo loco db migrate --environment $ENVIRONMENT
}

# Deploy Soroban contracts
deploy_soroban() {
    log "Deploying Soroban contracts to $ENVIRONMENT..."
    
    if [ "$DRY_RUN" = "true" ]; then
        log "[DRY RUN] Would deploy Soroban contracts"
        return
    fi
    
    cd soroban
    
    # Set network configuration
    case $ENVIRONMENT in
        development)
            SOROBAN_NETWORK_URL="http://localhost:8000/soroban/rpc"
            SOROBAN_NETWORK_PASSPHRASE="Standalone Network ; February 2017"
            ;;
        staging)
            SOROBAN_NETWORK_URL="https://soroban-testnet.stellar.org"
            SOROBAN_NETWORK_PASSPHRASE="Test SDF Network ; September 2015"
            ;;
        production)
            SOROBAN_NETWORK_URL="https://soroban-mainnet.stellar.org"
            SOROBAN_NETWORK_PASSPHRASE="Public Global Stellar Network ; September 2015"
            ;;
    esac
    
    # Configure soroban CLI
    log "Configuring Soroban network: $SOROBAN_NETWORK"
    soroban config network add "$ENVIRONMENT" \
        --rpc-url "$SOROBAN_NETWORK_URL" \
        --network-passphrase "$SOROBAN_NETWORK_PASSPHRASE"
    
    # Deploy contracts
    log "Deploying contracts to $SOROBAN_NETWORK network..."
    
    # Find optimized contract files
    local contracts_dir="../$BUILD_DIR/artifacts/contracts"
    if [ ! -d "$contracts_dir" ]; then
        error "Contract artifacts not found. Please run build first."
        exit 1
    fi
    
    # Deploy each contract
    local deployment_addresses=()
    for wasm_file in "$contracts_dir"/*-$ENVIRONMENT-*.wasm; do
        if [ -f "$wasm_file" ]; then
            local contract_name=$(basename "$wasm_file" | cut -d'-' -f1)
            log "Deploying contract: $contract_name"
            
            local contract_address=$(soroban contract deploy \
                --wasm "$wasm_file" \
                --network "$ENVIRONMENT" \
                --source-account "$SOROBAN_DEPLOY_ACCOUNT")
            
            log "Contract $contract_name deployed at: $contract_address"
            deployment_addresses+=("$contract_name:$contract_address")
        fi
    done
    
    # Save deployment addresses
    local addresses_file="../$DEPLOY_DIR/contract-addresses-$ENVIRONMENT-$TIMESTAMP.json"
    log "Saving contract addresses to: $addresses_file"
    
    echo "{" > "$addresses_file"
    echo "  \"network\": \"$ENVIRONMENT\"," >> "$addresses_file"
    echo "  \"timestamp\": \"$(date -u +%Y-%m-%dT%H:%M:%SZ)\"," >> "$addresses_file"
    echo "  \"contracts\": {" >> "$addresses_file"
    
    local first=true
    for addr in "${deployment_addresses[@]}"; do
        local name=$(echo "$addr" | cut -d':' -f1)
        local address=$(echo "$addr" | cut -d':' -f2)
        
        if [ "$first" = true ]; then
            first=false
        else
            echo "," >> "$addresses_file"
        fi
        
        echo "    \"$name\": \"$address\"" >> "$addresses_file"
    done
    
    echo "  }" >> "$addresses_file"
    echo "}" >> "$addresses_file"
    
    cd ..
    success "Soroban contracts deployed successfully"
}

# Health check after deployment
post_deployment_health_check() {
    log "Running post-deployment health checks..."
    
    if [ "$DRY_RUN" = "true" ]; then
        log "[DRY RUN] Would run health checks"
        return
    fi
    
    # Wait for services to stabilize
    sleep 10
    
    # Check frontend
    if [ "$COMPONENT" = "all" ] || [ "$COMPONENT" = "frontend" ]; then
        log "Checking frontend health..."
        if curl -f "$FRONTEND_URL" >/dev/null 2>&1; then
            success "Frontend is responding"
        else
            warn "Frontend health check failed"
        fi
    fi
    
    # Check backend
    if [ "$COMPONENT" = "all" ] || [ "$COMPONENT" = "backend" ]; then
        log "Checking backend health..."
        if curl -f "$BACKEND_URL/health" >/dev/null 2>&1; then
            success "Backend is responding"
        else
            warn "Backend health check failed"
        fi
    fi
    
    success "Post-deployment health checks completed"
}

# Generate deployment report
generate_deployment_report() {
    log "Generating deployment report..."
    
    local report_file="$DEPLOY_DIR/deployment-report-$ENVIRONMENT-$TIMESTAMP.json"
    
    cat > "$report_file" << EOF
{
  "deployment_id": "$TIMESTAMP",
  "environment": "$ENVIRONMENT",
  "component": "$COMPONENT",
  "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "git_commit": "$(git rev-parse HEAD 2>/dev/null || echo 'unknown')",
  "git_branch": "$(git rev-parse --abbrev-ref HEAD 2>/dev/null || echo 'unknown')",
  "deployed_by": "$(whoami)",
  "deployment_host": "$DEPLOY_HOST",
  "dry_run": $DRY_RUN,
  "status": "completed",
  "urls": {
    "frontend": "$FRONTEND_URL",
    "backend": "$BACKEND_URL"
  }
}
EOF
    
    success "Deployment report generated: $report_file"
}

# Main deployment function
main() {
    log "Starting deployment process..."
    log "Environment: $ENVIRONMENT"
    log "Component: $COMPONENT"
    log "Dry Run: $DRY_RUN"
    log "Deployment ID: $TIMESTAMP"
    
    # Create deployment directory
    mkdir -p "$DEPLOY_DIR"
    mkdir -p "$DEPLOY_DIR/logs"
    
    validate_environment
    load_config
    pre_deployment_checks
    
    case $COMPONENT in
        frontend)
            deploy_frontend
            ;;
        backend)
            deploy_backend
            ;;
        soroban)
            deploy_soroban
            ;;
        all)
            deploy_frontend
            deploy_backend
            deploy_soroban
            ;;
        *)
            error "Unknown component: $COMPONENT"
            echo "Available components: frontend, backend, soroban, all"
            exit 1
            ;;
    esac
    
    post_deployment_health_check
    generate_deployment_report
    
    success "Deployment process completed successfully!"
    log "Deployment report: $DEPLOY_DIR/deployment-report-$ENVIRONMENT-$TIMESTAMP.json"
}

# Show usage if help requested
if [ "$1" = "--help" ] || [ "$1" = "-h" ]; then
    echo "Usage: $0 [environment] [component] [--dry-run]"
    echo ""
    echo "Environments:"
    echo "  development - Local development deployment"
    echo "  staging     - Staging environment deployment"
    echo "  production  - Production environment deployment"
    echo ""
    echo "Components:"
    echo "  frontend  - Deploy React frontend only"
    echo "  backend   - Deploy Loco.rs backend only"
    echo "  soroban   - Deploy Soroban contracts only"
    echo "  all       - Deploy all components (default)"
    echo ""
    echo "Options:"
    echo "  --dry-run   - Show what would be deployed without actually deploying"
    echo ""
    echo "Examples:"
    echo "  $0 development           # Deploy all to development"
    echo "  $0 staging frontend      # Deploy frontend to staging"
    echo "  $0 production all --dry-run # Dry run production deployment"
    exit 0
fi

# Check for dry run flag
if [ "$3" = "--dry-run" ] || [ "$4" = "--dry-run" ]; then
    DRY_RUN=true
fi

# Run main function
main "$@"