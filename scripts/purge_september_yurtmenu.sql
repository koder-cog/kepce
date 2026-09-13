-- scripts/purge_september_yurtmenu.sql
-- Eylül 2026 döneminde yurtmenu.net kaynaklı eklenmiş olan tüm menüleri,
-- ilişkili alt kayıtları (CASCADE) ve menu_history alternatiflerini temizler.

BEGIN;

-- 1. Menü geçmişindeki (alternatif görünümde listelenen) yurtmenu kayıtlarını sil
DELETE FROM menu_history
WHERE serve_date >= '2026-09-01'
  AND serve_date <= '2026-09-30'
  AND source_type ILIKE '%yurtmenu%';

-- 2. Menüleri sil (menu_dishes, votes vb. ON DELETE CASCADE ile temizlenir)
DELETE FROM menus
WHERE serve_date >= '2026-09-01'
  AND serve_date <= '2026-09-30'
  AND source_type ILIKE '%yurtmenu%';

COMMIT;
