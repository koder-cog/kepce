-- 0034_add_price_to_menu_dishes.sql
-- Hibrit fiyatlandırma modeli: Menü satırında özel veya basılı fiyat varsa saklar

ALTER TABLE menu_dishes
ADD COLUMN IF NOT EXISTS price NUMERIC(10, 2) DEFAULT NULL;

COMMENT ON COLUMN menu_dishes.price IS 'Özel veya basılı menü fiyatı. NULL ise dinamik kategori kurallarından hesaplanır.';
