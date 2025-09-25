# Bitcoin Custody Full-Stack Troubleshooting Guide

This comprehensive guide covers common issues, debugging techniques, and development workflows for the Bitcoin Custody full-stack application.

## 🚨 Quick Diagnosis

### System Health Check

Run this quick health check to identify which component has issues:

```bash
# Check if all services are running
docker-compose ps

# Test frontend
curl -f http://localhost:3000 || echo "Frontend not responding"

# Test backend API
curl -f http://localhost:8080/api/system/health || echo "Backend not responding"

# Test database connection
pg_isready -h localhost -p 5432 || echo "Database not responding"

# Test Soroban network connectivity
curl -X POST https://soroban-testnet.stellar.org \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"getHealth"}' || echo "Soroban network not reachable"
```

### Component Status Overview

| Component | Port | Health Check | Log Location |
|-----------|------|--------------|--------------|
| Frontend | 3000 | `curl http://localhost:3000` | Browser console |
| Backend | 8080 | `curl http://localhost:8080/api/system/health` | `RUST_LOG=debug cargo loco start` |
| Database | 5432 | `pg_isready -h localhost -p 5432` | PostgreSQL logs |
| Soroban | N/A | Contract interactions | Soroban CLI output |

## 🖥️ Frontend Issues

### Development Server Issues

**Issue: Frontend won't start**
```bash
# Symptoms
npm run dev
# Error: Cannot find module or dependency issues

# Solutions
# 1. Clear npm cache and reinstall
npm cache clean --force
rm -rf node_modules package-lock.json
npm install

# 2. Check Node.js version
node --version  # Should be 18+
nvm use 18      # If using nvm

# 3. Check for port conflicts
lsof -i :3000   # Check what's using port 3000
kill -9 <PID>   # Kill conflicting process
```

**Issue: Hot reloading not working**
```bash
# Solutions
# 1. Check file watcher limits (Linux)
echo fs.inotify.max_user_watches=524288 | sudo tee -a /etc/sysctl.conf
sudo sysctl -p

# 2. Restart development server
npm run dev

# 3. Clear Vite cache
rm -rf node_modules/.vite
npm run dev
```

### API Connection Issues

**Issue: API calls failing with CORS errors**
```javascript
// Check browser console for errors like:
// "Access to fetch at 'http://localhost:8080/api/...' from origin 'http://localhost:3000' has been blocked by CORS policy"

// Solutions:
// 1. Verify backend CORS configuration in config/development.yaml
cors:
  allow_origins: ["http://localhost:3000"]
  allow_methods: ["GET", "POST", "PUT", "DELETE", "OPTIONS"]
  allow_headers: ["Content-Type", "Authorization"]

// 2. Check Vite proxy configuration in vite.config.ts
server: {
  proxy: {
    '/api': {
      target: 'http://localhost:8080',
      changeOrigin: true,
    },
  },
}

// 3. Verify backend is running and accessible
curl http://localhost:8080/api/system/health
```

**Issue: Authentication not working**
```javascript
// Check for JWT token issues
// 1. Verify token is stored correctly
console.log(localStorage.getItem('auth_token'));

// 2. Check token expiration
const token = localStorage.getItem('auth_token');
if (token) {
  const payload = JSON.parse(atob(token.split('.')[1]));
  console.log('Token expires:', new Date(payload.exp * 1000));
}

// 3. Clear stored tokens and re-authenticate
localStorage.removeItem('auth_token');
// Re-login through the application
```

### Build and Deployment Issues

**Issue: Production build fails**
```bash
# Check for TypeScript errors
npm run type-check

# Check for linting errors
npm run lint

# Build with verbose output
npm run build -- --verbose

# Common fixes:
# 1. Fix TypeScript errors
# 2. Update environment variables for production
# 3. Check for missing dependencies
```

