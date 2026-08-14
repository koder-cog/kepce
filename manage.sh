#!/bin/bash

# Configuration
API_DIR="api"
WEB_DIR="webapp"
WORKER_DIR="worker"
API_LOG="log/server.log"
WEB_LOG="log/dev.log"
WORKER_LOG="log/worker.log"
API_PID_FILE="server.pid"
WEB_PID_FILE="dev.pid"
WORKER_PID_FILE="worker.pid"
DB_CONTAINER="kepce-db"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to start Database
start_db() {
    echo -e "${BLUE}Starting Database ($DB_CONTAINER)...${NC}"
    if podman ps --filter "name=$DB_CONTAINER" --filter "status=running" --format "{{.Names}}" | grep -q "^$DB_CONTAINER$"; then
        echo -e "${YELLOW}Database is already running.${NC}"
    else
        if podman ps -a --filter "name=$DB_CONTAINER" --format "{{.Names}}" | grep -q "^$DB_CONTAINER$"; then
            podman start "$DB_CONTAINER"
        else
            echo -e "${YELLOW}Database container '$DB_CONTAINER' not found. Creating it...${NC}"
            if [ -f ".env" ]; then
                export $(grep -v '^#' .env | xargs)
            elif [ -f "../.env" ]; then
                export $(grep -v '^#' ../.env | xargs)
            fi
            DB_USER="${POSTGRES_USER:-kepce_admin}"
            DB_PASS="${POSTGRES_PASSWORD:-SIFRE}"
            DB_NAME="${POSTGRES_DB:-kepce}"
            podman run --name "$DB_CONTAINER" \
                -e POSTGRES_USER="$DB_USER" \
                -e POSTGRES_PASSWORD="$DB_PASS" \
                -e POSTGRES_DB="$DB_NAME" \
                -p 5432:5432 \
                -v kepce-pgdata:/var/lib/postgresql/data \
                -d postgres:15-alpine
        fi
        
        # Wait for Postgres to be ready
        echo -e "${BLUE}Waiting for Database to be ready...${NC}"
        attempts=0
        if [ -f ".env" ]; then
            export $(grep -v '^#' .env | xargs)
        fi
        DB_USER="${POSTGRES_USER:-kepce_admin}"
        DB_NAME="${POSTGRES_DB:-kepce}"
        until podman exec "$DB_CONTAINER" pg_isready -U "$DB_USER" -d "$DB_NAME" >/dev/null 2>&1 || [ $attempts -eq 15 ]; do
            sleep 1
            attempts=$((attempts + 1))
        done
        
        if [ $attempts -eq 15 ]; then
            echo -e "${RED}Warning: Database startup timeout. API might fail to connect.${NC}"
        else
            echo -e "${GREEN}Database is ready.${NC}"
        fi
    fi
}

# Function to stop Database
stop_db() {
    echo -e "${BLUE}Stopping Database ($DB_CONTAINER)...${NC}"
    if podman ps --filter "name=$DB_CONTAINER" --filter "status=running" --format "{{.Names}}" | grep -q "^$DB_CONTAINER$"; then
        podman stop "$DB_CONTAINER"
        echo -e "${GREEN}Database stopped.${NC}"
    else
        echo -e "${YELLOW}Database is not running.${NC}"
    fi
}

# Function to start API
start_api() {
    start_db || return 1
    # Port 8000 ve eski API hayalet süreçlerini temizle
    fuser -k 8000/tcp 2>/dev/null || true
    pkill -f "./target/release/api" 2>/dev/null || true
    pkill -f "./target/debug/api" 2>/dev/null || true
    sleep 0.5

    echo -e "${BLUE}Starting API...${NC}"
    if [ -f "$API_DIR/$API_PID_FILE" ] && kill -0 $(cat "$API_DIR/$API_PID_FILE") 2>/dev/null; then
        echo -e "${YELLOW}API is already running (PID: $(cat "$API_DIR/$API_PID_FILE"))${NC}"
    else
        mkdir -p "$API_DIR/log"
        echo -e "${BLUE}Compiling Rust API (if needed)...${NC}"
        # Make sure we compile from workspace root
        if ! cargo build --release -p api; then
            echo -e "${RED}API derleme hatası! Eski binary ile başlatılmayacak.${NC}"
            return 1
        fi
        export RUST_LOG="${RUST_LOG:-info}"
        nohup ./target/release/api > "$API_DIR/$API_LOG" 2>&1 &
        echo $! > "$API_DIR/$API_PID_FILE"
        echo -e "${GREEN}API started with PID: $(cat "$API_DIR/$API_PID_FILE")${NC}"
    fi
}

