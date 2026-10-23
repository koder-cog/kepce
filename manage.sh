#!/usr/bin/env bash
# ==============================================================================
# KEPÇE - TAM KAPSAMLI SİSTEM YÖNETİM SİHİRBAZI & CLI ARACI
# ==============================================================================

set -o pipefail

# Betiğin bulunduğu kök dizini her koşulda doğru tespit et
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$ROOT_DIR" || exit 1

VERSION="2.0.0"

# ==============================================================================
# RENKLER VE GÖRSEL TANIMLAMALAR
# ==============================================================================
RED='\033[0;31m'
BOLD_RED='\033[1;31m'
GREEN='\033[0;32m'
BOLD_GREEN='\033[1;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
BOLD_BLUE='\033[1;34m'
MAGENTA='\033[0;35m'
CYAN='\033[0;36m'
BOLD_CYAN='\033[1;36m'
WHITE='\033[1;37m'
GRAY='\033[0;90m'
NC='\033[0m' # No Color

# ==============================================================================
# ORTAM DEĞİŞKENLERİNİ YÜKLEME (.env)
# ==============================================================================
ENV_FILE=""
load_env() {
    if [ -f "$ROOT_DIR/.env" ]; then
        ENV_FILE="$ROOT_DIR/.env"
        set -a
        # shellcheck disable=SC1091
        source "$ROOT_DIR/.env"
        set +a
    elif [ -f "$ROOT_DIR/../.env" ]; then
        ENV_FILE="$ROOT_DIR/../.env"
        set -a
        # shellcheck disable=SC1091
        source "$ROOT_DIR/../.env"
        set +a
    fi

    # Yerel geliştirme için varsayılan (fallback) değerler
    export POSTGRES_USER="${POSTGRES_USER:-kepce_admin}"
    export POSTGRES_PASSWORD="${POSTGRES_PASSWORD:-SIFRE}"
    export POSTGRES_DB="${POSTGRES_DB:-kepce}"
    export POSTGRES_PORT="${POSTGRES_PORT:-5432}"
    export DATABASE_URL="${DATABASE_URL:-postgresql://${POSTGRES_USER}:${POSTGRES_PASSWORD}@127.0.0.1:${POSTGRES_PORT}/${POSTGRES_DB}}"
    export KEPCE_SECRET_KEY="${KEPCE_SECRET_KEY:-0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef}"
    export RUST_LOG="${RUST_LOG:-info}"
}

load_env

# ==============================================================================
# YAPILANDIRMA SABİTLERİ
# ==============================================================================
API_DIR="api"
WEB_DIR="webapp"
WORKER_DIR="worker"

API_LOG="log/server.log"
WEB_LOG="log/dev.log"
WORKER_LOG="log/worker.log"

API_PID_FILE="server.pid"
WEB_PID_FILE="dev.pid"
WORKER_PID_FILE="worker.pid"

API_PORT="${API_PORT:-8000}"
WEB_DEV_PORT="${WEB_DEV_PORT:-5173}"
WEB_PREVIEW_PORT="${WEB_PREVIEW_PORT:-4173}"

DB_CONTAINER="kepce-db"
SEARXNG_CONTAINER="kepce-searxng"
SEARXNG_PORT="${SEARXNG_PORT:-8080}"
SEARXNG_DIR="${SEARXNG_CONFIG_DIR:-config/searxng}"

BACKUP_DIR="${BACKUP_DIR:-backups}"

# Container motoru tespiti (Podman öncelikli, yoksa Docker)
CONTAINER_ENGINE="podman"
if ! command -v podman >/dev/null 2>&1; then
    if command -v docker >/dev/null 2>&1; then
        CONTAINER_ENGINE="docker"
    fi
fi

# ==============================================================================
# YARDIMCI VE LOGLAMA FONKSİYONLARI
# ==============================================================================
log_info()  { echo -e "${BLUE}[BİLGİ]${NC} $1"; }
log_ok()    { echo -e "${GREEN}[BAŞARILI]${NC} $1"; }
log_warn()  { echo -e "${YELLOW}[UYARI]${NC} $1"; }
log_err()   { echo -e "${RED}[HATA]${NC} $1"; }
log_step()  { echo -e "${BOLD_CYAN}==> $1${NC}"; }

print_banner() {
    echo -e "${BOLD_CYAN}"
    echo "  ██╗  ██╗███████╗██████╗  ██████╗███████╗"
    echo "  ██║ ██╔╝██╔════╝██╔══██╗██╔════╝██╔════╝"
    echo "  █████╔╝ █████╗  ██████╔╝██║     █████╗  "
    echo "  ██╔═██╗ ██╔══╝  ██╔═══╝ ██║     ██╔══╝  "
    echo "  ██║  ██╗███████╗██║     ╚██████╗███████╗"
    echo "  ╚═╝  ╚═╝╚══════╝╚═╝      ╚═════╝╚══════╝"
    echo "        ╚══╝                            "
    echo -e "  ${WHITE}Tam Kapsamlı Sistem Yönetim Sihirbazı v${VERSION}${NC}"
    echo -e "${GRAY}  Dizin: ${ROOT_DIR} | Motor: ${CONTAINER_ENGINE} | Mimari: $(uname -m)${NC}"
    echo -e "${GRAY}──────────────────────────────────────────────────────────────────────${NC}"
}

confirm_action() {
    local prompt="${1:-Devam etmek istiyor musunuz?}"
    read -r -p "$(echo -e "${YELLOW}? ${prompt} [e/H]: ${NC}")" response
    case "$response" in
        [eE][vV][eE][tT]|[eE]|[yY][eE][sS]|[yY]) return 0 ;;
        *) return 1 ;;
    esac
}

pause_key() {
    echo ""
    read -r -p "$(echo -e "${GRAY}Devam etmek için [ENTER] tuşuna basın...${NC}")" _
}

# Port temizleme yardımcı fonksiyonu (fuser veya lsof destekli)
kill_port() {
    local port="$1"
    if command -v fuser >/dev/null 2>&1; then
        fuser -k "${port}/tcp" >/dev/null 2>&1 || true
    elif command -v lsof >/dev/null 2>&1; then
        local pids
        pids=$(lsof -t -i:"$port" 2>/dev/null || true)
        if [ -n "$pids" ]; then
            kill -9 $pids >/dev/null 2>&1 || true
        fi
    fi
}

# Port dinleniyor mu kontrolü
is_port_open() {
    local port="$1"
    if command -v ss >/dev/null 2>&1; then
        ss -tln | grep -qE ":${port}[[:space:]]"
    elif command -v lsof >/dev/null 2>&1; then
        lsof -iTCP:"$port" -sTCP:LISTEN >/dev/null 2>&1
    else
        (echo >/dev/tcp/127.0.0.1/"$port") >/dev/null 2>&1
    fi
}

# Süreçten bellek ve çalışma süresi bilgisi al
get_proc_stats() {
    local pid="$1"
    if [ -n "$pid" ] && kill -0 "$pid" 2>/dev/null; then
        local rss cpu etime mem_mb
        read -r rss cpu etime < <(ps -p "$pid" -o rss=,%cpu=,etime= 2>/dev/null || echo "0 0.0 00:00")
        mem_mb=$(( rss / 1024 ))
        echo "${mem_mb}MB | CPU: ${cpu}% | Süre: ${etime}"
    else
        echo "-"
    fi
}

