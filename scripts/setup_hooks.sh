#!/bin/bash
# setup_hooks.sh - Links developer Git pre-commit hook

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
# Commitlemeden önce Rust ve Webapp dosyalarını otomatik formatlar.

if command -v cargo >/dev/null 2>&1; then
    cargo fmt --all
fi

if [ -d "webapp" ] && [ -f "webapp/package.json" ]; then
    (cd webapp && npm run format >/dev/null 2>&1 || true)
fi

git update-index --again 2>/dev/null || true

exit 0
EOF

chmod +x "$HOOK_FILE"
echo "Git pre-commit kancası başarıyla güncellendi ve kuruldu!"
