# Project Restructure Design Document

## Overview

This design document outlines the restructuring of the current project into a clean, organized monorepo with separate directories for frontend (`/frontend`), backend (`/backend`), and smart contracts (`/soroban`). The restructure transforms the current mixed architecture into a professional full-stack setup that enables independent development while maintaining seamless integration.

The design establishes clear separation of concerns, proper dependency management, and unified development workflows while preserving all existing functionality from the integration-features implementation.

## Current vs. Target Structure

### Previous Structure (Migrated)

```
Previous Project Root/
├── uxui/                    # React frontend (MIGRATED to /frontend)
├── contracts/               # Soroban contracts (MIGRATED to /soroban)
├── .kiro/specs/integration-features/  # Smart contract implementation
├── Cargo.toml              # Rust workspace config
└── various config files
```

### Target Structure

```
Project Root/
├── frontend/               # React + TypeScript frontend
│   ├── src/
│   ├── public/
│   ├── package.json
│   ├── vite.config.ts
│   └── README.md
├── backend/                # Loco.rs + PostgreSQL backend  
│   ├── src/
│   ├── migration/
│   ├── config/
│   ├── Cargo.toml
│   └── README.md
├── soroban/               # Soroban smart contracts
│   ├── contracts/
│   ├── integration-router/
│   ├── Cargo.toml
│   └── README.md
├── docker-compose.yml     # Full-stack development environment
├── README.md             # Main project documentation
└── scripts/              # Build and deployment scripts
```

## Architecture Design

### Component Separation Strategy

```mermaid
graph TB
    subgraph "Frontend Layer"
        REACT[React App]
        COMP[UI Components]
        STORE[State Management]
    end
    
    subgraph "Backend Layer"
        API[Loco.rs API Server]
        AUTH[Authentication]
        BIZ[Business Logic]
        DB[(PostgreSQL)]
    end
    
    subgraph "Blockchain Layer"
        ROUTER[Integration Router]
        KYC[KYC Registry]
        TOKEN[iSTSi Token]
        RESERVE[Reserve Manager]
    end
    
    REACT --> API
    API --> DB
    API --> ROUTER
    ROUTER --> KYC
    ROUTER --> TOKEN
    ROUTER --> RESERVE
```

### Communication Patterns

**Frontend ↔ Backend:**
- HTTP REST APIs for standard operations
- WebSocket connections for real-time updates
- JWT authentication for secure communication

**Backend ↔ Soroban:**
- Soroban SDK for contract interactions
- Event monitoring for blockchain updates
- Transaction signing and submission

## Detailed Component Design

### 1. Frontend Directory (`/frontend`)

**Structure:**
```
frontend/
├── src/
│   ├── components/         # React components (migrated from uxui/src/components)
│   │   ├── ui/            # Radix UI components
│   │   ├── SystemOverview.tsx
│   │   ├── IntegrationRouter.tsx
│   │   └── ...
│   ├── services/          # API client and business logic
│   │   ├── api.ts         # HTTP client for backend APIs
│   │   ├── websocket.ts   # WebSocket client for real-time updates
│   │   └── auth.ts        # Authentication service
│   ├── hooks/             # Custom React hooks
│   ├── store/             # State management (Redux Toolkit)
│   ├── types/             # TypeScript type definitions
│   ├── utils/             # Utility functions
│   └── App.tsx
├── public/                # Static assets
├── package.json           # Frontend dependencies
├── vite.config.ts         # Vite configuration
├── tailwind.config.js     # Tailwind CSS configuration
├── tsconfig.json          # TypeScript configuration
├── .env.example           # Environment variables template
└── README.md              # Frontend setup and development guide
```

**Key Configuration Files:**

**package.json:**
```json
{
  "name": "bitcoin-custody-frontend",
  "version": "1.0.0",
  "type": "module",
  "scripts": {
    "dev": "vite",
    "build": "vite build",
    "preview": "vite preview",
    "lint": "eslint src --ext ts,tsx",
    "type-check": "tsc --noEmit"
  },
  "dependencies": {
    "react": "^18.3.1",
    "react-dom": "^18.3.1",
    "@reduxjs/toolkit": "^2.0.1",
    "react-redux": "^9.0.4",
    "axios": "^1.6.2",
    "socket.io-client": "^4.7.4",
    "@radix-ui/react-*": "latest"
  },
  "devDependencies": {
    "@vitejs/plugin-react-swc": "^3.10.2",
    "vite": "^6.3.5",
    "typescript": "^5.3.3"
  }
}
```

**vite.config.ts:**
```typescript
import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react-swc';
import path from 'path';

export default defineConfig({
  plugins: [react()],
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src'),
    },
  },
  server: {
    port: 3000,
    proxy: {
      '/api': {
        target: 'http://localhost:8080',
        changeOrigin: true,
      },
      '/ws': {
        target: 'ws://localhost:8080',
        ws: true,
      },
    },
  },
  build: {
    outDir: 'dist',
    sourcemap: true,
  },
});
```

### 2. Backend Directory (`/backend`)