# Güvenli süreç durdurma (Önce SIGTERM, 10sn sonra SIGKILL)
graceful_kill() {
    local pid="$1"
    local name="$2"
    local timeout="${3:-10}"

    if [ -z "$pid" ] || ! kill -0 "$pid" 2>/dev/null; then
        return 0
    fi

    kill "$pid" 2>/dev/null || true
    local count=0
    local max_ticks=$(( timeout * 2 ))

    while kill -0 "$pid" 2>/dev/null; do
        sleep 0.5
        count=$(( count + 1 ))
        if [ "$count" -ge "$max_ticks" ]; then
            log_warn "$name ($pid) $timeout saniye içinde kapanmadı, SIGKILL (-9) gönderiliyor..."
            kill -9 "$pid" 2>/dev/null || true
            sleep 0.5
            break
        fi
    done
}

# Container çalışıyor mu?
is_container_running() {
    local name="$1"
    "$CONTAINER_ENGINE" ps --filter "name=^${name}$" --filter "status=running" --format "{{.Names}}" 2>/dev/null | grep -q "^${name}$"
}

# Container var mı (durdurulmuş dahil)?
container_exists() {
    local name="$1"
    "$CONTAINER_ENGINE" ps -a --filter "name=^${name}$" --format "{{.Names}}" 2>/dev/null | grep -q "^${name}$"
}

# ==============================================================================
# 1. SERVİS YÖNETİM FONKSİYONLARI (DB, SEARXNG, API, WORKER, WEB)
# ==============================================================================

# --- SEARXNG ---
generate_searxng_config() {
    mkdir -p "$ROOT_DIR/$SEARXNG_DIR"
    log_info "Varsayılan SearXNG settings.yml oluşturuluyor ($SEARXNG_DIR/settings.yml)..."
    cat << 'EOF' > "$ROOT_DIR/$SEARXNG_DIR/settings.yml"
use_default_settings: true
general:
  debug: false
  instance_name: "Kepçe Ara"
server:
  port: 8080
  bind_address: "0.0.0.0"
  secret_key: "kepce_searxng_secret_development_key_1234567890"
  image_proxy: true
search:
  safe_search: 0
  autocomplete: "duckduckgo"
  autocomplete_min: 2
  default_lang: "auto"
  request_timeout: 5.0
  formats:
    - html
    - json
outgoing:
  request_timeout: 5.0
  max_request_timeout: 8.0
  pool_connections: 100
  pool_maxsize: 20
  enable_http2: true
engines:
  - name: duckduckgo
    engine: duckduckgo
    shortcut: ddg
    disabled: false
    weight: 2
  - name: brave
    engine: brave
    shortcut: br
    disabled: false
    weight: 2
  - name: google
    engine: google
    shortcut: g
    disabled: false
    weight: 1
  - name: bing
    engine: bing
    shortcut: b
    disabled: false
    weight: 1
  - name: qwant
    engine: qwant
    shortcut: qw
    disabled: false
    weight: 1
  - name: wikipedia
    engine: wikipedia
    shortcut: w
    disabled: false
    weight: 1
EOF
}

start_searxng() {
    log_step "SearXNG Başlatılıyor ($SEARXNG_CONTAINER - Port: $SEARXNG_PORT)..."
    if is_container_running "$SEARXNG_CONTAINER"; then
        log_warn "SearXNG zaten çalışıyor."
    else
        mkdir -p "$ROOT_DIR/$SEARXNG_DIR"
        if [ ! -f "$ROOT_DIR/$SEARXNG_DIR/settings.yml" ]; then
            generate_searxng_config
        fi

        if container_exists "$SEARXNG_CONTAINER"; then
            "$CONTAINER_ENGINE" start "$SEARXNG_CONTAINER" >/dev/null
        else
            "$CONTAINER_ENGINE" run --name "$SEARXNG_CONTAINER" \
                -p "$SEARXNG_PORT:8080" \
                -v "$ROOT_DIR/$SEARXNG_DIR:/etc/searxng:Z" \
                -d docker.io/searxng/searxng:latest >/dev/null
        fi
        log_ok "SearXNG başlatıldı: http://localhost:$SEARXNG_PORT"
    fi
}

stop_searxng() {
    log_step "SearXNG Durduruluyor ($SEARXNG_CONTAINER)..."
    if is_container_running "$SEARXNG_CONTAINER"; then
        "$CONTAINER_ENGINE" stop "$SEARXNG_CONTAINER" >/dev/null
        log_ok "SearXNG durduruldu."
    else
        log_warn "SearXNG zaten çalışmıyor."
    fi
}

# --- DATABASE (PostgreSQL) ---
start_db() {
    log_step "Veritabanı Başlatılıyor ($DB_CONTAINER - Port: $POSTGRES_PORT)..."
    load_env
    local db_user="${POSTGRES_USER:-kepce_admin}"
    local db_pass="${POSTGRES_PASSWORD:-SIFRE}"
    local db_name="${POSTGRES_DB:-kepce}"

    if is_container_running "$DB_CONTAINER"; then
        log_warn "Veritabanı zaten çalışıyor."
    else
        if container_exists "$DB_CONTAINER"; then
            "$CONTAINER_ENGINE" start "$DB_CONTAINER" >/dev/null
        else
            log_warn "'$DB_CONTAINER' container'ı bulunamadı. Yeni oluşturuluyor..."
            "$CONTAINER_ENGINE" run --name "$DB_CONTAINER" \
                -e POSTGRES_USER="$db_user" \
                -e POSTGRES_PASSWORD="$db_pass" \
                -e POSTGRES_DB="$db_name" \
                -p "${POSTGRES_PORT}:5432" \
                -v kepce-pgdata:/var/lib/postgresql/data \
                -d docker.io/library/postgres:15-alpine >/dev/null
        fi

        log_info "Veritabanının bağlantıya hazır olması bekleniyor..."
        local attempts=0
        until "$CONTAINER_ENGINE" exec "$DB_CONTAINER" pg_isready -U "$db_user" -d "$db_name" >/dev/null 2>&1 || [ $attempts -eq 20 ]; do
            sleep 1
            attempts=$((attempts + 1))
        done

        if [ $attempts -eq 20 ]; then
            log_err "Uyarı: Veritabanı başlatma zaman aşımına uğradı (20sn). API bağlanamayabilir."
            return 1
        else
            log_ok "Veritabanı hazır (PostgreSQL - $db_name)."
        fi
    fi
}

stop_db() {
    log_step "Veritabanı Durduruluyor ($DB_CONTAINER)..."
    if is_container_running "$DB_CONTAINER"; then
        "$CONTAINER_ENGINE" stop "$DB_CONTAINER" >/dev/null
        log_ok "Veritabanı durduruldu."
    else
        log_warn "Veritabanı zaten çalışmıyor."
    fi
}

