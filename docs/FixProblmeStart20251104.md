# Fix Problems Start - November 4, 2025

## Overview

This document tracks the changes made to fix startup problems in the Bitcoin Custody Full-Stack Application development environment on November 4, 2025.

## Problems Identified

### 1. Docker Compose Configuration Issues
- **Issue**: Services not starting in correct order
- **Impact**: Backend trying to connect to database before it's ready
- **Symptoms**: Connection refused errors, service startup failures

### 2. Port Mapping Conflicts
- **Issue**: Soroban RPC port conflicts with documentation
- **Impact**: Inconsistent port references between services
- **Symptoms**: Connection errors when frontend tries to reach Soroban

### 3. Health Check Timing
- **Issue**: Health checks too aggressive, services marked unhealthy prematurely
- **Impact**: Services restarting unnecessarily
- **Symptoms**: Intermittent service availability

### 4. Volume Mount Issues
- **Issue**: Cargo build cache not properly persisted
- **Impact**: Slow rebuilds, dependency re-downloads
- **Symptoms**: Long startup times, network timeouts

## Changes Made

### Docker Compose Configuration (`docker-compose.yml`)

#### Service Dependencies
```yaml
# Fixed service startup order
backend:
  depends_on:
    postgres:
      condition: service_healthy
    soroban-rpc:
      condition: service_healthy

frontend:
  depends_on:
    backend:
      condition: service_healthy
```

#### Port Mapping Standardization
```yaml
# Soroban RPC port mapping corrected
soroban-rpc:
  ports:
    - "8001:8000"  # External:Internal
    - "11626:11626"
```

#### Health Check Improvements
```yaml
# Backend health check with longer startup period
backend:
  healthcheck:
    test: ["CMD", "curl", "-f", "http://localhost:8080/api/_health"]
    interval: 15s
    timeout: 10s
    retries: 5
    start_period: 120s  # Increased from 60s

# Soroban health check with proper startup time
soroban-rpc:
  healthcheck:
    test: ["CMD", "curl", "-f", "http://localhost:8000/health"]
    interval: 15s
    timeout: 10s
    retries: 5
    start_period: 60s  # Added proper startup period
```

#### Volume Optimization
```yaml
# Added persistent volumes for better caching
volumes:
  backend_target:
    driver: local
  backend_cargo_registry:
    driver: local
  backend_cargo_git:
    driver: local
  frontend_node_modules:
    driver: local
  frontend_vite_cache:
    driver: local

# Backend volume mounts
backend:
  volumes:
    - ./backend:/app
    - backend_target:/app/target
    - backend_cargo_registry:/usr/local/cargo/registry
    - backend_cargo_git:/usr/local/cargo/git
```

#### Environment Variables Cleanup
```yaml
# Backend environment standardization
backend:
  environment:
    - DATABASE_URL=postgres://postgres:password@postgres:5432/bitcoin_custody_dev
    - RUST_LOG=debug
    - SOROBAN_RPC_URL=http://soroban-rpc:8000
    - SOROBAN_NETWORK_PASSPHRASE=Standalone Network ; February 2017
    - JWT_SECRET=development-secret-key-change-in-production
    - CORS_ORIGINS=http://localhost:3000,http://frontend:3000
    - CARGO_INCREMENTAL=1
    - CARGO_TARGET_DIR=/app/target

# Frontend environment fixes
frontend:
  environment:
    - VITE_API_URL=http://localhost:8080
    - VITE_WS_URL=ws://localhost:8080
    - VITE_SOROBAN_RPC_URL=http://localhost:8001  # Fixed port reference
    - NODE_ENV=development
    - CHOKIDAR_USEPOLLING=true
    - VITE_CACHE_DIR=/app/node_modules/.vite
```

### Script Improvements

#### Development Setup Script (`scripts/dev-setup.sh`)
- **Added**: Docker and Docker Compose availability checks
- **Added**: Directory structure validation
- **Added**: Automatic environment file creation with sensible defaults
- **Added**: Frontend dependency installation for IDE support
- **Added**: Database initialization script creation

#### Development Start Script (`scripts/dev-start.sh`)
- **Added**: Staged service startup (infrastructure first, then applications)
- **Added**: Health check validation with proper timeouts
- **Added**: Database migration execution before starting applications
- **Added**: Comprehensive error handling and logging
- **Added**: Service URL display for easy access

### Makefile Enhancements

#### New Commands Added
```makefile
# Service-specific log viewing
logs-backend: ## View backend logs only
logs-frontend: ## View frontend logs only
logs-postgres: ## View PostgreSQL logs only
logs-soroban: ## View Soroban RPC logs only

# Service management
restart-backend: ## Restart backend service
restart-frontend: ## Restart frontend service
restart-postgres: ## Restart PostgreSQL service

# Database operations
db-migrate: ## Run database migrations
db-reset: ## Reset database
db-shell: ## Access database shell
```

## Network Configuration

### Custom Bridge Network
```yaml
networks:
  bitcoin-custody-network:
    driver: bridge
    ipam:
      config:
        - subnet: 172.20.0.0/16
```

**Benefits**:
- Isolated network for all services
- Predictable IP addressing
- Better service discovery
- Reduced conflicts with host networking

## Testing and Validation