# Function to stop API
stop_api() {
    echo -e "${BLUE}Stopping API...${NC}"
    if [ -f "$API_DIR/$API_PID_FILE" ]; then
        PID=$(cat "$API_DIR/$API_PID_FILE")
        if kill -0 $PID 2>/dev/null; then
            kill $PID
            # Wait for it to stop
            while kill -0 $PID 2>/dev/null; do sleep 0.5; done
            echo -e "${GREEN}API stopped.${NC}"
        else
            echo -e "${YELLOW}API is not running but PID file exists.${NC}"
        fi
        rm "$API_DIR/$API_PID_FILE"
    else
        echo -e "${YELLOW}API is not running.${NC}"
    fi
    
    # Zorla port/PID temizliği (Ghost process'leri engellemek için)
    pkill -f "./target/release/api" 2>/dev/null || true
    pkill -f "./target/debug/api" 2>/dev/null || true
    fuser -k 8000/tcp 2>/dev/null || true
}

# Function to start Worker
start_worker() {
    start_db || return 1
    echo -e "${BLUE}Starting Worker...${NC}"
    if [ -f "$WORKER_DIR/$WORKER_PID_FILE" ] && kill -0 $(cat "$WORKER_DIR/$WORKER_PID_FILE") 2>/dev/null; then
        echo -e "${YELLOW}Worker is already running (PID: $(cat "$WORKER_DIR/$WORKER_PID_FILE"))${NC}"
    else
        mkdir -p "$WORKER_DIR/log"
        echo -e "${BLUE}Compiling Rust Worker (if needed)...${NC}"
        if ! cargo build --release -p worker; then
            echo -e "${RED}Worker derleme hatası! Eski binary ile başlatılmayacak.${NC}"
            return 1
        fi
        nohup ./target/release/worker > "$WORKER_DIR/$WORKER_LOG" 2>&1 &
        echo $! > "$WORKER_DIR/$WORKER_PID_FILE"
        echo -e "${GREEN}Worker started with PID: $(cat "$WORKER_DIR/$WORKER_PID_FILE")${NC}"
    fi
}

# Function to stop Worker
stop_worker() {
    echo -e "${BLUE}Stopping Worker...${NC}"
    if [ -f "$WORKER_DIR/$WORKER_PID_FILE" ]; then
        PID=$(cat "$WORKER_DIR/$WORKER_PID_FILE")
        if kill -0 $PID 2>/dev/null; then
            kill $PID
            while kill -0 $PID 2>/dev/null; do sleep 0.5; done
            echo -e "${GREEN}Worker stopped.${NC}"
        else
            echo -e "${YELLOW}Worker is not running but PID file exists.${NC}"
        fi
        rm "$WORKER_DIR/$WORKER_PID_FILE"
    else
        echo -e "${YELLOW}Worker is not running.${NC}"
    fi

    # Zorla PID temizliği
    pkill -f "./target/release/worker" 2>/dev/null || true
    pkill -f "./target/debug/worker" 2>/dev/null || true
}

