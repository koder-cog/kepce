#!/bin/bash
set -euo pipefail

# ==============================================================================
# KEPÇE - Dağıtım ve Düğüm (Node) Kurulum Sihirbazı
# ==============================================================================

# Renkler
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
BOLD='\033[1m'
NC='\033[0m'

echo -e "${BLUE}${BOLD}=====================================================${NC}"
echo -e "${BLUE}${BOLD}       KEPÇE - Sunucu & Düğüm Dağıtım Sihirbazı      ${NC}"
echo -e "${BLUE}${BOLD}=====================================================${NC}"
echo ""

# 1. Sunucu Bağlantı Bilgileri (Yerel .env dosyasından veya ortam değişkenlerinden beslenir)
if [ -f .env ]; then
    export $(grep -E '^KEPCE_DEPLOY_' .env 2>/dev/null | xargs) 2>/dev/null || true
fi

DEFAULT_SERVER="${KEPCE_DEPLOY_HOST:-}"
DEFAULT_KEY="${KEPCE_DEPLOY_KEY:-$HOME/.ssh/id_rsa}"
DEFAULT_DEST="${KEPCE_DEPLOY_DEST:-/home/ubuntu/kepce}"

if [ -n "$DEFAULT_SERVER" ]; then
    read -r -p "Hedef Sunucu [$DEFAULT_SERVER]: " SERVER_HOST || true
    SERVER_HOST="${SERVER_HOST:-$DEFAULT_SERVER}"
else
    read -r -p "Hedef Sunucu (örn: user@ip): " SERVER_HOST || true
fi

if [ -z "$SERVER_HOST" ]; then
    echo -e "${RED}Hata: Hedef sunucu belirtilmedi.${NC}"
    exit 1
fi

read -r -p "SSH Anahtar Yolu [$DEFAULT_KEY]: " SSH_KEY || true
SSH_KEY="${SSH_KEY:-$DEFAULT_KEY}"

read -r -p "Sunucu Kurulum Dizini [$DEFAULT_DEST]: " REMOTE_DIR || true
REMOTE_DIR="${REMOTE_DIR:-$DEFAULT_DEST}"

if [ ! -f "$SSH_KEY" ]; then
    echo -e "${RED}Hata: SSH anahtarı bulunamadı: $SSH_KEY${NC}"
    exit 1
fi
chmod 600 "$SSH_KEY" 2>/dev/null || true

echo -e "\n${BLUE}SSH bağlantısı test ediliyor...${NC}"
if ! ssh -o BatchMode=yes -o ConnectTimeout=5 -i "$SSH_KEY" "$SERVER_HOST" "echo 'OK'" >/dev/null 2>&1; then
    echo -e "${RED}Hata: $SERVER_HOST sunucusuna bağlanılamadı. Lütfen SSH anahtarını ve bağlantıyı kontrol edin.${NC}"
    exit 1
fi
echo -e "${GREEN}Bağlantı başarılı.${NC}\n"

# 2. Modül Tercihleri
echo -e "${BOLD}--- Modül Seçimleri ---${NC}"

read -r -p "1. 5651 Sayılı Kanun Uyumlu IP & Erişim Loglaması aktif edilsin mi? [E/h]: " OPT_LOGS || true
OPT_LOGS="${OPT_LOGS:-E}"

read -r -p "2. Umami Web Analitiği (Self-Hosted) kurulsun mu? [E/h]: " OPT_UMAMI || true
OPT_UMAMI="${OPT_UMAMI:-E}"

read -r -p "3. Yerel Türkçe BERT Moderasyon Modeli aktif edilsin mi? [E/h]: " OPT_BERT || true
OPT_BERT="${OPT_BERT:-E}"

echo -e "\n${BLUE}--- Dağıtım Başlatılıyor ---${NC}"

# 3. Dosyaların Senkronizasyonu (Rsync)
echo -e "${YELLOW}[1/5] Proje dosyaları sunucuya aktarılıyor...${NC}"

