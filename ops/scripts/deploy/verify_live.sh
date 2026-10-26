#!/bin/bash
# ==============================================================================
# Kepçe Canlı Doğrulama Betiği (O-2, O-3, O-4)
# ==============================================================================
# ROADMAP.md Bölüm 7.1'deki operatör doğrulama maddelerini tek komutta çalıştırır.
# SSH gerektirir; yurt/ofis ağı port 22'yi kapatıyorsa whitelist'li bir ağdan çalıştır.
#
# Kullanım:
#   bash .scratch/verify_live.sh
#
# .env varsa oradan KEPCE_DEPLOY_* değerlerini okur, yoksa varsayılanları kullanır.
# ==============================================================================

set -uo pipefail

# --- .env'den deploy bilgilerini yükle (varsa) ---
if [ -f .env ]; then
    # shellcheck disable=SC1091
    set -a; . ./.env; set +a
fi

HOST="${KEPCE_DEPLOY_HOST:-ubuntu@130.110.247.87}"
KEY="${KEPCE_DEPLOY_KEY:-$HOME/.ssh/kepce-ampere.key}"
DEST="${KEPCE_DEPLOY_DEST:-/home/ubuntu/kepce}"
DB_USER="${POSTGRES_USER:-kepce_admin}"
DB_NAME="${POSTGRES_DB:-kepce}"

SSH="ssh -i $KEY -o ConnectTimeout=10 -o BatchMode=yes -o StrictHostKeyChecking=accept-new $HOST"
PSQL="docker exec kepce-db psql -U $DB_USER -d $DB_NAME -tAc"

section() { echo; echo "=== $1 ==="; }

# --- Bağlantı testi ---
section "Bağlantı testi"
if ! $SSH 'echo SSH_OK' 2>&1; then
    echo "HATA: SSH bağlantısı kurulamadı. Ağ kısıtı veya anahtar sorunu olabilir."
    exit 1
fi

# --- O-4: J-3 canlı DB ---
section "O-4: pricing_periods 2026-2027 (İstanbul)"
$SSH "$PSQL \"SELECT id, city_slug, period_start, period_end FROM pricing_periods WHERE city_slug='istanbul' AND period_start='2026-09-01';\""

section "O-4: 164 tavan fiyat kaydı"
$SSH "$PSQL \"SELECT count(*) FROM meal_category_prices WHERE pricing_period_id=(SELECT id FROM pricing_periods WHERE city_slug='istanbul' AND period_start='2026-09-01');\""

# --- O-2: Bölüm 3 canlı iddiaları ---
section "O-2: migration 0030 (storage_ref)"
$SSH "$PSQL \"SELECT version, applied_at FROM schema_migrations WHERE version LIKE '0030%';\""

section "O-2: data/menuler izinleri"
$SSH "ls -ld $DEST/data/menuler $DEST/data/menuler/anonim/bekleyen 2>&1"

section "O-2: ONNX yön modeli imajda"
$SSH "docker exec kepce-worker ls -l /app/models/doc_orientation.onnx 2>&1"

section "O-2: Gemini model zinciri (son 24 saat)"
$SSH "docker logs kepce-worker --since 24h 2>&1 | grep -i 'Gemini model zinciri' | tail -3"

section "O-2: kykyemek devre kesici durumu"
$SSH "docker exec kepce-worker cat /app/cache/kykyemek_ban_until 2>&1"

section "O-2: konteyner durumu"
$SSH "cd $DEST && docker compose ps --format '{{.Name}} {{.Status}}'"

# --- O-3: yetim karantina dosyaları ---
section "O-3: bekleyen dosyalar"
$SSH "find $DEST/data/menuler -path '*/bekleyen/*' -type f 2>/dev/null | head -20"

section "O-3: karantina dizini (yetim dosyalar)"
$SSH "find $DEST/uploads/quarantine/menus -type f 2>/dev/null | head -20"

section "O-3: reddedilen arşivi"
$SSH "find $DEST/data/menuler/reddedilen -type f 2>/dev/null | head -20"

# --- O-1: son ingest logu (workflow çalıştıysa) ---
section "O-1: son ingest logu (worker)"
$SSH "docker logs kepce-worker --since 24h 2>&1 | grep -iE 'Aktarıldı|Alınan|internal/ingest' | tail -10"

echo
echo "=== Bitti. Çıktıyı kopyalayıp paylaş. ==="
