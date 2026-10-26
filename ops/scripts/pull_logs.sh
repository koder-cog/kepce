#!/usr/bin/env bash
set -euo pipefail

ARCHIVE_DIR="${ARCHIVE_DIR:-/var/log/kepce_arsiv}"
TEMP_FILE="/tmp/batch.zst.age"
HEADER_FILE="/tmp/batch.headers"
API_URL="${API_URL:-https://kepce.org/api/v1/internal/traffic-logs}"
SYNC_TOKEN="${SYNC_TOKEN:-}"

if [ -z "${SYNC_TOKEN}" ]; then
    echo "Hata: SYNC_TOKEN ortam değişkeni tanımlanmalıdır." >&2
    exit 1
fi

mkdir -p "${ARCHIVE_DIR}"

# 1. Fransa sunucusundan son log paketini çek
HTTP_CODE=$(curl -s -f -D "${HEADER_FILE}" \
    -H "X-Sync-Token: ${SYNC_TOKEN}" \
    "${API_URL}/pull" \
    -o "${TEMP_FILE}" \
    -w "%{http_code}" || echo "000")

if [ "${HTTP_CODE}" != "200" ]; then
    rm -f "${TEMP_FILE}" "${HEADER_FILE}"
    exit 0 # Yeni log yok (204 No Content) veya ağ henüz aktif değil
fi

# 2. Captive Portal ve Sağlama Doğrulaması
EXPECTED_SEQ=$(grep -i '^x-batch-seq-id:' "${HEADER_FILE}" | tr -d '\r\n' | awk '{print $2}')
EXPECTED_HASH=$(grep -i '^x-batch-hash:' "${HEADER_FILE}" | tr -d '\r\n' | awk '{print $2}')

if [ -z "${EXPECTED_SEQ}" ] || [ -z "${EXPECTED_HASH}" ]; then
    echo "Hata: Sunucu meta verisi eksik (X-Batch-Seq-Id veya X-Batch-Hash bulunamadı)." >&2
    rm -f "${TEMP_FILE}" "${HEADER_FILE}"
    exit 1
fi

# BLAKE3 Hash Kontrolü (b3sum veya python3)
if command -v b3sum >/dev/null 2>&1; then
    ACTUAL_HASH=$(b3sum --no-names "${TEMP_FILE}" | awk '{print $1}')
elif python3 -c "import blake3" >/dev/null 2>&1; then
    ACTUAL_HASH=$(python3 -c "import blake3, sys; print(blake3.blake3(open(sys.argv[1], 'rb').read()).hexdigest())" "${TEMP_FILE}")
else
    echo "Uyarı: b3sum veya python3-blake3 bulunamadı, hash doğrulaması atlanıyor." >&2
    ACTUAL_HASH="${EXPECTED_HASH}"
fi

if [ "${ACTUAL_HASH}" != "${EXPECTED_HASH}" ]; then
    echo "Hata: BLAKE3 özeti eşleşmedi. Beklenen: ${EXPECTED_HASH}, Gerçek: ${ACTUAL_HASH}" >&2
    rm -f "${TEMP_FILE}" "${HEADER_FILE}"
    exit 1
fi

# 3. Append-only Arşive Ekleme (chattr +a uygulanan klasöre güvenli kopyalama)
TARGET_FILE="${ARCHIVE_DIR}/$(date +%Y%m%d_%H%M%S)_seq${EXPECTED_SEQ}.zst.age"
cp "${TEMP_FILE}" "${TARGET_FILE}"
rm -f "${TEMP_FILE}" "${HEADER_FILE}"

# 4. Yarış Durumsuz ACK (Sadece çekilen seq_id'ye kadar olanı temizle)
curl -s -f -X POST \
    -H "X-Sync-Token: ${SYNC_TOKEN}" \
    "${API_URL}/ack?up_to_seq_id=${EXPECTED_SEQ}"

# 5. Felaket Kurtarma Kopyası (Opsiyonel çevrimdışı yedek)
if [ -d "/opt/kepce_offsite_backup" ]; then
    cp "${TARGET_FILE}" "/opt/kepce_offsite_backup/"
fi
