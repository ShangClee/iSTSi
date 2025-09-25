#!/bin/bash
set -e

# Bitcoin Custody System - Development Environment Setup
# This script sets up the full development environment for all components

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

echo "🚀 Setting up Bitcoin Custody System development environment..."
echo "Project root: $PROJECT_ROOT"
echo ""

# Check prerequisites
echo "🔍 Checking prerequisites..."

# Check Node.js
if command -v node &> /dev/null; then
    NODE_VERSION=$(node --version)
    echo "✅ Node.js found: $NODE_VERSION"
else
    echo "❌ Node.js not found. Please install Node.js 18+ and npm"
    exit 1
fi

# Check Rust
if command -v cargo &> /dev/null; then
    RUST_VERSION=$(rustc --version)
    echo "✅ Rust found: $RUST_VERSION"
else
    echo "❌ Rust not found. Please install Rust 1.70+"
    exit 1
fi

# Check Docker
if command -v docker &> /dev/null; then
    echo "✅ Docker found"
else
    echo "⚠️  Docker not found. Some services may not work without Docker"
fi

echo ""
echo "📦 Installing dependencies..."

# Frontend setup (when files are migrated)
if [ -f "$PROJECT_ROOT/frontend/package.json" ]; then
    echo "📱 Setting up frontend..."
    cd "$PROJECT_ROOT/frontend"
    npm install
    echo "✅ Frontend dependencies installed"
else
    echo "⏳ Frontend package.json not found - will be created during migration"
fi

# Backend setup (when Cargo.toml is created)
if [ -f "$PROJECT_ROOT/backend/Cargo.toml" ]; then
    echo "🔧 Setting up backend..."
    cd "$PROJECT_ROOT/backend"
    cargo build
    echo "✅ Backend dependencies installed"
else
    echo "⏳ Backend Cargo.toml not found - will be created during migration"
fi

# Soroban setup (when workspace is created)
if [ -f "$PROJECT_ROOT/soroban/Cargo.toml" ]; then
    echo "⛓️  Setting up Soroban contracts..."
    cd "$PROJECT_ROOT/soroban"
    
    # Add WASM target if not present
    rustup target add wasm32-unknown-unknown
    
    cargo build --target wasm32-unknown-unknown --release
    echo "✅ Soroban contracts built"
else
    echo "⏳ Soroban Cargo.toml not found - will be created during migration"
fi

cd "$PROJECT_ROOT"

echo ""
echo "🐳 Setting up Docker services..."

# Create basic docker-compose.yml if it doesn't exist
if [ ! -f "$PROJECT_ROOT/docker-compose.yml" ]; then
    cat > "$PROJECT_ROOT/docker-compose.yml" << 'EOF'
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

  soroban-rpc:
    image: stellar/quickstart:soroban-dev
    ports:
      - "8000:8000"
    command: --local --enable-soroban-rpc

volumes:
  postgres_data:
EOF
    echo "✅ Created docker-compose.yml"
fi

# Start infrastructure services
echo "🚀 Starting infrastructure services..."
docker-compose up -d postgres soroban-rpc

echo ""
echo "✅ Development environment setup complete!"
echo ""
echo "📋 Next steps:"
echo "   1. Complete the migration by running remaining tasks"
echo "   2. Configure environment variables in each component"
echo "   3. Run database migrations: cd backend && cargo loco db migrate"
echo "   4. Start development servers:"
echo "      - Frontend: cd frontend && npm run dev"
echo "      - Backend: cd backend && cargo loco start"
echo ""
echo "🌐 Services will be available at:"
echo "   - Frontend: http://localhost:3000"
echo "   - Backend API: http://localhost:8080"
echo "   - PostgreSQL: localhost:5432"
echo "   - Soroban RPC: http://localhost:8000"