### Health Check Endpoints
- **Backend**: `http://localhost:8080/api/_health`
- **Frontend**: `http://localhost:3000`
- **Soroban RPC**: `http://localhost:8000/health`
- **PostgreSQL**: `pg_isready -U postgres -d bitcoin_custody_dev`
- **Redis**: `redis-cli ping`

### Service Startup Order
1. **Infrastructure Services**: PostgreSQL, Redis, Soroban RPC
2. **Database Migration**: Run migrations once infrastructure is ready
3. **Application Services**: Backend, Frontend

### Port Mapping Summary
| Service | Internal Port | External Port | Purpose |
|---------|---------------|---------------|---------|
| Frontend | 3000 | 3000 | React development server |
| Backend | 8080 | 8080 | Loco.rs API server |
| Soroban RPC | 8000 | 8001 | Stellar blockchain RPC |
| Stellar Core | 11626 | 11626 | Stellar network protocol |
| PostgreSQL | 5432 | 5432 | Database server |
| Redis | 6379 | 6379 | Cache and sessions |

## Performance Improvements

### Build Caching
- **Cargo Registry Cache**: Persistent volume for Rust dependencies
- **Cargo Git Cache**: Persistent volume for Git-based dependencies
- **Target Directory**: Persistent volume for compiled artifacts
- **Node Modules**: Persistent volume for npm dependencies
- **Vite Cache**: Persistent volume for frontend build cache

### Development Experience
- **Hot Reloading**: Enabled for both frontend and backend
- **File Watching**: Optimized with polling for cross-platform compatibility
- **Incremental Builds**: Enabled for faster Rust compilation
- **Source Maps**: Enabled for better debugging

## Troubleshooting Guide

### Common Issues and Solutions

#### Services Won't Start
```bash
# Check Docker daemon
docker info

# Clean up and restart
make clean
make setup
make start
```

#### Port Conflicts
```bash
# Check what's using ports
lsof -i :3000
lsof -i :8080
lsof -i :8001

# Stop conflicting services
make stop
```

#### Database Connection Issues
```bash
# Check PostgreSQL health
docker-compose exec postgres pg_isready -U postgres

# Reset database
make db-reset
```

#### Slow Startup Times
```bash
# Check volume usage
docker volume ls
docker system df

# Clean up unused resources
docker system prune -f
```

## Monitoring and Logging

### Log Aggregation
- All services log to Docker Compose logs
- Structured logging with timestamps
- Color-coded output for better readability
- Service-specific log filtering available

### Health Monitoring
- Automated health checks for all services
- Configurable retry policies
- Graceful degradation on service failures
- Comprehensive status reporting

## Future Improvements

### Planned Enhancements
1. **Service Mesh**: Consider Traefik for advanced routing
2. **Observability**: Add Prometheus and Grafana for metrics
3. **Security**: Implement proper secrets management
4. **CI/CD**: Add GitHub Actions for automated testing
5. **Documentation**: Auto-generate API documentation

### Performance Optimizations
1. **Multi-stage Builds**: Optimize Docker image sizes
2. **Build Parallelization**: Improve build times
3. **Resource Limits**: Add memory and CPU constraints
4. **Load Balancing**: Prepare for horizontal scaling

## Docker Compose Version Warning Fix

### Issue
Docker Compose was showing warnings about obsolete `version` attribute:
```
WARN[0000] /Users/shang/Prj2025/MintToken/iSTSi/docker-compose.yml: the attribute `version` is obsolete, it will be ignored, please remove it to avoid potential confusion
WARN[0000] /Users/shang/Prj2025/MintToken/iSTSi/docker-compose.override.yml: the attribute `version` is obsolete, it will be ignored, please remove it to avoid potential confusion
```

### Solution
Removed the obsolete `version: '3.8'` attribute from both files and added update comments:

**docker-compose.yml**:
```yaml
# Docker Compose configuration for Bitcoin Custody Full-Stack Application
# Updated: November 4, 2025 - Removed obsolete version attribute

services:
  # ... rest of configuration
```

**docker-compose.override.yml**:
```yaml
# Docker Compose override for development-specific configurations
# This file extends docker-compose.yml with development-only settings
# Updated: November 4, 2025 - Removed obsolete version attribute

services:
  # ... rest of configuration
```

### Background
The `version` attribute in Docker Compose files became obsolete in newer versions of Docker Compose. The Compose file format is now automatically detected based on the features used in the file, making the version specification unnecessary and potentially confusing.

## Validation Checklist

- [x] All services start successfully
- [x] Health checks pass for all services
- [x] Database migrations run automatically
- [x] Frontend can communicate with backend
- [x] Backend can communicate with Soroban RPC
- [x] Hot reloading works for development
- [x] Build caching improves startup times
- [x] Error handling provides useful feedback
- [x] Documentation is up to date
- [x] Scripts are executable and robust
- [x] Docker Compose version warnings resolved

## Conclusion

The startup problems have been resolved through:
1. **Proper service orchestration** with health checks and dependencies
2. **Consistent port mapping** and environment configuration
3. **Optimized build caching** for faster development cycles
4. **Robust error handling** and logging for better debugging
5. **Comprehensive documentation** for maintainability

The development environment now provides a reliable, fast, and developer-friendly experience for working with the Bitcoin Custody Full-Stack Application.

---

**Document Version**: 1.0  
**Last Updated**: November 4, 2025  
**Author**: Development Team  
**Status**: Complete