# Developer Onboarding Guide

Welcome to the Bitcoin Custody Full-Stack Application! This guide will help you get up and running with the development environment in under 30 minutes.

## Prerequisites

Before you begin, ensure you have the following installed:

- **Node.js** (v18 or higher) - [Download here](https://nodejs.org/)
- **Rust** (latest stable) - [Install via rustup](https://rustup.rs/)
- **Docker & Docker Compose** - [Install Docker Desktop](https://www.docker.com/products/docker-desktop/)
- **PostgreSQL** (optional, can use Docker) - [Download here](https://www.postgresql.org/download/)
- **Git** - [Download here](https://git-scm.com/downloads)

### Soroban-Specific Prerequisites

- **Soroban CLI** - Install with: `cargo install --locked soroban-cli`
- **Stellar CLI** - Install with: `cargo install --locked stellar-cli`

## Quick Start (5 Minutes)

1. **Clone the repository:**
   ```bash
   git clone <repository-url>
   cd bitcoin-custody-app
   ```

2. **Run the setup script:**
   ```bash
   chmod +x scripts/setup.sh
   ./scripts/setup.sh
   ```

3. **Start the development environment:**
   ```bash
   ./scripts/dev.sh
   ```

4. **Access the application:**
   - Frontend: http://localhost:3000
   - Backend API: http://localhost:8080
   - API Documentation: http://localhost:8080/docs

## Detailed Setup Instructions

### Step 1: Environment Configuration

1. **Copy environment files:**
   ```bash
   cp frontend/.env.example frontend/.env.development
   cp backend/.env.example backend/.env
   cp soroban/.env.example soroban/.env
   ```

2. **Configure environment variables:**
   - Edit `frontend/.env.development` for frontend settings
   - Edit `backend/.env` for backend database and API settings
   - Edit `soroban/.env` for Soroban network configuration

### Step 2: Database Setup

1. **Start PostgreSQL (using Docker):**
   ```bash
   docker-compose up -d postgres
   ```

2. **Run database migrations:**
   ```bash
   cd backend
   cargo loco db migrate
   cd ..
   ```

3. **Seed development data (optional):**
   ```bash
   cd backend
   cargo loco db seed
   cd ..
   ```

### Step 3: Frontend Setup

1. **Install dependencies:**
   ```bash
   cd frontend
   npm install
   ```

2. **Start development server:**
   ```bash
   npm run dev
   ```

### Step 4: Backend Setup

1. **Build the backend:**
   ```bash
   cd backend
   cargo build
   ```

2. **Start the backend server:**
   ```bash
   cargo loco start
   ```

### Step 5: Soroban Contracts Setup

1. **Build contracts:**
   ```bash
   cd soroban
   cargo build --target wasm32-unknown-unknown --release
   ```

2. **Deploy to local network (optional):**
   ```bash
   ./scripts/deploy-local.sh
   ```

## Development Workflow Overview

### Project Structure

```
bitcoin-custody-app/
├── frontend/          # React + TypeScript frontend
├── backend/           # Loco.rs + PostgreSQL backend
├── soroban/          # Soroban smart contracts
├── scripts/          # Build and deployment scripts
├── docs/             # Additional documentation
└── docker-compose.yml # Development environment
```

### Component Responsibilities

- **Frontend**: User interface, state management, API communication
- **Backend**: REST APIs, business logic, database operations, Soroban integration
- **Soroban**: Smart contracts for KYC, tokens, reserves, and integration routing

## Common Development Tasks

### Making Changes to Frontend

1. Navigate to frontend directory: `cd frontend`
2. Make your changes to React components
3. Hot reload will automatically update the browser
4. Run tests: `npm test`
5. Build for production: `npm run build`

### Making Changes to Backend

1. Navigate to backend directory: `cd backend`
2. Make changes to Rust code
3. Restart the server: `cargo loco start`
4. Run tests: `cargo test`
5. Check API documentation at http://localhost:8080/docs

### Making Changes to Soroban Contracts

1. Navigate to soroban directory: `cd soroban`
2. Make changes to contract code
3. Rebuild contracts: `cargo build --target wasm32-unknown-unknown --release`
4. Run contract tests: `cargo test`
5. Deploy to testnet: `./scripts/deploy-testnet.sh`

### Cross-Component Changes

When making changes that affect multiple components:

1. **Plan the change** - Document what needs to change in each component
2. **Update contracts first** - If blockchain logic changes
3. **Update backend APIs** - Modify endpoints and business logic
4. **Update frontend** - Modify UI and API calls
5. **Test integration** - Run full-stack tests
6. **Update documentation** - Keep docs in sync

## Testing Your Setup

### Verify Frontend

```bash
cd frontend
npm run type-check  # TypeScript compilation
npm run lint        # ESLint checks
npm test           # Unit tests
```

### Verify Backend

```bash
cd backend
cargo check        # Rust compilation
cargo test         # Unit and integration tests
cargo loco doctor  # Health check
```

### Verify Soroban

```bash
cd soroban
cargo test         # Contract tests
soroban contract invoke --help  # CLI availability
```

### Full Integration Test

```bash
# Start all services
./scripts/dev.sh

# In another terminal, run integration tests
./scripts/test-integration.sh
```

## Troubleshooting

### Common Issues

1. **Port conflicts**: Ensure ports 3000, 8080, and 5432 are available
2. **Database connection**: Check PostgreSQL is running and credentials are correct
3. **Rust compilation**: Ensure you have the latest stable Rust version
4. **Node modules**: Try deleting `node_modules` and running `npm install` again
5. **Docker issues**: Restart Docker Desktop and try again

### Getting Help

1. **Check logs**: Each component has detailed logging
2. **Read error messages**: They usually contain helpful information
3. **Check documentation**: Component-specific READMEs have more details
4. **Ask the team**: Use our communication channels for help

### Useful Commands

```bash
# View all running services
docker-compose ps

# View logs for specific service
docker-compose logs backend

# Restart a specific service
docker-compose restart postgres

# Clean rebuild everything
./scripts/clean-build.sh
```

## Next Steps

After completing the setup:

1. **Read the Architecture Guide** - Understand how components interact
2. **Review the API Documentation** - Familiarize yourself with endpoints
3. **Explore the Codebase** - Start with the main entry points
4. **Run the Test Suite** - Ensure everything works correctly
5. **Make a Small Change** - Try modifying a component to test your setup

## Development Environment Tips

- Use VS Code with recommended extensions (see `.vscode/extensions.json`)
- Enable format-on-save for consistent code style
- Use the integrated terminal for running commands
- Install the Rust Analyzer extension for better Rust development
- Use the Thunder Client extension for API testing

Welcome to the team! 🚀