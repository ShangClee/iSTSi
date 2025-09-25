# Bitcoin Custody Backend

A robust Rust backend built with Loco.rs framework, providing secure APIs for Bitcoin custody operations, KYC compliance, and Soroban smart contract integration.

## 🏗️ Architecture

This backend is built with:
- **Loco.rs** - Modern Rust web framework with Rails-like conventions
- **Sea-ORM** - Async ORM for PostgreSQL database operations
- **PostgreSQL** - Primary database for user data and operations
- **Soroban SDK** - Integration with Stellar smart contracts
- **JWT Authentication** - Secure token-based authentication
- **WebSocket Support** - Real-time updates for frontend clients

## 📁 Project Structure

```
backend/
├── src/
│   ├── controllers/         # API endpoint handlers
│   │   ├── auth.rs         # Authentication endpoints
│   │   ├── users.rs        # User management
│   │   ├── integration.rs  # Integration operations
│   │   ├── kyc.rs          # KYC and compliance
│   │   ├── tokens.rs       # Token operations
│   │   ├── reserves.rs     # Reserve management
│   │   ├── system.rs       # System status and health
│   │   └── mod.rs
│   ├── models/             # Database models and entities
│   │   ├── _entities/      # Sea-ORM generated entities
│   │   ├── user.rs         # User model and business logic
│   │   ├── kyc_record.rs   # KYC record model
│   │   ├── token_balance.rs # Token balance model
│   │   ├── operation.rs    # Operation history model
│   │   ├── audit_log.rs    # Audit logging model
│   │   └── mod.rs
│   ├── services/           # Business logic services
│   │   ├── integration_service.rs  # Core integration logic
│   │   ├── kyc_service.rs         # KYC compliance service
│   │   ├── token_service.rs       # Token management service
│   │   ├── reserve_service.rs     # Reserve management service
│   │   ├── soroban_client.rs      # Soroban contract client
│   │   ├── event_monitor_service.rs # Blockchain event monitoring
│   │   └── mod.rs
│   ├── workers/            # Background job workers
│   │   ├── reconciliation.rs      # Balance reconciliation
│   │   ├── proof_generation.rs    # Proof of reserves generation
│   │   ├── event_monitor.rs       # Blockchain event monitoring
│   │   └── mod.rs
│   ├── middleware/         # Custom middleware
│   │   ├── auth.rs         # JWT authentication middleware
│   │   ├── cors.rs         # CORS configuration
│   │   └── mod.rs
│   ├── utils/              # Utility functions
│   │   ├── database.rs     # Database utilities
│   │   └── mod.rs
│   ├── commands/           # CLI commands
│   │   ├── database.rs     # Database management commands
│   │   └── mod.rs
│   ├── app.rs              # Application configuration
│   ├── lib.rs              # Library exports
│   └── main.rs             # Application entry point
├── migration/              # Database migrations
│   ├── src/
│   │   ├── m20240101_000001_create_users.rs
│   │   ├── m20240101_000002_create_kyc_records.rs
│   │   ├── m20240101_000003_create_token_balances.rs
│   │   ├── m20240101_000004_create_operations.rs
│   │   ├── m20240101_000005_create_audit_logs.rs
│   │   └── lib.rs
│   └── Cargo.toml
├── config/                 # Configuration files
│   ├── development.yaml    # Development environment config
│   ├── production.yaml     # Production environment config
│   └── test.yaml          # Test environment config
├── tests/                  # Integration tests
│   ├── integration/        # Integration test modules
│   └── soroban_integration.rs
├── scripts/                # Utility scripts
│   ├── setup-db.sh        # Database setup script
│   └── backup-db.sh       # Database backup script
├── Cargo.toml              # Rust dependencies and metadata
├── .env.example            # Environment variables template
└── Dockerfile.dev          # Development Docker configuration
```

## 🚀 Quick Start

### Prerequisites

- Rust 1.70+ with Cargo
- PostgreSQL 14+
- Soroban CLI (for contract interactions)
- Docker and Docker Compose (optional, for containerized development)

### Installation

1. **Navigate to backend directory:**
   ```bash
   cd backend
   ```

2. **Install Rust dependencies:**
   ```bash
   cargo build
   ```

