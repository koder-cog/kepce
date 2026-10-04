#!/usr/bin/env bash
# ==============================================================================
# KEPÇE - 4 Şehir Saha Gönderimleri Multimodal Ingestion Pipeline
# ==============================================================================
# Bu betik; pano fotoğrafları ve menü PDF'lerini data/menuler yapısına yerleştirir
# ve multimodal LLM (Gemini/OpenRouter Vision) motoru üzerinden ayrıştırarak
# veritabanına tescilli GroundTruth (saha teyitli) kaynak olarak kaydeder.
# ==============================================================================

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$REPO_ROOT"

# Note: Rust binary loads .env automatically via dotenvy

SOURCE_DIR="${1:-${SUBMISSIONS_DIR:-data/submissions}}"
DATA_DIR="${WORKER_MENU_DIR:-data/menuler}"

echo "================================================================="
echo " Kepçe - Multimodal Submissions Ingestion Pipeline"
echo "================================================================="
echo "Kaynak Dizin : $SOURCE_DIR"
echo "Hedef Dizin  : $DATA_DIR"
echo "Tarih        : $(date '+%Y-%m-%d %H:%M:%S')"
echo "-----------------------------------------------------------------"

if [ ! -d "$SOURCE_DIR" ]; then
    echo "Hata: Kaynak dizin bulunamadı: $SOURCE_DIR"
    exit 1
fi

# 1. Klasör yapısını hazırla
mkdir -p "$DATA_DIR/admin/bekleyen/istanbul"
mkdir -p "$DATA_DIR/anonim/bekleyen/afyonkarahisar"
mkdir -p "$DATA_DIR/anonim/bekleyen/bursa"
mkdir -p "$DATA_DIR/anonim/bekleyen/corum"

# 2. Dosyaları bekleyen klasörlerine kopyala (orijinalleri korumak için cp kullanılır)
if [ -d "$SOURCE_DIR/admin/istanbul" ]; then
    echo "-> İstanbul admin pano fotoğrafları kopyalanıyor..."
    cp -v "$SOURCE_DIR/admin/istanbul/"* "$DATA_DIR/admin/bekleyen/istanbul/" 2>/dev/null || true
fi

if [ -d "$SOURCE_DIR/anonim/afyonkarahisar" ]; then
    echo "-> Afyonkarahisar anonim pano fotoğrafları kopyalanıyor..."
    cp -v "$SOURCE_DIR/anonim/afyonkarahisar/"* "$DATA_DIR/anonim/bekleyen/afyonkarahisar/" 2>/dev/null || true
fi

if [ -d "$SOURCE_DIR/anonim/bursa" ]; then
    echo "-> Bursa anonim menü PDF'i kopyalanıyor..."
    cp -v "$SOURCE_DIR/anonim/bursa/"* "$DATA_DIR/anonim/bekleyen/bursa/" 2>/dev/null || true
fi

if [ -d "$SOURCE_DIR/anonim/corum" ]; then
    echo "-> Çorum anonim pano fotoğrafları kopyalanıyor..."
    cp -v "$SOURCE_DIR/anonim/corum/"* "$DATA_DIR/anonim/bekleyen/corum/" 2>/dev/null || true
fi

echo "-----------------------------------------------------------------"
echo "3. Worker multimodal LLM ingest motoru çalıştırılıyor..."
echo "-----------------------------------------------------------------"

WORKER_MENU_DIR="$DATA_DIR" WORKER_LOCAL_INGEST=1 WORKER_ONESHOT=1 cargo run -p worker --bin worker

echo "-----------------------------------------------------------------"
echo "4. Ingestion tamamlandı. Vault ve veritabanı durumu denetleniyor..."
echo "-----------------------------------------------------------------"

echo "İşlenen dosyalar (Vault):"
find "$DATA_DIR/vault" -type f 2>/dev/null | tail -n 20 || echo "Vault henüz boş."

echo "================================================================="
echo " Ingestion süreci başarıyla tamamlandı!"
echo "================================================================="
