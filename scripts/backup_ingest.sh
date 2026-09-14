#!/bin/bash
# ==============================================================================
# KEPÇE - KYKYEMEK Backup Ingestion Utility
# ==============================================================================
# Kullanım:
#   ./scripts/backup_ingest.sh <yedek_dizini> [SEÇENEKLER]
#
# Seçenekler:
#   --local     Yerel veritabanına aktar (varsayılan)
#   --remote    Canlı sunucuya aktar (SSH ve Docker ile)
#   -h, --help  Yardım iletisini göster
# ==============================================================================

set -euo pipefail

# Load environment variables
if [ -f ".env" ]; then
    # shellcheck disable=SC2046
    export $(grep -v '^#' .env | xargs -0 -d '\n' 2>/dev/null || grep -v '^#' .env | xargs)
fi

show_help() {
    echo "Kullanım:"
    echo "  $0 <yedek_dizini> [SEÇENEKLER]"
    echo ""
    echo "Seçenekler:"
    echo "  --local     Yerel veritabanına aktar (varsayılan)"
    echo "  --remote    Canlı sunucuya aktar (SSH ve Docker ile)"
    echo "  -h, --help  Bu yardım iletisini göster"
    echo ""
    echo "Örnekler:"
    echo "  $0 /path/to/backup_folder"
    echo "  $0 /path/to/backup_folder --remote"
}

BACKUP_DIR=""
IS_REMOTE=false

for arg in "$@"; do
    case "$arg" in
        -h|--help)
            show_help
            exit 0
            ;;
        --remote)
            IS_REMOTE=true
            ;;
        --local)
            IS_REMOTE=false
            ;;
        -*)
            echo "Hata: Bilinmeyen parametre: $arg" >&2
            echo "Yardım için: $0 --help" >&2
            exit 1
            ;;
        *)
            if [ -n "$BACKUP_DIR" ]; then
                echo "Hata: Birden fazla yedek dizini belirtildi: '$BACKUP_DIR' ve '$arg'" >&2
                exit 1
            fi
            BACKUP_DIR="$arg"
            ;;
    esac
done

if [ -z "$BACKUP_DIR" ]; then
    echo "Hata: Yedek dizini belirtilmedi." >&2
    echo "Kullanım: $0 <yedek_dizini> [--local|--remote]" >&2
    echo "Yardım için: $0 --help" >&2
    exit 1
fi

if [ ! -d "$BACKUP_DIR" ]; then
    echo "Hata: Belirtilen yedek dizini bulunamadı: $BACKUP_DIR" >&2
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
        echo "Hata: KEPCE_DEPLOY_HOST veya KEPCE_DEPLOY_KEY tanımlı değil (.env dosyasını kontrol edin)." >&2
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
