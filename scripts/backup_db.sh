#!/usr/bin/env bash
# ==============================================================================
# KEPÇE - Veritabanı Yedekleme Yardımcı Betiği
# ==============================================================================
# pgBackRest (sunucu/üretim) veya pg_dump (yerel geliştirme) üzerinden
# veritabanı yedeği alır.
#
# Kullanım:
#   ./scripts/backup_db.sh [SEÇENEKLER]
#
# Seçenekler:
#   --full        pgBackRest ile tam (full) fiziksel yedek al
#   --diff        pgBackRest ile fark (diff) fiziksel yedek al
#   --logical     pg_dump ile mantıksal SQL yedeği al (backups/ dizinine yazar)
#   -h, --help    Yardım iletisini göster
# ==============================================================================

set -euo pipefail

# Load environment variables
if [ -f ".env" ]; then
    # shellcheck disable=SC2046
    export $(grep -v '^#' .env | xargs -0 -d '\n' 2>/dev/null || grep -v '^#' .env | xargs)
elif [ -f "../.env" ]; then
    # shellcheck disable=SC2046
    export $(grep -v '^#' ../.env | xargs -0 -d '\n' 2>/dev/null || grep -v '^#' ../.env | xargs)
fi

DB_CONTAINER="${DB_CONTAINER:-kepce-db}"
DB_USER="${POSTGRES_USER:-kepce_admin}"
DB_NAME="${POSTGRES_DB:-kepce}"

show_help() {
    echo "Kullanım: $0 [SEÇENEKLER]"
    echo ""
    echo "Seçenekler:"
    echo "  --full        pgBackRest ile tam (full) fiziksel yedek al"
    echo "  --diff        pgBackRest ile fark (diff) fiziksel yedek al"
    echo "  --logical     pg_dump ile sıkıştırılmış SQL mantıksal yedeği al"
    echo "  -h, --help    Bu yardım iletisini göster"
    echo ""
    echo "Not: Hiçbir seçenek verilmezse konteynerde pgBackRest kontrol edilir;"
    echo "bulunursa tam yedek, bulunamazsa yerel pg_dump yedeği alınır."
}

MODE="auto"

for arg in "$@"; do
    case "$arg" in
        -h|--help)
            show_help
            exit 0
            ;;
        --full)
            MODE="full"
            ;;
        --diff)
            MODE="diff"
            ;;
        --logical)
            MODE="logical"
            ;;
        *)
            echo "Hata: Bilinmeyen parametre: $arg" >&2
            echo "Yardım için: $0 --help" >&2
            exit 1
            ;;
    esac
done

# Konteyner CLI tespiti (docker veya podman)
if command -v docker &> /dev/null && docker ps -q -f name="$DB_CONTAINER" | grep -q .; then
    CMD="docker"
elif command -v podman &> /dev/null && podman ps -q -f name="$DB_CONTAINER" | grep -q .; then
    CMD="podman"
else
    echo "Hata: '$DB_CONTAINER' konteyneri bulunamadı veya çalışmıyor." >&2
    exit 1
fi

take_pgbackrest_backup() {
    local type="$1"
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] [BİLGİ] pgBackRest ($type) yedeklemesi başlatılıyor..."
    $CMD exec -u postgres "$DB_CONTAINER" pgbackrest --type="$type" --stanza=kepce-stanza backup
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] [BAŞARI] pgBackRest ($type) yedeklemesi tamamlandı."
}

take_logical_backup() {
    mkdir -p backups
    local timestamp
    timestamp=$(date '+%Y%m%d_%H%M%S')
    local backup_file="backups/kepce_${DB_NAME}_${timestamp}.sql.gz"
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] [BİLGİ] pg_dump ile mantıksal yedek alınıyor..."
    $CMD exec "$DB_CONTAINER" pg_dump -U "$DB_USER" "$DB_NAME" | gzip > "$backup_file"
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] [BAŞARI] Mantıksal yedek oluşturuldu: $backup_file"
}

case "$MODE" in
    full)
        take_pgbackrest_backup "full"
        ;;
    diff)
        take_pgbackrest_backup "diff"
        ;;
    logical)
        take_logical_backup
        ;;
    auto)
        if $CMD exec "$DB_CONTAINER" which pgbackrest &>/dev/null && \
           $CMD exec -u postgres "$DB_CONTAINER" pgbackrest --stanza=kepce-stanza check &>/dev/null; then
            take_pgbackrest_backup "full"
        else
            take_logical_backup
        fi
        ;;
esac
