-- 02_pricing_2025_2026.sql
-- İstanbul 2025-2026 Dönemi Fiyatlandırma Seed Verileri

BEGIN;

INSERT INTO pricing_periods (city_slug, period_start, period_end)
VALUES ('istanbul', '2025-09-01', '2026-08-31')
ON CONFLICT (city_slug, period_start, period_end) DO NOTHING;

DO $$
DECLARE
    v_period_id INT;
BEGIN
    SELECT id INTO v_period_id 
    FROM pricing_periods 
    WHERE city_slug = 'istanbul' AND period_start = '2025-09-01' AND period_end = '2026-08-31';

    IF v_period_id IS NOT NULL THEN
        -- Breakfast items & categories
        INSERT INTO meal_category_prices (pricing_period_id, meal_type, category_name, portion_amount, price) VALUES
        (v_period_id, 'breakfast', 'ZEYTİN', '30 g', 10.00),
        (v_period_id, 'breakfast', 'BEYAZ PEYNİR', '40 g', 13.00),
        (v_period_id, 'breakfast', 'KAŞAR PEYNİRİ', '40 g', 17.00),
        (v_period_id, 'breakfast', 'HAŞLANMIŞ YUMURTA', '1 Adet', 11.00),
        (v_period_id, 'breakfast', 'KARIŞIK TOST', '50 g', 47.00),
        (v_period_id, 'breakfast', 'ÇORBA ÇEŞİTLERİ', '250 g', 30.00)
        ON CONFLICT (pricing_period_id, meal_type, category_name) DO UPDATE 
        SET portion_amount = EXCLUDED.portion_amount, price = EXCLUDED.price;

        -- Lunch categories
        INSERT INTO meal_category_prices (pricing_period_id, meal_type, category_name, portion_amount, price) VALUES
        (v_period_id, 'lunch', 'ÇORBA ÇEŞİTLERİ', '250 g', 30.00),
        (v_period_id, 'lunch', 'PİRİNÇ PİLAVI ÇEŞİTLERİ', '150 g', 35.00),
        (v_period_id, 'lunch', 'BULGUR PİLAVI ÇEŞİTLERİ', '200 g', 35.00),
        (v_period_id, 'lunch', 'MAKARNA ÇEŞİTLERİ', '200 g', 35.00),
        (v_period_id, 'lunch', 'ETLİ BAKLAGİLLER', '200 g', 60.00),
        (v_period_id, 'lunch', 'ETSİZ BAKLAGİLLER', '200 g', 53.00),
        (v_period_id, 'lunch', 'KEMİKLİ ET YEMEKLERİ', '250 g', 105.00),
        (v_period_id, 'lunch', 'KEMİKSİZ ET YEMEKLERİ', '250 g', 105.00),
        (v_period_id, 'lunch', 'SULU SALÇALI ETLİ YEMEKLER', '250 g', 80.00),
        (v_period_id, 'lunch', 'SALATA-I', '150 g', 23.00),
        (v_period_id, 'lunch', 'SALATA-II', '100 g', 20.00),
        (v_period_id, 'lunch', 'TATLI ÇEŞİTLERİ', '100 g', 39.00)
        ON CONFLICT (pricing_period_id, meal_type, category_name) DO UPDATE 
        SET portion_amount = EXCLUDED.portion_amount, price = EXCLUDED.price;

        -- Dinner categories
        INSERT INTO meal_category_prices (pricing_period_id, meal_type, category_name, portion_amount, price) VALUES
        (v_period_id, 'dinner', 'ÇORBA ÇEŞİTLERİ', '250 g', 30.00),
        (v_period_id, 'dinner', 'PİRİNÇ PİLAVI ÇEŞİTLERİ', '150 g', 35.00),
        (v_period_id, 'dinner', 'BULGUR PİLAVI ÇEŞİTLERİ', '200 g', 35.00),
        (v_period_id, 'dinner', 'MAKARNA ÇEŞİTLERİ', '200 g', 35.00),
        (v_period_id, 'dinner', 'ETLİ BAKLAGİLLER', '200 g', 60.00),
        (v_period_id, 'dinner', 'ETSİZ BAKLAGİLLER', '200 g', 53.00),
        (v_period_id, 'dinner', 'KEMİKLİ ET YEMEKLERİ', '250 g', 105.00),
        (v_period_id, 'dinner', 'KEMİKSİZ ET YEMEKLERİ', '250 g', 105.00),
        (v_period_id, 'dinner', 'SULU SALÇALI ETLİ YEMEKLER', '250 g', 80.00),
        (v_period_id, 'dinner', 'SALATA-I', '150 g', 23.00),
        (v_period_id, 'dinner', 'SALATA-II', '100 g', 20.00),
        (v_period_id, 'dinner', 'TATLI ÇEŞİTLERİ', '100 g', 39.00)
        ON CONFLICT (pricing_period_id, meal_type, category_name) DO UPDATE 
        SET portion_amount = EXCLUDED.portion_amount, price = EXCLUDED.price;
    END IF;
END$$;

COMMIT;
