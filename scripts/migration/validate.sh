#!/bin/bash
set -e

# Bitcoin Custody System - Migration Validation Script
# This script validates the new project structure after migration

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

echo "🔍 Validating new project structure..."
echo "Project root: $PROJECT_ROOT"
echo ""

# Check main directories exist
echo "📁 Checking main directories..."
required_dirs=(
    "frontend"
    "backend" 
    "soroban"
)

for dir in "${required_dirs[@]}"; do
    if [ -d "$PROJECT_ROOT/$dir" ]; then
        echo "✅ /$dir directory exists"
    else
        echo "❌ /$dir directory missing"
        exit 1
    fi
done

# Check frontend structure
echo ""
echo "📁 Validating frontend structure..."
frontend_dirs=(
    "frontend/src"
    "frontend/src/components"
    "frontend/src/components/ui"
    "frontend/src/services"
    "frontend/src/hooks"
    "frontend/src/store"
    "frontend/src/types"
    "frontend/src/utils"
    "frontend/public"
)

for dir in "${frontend_dirs[@]}"; do
    if [ -d "$PROJECT_ROOT/$dir" ]; then
        echo "✅ $dir exists"
    else
        echo "❌ $dir missing"
        exit 1
    fi
done

# Check backend structure
echo ""
echo "📁 Validating backend structure..."
backend_dirs=(
    "backend/src"
    "backend/src/controllers"
    "backend/src/models"
    "backend/src/services"
    "backend/src/workers"
    "backend/src/middleware"
    "backend/migration"
    "backend/config"
    "backend/tests"
)

for dir in "${backend_dirs[@]}"; do
    if [ -d "$PROJECT_ROOT/$dir" ]; then
        echo "✅ $dir exists"
    else
        echo "❌ $dir missing"
        exit 1
    fi
done

# Check soroban structure
echo ""
echo "📁 Validating soroban structure..."
soroban_dirs=(
    "soroban/contracts"
    "soroban/contracts/integration_router"
    "soroban/contracts/kyc_registry"
    "soroban/contracts/istsi_token"
    "soroban/contracts/reserve_manager"
    "soroban/contracts/fungible"
    "soroban/shared/src"
    "soroban/tests"
    "soroban/scripts"
)

for dir in "${soroban_dirs[@]}"; do
    if [ -d "$PROJECT_ROOT/$dir" ]; then
        echo "✅ $dir exists"
    else
        echo "❌ $dir missing"
        exit 1
    fi
done

# Check README files
echo ""
echo "📄 Checking README files..."
readme_files=(
    "frontend/README.md"
    "backend/README.md"
    "soroban/README.md"
)

for file in "${readme_files[@]}"; do
    if [ -f "$PROJECT_ROOT/$file" ]; then
        echo "✅ $file exists"
    else
        echo "❌ $file missing"
        exit 1
    fi
done

# Check migration scripts
echo ""
echo "🔧 Checking migration scripts..."
migration_files=(
    "scripts/migration/backup.sh"
    "scripts/migration/validate.sh"
)

for file in "${migration_files[@]}"; do
    if [ -f "$PROJECT_ROOT/$file" ]; then
        echo "✅ $file exists"
        if [ -x "$PROJECT_ROOT/$file" ]; then
            echo "✅ $file is executable"
        else
            echo "⚠️  $file is not executable"
        fi
    else
        echo "❌ $file missing"
        exit 1
    fi
done

echo ""
echo "✅ All validation checks passed!"
echo ""
echo "📊 Structure Summary:"
echo "   - Frontend: React + TypeScript setup ready"
echo "   - Backend: Loco.rs structure prepared"
echo "   - Soroban: Contract organization complete"
echo "   - Migration: Backup and validation scripts ready"
echo ""
echo "🚀 Ready for next migration phase!"