# --- RUST API ---
start_api() {
    local skip_build="${1:-false}"
    start_db || return 1

    local pid_path="$ROOT_DIR/$API_DIR/$API_PID_FILE"
    if [ -f "$pid_path" ] && kill -0 "$(cat "$pid_path")" 2>/dev/null; then
        log_warn "API zaten çalışıyor (PID: $(cat "$pid_path"))"
        return 0
    fi

    # Port 8000 ve eski API hayalet süreçlerini temizle
    kill_port "$API_PORT"
    pkill -f "target/release/api" 2>/dev/null || true
    pkill -f "target/debug/api" 2>/dev/null || true
    sleep 0.5

    log_step "API Başlatılıyor..."
    mkdir -p "$ROOT_DIR/$API_DIR/log"

    if [ "$skip_build" != "--no-build" ]; then
        log_info "Rust API derleniyor (cargo build --release -p api)..."
        if ! (cd "$ROOT_DIR" && cargo build --release -p api); then
            log_err "API derleme hatası! Eski binary ile başlatılmayacak."
            return 1
        fi
    elif [ ! -x "$ROOT_DIR/target/release/api" ]; then
        log_err "target/release/api bulunamadı! Önce derleme yapmalısınız."
        return 1
    fi

    export RUST_LOG="${RUST_LOG:-info}"
    nohup "$ROOT_DIR/target/release/api" > "$ROOT_DIR/$API_DIR/$API_LOG" 2>&1 &
    local api_pid=$!
    echo "$api_pid" > "$pid_path"
    disown "$api_pid" 2>/dev/null || true

    sleep 1
    if kill -0 "$api_pid" 2>/dev/null; then
        log_ok "API başlatıldı (PID: $api_pid, Log: $API_DIR/$API_LOG)"
    else
        log_err "API başlatıldıktan hemen sonra kapandı! Son log satırları:"
        tail -n 15 "$ROOT_DIR/$API_DIR/$API_LOG" 2>/dev/null || true
        return 1
    fi
}

stop_api() {
    log_step "API Durduruluyor..."
    local pid_path="$ROOT_DIR/$API_DIR/$API_PID_FILE"
    if [ -f "$pid_path" ]; then
        local pid
        pid=$(cat "$pid_path")
        if kill -0 "$pid" 2>/dev/null; then
            graceful_kill "$pid" "API" 10
            log_ok "API durduruldu."
        else
            log_warn "API çalışmıyor ancak PID dosyası mevcuttu (temizlendi)."
        fi
        rm -f "$pid_path"
    else
        log_warn "API çalışmıyor."
    fi

    # Zorla port/PID temizliği (Ghost process'leri engellemek için)
    pkill -f "target/release/api" 2>/dev/null || true
    pkill -f "target/debug/api" 2>/dev/null || true
    kill_port "$API_PORT"
}

# --- RUST WORKER ---
start_worker() {
    local skip_build="${1:-false}"
    start_db || return 1

    local pid_path="$ROOT_DIR/$WORKER_DIR/$WORKER_PID_FILE"
    log_step "Worker Başlatılıyor..."
    if [ -f "$pid_path" ] && kill -0 "$(cat "$pid_path")" 2>/dev/null; then
        log_warn "Worker zaten çalışıyor (PID: $(cat "$pid_path"))"
        return 0
    fi

    # Eski worker hayalet süreçlerini temizle
    pkill -f "target/release/worker" 2>/dev/null || true
    pkill -f "target/debug/worker" 2>/dev/null || true
    sleep 0.5

    mkdir -p "$ROOT_DIR/$WORKER_DIR/log"
    if [ "$skip_build" != "--no-build" ]; then
        log_info "Rust Worker derleniyor (cargo build --release -p worker)..."
        if ! (cd "$ROOT_DIR" && cargo build --release -p worker); then
            log_err "Worker derleme hatası! Eski binary ile başlatılmayacak."
            return 1
        fi
    elif [ ! -x "$ROOT_DIR/target/release/worker" ]; then
        log_err "target/release/worker bulunamadı! Önce derleme yapmalısınız."
        return 1
    fi

    export RUST_LOG="${RUST_LOG:-info}"
    nohup "$ROOT_DIR/target/release/worker" > "$ROOT_DIR/$WORKER_DIR/$WORKER_LOG" 2>&1 &
    local worker_pid=$!
    echo "$worker_pid" > "$pid_path"
    disown "$worker_pid" 2>/dev/null || true

    sleep 1
    if kill -0 "$worker_pid" 2>/dev/null; then
        log_ok "Worker başlatıldı (PID: $worker_pid, Log: $WORKER_DIR/$WORKER_LOG)"
    else
        log_err "Worker başlatıldıktan hemen sonra kapandı! Son log satırları:"
        tail -n 15 "$ROOT_DIR/$WORKER_DIR/$WORKER_LOG" 2>/dev/null || true
        return 1
    fi
}

stop_worker() {
    log_step "Worker Durduruluyor..."
    local pid_path="$ROOT_DIR/$WORKER_DIR/$WORKER_PID_FILE"
    if [ -f "$pid_path" ]; then
        local pid
        pid=$(cat "$pid_path")
        if kill -0 "$pid" 2>/dev/null; then
            graceful_kill "$pid" "Worker" 10
            log_ok "Worker durduruldu."
        else
            log_warn "Worker çalışmıyor ancak PID dosyası mevcuttu (temizlendi)."
        fi
        rm -f "$pid_path"
    else
        log_warn "Worker çalışmıyor."
    fi

    pkill -f "target/release/worker" 2>/dev/null || true
    pkill -f "target/debug/worker" 2>/dev/null || true
}

# --- WEBAPP (PRODUCTION / DEV / PREVIEW) ---
ensure_web_deps() {
    if [ ! -d "$ROOT_DIR/$WEB_DIR/node_modules" ]; then
        log_warn "node_modules bulunamadı. 'npm install' çalıştırılıyor..."
        (cd "$ROOT_DIR/$WEB_DIR" && npm install) || return 1
    fi
}

start_web() {
    log_step "Webapp Production Derlemesi (Nginx) Başlatılıyor..."
    ensure_web_deps || return 1

    if ! (cd "$ROOT_DIR/$WEB_DIR" && npm run build); then
        log_err "Webapp build hatası! Deploy iptal edildi."
        return 1
    fi

    log_info "Build başarılı. Nginx yapılandırması test ediliyor..."
    if ! sudo nginx -t; then
        log_err "Nginx yapılandırma hatası! Reload iptal edildi."
        return 1
    fi
    if ! sudo systemctl reload nginx; then
        log_err "Nginx reload hatası! Nginx servisinin aktif (running) olduğundan emin olun."
        return 1
    fi
    log_ok "Webapp production'a alındı (Nginx reload tamamlandı)."
}

start_web_dev() {
    log_step "Webapp (DEV MODE - Port: $WEB_DEV_PORT) Başlatılıyor..."
    local pid_path="$ROOT_DIR/$WEB_DIR/$WEB_PID_FILE"
    if [ -f "$pid_path" ] && kill -0 "$(cat "$pid_path")" 2>/dev/null; then
        log_warn "Webapp (dev) zaten çalışıyor (PID: $(cat "$pid_path"))"
        return 0
    fi

    mkdir -p "$ROOT_DIR/$WEB_DIR/log"
    ensure_web_deps || return 1

    (
        cd "$ROOT_DIR/$WEB_DIR" || exit 1
        nohup npm run dev -- --host 0.0.0.0 --port "$WEB_DEV_PORT" --strictPort > "$WEB_LOG" 2>&1 &
        local web_pid=$!
        echo "$web_pid" > "$WEB_PID_FILE"
        disown "$web_pid" 2>/dev/null || true
    )
    log_ok "Webapp (dev) başlatıldı (PID: $(cat "$pid_path")) -> http://localhost:$WEB_DEV_PORT"
}

