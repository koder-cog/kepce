#!/usr/bin/env bash
# backup-full.sh — pgBackRest Tam Yedek Alım Betiği (Cron Uyumlu)

set -euo pipefail

DB_CONTAINER="${DB_CONTAINER:-kepce-db}"

if command -v docker &> /dev/null && docker ps -q -f name="$DB_CONTAINER" | grep -q .; then
    CMD="docker"
elif command -v podman &> /dev/null && podman ps -q -f name="$DB_CONTAINER" | grep -q .; then
    CMD="podman"
else
    echo "[$(date)] HATA: '$DB_CONTAINER' konteyneri bulunamadı veya çalışmıyor." >&2
    exit 1
fi

echo "[$(date)] 🚀 Tam (FULL) pgBackRest yedeklemesi başlatılıyor..."
$CMD exec -u postgres "$DB_CONTAINER" pgbackrest --type=full --stanza=kepce-stanza backup
echo "[$(date)] ✅ Tam yedekleme tamamlandı."
