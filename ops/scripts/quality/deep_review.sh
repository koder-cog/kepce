#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

DEEP_REVIEW_DIR="${DEEP_REVIEW_DIR:-$ROOT_DIR/.deep-review}"
mkdir -p "$DEEP_REVIEW_DIR"

OUTPUT_FILE="$DEEP_REVIEW_DIR/faz0_baglam.txt"
echo "=== Kepçe Derin İnceleme: Faz 0 Başlatılıyor ==="
echo "Hedef dizin: $DEEP_REVIEW_DIR"
echo "Çıktı dosyası: $OUTPUT_FILE"

{
  echo "=================================================="
  echo "Derin İnceleme Faz 0 Bağlam Göstergeleri"
  echo "Tarih: $(date -u +"%Y-%m-%dT%H:%M:%SZ")"
  echo "Commit: $(git rev-parse HEAD 2>/dev/null || echo 'N/A')"
  echo "Dal: $(git branch --show-current 2>/dev/null || echo 'N/A')"
  echo "=================================================="
  echo ""

  echo "=== 1. Git Geçmişi ve Değişim Yoğunluğu (Son 3 Ay) ==="
  echo -n "Toplam commit sayısı: "
  git log --since="3 months ago" --oneline 2>/dev/null | wc -l

  echo ""
  echo "=== En Çok Değiştirilen 20 Dosya ==="
  git log --since="3 months ago" --diff-filter=M --name-only --pretty=format: 2>/dev/null \
    | grep -v '^$' | sort | uniq -c | sort -rn | head -20 || true

  echo ""
  echo "=== Son Hata Düzeltmeleri ==="
  git log --since="3 months ago" --grep="fix\|bug\|patch\|hotfix" --oneline 2>/dev/null | head -30 || true

  echo ""
  echo "=== 2. Rust Bağımlılık Güvenlik Denetimi ==="
  if command -v cargo-audit >/dev/null 2>&1 || cargo audit --version >/dev/null 2>&1; then
    cargo audit 2>&1 || true
  else
    echo "cargo audit bulunamadı, denetim atlandı."
  fi

  echo ""
  echo "=== 3. Node.js Bağımlılık Güvenlik Denetimi ==="
  if [ -d "webapp" ] && [ -f "webapp/package.json" ]; then
    (cd webapp && npm audit 2>&1 | head -40) || true
  fi

  echo ""
  echo "=== 4. Eşzamanlılık ve Güvenilirlik Göstergeleri ==="
  rg -n "worker|retry|backoff|idempot|queue|cron|cache|timeout|lock|mutex|transaction|select_for_update" \
    --glob '!**/target/**' --glob '!**/node_modules/**' --glob '!**/.scratch/**' 2>/dev/null | head -100 || true

} > "$OUTPUT_FILE"

echo "Faz 0 tamamlandı. Veriler $OUTPUT_FILE dosyasına kaydedildi."