start_web_preview() {
    log_step "Webapp (PREVIEW NODE SSR MODE - Port: $WEB_PREVIEW_PORT) Başlatılıyor..."
    local pid_path="$ROOT_DIR/$WEB_DIR/$WEB_PID_FILE"
    if [ -f "$pid_path" ] && kill -0 "$(cat "$pid_path")" 2>/dev/null; then
        log_warn "Webapp (preview) zaten çalışıyor (PID: $(cat "$pid_path"))"
        return 0
    fi

    mkdir -p "$ROOT_DIR/$WEB_DIR/log"
    ensure_web_deps || return 1

    log_info "Önizleme için Webapp derleniyor..."
    if ! (cd "$ROOT_DIR/$WEB_DIR" && npm run build); then
        log_err "Webapp build başarısız oldu!"
        return 1
    fi

    log_info "Node SSR sunucusu port $WEB_PREVIEW_PORT üzerinde başlatılıyor..."
    (
        cd "$ROOT_DIR/$WEB_DIR" || exit 1
        export PORT="$WEB_PREVIEW_PORT"
        export ORIGIN="http://localhost:$WEB_PREVIEW_PORT"
        nohup node build/index.js > "$WEB_LOG" 2>&1 &
        local web_pid=$!
        echo "$web_pid" > "$WEB_PID_FILE"
        disown "$web_pid" 2>/dev/null || true
    )
    log_ok "Webapp (preview) başlatıldı (PID: $(cat "$pid_path")) -> http://localhost:$WEB_PREVIEW_PORT"
}

stop_web() {
    log_step "Webapp Durduruluyor..."
    local pid_path="$ROOT_DIR/$WEB_DIR/$WEB_PID_FILE"
    if [ -f "$pid_path" ]; then
        local pid
        pid=$(cat "$pid_path")
        if kill -0 "$pid" 2>/dev/null; then
            graceful_kill "$pid" "Webapp" 8
            log_ok "Webapp durduruldu."
        else
            log_warn "Webapp çalışmıyor ancak PID dosyası mevcuttu (temizlendi)."
        fi
        rm -f "$pid_path"
    else
        log_warn "Webapp (dev/preview) çalışmıyor."
    fi

    pkill -f "vite dev" 2>/dev/null || true
    pkill -f "vite preview" 2>/dev/null || true
    pkill -f "node build/index.js" 2>/dev/null || true
    kill_port "$WEB_DEV_PORT"
    kill_port "$WEB_PREVIEW_PORT"
}

# ==============================================================================
# 2. DERLEME MERKEZİ (BUILD ARM64 / NATIVE / WORKSPACE)
# ==============================================================================
build_arm64() {
    # ONNX Runtime (ort_sys) statik kütüphanesi glibc 2.38+ sembolleri ve
    # libstdc++ istediği için trixie tabanı + g++ ZORUNLUDUR; bookworm'da
    # link hatası verir. Ayrıca binary yalnızca aarch64 makinede anlamlıdır.
    local arch
    arch=$(uname -m)
    if [ "$arch" != "aarch64" ]; then
        log_err "Bu komut yalnızca aarch64 makinede çalışır (Mevcut mimari: $arch)."
        echo -e "${YELLOW}Uzak ARM64 sunucuda derlemek için:${NC}"
        echo "  ssh <sunucu> 'cd ~/kepce && $CONTAINER_ENGINE run --rm -v \$PWD:/app -w /app rust:trixie bash -c \"apt-get update && apt-get install -y pkg-config libssl-dev g++ && cargo build --release -p api -p worker\"'"
        return 1
    fi

    log_step "Container içinde native aarch64 derlemesi yapılıyor (rust:trixie)..."
    "$CONTAINER_ENGINE" run --rm -v "$ROOT_DIR":/app -w /app docker.io/library/rust:trixie bash -c "
        set -euo pipefail
        apt-get update -qq && apt-get install -y -qq pkg-config libssl-dev g++ >/dev/null
        cargo build --release -p api -p worker
        mkdir -p target/aarch64-unknown-linux-gnu/release
        cp target/release/api target/aarch64-unknown-linux-gnu/release/api
        cp target/release/worker target/aarch64-unknown-linux-gnu/release/worker
    " || { log_err "Container derlemesi başarısız oldu."; return 1; }

    if [ ! -x "$ROOT_DIR/target/aarch64-unknown-linux-gnu/release/api" ] || [ ! -x "$ROOT_DIR/target/aarch64-unknown-linux-gnu/release/worker" ]; then
        log_err "Binary'ler üretilemedi."
        return 1
    fi
    log_ok "ARM64 derlemesi tamamlandı: target/aarch64-unknown-linux-gnu/release/{api,worker}"
}

build_target() {
    case "$1" in
        api)
            log_step "Rust API derleniyor (Release)..."
            (cd "$ROOT_DIR" && cargo build --release -p api) && log_ok "API derlendi: target/release/api"
            ;;
        worker)
            log_step "Rust Worker derleniyor (Release)..."
            (cd "$ROOT_DIR" && cargo build --release -p worker) && log_ok "Worker derlendi: target/release/worker"
            ;;
        web)
            log_step "Webapp derleniyor..."
            ensure_web_deps && (cd "$ROOT_DIR/$WEB_DIR" && npm run build) && log_ok "Webapp derlendi."
            ;;
        arm64|native)
            build_arm64
            ;;
        all|"")
            log_step "Tüm bileşenler derleniyor (API + Worker + Web)..."
            (cd "$ROOT_DIR" && cargo build --release -p api -p worker) || return 1
            ensure_web_deps && (cd "$ROOT_DIR/$WEB_DIR" && npm run build) || return 1
            log_ok "Tüm bileşenler başarıyla derlendi!"
            ;;
        clean)
            log_step "Derleme önbelleği (target & webapp/build) temizleniyor..."
            (cd "$ROOT_DIR" && cargo clean)
            rm -rf "$ROOT_DIR/$WEB_DIR/build" "$ROOT_DIR/$WEB_DIR/.svelte-kit"
            log_ok "Derleme çıktıları temizlendi."
            ;;
        *)
            log_err "Kullanım: $0 build {api|worker|web|arm64|all|clean}"
            return 1
            ;;
    esac
}

