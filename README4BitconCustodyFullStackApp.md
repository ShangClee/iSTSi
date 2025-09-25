# Bitcoin Custody Full-Stack Application

A comprehensive Bitcoin custody solution built with React frontend, Loco.rs backend, and Soroban smart contracts. This project provides a complete infrastructure for Bitcoin-backed token operations with KYC compliance, reserve management, and integration capabilities.

> **✅ Project Restructure Complete:** This project has been successfully migrated from a mixed directory structure to a clean monorepo architecture. See [MIGRATION_GUIDE.md](./MIGRATION_GUIDE.md) for migration details and [LESSONS_LEARNED.md](./LESSONS_LEARNED.md) for insights from the restructure process.

## Architecture Overview

```
┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│   Frontend  │    │   Backend   │    │   Soroban   │
│   (React)   │◄──►│  (Loco.rs)  │◄──►│ Contracts   │
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

## Quick Start

### Prerequisites

- Docker and Docker Compose
- Node.js 20+ (for local development)
- Rust 1.75+ (for local development)

### Development Environment

1. **Setup the environment:**
   ```bash
   make setup
   # or
   ./scripts/dev-setup.sh
   ```

2. **Start all services:**
   ```bash
   make start
   # or
   ./scripts/dev-start.sh
   ```

3. **Access the application:**
   - Frontend: http://localhost:3000
   - Backend API: http://localhost:8080
   - Soroban RPC: http://localhost:8000

### Available Commands

```bash
# Development
make help          # Show all available commands
make start         # Start development environment
make stop          # Stop development environment
make status        # Check service status
make logs          # View all logs
make health        # Run health check

# Testing
make test          # Run all tests
make test-backend  # Run backend tests only
make test-frontend # Run frontend tests only

# Database
make db-migrate    # Run database migrations
make db-reset      # Reset database
make db-shell      # Access database shell

# Utilities
make clean         # Clean up Docker resources
make reset         # Complete environment reset
```

## Project Structure

This project follows a clean monorepo architecture with clear separation of concerns:

```
├── frontend/              # React + TypeScript frontend
│   ├── src/
│   │   ├── components/    # UI components
│   │   ├── services/      # API clients and business logic
│   │   ├── hooks/         # Custom React hooks
│   │   ├── store/         # State management (Redux)
│   │   └── types/         # TypeScript definitions
│   ├── public/
│   └── package.json
├── backend/               # Loco.rs + PostgreSQL backend
│   ├── src/
│   │   ├── controllers/   # API endpoint handlers
│   │   ├── models/        # Database models
│   │   ├── services/      # Business logic services
│   │   ├── middleware/    # Custom middleware
│   │   └── workers/       # Background jobs
│   ├── migration/         # Database migrations
│   └── Cargo.toml
├── soroban/              # Soroban smart contracts
│   ├── contracts/
│   │   ├── integration_router/  # Main integration contract
│   │   ├── kyc_registry/       # KYC compliance
│   │   ├── istsi_token/        # Bitcoin-backed token
│   │   └── reserve_manager/    # Reserve management
│   ├── shared/           # Shared contract utilities
│   └── Cargo.toml
├── scripts/              # Development and deployment scripts
├── docs/                 # Project documentation
├── docker-compose.yml    # Development environment
├── Makefile             # Development commands
├── MIGRATION_GUIDE.md   # Migration documentation
└── LESSONS_LEARNED.md   # Project insights
```

### Architecture Benefits

- **🎯 Clear Separation of Concerns** - Each directory has focused responsibilities
- **🚀 Independent Development** - Teams can work on components without interference  
- **📦 Independent Deployment** - Components can be deployed and scaled separately
- **🔧 Better Tooling** - Component-specific configurations and optimizations
- **📚 Easier Onboarding** - New developers can understand the architecture quickly
- **🔄 Industry Standards** - Follows common patterns for full-stack applications

## Components

### Frontend (`/frontend`)
- **Technology:** React 18, TypeScript, Vite, Tailwind CSS
- **Features:** Real-time updates, responsive design, comprehensive UI components
- **State Management:** Redux Toolkit with RTK Query

### Backend (`/backend`)
- **Technology:** Rust, Loco.rs framework, Sea-ORM, PostgreSQL
- **Features:** RESTful API, WebSocket support, JWT authentication, database migrations
- **Architecture:** Controllers, services, models, workers pattern

### Smart Contracts (`/soroban`)
- **Technology:** Soroban (Stellar smart contracts), Rust
- **Contracts:** Integration router, KYC registry, iSTSi token, reserve manager
- **Features:** Bitcoin deposit/withdrawal, compliance checks, reserve management

## Development Workflow

### Making Changes

1. **Frontend Development:**
   ```bash
   # Changes in ./frontend/src are hot-reloaded automatically
   make logs-frontend  # Monitor frontend logs
   ```

2. **Backend Development:**
   ```bash
   # Changes in ./backend/src trigger automatic rebuild
   make logs-backend   # Monitor backend logs
   make db-migrate     # Run new migrations
   ```

3. **Smart Contract Development:**
   ```bash
   cd soroban
   cargo build --target wasm32-unknown-unknown --release
   make test-contracts
   ```

### Testing

```bash
# Run all tests
make test