3. **Set up environment variables:**
   ```bash
   cp .env.example .env
   ```
   
   Edit `.env` with your configuration:
   ```env
   DATABASE_URL=postgres://postgres:password@localhost:5432/bitcoin_custody_dev
   JWT_SECRET=your-super-secret-jwt-key-change-in-production
   SOROBAN_NETWORK=testnet
   SOROBAN_RPC_URL=https://soroban-testnet.stellar.org
   RUST_LOG=debug
   ```

4. **Set up PostgreSQL database:**
   ```bash
   # Using provided script
   ./scripts/setup-db.sh
   
   # Or manually
   createdb bitcoin_custody_dev
   ```

5. **Run database migrations:**
   ```bash
   cargo loco db migrate
   ```

6. **Start the development server:**
   ```bash
   cargo loco start
   ```

The API will be available at `http://localhost:8080`

## 🛠️ Development

### Available Commands

```bash
# Development server with hot reloading
cargo loco start

# Run database migrations
cargo loco db migrate

# Reset database (drop and recreate)
cargo loco db reset

# Generate new migration
cargo loco generate migration create_new_table

# Run tests
cargo test

# Run with specific log level
RUST_LOG=debug cargo loco start

# Build for production
cargo build --release
```

### Development Workflow

1. **Database Changes:**
   ```bash
   # Create new migration
   cargo loco generate migration add_new_column_to_users
   
   # Edit the generated migration file
   # Run migration
   cargo loco db migrate
   ```

2. **Adding New Endpoints:**
   - Create controller in `src/controllers/`
   - Add routes in `src/app.rs`
   - Implement business logic in `src/services/`
   - Add tests in `tests/`

3. **Model Development:**
   - Define models in `src/models/`
   - Create corresponding migrations
   - Implement business logic methods

4. **Service Integration:**
   - Add service logic in `src/services/`
   - Integrate with Soroban contracts via `soroban_client.rs`
   - Handle errors appropriately

### Hot Reloading

The development server supports hot reloading with `cargo-watch`:

```bash
# Install cargo-watch
cargo install cargo-watch

# Start with hot reloading
cargo watch -x "loco start"
```

## 🗄️ Database Schema

### Core Tables

**Users Table:**
```sql
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email VARCHAR UNIQUE NOT NULL,
    password_hash VARCHAR NOT NULL,
    stellar_address VARCHAR,
    role VARCHAR NOT NULL DEFAULT 'user',
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);
```

**KYC Records Table:**
```sql
CREATE TABLE kyc_records (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id),
    tier INTEGER NOT NULL,
    status VARCHAR NOT NULL,
    verification_data JSONB,
    approved_at TIMESTAMPTZ,
    expires_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);
```

**Token Balances Table:**
```sql
CREATE TABLE token_balances (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id),
    token_address VARCHAR NOT NULL,
    balance BIGINT NOT NULL DEFAULT 0,
    reserved_balance BIGINT NOT NULL DEFAULT 0,
    last_updated TIMESTAMPTZ DEFAULT NOW(),
    created_at TIMESTAMPTZ DEFAULT NOW()
);
```

**Operations Table:**
```sql
CREATE TABLE operations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id),
    operation_type VARCHAR NOT NULL,
    status VARCHAR NOT NULL,
    amount BIGINT,
    token_address VARCHAR,
    btc_tx_hash VARCHAR,
    stellar_tx_hash VARCHAR,
    metadata JSONB,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    completed_at TIMESTAMPTZ
);
```

**Audit Logs Table:**
```sql
CREATE TABLE audit_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id),
    action VARCHAR NOT NULL,
    resource_type VARCHAR NOT NULL,
    resource_id VARCHAR,
    old_values JSONB,
    new_values JSONB,
    ip_address INET,
    user_agent TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW()
);
```

### Database Management

**Running Migrations:**
```bash
# Apply all pending migrations
cargo loco db migrate

# Check migration status
cargo loco db status

# Rollback last migration
cargo loco db rollback

# Reset database (development only)
cargo loco db reset
```

**Creating Migrations:**
```bash
# Generate new migration
cargo loco generate migration create_new_table

# Example migration content
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(NewTable::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(NewTable::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(NewTable::Name).string().not_null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(NewTable::Table).to_owned())
            .await
    }
}
```

## 🔌 API Endpoints

### Authentication Endpoints

**POST /api/auth/register**
```json
{
  "email": "user@example.com",
  "password": "secure_password",
  "stellar_address": "GXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX"
}
```

