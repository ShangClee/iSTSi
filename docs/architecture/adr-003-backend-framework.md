# ADR-003: Backend Framework Selection

## Status

Accepted

## Date

2024-01-17

## Context

The backend needs to provide a robust, secure API server for the Bitcoin custody application. The requirements include:

- RESTful API endpoints for frontend communication
- Database operations with PostgreSQL
- Integration with Soroban smart contracts
- JWT-based authentication and authorization
- Background job processing for blockchain monitoring
- WebSocket support for real-time updates
- High performance and security for financial operations

We need to choose a backend framework that can handle these requirements while providing good developer experience and maintainability.

## Decision

We will use **Loco.rs** as our backend framework with the following supporting technologies:

### Core Framework
- **Loco.rs** - Modern Rust web framework inspired by Rails
- **Axum** - Web server (used internally by Loco.rs)
- **Sea-ORM** - Database ORM with PostgreSQL
- **Tokio** - Async runtime

### Additional Libraries
- **Serde** - Serialization/deserialization
- **jsonwebtoken** - JWT authentication
- **bcrypt** - Password hashing
- **soroban-sdk** - Soroban smart contract integration
- **reqwest** - HTTP client for external APIs
- **tokio-tungstenite** - WebSocket support

## Rationale

### Loco.rs Framework
- **Modern Rust**: Memory safety and performance benefits of Rust
- **Rails-inspired**: Familiar patterns for rapid development
- **Built-in Features**: Authentication, migrations, background jobs out of the box
- **Type Safety**: Compile-time guarantees prevent many runtime errors
- **Performance**: Excellent performance characteristics for financial applications
- **Security**: Rust's memory safety helps prevent common security vulnerabilities

### Sea-ORM
- **Type Safety**: Compile-time checked database queries
- **Migration System**: Robust database schema management
- **Async Support**: Non-blocking database operations
- **PostgreSQL Integration**: Excellent PostgreSQL support with advanced features
- **Code Generation**: Automatic entity generation from database schema

### Soroban Integration
- **Native Support**: Direct integration with Soroban smart contracts
- **Type Safety**: Rust types for contract interactions
- **Performance**: Efficient contract calls and event monitoring
- **Ecosystem**: Growing Stellar/Soroban ecosystem in Rust

## Implementation Details

### Project Structure
```
backend/
├── src/
│   ├── controllers/         # HTTP request handlers
│   │   ├── auth.rs
│   │   ├── deposits.rs
│   │   ├── users.rs
│   │   └── mod.rs
│   ├── models/             # Database models
│   │   ├── user.rs
│   │   ├── deposit.rs
│   │   └── mod.rs
│   ├── services/           # Business logic
│   │   ├── deposit_service.rs
│   │   ├── soroban_client.rs
│   │   └── mod.rs
│   ├── workers/            # Background jobs
│   │   ├── blockchain_monitor.rs
│   │   └── mod.rs
│   ├── middleware/         # Custom middleware
│   │   ├── auth.rs
│   │   └── mod.rs
│   ├── app.rs             # Application setup
│   └── main.rs            # Entry point
├── migration/             # Database migrations
├── config/               # Configuration files
├── tests/                # Integration tests
└── Cargo.toml           # Dependencies
```

### Configuration Example
```yaml
# config/development.yaml
server:
  port: 8080
  host: 0.0.0.0

database:
  uri: postgres://postgres:password@localhost:5432/bitcoin_custody_dev
  auto_migrate: true

auth:
  jwt:
    secret: "development-secret"
    expiration: 86400

soroban:
  network: "testnet"
  rpc_url: "https://soroban-testnet.stellar.org"
```

### Controller Example
```rust
use loco_rs::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct CreateDepositRequest {
    pub amount: u64,
    pub bitcoin_address: String,
}

#[derive(Serialize)]
pub struct DepositResponse {
    pub id: String,
    pub status: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

pub async fn create_deposit(
    State(ctx): State<AppContext>,
    Json(req): Json<CreateDepositRequest>,
) -> Result<Json<DepositResponse>> {
    let deposit = ctx.deposit_service
        .create_deposit(req)
        .await?;
    
    Ok(Json(DepositResponse {
        id: deposit.id,
        status: deposit.status.to_string(),
        created_at: deposit.created_at,
    }))
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("api/deposits")
        .add("/", post(create_deposit))
}
```

### Service Layer Example
```rust
use sea_orm::DatabaseConnection;
use crate::models::deposit::{Deposit, CreateDeposit};

pub struct DepositService {
    db: DatabaseConnection,
    soroban_client: SorobanClient,
}

impl DepositService {
    pub async fn create_deposit(
        &self,
        request: CreateDepositRequest,
    ) -> Result<Deposit, ServiceError> {
        // Validate business rules
        self.validate_deposit(&request).await?;
        
        // Create database record
        let deposit = Deposit::create(&self.db, CreateDeposit {
            amount: request.amount,
            bitcoin_address: request.bitcoin_address,
            status: DepositStatus::Pending,
        }).await?;
        
        // Trigger blockchain operation
        self.soroban_client
            .execute_deposit(&deposit)
            .await?;
        
        Ok(deposit)
    }
}
```

## Consequences

### Positive
- **Performance**: Rust provides excellent performance for financial applications
- **Safety**: Memory safety and type safety prevent many classes of bugs
- **Concurrency**: Tokio async runtime handles high concurrency efficiently
- **Developer Experience**: Loco.rs provides Rails-like productivity in Rust
- **Ecosystem**: Growing Rust web ecosystem with excellent libraries
- **Soroban Integration**: Native Rust integration with Stellar smart contracts

