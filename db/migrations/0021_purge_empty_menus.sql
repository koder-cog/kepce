-- Migration 0021: Purge empty menus (menus without any dishes/items)
-- =================================================================
-- Yapısal kalite temizliği: menu_dishes ilişkisi bulunmayan (yemek listesi boş olan)
-- tüm menus kayıtlarını siler. ON DELETE CASCADE sayesinde yetim oylar/yorumlar da temizlenir.

DELETE FROM menus
WHERE id NOT IN (
    SELECT DISTINCT menu_id
    FROM menu_dishes
);
