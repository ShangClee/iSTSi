#!/bin/bash

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Health check configuration
TIMEOUT=10
MAX_RETRIES=3

# Function to check HTTP endpoint
check_http_endpoint() {
    local name=$1
    local url=$2
    local expected_status=${3:-200}
    
    echo -n "  $name: "
    
    local response=$(curl -s -o /dev/null -w "%{http_code}" --connect-timeout $TIMEOUT "$url" 2>/dev/null)
    
    if [ "$response" = "$expected_status" ]; then
        echo -e "${GREEN}✅ Healthy (HTTP $response)${NC}"
        return 0
    else
        echo -e "${RED}❌ Unhealthy (HTTP $response)${NC}"
        return 1
    fi
}

# Function to check TCP port
check_tcp_port() {
    local name=$1
    local host=$2
    local port=$3
    
    echo -n "  $name: "
    
    if timeout $TIMEOUT bash -c "</dev/tcp/$host/$port" 2>/dev/null; then
        echo -e "${GREEN}✅ Reachable${NC}"
        return 0
    else
        echo -e "${RED}❌ Unreachable${NC}"
        return 1
    fi
}

# Function to check Docker service
check_docker_service() {
    local service=$1
    
    echo -n "  $service: "
    
    if docker-compose ps $service | grep -q "Up"; then
        local health=$(docker inspect --format='{{.State.Health.Status}}' $(docker-compose ps -q $service) 2>/dev/null)
        
        if [ "$health" = "healthy" ]; then
            echo -e "${GREEN}✅ Running (Healthy)${NC}"
            return 0
        elif [ "$health" = "unhealthy" ]; then
            echo -e "${YELLOW}⚠️  Running (Unhealthy)${NC}"
            return 1
        else
            echo -e "${GREEN}✅ Running${NC}"
            return 0
        fi
    else
        echo -e "${RED}❌ Not running${NC}"
        return 1
    fi
}

echo -e "${BLUE}🏥 Bitcoin Custody Health Check${NC}"
echo -e "${BLUE}==============================${NC}"

# Check Docker services
echo -e "\n${YELLOW}🐳 Docker Services:${NC}"
services_healthy=0
total_services=4

check_docker_service "postgres" && ((services_healthy++))
check_docker_service "redis" && ((services_healthy++))
check_docker_service "soroban-rpc" && ((services_healthy++))
check_docker_service "frontend" && ((services_healthy++))
# Backend is not running in Docker for now
# check_docker_service "backend" && ((services_healthy++))

# Check network connectivity
echo -e "\n${YELLOW}🌐 Network Connectivity:${NC}"
network_healthy=0
total_network=4

check_tcp_port "PostgreSQL" "localhost" "5432" && ((network_healthy++))
check_tcp_port "Redis" "localhost" "6379" && ((network_healthy++))
check_tcp_port "Soroban RPC" "localhost" "8001" && ((network_healthy++))
check_tcp_port "Frontend" "localhost" "3000" && ((network_healthy++))
# Backend is not running in Docker for now
# check_tcp_port "Backend API" "localhost" "8080" && ((network_healthy++))

# Check HTTP endpoints
echo -e "\n${YELLOW}🔗 HTTP Endpoints:${NC}"
http_healthy=0
total_http=2

check_http_endpoint "Frontend" "http://localhost:3000" && ((http_healthy++))
check_http_endpoint "Soroban RPC" "http://localhost:8001/health" && ((http_healthy++))
# Backend is not running in Docker for now
# check_http_endpoint "Backend API" "http://localhost:8080/api/health" && ((http_healthy++))

# Summary
echo -e "\n${BLUE}📊 Health Summary:${NC}"
echo -e "  Docker Services: $services_healthy/$total_services healthy"
echo -e "  Network Ports:   $network_healthy/$total_network reachable"
echo -e "  HTTP Endpoints:  $http_healthy/$total_http responding"

total_checks=$((services_healthy + network_healthy + http_healthy))
max_checks=$((total_services + total_network + total_http))

if [ $total_checks -eq $max_checks ]; then
    echo -e "\n${GREEN}🎉 All systems healthy! ($total_checks/$max_checks)${NC}"
    exit 0
elif [ $total_checks -gt $((max_checks * 2 / 3)) ]; then
    echo -e "\n${YELLOW}⚠️  Most systems healthy ($total_checks/$max_checks)${NC}"
    exit 1
else
    echo -e "\n${RED}❌ Multiple systems unhealthy ($total_checks/$max_checks)${NC}"
    echo -e "${YELLOW}💡 Try running: ./scripts/dev-start.sh${NC}"
    exit 2
fi