# Function to start Webapp (PRODUCTION) - build + nginx reload
start_web() {
    echo -e "${BLUE}Building Webapp for production (Nginx)...${NC}"
    cd "$WEB_DIR"
    if [ ! -d "node_modules" ]; then
        echo -e "${YELLOW}node_modules not found. Running npm install...${NC}"
        npm install
    fi
    if ! npm run build; then
        echo -e "${RED}Webapp build hatası! Deploy iptal.${NC}"
        cd ..
        return 1
    fi
    cd ..
    echo -e "${GREEN}Build başarılı. Nginx yapılandırması test ediliyor...${NC}"
    if ! sudo nginx -t; then
        echo -e "${RED}Nginx yapılandırma hatası! Reload iptal.${NC}"
        return 1
    fi
    if ! sudo systemctl reload nginx; then
        echo -e "${RED}Nginx reload hatası! Nginx servisinin aktif (running) olduğundan emin olun.${NC}"
        return 1
    fi
    echo -e "${GREEN}Webapp production'a alındı (Nginx reload tamamlandı).${NC}"
}

# Function to start Webapp (DEV) - vite dev server
start_web_dev() {
    echo -e "${BLUE}Starting Webapp (DEV MODE)...${NC}"
    if [ -f "$WEB_DIR/$WEB_PID_FILE" ] && kill -0 $(cat "$WEB_DIR/$WEB_PID_FILE") 2>/dev/null; then
        echo -e "${YELLOW}Webapp (dev) is already running (PID: $(cat "$WEB_DIR/$WEB_PID_FILE"))${NC}"
    else
        cd "$WEB_DIR"
        mkdir -p log
        if [ ! -d "node_modules" ]; then
            echo -e "${YELLOW}node_modules not found. Running npm install...${NC}"
            npm install
        fi
        nohup npm run dev -- --host 0.0.0.0 --port 5173 --strictPort > "$WEB_LOG" 2>&1 &
        echo $! > "../$WEB_PID_FILE"
        cd ..
        echo -e "${GREEN}Webapp (dev) started with PID: $(cat "$WEB_DIR/$WEB_PID_FILE")${NC}"
    fi
}

# Function to start Webapp (PREVIEW) - built production version locally
start_web_preview() {
    echo -e "${BLUE}Starting Webapp (PREVIEW MODE)...${NC}"
    if [ -f "$WEB_DIR/$WEB_PID_FILE" ] && kill -0 $(cat "$WEB_DIR/$WEB_PID_FILE") 2>/dev/null; then
        echo -e "${YELLOW}Webapp (preview) is already running (PID: $(cat "$WEB_DIR/$WEB_PID_FILE"))${NC}"
    else
        cd "$WEB_DIR"
        mkdir -p log
        if [ ! -d "node_modules" ]; then
            echo -e "${YELLOW}node_modules not found. Running npm install...${NC}"
            npm install
        fi
        echo -e "${BLUE}Building Webapp for preview...${NC}"
        npm run build
        echo -e "${BLUE}Starting preview server...${NC}"
        nohup npm run preview -- --host 0.0.0.0 --port 5173 --strictPort > "$WEB_LOG" 2>&1 &
        echo $! > "../$WEB_PID_FILE"
        cd ..
        echo -e "${GREEN}Webapp (preview) started with PID: $(cat "$WEB_DIR/$WEB_PID_FILE")${NC}"
    fi
}

# Function to stop Webapp
stop_web() {
    echo -e "${BLUE}Stopping Webapp...${NC}"
    if [ -f "$WEB_DIR/$WEB_PID_FILE" ]; then
        PID=$(cat "$WEB_DIR/$WEB_PID_FILE")
        if kill -0 $PID 2>/dev/null; then
            kill $PID
            while kill -0 $PID 2>/dev/null; do sleep 0.5; done
            echo -e "${GREEN}Webapp stopped.${NC}"
        else
            echo -e "${YELLOW}Webapp is not running but PID file exists.${NC}"
        fi
        rm "$WEB_DIR/$WEB_PID_FILE"
    else
        echo -e "${YELLOW}Webapp is not running.${NC}"
    fi

    # Zorla PID/Port temizliği
    pkill -f "vite dev" 2>/dev/null || true
    pkill -f "vite preview" 2>/dev/null || true
    fuser -k 5173/tcp 2>/dev/null || true
    fuser -k 4173/tcp 2>/dev/null || true
}