**Issue: Deployed app shows blank page**
```bash
# Check browser console for errors
# Common causes:
# 1. Incorrect base URL in production
# 2. Missing environment variables
# 3. Asset loading issues

# Solutions:
# 1. Check vite.config.ts base configuration
export default defineConfig({
  base: '/your-app-path/', // If deployed in subdirectory
});

# 2. Verify environment variables are set
echo $VITE_API_URL

# 3. Check network tab for failed asset requests
```

## 🦀 Backend Issues

### Server Startup Issues

**Issue: Backend won't start**
```bash
# Check for common startup errors
RUST_LOG=debug cargo loco start

# Common errors and solutions:

# 1. Database connection failed
# Error: "Failed to connect to database"
# Check database is running
pg_isready -h localhost -p 5432
# Check connection string
echo $DATABASE_URL
# Test manual connection
psql $DATABASE_URL

# 2. Port already in use
# Error: "Address already in use (os error 98)"
lsof -i :8080
kill -9 <PID>

# 3. Migration errors
# Error: "Migration failed"
cargo loco db status
cargo loco db migrate
# If needed, reset database (development only)
cargo loco db reset
```

**Issue: Compilation errors**
```bash
# Check for Rust compilation issues
cargo check

# Common fixes:
# 1. Update dependencies
cargo update

# 2. Clear build cache
cargo clean
cargo build

# 3. Check Rust version
rustc --version  # Should be 1.70+
rustup update
```

### Database Issues

**Issue: Migration failures**
```bash
# Check migration status
cargo loco db status

# Common migration issues:
# 1. Conflicting migrations
cargo loco db rollback  # Rollback last migration
# Edit migration file
cargo loco db migrate

# 2. Database schema conflicts
# Development only - reset database
cargo loco db reset

# 3. Permission issues
# Check PostgreSQL user permissions
psql -U postgres -c "\du"
```

**Issue: Database performance problems**
```bash
# Check database connections
psql $DATABASE_URL -c "SELECT count(*) FROM pg_stat_activity;"

# Check slow queries (enable in postgresql.conf)
psql $DATABASE_URL -c "SELECT query, mean_exec_time FROM pg_stat_statements ORDER BY mean_exec_time DESC LIMIT 10;"

# Solutions:
# 1. Add database indexes
# 2. Optimize queries
# 3. Adjust connection pool settings in config
database:
  max_connections: 20
  connect_timeout: 30000
```

### Soroban Integration Issues

**Issue: Contract interaction failures**
```bash
# Check Soroban client configuration
# Verify contract addresses in config/development.yaml
soroban:
  contracts:
    integration_router: "CXXXXXXX..."  # Must be valid contract ID

# Test contract connectivity
soroban contract invoke \
  --id $CONTRACT_ID \
  --source user \
  --network testnet \
  --simulate-only \
  -- get_admin

# Common fixes:
# 1. Update contract addresses after redeployment
# 2. Check network configuration
# 3. Verify account has sufficient XLM for fees
```

### API Response Issues

**Issue: Slow API responses**
```bash
# Enable detailed logging
RUST_LOG=debug,sqlx=info cargo loco start

# Check for:
# 1. Slow database queries
# 2. Inefficient Soroban calls
# 3. Missing database indexes

# Profile with tools like:
cargo install cargo-profiler
cargo profiler flamegraph --bin bitcoin-custody-backend
```

## 🌟 Soroban Contract Issues

### Build and Deployment Issues

**Issue: Contract build failures**
```bash
# Check Rust and Soroban setup
rustc --version
soroban --version
rustup target list --installed | grep wasm32

# Common fixes:
# 1. Install WebAssembly target
rustup target add wasm32-unknown-unknown

# 2. Update Soroban CLI
cargo install --locked soroban-cli --force

# 3. Clear build cache
cargo clean
./scripts/build.sh
```

