-- 03_pricing_2026_2027_istanbul.sql
-- İstanbul 2026-2027 Dönemi Resmi Tavan Fiyatlandırma Seed Verileri
-- Kaynak: T.C. Gençlik ve Spor Bakanlığı İstanbul İl Müdürlüğü (2026-2027 Resmi Liste)

BEGIN;

INSERT INTO pricing_periods (city_slug, period_start, period_end)
VALUES ('istanbul', '2026-09-01', '2027-08-31')
ON CONFLICT (city_slug, period_start, period_end) DO NOTHING;

DO $$
DECLARE
    v_period_id INT;
BEGIN
    SELECT id INTO v_period_id 
    FROM pricing_periods 
    WHERE city_slug = 'istanbul' AND period_start = '2026-09-01' AND period_end = '2027-08-31';

    IF v_period_id IS NOT NULL THEN
        -- ==========================================
        -- 1. KAHVALTI GRAMAJ VE FİYAT LİSTESİ
        -- ==========================================
        INSERT INTO meal_category_prices (pricing_period_id, meal_type, category_name, portion_amount, price) VALUES
        (v_period_id, 'breakfast', 'ZEYTİN', '30 g', 11.00),
        (v_period_id, 'breakfast', 'BEYAZ PEYNİR', '40 g', 15.00),
        (v_period_id, 'breakfast', 'KAŞAR PEYNİRİ', '40 g', 20.00),
        (v_period_id, 'breakfast', 'KAŞAR PEYNİR', '40 g', 20.00),
        (v_period_id, 'breakfast', 'YÖRESEL PEYNİRLER', '40 g', 20.00),
        (v_period_id, 'breakfast', 'KREM PEYNİR', '20 g', 10.00),
        (v_period_id, 'breakfast', 'ÜÇGEN PEYNİR', '12.5 g', 8.00),
        (v_period_id, 'breakfast', 'LABNE PEYNİR', '20 g', 10.00),
        (v_period_id, 'breakfast', 'HAŞLANMIŞ YUMURTA', '1 Adet L Boyutunda', 13.00),
        (v_period_id, 'breakfast', 'SALAM (PİLİÇ)', '35 g', 11.00),
        (v_period_id, 'breakfast', 'SÜRÜLEBİLİR ÇİKOLATA', '20 g', 9.00),
        (v_period_id, 'breakfast', 'PİKNİK BAL', '20 g', 10.00),
        (v_period_id, 'breakfast', 'PİKNİK REÇEL', '20 g', 6.00),
        (v_period_id, 'breakfast', 'PİKNİK TEREYAĞI', '10 g', 10.00),
        (v_period_id, 'breakfast', 'PİKNİK HELVA', '40 g', 11.00),
        (v_period_id, 'breakfast', 'TAHİNLİ PEKMEZ', '20 g', 9.00),
        (v_period_id, 'breakfast', 'KAŞARLI TOST', '100 g Kaşar', 65.00),
        (v_period_id, 'breakfast', 'BEYAZ ETLİ SUCUKLU TOST', '100 g Beyaz Eti Sucuk', 55.00),
        (v_period_id, 'breakfast', 'DANA ETLİ SUCUKLU TOST', '50 g Dana Eti Sucuk', 190.00),
        (v_period_id, 'breakfast', 'BEYAZ ETLİ KAŞARLI KARIŞIK TOST', '50 g Kaşar + 50 g Sucuk', 65.00),
        (v_period_id, 'breakfast', 'DANA ETLİ KAŞARLI KARIŞIK TOST', '25 g Sucuk + 25 g Kaşar', 180.00),
        (v_period_id, 'breakfast', 'KARIŞIK TOST', '50 g Kaşar + 50 g Sucuk', 65.00),
        (v_period_id, 'breakfast', 'KAŞARLI BAZLAMA TOST', '75 g Kaşar', 160.00),
        (v_period_id, 'breakfast', 'BEYAZ ETLİ SUCUKLU BAZLAMA TOST', '75 g Beyaz Eti Sucuk', 150.00),
        (v_period_id, 'breakfast', 'DANA ETLİ SUCUKLU BAZLAMA TOST', '75 g Dana Eti Sucuk', 250.00),
        (v_period_id, 'breakfast', 'BEYAZ ETLİ KARIŞIK BAZLAMA TOST', '50 g Kaşar + 25 g Sucuk', 140.00),
        (v_period_id, 'breakfast', 'DANA ETLİ KARIŞIK BAZLAMA TOST', '50 g Kaşar + 25 g Sucuk', 180.00),
        (v_period_id, 'breakfast', 'ATOM SANDVİÇ', '150 g', 220.00),
        (v_period_id, 'breakfast', 'KAŞARLI SOĞUK SANDVİÇ', '100 g Kaşar', 65.00),
        (v_period_id, 'breakfast', 'KAŞARLI SALAMLI SOĞUK SANDVİÇ', '50 g Kaşar + 50 g Salam', 65.00),
        (v_period_id, 'breakfast', 'KARIŞIK TEPSİ PİZZA-TEPSİ KUMPİR', '150 g', 50.00),
        (v_period_id, 'breakfast', 'SOSİS KOKTEYL', '100 g', 28.00),
        (v_period_id, 'breakfast', 'SOSİSLİ PATATES KIZARTMASI', '150 g', 32.00),
        (v_period_id, 'breakfast', 'PATATES HAŞLAMA', '150 g', 10.00),
        (v_period_id, 'breakfast', 'PATATES KROKET', '20 g (1 Adet)', 8.00),
        (v_period_id, 'breakfast', 'PATATES KIZARTMASI-KAVURMASI-SALATASI-KÖFTESİ-YUMURTALI PATATES', '150 g', 37.00),
        (v_period_id, 'breakfast', 'KARIŞIK KIZARTMA', '150 g', 40.00),
        (v_period_id, 'breakfast', 'GÖZLEME ÇEŞİTLERİ', '250 g', 65.00),
        (v_period_id, 'breakfast', 'BÖREK ÇEŞİTLERİ', '120 g', 35.00),
        (v_period_id, 'breakfast', 'MENEMEN', '150 g', 41.00),
        (v_period_id, 'breakfast', 'SADE OMLET', '1 Adet L Boy', 21.00),
        (v_period_id, 'breakfast', 'PEYNİRLİ/SEBZELİ OMLET', '150 g', 25.00),
        (v_period_id, 'breakfast', 'KAŞARLI OMLET', '150 g', 31.00),
        (v_period_id, 'breakfast', 'SUCUKLU OMLET', '150 g', 28.00),
        (v_period_id, 'breakfast', 'SAHANDA ÇİFT YUMURTA', '2 Adet L Boy', 50.00),
        (v_period_id, 'breakfast', 'SAHANDA DANA ETİ SUCUKLU TEK YUMURTA', '1 Adet L Boy + 25 g Sucuk', 65.00),
        (v_period_id, 'breakfast', 'SAHANDA BEYAZ ETİ SUCUKLU ÇİFT YUMURTA', '2 Adet L Boy + 40 g Sucuk', 60.00),
        (v_period_id, 'breakfast', 'SAHANDA DANA ETİ SUCUKLU ÇİFT YUMURTA', '2 Adet L Boy + 40 g Sucuk', 130.00),
        (v_period_id, 'breakfast', 'SAHANDA KAŞARLI ÇİFT YUMURTA', '2 Adet L Boy + 40 g Kaşar', 80.00),
        (v_period_id, 'breakfast', 'SAHANDA KIYMALI YUMURTA', '2 Adet L Boy + 40 g Kıyma', 120.00),
        (v_period_id, 'breakfast', 'YUMURTALI EKMEK', '1 Dilim', 11.00),
        (v_period_id, 'breakfast', 'KEK', '50 g', 17.00),
        (v_period_id, 'breakfast', 'KAHVALTI KEKİ (TUZLU)', '100 g', 30.00),
        (v_period_id, 'breakfast', 'KREP/PANKEK', '1 Adet (25-30 g)', 7.00),
        (v_period_id, 'breakfast', 'KAHVALTILIK GEVREK', '40 g', 12.00),
        (v_period_id, 'breakfast', 'PİŞİ', '60 g', 7.00),
        (v_period_id, 'breakfast', 'AÇMA', '1 Adet (80 g)', 22.00),
        (v_period_id, 'breakfast', 'SİMİT', '1 Adet (100 g)', 22.00),
        (v_period_id, 'breakfast', 'POĞAÇA', '1 Adet (80 g)', 22.00),
        (v_period_id, 'breakfast', 'EKMEK', '1/4 Adet (80 g)', 4.00),
        (v_period_id, 'breakfast', 'ÇORBA ÇEŞİTLERİ', '250 g', 33.00),
        (v_period_id, 'breakfast', 'SÜT', '200 ml', 15.00),
        (v_period_id, 'breakfast', 'MEYVE SUYU', '200 ml', 15.00),
        (v_period_id, 'breakfast', 'AYRAN', '200 ml', 10.00),
        (v_period_id, 'breakfast', 'SU', '500 ml', 5.00),
        (v_period_id, 'breakfast', 'ÇAY', '1 Bardak', 3.00),
        (v_period_id, 'breakfast', 'MEYVE ÇEŞİTLERİ', '1 Adet', 17.00),
        (v_period_id, 'breakfast', 'MEYVE', '150-200 g', 17.00)
        ON CONFLICT (pricing_period_id, meal_type, category_name) DO UPDATE 
        SET portion_amount = EXCLUDED.portion_amount, price = EXCLUDED.price;

        -- ==========================================
        -- 2. ÖĞLE YEMEĞİ GRAMAJ VE FİYAT LİSTESİ
        -- ==========================================
        INSERT INTO meal_category_prices (pricing_period_id, meal_type, category_name, portion_amount, price) VALUES
        (v_period_id, 'lunch', 'ÇORBA ÇEŞİTLERİ', '250 g', 33.00),
        (v_period_id, 'lunch', 'PİRİNÇ PİLAVI ÇEŞİTLERİ', '150 g', 38.00),
        (v_period_id, 'lunch', 'BULGUR PİLAVI ÇEŞİTLERİ', '200 g', 38.00),
        (v_period_id, 'lunch', 'MAKARNA ÇEŞİTLERİ', '200 g', 38.00),
        (v_period_id, 'lunch', 'FIRIN MAKARNA', '150 g', 38.00),
        (v_period_id, 'lunch', 'MANTI', '250 g', 45.00),
        (v_period_id, 'lunch', 'BÖREK ÇEŞİTLERİ', '120 g', 35.00),
        (v_period_id, 'lunch', 'ETLİ BAKLAGİLLER', '200 g', 93.00),
        (v_period_id, 'lunch', 'ETSİZ BAKLAGİLLER', '200 g', 60.00),
        (v_period_id, 'lunch', 'ETLİ SEBZE YEMEKLERİ', '200 g', 93.00),
        (v_period_id, 'lunch', 'ETSİZ SEBZE YEMEKLERİ', '200 g', 60.00),
        (v_period_id, 'lunch', 'ETLİ DOLMA VE SARMALAR', '200 g', 93.00),
        (v_period_id, 'lunch', 'ETSİZ DOLMA VE SARMALAR', '200 g', 60.00),
        (v_period_id, 'lunch', 'KEMİKLİ ET YEMEKLERİ', '250 g', 125.00),
        (v_period_id, 'lunch', 'KEMİKSİZ ET YEMEKLERİ', '250 g', 125.00),
        (v_period_id, 'lunch', 'DANA CİĞER', '250 g', 125.00),
        (v_period_id, 'lunch', 'PİDELİ VEYA 1/2 EKMEKLİ ET DÖNER', '250 g', 125.00),
        (v_period_id, 'lunch', 'IZGARA KÖFTELER', '200 g', 110.00),
        (v_period_id, 'lunch', 'SULU SALÇALI ETLİ YEMEKLER VE TERBİYELİ SEBZELİ KÖFTELER', '250 g', 93.00),
        (v_period_id, 'lunch', 'KEMİKSİZ IZGARA/KIZARTMA TAVUK YEMEKLERİ', '200 g', 93.00),
        (v_period_id, 'lunch', 'PİDELİ VEYA 1/2 EKMEKLİ TAVUK DÖNER', '200 g', 93.00),
        (v_period_id, 'lunch', 'KEMİKSİZ TAVUK YEMEKLERİ', '250 g', 93.00),
        (v_period_id, 'lunch', 'KEMİKLİ TAVUK YEMEKLERİ', '250 g', 93.00),
        (v_period_id, 'lunch', 'SALATA-I', '150 g', 24.00),
        (v_period_id, 'lunch', 'SALATA-II', '100 g', 21.00),
        (v_period_id, 'lunch', 'SALATA-III', '50 g', 18.00),
        (v_period_id, 'lunch', 'TURŞU', '80 g', 13.00),
        (v_period_id, 'lunch', 'YOĞURT', '120 g', 17.00),
        (v_period_id, 'lunch', 'CACIK', '150 g', 18.00),
        (v_period_id, 'lunch', 'MEZELER', '100 g', 24.00),
        (v_period_id, 'lunch', 'MERCİMEKLİ KÖFTE - KISIR', '100 g', 24.00),
        (v_period_id, 'lunch', 'ÇİĞ KÖFTE', '100 g', 24.00),
        (v_period_id, 'lunch', 'KOMPOSTO - HOŞAF ÇEŞİTLERİ', '200 g', 19.00),
        (v_period_id, 'lunch', 'PATATES KIZARTMASI-KAVURMASI-SALATASI-KÖFTESİ-YUMURTALI PATATES', '150 g', 37.00),
        (v_period_id, 'lunch', 'KARIŞIK KIZARTMA', '150 g', 60.00),
        (v_period_id, 'lunch', 'BAKLAVA-KADAYIF (CEVİZLİ-FINDIKLI)', '100 g', 46.00),
        (v_period_id, 'lunch', 'BAKLAVA-KADAYIF (FISTIKLI)', '100 g', 50.00),
        (v_period_id, 'lunch', 'AŞURE', '150 g', 40.00),
        (v_period_id, 'lunch', 'HELVA TATLISI ÇEŞİTLERİ', '100 g', 38.00),
        (v_period_id, 'lunch', 'HAMUR TATLILARI', '100 g', 38.00),
        (v_period_id, 'lunch', 'SÜTLÜ TATLILAR', '150 g', 38.00),
        (v_period_id, 'lunch', 'YAŞ PASTA', '120 g', 40.00),
        (v_period_id, 'lunch', 'MEYVELİ TATLILAR', '150 g', 41.00),
        (v_period_id, 'lunch', 'MEYVE ÇEŞİTLERİ', '1 Adet', 17.00),
        (v_period_id, 'lunch', 'EKMEK', '1/4 Adet', 4.00),
        (v_period_id, 'lunch', 'AYRAN', '200 ml', 10.00),
        (v_period_id, 'lunch', 'SU', '500 ml', 5.00),
        (v_period_id, 'lunch', 'MEYVE SUYU', '200 ml', 15.00)
        ON CONFLICT (pricing_period_id, meal_type, category_name) DO UPDATE 
        SET portion_amount = EXCLUDED.portion_amount, price = EXCLUDED.price;

        -- ==========================================
        -- 3. AKŞAM YEMEĞİ GRAMAJ VE FİYAT LİSTESİ
        -- ==========================================
        INSERT INTO meal_category_prices (pricing_period_id, meal_type, category_name, portion_amount, price) VALUES
        (v_period_id, 'dinner', 'ÇORBA ÇEŞİTLERİ', '250 g', 33.00),
        (v_period_id, 'dinner', 'PİRİNÇ PİLAVI ÇEŞİTLERİ', '150 g', 38.00),
        (v_period_id, 'dinner', 'BULGUR PİLAVI ÇEŞİTLERİ', '200 g', 38.00),
        (v_period_id, 'dinner', 'MAKARNA ÇEŞİTLERİ', '200 g', 38.00),
        (v_period_id, 'dinner', 'FIRIN MAKARNA', '150 g', 38.00),
        (v_period_id, 'dinner', 'MANTI', '250 g', 45.00),
        (v_period_id, 'dinner', 'BÖREK ÇEŞİTLERİ', '120 g', 35.00),
        (v_period_id, 'dinner', 'ETLİ BAKLAGİLLER', '200 g', 93.00),
        (v_period_id, 'dinner', 'ETSİZ BAKLAGİLLER', '200 g', 60.00),
        (v_period_id, 'dinner', 'ETLİ SEBZE YEMEKLERİ', '200 g', 93.00),
        (v_period_id, 'dinner', 'ETSİZ SEBZE YEMEKLERİ', '200 g', 60.00),
        (v_period_id, 'dinner', 'ETLİ DOLMA VE SARMALAR', '200 g', 93.00),
        (v_period_id, 'dinner', 'ETSİZ DOLMA VE SARMALAR', '200 g', 60.00),
        (v_period_id, 'dinner', 'KEMİKLİ ET YEMEKLERİ', '250 g', 125.00),
        (v_period_id, 'dinner', 'KEMİKSİZ ET YEMEKLERİ', '250 g', 125.00),
        (v_period_id, 'dinner', 'DANA CİĞER', '250 g', 125.00),
        (v_period_id, 'dinner', 'PİDELİ VEYA 1/2 EKMEKLİ ET DÖNER', '250 g', 125.00),
        (v_period_id, 'dinner', 'IZGARA KÖFTELER', '200 g', 110.00),
        (v_period_id, 'dinner', 'SULU SALÇALI ETLİ YEMEKLER VE TERBİYELİ SEBZELİ KÖFTELER', '250 g', 93.00),
        (v_period_id, 'dinner', 'KEMİKSİZ IZGARA/KIZARTMA TAVUK YEMEKLERİ', '200 g', 93.00),
        (v_period_id, 'dinner', 'PİDELİ VEYA 1/2 EKMEKLİ TAVUK DÖNER', '200 g', 93.00),
        (v_period_id, 'dinner', 'KEMİKSİZ TAVUK YEMEKLERİ', '250 g', 93.00),
        (v_period_id, 'dinner', 'KEMİKLİ TAVUK YEMEKLERİ', '250 g', 93.00),
        (v_period_id, 'dinner', 'SALATA-I', '150 g', 24.00),
        (v_period_id, 'dinner', 'SALATA-II', '100 g', 21.00),
        (v_period_id, 'dinner', 'SALATA-III', '50 g', 18.00),
        (v_period_id, 'dinner', 'TURŞU', '80 g', 13.00),
        (v_period_id, 'dinner', 'YOĞURT', '120 g', 17.00),
        (v_period_id, 'dinner', 'CACIK', '150 g', 18.00),
        (v_period_id, 'dinner', 'MEZELER', '100 g', 24.00),
        (v_period_id, 'dinner', 'MERCİMEKLİ KÖFTE - KISIR', '100 g', 24.00),
        (v_period_id, 'dinner', 'ÇİĞ KÖFTE', '100 g', 24.00),
        (v_period_id, 'dinner', 'KOMPOSTO - HOŞAF ÇEŞİTLERİ', '200 g', 19.00),
        (v_period_id, 'dinner', 'PATATES KIZARTMASI-KAVURMASI-SALATASI-KÖFTESİ-YUMURTALI PATATES', '150 g', 37.00),
        (v_period_id, 'dinner', 'KARIŞIK KIZARTMA', '150 g', 60.00),
        (v_period_id, 'dinner', 'BAKLAVA-KADAYIF (CEVİZLİ-FINDIKLI)', '100 g', 46.00),
        (v_period_id, 'dinner', 'BAKLAVA-KADAYIF (FISTIKLI)', '100 g', 50.00),
        (v_period_id, 'dinner', 'AŞURE', '150 g', 40.00),
        (v_period_id, 'dinner', 'HELVA TATLISI ÇEŞİTLERİ', '100 g', 38.00),
        (v_period_id, 'dinner', 'HAMUR TATLILARI', '100 g', 38.00),
        (v_period_id, 'dinner', 'SÜTLÜ TATLILAR', '150 g', 38.00),
        (v_period_id, 'dinner', 'YAŞ PASTA', '120 g', 40.00),
        (v_period_id, 'dinner', 'MEYVELİ TATLILAR', '150 g', 41.00),
        (v_period_id, 'dinner', 'MEYVE ÇEŞİTLERİ', '1 Adet', 17.00),
        (v_period_id, 'dinner', 'EKMEK', '1/4 Adet', 4.00),
        (v_period_id, 'dinner', 'AYRAN', '200 ml', 10.00),
        (v_period_id, 'dinner', 'SU', '500 ml', 5.00),
        (v_period_id, 'dinner', 'MEYVE SUYU', '200 ml', 15.00)
        ON CONFLICT (pricing_period_id, meal_type, category_name) DO UPDATE 
        SET portion_amount = EXCLUDED.portion_amount, price = EXCLUDED.price;

    END IF;
END$$;

COMMIT;
