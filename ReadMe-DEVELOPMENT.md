# Bitcoin Custody Development Environment

This document provides comprehensive instructions for setting up and using the Bitcoin Custody full-stack development environment.

## Quick Start

1. **Setup the environment:**
   ```bash
   chmod +x scripts/*.sh
   ./scripts/dev-setup.sh
   ```

2. **Start all services:**
   ```bash
   ./scripts/dev-start.sh
   ```

3. **Access the application:**
   - Frontend: http://localhost:3000
   - Backend API: http://localhost:8080
   - Soroban RPC: http://localhost:8000

## Architecture Overview

The development environment consists of five main services:

```
┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│   Frontend  │    │   Backend   │    │   Soroban   │
│   (React)   │◄──►│  (Loco.rs)  │◄──►│    (RPC)    │
│   :3000     │    │   :8080     │    │   :8000     │
└─────────────┘    └─────────────┘    └─────────────┘
                           │
                   ┌───────┴───────┐
                   │               │
            ┌─────────────┐ ┌─────────────┐
            │ PostgreSQL  │ │    Redis    │
            │   :5432     │ │   :6379     │
            └─────────────┘ └─────────────┘
```

## Service Details

### Frontend (React + TypeScript)
- **Port:** 3000
- **Technology:** React 18, TypeScript, Vite, Tailwind CSS
- **Hot Reload:** Enabled
- **Source:** `./frontend/src`

### Backend (Loco.rs)
- **Port:** 8080
- **Technology:** Rust, Loco.rs framework, Sea-ORM
- **Hot Reload:** Enabled with cargo-watch
- **Source:** `./backend/src`

### Soroban RPC
- **Port:** 8000
- **Technology:** Stellar Soroban local network
- **Purpose:** Smart contract development and testing

### PostgreSQL Database
- **Port:** 5432
- **Database:** `bitcoin_custody_dev`
- **Credentials:** `postgres:password`

### Redis Cache
- **Port:** 6379
- **Purpose:** Session management and caching

## Development Scripts

### Core Scripts

| Script | Purpose |
|--------|---------|
| `./scripts/dev-setup.sh` | Initial environment setup |
| `./scripts/dev-start.sh` | Start all services |
| `./scripts/dev-stop.sh` | Stop all services |
| `./scripts/dev-status.sh` | Check service status |
| `./scripts/dev-logs.sh [service]` | View service logs |
| `./scripts/health-check.sh` | Comprehensive health check |

### Usage Examples

```bash
# Start development environment
./scripts/dev-start.sh

# Check status of all services
./scripts/dev-status.sh

# View backend logs
./scripts/dev-logs.sh backend

# View all logs
./scripts/dev-logs.sh

# Check system health
./scripts/health-check.sh

# Stop environment
./scripts/dev-stop.sh
```

## Service Communication

### Frontend ↔ Backend
- **Protocol:** HTTP REST API + WebSocket
- **Base URL:** `http://localhost:8080/api`
- **WebSocket:** `ws://localhost:8080/ws`
- **Authentication:** JWT tokens

### Backend ↔ Soroban
- **Protocol:** Soroban SDK + RPC calls
- **RPC URL:** `http://localhost:8000`
- **Network:** Standalone local network

### Backend ↔ Database
- **Protocol:** PostgreSQL connection via Sea-ORM
- **Connection:** `postgres://postgres:password@postgres:5432/bitcoin_custody_dev`

## Environment Configuration

### Environment Files

| File | Purpose |
|------|---------|
| `.env.development` | Shared development configuration |
| `backend/.env` | Backend-specific variables |
| `frontend/.env.development` | Frontend-specific variables |

### Key Environment Variables

```bash
# Database
DATABASE_URL=postgres://postgres:password@localhost:5432/bitcoin_custody_dev

# API Endpoints
VITE_API_URL=http://localhost:8080
VITE_WS_URL=ws://localhost:8080
VITE_SOROBAN_RPC_URL=http://localhost:8000

# Security (Development Only)
JWT_SECRET=development-secret-key-change-in-production

# Logging
RUST_LOG=debug
LOG_LEVEL=debug
```

## Development Workflow