**Issue: Contract deployment failures**
```bash
# Check deployment prerequisites
soroban config identity ls
soroban config network ls

# Verify account funding
soroban config identity address deployer
# Check balance at https://laboratory.stellar.org/#account-creator

# Common deployment errors:
# 1. Insufficient XLM for fees
soroban config identity fund deployer --network testnet

# 2. Network connectivity issues
curl -X POST https://soroban-testnet.stellar.org \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"getHealth"}'

# 3. Contract size too large
ls -la target/wasm32-unknown-unknown/release/*.wasm
# Optimize if > 64KB
```

### Contract Interaction Issues

**Issue: Contract function calls failing**
```bash
# Debug contract calls
soroban contract invoke \
  --id $CONTRACT_ID \
  --source user \
  --network testnet \
  --simulate-only \
  -- function_name --param value

# Common issues:
# 1. Incorrect function parameters
# Check contract ABI documentation

# 2. Access control violations
# Verify caller has required permissions

# 3. Contract state issues
# Check contract storage state
soroban contract read --id $CONTRACT_ID --key storage_key --network testnet
```

**Issue: Events not being emitted**
```bash
# Check transaction details for events
soroban events --start-ledger 1000 --id $CONTRACT_ID --network testnet

# Verify events are properly defined in contract:
#[contractevent]
pub struct EventName {
    pub field: Type,
}

# Ensure events are emitted in contract functions:
env.events().publish((symbol_short!("event"), data));
```

### Testing Issues

**Issue: Contract tests failing**
```bash
# Run tests with verbose output
cargo test -- --nocapture

# Common test issues:
# 1. Environment setup problems
# Ensure test environment is properly initialized

# 2. Mock data issues
# Verify test data matches expected contract behavior

# 3. Assertion failures
# Check expected vs actual values in test output
```

## 🐳 Docker and Development Environment Issues

### Docker Compose Issues

**Issue: Services won't start**
```bash
# Check Docker Compose status
docker-compose ps

# View service logs
docker-compose logs frontend
docker-compose logs backend
docker-compose logs postgres

# Common fixes:
# 1. Rebuild containers
docker-compose down
docker-compose build --no-cache
docker-compose up -d

# 2. Check port conflicts
netstat -tlnp | grep -E ':(3000|8080|5432)'

# 3. Clear Docker cache
docker system prune -a
```

**Issue: Database container issues**
```bash
# Check PostgreSQL container
docker-compose logs postgres

# Common issues:
# 1. Data directory permissions
docker-compose down
docker volume rm $(docker volume ls -q)
docker-compose up -d postgres

# 2. Port conflicts
# Change port in docker-compose.yml if needed
ports:
  - "5433:5432"  # Use different host port
```

### Volume and Networking Issues

**Issue: File changes not reflected in containers**
```bash
# Check volume mounts
docker-compose config

# Restart services with fresh mounts
docker-compose down
docker-compose up -d

# For Windows/WSL users:
# Ensure files are in WSL filesystem, not Windows filesystem
```

## 🔧 Development Workflow Issues

### Git and Version Control

**Issue: Merge conflicts in generated files**
```bash
# Common conflicts in:
# - package-lock.json
# - Cargo.lock
# - Migration files

# Solutions:
# 1. For package-lock.json
rm package-lock.json
npm install

# 2. For Cargo.lock
rm Cargo.lock
cargo build

# 3. For migrations
# Coordinate with team to avoid conflicting migrations
```

### Environment Configuration

**Issue: Environment variables not loading**
```bash
# Check environment files exist
ls -la .env*

# Verify environment loading:
# Frontend: Check vite.config.ts and .env files
# Backend: Check config/*.yaml files
# Soroban: Check .env and soroban-project.toml

# Common fixes:
# 1. Restart services after env changes
# 2. Check variable naming (VITE_ prefix for frontend)
# 3. Verify file permissions
chmod 644 .env*
```

## 🔍 Debugging Techniques

### Frontend Debugging

```javascript
// 1. Redux DevTools
// Install Redux DevTools browser extension

// 2. React Developer Tools
// Install React Developer Tools browser extension

// 3. Network debugging
// Use browser Network tab to inspect API calls

// 4. Console debugging
console.log('Debug info:', data);
console.table(arrayData);
console.group('API Call');
console.log('Request:', request);
console.log('Response:', response);
console.groupEnd();
```

