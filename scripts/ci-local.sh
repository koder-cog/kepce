#!/bin/bash
# ==============================================================================
# KEPÇE - Yerel CI Simülasyonu
# ==============================================================================
# Push öncesi tüm doğrulama hattını tek komutta çalıştırır:
#   1) Rust: clippy + test
#   2) Webapp: svelte-check + vitest + production build
#   3) Build çıktı doğrulaması
#   4) SSR smoke testi (gerçek node sunucusu üzerinde)
#
# Kullanım: ./scripts/ci-local.sh
# Çıkış kodu: 0 = hepsi geçti, 1 = en az bir adım başarısız
# ==============================================================================

set -uo pipefail

RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'; BLUE='\033[0;34m'; BOLD='\033[1m'; NC='\033[0m'
PASS=0; FAIL=0
RESULTS=()

report() {
    local name="$1" ok="$2" detail="$3"
    if [ "$ok" = "true" ]; then
        PASS=$((PASS+1)); RESULTS+=("PASS | $name | $detail")
        echo -e "${GREEN}[PASS]${NC} $name ${NC}($detail)"
    else
        FAIL=$((FAIL+1)); RESULTS+=("FAIL | $name | $detail")
        echo -e "${RED}[FAIL]${NC} $name ${NC}($detail)"
    fi
}

section() { echo -e "\n${BLUE}${BOLD}=== $1 ===${NC}"; }

# ------------------------------------------------------------------------------
section "1/4 Rust: clippy"
# ------------------------------------------------------------------------------
CLIPPY_OUT=$(cargo clippy --workspace 2>&1)
CLIPPY_WARN=$(echo "$CLIPPY_OUT" | grep -c "^warning" || true)
if echo "$CLIPPY_OUT" | grep -q "^error"; then
    report "cargo clippy" false "$(echo "$CLIPPY_OUT" | grep '^error' | head -3 | tr '\n' ' ')"
else
    report "cargo clippy" true "$CLIPPY_WARN uyarı"
fi

# ------------------------------------------------------------------------------
section "2/4 Rust: test"
# ------------------------------------------------------------------------------
TEST_OUT=$(cargo test --workspace 2>&1)
if echo "$TEST_OUT" | grep -qE "test result: .*FAILED|^error"; then
    report "cargo test" false "$(echo "$TEST_OUT" | grep -E 'FAILED|panicked' | head -3 | tr '\n' ' ')"
else
    TESTED=$(echo "$TEST_OUT" | grep -oE "[0-9]+ passed" | awk '{s+=$1} END {print s+0}')
    report "cargo test" true "${TESTED:-0} test geçti"
fi

# ------------------------------------------------------------------------------
section "3/4 Webapp: check + test + build"
# ------------------------------------------------------------------------------
cd webapp || exit 1

CHECK_OUT=$(npm run check 2>&1)
if echo "$CHECK_OUT" | grep -q "0 errors"; then
    report "svelte-check" true "0 hata"
else
    report "svelte-check" false "$(echo "$CHECK_OUT" | grep -E 'Error|error' | head -3 | tr '\n' ' ')"
fi

VITEST_OUT=$(npm test 2>&1)
# ANSI renk kodlarini temizle; "Tests N passed" satirindan test sayisini al
# (ilk eslesme "Test Files N passed" olabilir, o yuzden Tests satiri hedeflenir)
VT_PASS=$(echo "$VITEST_OUT" | sed 's/\x1b\[[0-9;]*m//g' | grep -E "Tests +[0-9]+ passed" | grep -oE "[0-9]+ passed" | head -1 | tr -dc '0-9')
if echo "$VITEST_OUT" | grep -q "failed"; then
    report "vitest" false "$(echo "$VITEST_OUT" | grep 'failed' | head -2 | tr '\n' ' ')"
else
    report "vitest" true "${VT_PASS:-0} test geçti"
fi

BUILD_OUT=$(npm run build 2>&1)
if [ -f build/index.js ]; then
    report "webapp build" true "build/index.js üretildi"
else
    report "webapp build" false "$(echo "$BUILD_OUT" | tail -5 | tr '\n' ' ')"
fi

cd ..

# ------------------------------------------------------------------------------
section "4/4 SSR smoke testi"
# ------------------------------------------------------------------------------
SMOKE_PORT=3987
API_INTERNAL=http://127.0.0.1:59999 nohup env PORT=$SMOKE_PORT HOST=127.0.0.1 node webapp/build/index.js > /tmp/kepce-ci-smoke.log 2>&1 &
SMOKE_PID=$!
sleep 3

smoke() {
    local path="$1" expect="$2"
    local code
    code=$(curl -s -o /dev/null -w "%{http_code}" --max-time 10 "http://127.0.0.1:$SMOKE_PORT$path" 2>/dev/null || echo "000")
    [ "$code" = "$expect" ]
}

if smoke "/" 200; then report "smoke: ana sayfa" true 200; else report "smoke: ana sayfa" false "$(curl -s -o /dev/null -w '%{http_code}' http://127.0.0.1:$SMOKE_PORT/)"; fi
if smoke "/sss" 200; then report "smoke: statik sayfa" true 200; else report "smoke: statik sayfa" false "-"; fi
if smoke "/olmayan-rota-ci-testi" 404; then report "smoke: gerçek 404" true 404; else report "smoke: gerçek 404" false "-"; fi
if smoke "/nirvana-sehir" 404; then report "smoke: bilinmeyen şehir 404" true 404; else report "smoke: bilinmeyen şehir" false "-"; fi
if smoke "/sitemap.xml" 200; then report "smoke: sitemap index" true 200; else report "smoke: sitemap index" false "-"; fi

kill $SMOKE_PID 2>/dev/null

# ------------------------------------------------------------------------------
section "ÖZET"
# ------------------------------------------------------------------------------
for r in "${RESULTS[@]}"; do echo "  $r"; done
echo ""
if [ "$FAIL" -eq 0 ]; then
    echo -e "${GREEN}${BOLD}CI YEŞİL: $PASS kontrol geçti, 0 başarısız.${NC}"
    exit 0
else
    echo -e "${RED}${BOLD}CI KIRMIZI: $PASS geçti, $FAIL başarısız.${NC}"
    exit 1
fi