### 1. Making Changes

**Frontend Changes:**
- Edit files in `./frontend/src`
- Changes are automatically hot-reloaded
- TypeScript compilation happens in real-time

**Backend Changes:**
- Edit files in `./backend/src`
- cargo-watch automatically rebuilds and restarts
- Database migrations run automatically

**Smart Contract Changes:**
- Edit files in `./soroban/contracts`
- Rebuild contracts: `cd soroban && cargo build --target wasm32-unknown-unknown --release`
- Redeploy contracts as needed

### 2. Database Operations

```bash
# Run migrations
docker-compose exec backend cargo loco db migrate

# Reset database
docker-compose exec backend cargo loco db reset

# Create new migration
docker-compose exec backend cargo loco generate migration create_new_table

# Access database directly
docker-compose exec postgres psql -U postgres -d bitcoin_custody_dev
```

### 3. Testing

```bash
# Run backend tests
docker-compose exec backend cargo test

# Run frontend tests
docker-compose exec frontend npm test

# Run Soroban contract tests
cd soroban && cargo test
```

## Debugging

### Viewing Logs

```bash
# All services
docker-compose logs -f

# Specific service
docker-compose logs -f backend
docker-compose logs -f frontend
docker-compose logs -f postgres
```

### Common Issues

**Port Already in Use:**
```bash
# Check what's using the port
lsof -i :3000
lsof -i :8080

# Stop conflicting services
./scripts/dev-stop.sh
```

**Database Connection Issues:**
```bash
# Check PostgreSQL status
docker-compose exec postgres pg_isready -U postgres

# Reset database
docker-compose down -v
./scripts/dev-start.sh
```

**Frontend Build Issues:**
```bash
# Clear node_modules and reinstall
docker-compose exec frontend rm -rf node_modules package-lock.json
docker-compose exec frontend npm install
```

## Advanced Features

### Optional Services

Enable additional development tools:

```bash
# Start with database admin tools
docker-compose --profile tools up -d

# Start with Traefik reverse proxy
docker-compose --profile traefik up -d
```

**Available Tools:**
- **PgAdmin:** http://localhost:5050 (admin@localhost / admin)
- **Redis Commander:** http://localhost:8081
- **Traefik Dashboard:** http://localhost:8090

### Service Discovery

Services can communicate using Docker network names:

```bash
# From backend to database
postgres://postgres:password@postgres:5432/bitcoin_custody_dev

# From frontend to backend (in container)
http://backend:8080/api

# From backend to Soroban
http://soroban-rpc:8000
```

### Health Checks

All services include health checks:

```bash
# Manual health check
curl http://localhost:8080/api/health
curl http://localhost:8000/health
curl http://localhost:3000

# Automated health monitoring
./scripts/health-check.sh
```

## Production Considerations

This development environment is optimized for development and includes:

- Debug logging enabled
- Hot reloading for faster development
- Insecure default passwords
- Development-only JWT secrets
- Exposed database ports

**Never use this configuration in production!**

For production deployment, use:
- Secure passwords and secrets
- Production-optimized builds
- Proper SSL/TLS certificates
- Restricted network access
- Production logging configuration

## Troubleshooting

### Reset Everything

```bash
# Complete reset (removes all data)
./scripts/dev-stop.sh
docker-compose down -v --remove-orphans
docker system prune -f
./scripts/dev-setup.sh
./scripts/dev-start.sh
```

### Performance Issues

```bash
# Check resource usage
docker stats

# Limit resource usage (add to docker-compose.yml)
services:
  backend:
    deploy:
      resources:
        limits:
          memory: 1G
          cpus: '0.5'
```

### Network Issues

```bash
# Check network connectivity
docker network ls
docker network inspect bitcoin-custody_bitcoin-custody-network

# Recreate network
docker-compose down
docker network prune
./scripts/dev-start.sh
```

## Contributing

1. Make sure all services are healthy: `./scripts/health-check.sh`
2. Run tests before committing: `docker-compose exec backend cargo test`
3. Check logs for errors: `./scripts/dev-logs.sh`
4. Follow the established patterns for service communication

For more information, see the individual README files in each component directory.