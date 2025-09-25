# Migration Guide: Project Restructure

This guide helps existing developers and deployments migrate from the old project structure to the new organized architecture.

## Overview

The project has been restructured from a mixed directory layout to a clean monorepo with separate components:

### Before (Old Structure)
```
project/
├── uxui/                    # React frontend
├── contracts/               # Soroban contracts
├── various config files
└── mixed documentation
```

### After (New Structure)
```
project/
├── frontend/               # React + TypeScript frontend (from uxui/)
├── backend/                # Loco.rs + PostgreSQL backend (new)
├── soroban/               # Soroban smart contracts (from contracts/)
├── scripts/               # Build and deployment scripts
├── docs/                  # Documentation
└── docker-compose.yml     # Development environment
```

## Migration Steps

### For Developers

#### 1. Update Local Environment

**Clean up old setup:**
```bash
# Stop any running services
docker-compose down
pkill -f "npm\|cargo\|soroban"

# Clean Docker resources
docker system prune -f
```

**Set up new environment:**
```bash
# Pull latest changes
git pull origin main

# Setup new development environment
make setup
# or
./scripts/dev-setup.sh

# Start new services
make start
```

#### 2. Update Development Workflow

**Old workflow:**
```bash
# Frontend development
cd uxui && npm run dev

# Contract development  
cd contracts && cargo build --target wasm32-unknown-unknown --release
```

**New workflow:**
```bash
# Full-stack development (recommended)
make start                    # Starts all services with hot reload

# Component-specific development
make logs-frontend           # Monitor frontend
make logs-backend           # Monitor backend
make test-contracts         # Test contracts
```

#### 3. Update IDE Configuration

**VS Code settings:**
- Update workspace folders to point to `frontend/`, `backend/`, `soroban/`
- Update TypeScript paths in `frontend/tsconfig.json`
- Update Rust analyzer settings for `backend/` and `soroban/`

**File paths:**
- Frontend code: `uxui/src/` → `frontend/src/`
- Contract code: `contracts/` → `soroban/contracts/`
- New backend code: `backend/src/`

#### 4. Update Import Paths

**Frontend imports (if you have local modifications):**
```typescript
// Old
import { Component } from '../components/Component'

// New (same - no changes needed)
import { Component } from '../components/Component'
```

**Contract references:**
```rust
// Old
use contracts::integration_router::IntegrationRouter;

// New  
use soroban::contracts::integration_router::IntegrationRouter;
```

### For Deployments

#### 1. CI/CD Pipeline Updates

**GitHub Actions (already updated):**
- Workflows now build `frontend/`, `backend/`, `soroban/` separately
- Artifacts are organized by component
- Integration tests run across all components

**Custom CI/CD:**
```yaml
# Update your pipeline to use new paths
build-frontend:
  script:
    - cd frontend && npm ci && npm run build

build-backend:
  script:
    - cd backend && cargo build --release

build-contracts:
  script:
    - cd soroban && cargo build --target wasm32-unknown-unknown --release
```

#### 2. Docker Deployment

**Old Dockerfile references:**
```dockerfile
COPY uxui/ ./frontend/
COPY contracts/ ./contracts/
```

**New Dockerfile references:**
```dockerfile
COPY frontend/ ./frontend/
COPY backend/ ./backend/
COPY soroban/ ./soroban/
```

#### 3. Environment Variables

**New backend environment variables:**
```bash
# Database
DATABASE_URL=postgres://user:pass@localhost:5432/bitcoin_custody

# JWT
JWT_SECRET=your-secret-key

# Soroban
SOROBAN_NETWORK=testnet
SOROBAN_RPC_URL=https://soroban-testnet.stellar.org
```

### For Production Deployments

#### 1. Backup Current Deployment

```bash
# Backup current deployment
kubectl get all -o yaml > backup-deployment.yaml
# or for Docker
docker-compose config > backup-compose.yaml
```

#### 2. Update Deployment Configuration

**Kubernetes:**
```yaml
# Update service definitions
apiVersion: apps/v1
kind: Deployment
metadata:
  name: frontend
spec:
  template:
    spec:
      containers:
      - name: frontend
        image: your-registry/bitcoin-custody-frontend:latest
        # Update build context and paths

---
apiVersion: apps/v1  
kind: Deployment
metadata:
  name: backend
spec:
  template:
    spec:
      containers:
      - name: backend
        image: your-registry/bitcoin-custody-backend:latest
        # New backend service
```