**POST /api/auth/login**
```json
{
  "email": "user@example.com",
  "password": "secure_password"
}
```

**POST /api/auth/logout**
- Requires: `Authorization: Bearer <token>`

### User Management Endpoints

**GET /api/users/profile**
- Requires: Authentication
- Returns: User profile information

**PUT /api/users/profile**
- Requires: Authentication
- Updates user profile

### Integration Endpoints

**POST /api/integration/bitcoin-deposit**
```json
{
  "btc_amount": 100000000,
  "btc_tx_hash": "abc123...",
  "stellar_address": "GXXXXXXX..."
}
```

**POST /api/integration/token-withdrawal**
```json
{
  "token_amount": 100000000,
  "btc_address": "bc1qxxxxxxx...",
  "token_address": "CXXXXXXX..."
}
```

**GET /api/integration/operations**
- Returns: List of user operations with pagination

### KYC Endpoints

**POST /api/kyc/submit**
```json
{
  "tier": 2,
  "verification_data": {
    "document_type": "passport",
    "document_number": "123456789",
    "full_name": "John Doe"
  }
}
```

**GET /api/kyc/status**
- Returns: Current KYC status and tier information

### Token Management Endpoints

**GET /api/tokens/balances**
- Returns: User token balances

**GET /api/tokens/history**
- Returns: Token transaction history

### Reserve Management Endpoints

**GET /api/reserves/status**
- Returns: Current reserve ratios and status

**GET /api/reserves/proof**
- Returns: Latest proof of reserves data

### System Endpoints

**GET /api/system/health**
- Returns: System health status

**GET /api/system/overview**
- Returns: System overview data for dashboard

## 🔐 Authentication & Security

### JWT Authentication

The backend uses JWT tokens for authentication:

```rust
// Example middleware usage
use crate::middleware::auth::AuthMiddleware;

// Protected route
#[tokio::main]
async fn protected_endpoint(
    auth: AuthMiddleware,
    State(app_state): State<AppState>,
) -> Result<Json<UserProfile>, AppError> {
    let user = auth.user;
    // Handle authenticated request
}
```

### Security Features

- **Password Hashing:** bcrypt with configurable rounds
- **JWT Tokens:** Secure token generation and validation
- **CORS Configuration:** Configurable cross-origin resource sharing
- **Rate Limiting:** Built-in rate limiting for API endpoints
- **Input Validation:** Comprehensive request validation
- **SQL Injection Protection:** Parameterized queries via Sea-ORM

### Environment Security

**Development (.env):**
```env
JWT_SECRET=development-secret-change-in-production
DATABASE_URL=postgres://postgres:password@localhost:5432/bitcoin_custody_dev
RUST_LOG=debug
```

**Production:**
- Use strong, randomly generated JWT secrets
- Enable TLS/SSL for database connections
- Use environment-specific configuration files
- Implement proper secret management

## 🔗 Soroban Integration

### Contract Client Usage

```rust
use crate::services::soroban_client::SorobanClient;

// Initialize client
let soroban_client = SorobanClient::new(
    &config.soroban.network,
    &config.soroban.rpc_url,
    &config.soroban.integration_router_address,
)?;

// Execute Bitcoin deposit
let result = soroban_client
    .execute_bitcoin_deposit(
        user_address,
        btc_amount,
        btc_tx_hash,
    )
    .await?;
```

### Event Monitoring

The backend monitors Soroban contract events:

```rust
// Event monitoring service
use crate::services::event_monitor_service::EventMonitorService;

let event_monitor = EventMonitorService::new(soroban_client);
event_monitor.start_monitoring().await?;
```

### Contract Addresses Configuration

Contract addresses are managed in configuration files:

```yaml
# config/development.yaml
soroban:
  network: "testnet"
  rpc_url: "https://soroban-testnet.stellar.org"
  network_passphrase: "Test SDF Network ; September 2015"
  contracts:
    integration_router: "CXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX"
    kyc_registry: "CXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX"
    istsi_token: "CXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX"
    reserve_manager: "CXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX"
```

## 🏗️ Building for Production

### Production Build

```bash
# Build optimized binary
cargo build --release

# The binary will be available at target/release/bitcoin-custody-backend
```

### Docker Deployment

