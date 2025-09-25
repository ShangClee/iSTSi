#!/bin/bash

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}📊 Bitcoin Custody Development Environment Status${NC}"
echo -e "${BLUE}================================================${NC}"

# Check if Docker Compose is running
if ! docker-compose ps &> /dev/null; then
    echo -e "${RED}❌ Docker Compose is not running or not available${NC}"
    exit 1
fi

# Function to get service status with health check
get_service_status() {
    local service=$1
    local status=$(docker-compose ps -q $service 2>/dev/null)
    
    if [ -z "$status" ]; then
        echo -e "${RED}❌ Not running${NC}"
        return
    fi
    
    local health=$(docker inspect --format='{{.State.Health.Status}}' $(docker-compose ps -q $service) 2>/dev/null)
    local state=$(docker inspect --format='{{.State.Status}}' $(docker-compose ps -q $service) 2>/dev/null)
    
    if [ "$state" = "running" ]; then
        if [ "$health" = "healthy" ]; then
            echo -e "${GREEN}✅ Running (Healthy)${NC}"
        elif [ "$health" = "unhealthy" ]; then
            echo -e "${YELLOW}⚠️  Running (Unhealthy)${NC}"
        elif [ "$health" = "starting" ]; then
            echo -e "${YELLOW}🔄 Running (Starting)${NC}"
        else
            echo -e "${GREEN}✅ Running${NC}"
        fi
    else
        echo -e "${RED}❌ $state${NC}"
    fi
}

# Function to get service URL
get_service_url() {
    local service=$1
    local port=$2
    local path=$3
    
    if docker-compose ps $service | grep -q "Up"; then
        echo -e "${BLUE}http://localhost:$port$path${NC}"
    else
        echo -e "${RED}Service not running${NC}"
    fi
}

# Check each service
echo -e "\n${YELLOW}🔍 Service Status:${NC}"
echo -e "  PostgreSQL:  $(get_service_status postgres)"
echo -e "  Redis:       $(get_service_status redis)"
echo -e "  Soroban RPC: $(get_service_status soroban-rpc)"
echo -e "  Backend:     $(get_service_status backend)"
echo -e "  Frontend:    $(get_service_status frontend)"

# Show service URLs
echo -e "\n${YELLOW}🌐 Service URLs:${NC}"
echo -e "  Frontend:    $(get_service_url frontend 3000 "")"
echo -e "  Backend API: $(get_service_url backend 8080 "/api")"
echo -e "  Soroban RPC: $(get_service_url soroban-rpc 8000 "")"

# Show resource usage
echo -e "\n${YELLOW}💾 Resource Usage:${NC}"
docker-compose ps --format "table {{.Name}}\t{{.Status}}\t{{.Ports}}"

# Show recent logs summary
echo -e "\n${YELLOW}📋 Recent Activity (last 5 lines per service):${NC}"
for service in postgres redis soroban-rpc backend frontend; do
    if docker-compose ps -q $service &> /dev/null && [ -n "$(docker-compose ps -q $service)" ]; then
        echo -e "\n${BLUE}--- $service ---${NC}"
        docker-compose logs --tail=3 $service 2>/dev/null | tail -3
    fi
done

# Health check endpoints
echo -e "\n${YELLOW}🏥 Health Check Commands:${NC}"
echo -e "  Backend:     ${BLUE}curl http://localhost:8080/api/health${NC}"
echo -e "  Soroban:     ${BLUE}curl http://localhost:8000/health${NC}"
echo -e "  Frontend:    ${BLUE}curl http://localhost:3000${NC}"