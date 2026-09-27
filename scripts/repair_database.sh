#!/bin/bash
# ==============================================================================
# KEPÇE - Otonom Veritabanı Onarım ve Uzlaşma Betiği (Self-Repair Pipeline)
# ==============================================================================
# 1. Devre kesici kilit dosyalarını temizler.
# 2. Proxy havuzu üzerinden kykyemek.com bültenlerini yeni ayrıştırıcıyla baştan tarar.
# 3. Al-Götür paketlerini şablonlardan giydirir (500 ml su, sandviç vb. eksikleri tamamlar).
# 4. Yemek ve alias mükerrerliklerini birleştirir, yabancı anahtarları normalize eder.
# 5. Kategori eşlemelerini günceller ve güncel istatistikleri ekrana basar.
# ==============================================================================

set -euo pipefail

if [ -f ".env" ]; then
    # shellcheck disable=SC2046
    export $(grep -v '^#' .env | xargs -0 -d '\n' 2>/dev/null || grep -v '^#' .env | xargs)
fi

DEPLOY_HOST="${KEPCE_DEPLOY_HOST:-}"
DEPLOY_KEY="${KEPCE_DEPLOY_KEY:-$HOME/.ssh/id_rsa}"
DEPLOY_DEST="${KEPCE_DEPLOY_DEST:-/home/ubuntu/kepce}"
MODE="${1:-auto}"

echo "================================================================="
echo " Kepçe - Otonom Veritabanı Onarım ve Uzlaşma Süreci"
echo "================================================================="

if [ "$MODE" = "local" ] || ([ "$MODE" = "auto" ] && [ -z "$DEPLOY_HOST" ]); then
    echo "1. Devre kesici kilit dosyaları temizleniyor..."
    rm -f /tmp/kykyemek_ban_until /tmp/kyk_ban_until

    echo "2. kykyemek bültenleri güncel HTML ayrıştırıcıyla taranıyor..."
    WORKER_RUN_SCRAPER=1 WORKER_ONESHOT=1 cargo run -p worker

    echo "3. Al-Götür menüleri şablonlardan zenginleştiriliyor..."
    WORKER_ENRICH_TAKEAWAY=1 WORKER_ONESHOT=1 cargo run -p worker

    echo "4. Yemek ve alias mükerrerlikleri uzlaştırılıyor..."
    WORKER_RECONCILE_DISHES=1 WORKER_ONESHOT=1 cargo run -p worker

    echo "5. Yemek kategorileri yenileniyor..."
    WORKER_RECATEGORIZE=1 WORKER_ONESHOT=1 cargo run -p worker
else
    echo "1. Hedef sunucuya bağlanılıyor ($DEPLOY_HOST)..."

    echo "-> Devre kesici kilit dosyaları temizleniyor..."
    ssh -i "$DEPLOY_KEY" "$DEPLOY_HOST" "docker exec kepce-worker rm -f /app/cache/kykyemek_ban_until /app/cache/kyk_ban_until || true"

    echo "-> kykyemek bültenleri güncel HTML ayrıştırıcıyla taranıyor..."
    ssh -i "$DEPLOY_KEY" "$DEPLOY_HOST" "cd '$DEPLOY_DEST' && docker compose run --rm -e WORKER_RUN_SCRAPER=1 -e WORKER_ONESHOT=1 worker /app/kepce-worker"

    echo "-> Al-Götür menüleri şablonlardan zenginleştiriliyor..."
    ssh -i "$DEPLOY_KEY" "$DEPLOY_HOST" "cd '$DEPLOY_DEST' && docker compose run --rm -e WORKER_ENRICH_TAKEAWAY=1 -e WORKER_ONESHOT=1 worker /app/kepce-worker"

    echo "-> Yemek ve alias mükerrerlikleri uzlaştırılıyor..."
    ssh -i "$DEPLOY_KEY" "$DEPLOY_HOST" "cd '$DEPLOY_DEST' && docker compose run --rm -e WORKER_RECONCILE_DISHES=1 -e WORKER_ONESHOT=1 worker /app/kepce-worker"

    echo "-> Yemek kategorileri yenileniyor..."
    ssh -i "$DEPLOY_KEY" "$DEPLOY_HOST" "cd '$DEPLOY_DEST' && docker compose run --rm -e WORKER_RECATEGORIZE=1 -e WORKER_ONESHOT=1 worker /app/kepce-worker"

    echo "-> Güncel veritabanı durumu özeti:"
    ssh -i "$DEPLOY_KEY" "$DEPLOY_HOST" "docker exec kepce-db psql -U kepce_admin -d kepce -c 'SELECT status, count(*) FROM menus GROUP BY status;'"
fi

echo "================================================================="
echo " Veritabanı onarım ve uzlaşma süreci tamamlandı!"
echo "================================================================="
