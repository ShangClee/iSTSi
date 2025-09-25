# Build and Deployment Scripts

This directory contains unified build, test, and deployment scripts for the Bitcoin Custody full-stack application.

## Overview

The scripts provide a consistent interface for building, testing, and deploying all components of the application across different environments.

## Scripts

### 🔨 build.sh

Unified build script for all components.

**Usage:**
```bash
./scripts/build.sh [component] [environment]
```

**Components:**
- `frontend` - Build React frontend only
- `backend` - Build Loco.rs backend only  
- `soroban` - Build Soroban contracts only
- `docker` - Build Docker images only
- `all` - Build all components (default)

**Environments:**
- `development` - Development build with debug symbols
- `staging` - Staging build with optimizations
- `production` - Production build with full optimizations (default)

**Examples:**
```bash
./scripts/build.sh                    # Build all for production
./scripts/build.sh frontend           # Build frontend for production
./scripts/build.sh backend development # Build backend for development
./scripts/build.sh all staging        # Build all for staging
```

**Features:**
- Parallel builds when possible
- Build artifact generation
- Build manifest creation
- Docker image building and tagging
- Comprehensive error handling and logging

### 🧪 test.sh

Unified testing script for all components.

**Usage:**
```bash
./scripts/test.sh [component] [test-type] [options]
```

**Components:**
- `frontend` - Test React frontend only
- `backend` - Test Loco.rs backend only
- `soroban` - Test Soroban contracts only
- `integration` - Test cross-component integration
- `e2e` - Test end-to-end user workflows
- `all` - Test all components (default)

**Test Types:**
- `unit` - Unit tests only
- `integration` - Integration tests only
- `e2e` - End-to-end tests only
- `all` - All test types (default)

**Options:**
- `--parallel` - Run component tests in parallel
- `--coverage` - Generate code coverage reports

**Examples:**
```bash
./scripts/test.sh                           # Run all tests
./scripts/test.sh frontend unit             # Run frontend unit tests
./scripts/test.sh all integration --parallel # Run integration tests in parallel
./scripts/test.sh backend all --coverage    # Run backend tests with coverage
```

**Features:**
- Comprehensive test reporting
- Code coverage generation
- Parallel test execution
- Test result archiving
- Integration with CI/CD pipelines

### 🚀 deploy.sh

Unified deployment script for all environments.

**Usage:**
```bash
./scripts/deploy.sh [environment] [component] [options]
```

**Environments:**
- `development` - Local development deployment
- `staging` - Staging environment deployment
- `production` - Production environment deployment

**Components:**
- `frontend` - Deploy React frontend only
- `backend` - Deploy Loco.rs backend only
- `soroban` - Deploy Soroban contracts only
- `all` - Deploy all components (default)

**Options:**
- `--dry-run` - Show what would be deployed without actually deploying

**Examples:**
```bash
./scripts/deploy.sh development           # Deploy all to development
./scripts/deploy.sh staging frontend      # Deploy frontend to staging
./scripts/deploy.sh production all --dry-run # Dry run production deployment
```

**Features:**
- Environment-specific configuration
- Pre-deployment validation
- Health checks after deployment
- Rollback capabilities
- Deployment reporting

## Configuration

### Environment Configuration

Each environment has its own configuration file in `deployments/config/`:

- `development.env` - Development environment settings
- `staging.env` - Staging environment settings  
- `production.env` - Production environment settings

### Docker Compose Files

- `docker-compose.yml` - Base configuration
- `docker-compose.development.yml` - Development overrides
- `docker-compose.staging.yml` - Staging overrides
- `docker-compose.production.yml` - Production overrides
- `docker-compose.test.yml` - Test environment configuration

## CI/CD Integration

### GitHub Actions Workflows

The scripts integrate with GitHub Actions workflows:

- `.github/workflows/ci.yml` - Continuous Integration
- `.github/workflows/cd.yml` - Continuous Deployment

### Workflow Triggers

**CI Workflow:**
- Push to `main` or `develop` branches
- Pull requests to `main` or `develop`

**CD Workflow:**
- Push to `main` branch (deploys to staging)
- Git tags matching `v*` (deploys to production)
- Manual workflow dispatch

## Build Artifacts

### Directory Structure

```
build/
├── artifacts/
│   ├── frontend-{env}-{timestamp}.tar.gz
│   ├── backend-{env}-{timestamp}
│   ├── backend-config-{env}-{timestamp}/
│   └── contracts/
│       └── {contract}-{env}-{timestamp}.wasm
├── logs/
│   └── {component}-{timestamp}.log
└── build-manifest-{timestamp}.json
```

### Artifact Contents

- **Frontend:** Optimized static assets ready for CDN deployment
- **Backend:** Compiled binary with configuration files
- **Contracts:** Optimized WASM files ready for Soroban deployment

## Test Results

### Directory Structure

```
test-results/
├── reports/
│   ├── frontend-unit-{timestamp}.json
│   ├── backend-unit-{timestamp}.json
│   ├── api-tests-{timestamp}.json
│   └── e2e-{timestamp}/
├── coverage/
│   ├── frontend/
│   └── backend/
├── logs/
│   └── {component}-{timestamp}.log
└── test-report-{timestamp}.json
```

## Deployment Reports

### Directory Structure

```
deployments/
├── config/
│   ├── development.env
│   ├── staging.env
│   └── production.env
├── logs/
│   └── deployment-{env}-{timestamp}.log
├── contract-addresses-{env}-{timestamp}.json
└── deployment-report-{env}-{timestamp}.json
```

## Security Considerations

### Secrets Management

- Environment variables for sensitive data
- SSH keys for deployment access
- Database credentials
- API keys and tokens

### Access Control

- Role-based deployment permissions
- Environment-specific access controls
- Audit logging for all deployments

## Monitoring and Alerting

### Health Checks

- Service availability monitoring
- Database connectivity checks
- Contract deployment verification

### Notifications

- Slack integration for deployment status
- Email alerts for failures
- GitHub status checks

## Troubleshooting

### Common Issues

1. **Build Failures:**
   - Check dependency versions
   - Verify environment configuration
   - Review build logs

2. **Test Failures:**
   - Ensure test infrastructure is running
   - Check database connectivity
   - Verify contract deployments

3. **Deployment Issues:**
   - Validate deployment target connectivity
   - Check environment configuration
   - Review deployment logs

### Debug Mode

Enable debug logging by setting:
```bash
export DEBUG=true
```

### Log Locations

- Build logs: `build/logs/`
- Test logs: `test-results/logs/`
- Deployment logs: `deployments/logs/`

## Best Practices

### Development Workflow

1. Run tests locally before pushing
2. Use staging environment for integration testing
3. Perform dry-run deployments before production
4. Monitor deployment health checks

### Production Deployments

1. Always backup before deployment
2. Use blue-green deployment strategy
3. Monitor metrics during rollout
4. Have rollback plan ready

### Security

1. Rotate secrets regularly
2. Use least-privilege access
3. Audit deployment activities
4. Keep dependencies updated

## Contributing

When adding new scripts or modifying existing ones:

1. Follow the established patterns
2. Add comprehensive error handling
3. Include logging and progress indicators
4. Update documentation
5. Test across all environments