**Docker Compose:**
```yaml
version: '3.8'
services:
  frontend:
    build: ./frontend
    ports:
      - "3000:3000"
  
  backend:
    build: ./backend
    ports:
      - "8080:8080"
    environment:
      - DATABASE_URL=postgres://postgres:password@postgres:5432/bitcoin_custody
  
  postgres:
    image: postgres:15
    environment:
      POSTGRES_DB: bitcoin_custody
      POSTGRES_USER: postgres
      POSTGRES_PASSWORD: password
```

#### 3. Database Migration

**New backend includes database migrations:**
```bash
# Run migrations in production
cd backend && cargo loco db migrate

# Or in Docker
docker-compose exec backend cargo loco db migrate
```

## Verification Steps

### 1. Development Environment

```bash
# Verify all services start
make start

# Check service health
make health

# Verify frontend loads
curl http://localhost:3000

# Verify backend API
curl http://localhost:8080/api/system/health

# Verify contracts build
cd soroban && cargo build --target wasm32-unknown-unknown --release
```

### 2. Integration Testing

```bash
# Run full test suite
make test

# Run integration tests
make test-integration

# Verify cross-component communication
./scripts/test.sh integration
```

### 3. Production Readiness

```bash
# Build production artifacts
make build-production

# Run security scan
make security-scan

# Performance test
make performance-test
```

## Rollback Plan

If issues occur, you can rollback using the backup:

### Development Rollback

```bash
# Stop new services
make stop

# Restore from backup (if backup was created)
# Note: The old structure backup was cleaned up after successful migration
# You would need to restore from git history if needed

git checkout <previous-commit>
```

### Production Rollback

```bash
# Restore previous deployment
kubectl apply -f backup-deployment.yaml
# or
docker-compose -f backup-compose.yaml up -d
```

## Common Issues and Solutions

### Issue: Port Conflicts

**Problem:** Services won't start due to port conflicts

**Solution:**
```bash
# Check what's using ports
lsof -i :3000
lsof -i :8080

# Stop conflicting services
make stop
pkill -f "node\|cargo"

# Restart
make start
```

### Issue: Database Connection Errors

**Problem:** Backend can't connect to database

**Solution:**
```bash
# Check database status
make status

# Reset database
make db-reset

# Run migrations
make db-migrate
```

### Issue: Contract Build Failures

**Problem:** Soroban contracts won't build

**Solution:**
```bash
# Update Rust toolchain
rustup update

# Add wasm target
rustup target add wasm32-unknown-unknown

# Clean and rebuild
cd soroban && cargo clean && cargo build --target wasm32-unknown-unknown --release
```

### Issue: Frontend Build Errors

**Problem:** Frontend won't build or start

**Solution:**
```bash
# Clear node modules and reinstall
cd frontend
rm -rf node_modules package-lock.json
npm install

# Clear Vite cache
rm -rf node_modules/.vite

# Restart
npm run dev
```

## Getting Help

1. **Check service status:** `make status`
2. **View logs:** `make logs`
3. **Run health check:** `make health`
4. **Review documentation:** Each component has its own README
5. **Check troubleshooting:** [TROUBLESHOOTING.md](./TROUBLESHOOTING.md)

## Post-Migration Checklist

- [ ] All services start successfully
- [ ] Frontend loads at http://localhost:3000
- [ ] Backend API responds at http://localhost:8080
- [ ] Database migrations run successfully
- [ ] Contracts build without errors
- [ ] Integration tests pass
- [ ] CI/CD pipeline works with new structure
- [ ] Production deployment updated
- [ ] Team members updated on new workflow
- [ ] Documentation updated

## Benefits of New Structure

### For Developers
- **Clear separation of concerns** - Each directory has a single responsibility
- **Independent development** - Work on components without interference
- **Better tooling** - Component-specific configurations and optimizations
- **Easier onboarding** - New developers can understand the architecture quickly

### For Operations
- **Independent deployment** - Deploy components separately
- **Better scaling** - Scale components based on load
- **Clearer monitoring** - Component-specific metrics and logs
- **Easier maintenance** - Update components independently

### For the Project
- **Industry standards** - Follows common patterns for full-stack applications
- **Future-proof** - Easier to add new components or migrate to microservices
- **Better testing** - Component-specific and integration testing strategies
- **Improved CI/CD** - Parallel builds and deployments

The migration provides a solid foundation for future development and scaling of the Bitcoin custody system.