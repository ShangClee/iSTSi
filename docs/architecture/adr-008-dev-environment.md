# ADR-008: Development Environment Setup

## Status

Accepted

## Date

2024-01-22

## Context

The development team needs a consistent, reproducible development environment that supports:

- Full-stack development with frontend, backend, and smart contracts
- Database setup and management
- Soroban network simulation for contract testing
- Hot reloading and fast development cycles
- Easy onboarding for new developers
- Cross-platform compatibility (macOS, Linux, Windows)

The environment should minimize setup time and reduce "works on my machine" issues.

## Decision

We will use **Docker Compose** as the primary development environment orchestration tool with the following setup:

### Core Services
- **PostgreSQL** - Database service
- **Redis** - Caching and session storage
- **Soroban RPC** - Local Soroban network for contract testing
- **Frontend Dev Server** - Vite development server with hot reload
- **Backend Dev Server** - Loco.rs server with auto-restart

### Development Tools
- **Docker & Docker Compose** - Container orchestration
- **Make** - Task automation and command shortcuts
- **Scripts** - Shell scripts for common development tasks
- **VS Code Dev Containers** - Optional containerized development

## Implementation

### Docker Compose Configuration

**docker-compose.yml**:
```yaml
version: '3.8'

services:
  postgres:
    image: postgres:15
    environment:
      POSTGRES_DB: bitcoin_custody_dev
      POSTGRES_USER: postgres
      POSTGRES_PASSWORD: password
    ports:
      - "5432:5432"
    volumes:
      - postgres_data:/var/lib/postgresql/data
      - ./backend/scripts/init-db.sql:/docker-entrypoint-initdb.d/init-db.sql
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U postgres"]
      interval: 10s
      timeout: 5s
      retries: 5

  redis:
    image: redis:7-alpine
    ports:
      - "6379:6379"
    volumes:
      - redis_data:/data
    healthcheck:
      test: ["CMD", "redis-cli", "ping"]
      interval: 10s
      timeout: 5s
      retries: 5

  soroban-rpc:
    image: stellar/quickstart:soroban-dev
    ports:
      - "8000:8000"
    command: --local --enable-soroban-rpc
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8000/health"]
      interval: 30s
      timeout: 10s
      retries: 3

  backend:
    build:
      context: ./backend
      dockerfile: Dockerfile.dev
    ports:
      - "8080:8080"
    environment:
      DATABASE_URL: postgres://postgres:password@postgres:5432/bitcoin_custody_dev
      REDIS_URL: redis://redis:6379
      SOROBAN_RPC_URL: http://soroban-rpc:8000
      RUST_LOG: debug
    volumes:
      - ./backend:/app
      - backend_target:/app/target
      - backend_cargo:/usr/local/cargo/registry
    depends_on:
      postgres:
        condition: service_healthy
      redis:
        condition: service_healthy
      soroban-rpc:
        condition: service_healthy
    command: cargo loco start
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8080/health"]
      interval: 30s
      timeout: 10s
      retries: 3

  frontend:
    build:
      context: ./frontend
      dockerfile: Dockerfile.dev
    ports:
      - "3000:3000"
    environment:
      VITE_API_URL: http://localhost:8080
      VITE_WS_URL: ws://localhost:8080
    volumes:
      - ./frontend:/app
      - frontend_node_modules:/app/node_modules
    depends_on:
      - backend
    command: npm run dev

volumes:
  postgres_data:
  redis_data:
  backend_target:
  backend_cargo:
  frontend_node_modules:
```

### Development Dockerfiles

**backend/Dockerfile.dev**:
```dockerfile
FROM rust:1.70

WORKDIR /app

# Install system dependencies
RUN apt-get update && apt-get install -y \
    postgresql-client \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Install Soroban CLI
RUN cargo install --locked soroban-cli

# Copy dependency files
COPY Cargo.toml Cargo.lock ./

# Create dummy main.rs to cache dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm -rf src

# Copy source code
COPY . .

# Build application
RUN cargo build --release

EXPOSE 8080

CMD ["cargo", "loco", "start"]
```

**frontend/Dockerfile.dev**:
```dockerfile
FROM node:18-alpine

WORKDIR /app

# Copy package files
COPY package*.json ./

# Install dependencies
RUN npm ci

# Copy source code
COPY . .

EXPOSE 3000

CMD ["npm", "run", "dev", "--", "--host", "0.0.0.0"]
```

### Development Scripts