**Structure:**
```
backend/
├── src/
│   ├── controllers/       # API endpoint handlers
│   │   ├── auth.rs       # Authentication endpoints
│   │   ├── users.rs      # User management
│   │   ├── integration.rs # Integration operations
│   │   ├── kyc.rs        # KYC and compliance
│   │   ├── tokens.rs     # Token operations
│   │   ├── reserves.rs   # Reserve management
│   │   └── mod.rs
│   ├── models/           # Database models
│   │   ├── user.rs
│   │   ├── kyc_record.rs
│   │   ├── token_balance.rs
│   │   ├── operation.rs
│   │   └── mod.rs
│   ├── services/         # Business logic services
│   │   ├── integration_service.rs
│   │   ├── kyc_service.rs
│   │   ├── token_service.rs
│   │   ├── reserve_service.rs
│   │   ├── soroban_client.rs
│   │   └── mod.rs
│   ├── workers/          # Background jobs
│   │   ├── reconciliation.rs
│   │   ├── proof_generation.rs
│   │   └── mod.rs
│   ├── middleware/       # Custom middleware
│   │   ├── auth.rs
│   │   ├── cors.rs
│   │   └── mod.rs
│   ├── app.rs           # Application setup
│   ├── lib.rs           # Library exports
│   └── main.rs          # Application entry point
├── migration/           # Database migrations
│   ├── m20240101_000001_create_users.rs
│   ├── m20240101_000002_create_kyc_records.rs
│   └── mod.rs
├── config/             # Configuration files
│   ├── development.yaml
│   ├── production.yaml
│   └── test.yaml
├── tests/              # Integration tests
├── Cargo.toml          # Rust dependencies
├── .env.example        # Environment variables template
└── README.md           # Backend setup and API documentation
```

**Key Configuration Files:**

**Cargo.toml:**
```toml
[package]
name = "bitcoin-custody-backend"
version = "0.1.0"
edition = "2021"

[dependencies]
loco-rs = { version = "0.2", features = ["with-db", "auth"] }
sea-orm = { version = "0.12", features = ["sqlx-postgres", "runtime-tokio-rustls", "macros"] }
tokio = { version = "1.0", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
uuid = { version = "1.0", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
jsonwebtoken = "9.0"
bcrypt = "0.15"
soroban-sdk = "22.0.8"
stellar-strkey = "0.0.8"
reqwest = { version = "0.11", features = ["json"] }
tokio-tungstenite = "0.20"

[dev-dependencies]
tower-test = "0.4"
```

**config/development.yaml:**
```yaml
server:
  port: 8080
  host: 0.0.0.0

database:
  uri: postgres://postgres:password@localhost:5432/bitcoin_custody_dev
  auto_migrate: true
  connect_timeout: 500
  idle_timeout: 500

auth:
  jwt:
    secret: "development-secret-key-change-in-production"
    expiration: 86400

soroban:
  network: "testnet"
  rpc_url: "https://soroban-testnet.stellar.org"
  network_passphrase: "Test SDF Network ; September 2015"

cors:
  allow_origins: ["http://localhost:3000"]
  allow_methods: ["GET", "POST", "PUT", "DELETE", "OPTIONS"]
  allow_headers: ["Content-Type", "Authorization"]

logging:
  level: "debug"
  format: "json"
```

### 3. Soroban Directory (`/soroban`)

**Structure:**
```
soroban/
├── contracts/
│   ├── integration-router/    # Main integration contract
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── storage.rs
│   │   │   ├── events.rs
│   │   │   └── operations.rs
│   │   └── Cargo.toml
│   ├── kyc-registry/         # KYC compliance contract
│   ├── istsi-token/          # Bitcoin-backed token contract
│   ├── reserve-manager/      # Reserve management contract
│   └── fungible-token/       # Additional token contract
├── shared/                   # Shared contract utilities
│   ├── src/
│   │   ├── lib.rs
│   │   ├── types.rs
│   │   └── utils.rs
│   └── Cargo.toml
├── tests/                    # Integration tests
│   ├── integration_test.rs
│   ├── bitcoin_deposit_test.rs
│   └── cross_token_test.rs
├── scripts/                  # Deployment and utility scripts
│   ├── deploy.sh
│   ├── test.sh
│   └── build.sh
├── Cargo.toml               # Workspace configuration
├── .env.example             # Environment variables for deployment
└── README.md                # Contract documentation and deployment guide
```

**Key Configuration Files:**

**Cargo.toml (Workspace):**
```toml
[workspace]
resolver = "2"
members = [
    "contracts/integration-router",
    "contracts/kyc-registry", 
    "contracts/istsi-token",
    "contracts/reserve-manager",
    "contracts/fungible-token",
    "shared"
]

[workspace.package]
version = "0.1.0"
edition = "2021"
license = "Apache-2.0"

[workspace.dependencies]
soroban-sdk = "22.0.8"
stellar-tokens = "0.4.1"
stellar-access = "0.4.1"
stellar-contract-utils = "0.4.1"
stellar-macros = "0.4.1"
```

## Integration and Communication Design

### Development Environment Integration