# ==============================================================================
# 3. DURUM, SAĞLIK KONTROLÜ VE CANLI İZLEME (STATUS & WATCH)
# ==============================================================================
status() {
    echo -e "${BOLD_BLUE}┌────────────────────────────────────────────────────────────────────────────┐${NC}"
    echo -e "${BOLD_BLUE}│                         KEPÇE SİSTEM DURUM PANELİ                          │${NC}"
    echo -e "${BOLD_BLUE}└────────────────────────────────────────────────────────────────────────────┘${NC}"

    # 1. API
    local api_pid_file="$ROOT_DIR/$API_DIR/$API_PID_FILE"
    if [ -f "$api_pid_file" ] && kill -0 "$(cat "$api_pid_file")" 2>/dev/null; then
        local apid
        apid=$(cat "$api_pid_file")
        local astats
        astats=$(get_proc_stats "$apid")
        local ahealth="${YELLOW}PORT KAPALI${NC}"
        if is_port_open "$API_PORT"; then
            ahealth="${GREEN}HTTP :${API_PORT} AKTİF${NC}"
        fi
        echo -e "  API      : ${BOLD_GREEN}RUNNING${NC} (PID: $apid) [$ahealth] ${GRAY}($astats)${NC}"
    else
        echo -e "  API      : ${BOLD_RED}STOPPED${NC}"
    fi

    # 2. Worker
    local worker_pid_file="$ROOT_DIR/$WORKER_DIR/$WORKER_PID_FILE"
    if [ -f "$worker_pid_file" ] && kill -0 "$(cat "$worker_pid_file")" 2>/dev/null; then
        local wpid
        wpid=$(cat "$worker_pid_file")
        local wstats
        wstats=$(get_proc_stats "$wpid")
        echo -e "  Worker   : ${BOLD_GREEN}RUNNING${NC} (PID: $wpid) ${GRAY}($wstats)${NC}"
    else
        echo -e "  Worker   : ${BOLD_RED}STOPPED${NC}"
    fi

    # 3. Web (Dev / Preview) & Nginx
    local web_pid_file="$ROOT_DIR/$WEB_DIR/$WEB_PID_FILE"
    local nginx_state=""
    if command -v systemctl >/dev/null 2>&1 && systemctl is-active --quiet nginx 2>/dev/null; then
        nginx_state=" | ${GREEN}Nginx: AKTİF${NC}"
    fi
    if [ -f "$web_pid_file" ] && kill -0 "$(cat "$web_pid_file")" 2>/dev/null; then
        local webpid
        webpid=$(cat "$web_pid_file")
        local webstats
        webstats=$(get_proc_stats "$webpid")
        local webmode="Node/Vite"
        is_port_open "$WEB_DEV_PORT" && webmode="DEV :$WEB_DEV_PORT"
        is_port_open "$WEB_PREVIEW_PORT" && webmode="PREVIEW :$WEB_PREVIEW_PORT"
        echo -e "  Web      : ${BOLD_GREEN}RUNNING${NC} (PID: $webpid, $webmode)${nginx_state} ${GRAY}($webstats)${NC}"
    else
        echo -e "  Web      : ${BOLD_RED}STOPPED${NC} (Dev/Preview kapalı)${nginx_state}"
    fi

    # 4. SearXNG
    if is_container_running "$SEARXNG_CONTAINER"; then
        echo -e "  SearXNG  : ${BOLD_GREEN}RUNNING${NC} ($SEARXNG_CONTAINER, Port: $SEARXNG_PORT)"
    else
        echo -e "  SearXNG  : ${BOLD_RED}STOPPED${NC}"
    fi

    # 5. DB
    if is_container_running "$DB_CONTAINER"; then
        local db_ready="${YELLOW}BAĞLANTI BEKLENİYOR${NC}"
        if "$CONTAINER_ENGINE" exec "$DB_CONTAINER" pg_isready -U "${POSTGRES_USER:-kepce_admin}" -d "${POSTGRES_DB:-kepce}" >/dev/null 2>&1; then
            db_ready="${GREEN}HAZIR (Port: $POSTGRES_PORT)${NC}"
        fi
        echo -e "  DB       : ${BOLD_GREEN}RUNNING${NC} ($DB_CONTAINER) [$db_ready]"
    else
        echo -e "  DB       : ${BOLD_RED}STOPPED${NC}"
    fi
    echo -e "${GRAY}──────────────────────────────────────────────────────────────────────────────${NC}"
}

watch_status() {
    trap 'echo -e "\n${YELLOW}Canlı izleme sonlandırıldı.${NC}"; return 0' INT
    while true; do
        clear
        echo -e "${BOLD_CYAN}KEPÇE CANLI İZLEME MODU${NC} ${GRAY}(Çıkmak için Ctrl+C - Yenileme: 2sn | $(date '+%H:%M:%S'))${NC}"
        status
        echo -e "${BOLD_BLUE}Son Log Satırları (API & Worker):${NC}"
        echo -e "${CYAN}--- API ---${NC}"
        tail -n 4 "$ROOT_DIR/$API_DIR/$API_LOG" 2>/dev/null || echo "Log yok."
        echo -e "${CYAN}--- Worker ---${NC}"
        tail -n 4 "$ROOT_DIR/$WORKER_DIR/$WORKER_LOG" 2>/dev/null || echo "Log yok."
        sleep 2
    done
}

# ==============================================================================
# 4. LOG YÖNETİM MERKEZİ
# ==============================================================================
show_logs() {
    local target="${1:-}"
    local mode="${2:-}"
    local lines="${3:-100}"

    if [ "$mode" = "--errors" ] || [ "$mode" = "-e" ]; then
        log_info "Sadece HATA (ERROR/WARN/PANIC) kayıtları filtreleniyor..."
        case "$target" in
            api)     grep -iE "error|warn|panic|fatal" "$ROOT_DIR/$API_DIR/$API_LOG" | tail -n "$lines" ;;
            worker)  grep -iE "error|warn|panic|fatal" "$ROOT_DIR/$WORKER_DIR/$WORKER_LOG" | tail -n "$lines" ;;
            web)     grep -iE "error|warn|fail" "$ROOT_DIR/$WEB_DIR/$WEB_LOG" | tail -n "$lines" ;;
            *)       grep -iE "error|warn|panic|fatal" "$ROOT_DIR/$API_DIR/$API_LOG" "$ROOT_DIR/$WORKER_DIR/$WORKER_LOG" "$ROOT_DIR/$WEB_DIR/$WEB_LOG" 2>/dev/null | tail -n "$lines" ;;
        esac
        return 0
    fi

    case "$target" in
        api)
            log_info "API logları izleniyor ($API_DIR/$API_LOG) - Çıkış: Ctrl+C"
            tail -n "$lines" -f "$ROOT_DIR/$API_DIR/$API_LOG"
            ;;
        worker)
            log_info "Worker logları izleniyor ($WORKER_DIR/$WORKER_LOG) - Çıkış: Ctrl+C"
            tail -n "$lines" -f "$ROOT_DIR/$WORKER_DIR/$WORKER_LOG"
            ;;
        web)
            log_info "Webapp logları izleniyor ($WEB_DIR/$WEB_LOG) - Çıkış: Ctrl+C"
            tail -n "$lines" -f "$ROOT_DIR/$WEB_DIR/$WEB_LOG"
            ;;
        searxng)
            log_info "SearXNG container logları izleniyor ($SEARXNG_CONTAINER) - Çıkış: Ctrl+C"
            "$CONTAINER_ENGINE" logs -f --tail "$lines" "$SEARXNG_CONTAINER"
            ;;
        db)
            log_info "PostgreSQL container logları izleniyor ($DB_CONTAINER) - Çıkış: Ctrl+C"
            "$CONTAINER_ENGINE" logs -f --tail "$lines" "$DB_CONTAINER"
            ;;
        nginx)
            log_info "Nginx logları izleniyor (/var/log/nginx/*) - Çıkış: Ctrl+C"
            sudo tail -n "$lines" -f /var/log/nginx/error.log /var/log/nginx/access.log
            ;;
        all)
            log_info "Tüm uygulama logları (API + Worker + Web) aynı anda izleniyor - Çıkış: Ctrl+C"
            tail -n 25 -f "$ROOT_DIR/$API_DIR/$API_LOG" "$ROOT_DIR/$WORKER_DIR/$WORKER_LOG" "$ROOT_DIR/$WEB_DIR/$WEB_LOG" 2>/dev/null
            ;;
        clean)
            log_step "Log dosyaları sıfırlanıyor..."
            : > "$ROOT_DIR/$API_DIR/$API_LOG" 2>/dev/null || true
            : > "$ROOT_DIR/$WORKER_DIR/$WORKER_LOG" 2>/dev/null || true
            : > "$ROOT_DIR/$WEB_DIR/$WEB_LOG" 2>/dev/null || true
            log_ok "API, Worker ve Web log dosyalarının içeriği temizlendi."
            ;;
        *)
            echo -e "${YELLOW}Kullanım:${NC} $0 logs {api|worker|web|searxng|db|nginx|all|clean} [--errors]"
            ;;
    esac
}

# ==============================================================================
# 5. VERİTABANI VE YEDEKLEME MERKEZİ
# ==============================================================================
db_shell() {
    start_db || return 1
    log_info "PostgreSQL etkileşimli kabuğuna (psql) bağlanılıyor (Çıkış için \\q yazın)..."
    local tty_flags="-i"
    if [ -t 0 ] && [ -t 1 ]; then
        tty_flags="-it"
    fi
    "$CONTAINER_ENGINE" exec $tty_flags "$DB_CONTAINER" psql -U "${POSTGRES_USER:-kepce_admin}" -d "${POSTGRES_DB:-kepce}"
}

