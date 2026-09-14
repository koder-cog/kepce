#!/usr/bin/env bash
# reset-db.sh - Lokal veritabanını sıfırlama ve baştan migrasyon/seed koşturma betiği

set -euo pipefail

DB_CONTAINER="${DB_CONTAINER:-kepce-db}"
POSTGRES_USER="${POSTGRES_USER:-postgres}"
POSTGRES_DB="${POSTGRES_DB:-kepce}"

if command -v docker &> /dev/null && docker ps -q -f name="$DB_CONTAINER" | grep -q .; then
    CMD="docker"
elif command -v podman &> /dev/null && podman ps -q -f name="$DB_CONTAINER" | grep -q .; then
    CMD="podman"
else
    echo "[$(date)] HATA: '$DB_CONTAINER' konteyneri bulunamadı." >&2
    exit 1
fi

echo "[UYARI] Lokal veritabanı sıfırlanıyor: $POSTGRES_DB..."

$CMD exec -i "$DB_CONTAINER" psql -U "$POSTGRES_USER" -d "$POSTGRES_DB" -c "DROP SCHEMA public CASCADE; CREATE SCHEMA public;"

echo "[BİLGİ] Migrasyonlar sırayla koşturuluyor..."
$CMD exec -i "$DB_CONTAINER" psql -U "$POSTGRES_USER" -d "$POSTGRES_DB" -c \
    "CREATE TABLE IF NOT EXISTS schema_migrations (version VARCHAR(255) PRIMARY KEY, applied_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP);"

for sql in $(ls db/migrations/*.sql | sort); do
    fname=$(basename "$sql")
    echo " -> $fname"
    $CMD exec -i "$DB_CONTAINER" psql -U "$POSTGRES_USER" -d "$POSTGRES_DB" -f - < "$sql"
    $CMD exec -i "$DB_CONTAINER" psql -U "$POSTGRES_USER" -d "$POSTGRES_DB" -c \
        "INSERT INTO schema_migrations (version) VALUES ('$fname') ON CONFLICT DO NOTHING;"
done

echo "[BİLGİ] Prod seed verileri koşturuluyor..."
for sql in $(ls db/seeds/prod/*.sql | sort); do
    echo " -> $sql"
    $CMD exec -i "$DB_CONTAINER" psql -U "$POSTGRES_USER" -d "$POSTGRES_DB" -f - < "$sql"
done

echo "[BAŞARI] Veritabanı başarıyla sıfırlandı ve güncellendi!"