# Function to show status
status() {
    echo -e "${BLUE}--- Status ---${NC}"
    
    # API
    if [ -f "$API_DIR/$API_PID_FILE" ] && kill -0 $(cat "$API_DIR/$API_PID_FILE") 2>/dev/null; then
        echo -e "API:    ${GREEN}RUNNING${NC} (PID: $(cat "$API_DIR/$API_PID_FILE"))"
    else
        echo -e "API:    ${RED}STOPPED${NC}"
    fi
    
    # Web
    if [ -f "$WEB_DIR/$WEB_PID_FILE" ] && kill -0 $(cat "$WEB_DIR/$WEB_PID_FILE") 2>/dev/null; then
        echo -e "Web:    ${GREEN}RUNNING${NC} (PID: $(cat "$WEB_DIR/$WEB_PID_FILE"))"
    else
        echo -e "Web:    ${RED}STOPPED${NC}"
    fi

    # Worker
    if [ -f "$WORKER_DIR/$WORKER_PID_FILE" ] && kill -0 $(cat "$WORKER_DIR/$WORKER_PID_FILE") 2>/dev/null; then
        echo -e "Worker: ${GREEN}RUNNING${NC} (PID: $(cat "$WORKER_DIR/$WORKER_PID_FILE"))"
    else
        echo -e "Worker: ${RED}STOPPED${NC}"
    fi

    # DB
    if podman ps --filter "name=$DB_CONTAINER" --filter "status=running" --format "{{.Names}}" | grep -q "^$DB_CONTAINER$"; then
        echo -e "DB:     ${GREEN}RUNNING${NC} ($DB_CONTAINER)"
    else
        echo -e "DB:     ${RED}STOPPED${NC}"
    fi
}

# Function to show logs
show_logs() {
    case "$1" in
        api)
            tail -f "$API_DIR/$API_LOG"
            ;;
        worker)
            tail -f "$WORKER_DIR/$WORKER_LOG"
            ;;
        web)
            tail -f "$WEB_DIR/$WEB_LOG"
            ;;
        db)
            podman logs -f "$DB_CONTAINER"
            ;;
        *)
            echo "Usage: $0 logs {api|worker|web|db}"
            ;;
    esac
}

# Main command handling
COMMAND=$1
TARGET=$2

case "$COMMAND" in
    start)
        case "$TARGET" in
            api) start_api ;;
            worker) start_worker ;;
            web) start_web ;;
            web-dev) start_web_dev ;;
            web-preview) start_web_preview ;;
            db)  start_db ;;
            all|"") start_api; start_worker; start_web ;;
            *) echo "Usage: $0 start {api|worker|web|web-dev|web-preview|db|all}" ;;
        esac
        ;;
    stop)
        case "$TARGET" in
            api) stop_api ;;
            worker) stop_worker ;;
            web) stop_web ;;
            web-dev|web-preview) stop_web ;;
            db)  stop_db ;;
            all|"") stop_api; stop_worker; stop_web; stop_db ;;
            *) echo "Usage: $0 stop {api|worker|web|web-dev|web-preview|db|all}" ;;
        esac
        ;;
    restart)
        case "$TARGET" in
            api) stop_api; start_api ;;
            worker) stop_worker; start_worker ;;
            web) stop_web; start_web ;;
            web-dev) stop_web; start_web_dev ;;
            web-preview) stop_web; start_web_preview ;;
            db)  stop_db; start_db ;;
            all|"") stop_api; stop_worker; stop_web; stop_db; start_api; start_worker; start_web ;;
            *) echo "Usage: $0 restart {api|worker|web|web-dev|web-preview|db|all}" ;;
        esac
        ;;
    status)
        status
        ;;
    logs)
        show_logs "$TARGET"
        ;;
    backup)
        ./scripts/backup_db.sh
        ;;
    ingest-backup)
        shift
        ./scripts/backup_ingest.sh "$@"
        ;;
    deploy)
        shift
        ./scripts/deploy.sh "$@"
        ;;
    *)
        echo "Usage: $0 {start|stop|restart|status|logs|backup|ingest-backup|deploy} [api|worker|web|web-dev|web-preview|db|all]"
        exit 1
        ;;
esac

exit 0