db_stats() {
    start_db || return 1
    log_step "Veritabanı Boyut ve Tablo İstatistikleri ($POSTGRES_DB)"
    "$CONTAINER_ENGINE" exec -i "$DB_CONTAINER" psql -U "${POSTGRES_USER:-kepce_admin}" -d "${POSTGRES_DB:-kepce}" << 'EOF'
SELECT
    pg_database.datname AS "Veritabanı",
    pg_size_pretty(pg_database_size(pg_database.datname)) AS "Toplam Boyut"
FROM pg_database WHERE datname = current_database();

SELECT
    relname AS "Tablo Adı",
    n_live_tup AS "Tahmini Satır Sayısı",
    pg_size_pretty(pg_total_relation_size(relid)) AS "Toplam Boyut (İndeks Dahil)"
FROM pg_stat_user_tables
ORDER BY pg_total_relation_size(relid) DESC
LIMIT 20;
EOF
}

run_backup() {
    if [ -x "$ROOT_DIR/ops/scripts/db/backup_db.sh" ]; then
        "$ROOT_DIR/ops/scripts/db/backup_db.sh" "$@"
    else
        log_warn "./ops/scripts/db/backup_db.sh bulunamadı. Dahili pg_dump yedeklemesi çalıştırılıyor..."
        start_db || return 1
        mkdir -p "$ROOT_DIR/$BACKUP_DIR"
        local ts
        ts=$(date '+%Y%m%d_%H%M%S')
        local out_file="$ROOT_DIR/$BACKUP_DIR/kepce_db_${ts}.sql.gz"
        log_info "Yedek alınıyor: $out_file"
        if "$CONTAINER_ENGINE" exec "$DB_CONTAINER" pg_dump -U "${POSTGRES_USER:-kepce_admin}" "${POSTGRES_DB:-kepce}" | gzip > "$out_file"; then
            log_ok "Veritabanı yedeği başarıyla alındı: $out_file ($(du -h "$out_file" | cut -f1))"
        else
            log_err "Yedekleme başarısız oldu!"
            rm -f "$out_file"
            return 1
        fi
    fi
}

run_restore() {
    local backup_file="${1:-}"
    if [ -z "$backup_file" ]; then
        echo -e "${BOLD_CYAN}Mevcut Yedek Dosyaları:${NC}"
        find "$ROOT_DIR" -maxdepth 3 \( -name "*.sql" -o -name "*.sql.gz" -o -name "*.dump" \) 2>/dev/null | nl
        echo ""
        read -r -p "Geri yüklenecek yedek dosyasının tam yolunu girin: " backup_file
    fi

    if [ ! -f "$backup_file" ]; then
        log_err "Yedek dosyası bulunamadı: $backup_file"
        return 1
    fi

    if ! confirm_action "DİKKAT: '$backup_file' veritabanına (${POSTGRES_DB}) geri yüklenecek. Onaylıyor musunuz?"; then
        log_warn "Geri yükleme iptal edildi."
        return 0
    fi

    start_db || return 1
    log_step "Yedek geri yükleniyor: $backup_file..."
    if [[ "$backup_file" == *.gz ]]; then
        gunzip -c "$backup_file" | "$CONTAINER_ENGINE" exec -i "$DB_CONTAINER" psql -U "${POSTGRES_USER:-kepce_admin}" -d "${POSTGRES_DB:-kepce}"
    else
        "$CONTAINER_ENGINE" exec -i "$DB_CONTAINER" psql -U "${POSTGRES_USER:-kepce_admin}" -d "${POSTGRES_DB:-kepce}" < "$backup_file"
    fi
    log_ok "Veritabanı geri yükleme işlemi tamamlandı."
}

