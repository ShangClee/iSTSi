# Migration Guides

This document provides step-by-step migration guides for upgrading between different versions of the Bitcoin Custody Full-Stack Application components.

## Table of Contents

- [General Migration Principles](#general-migration-principles)
- [Patch Version Migrations (x.y.z → x.y.z+1)](#patch-version-migrations)
- [Minor Version Migrations (x.y.0 → x.y+1.0)](#minor-version-migrations)
- [Major Version Migrations (x.0.0 → x+1.0.0)](#major-version-migrations)
- [Component-Specific Migrations](#component-specific-migrations)
- [Rollback Procedures](#rollback-procedures)
- [Troubleshooting](#troubleshooting)

## General Migration Principles

### Pre-Migration Checklist

Before starting any migration:

1. **Backup Everything**
   ```bash
   # Backup database
   pg_dump bitcoin_custody_prod > backup_$(date +%Y%m%d_%H%M%S).sql
   
   # Backup configuration
   cp -r config/ config_backup_$(date +%Y%m%d_%H%M%S)/
   
   # Tag current version
   git tag -a "pre-migration-$(date +%Y%m%d)" -m "Pre-migration backup"
   ```

2. **Test in Staging**
   - Always test migrations in staging environment first
   - Validate all functionality works after migration
   - Performance test with production-like data

3. **Check Dependencies**
   ```bash
   # Validate current setup
   ./scripts/dependency-validator.sh all
   
   # Check version compatibility
   ./scripts/version-manager.sh compatibility
   ```

4. **Plan Downtime**
   - Schedule maintenance window for production
   - Prepare rollback plan and timeline
   - Notify stakeholders of planned downtime

### Migration Order

Always migrate components in this order to maintain dependencies:

1. **Soroban Contracts** (blockchain layer)
2. **Backend Services** (API layer)  
3. **Frontend Application** (UI layer)

This ensures that dependent services can connect to updated dependencies.

## Patch Version Migrations

Patch versions (1.0.0 → 1.0.1) contain bug fixes and small improvements with no breaking changes.

### Migration Steps

1. **Update Individual Components**
   ```bash
   # Update frontend
   cd frontend
   npm update
   npm run build
   
   # Update backend
   cd ../backend
   cargo update
   cargo build --release
   
   # Update soroban
   cd ../soroban
   cargo update
   cargo build --target wasm32-unknown-unknown --release
   ```

2. **Restart Services**
   ```bash
   # Development
   docker-compose restart
   
   # Production
   systemctl restart bitcoin-custody-backend
   systemctl restart bitcoin-custody-frontend
   ```

3. **Verify Functionality**
   ```bash
   # Run health checks
   curl http://localhost:8080/health
   curl http://localhost:3000/health
   
   # Run smoke tests
   npm run test:smoke
   ```

### Rollback for Patch Versions

```bash
# Revert to previous version
git checkout previous-patch-tag

# Rebuild and restart
docker-compose down
docker-compose up --build
```

## Minor Version Migrations

Minor versions (1.0.0 → 1.1.0) add new features while maintaining backward compatibility.

### Pre-Migration Steps

1. **Review Release Notes**
   ```bash
   # Check what's new
   cat CHANGELOG.md | grep -A 20 "## \[.*v1.1.0\]"
   ```

2. **Update Configuration**
   ```bash
   # Check for new config options
   diff config/development.yaml.example config/development.yaml
   ```

### Migration Steps

1. **Update Soroban Contracts**
   ```bash
   cd soroban
   
   # Build new contracts
   cargo build --target wasm32-unknown-unknown --release
   
   # Deploy to testnet first
   ./scripts/deploy-testnet.sh
   
   # Verify deployment
   ./scripts/test.sh
   
   # Deploy to mainnet (production only)
   ./scripts/deploy-mainnet.sh
   ```

2. **Update Backend**
   ```bash
   cd backend
   
   # Run database migrations
   cargo loco db migrate
   
   # Build new version
   cargo build --release
   
   # Update configuration if needed
   cp config/production.yaml.example config/production.yaml
   # Edit config/production.yaml with your settings
   
   # Restart backend
   systemctl restart bitcoin-custody-backend
   ```

3. **Update Frontend**
   ```bash
   cd frontend
   
   # Install new dependencies
   npm install
   
   # Build production version
   npm run build
   
   # Deploy static files
   cp -r dist/* /var/www/bitcoin-custody/
   
   # Restart web server
   systemctl restart nginx
   ```

### Post-Migration Validation

```bash
# Run integration tests
npm run test:integration

# Check all endpoints
./scripts/health-check.sh

# Verify new features work
./scripts/feature-validation.sh
```

### Rollback for Minor Versions

```bash
# 1. Revert code
git checkout previous-minor-tag

# 2. Rollback database (if migrations were run)
cd backend
cargo loco db rollback --steps 1

# 3. Redeploy contracts (if changed)
cd ../soroban
./scripts/deploy-mainnet.sh --rollback

# 4. Rebuild and restart
docker-compose down
docker-compose up --build
```

## Major Version Migrations

Major versions (1.x.x → 2.0.0) contain breaking changes that require careful migration.

### Pre-Migration Planning

1. **Read Breaking Changes**
   ```bash
   # Review all breaking changes
   cat CHANGELOG.md | grep -A 50 "## \[.*v2.0.0\]" | grep -A 20 "### Breaking"
   ```

2. **Plan Migration Timeline**
   - Schedule extended maintenance window (2-4 hours)
   - Prepare detailed rollback plan
   - Set up monitoring and alerting

3. **Prepare Migration Scripts**
   ```bash
   # Check for provided migration scripts
   ls scripts/migrations/v2.0.0/
   
   # Test migration scripts in staging
   ./scripts/migrations/v2.0.0/test-migration.sh
   ```

### Migration Steps

1. **Database Migration**
   ```bash
   cd backend
   
   # Backup database
   pg_dump bitcoin_custody_prod > backup_pre_v2.sql
   
   # Run migrations with verification
   cargo loco db migrate --dry-run
   cargo loco db migrate
   
   # Verify data integrity
   ./scripts/verify-migration.sh
   ```

2. **Contract Migration**
   ```bash
   cd soroban
   
   # Deploy new contracts
   ./scripts/deploy-mainnet.sh --version 2.0.0
   
   # Migrate contract state (if needed)
   ./scripts/migrate-contract-state.sh
   
   # Verify contract functionality
   ./scripts/test-contracts.sh
   ```

3. **Backend Migration**
   ```bash
   cd backend
   
   # Update configuration for v2.0.0
   cp config/v2.0.0/production.yaml config/production.yaml
   
   # Build new version
   cargo build --release
   
   # Run post-migration scripts
   ./scripts/post-migration.sh
   
   # Start new backend
   systemctl start bitcoin-custody-backend-v2
   systemctl stop bitcoin-custody-backend-v1
   ```

4. **Frontend Migration**
   ```bash
   cd frontend
   
   # Clear browser caches (important for major versions)
   # Update cache-busting in build
   
   # Build and deploy
   npm run build
   cp -r dist/* /var/www/bitcoin-custody/
   
   # Update nginx config if needed
   systemctl reload nginx
   ```

### Post-Migration Validation

```bash
# Comprehensive testing
./scripts/test-suite.sh --full

# Performance validation
./scripts/performance-test.sh

# Security validation
./scripts/security-audit.sh

# User acceptance testing
./scripts/uat-checklist.sh
```

### Rollback for Major Versions

Major version rollbacks are complex and should be avoided. If necessary:

```bash
# 1. Stop new services
systemctl stop bitcoin-custody-backend-v2

# 2. Restore database
psql bitcoin_custody_prod < backup_pre_v2.sql

# 3. Rollback contracts
cd soroban
./scripts/rollback-contracts.sh --to-version 1.x.x

# 4. Start old services
systemctl start bitcoin-custody-backend-v1

# 5. Restore old frontend
git checkout v1.x.x
cd frontend
npm run build
cp -r dist/* /var/www/bitcoin-custody/
```

## Component-Specific Migrations

### Frontend Migrations

#### React Version Updates
```bash
# Update React and related packages
npm update react react-dom @types/react @types/react-dom

# Check for breaking changes in components
npm run lint
npm run type-check

# Update component patterns if needed
./scripts/update-components.sh
```

#### State Management Changes
```bash
# Update Redux Toolkit if needed
npm update @reduxjs/toolkit react-redux

# Migrate store structure
./scripts/migrate-store.sh

# Update component connections
./scripts/update-selectors.sh
```

### Backend Migrations

#### Loco.rs Framework Updates
```bash
# Update Loco.rs
cargo update loco-rs

# Check for breaking changes
cargo check

# Update controller patterns
./scripts/update-controllers.sh

# Update middleware
./scripts/update-middleware.sh
```

#### Database Schema Changes
```bash
# Generate new migration
cargo loco generate migration add_new_feature

# Edit migration file
# Run migration
cargo loco db migrate

# Update models
./scripts/update-models.sh
```

### Soroban Migrations

#### Contract Upgrades
```bash
# Build new contract versions
cargo build --target wasm32-unknown-unknown --release

# Deploy with upgrade mechanism
./scripts/upgrade-contracts.sh

# Migrate contract storage
./scripts/migrate-storage.sh
```

#### Network Changes
```bash
# Update network configuration
cp config/networks/mainnet-v2.yaml config/networks/mainnet.yaml

# Update RPC endpoints
./scripts/update-rpc-config.sh

# Test network connectivity
./scripts/test-network.sh
```

## Rollback Procedures

### Automated Rollback

```bash
# Use the rollback script
./scripts/rollback.sh --to-version 1.2.3 --component all

# Or rollback specific component
./scripts/rollback.sh --to-version 1.2.3 --component backend
```

### Manual Rollback Steps

1. **Identify Rollback Point**
   ```bash
   # List available versions
   git tag --sort=-version:refname
   
   # Check deployment history
   ./scripts/deployment-history.sh
   ```

2. **Execute Rollback**
   ```bash
   # Rollback code
   git checkout v1.2.3
   
   # Rollback database
   cargo loco db rollback --to-migration 20240101000001
   
   # Rollback contracts
   ./scripts/rollback-contracts.sh --to-version 1.2.3
   
   # Rebuild and restart
   docker-compose down
   docker-compose up --build
   ```

3. **Verify Rollback**
   ```bash
   # Check versions
   ./scripts/version-manager.sh show
   
   # Run health checks
   ./scripts/health-check.sh
   
   # Validate functionality
   ./scripts/smoke-test.sh
   ```

## Troubleshooting

### Common Migration Issues

#### Database Migration Failures
```bash
# Check migration status
cargo loco db status

# Rollback failed migration
cargo loco db rollback --steps 1

# Fix migration file and retry
cargo loco db migrate
```

#### Contract Deployment Failures
```bash
# Check network connectivity
./scripts/test-network.sh

# Verify contract compilation
cargo build --target wasm32-unknown-unknown --release

# Check deployment logs
./scripts/deployment-logs.sh
```

#### Frontend Build Failures
```bash
# Clear node modules and reinstall
rm -rf node_modules package-lock.json
npm install

# Check for TypeScript errors
npm run type-check

# Update dependencies
npm update
```

#### Version Compatibility Issues
```bash
# Check compatibility matrix
./scripts/version-manager.sh compatibility

# Sync versions
./scripts/version-manager.sh sync 1.2.3

# Validate dependencies
./scripts/dependency-validator.sh all
```

### Emergency Procedures

#### Complete System Rollback
```bash
# Emergency rollback script
./scripts/emergency-rollback.sh

# This script will:
# 1. Stop all services
# 2. Restore from last known good backup
# 3. Restart services
# 4. Verify functionality
```

#### Data Recovery
```bash
# Restore database from backup
./scripts/restore-database.sh backup_20240101_120000.sql

# Verify data integrity
./scripts/verify-data.sh

# Rebuild indexes if needed
./scripts/rebuild-indexes.sh
```

### Getting Help

1. **Check Logs**
   ```bash
   # Backend logs
   journalctl -u bitcoin-custody-backend -f
   
   # Frontend logs (nginx)
   tail -f /var/log/nginx/error.log
   
   # Contract logs
   ./scripts/contract-logs.sh
   ```

2. **Run Diagnostics**
   ```bash
   # System diagnostics
   ./scripts/system-diagnostics.sh
   
   # Component health check
   ./scripts/health-check.sh --verbose
   ```

3. **Contact Support**
   - Create GitHub issue with migration details
   - Include logs and error messages
   - Specify versions involved in migration
   - Describe steps taken and current system state

## Best Practices

1. **Always Test First**: Never migrate production without testing in staging
2. **Backup Everything**: Database, configuration, and code
3. **Monitor Closely**: Watch system metrics during and after migration
4. **Have Rollback Ready**: Prepare and test rollback procedures
5. **Document Changes**: Keep detailed logs of migration steps
6. **Validate Thoroughly**: Test all functionality after migration
7. **Communicate**: Keep stakeholders informed of progress and issues