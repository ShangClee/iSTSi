#!/bin/bash
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}🛑 Stopping Bitcoin Custody development environment...${NC}"

# Stop all services gracefully
echo -e "${YELLOW}⏹️  Stopping all services...${NC}"
docker-compose down

# Option to clean up volumes and images
read -p "$(echo -e ${YELLOW}🧹 Do you want to clean up volumes and unused images? [y/N]: ${NC})" -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    echo -e "${YELLOW}🗑️  Removing volumes...${NC}"
    docker-compose down -v
    
    echo -e "${YELLOW}🗑️  Removing unused Docker images...${NC}"
    docker image prune -f
    
    echo -e "${GREEN}✅ Cleanup complete${NC}"
else
    echo -e "${BLUE}ℹ️  Volumes and images preserved${NC}"
fi

echo -e "${GREEN}✅ Development environment stopped${NC}"
echo -e "${BLUE}💡 To start again, run: ${YELLOW}./scripts/dev-start.sh${NC}"