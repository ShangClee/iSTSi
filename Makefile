# Bitcoin Custody Development Makefile
# Provides convenient shortcuts for common development tasks

.PHONY: help setup start stop status logs health clean test build

# Default target
help: ## Show this help message
	@echo "Bitcoin Custody Development Commands:"
	@echo "====================================="
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-15s\033[0m %s\n", $$1, $$2}'

setup: ## Initial setup of development environment
	@echo "🚀 Setting up development environment..."
	@./scripts/dev-setup.sh

start: ## Start all development services
	@echo "🚀 Starting development environment..."
	@./scripts/dev-start.sh

stop: ## Stop all development services
	@echo "🛑 Stopping development environment..."
	@./scripts/dev-stop.sh

status: ## Check status of all services
	@echo "📊 Checking service status..."
	@./scripts/dev-status.sh

logs: ## View logs from all services
	@echo "📋 Showing logs..."
	@./scripts/dev-logs.sh

logs-backend: ## View backend logs only
	@./scripts/dev-logs.sh backend

logs-frontend: ## View frontend logs only
	@./scripts/dev-logs.sh frontend

logs-postgres: ## View PostgreSQL logs only
	@./scripts/dev-logs.sh postgres

logs-soroban: ## View Soroban RPC logs only
	@./scripts/dev-logs.sh soroban-rpc

health: ## Run comprehensive health check
	@echo "🏥 Running health check..."
	@./scripts/health-check.sh

clean: ## Clean up Docker resources (removes volumes)
	@echo "🧹 Cleaning up Docker resources..."
	@docker-compose down -v --remove-orphans
	@docker system prune -f

reset: clean setup start ## Complete reset of development environment

# Development commands
dev-backend: ## Run backend development server locally
	@echo "🦀 Starting backend development server..."
	@cd backend && cargo loco start --environment development

dev-frontend: ## Run frontend development server locally
	@echo "⚛️  Starting frontend development server..."
	@cd frontend && npm run dev

# Testing commands
test: ## Run all tests
	@echo "🧪 Running all tests..."
	@./scripts/test.sh all

test-unit: ## Run unit tests only
	@echo "🧪 Running unit tests..."
	@./scripts/test.sh all unit

test-integration: ## Run integration tests only
	@echo "🧪 Running integration tests..."
	@./scripts/test.sh integration

test-e2e: ## Run end-to-end tests
	@echo "🧪 Running E2E tests..."
	@./scripts/test.sh e2e

test-backend: ## Run backend tests only
	@echo "🦀 Running backend tests..."
	@./scripts/test.sh backend

test-frontend: ## Run frontend tests only
	@echo "⚛️  Running frontend tests..."
	@./scripts/test.sh frontend

test-contracts: ## Run Soroban contract tests
	@echo "🌟 Running contract tests..."
	@./scripts/test.sh soroban

test-parallel: ## Run tests in parallel
	@echo "🧪 Running tests in parallel..."
	@./scripts/test.sh all all --parallel

test-coverage: ## Run tests with coverage
	@echo "🧪 Running tests with coverage..."
	@./scripts/test.sh all all --coverage

# Build commands
build: ## Build all components for production
	@echo "🔨 Building all components for production..."
	@./scripts/build.sh all production

build-dev: ## Build all components for development
	@echo "🔨 Building all components for development..."
	@./scripts/build.sh all development

build-backend: ## Build backend only
	@echo "🦀 Building backend..."
	@./scripts/build.sh backend

build-frontend: ## Build frontend only
	@echo "⚛️  Building frontend..."
	@./scripts/build.sh frontend

build-contracts: ## Build Soroban contracts
	@echo "🌟 Building contracts..."
	@./scripts/build.sh soroban

build-docker: ## Build Docker images
	@echo "🐳 Building Docker images..."
	@./scripts/build.sh docker

# Database commands
db-migrate: ## Run database migrations
	@echo "🗄️  Running database migrations..."
	@docker-compose exec backend cargo loco db migrate

db-reset: ## Reset database
	@echo "🗄️  Resetting database..."
	@docker-compose exec backend cargo loco db reset

db-shell: ## Access database shell
	@echo "🗄️  Opening database shell..."
	@docker-compose exec postgres psql -U postgres -d bitcoin_custody_dev

# Deployment commands
deploy-dev: ## Deploy to development environment
	@echo "🚀 Deploying to development..."
	@./scripts/deploy.sh development

deploy-staging: ## Deploy to staging environment
	@echo "🚀 Deploying to staging..."
	@./scripts/deploy.sh staging

deploy-prod: ## Deploy to production environment
	@echo "🚀 Deploying to production..."
	@./scripts/deploy.sh production

deploy-frontend-dev: ## Deploy frontend to development
	@echo "🚀 Deploying frontend to development..."
	@./scripts/deploy.sh development frontend

deploy-backend-dev: ## Deploy backend to development
	@echo "🚀 Deploying backend to development..."
	@./scripts/deploy.sh development backend

deploy-contracts-dev: ## Deploy contracts to development
	@echo "🚀 Deploying contracts to development..."
	@./scripts/deploy.sh development soroban

deploy-dry-run: ## Dry run deployment to production
	@echo "🚀 Dry run deployment to production..."
	@./scripts/deploy.sh production all --dry-run

# Utility commands
shell-backend: ## Access backend container shell
	@docker-compose exec backend bash

shell-frontend: ## Access frontend container shell
	@docker-compose exec frontend sh

install-frontend: ## Install frontend dependencies
	@echo "📦 Installing frontend dependencies..."
	@cd frontend && npm install

install-backend: ## Install backend dependencies
	@echo "📦 Installing backend dependencies..."
	@cd backend && cargo build

# Service management
restart-backend: ## Restart backend service
	@docker-compose restart backend

restart-frontend: ## Restart frontend service
	@docker-compose restart frontend

restart-postgres: ## Restart PostgreSQL service
	@docker-compose restart postgres

# Documentation
docs: ## Open development documentation
	@echo "📚 Opening development documentation..."
	@open DEVELOPMENT.md || xdg-open DEVELOPMENT.md || cat DEVELOPMENT.md