if [ ! -f "target/aarch64-unknown-linux-gnu/release/api" ]; then
    echo -e "${YELLOW}İpucu: Henüz yerel ARM64 binary derlenmemiş.${NC}"
    echo -e "${YELLOW}Yerel makinenizde cross-compile yapmak derlemeyi hızlandırabilir: './manage.sh build-arm64'${NC}"
fi

ssh -i "$SSH_KEY" "$SERVER_HOST" "mkdir -p $REMOTE_DIR/{certs,logs/caddy,db/migrations,api,worker,webapp,static,target/aarch64-unknown-linux-gnu/release}"

rsync -avz --delete \
    --exclude-from='.gitignore' \
    --exclude='target/debug' \
    --exclude='target/release' \
    --exclude='node_modules' \
    --exclude='.git' \
    --exclude='.agents' \
    --exclude='certs' \
    -e "ssh -i $SSH_KEY" \
    ./ "$SERVER_HOST:$REMOTE_DIR/"

# 4. Uzak Sunucuda Ortam Değişkenleri (.env) Kontrolü
echo -e "${YELLOW}[2/5] Sunucu ortam değişkenleri doğrulanıyor...${NC}"
ssh -i "$SSH_KEY" "$SERVER_HOST" "
    set -euo pipefail
    if [ ! -f $REMOTE_DIR/.env ]; then
        if [ -f $REMOTE_DIR/.env.production ]; then
            cp $REMOTE_DIR/.env.production $REMOTE_DIR/.env
        elif [ -f $REMOTE_DIR/.env.example ]; then
            cp $REMOTE_DIR/.env.example $REMOTE_DIR/.env
        fi
        echo 'Yeni .env dosyası oluşturuldu.'
    fi
"

# 5. Compose Dosyalarının Belirlenmesi
COMPOSE_CMD="docker compose -f docker-compose.yml -f docker-compose.prod.yml"

if [[ "$OPT_UMAMI" =~ ^[EeYy]$ ]]; then
    COMPOSE_CMD="$COMPOSE_CMD -f docker-compose.analytics.yml"
    echo -e "${GREEN}Umami Analitik modülü dahil edildi.${NC}"
fi

if [[ "$OPT_BERT" =~ ^[EeYy]$ ]]; then
    COMPOSE_CMD="$COMPOSE_CMD -f docker-compose.ai.yml"
    echo -e "${GREEN}Yerel Türkçe BERT Moderasyon modülü dahil edildi.${NC}"
fi

