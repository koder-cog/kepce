#!/bin/bash
# ==============================================================================
# KEPÇE - Dish Auto-Categorization Batch Utility
# ==============================================================================
# Applies official ministry category rules to all dishes in the database.
# ==============================================================================

set -euo pipefail

if [ -f ".env" ]; then
    # shellcheck disable=SC2046
    export $(grep -v '^#' .env | xargs -0 -d '\n' 2>/dev/null || grep -v '^#' .env | xargs)
fi

DEPLOY_HOST="${KEPCE_DEPLOY_HOST:-}"
DEPLOY_KEY="${KEPCE_DEPLOY_KEY:-$HOME/.ssh/id_rsa}"

echo "================================================================="
echo " Kepçe - Yemekleri Otomatik Kategorize Etme"
echo "================================================================="

# Create SQL script from rules
python3 - << 'EOF' > /tmp/update_categories.sql
import json

with open("config/pricing/category_rules.json", "r", encoding="utf-8") as f:
    rules = json.load(f)

print("BEGIN;")
for r in rules:
    cat = r["category"].replace("'", "''")
    kws = r.get("keywords", [])
    excs = r.get("excludes", [])
    
    if not kws:
        continue
        
    where_clauses = []
    # Build OR for keywords
    kw_conditions = ["name ILIKE '%" + kw.replace("'", "''") + "%'" for kw in kws]
    where_clauses.append("(" + " OR ".join(kw_conditions) + ")")
    
    # Build NOT AND for excludes
    for exc in excs:
        where_clauses.append("name NOT ILIKE '%" + exc.replace("'", "''") + "%'")
        
    full_where = " AND ".join(where_clauses)
    sql = f"UPDATE dishes SET category = '{cat}' WHERE ({full_where}) AND (category IS NULL OR category = '');"
    print(sql)

print("COMMIT;")
EOF

echo "1. Kurallar SQL betiğine dönüştürüldü ($(wc -l < /tmp/update_categories.sql) satır)."
echo "2. Sunucu veritabanına uygulanıyor..."

ssh -i "$DEPLOY_KEY" "$DEPLOY_HOST" 'cat > /tmp/update_categories.sql' < /tmp/update_categories.sql
ssh -i "$DEPLOY_KEY" "$DEPLOY_HOST" 'docker exec -i kepce-db psql -U kepce_admin -d kepce < /tmp/update_categories.sql && rm /tmp/update_categories.sql'

echo "3. Kategori dağılım istatistikleri:"
ssh -i "$DEPLOY_KEY" "$DEPLOY_HOST" 'docker exec kepce-db psql -U kepce_admin -d kepce -c "SELECT COALESCE(category, '\''[KATEGORİSİZ]'\'') as category, count(*) as count FROM dishes GROUP BY category ORDER BY count DESC LIMIT 25;"'

rm -f /tmp/update_categories.sql
echo "================================================================="
echo " Kategorizasyon başarıyla tamamlandı!"
echo "================================================================="
