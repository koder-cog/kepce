#!/usr/bin/env bash
# Kepçe - pgBackRest İlk Kurulum ve Test Betiği
# Bu betik docker-compose up ile veritabanı ilk kez ayağa kalktıktan sonra 
# bir kez (one-time) çalıştırılır.

set -euo pipefail

DB_CONTAINER="${DB_CONTAINER:-kepce-db}"

# Container'ın ayakta olup olmadığını kontrol et
if ! podman exec "$DB_CONTAINER" /bin/true &> /dev/null; then
    # Podman yoksa docker ile dene
    if docker exec "$DB_CONTAINER" /bin/true &> /dev/null; then
        CMD="docker"
    else
        echo "[$(date)] HATA: '$DB_CONTAINER' container'ı ayakta değil veya ulaşılamıyor!" >&2
        exit 1
    fi
else
    CMD="podman"
fi

echo "[BİLGİ] pgBackRest Stanza (Profil) oluşturuluyor..."
$CMD exec -u postgres "$DB_CONTAINER" pgbackrest --stanza=kepce-stanza stanza-create

echo "[BAŞARI] Stanza oluşturuldu. Konfigürasyon kontrol ediliyor..."
$CMD exec -u postgres "$DB_CONTAINER" pgbackrest --stanza=kepce-stanza check

echo "[BİLGİ] İlk tam (FULL) yedekleme başlatılıyor..."
$CMD exec -u postgres "$DB_CONTAINER" pgbackrest --type=full --stanza=kepce-stanza backup

echo "[BAŞARI] Kurulum tamamlandı! Her şey çalışıyor."
echo "Not: Aşağıdaki crontab görevlerini sunucunuza eklemeyi unutmayın:"
echo "--------------------------------------------------------"
echo "0 3 * * 0 $CMD exec -u postgres $DB_CONTAINER pgbackrest --type=full --stanza=kepce-stanza backup"
echo "0 3 * * 1-6 $CMD exec -u postgres $DB_CONTAINER pgbackrest --type=diff --stanza=kepce-stanza backup"
echo "--------------------------------------------------------"