# 6. Sunucuda Konteynerlerin Başlatılması ve Veritabanı Hazırlığı
echo -e "${YELLOW}[3/5] Veritabanı ve servisler hazırlanıyor...${NC}"
ssh -i "$SSH_KEY" "$SERVER_HOST" "
    set -euo pipefail
    cd $REMOTE_DIR
    export \$(grep -v '^#' .env | xargs)

    # 1. DB başlat (Tüm compose dosyalarını kullanarak sahipsiz konteyner uyarısını engelle)
    $COMPOSE_CMD up -d --no-deps db
    
    # 2. DB sağlık kontrolü bekle
    echo 'Veritabanının hazır olması bekleniyor...'
    until docker exec kepce-db pg_isready -U \${POSTGRES_USER:-kepce_admin} -d \${POSTGRES_DB:-kepce} >/dev/null 2>&1; do
        sleep 1
    done

    # 3. Umami için DB aç (varsa hata vermez)
    docker exec kepce-db psql -U \${POSTGRES_USER:-kepce_admin} -d postgres -tc \"SELECT 1 FROM pg_database WHERE datname = 'umami'\" | grep -q 1 || \
    docker exec kepce-db psql -U \${POSTGRES_USER:-kepce_admin} -d postgres -c \"CREATE DATABASE umami;\" 2>/dev/null || true

    # 4. Eksik migrasyonları uygula
    for migration in db/migrations/*.sql; do
        fname=\$(basename \$migration)
        is_applied=\$(docker exec kepce-db psql -U \${POSTGRES_USER:-kepce_admin} -d \${POSTGRES_DB:-kepce} -tAc \"SELECT EXISTS (SELECT 1 FROM schema_migrations WHERE version = '\$fname');\" 2>/dev/null || echo 'false')
        if [ \"\$is_applied\" != \"t\" ]; then
            echo \"Migrasyon uygulanıyor: \$fname\"
            docker exec -i kepce-db psql -U \${POSTGRES_USER:-kepce_admin} -d \${POSTGRES_DB:-kepce} < \$migration
            docker exec kepce-db psql -U \${POSTGRES_USER:-kepce_admin} -d \${POSTGRES_DB:-kepce} -c \"INSERT INTO schema_migrations (version) VALUES ('\$fname') ON CONFLICT DO NOTHING;\" >/dev/null 2>&1 || true
        fi
    done

    # 5. Tüm servisleri ayağa kaldır (Hata olursa derleme çıkış yapsın)
    echo 'Konteynerler derleniyor ve başlatılıyor...'
    $COMPOSE_CMD up -d --build --remove-orphans

    # 6. Yetkisiz konteyner dosya izinlerini düzelt
    docker exec -u 0 kepce-api chown -R nobody:nogroup /app/static /app/uploads 2>/dev/null || true

    # 7. Caddy config'i yeniden yükle: Caddyfile bind-mount olduğu için dosya
    #    değişimi 'up -d' tarafından algılanmaz; restart mount'ı yeniden çözer.
    \$COMPOSE_CMD restart caddy
"

# 7. Sağlık Kontrolü
echo -e "${YELLOW}[4/5] Canlı sağlık kontrolü yapılıyor...${NC}"
sleep 3
HEALTH_CODE=$(curl -s -o /dev/null -w "%{http_code}" --connect-timeout 10 https://kepce.org/api/v1/system/health || echo "000")

if [ "$HEALTH_CODE" != "200" ]; then
    echo -e "${RED}${BOLD}Hata: Sağlık kontrolü başarısız oldu! (HTTP $HEALTH_CODE)${NC}"
    echo -e "${RED}Lütfen sunucu loglarını kontrol edin: 'docker compose logs -f'${NC}"
    exit 1
fi

HEALTH_STATUS=$(curl -s --connect-timeout 5 https://kepce.org/api/v1/system/health)

# Webapp (SSR) kontrolu: ana sayfa 200 donmeli
WEB_CODE=$(curl -s -o /dev/null -w "%{http_code}" --connect-timeout 10 https://kepce.org/ || echo "000")
if [ "$WEB_CODE" != "200" ]; then
    echo -e "${RED}${BOLD}Hata: Webapp ana sayfa kontrolu basarisiz! (HTTP $WEB_CODE)${NC}"
    echo -e "${RED}Kontrol: docker compose logs webapp${NC}"
    exit 1
fi

# Soft-404 gerileme testi: bilinmeyen rota GERCEK 404 donmeli
SOFT404_CODE=$(curl -s -o /dev/null -w "%{http_code}" --connect-timeout 10 https://kepce.org/seo-404-dogrulama-testi || echo "000")
if [ "$SOFT404_CODE" = "200" ]; then
    echo -e "${YELLOW}Uyari: Bilinmeyen rota 200 dondu (soft-404 gerilemesi). Caddy config'ini kontrol edin.${NC}"
fi

echo -e "${YELLOW}[5/5] Tamamlandı.${NC}"
echo -e "${GREEN}${BOLD}=====================================================${NC}"
echo -e "${GREEN}${BOLD}     Kepçe Başarıyla Yayına Alındı!                  ${NC}"
echo -e "${GREEN}${BOLD}=====================================================${NC}"
echo -e "Site Adresi:        https://kepce.org"
if [[ "$OPT_UMAMI" =~ ^[EeYy]$ ]]; then
    echo -e "Analitik Paneli:    https://analitik.kepce.org"
fi
echo -e "Sağlık Çıktısı:     $HEALTH_STATUS"
echo ""