# ==============================================================================
# 6. SİSTEM DOKTORU (DOCTOR) VE .ENV SİHİRBAZI (INIT)
# ==============================================================================
system_doctor() {
    log_step "Kepçe Sistem Doktoru - Ortam ve Bağımlılık Denetimi"
    local issues=0

    # 1. Mimari ve Glibc Kontrolü
    local arch glibc_ver
    arch=$(uname -m)
    glibc_ver=$(ldd --version 2>/dev/null | head -n1 | awk '{print $NF}')
    echo -e "  • Sistem Mimarisi : ${WHITE}$arch${NC} (Glibc: ${glibc_ver:-Bilinmiyor})"

    # 2. Gerekli Araçlar
    local tools=("$CONTAINER_ENGINE" "cargo" "rustc" "node" "npm" "nginx" "curl" "fuser")
    for t in "${tools[@]}"; do
        if command -v "$t" >/dev/null 2>&1; then
            local ver
            ver=$("$t" --version 2>/dev/null | head -n1)
            echo -e "  • $(printf '%-15s' "$t") : ${GREEN}MEVCUT${NC} ${GRAY}($ver)${NC}"
        else
            echo -e "  • $(printf '%-15s' "$t") : ${RED}EKSİK!${NC}"
            issues=$((issues + 1))
        fi
    done

    # 3. .env ve Güvenlik Kontrolü
    if [ -n "$ENV_FILE" ]; then
        echo -e "  • .env Dosyası    : ${GREEN}YÜKLENDİ${NC} ($ENV_FILE)"
        if [ "$KEPCE_SECRET_KEY" = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef" ]; then
            echo -e "  • Secret Key      : ${YELLOW}UYARI (Varsayılan geliştirme anahtarı kullanılıyor!)${NC}"
            issues=$((issues + 1))
        else
            echo -e "  • Secret Key      : ${GREEN}ÖZEL ANAHTAR TANIMLI${NC}"
        fi
        if [ "$POSTGRES_PASSWORD" = "SIFRE" ]; then
            echo -e "  • DB Parolası     : ${YELLOW}UYARI (Varsayılan 'SIFRE' parolası kullanılıyor!)${NC}"
            issues=$((issues + 1))
        else
            echo -e "  • DB Parolası     : ${GREEN}ÖZEL PAROLA TANIMLI${NC}"
        fi
    else
        echo -e "  • .env Dosyası    : ${YELLOW}BULUNAMADI (Fallback değerler aktif, '$0 init' ile oluşturabilirsiniz)${NC}"
        issues=$((issues + 1))
    fi

    # 4. Disk Alanı Kontrolü
    local disk_free
    disk_free=$(df -h "$ROOT_DIR" | awk 'NR==2 {print $4 " boş (" $5 " dolu)"}')
    echo -e "  • Disk Durumu     : ${WHITE}$disk_free${NC}"

    echo -e "${GRAY}──────────────────────────────────────────────────────────────────────────────${NC}"
    if [ "$issues" -eq 0 ]; then
        log_ok "Sistem tamamen sağlıklı! Hiçbir sorun tespit edilmedi."
    else
        log_warn "Toplam $issues uyarı/eksiklik tespit edildi. Yukarıdaki listeyi kontrol edin."
    fi
}

init_env_wizard() {
    log_step ".env Yapılandırma Sihirbazı"
    local target_env="$ROOT_DIR/.env"
    if [ -f "$target_env" ]; then
        if ! confirm_action "$target_env zaten mevcut. Üzerine yazmak (yedek alınarak) ister misiniz?"; then
            log_info "İşlem iptal edildi."
            return 0
        fi
        cp "$target_env" "${target_env}.bak.$(date +%s)"
        log_info "Eski .env dosyası yedeklendi."
    fi

    local rand_secret rand_db_pass
    rand_secret=$(head -c 32 /dev/urandom | od -An -tx1 | tr -d ' \n')
    rand_db_pass=$(head -c 16 /dev/urandom | od -An -tx1 | tr -d ' \n')

    read -r -p "PostgreSQL Kullanıcısı [kepce_admin]: " in_user
    in_user="${in_user:-kepce_admin}"

    read -r -p "PostgreSQL Parolası [Otomatik Üretilsin]: " in_pass
    in_pass="${in_pass:-$rand_db_pass}"

    read -r -p "PostgreSQL Veritabanı Adı [kepce]: " in_db
    in_db="${in_db:-kepce}"

    read -r -p "SearXNG Portu [8080]: " in_sx_port
    in_sx_port="${in_sx_port:-8080}"

    cat << EOF > "$target_env"
# Kepçe Sistem Ortam Değişkenleri (Oluşturulma: $(date))
POSTGRES_USER=${in_user}
POSTGRES_PASSWORD=${in_pass}
POSTGRES_DB=${in_db}
POSTGRES_PORT=5432
DATABASE_URL=postgresql://${in_user}:${in_pass}@127.0.0.1:5432/${in_db}

KEPCE_SECRET_KEY=${rand_secret}
KEPCE_CORS_ORIGIN=http://localhost:5173
COOKIE_SECURE=false
RUST_LOG=info

SEARXNG_PORT=${in_sx_port}
SEARXNG_URL=http://127.0.0.1:${in_sx_port}
EOF
    chmod 600 "$target_env"
    load_env
    log_ok "$target_env dosyası güvenli anahtarlarla oluşturuldu (chmod 600)."
}

# ==============================================================================
# 7. ETKİLEŞİMLİ YÖNETİM SİHİRBAZI (INTERACTIVE TUI MENU)
# ==============================================================================
interactive_wizard() {
    while true; do
        clear
        print_banner
        status
        echo -e "${BOLD_CYAN}  [ SERVİS KONTROLÜ ]                    [ WEB & ÖNYÜZ MODLARI ]${NC}"
        echo -e "   ${WHITE}1)${NC} Tüm Sistemi Başlat (Prod)           ${WHITE}10)${NC} Web Production Deploy (Build + Nginx)"
        echo -e "   ${WHITE}2)${NC} Tüm Sistemi Başlat (Geliştirici)    ${WHITE}11)${NC} Web Dev Sunucusu Başlat (Port $WEB_DEV_PORT)"
        echo -e "   ${WHITE}3)${NC} Tüm Sistemi Durdur                  ${WHITE}12)${NC} Web Preview SSR Başlat (Port $WEB_PREVIEW_PORT)"
        echo -e "   ${WHITE}4)${NC} Tüm Sistemi Yeniden Başlat          ${WHITE}13)${NC} Web Sunucusunu Durdur"
        echo -e "   ${WHITE}5)${NC} Tekil Servis Başlat / Durdur / Seç"
        echo ""
        echo -e "${BOLD_CYAN}  [ LOGLAR & İZLEME ]                    [ DERLEME & DEPLOY ]${NC}"
        echo -e "   ${WHITE}6)${NC} Canlı Gösterge Paneli (Watch)       ${WHITE}14)${NC} Rust API & Worker Derle (Release)"
        echo -e "   ${WHITE}7)${NC} Logları Canlı İzle (Seçmeli)        ${WHITE}15)${NC} Native ARM64 Derle (rust:trixie)"
        echo -e "   ${WHITE}8)${NC} Sadece Hataları Göster (--errors)   ${WHITE}16)${NC} Deploy Betiğini Çalıştır (deploy.sh)"
        echo -e "   ${WHITE}9)${NC} Log Dosyalarını Temizle"
        echo ""
        echo -e "${BOLD_CYAN}  [ VERİTABANI & YEDEKLEME ]             [ SİSTEM & BAKIM ]${NC}"
        echo -e "   ${WHITE}17)${NC} PostgreSQL Kabuğu Aç (psql)        ${WHITE}22)${NC} Sistem Doktoru (Doctor)"
        echo -e "   ${WHITE}18)${NC} Veritabanı Tablo/Boyut Analizi     ${WHITE}23)${NC} .env Yapılandırma Sihirbazı (Init)"
        echo -e "   ${WHITE}19)${NC} Veritabanı Yedeği Al (Backup)      ${WHITE}24)${NC} SearXNG Ayarlarını Sıfırla"
        echo -e "   ${WHITE}20)${NC} Ingest Yedeği Al (Ingest-Backup)   ${WHITE}0)${NC}  ${BOLD_RED}Çıkış${NC}"
        echo -e "   ${WHITE}21)${NC} Yedekten Geri Yükle (Restore)"
        echo -e "${GRAY}──────────────────────────────────────────────────────────────────────────────${NC}"
        read -r -p "$(echo -e "${BOLD_GREEN}Seçiminiz [0-24]: ${NC}")" choice

        case "$choice" in
            1)  start_db; start_searxng; start_api; start_worker; start_web; pause_key ;;
            2)  start_db; start_searxng; start_api; start_worker; start_web_dev; pause_key ;;
            3)  stop_api; stop_worker; stop_web; stop_searxng; stop_db; pause_key ;;
            4)  stop_api; stop_worker; stop_web; stop_searxng; stop_db
                start_db; start_searxng; start_api; start_worker; start_web; pause_key ;;
            5)  submenu_single_service ;;
            6)  watch_status ;;
            7)  submenu_logs ;;
            8)  show_logs all --errors 50; pause_key ;;
            9)  show_logs clean; pause_key ;;
            10) start_web; pause_key ;;
            11) stop_web; start_web_dev; pause_key ;;
            12) stop_web; start_web_preview; pause_key ;;
            13) stop_web; pause_key ;;
            14) build_target api; build_target worker; pause_key ;;
            15) build_arm64; pause_key ;;
            16) ./ops/scripts/deploy/deploy.sh; pause_key ;;
            17) db_shell ;;
            18) db_stats; pause_key ;;
            19) run_backup; pause_key ;;
            20) ./ops/scripts/ingest/backup_ingest.sh; pause_key ;;
            21) run_restore; pause_key ;;
            22) system_doctor; pause_key ;;
            23) init_env_wizard; pause_key ;;
            24) generate_searxng_config; stop_searxng; start_searxng; pause_key ;;
            0|q|Q) echo -e "${GREEN}İyi çalışmalar!${NC}"; exit 0 ;;
            *) log_warn "Geçersiz seçim: $choice"; sleep 1 ;;
        esac
    done
}

submenu_single_service() {
    echo ""
    echo -e "${BOLD_CYAN}Servis Seçin:${NC} 1) API  2) Worker  3) DB (Postgres)  4) SearXNG  5) Web  0) Geri"
    read -r -p "Servis [0-5]: " srv_c
    [ "$srv_c" = "0" ] || [ -z "$srv_c" ] && return 0

    echo -e "${BOLD_CYAN}İşlem Seçin:${NC} 1) Başlat (Start)  2) Durdur (Stop)  3) Yeniden Başlat (Restart)  4) Hızlı Başlat (--no-build)"
    read -r -p "İşlem [1-4]: " act_c

    case "${srv_c}:${act_c}" in
        1:1) start_api ;;
        1:2) stop_api ;;
        1:3) stop_api; start_api ;;
        1:4) stop_api; start_api --no-build ;;
        2:1) start_worker ;;
        2:2) stop_worker ;;
        2:3) stop_worker; start_worker ;;
        2:4) stop_worker; start_worker --no-build ;;
        3:1) start_db ;;
        3:2) stop_db ;;
        3:3) stop_db; start_db ;;
        4:1) start_searxng ;;
        4:2) stop_searxng ;;
        4:3) stop_searxng; start_searxng ;;
        5:1) start_web ;;
        5:2) stop_web ;;
        5:3) stop_web; start_web ;;
        *) log_warn "Geçersiz seçim." ;;
    esac
    pause_key
}

