#!/bin/bash

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Default service (all if not specified)
SERVICE=${1:-""}
LINES=${2:-50}

echo -e "${BLUE}📋 Bitcoin Custody Development Logs${NC}"

if [ -z "$SERVICE" ]; then
    echo -e "${YELLOW}📊 Showing logs for all services (last $LINES lines each)${NC}"
    echo -e "${BLUE}Use: $0 [service] [lines] to view specific service logs${NC}"
    echo -e "${BLUE}Available services: postgres, redis, soroban-rpc, backend, frontend${NC}"
    echo -e "${BLUE}================================================${NC}"
    
    # Show logs for all services
    docker-compose logs --tail=$LINES -f
else
    # Validate service name
    if ! docker-compose ps $SERVICE &> /dev/null; then
        echo -e "${RED}❌ Service '$SERVICE' not found${NC}"
        echo -e "${YELLOW}Available services:${NC}"
        docker-compose ps --services
        exit 1
    fi
    
    echo -e "${YELLOW}📊 Showing logs for $SERVICE (last $LINES lines)${NC}"
    echo -e "${BLUE}================================================${NC}"
    
    # Show logs for specific service
    docker-compose logs --tail=$LINES -f $SERVICE
fi