**Makefile**:
```makefile
.PHONY: setup start stop clean logs test build

# Setup development environment
setup:
	@echo "Setting up development environment..."
	docker-compose build
	docker-compose up -d postgres redis soroban-rpc
	@echo "Waiting for services to be ready..."
	sleep 10
	cd backend && cargo loco db migrate
	@echo "Setup complete!"

# Start all services
start:
	docker-compose up -d
	@echo "Development environment started!"
	@echo "Frontend: http://localhost:3000"
	@echo "Backend: http://localhost:8080"
	@echo "Database: localhost:5432"

# Stop all services
stop:
	docker-compose down

# Clean up everything
clean:
	docker-compose down -v
	docker system prune -f

# View logs
logs:
	docker-compose logs -f

# Run tests
test:
	docker-compose exec backend cargo test
	docker-compose exec frontend npm test

# Build for production
build:
	docker-compose -f docker-compose.prod.yml build
```

**scripts/dev-setup.sh**:
```bash
#!/bin/bash
set -e

echo "🚀 Setting up Bitcoin Custody development environment..."

# Check prerequisites
command -v docker >/dev/null 2>&1 || { echo "❌ Docker is required but not installed. Aborting." >&2; exit 1; }
command -v docker-compose >/dev/null 2>&1 || { echo "❌ Docker Compose is required but not installed. Aborting." >&2; exit 1; }

# Create environment files if they don't exist
if [ ! -f frontend/.env.development ]; then
    echo "📝 Creating frontend environment file..."
    cp frontend/.env.example frontend/.env.development
fi

if [ ! -f backend/.env ]; then
    echo "📝 Creating backend environment file..."
    cp backend/.env.example backend/.env
fi

if [ ! -f soroban/.env ]; then
    echo "📝 Creating soroban environment file..."
    cp soroban/.env.example soroban/.env
fi

# Build and start services
echo "🏗️  Building Docker images..."
docker-compose build

echo "🚀 Starting infrastructure services..."
docker-compose up -d postgres redis soroban-rpc

# Wait for services to be ready
echo "⏳ Waiting for services to be ready..."
sleep 15

# Run database migrations
echo "🗄️  Running database migrations..."
docker-compose exec -T postgres psql -U postgres -d bitcoin_custody_dev -c "SELECT 1;" > /dev/null 2>&1
cd backend && cargo loco db migrate && cd ..

# Build Soroban contracts
echo "📦 Building Soroban contracts..."
cd soroban && cargo build --target wasm32-unknown-unknown --release && cd ..

echo "✅ Development environment setup complete!"
echo ""
echo "🌐 Access points:"
echo "   Frontend: http://localhost:3000"
echo "   Backend API: http://localhost:8080"
echo "   Database: localhost:5432"
echo "   Redis: localhost:6379"
echo "   Soroban RPC: http://localhost:8000"
echo ""
echo "🚀 To start development:"
echo "   make start    # Start all services"
echo "   make logs     # View logs"
echo "   make stop     # Stop all services"
```

**scripts/dev-start.sh**:
```bash
#!/bin/bash
set -e

echo "🚀 Starting development environment..."

# Start infrastructure first
docker-compose up -d postgres redis soroban-rpc

# Wait for infrastructure to be ready
echo "⏳ Waiting for infrastructure..."
sleep 10

# Start application services
docker-compose up -d backend frontend

echo "✅ Development environment started!"
echo ""
echo "🌐 Services available at:"
echo "   Frontend: http://localhost:3000"
echo "   Backend: http://localhost:8080/api"
echo "   API Docs: http://localhost:8080/docs"
echo ""
echo "📊 To monitor:"
echo "   make logs              # All logs"
echo "   docker-compose logs -f backend   # Backend logs only"
echo "   docker-compose logs -f frontend  # Frontend logs only"
```

### VS Code Dev Container (Optional)

**.devcontainer/devcontainer.json**:
```json
{
  "name": "Bitcoin Custody Dev Environment",
  "dockerComposeFile": "../docker-compose.yml",
  "service": "backend",
  "workspaceFolder": "/app",
  "customizations": {
    "vscode": {
      "extensions": [
        "rust-lang.rust-analyzer",
        "bradlc.vscode-tailwindcss",
        "esbenp.prettier-vscode",
        "ms-vscode.vscode-typescript-next"
      ],
      "settings": {
        "rust-analyzer.cargo.target": "x86_64-unknown-linux-gnu",
        "rust-analyzer.checkOnSave.command": "clippy"
      }
    }
  },
  "forwardPorts": [3000, 8080, 5432, 6379, 8000],
  "postCreateCommand": "make setup"
}
```

### Environment Configuration

**frontend/.env.example**:
```env
# API Configuration
VITE_API_URL=http://localhost:8080
VITE_WS_URL=ws://localhost:8080

# Feature Flags
VITE_ENABLE_DEBUG=true
VITE_ENABLE_MOCK_DATA=false

# Soroban Configuration
VITE_SOROBAN_NETWORK=testnet
VITE_SOROBAN_RPC_URL=http://localhost:8000
```

