#!/bin/bash
set -e

# init-db.sh — Docker PostgreSQL container startup init script
# Auto-runs migrations and prod seeds if initial startup.

POSTGRES_USER="${POSTGRES_USER:-postgres}"
POSTGRES_DB="${POSTGRES_DB:-kepce}"

echo "=== Kepçe PostgreSQL Initialization ==="
echo "Database: $POSTGRES_DB | User: $POSTGRES_USER"

# Function to execute SQL file with psql
run_sql() {
    local file="$1"
    echo "Executing: $file"
    psql -v ON_ERROR_STOP=1 --username "$POSTGRES_USER" --dbname "$POSTGRES_DB" -f "$file"
}

# 1. Run migrations in order
if [ -d "/docker-entrypoint-initdb.d/migrations" ]; then
    echo "--- Running DB Migrations ---"
    for sql in $(ls /docker-entrypoint-initdb.d/migrations/*.sql | sort); do
        run_sql "$sql"
    done
fi

# 2. Run prod seeds
if [ -d "/docker-entrypoint-initdb.d/seeds/prod" ]; then
    echo "--- Running Prod Seeds ---"
    for sql in $(ls /docker-entrypoint-initdb.d/seeds/prod/*.sql | sort); do
        run_sql "$sql"
    done
fi

echo "=== Kepçe PostgreSQL Initialization Completed Successfully ==="
