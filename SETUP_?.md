# Bitcoin Custody System - Development Environment Setup

This guide will help you set up the complete development environment for the Bitcoin Custody System, including all three main components: Frontend (React), Backend (Loco.rs), and Soroban Smart Contracts.

## System Requirements

- **Operating System**: macOS, Linux, or Windows (with WSL2)
- **Git**: Version control system
- **Docker**: For running PostgreSQL and other services
- **Node.js**: 18+ for frontend development
- **Rust**: 1.70+ for backend and smart contracts

## Quick Start

Run the automated setup script to install all dependencies and configure the development environment:

```bash
# Clone the repository
git clone <repository-url>
cd bitcoin-custody-system

# Run the automated setup script
./scripts/setup-dev.sh
```

This script will:
- Check and install required dependencies
- Set up all three components (frontend, backend, soroban)
- Start necessary infrastructure services (PostgreSQL, Soroban RPC)
- Provide next steps for development

## Manual Setup

If you prefer to set up components individually, follow these steps:

### 1. Install Core Dependencies

#### Node.js (for Frontend)
```bash
# Using Node Version Manager (recommended)
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.0/install.sh | bash
nvm install 18
nvm use 18

# Or install directly from nodejs.org
# Download from: https://nodejs.org/
```

#### Rust (for Backend and Soroban)
```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Add WASM target for Soroban contracts
rustup target add wasm32-unknown-unknown
```

#### Docker (for Infrastructure)
```bash
# macOS (using Homebrew)
brew install docker docker-compose

# Linux (Ubuntu/Debian)
sudo apt-get update
sudo apt-get install docker.io docker-compose

# Start Docker service
sudo systemctl start docker
sudo systemctl enable docker
```

### 2. Component-Specific Setup

#### Frontend (React + TypeScript)
```bash
cd frontend

# Install dependencies
npm install

# Copy environment variables
cp .env.example .env

# Start development server
npm run dev
```

The frontend will be available at http://localhost:3000

#### Backend (Loco.rs)
```bash
cd backend

# Install Loco CLI (when ready)
cargo install loco-cli

# Set up database
docker-compose up -d postgres

# Copy environment variables
cp .env.example .env

# Run database migrations
cargo loco db migrate

# Start development server
cargo loco start
```

The backend API will be available at http://localhost:8080

#### Soroban Smart Contracts
```bash
cd soroban

# Install Soroban CLI
cargo install --locked soroban-cli

# Build all contracts
cargo build --target wasm32-unknown-unknown --release

# Run tests
cargo test

# Start local Soroban network (optional)
docker-compose up -d soroban-rpc
```

For detailed Soroban setup instructions, see [soroban/SETUP.md](soroban/SETUP.md)

### 3. Infrastructure Services

Start the required infrastructure services:

```bash
# Start all services
docker-compose up -d

# Or start individual services
docker-compose up -d postgres    # PostgreSQL database
docker-compose up -d soroban-rpc # Soroban RPC server
```

## Environment Configuration

### Environment Variables

Each component requires specific environment variables:

#### Frontend (.env)
```bash
VITE_API_URL=http://localhost:8080
VITE_WS_URL=ws://localhost:8080/ws
```

#### Backend (.env)
```bash
DATABASE_URL=postgresql://postgres:password@localhost:5432/bitcoin_custody_dev
JWT_SECRET=your-jwt-secret-here
SOROBAN_RPC_URL=http://localhost:8000
```

#### Soroban (.env)
```bash
STELLAR_NETWORK=testnet
STELLAR_RPC_URL=https://soroban-testnet.stellar.org
STELLAR_SECRET_KEY=your-testnet-secret-key
```

### Network Configuration

The system supports multiple deployment environments:

- **Local Development**: All services running locally
- **Testnet**: Frontend/Backend local, Soroban on Stellar testnet
- **Production**: All components deployed to production infrastructure

## Development Workflow

### 1. Start Development Environment

```bash
# Start infrastructure
docker-compose up -d

# Terminal 1: Frontend
cd frontend && npm run dev

# Terminal 2: Backend (when implemented)
cd backend && cargo loco start

# Terminal 3: Soroban (for contract development)
cd soroban && cargo test --watch
```

### 2. Making Changes

- **Frontend**: Hot reloading enabled, changes reflect immediately
- **Backend**: Auto-restart on file changes (when implemented)
- **Soroban**: Rebuild and redeploy contracts as needed

### 3. Testing

```bash
# Frontend tests
cd frontend && npm test

# Backend tests (when implemented)
cd backend && cargo test

# Soroban contract tests
cd soroban && cargo test

# Integration tests (when implemented)
./scripts/run-integration-tests.sh
```

## Troubleshooting

### Common Issues

1. **Port Conflicts**
   - Frontend (3000), Backend (8080), PostgreSQL (5432), Soroban RPC (8000)
   - Stop conflicting services or change ports in configuration

2. **Database Connection Issues**
   ```bash
   # Reset database
   docker-compose down postgres
   docker-compose up -d postgres
   # Wait 30 seconds, then retry
   ```

3. **Rust/Cargo Issues**
   ```bash
   # Clean and rebuild
   cargo clean
   cargo build
   ```

4. **Node.js/npm Issues**
   ```bash
   # Clear cache and reinstall
   rm -rf node_modules package-lock.json
   npm install
   ```

### Getting Help

- Check component-specific README files for detailed documentation
- Review logs: `docker-compose logs <service-name>`
- Ensure all prerequisites are installed and up to date

## Next Steps

After setup is complete:

1. **Explore the codebase**: Each component has its own README with detailed information
2. **Run the test suites**: Ensure everything is working correctly
3. **Start development**: Pick a component and begin implementing features
4. **Review the specs**: Check `.kiro/specs/` for feature specifications and tasks

## Component Documentation

- [Frontend Documentation](frontend/README.md) - React + TypeScript frontend
- [Backend Documentation](backend/README.md) - Loco.rs backend API
- [Soroban Documentation](soroban/README.md) - Smart contracts and blockchain integration
- [Soroban Setup Guide](soroban/SETUP.md) - Detailed Soroban development setup

## Architecture Overview

The Bitcoin Custody System is organized as a monorepo with three main components:

```
bitcoin-custody-system/
├── frontend/          # React + TypeScript UI
├── backend/           # Loco.rs API server
├── soroban/          # Smart contracts
├── scripts/          # Development and deployment scripts
└── docs/             # Project documentation
```

Each component is designed to be independently developable while maintaining seamless integration for the complete system.