**backend/.env.example**:
```env
# Database
DATABASE_URL=postgres://postgres:password@localhost:5432/bitcoin_custody_dev

# Redis
REDIS_URL=redis://localhost:6379

# JWT
JWT_SECRET=your-development-secret-key-change-in-production

# Soroban
SOROBAN_NETWORK=testnet
SOROBAN_RPC_URL=http://localhost:8000
SOROBAN_NETWORK_PASSPHRASE=Test SDF Network ; September 2015

# Logging
RUST_LOG=debug
RUST_BACKTRACE=1
```

## Rationale

### Docker Compose Benefits
- **Consistency**: Same environment across all developer machines
- **Isolation**: Services run in isolated containers
- **Reproducibility**: Easy to recreate exact environment
- **Scalability**: Easy to add new services as needed
- **Cross-platform**: Works on macOS, Linux, and Windows

### Service Architecture
- **PostgreSQL**: Reliable, feature-rich database for development
- **Redis**: Fast caching and session storage
- **Soroban RPC**: Local blockchain network for contract testing
- **Hot Reload**: Fast development cycles with automatic reloading

### Development Scripts
- **Automation**: Reduce manual setup steps
- **Consistency**: Standardized commands across team
- **Documentation**: Scripts serve as executable documentation
- **Efficiency**: Quick setup and teardown of environment

## Consequences

### Positive
- **Fast Onboarding**: New developers can start in minutes
- **Consistency**: Eliminates "works on my machine" issues
- **Isolation**: Development doesn't interfere with host system
- **Reproducibility**: Easy to recreate issues and test fixes
- **Scalability**: Easy to add new services or modify existing ones

### Negative
- **Resource Usage**: Docker containers use more resources than native
- **Complexity**: Additional layer of abstraction to understand
- **Disk Space**: Docker images and volumes consume disk space
- **Network Overhead**: Container networking adds slight latency

### Neutral
- **Learning Curve**: Team needs to understand Docker basics
- **Debugging**: Slightly different debugging workflow in containers

## Alternatives Considered

### Alternative 1: Native Development
- **Pros**: Better performance, simpler debugging
- **Cons**: Complex setup, inconsistent environments, dependency conflicts
- **Rejected**: Too much setup complexity and inconsistency

### Alternative 2: Vagrant
- **Pros**: Full VM isolation, consistent environments
- **Cons**: Heavy resource usage, slower than containers
- **Rejected**: Docker provides better performance and developer experience

### Alternative 3: Cloud Development Environments
- **Pros**: No local setup required, powerful remote machines
- **Cons**: Requires internet connection, potential latency issues
- **Rejected**: Team prefers local development for responsiveness

## Health Checks and Monitoring

### Service Health Checks
```yaml
# Example health check configuration
healthcheck:
  test: ["CMD", "curl", "-f", "http://localhost:8080/health"]
  interval: 30s
  timeout: 10s
  retries: 3
  start_period: 40s
```

### Monitoring Script
```bash
#!/bin/bash
# scripts/health-check.sh

echo "🔍 Checking service health..."

services=("postgres" "redis" "soroban-rpc" "backend" "frontend")

for service in "${services[@]}"; do
    if docker-compose ps $service | grep -q "Up (healthy)"; then
        echo "✅ $service: healthy"
    elif docker-compose ps $service | grep -q "Up"; then
        echo "⚠️  $service: running (no health check)"
    else
        echo "❌ $service: not running"
    fi
done
```

## Troubleshooting Guide

### Common Issues

1. **Port Conflicts**
   ```bash
   # Check what's using a port
   lsof -i :3000
   
   # Kill process using port
   kill -9 $(lsof -t -i:3000)
   ```

2. **Database Connection Issues**
   ```bash
   # Check PostgreSQL logs
   docker-compose logs postgres
   
   # Connect to database directly
   docker-compose exec postgres psql -U postgres -d bitcoin_custody_dev
   ```

3. **Container Build Issues**
   ```bash
   # Clean rebuild
   docker-compose down -v
   docker-compose build --no-cache
   docker-compose up -d
   ```

## Related Decisions

- ADR-001: Project Structure Reorganization
- ADR-002: Frontend Technology Stack
- ADR-003: Backend Framework Selection

## Success Metrics

- **Setup Time**: New developers can start development in < 10 minutes
- **Consistency**: Zero "works on my machine" issues
- **Reliability**: 99%+ uptime for development services
- **Performance**: Development workflow performance comparable to native setup