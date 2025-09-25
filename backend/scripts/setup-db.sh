#!/bin/bash

# Database setup script for Bitcoin Custody Backend
set -e

echo "Setting up Bitcoin Custody database..."

# Check if PostgreSQL is running
if ! pg_isready -h localhost -p 5432 > /dev/null 2>&1; then
    echo "Error: PostgreSQL is not running on localhost:5432"
    echo "Please start PostgreSQL or update the connection settings"
    exit 1
fi

# Load environment variables
if [ -f .env ]; then
    export $(cat .env | grep -v '^#' | xargs)
fi

# Set default database URL if not provided
DATABASE_URL=${DATABASE_URL:-"postgres://postgres:password@localhost:5432/bitcoin_custody_dev"}

echo "Using database: $DATABASE_URL"

# Create database if it doesn't exist
DB_NAME=$(echo $DATABASE_URL | sed 's/.*\///')
DB_HOST=$(echo $DATABASE_URL | sed 's/.*@\(.*\):.*/\1/')
DB_PORT=$(echo $DATABASE_URL | sed 's/.*:\([0-9]*\)\/.*/\1/')
DB_USER=$(echo $DATABASE_URL | sed 's/.*\/\/\(.*\):.*/\1/')

echo "Creating database '$DB_NAME' if it doesn't exist..."
createdb -h $DB_HOST -p $DB_PORT -U $DB_USER $DB_NAME 2>/dev/null || echo "Database already exists"

# Run migrations
echo "Running database migrations..."
cargo loco db migrate

# Seed database in development
if [ "${LOCO_ENV:-development}" = "development" ]; then
    echo "Seeding development database..."
    cargo run --bin bitcoin-custody-backend -- db seed
fi

echo "Database setup completed successfully!"
echo ""
echo "Database URL: $DATABASE_URL"
echo "Environment: ${LOCO_ENV:-development}"
echo ""
echo "You can now start the server with: cargo loco start"