**Dockerfile:**
```dockerfile
FROM rust:1.70 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/bitcoin-custody-backend /usr/local/bin/
EXPOSE 8080
CMD ["bitcoin-custody-backend"]
```

**Build and run:**
```bash
# Build Docker image
docker build -t bitcoin-custody-backend .

# Run container
docker run -p 8080:8080 \
  -e DATABASE_URL=postgres://user:pass@host:5432/db \
  -e JWT_SECRET=your-secret \
  bitcoin-custody-backend
```

### Production Configuration

**config/production.yaml:**
```yaml
server:
  port: 8080
  host: 0.0.0.0

database:
  uri: ${DATABASE_URL}
  auto_migrate: false
  connect_timeout: 30000
  idle_timeout: 600000
  max_connections: 100

auth:
  jwt:
    secret: ${JWT_SECRET}
    expiration: 86400

soroban:
  network: "mainnet"
  rpc_url: "https://soroban-mainnet.stellar.org"
  network_passphrase: "Public Global Stellar Network ; September 2015"

logging:
  level: "info"
  format: "json"

cors:
  allow_origins: ["https://yourdomain.com"]
  allow_methods: ["GET", "POST", "PUT", "DELETE"]
  allow_headers: ["Content-Type", "Authorization"]
```

## 🧪 Testing

### Running Tests

```bash
# Run all tests
cargo test

# Run specific test module
cargo test integration

# Run tests with output
cargo test -- --nocapture

# Run tests with specific log level
RUST_LOG=debug cargo test
```

### Test Structure

**Unit Tests:**
```rust
// src/services/integration_service.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_bitcoin_deposit_validation() {
        // Test implementation
    }
}
```

**Integration Tests:**
```rust
// tests/integration/auth_test.rs
use bitcoin_custody_backend::app::App;

#[tokio::test]
async fn test_user_registration() {
    let app = App::new().await;
    // Test API endpoints
}
```

### Database Testing

```rust
// Use test database for integration tests
#[tokio::test]
async fn test_with_database() {
    let db = setup_test_db().await;
    // Run test with database
    cleanup_test_db(db).await;
}
```

## 🔧 Troubleshooting

### Common Issues

**1. Database Connection Issues**
```bash
# Check PostgreSQL is running
pg_isready -h localhost -p 5432

# Check database exists
psql -h localhost -U postgres -l

# Test connection string
psql "postgres://postgres:password@localhost:5432/bitcoin_custody_dev"
```

**2. Migration Failures**
```bash
# Check migration status
cargo loco db status

# Reset database (development only)
cargo loco db reset

# Run migrations manually
cargo loco db migrate
```

**3. Soroban Integration Issues**
```bash
# Check Soroban CLI installation
soroban --version

# Test network connectivity
curl -X POST https://soroban-testnet.stellar.org \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"getHealth"}'
```

**4. Build Issues**
```bash
# Clean build cache
cargo clean

# Update dependencies
cargo update

# Check for compilation errors
cargo check
```

**5. Runtime Issues**
```bash
# Check logs
RUST_LOG=debug cargo loco start

# Check system resources
htop
df -h

# Check network connectivity
netstat -tlnp | grep 8080
```

### Performance Optimization

**Database Performance:**
- Add appropriate indexes for frequently queried columns
- Use connection pooling (configured in Sea-ORM)
- Monitor slow queries with PostgreSQL logs
- Implement database query optimization

**Memory Usage:**
- Monitor memory usage with `htop` or similar tools
- Adjust database connection pool size
- Use `cargo build --release` for production builds

**Logging Configuration:**
```rust
// Adjust log levels for production
RUST_LOG=info,bitcoin_custody_backend=debug
```

## 📚 Additional Resources

- [Loco.rs Documentation](https://loco.rs/)
- [Sea-ORM Documentation](https://www.sea-ql.org/SeaORM/)
- [Soroban Documentation](https://soroban.stellar.org/docs)
- [PostgreSQL Documentation](https://www.postgresql.org/docs/)
- [Rust Book](https://doc.rust-lang.org/book/)
- [Tokio Documentation](https://tokio.rs/)

## 🤝 Contributing

1. Follow Rust coding conventions and use `rustfmt`
2. Add comprehensive tests for new functionality
3. Update documentation for API changes
4. Use proper error handling with custom error types
5. Follow the established project structure and patterns

For questions or issues, refer to the main project documentation or create an issue in the project repository.