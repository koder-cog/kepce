#!/bin/bash
# ==============================================================================
# KEPÇE - Al-Götür Menülerini Şablonlardan Zenginleştirme Betiği
# ==============================================================================
# config/takeaway/{sehir}/{donem}/{ogun}.json dosyalarındaki detaylı yemekleri
# veritabanındaki placeholder (örn: "Al-Götür Menü 7") kayıtlarının yerine işler.
# ==============================================================================

set -euo pipefail

if [ -f ".env" ]; then
    # shellcheck disable=SC2046
    export $(grep -v '^#' .env | xargs -0 -d '\n' 2>/dev/null || grep -v '^#' .env | xargs)
fi

DEPLOY_HOST="${KEPCE_DEPLOY_HOST:-}"
DEPLOY_KEY="${KEPCE_DEPLOY_KEY:-$HOME/.ssh/id_rsa}"
DEPLOY_DEST="${KEPCE_DEPLOY_DEST:-/home/ubuntu/kepce}"

echo "================================================================="
echo " Kepçe - Al-Götür Menülerini Şablonlardan Zenginleştirme"
echo "================================================================="

# Hedef ortamı belirle (varsayılan: sunucu varsa sunucu, yoksa yerel)
MODE="${1:-auto}"

if [ "$MODE" = "local" ] || ([ "$MODE" = "auto" ] && [ -z "$DEPLOY_HOST" ]); then
    echo "1. Yerel ortamda Al-Götür zenginleştirme başlatılıyor..."
    if command -v docker >/dev/null 2>&1 && docker ps | grep -q "kepce-worker"; then
        docker exec -e WORKER_ENRICH_TAKEAWAY=1 -e WORKER_ONESHOT=1 kepce-worker /app/kepce-worker
    else
        WORKER_ENRICH_TAKEAWAY=1 WORKER_ONESHOT=1 cargo run -p worker
    fi
else
    echo "1. Hedef sunucuya bağlanılıyor ($DEPLOY_HOST)..."
    ssh -i "$DEPLOY_KEY" "$DEPLOY_HOST" "cd '$DEPLOY_DEST' && docker compose run --rm -e WORKER_ENRICH_TAKEAWAY=1 -e WORKER_ONESHOT=1 worker /app/kepce-worker"
fi

echo "================================================================="
echo " Al-Götür menü zenginleştirmesi tamamlandı!"
echo "================================================================="
