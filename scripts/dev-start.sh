#!/bin/bash
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}🚀 Starting Bitcoin Custody development environment...${NC}"

# Function to check if a service is healthy
check_service_health() {
    local service_name=$1
    local max_attempts=30
    local attempt=1
    
    echo -e "${YELLOW}⏳ Waiting for $service_name to be healthy...${NC}"
    
    while [ $attempt -le $max_attempts ]; do
        if docker-compose ps $service_name | grep -q "healthy"; then
            echo -e "${GREEN}✅ $service_name is healthy${NC}"
            return 0
        fi
        
        echo -e "${YELLOW}   Attempt $attempt/$max_attempts - $service_name not ready yet...${NC}"
        sleep 5
        attempt=$((attempt + 1))
    done
    
    echo -e "${RED}❌ $service_name failed to become healthy after $max_attempts attempts${NC}"
    return 1
}

# Function to show service logs
show_logs() {
    echo -e "${BLUE}📋 Recent logs from all services:${NC}"
    docker-compose logs --tail=10
}

# Trap to handle cleanup on script exit
cleanup() {
    echo -e "\n${YELLOW}🛑 Stopping development environment...${NC}"
    docker-compose down
    exit 0
}
trap cleanup INT TERM

# Start infrastructure services first
echo -e "${YELLOW}🗄️  Starting infrastructure services (postgres, redis, soroban-rpc)...${NC}"
docker-compose up -d postgres redis soroban-rpc

# Wait for infrastructure to be ready
check_service_health "postgres" || exit 1
check_service_health "redis" || exit 1
check_service_health "soroban-rpc" || exit 1

# Run database migrations
echo -e "${YELLOW}🔄 Running database migrations...${NC}"
docker-compose run --rm backend cargo loco db migrate || {
    echo -e "${RED}❌ Database migration failed${NC}"
    show_logs
    exit 1
}

# Start application services
echo -e "${YELLOW}🚀 Starting application services (backend, frontend)...${NC}"
docker-compose up -d backend frontend

# Wait for application services to be ready
check_service_health "backend" || {
    echo -e "${RED}❌ Backend failed to start${NC}"
    show_logs
    exit 1
}

check_service_health "frontend" || {
    echo -e "${RED}❌ Frontend failed to start${NC}"
    show_logs
    exit 1
}

echo -e "${GREEN}🎉 Development environment is ready!${NC}"
echo -e "${BLUE}📋 Service URLs:${NC}"
echo -e "  Frontend:    ${YELLOW}http://localhost:3000${NC}"
echo -e "  Backend API: ${YELLOW}http://localhost:8080${NC}"
echo -e "  Soroban RPC: ${YELLOW}http://localhost:8001${NC}"
echo -e "  PostgreSQL:  ${YELLOW}localhost:5432${NC}"
echo -e "  Redis:       ${YELLOW}localhost:6379${NC}"

echo -e "\n${BLUE}🔧 Useful commands:${NC}"
echo -e "  View logs:           ${YELLOW}docker-compose logs -f [service]${NC}"
echo -e "  Restart service:     ${YELLOW}docker-compose restart [service]${NC}"
echo -e "  Run backend command: ${YELLOW}docker-compose exec backend [command]${NC}"
echo -e "  Run frontend command:${YELLOW}docker-compose exec frontend [command]${NC}"
echo -e "  Stop environment:    ${YELLOW}./scripts/dev-stop.sh${NC}"

echo -e "\n${YELLOW}📊 Monitoring logs (Ctrl+C to stop):${NC}"
docker-compose logs -f