submenu_logs() {
    echo ""
    echo -e "${BOLD_CYAN}İzlenecek Log:${NC} 1) API  2) Worker  3) Web  4) SearXNG  5) DB  6) Nginx  7) Tümü (All)  0) Geri"
    read -r -p "Log Seçimi [0-7]: " log_c
    trap 'return 0' INT
    case "$log_c" in
        1) show_logs api ;;
        2) show_logs worker ;;
        3) show_logs web ;;
        4) show_logs searxng ;;
        5) show_logs db ;;
        6) show_logs nginx ;;
        7) show_logs all ;;
        *) return 0 ;;
    esac
}

# ==============================================================================
# 8. YARDIM EKRANI (HELP)
# ==============================================================================
print_help() {
    print_banner
    cat << EOF
${BOLD_CYAN}KULLANIM:${NC}
  $0                                    Etkileşimli Yönetim Sihirbazını (TUI) açar
  $0 <komut> [hedef] [seçenekler]       Doğrudan CLI komutunu çalıştırır

${BOLD_CYAN}SERVİS KOMUTLARI:${NC}
  start   {api|worker|web|web-dev|web-preview|searxng|db|all|all-dev} [--no-build]
  stop    {api|worker|web|web-dev|web-preview|searxng|db|all}
  restart {api|worker|web|web-dev|web-preview|searxng|db|all|all-dev} [--no-build]

${BOLD_CYAN}İZLEME VE LOG KOMUTLARI:${NC}
  status                                Tüm servislerin PID, RAM, CPU ve Port durumunu gösterir
  watch                                 Canlı yenilenen (2sn) durum ve log paneli açar
  logs    {api|worker|web|searxng|db|nginx|all|clean} [--errors]

${BOLD_CYAN}DERLEME VE DAĞITIM KOMUTLARI:${NC}
  build   {api|worker|web|arm64|all|clean}
  build-arm64 | build-native            Container (rust:trixie) içinde aarch64 derlemesi yapar
  deploy  [argümanlar...]               ./ops/scripts/deploy/deploy.sh betiğini çalıştırır

${BOLD_CYAN}VERİTABANI VE YEDEKLEME KOMUTLARI:${NC}
  db-shell                              PostgreSQL container'ı içinde etkileşimli psql açar
  db-stats                              Veritabanı ve tablo boyut istatistiklerini listeler
  backup  [argümanlar...]               Veritabanı yedeği alır (backup_db.sh veya pg_dump)
  restore [dosya_yolu]                  Yedek dosyasını veritabanına geri yükler
  ingest-backup [argümanlar...]         ./ops/scripts/ingest/backup_ingest.sh çalıştırır

${BOLD_CYAN}SİSTEM VE YAPILANDIRMA:${NC}
  doctor                                Sistem bağımlılıklarını, Glibc, port ve .env durumunu denetler
  init                                  Güvenli anahtarlarla yeni bir .env dosyası oluşturur
  menu | wizard                         Etkileşimli yönetim sihirbazını açar
  help | --help | -h                    Bu yardım mesajını gösterir
EOF
}

# ==============================================================================
# 9. ANA KOMUT YÖNLENDİRİCİ (CLI ENTRYPOINT)
# ==============================================================================
COMMAND="${1:-}"
TARGET="${2:-}"
EXTRA_FLAG="${3:-}"

# Argüman verilmediyse doğrudan etkileşimli sihirbazı başlat (terminal değilse yardımı göster)
if [ -z "$COMMAND" ]; then
    if [ ! -t 0 ]; then
        print_help
        exit 0
    fi
    interactive_wizard
    exit 0
fi

case "$COMMAND" in
    menu|wizard|tui)
        interactive_wizard
        ;;
    start)
        case "$TARGET" in
            api)         start_api "$EXTRA_FLAG" ;;
            worker)      start_worker "$EXTRA_FLAG" ;;
            web)         start_web ;;
            web-dev)     start_web_dev ;;
            web-preview) start_web_preview ;;
            searxng)     start_searxng ;;
            db)          start_db ;;
            all|"")      start_db; start_searxng; start_api "$EXTRA_FLAG"; start_worker "$EXTRA_FLAG"; start_web ;;
            all-dev)     start_db; start_searxng; start_api "$EXTRA_FLAG"; start_worker "$EXTRA_FLAG"; start_web_dev ;;
            *)           log_err "Kullanım: $0 start {api|worker|web|web-dev|web-preview|searxng|db|all|all-dev} [--no-build]"; exit 1 ;;
        esac
        ;;
    stop)
        case "$TARGET" in
            api)                 stop_api ;;
            worker)              stop_worker ;;
            web|web-dev|web-preview) stop_web ;;
            searxng)             stop_searxng ;;
            db)                  stop_db ;;
            all|"")              stop_api; stop_worker; stop_web; stop_searxng; stop_db ;;
            *)                   log_err "Kullanım: $0 stop {api|worker|web|web-dev|web-preview|searxng|db|all}"; exit 1 ;;
        esac
        ;;
    restart)
        case "$TARGET" in
            api)         stop_api; start_api "$EXTRA_FLAG" ;;
            worker)      stop_worker; start_worker "$EXTRA_FLAG" ;;
            web)         stop_web; start_web ;;
            web-dev)     stop_web; start_web_dev ;;
            web-preview) stop_web; start_web_preview ;;
            searxng)     stop_searxng; start_searxng ;;
            db)          stop_db; start_db ;;
            all|"")      stop_api; stop_worker; stop_web; stop_searxng; stop_db
                         start_db; start_searxng; start_api "$EXTRA_FLAG"; start_worker "$EXTRA_FLAG"; start_web ;;
            all-dev)     stop_api; stop_worker; stop_web; stop_searxng; stop_db
                         start_db; start_searxng; start_api "$EXTRA_FLAG"; start_worker "$EXTRA_FLAG"; start_web_dev ;;
            *)           log_err "Kullanım: $0 restart {api|worker|web|web-dev|web-preview|searxng|db|all|all-dev} [--no-build]"; exit 1 ;;
        esac
        ;;
    status)
        status
        ;;
    watch|mon|monitor)
        watch_status
        ;;
    logs|log)
        show_logs "$TARGET" "$EXTRA_FLAG"
        ;;
    build)
        build_target "$TARGET"
        ;;
    build-native|build-arm64)
        build_arm64
        ;;
    db-shell|psql)
        db_shell
        ;;
    db-stats)
        db_stats
        ;;
    backup)
        shift
        run_backup "$@"
        ;;
    restore)
        shift
        run_restore "$@"
        ;;
    ingest-backup)
        shift
        ./ops/scripts/ingest/backup_ingest.sh "$@"
        ;;
    deploy)
        shift
        ./ops/scripts/deploy/deploy.sh "$@"
        ;;
    doctor|check)
        system_doctor
        ;;
    init|setup)
        init_env_wizard
        ;;
    help|--help|-h)
        print_help
        ;;
    *)
        log_err "Bilinmeyen komut: $COMMAND"
        print_help
        exit 1
        ;;
esac

exit 0