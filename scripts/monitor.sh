#!/bin/bash

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
MONITOR_INTERVAL=${1:-30}  # Default 30 seconds
LOG_FILE="./logs/monitor.log"

# Create logs directory if it doesn't exist
mkdir -p logs

echo -e "${BLUE}🔍 Starting Bitcoin Custody Service Monitor${NC}"
echo -e "${BLUE}Monitor interval: ${YELLOW}${MONITOR_INTERVAL}s${NC}"
echo -e "${BLUE}Log file: ${YELLOW}${LOG_FILE}${NC}"
echo -e "${BLUE}Press Ctrl+C to stop monitoring${NC}"
echo -e "${BLUE}========================================${NC}"

# Function to log with timestamp
log_with_timestamp() {
    echo "$(date '+%Y-%m-%d %H:%M:%S') - $1" | tee -a "$LOG_FILE"
}

# Function to check service health
check_service_health() {
    local service=$1
    local url=$2
    local timeout=5
    
    if curl -s --connect-timeout $timeout "$url" > /dev/null 2>&1; then
        return 0
    else
        return 1
    fi
}

# Function to get container stats
get_container_stats() {
    local service=$1
    local container_id=$(docker-compose ps -q $service 2>/dev/null)
    
    if [ -n "$container_id" ]; then
        docker stats --no-stream --format "table {{.CPUPerc}}\t{{.MemUsage}}" $container_id 2>/dev/null | tail -n 1
    else
        echo "N/A\tN/A"
    fi
}

# Trap to handle cleanup on script exit
cleanup() {
    echo -e "\n${YELLOW}🛑 Stopping monitor...${NC}"
    log_with_timestamp "Monitor stopped"
    exit 0
}
trap cleanup INT TERM

# Start monitoring
log_with_timestamp "Monitor started with ${MONITOR_INTERVAL}s interval"

while true; do
    clear
    echo -e "${BLUE}🔍 Bitcoin Custody Service Monitor - $(date)${NC}"
    echo -e "${BLUE}================================================${NC}"
    
    # Check Docker Compose status
    if ! docker-compose ps &> /dev/null; then
        echo -e "${RED}❌ Docker Compose not available${NC}"
        log_with_timestamp "ERROR: Docker Compose not available"
        sleep $MONITOR_INTERVAL
        continue
    fi
    
    # Service status table
    echo -e "\n${YELLOW}📊 Service Status:${NC}"
    printf "%-15s %-12s %-15s %-20s %s\n" "Service" "Status" "Health" "Resources" "Last Check"
    printf "%-15s %-12s %-15s %-20s %s\n" "-------" "------" "------" "---------" "----------"
    
    services=("postgres" "redis" "soroban-rpc" "backend" "frontend")
    health_urls=("" "" "http://localhost:8000/health" "http://localhost:8080/api/health" "http://localhost:3000")
    
    all_healthy=true
    
    for i in "${!services[@]}"; do
        service="${services[$i]}"
        health_url="${health_urls[$i]}"
        
        # Get container status
        if docker-compose ps $service | grep -q "Up"; then
            status="${GREEN}Running${NC}"
            
            # Check health if URL provided
            if [ -n "$health_url" ]; then
                if check_service_health "$service" "$health_url"; then
                    health="${GREEN}Healthy${NC}"
                else
                    health="${RED}Unhealthy${NC}"
                    all_healthy=false
                fi
            else
                health="${BLUE}N/A${NC}"
            fi
            
            # Get resource usage
            resources=$(get_container_stats "$service")
            
        else
            status="${RED}Stopped${NC}"
            health="${RED}Down${NC}"
            resources="N/A\tN/A"
            all_healthy=false
        fi
        
        printf "%-25s %-22s %-25s %-20s %s\n" "$service" "$status" "$health" "$resources" "$(date '+%H:%M:%S')"
    done
    
    # Overall system status
    echo -e "\n${YELLOW}🎯 Overall Status:${NC}"
    if $all_healthy; then
        echo -e "  ${GREEN}✅ All services healthy${NC}"
        log_with_timestamp "INFO: All services healthy"
    else
        echo -e "  ${RED}❌ Some services unhealthy${NC}"
        log_with_timestamp "WARNING: Some services unhealthy"
    fi
    
    # Resource summary
    echo -e "\n${YELLOW}💾 Resource Usage:${NC}"
    docker stats --no-stream --format "table {{.Name}}\t{{.CPUPerc}}\t{{.MemUsage}}\t{{.NetIO}}" $(docker-compose ps -q) 2>/dev/null | head -6
    
    # Recent logs summary
    echo -e "\n${YELLOW}📋 Recent Activity (last 2 lines per service):${NC}"
    for service in "${services[@]}"; do
        if docker-compose ps -q $service &> /dev/null && [ -n "$(docker-compose ps -q $service)" ]; then
            echo -e "${BLUE}--- $service ---${NC}"
            docker-compose logs --tail=1 $service 2>/dev/null | tail -1
        fi
    done
    
    # Wait for next check
    echo -e "\n${BLUE}⏱️  Next check in ${MONITOR_INTERVAL}s... (Ctrl+C to stop)${NC}"
    sleep $MONITOR_INTERVAL
done