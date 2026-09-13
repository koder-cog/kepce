-- scripts/quarantine_september_yurtmenu.sql
-- Eylül 2026 döneminde yurtmenu.net kaynaklı eklenmiş olan tüm menüleri
-- karantinaya alır (status = 'pending') ve bunlara bağlı üretilmiş
-- bot yorumlarını sıfırlar.

BEGIN;

UPDATE menus
SET 
    status = 'pending',
    bot_commentary = NULL
WHERE 
    serve_date >= '2026-09-01' 
    AND serve_date <= '2026-09-30'
    AND source_type ILIKE '%yurtmenu%';

COMMIT;
