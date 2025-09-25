#!/bin/bash
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}🚀 Setting up Bitcoin Custody full-stack development environment...${NC}"

# Check if Docker and Docker Compose are installed
if ! command -v docker &> /dev/null; then
    echo -e "${RED}❌ Docker is not installed. Please install Docker first.${NC}"
    exit 1
fi

if ! command -v docker-compose &> /dev/null; then
    echo -e "${RED}❌ Docker Compose is not installed. Please install Docker Compose first.${NC}"
    exit 1
fi

# Check if required directories exist
if [ ! -d "frontend" ] || [ ! -d "backend" ] || [ ! -d "soroban" ]; then
    echo -e "${RED}❌ Required directories (frontend, backend, soroban) not found.${NC}"
    echo -e "${YELLOW}Please ensure you're running this from the project root directory.${NC}"
    exit 1
fi

# Create environment files if they don't exist
echo -e "${YELLOW}📝 Creating environment files...${NC}"

# Backend environment
if [ ! -f "backend/.env" ]; then
    cp backend/.env.example backend/.env 2>/dev/null || cat > backend/.env << EOF
DATABASE_URL=postgres://postgres:password@localhost:5432/bitcoin_custody_dev
RUST_LOG=debug
SOROBAN_RPC_URL=http://localhost:8000
SOROBAN_NETWORK_PASSPHRASE=Standalone Network ; February 2017
JWT_SECRET=development-secret-key-change-in-production
CORS_ORIGINS=http://localhost:3000
EOF
    echo -e "${GREEN}✅ Created backend/.env${NC}"
fi

# Frontend environment
if [ ! -f "frontend/.env.development" ]; then
    cp frontend/.env.example frontend/.env.development 2>/dev/null || cat > frontend/.env.development << EOF
VITE_API_URL=http://localhost:8080
VITE_WS_URL=ws://localhost:8080
VITE_SOROBAN_RPC_URL=http://localhost:8000
NODE_ENV=development
EOF
    echo -e "${GREEN}✅ Created frontend/.env.development${NC}"
fi

# Install frontend dependencies locally (for IDE support)
echo -e "${YELLOW}📦 Installing frontend dependencies...${NC}"
cd frontend
if [ -f "package.json" ]; then
    npm install
    echo -e "${GREEN}✅ Frontend dependencies installed${NC}"
else
    echo -e "${RED}❌ frontend/package.json not found${NC}"
    exit 1
fi
cd ..

# Build Docker images
echo -e "${YELLOW}🐳 Building Docker images...${NC}"
docker-compose build --parallel

# Create database initialization script
mkdir -p backend/scripts
cat > backend/scripts/init-db.sql << EOF
-- Initialize database for development
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Create development user if not exists
DO \$\$
BEGIN
    IF NOT EXISTS (SELECT FROM pg_catalog.pg_roles WHERE rolname = 'dev_user') THEN
        CREATE ROLE dev_user WITH LOGIN PASSWORD 'dev_password';
    END IF;
END
\$\$;

-- Grant permissions
GRANT ALL PRIVILEGES ON DATABASE bitcoin_custody_dev TO dev_user;
EOF

echo -e "${GREEN}✅ Development environment setup complete!${NC}"
echo -e "${BLUE}📋 Next steps:${NC}"
echo -e "  1. Run: ${YELLOW}./scripts/dev-start.sh${NC} to start all services"
echo -e "  2. Access frontend at: ${YELLOW}http://localhost:3000${NC}"
echo -e "  3. Access backend API at: ${YELLOW}http://localhost:8080${NC}"
echo -e "  4. Access Soroban RPC at: ${YELLOW}http://localhost:8000${NC}"
echo -e "  5. Run: ${YELLOW}./scripts/dev-stop.sh${NC} to stop all services"