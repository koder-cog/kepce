#!/bin/bash
# setup_hooks.sh — Links developer Git pre-commit hook

HOOK_DIR=".git/hooks"
HOOK_FILE="$HOOK_DIR/pre-commit"

if [ ! -d ".git" ]; then
  echo "Bilgi: .git dizini bulunamadı (bu proje git reposu olmayabilir). Kanca kurulumu atlandı."
  exit 0
fi

mkdir -p "$HOOK_DIR"

cat << 'EOF' > "$HOOK_FILE"
#!/bin/bash
# Kepçe Git Pre-Commit Hook
# Prevents commits if formatting, clippy, or Svelte checks fail.

echo "=== Running Pre-Commit Checks ==="

# 1. Cargo Fmt
echo "Checking Rust formatting..."
if ! cargo fmt --all -- --check; then
    echo "Hata: Rust format kontrolü başarısız oldu. Lütfen 'cargo fmt' çalıştırın."
    exit 1
fi

# 2. Cargo Clippy
echo "Checking Rust clippy lints..."
if ! cargo clippy --workspace --all-targets -- -D warnings; then
    echo "Hata: Rust clippy lints başarısız oldu."
    exit 1
fi

# 3. Svelte Check
if [ -d "webapp" ]; then
    echo "Running Svelte check..."
    if ! (cd webapp && npm run check); then
        echo "Hata: Svelte check başarısız oldu."
        exit 1
    fi
fi

echo "=== All Checks Passed ==="
exit 0
EOF

chmod +x "$HOOK_FILE"
echo "Git pre-commit kancası başarıyla kuruldu!"
