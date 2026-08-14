#!/bin/bash
# ==============================================================================
# KEPÇE - KYKYEMEK Backup Ingestion Utility
# ==============================================================================
# Usage:
#   ./scripts/backup_ingest.sh [options] [backup_directory]
#
# Options:
#   --remote    Ingest into production server database via SSH/Docker
#   --local     Ingest into local database using local worker
# ==============================================================================

set -euo pipefail

# Load environment variables
if [ -f ".env" ]; then
    # shellcheck disable=SC2046
    export $(grep -v '^#' .env | xargs -0 -d '\n' 2>/dev/null || grep -v '^#' .env | xargs)
fi

BACKUP_DIR="${1:-.scratch/archive/misc/kykyemek-şmnmh-yedek}"
IS_REMOTE=false

for arg in "$@"; do
    case "$arg" in
        --remote)
            IS_REMOTE=true
            ;;
        --local)
            IS_REMOTE=false
            ;;
        -*)
            echo "Bilinmeyen parametre: $arg"
            exit 1
            ;;
        *)
            BACKUP_DIR="$arg"
            ;;
    esac
done

if [ ! -d "$BACKUP_DIR" ]; then
    echo "Hata: Belirtilen yedek dizini bulunamadı: $BACKUP_DIR"
    exit 1
fi

echo "================================================================="
echo " Kepçe - KYK Yemek Yedek Menü Aktarımı"
echo " Kaynak Dizin: $BACKUP_DIR"
echo " Mod: $([ "$IS_REMOTE" = true ] && echo "UZAK SUNUCU (Production)" || echo "YEREL (Local)")"
echo "================================================================="

if [ "$IS_REMOTE" = true ]; then
    DEPLOY_HOST="${KEPCE_DEPLOY_HOST:-}"
    DEPLOY_KEY="${KEPCE_DEPLOY_KEY:-}"
    DEPLOY_DEST="${KEPCE_DEPLOY_DEST:-/home/ubuntu/kepce}"

    if [ -z "$DEPLOY_HOST" ] || [ -z "$DEPLOY_KEY" ]; then
        echo "Hata: KEPCE_DEPLOY_HOST veya KEPCE_DEPLOY_KEY tanımlı değil (.env dosyasını kontrol edin)."
        exit 1
    fi

    echo "1. Yedek dosyaları sunucuya aktarılıyor..."
    SSH_CMD="ssh -i $DEPLOY_KEY"
    $SSH_CMD "$DEPLOY_HOST" "mkdir -p $DEPLOY_DEST/data/backup_ingest"
    rsync -avz -e "$SSH_CMD" "$BACKUP_DIR/" "$DEPLOY_HOST:$DEPLOY_DEST/data/backup_ingest/"

    echo "2. Sunucudaki worker ile veritabanına aktarılıyor..."
    $SSH_CMD "$DEPLOY_HOST" "cd $DEPLOY_DEST && docker compose -f docker-compose.yml -f docker-compose.prod.yml -f docker-compose.analytics.yml -f docker-compose.ai.yml run --rm -v $DEPLOY_DEST/data/backup_ingest:/app/data/backup_ingest -e WORKER_BACKUP_INGEST=1 -e WORKER_ONESHOT=1 -e WORKER_BACKUP_DIR=/app/data/backup_ingest worker"

    echo "================================================================="
    echo " Sunucu yedek aktarımı başarıyla tamamlandı!"
    echo "================================================================="
else
    echo "1. Yerel veritabanına aktarılıyor..."
    WORKER_BACKUP_INGEST=1 WORKER_ONESHOT=1 WORKER_BACKUP_DIR="$BACKUP_DIR" cargo run --release -p worker

    echo "================================================================="
    echo " Yerel yedek aktarımı başarıyla tamamlandı!"
    echo "================================================================="
fi
