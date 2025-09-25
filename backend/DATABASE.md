# Database Management Guide

This guide covers database setup, migrations, seeding, backup, and restore operations for the Bitcoin Custody Backend.

## Quick Start

### Prerequisites

- PostgreSQL 12+ installed and running
- Rust and Cargo installed
- Environment variables configured (see `.env.example`)

### Initial Setup

1. **Create environment file:**
   ```bash
   cp .env.example .env
   # Edit .env with your database credentials
   ```

2. **Run the setup script:**
   ```bash
   ./scripts/setup-db.sh
   ```

This script will:
- Create the database if it doesn't exist
- Run all migrations
- Seed the database with development data (in development mode)

## Database Schema

The database includes the following main tables:

### Core Tables

- **users** - User accounts with authentication and profile information
- **kyc_records** - KYC verification records with status and tier levels
- **token_balances** - User token balances for different token types
- **operations** - Transaction and operation records
- **audit_logs** - Comprehensive audit trail for all system activities

### Relationships

```
users (1) -> (many) kyc_records
users (1) -> (many) token_balances  
users (1) -> (many) operations
users (1) -> (many) audit_logs
```

## Migration Management

### Running Migrations

```bash
# Run all pending migrations
cargo loco db migrate

# Check migration status
cargo loco db status

# Reset database (drops and recreates)
cargo loco db reset
```

### Creating New Migrations

```bash
# Generate a new migration
cargo loco generate migration create_new_table

# The migration file will be created in migration/src/
```

## Database Seeding

### Development Seeding

The system automatically seeds development data when starting in development mode. This includes:

- Admin user (admin@bitcoincustody.dev / admin123)
- Test users with various KYC statuses
- Initial token balances
- Sample operations

### Manual Seeding

```bash
# Seed development data
cargo run -- db seed

# Force reseed (clears existing data first)
cargo run -- db seed --force

# Clear all seeded data
cargo run -- db clear
```

### Seeded Test Data

| User | Email | Password | Role | KYC Status | KYC Tier |
|------|-------|----------|------|------------|-----------|
| Admin | admin@bitcoincustody.dev | admin123 | admin | - | - |
| Alice | alice@example.com | password123 | user | approved | 2 |
| Bob | bob@example.com | password123 | user | approved | 1 |
| Charlie | charlie@example.com | password123 | user | pending | 0 |
| Diana | diana@example.com | password123 | compliance_officer | rejected | 0 |

## Backup and Restore

### Creating Backups

```bash
# Create a backup with default settings
./scripts/backup-db.sh

# Create backup with custom settings
cargo run -- db backup --path ./custom_backups --retention-days 60

# List available backups
cargo run -- db list-backups --path ./backups
```

### Restoring from Backup

```bash
# Restore from a specific backup file
cargo run -- db restore --file ./backups/bitcoin_custody_backup_20240101_120000.sql
```

### Automated Backups

In production, backups are configured to run automatically based on the schedule in the configuration:

- **Development**: Daily at 3 AM, 7-day retention
- **Production**: Daily at 2 AM, 30-day retention

## Database Status and Monitoring

### Check Database Status

```bash
# Show comprehensive database status
cargo run -- db status
```

This displays:
- Seeding status
- User count
- KYC record statistics
- Token balance summaries
- Operation statistics

### Audit Log Management

Audit logs are automatically created for:
- User authentication (login/logout)
- Entity changes (create/update/delete)
- KYC operations
- Token transfers
- System access

```bash
# Clean up old audit logs (based on retention policy)
# This is handled automatically, but can be run manually:
cargo run -- db cleanup-audit --days 365
```

## Environment-Specific Configuration

### Development
- Auto-migration enabled
- Auto-seeding enabled
- Debug logging
- 7-day backup retention

### Production
- Auto-migration disabled (run manually)
- No auto-seeding
- JSON logging
- 30-day backup retention
- Enhanced security settings

### Testing
- Auto-migration enabled
- No backups
- Minimal logging
- 1-day audit retention

## Troubleshooting

### Common Issues

1. **Connection refused**
   ```bash
   # Check if PostgreSQL is running
   pg_isready -h localhost -p 5432
   
   # Start PostgreSQL (macOS with Homebrew)
   brew services start postgresql
   ```

2. **Permission denied**
   ```bash
   # Ensure database user has proper permissions
   psql -c "ALTER USER postgres CREATEDB;"
   ```

3. **Migration failures**
   ```bash
   # Check migration status
   cargo loco db status
   
   # Reset and retry
   cargo loco db reset
   ./scripts/setup-db.sh
   ```

### Database Recovery

If you need to completely reset the database:

```bash
# 1. Drop the database
dropdb bitcoin_custody_dev

# 2. Run setup script
./scripts/setup-db.sh
```

## Performance Considerations

### Indexes

The schema includes optimized indexes for:
- User email lookups
- KYC status queries
- Token balance queries by user and token type
- Operation status and type filtering
- Audit log queries by entity and date

### Connection Pooling

Connection pool settings by environment:
- **Development**: 1-10 connections
- **Production**: 5-20 connections
- **Testing**: 1-5 connections

## Security

### Sensitive Data

- Passwords are hashed using bcrypt
- JWT secrets must be set in production
- Database credentials should use environment variables
- Audit logs capture all sensitive operations

### Backup Security

- Backups include all data and should be stored securely
- Consider encrypting backup files in production
- Implement proper access controls for backup directories

## Monitoring

### Health Checks

The application provides database health check endpoints:
- `/api/health` - Basic health check
- `/api/system/status` - Detailed system status including database

### Metrics

Key metrics to monitor:
- Connection pool utilization
- Query performance
- Backup success/failure
- Audit log growth
- Migration status