**docker-compose.yml (Root Level):**
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

  backend:
    build: ./backend
    ports:
      - "8080:8080"
    environment:
      DATABASE_URL: postgres://postgres:password@postgres:5432/bitcoin_custody_dev
    depends_on:
      - postgres
    volumes:
      - ./backend:/app
    command: cargo loco start

  frontend:
    build: ./frontend
    ports:
      - "3000:3000"
    volumes:
      - ./frontend:/app
      - /app/node_modules
    command: npm run dev

  soroban-rpc:
    image: stellar/quickstart:soroban-dev
    ports:
      - "8000:8000"
    command: --local --enable-soroban-rpc

volumes:
  postgres_data:
```

### Build and Development Scripts

**scripts/setup.sh:**
```bash
#!/bin/bash
set -e

echo "Setting up Bitcoin Custody full-stack development environment..."

# Install frontend dependencies
echo "Installing frontend dependencies..."
cd frontend && npm install && cd ..

# Install backend dependencies and run migrations
echo "Setting up backend..."
cd backend && cargo build && cd ..

# Build Soroban contracts
echo "Building Soroban contracts..."
cd soroban && cargo build --target wasm32-unknown-unknown --release && cd ..

# Start development environment
echo "Starting development environment..."
docker-compose up -d postgres soroban-rpc

# Wait for services to be ready
sleep 10

# Run database migrations
cd backend && cargo loco db migrate && cd ..

echo "Setup complete! Run 'npm run dev' to start development."
```

**scripts/dev.sh:**
```bash
#!/bin/bash
# Start all services for development

# Start infrastructure
docker-compose up -d postgres soroban-rpc

# Start backend in background
cd backend && cargo loco start &
BACKEND_PID=$!

# Start frontend in background  
cd frontend && npm run dev &
FRONTEND_PID=$!

echo "Development environment started!"
echo "Frontend: http://localhost:3000"
echo "Backend API: http://localhost:8080"
echo "Press Ctrl+C to stop all services"

# Wait for interrupt
trap "kill $BACKEND_PID $FRONTEND_PID; docker-compose down" INT
wait
```

### Cross-Component Communication

**API Client (Frontend):**
```typescript
// frontend/src/services/api.ts
import axios from 'axios';

const API_BASE_URL = import.meta.env.VITE_API_URL || 'http://localhost:8080';

export const apiClient = axios.create({
  baseURL: `${API_BASE_URL}/api`,
  timeout: 10000,
});

// Add auth token to requests
apiClient.interceptors.request.use((config) => {
  const token = localStorage.getItem('auth_token');
  if (token) {
    config.headers.Authorization = `Bearer ${token}`;
  }
  return config;
});

// Integration operations
export const integrationApi = {
  executeBitcoinDeposit: (params: BitcoinDepositRequest) =>
    apiClient.post('/integration/bitcoin-deposit', params),
  
  executeTokenWithdrawal: (params: TokenWithdrawalRequest) =>
    apiClient.post('/integration/token-withdrawal', params),
    
  getSystemOverview: () =>
    apiClient.get('/system/overview'),
};
```

**Soroban Client (Backend):**
```rust
// backend/src/services/soroban_client.rs
use soroban_sdk::{Env, Address};

pub struct SorobanClient {
    env: Env,
    integration_router_address: Address,
}

impl SorobanClient {
    pub async fn execute_bitcoin_deposit(
        &self,
        user: Address,
        btc_amount: u64,
        btc_tx_hash: String,
    ) -> Result<String, SorobanError> {
        // Call integration router contract
        let result = self.env.invoke_contract(
            &self.integration_router_address,
            &symbol_short!("exec_btc"),
            (user, btc_amount, btc_tx_hash).into_val(&self.env),
        );
        
        Ok(result.to_string())
    }
}
```

## Migration Strategy

### Phase 1: Directory Creation and File Movement

1. **✅ Created new directory structure**
2. **✅ Moved `/uxui` → `/frontend`** with configuration updates
3. **✅ Moved `/contracts` → `/soroban/contracts`** 
4. **✅ Created `/backend` with Loco.rs initialization**
5. **✅ Updated all import paths and references**

### Phase 2: Configuration and Integration

1. **Set up cross-component communication**
2. **Configure development environment with Docker Compose**
3. **Create unified build and deployment scripts**
4. **Update documentation and README files**

### Phase 3: Testing and Validation

1. **Verify all components build independently**
2. **Test full-stack integration in development**
3. **Validate existing functionality is preserved**
4. **Update CI/CD pipelines for new structure**

## Documentation Strategy

### Component-Specific Documentation

**Frontend README.md:**
- React development setup and commands
- Component architecture and state management
- API integration and WebSocket usage
- Build and deployment instructions

**Backend README.md:**
- Loco.rs setup and configuration
- API endpoint documentation
- Database schema and migrations
- Soroban integration guide

**Soroban README.md:**
- Contract architecture and deployment
- Integration router functionality
- Testing and development workflow
- Network configuration and deployment

### Root-Level Documentation

**Main README.md:**
- Project overview and architecture
- Quick start guide for full development environment
- Component interaction and communication patterns
- Deployment and production setup

This design provides a comprehensive restructuring plan that creates clear separation while maintaining seamless integration between all components.