### Backend Debugging

```rust
// 1. Logging
use tracing::{info, debug, error};

debug!("Processing request: {:?}", request);
info!("User authenticated: {}", user.id);
error!("Database error: {:?}", error);

// 2. Environment logging
RUST_LOG=debug,sqlx=info cargo loco start

// 3. Database query debugging
RUST_LOG=sqlx::query=debug cargo loco start
```

### Contract Debugging

```rust
// 1. Contract logging (tests only)
use soroban_sdk::log;
log!(&env, "Debug: {}", value);

// 2. Simulation debugging
soroban contract invoke --simulate-only --id $CONTRACT_ID -- function_name

// 3. Event debugging
soroban events --start-ledger 1000 --id $CONTRACT_ID --network testnet
```

## 📊 Performance Monitoring

### Frontend Performance

```javascript
// 1. Bundle analysis
npm run build -- --analyze

// 2. Performance monitoring
// Use browser Performance tab
// Monitor Core Web Vitals

// 3. Memory leaks
// Use browser Memory tab
// Check for growing memory usage
```

### Backend Performance

```bash
# 1. CPU profiling
cargo install cargo-profiler
cargo profiler flamegraph --bin bitcoin-custody-backend

# 2. Memory monitoring
htop
ps aux | grep bitcoin-custody-backend

# 3. Database performance
# Enable slow query logging in PostgreSQL
# Monitor connection pool usage
```

### Contract Performance

```bash
# 1. Gas usage monitoring
# Check transaction fees in Stellar Laboratory

# 2. Contract size optimization
ls -la target/wasm32-unknown-unknown/release/*.wasm
# Optimize if contracts are too large

# 3. Call frequency monitoring
# Monitor contract interaction patterns
```

## 🆘 Emergency Procedures

### System Recovery

**Complete system reset (development only):**
```bash
# 1. Stop all services
docker-compose down

# 2. Clear all data
docker volume prune -f
rm -rf node_modules
cargo clean

# 3. Rebuild everything
./scripts/setup.sh
```

**Database recovery:**
```bash
# 1. Backup current state
./backend/scripts/backup-db.sh

# 2. Reset database
cargo loco db reset

# 3. Restore from backup if needed
psql $DATABASE_URL < backup.sql
```

**Contract redeployment:**
```bash
# 1. Redeploy contracts
./soroban/scripts/deploy-testnet.sh

# 2. Update contract addresses in backend config
# Edit backend/config/development.yaml

# 3. Restart backend
cargo loco start
```

## 📞 Getting Help

### Log Collection

When reporting issues, collect these logs:

```bash
# Frontend logs
# Browser console output
# Network tab showing failed requests

# Backend logs
RUST_LOG=debug cargo loco start 2>&1 | tee backend.log

# Database logs
# PostgreSQL logs (location varies by system)
# /var/log/postgresql/postgresql-*.log

# Docker logs
docker-compose logs > docker.log

# System information
uname -a
docker --version
node --version
cargo --version
soroban --version
```

### Common Support Channels

1. **Project Documentation** - Check component README files
2. **GitHub Issues** - Search existing issues and create new ones
3. **Community Forums** - Stellar Developer Discord, Rust forums
4. **Stack Overflow** - Tag questions appropriately

### Issue Reporting Template

```markdown
## Issue Description
Brief description of the problem

## Environment
- OS: [e.g., macOS 13.0, Ubuntu 22.04]
- Node.js: [version]
- Rust: [version]
- Soroban CLI: [version]
- Docker: [version]

## Steps to Reproduce
1. Step one
2. Step two
3. Step three

## Expected Behavior
What should happen

## Actual Behavior
What actually happens

## Logs
```
Paste relevant logs here
```

## Additional Context
Any other relevant information
```

This troubleshooting guide should help you quickly identify and resolve common issues across all components of the Bitcoin Custody full-stack application.