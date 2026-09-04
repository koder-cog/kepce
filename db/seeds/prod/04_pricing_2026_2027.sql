-- 04_pricing_2026_2027.sql
-- İstanbul 2026-2027 Dönemi Resmi Fiyatlandırma Seed Verileri
-- Yeni tarife genelgesi yayımlanana kadar 2025-2026 resmi taban tarifesi devredilir.

BEGIN;

INSERT INTO pricing_periods (city_slug, period_start, period_end)
VALUES ('istanbul', '2026-09-01', '2027-08-31')
ON CONFLICT (city_slug, period_start, period_end) DO NOTHING;

DO $$
DECLARE
    v_old_period_id INT;
    v_new_period_id INT;
BEGIN
    SELECT id INTO v_old_period_id 
    FROM pricing_periods 
    WHERE city_slug = 'istanbul' AND period_start = '2025-09-01' AND period_end = '2026-08-31';

    SELECT id INTO v_new_period_id 
    FROM pricing_periods 
    WHERE city_slug = 'istanbul' AND period_start = '2026-09-01' AND period_end = '2027-08-31';

    IF v_new_period_id IS NOT NULL AND v_old_period_id IS NOT NULL THEN
        INSERT INTO meal_category_prices (pricing_period_id, meal_type, category_name, portion_amount, price)
        SELECT v_new_period_id, meal_type, category_name, portion_amount, price
        FROM meal_category_prices
        WHERE pricing_period_id = v_old_period_id
        ON CONFLICT (pricing_period_id, meal_type, category_name) DO NOTHING;
    END IF;
END;
$$;

COMMIT;