### Negative
- **Learning Curve**: Rust has a steeper learning curve than other languages
- **Compile Times**: Rust compilation can be slower during development
- **Ecosystem Maturity**: Some libraries may be less mature than alternatives
- **Team Expertise**: Team needs to develop Rust expertise

### Neutral
- **Community**: Smaller but growing Rust web development community
- **Tooling**: Good tooling but not as extensive as more established ecosystems

## Alternatives Considered

### Alternative 1: Node.js + Express/Fastify
- **Pros**: Team familiarity, large ecosystem, rapid development
- **Cons**: Runtime errors, performance limitations, security concerns
- **Rejected**: Performance and safety requirements favor compiled language

### Alternative 2: Python + FastAPI
- **Pros**: Rapid development, excellent API documentation, large ecosystem
- **Cons**: Performance limitations, GIL constraints, runtime errors
- **Rejected**: Performance requirements and type safety needs

### Alternative 3: Go + Gin/Echo
- **Pros**: Good performance, simple deployment, growing ecosystem
- **Cons**: Less sophisticated type system, no built-in ORM patterns
- **Rejected**: Rust provides better safety guarantees and Soroban integration

### Alternative 4: Rust + Actix Web
- **Pros**: High performance, mature Rust web framework
- **Cons**: More boilerplate, less Rails-like productivity features
- **Rejected**: Loco.rs provides better developer experience with similar performance

## Database Design

### Migration System
```rust
// migration/m20240101_000001_create_users.rs
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Users::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Users::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Users::Email).string().not_null().unique_key())
                    .col(ColumnDef::new(Users::PasswordHash).string().not_null())
                    .col(ColumnDef::new(Users::Role).string().not_null())
                    .col(ColumnDef::new(Users::CreatedAt).timestamp().not_null())
                    .col(ColumnDef::new(Users::UpdatedAt).timestamp().not_null())
                    .to_owned(),
            )
            .await
    }
}
```

### Model Definition
```rust
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub email: String,
    pub password_hash: String,
    pub role: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::deposit::Entity")]
    Deposits,
}

impl Related<super::deposit::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Deposits.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
```

## Security Considerations

### Authentication
```rust
use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,  // User ID
    pub role: String, // User role
    pub exp: usize,   // Expiration time
}

pub fn generate_jwt(user_id: &str, role: &str) -> Result<String, JwtError> {
    let expiration = chrono::Utc::now()
        .checked_add_signed(chrono::Duration::hours(24))
        .expect("valid timestamp")
        .timestamp() as usize;
    
    let claims = Claims {
        sub: user_id.to_string(),
        role: role.to_string(),
        exp: expiration,
    };
    
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(JWT_SECRET.as_ref()),
    )
}
```

### Input Validation
```rust
use validator::{Validate, ValidationError};

#[derive(Debug, Deserialize, Validate)]
pub struct CreateDepositRequest {
    #[validate(range(min = 1, max = 1000000))]
    pub amount: u64,
    
    #[validate(custom = "validate_bitcoin_address")]
    pub bitcoin_address: String,
}

fn validate_bitcoin_address(address: &str) -> Result<(), ValidationError> {
    if !is_valid_bitcoin_address(address) {
        return Err(ValidationError::new("invalid_bitcoin_address"));
    }
    Ok(())
}
```

## Performance Optimization

### Connection Pooling
```rust
use sea_orm::{Database, ConnectOptions};

pub async fn create_database_connection() -> Result<DatabaseConnection, DbErr> {
    let mut opt = ConnectOptions::new(DATABASE_URL);
    opt.max_connections(100)
        .min_connections(5)
        .connect_timeout(Duration::from_secs(8))
        .idle_timeout(Duration::from_secs(8))
        .max_lifetime(Duration::from_secs(8))
        .sqlx_logging(true);
    
    Database::connect(opt).await
}
```

### Caching Strategy
```rust
use redis::{Client, Commands};

pub struct CacheService {
    client: Client,
}

impl CacheService {
    pub async fn get<T>(&self, key: &str) -> Result<Option<T>, CacheError>
    where
        T: serde::de::DeserializeOwned,
    {
        let mut conn = self.client.get_connection()?;
        let value: Option<String> = conn.get(key)?;
        
        match value {
            Some(v) => Ok(Some(serde_json::from_str(&v)?)),
            None => Ok(None),
        }
    }
}
```

## Related Decisions

- ADR-001: Project Structure Reorganization
- ADR-005: Database Design and ORM
- ADR-006: Authentication and Authorization
- ADR-007: API Design and Communication

## Migration Strategy

1. **Framework Setup**: Initialize Loco.rs project with basic configuration
2. **Database Migration**: Set up PostgreSQL with Sea-ORM migrations
3. **API Development**: Implement core API endpoints
4. **Authentication**: Add JWT-based authentication system
5. **Soroban Integration**: Implement smart contract client
6. **Background Jobs**: Set up blockchain monitoring workers
7. **Testing**: Implement comprehensive test suite

## Success Metrics

- **Performance**: API response times < 100ms for 95th percentile
- **Reliability**: 99.9% uptime with proper error handling
- **Security**: Zero security vulnerabilities in production
- **Developer Productivity**: Reduced time for new feature development