# Run specific component tests
make test-backend
make test-frontend
make test-contracts

# Run integration tests
docker-compose exec backend cargo test --test integration
```

### Debugging

```bash
# Check service health
make health

# View service logs
make logs                    # All services
make logs-backend           # Backend only
make logs-frontend          # Frontend only

# Access service shells
make shell-backend          # Backend container
make shell-frontend         # Frontend container
make db-shell              # Database shell
```

## API Documentation

### Authentication
- **POST** `/api/auth/login` - User login
- **POST** `/api/auth/register` - User registration
- **POST** `/api/auth/logout` - User logout

### Integration Operations
- **POST** `/api/integration/bitcoin-deposit` - Execute Bitcoin deposit
- **POST** `/api/integration/token-withdrawal` - Execute token withdrawal
- **GET** `/api/integration/status` - Get integration status

### System Management
- **GET** `/api/system/overview` - System overview
- **GET** `/api/system/health` - Health check
- **GET** `/api/reserves/status` - Reserve status

## Environment Configuration

### Development
- All services run in Docker containers
- Hot reloading enabled for frontend and backend
- Debug logging enabled
- Local Soroban network for contract testing

### Production
- See individual component README files for production deployment
- Use secure environment variables
- Enable SSL/TLS
- Configure proper monitoring and logging

## Monitoring

### Health Checks
```bash
# Automated health monitoring
make health

# Continuous monitoring
./scripts/monitor.sh 30  # Check every 30 seconds
```

### Service URLs
- **Frontend:** http://localhost:3000
- **Backend API:** http://localhost:8080/api
- **Soroban RPC:** http://localhost:8000
- **Database:** localhost:5432
- **Redis:** localhost:6379

## Troubleshooting

### Common Issues

**Services won't start:**
```bash
make clean    # Clean up Docker resources
make reset    # Complete reset
```

**Port conflicts:**
```bash
# Check what's using ports
lsof -i :3000
lsof -i :8080

# Stop conflicting services
make stop
```

**Database issues:**
```bash
make db-reset    # Reset database
make db-migrate  # Run migrations
```

### Getting Help

1. Check service status: `make status`
2. View logs: `make logs`
3. Run health check: `make health`
4. See detailed guide: [DEVELOPMENT.md](./DEVELOPMENT.md)

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Run tests: `make test`
5. Check health: `make health`
6. Submit a pull request

## License

This project is licensed under the Apache License 2.0.

## Dependencies

- [Rust and Stellar installation guide](https://developers.stellar.org/docs/build/smart-contracts/getting-started/setup)
- [Docker installation guide](https://docs.docker.com/get-docker/)
- [Node.js installation guide](https://nodejs.org/)
