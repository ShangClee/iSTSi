#!/bin/bash

# Database backup script for Bitcoin Custody Backend
set -e

# Load environment variables
if [ -f .env ]; then
    export $(cat .env | grep -v '^#' | xargs)
fi

# Set default values
DATABASE_URL=${DATABASE_URL:-"postgres://postgres:password@localhost:5432/bitcoin_custody_dev"}
BACKUP_DIR=${BACKUP_DIR:-"./backups"}
RETENTION_DAYS=${RETENTION_DAYS:-30}

echo "Creating database backup..."
echo "Database: $DATABASE_URL"
echo "Backup directory: $BACKUP_DIR"
echo "Retention: $RETENTION_DAYS days"

# Create backup directory if it doesn't exist
mkdir -p $BACKUP_DIR

# Create backup using the Rust application
cargo run --bin bitcoin-custody-backend -- db backup --path $BACKUP_DIR --retention-days $RETENTION_DAYS

echo "Backup completed successfully!"
echo ""
echo "To list available backups:"
echo "cargo run --bin bitcoin-custody-backend -- db list-backups --path $BACKUP_DIR"
echo ""
echo "To restore from backup:"
echo "cargo run --bin bitcoin-custody-backend -- db restore --file <backup-file>"