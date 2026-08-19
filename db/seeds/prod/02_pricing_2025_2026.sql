-- 02_pricing_2025_2026.sql
-- İstanbul 2025-2026 Dönemi Resmi Fiyatlandırma Seed Verileri
-- Kaynak: T.C. Gençlik ve Spor Bakanlığı İstanbul İl Müdürlüğü (1 Şubat 2026 tarihinden itibaren geçerli liste)

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
        -- ==========================================
        -- 1. KAHVALTI GRAMAJ VE FİYAT LİSTESİ
        -- ==========================================
        INSERT INTO meal_category_prices (pricing_period_id, meal_type, category_name, portion_amount, price) VALUES
        (v_period_id, 'breakfast', 'ZEYTİN', '30 g', 10.00),
        (v_period_id, 'breakfast', 'BEYAZ PEYNİR', '40 g', 13.00),
        (v_period_id, 'breakfast', 'KAŞAR PEYNİRİ', '40 g', 17.00),
        (v_period_id, 'breakfast', 'YÖRESEL PEYNİRLER', '40 g', 17.00),
        (v_period_id, 'breakfast', 'KREM PEYNİR', '20 g', 9.00),
        (v_period_id, 'breakfast', 'ÜÇGEN PEYNİR', '12.5 g', 7.00),
        (v_period_id, 'breakfast', 'LABNE PEYNİR', '20 g', 9.00),
        (v_period_id, 'breakfast', 'HAŞLANMIŞ YUMURTA', '1 Adet L Boyutunda', 11.00),
        (v_period_id, 'breakfast', 'SALAM (HİNDİ)', '35 g', 12.00),
        (v_period_id, 'breakfast', 'SALAM (PİLİÇ)', '35 g', 10.00),
        (v_period_id, 'breakfast', 'SÜRÜLEBİLİR ÇİKOLATA', '20 g', 9.00),
        (v_period_id, 'breakfast', 'PİKNİK BAL', '20 g', 10.00),
        (v_period_id, 'breakfast', 'PİKNİK REÇEL', '20 g', 6.00),
        (v_period_id, 'breakfast', 'PİKNİK TEREYAĞI', '10 g', 9.00),
        (v_period_id, 'breakfast', 'PİKNİK HELVA', '40 g', 11.00),
        (v_period_id, 'breakfast', 'TAHİNLİ PEKMEZ', '20 g', 7.00),
        (v_period_id, 'breakfast', 'KAŞARLI TOST', '50 g', 36.00),
        (v_period_id, 'breakfast', 'BEYAZ ETLİ SUCUKLU TOST', '50 g', 33.00),
        (v_period_id, 'breakfast', 'DANA ETLİ SUCUKLU TOST', '50 g', 55.00),
        (v_period_id, 'breakfast', 'BEYAZ ETLİ KAŞARLI KARIŞIK TOST', '50 g', 36.00),
        (v_period_id, 'breakfast', 'DANA ETLİ KAŞARLI KARIŞIK TOST', '50 g', 55.00),
        (v_period_id, 'breakfast', 'EKMEK ARASI IZGARA TAVUK', '100 g', 80.00),
        (v_period_id, 'breakfast', 'EKMEK ARASI KÖFTE', '90 g', 94.00),
        (v_period_id, 'breakfast', 'KAŞARLI SOĞUK SANDVİÇ', '70 g', 47.00),
        (v_period_id, 'breakfast', 'KAŞARLI SALAMLI SOĞUK SANDVİÇ', '85 g', 47.00),
        (v_period_id, 'breakfast', 'PATSO', '100 g', 34.00),
        (v_period_id, 'breakfast', 'HAMBURGER (ORTA BOY)', '45 g', 67.00),
        (v_period_id, 'breakfast', 'HAMBURGER (BÜYÜK BOY)', '90 g', 116.00),
        (v_period_id, 'breakfast', 'KARIŞIK PİZZA-TEPSİ KUMPİR', '150 g', 46.00),
        (v_period_id, 'breakfast', 'KUMRU', '140 g', 46.00),
        (v_period_id, 'breakfast', 'SOSİS KOKTEYL', '100 g', 27.00),
        (v_period_id, 'breakfast', 'SOSİSLİ PATATES KIZARTMASI', '150 g', 29.00),
        (v_period_id, 'breakfast', 'PATATES HAŞLAMA', '150 g', 9.00),
        (v_period_id, 'breakfast', 'PATATES KROKET', '20 g', 8.00),
        (v_period_id, 'breakfast', 'SOĞAN HALKASI', '20 g', 10.00),
        (v_period_id, 'breakfast', 'ŞNİTZEL', '100 g', 40.00),
        (v_period_id, 'breakfast', 'NUGGET', '100 g', 34.00),
        (v_period_id, 'breakfast', 'PATATES KIZARTMASI-KAVURMASI-SALATASI-KÖFTESİ-YUMURTALI', '150 g', 35.00),
        (v_period_id, 'breakfast', 'KARIŞIK KIZARTMA', '150 g', 37.00),
        (v_period_id, 'breakfast', 'GÖZLEME ÇEŞİTLERİ', '250 g', 47.00),
        (v_period_id, 'breakfast', 'BÖREK ÇEŞİTLERİ', '120 g', 32.00),
        (v_period_id, 'breakfast', 'MENEMEN', '150 g', 40.00),
        (v_period_id, 'breakfast', 'OMLET', '150 g', 24.00),
        (v_period_id, 'breakfast', 'YUMURTALI EKMEK', '1 Dilim', 10.00),
        (v_period_id, 'breakfast', 'SAHANDA TEK YUMURTA', '1 Adet', 20.00),
        (v_period_id, 'breakfast', 'SAHANDA ÇİFT YUMURTA', '2 Adet', 27.00),
        (v_period_id, 'breakfast', 'BEYAZ ETLİ SUCUKLU TEK YUMURTA', '1 Adet', 27.00),
        (v_period_id, 'breakfast', 'DANA ETLİ SUCUKLU TEK YUMURTA', '1 Adet', 38.00),
        (v_period_id, 'breakfast', 'BEYAZ ETLİ SUCUKLU ÇİFT YUMURTA', '2 Adet', 37.00),
        (v_period_id, 'breakfast', 'DANA ETLİ SUCUKLU ÇİFT YUMURTA', '2 Adet', 50.00),
        (v_period_id, 'breakfast', 'KAŞARLI TEK YUMURTA', '1 Adet', 30.00),
        (v_period_id, 'breakfast', 'KAŞARLI ÇİFT YUMURTA', '2 Adet', 38.00),
        (v_period_id, 'breakfast', 'AÇMA', '1 Adet', 20.00),
        (v_period_id, 'breakfast', 'SİMİT', '1 Adet', 20.00),
        (v_period_id, 'breakfast', 'POĞAÇA', '1 Adet', 20.00),
        (v_period_id, 'breakfast', 'EKMEK', '1/4 Adet', 3.50),
        (v_period_id, 'breakfast', 'GLUTENSİZ ROLL EKMEK', '1 Paket', 15.00),
        (v_period_id, 'breakfast', 'KEK', '50 g', 15.00),
        (v_period_id, 'breakfast', 'KAHVALTI KEKİ (TUZLU)', '100 g', 28.00),
        (v_period_id, 'breakfast', 'KREP/PANKEK', '1 Adet', 7.00),
        (v_period_id, 'breakfast', 'KAHVALTILIK GEVREK', '40 g', 10.00),
        (v_period_id, 'breakfast', 'ÇORBA ÇEŞİTLERİ', '250 g', 30.00),
        (v_period_id, 'breakfast', 'MEYVE SUYU', '200 ml', 15.00),
        (v_period_id, 'breakfast', 'SÜT', '200 ml', 15.00),
        (v_period_id, 'breakfast', 'AYRAN', '200 ml', 8.00),
        (v_period_id, 'breakfast', 'SU', '500 ml', 5.00),
        (v_period_id, 'breakfast', 'ÇAY', '1 Bardak', 3.00),
        (v_period_id, 'breakfast', 'BİTKİ ÇAYLARI', '1 Bardak', 5.00),
        (v_period_id, 'breakfast', 'KAHVE', '1 Fincan', 10.00),
        (v_period_id, 'breakfast', 'MADEN SUYU', '200 ml', 8.00),
        (v_period_id, 'breakfast', 'GAZOZ-KOLA-FANTA-SOĞUK ÇAY', '330 ml', 22.00),
        (v_period_id, 'breakfast', 'ŞALGAM', '330 ml', 12.00),
        (v_period_id, 'breakfast', 'MEYVE ÇEŞİTLERİ', '1 Adet', 15.00)
        ON CONFLICT (pricing_period_id, meal_type, category_name) DO UPDATE 
        SET portion_amount = EXCLUDED.portion_amount, price = EXCLUDED.price;

        -- ==========================================
        -- 2. ÖĞLE YEMEĞİ GRAMAJ VE FİYAT LİSTESİ
        -- ==========================================
        INSERT INTO meal_category_prices (pricing_period_id, meal_type, category_name, portion_amount, price) VALUES
        (v_period_id, 'lunch', 'ÇORBA ÇEŞİTLERİ', '250 g', 30.00),
        (v_period_id, 'lunch', 'PİRİNÇ PİLAVI ÇEŞİTLERİ', '150 g', 35.00),
        (v_period_id, 'lunch', 'BULGUR PİLAVI ÇEŞİTLERİ', '200 g', 35.00),
        (v_period_id, 'lunch', 'MAKARNA ÇEŞİTLERİ', '200 g', 35.00),
        (v_period_id, 'lunch', 'FIRIN MAKARNA', '150 g', 35.00),
        (v_period_id, 'lunch', 'MANTI', '250 g', 40.00),
        (v_period_id, 'lunch', 'BÖREK ÇEŞİTLERİ', '120 g', 32.00),
        (v_period_id, 'lunch', 'ETLİ BAKLAGİLLER', '200 g', 80.00),
        (v_period_id, 'lunch', 'ETSİZ BAKLAGİLLER', '200 g', 53.00),
        (v_period_id, 'lunch', 'ETLİ SEBZE YEMEKLERİ', '200 g', 80.00),
        (v_period_id, 'lunch', 'ETSİZ SEBZE YEMEKLERİ', '200 g', 53.00),
        (v_period_id, 'lunch', 'ETLİ DOLMA VE SARMALAR', '200 g', 80.00),
        (v_period_id, 'lunch', 'ETSİZ DOLMA VE SARMALAR', '200 g', 53.00),
        (v_period_id, 'lunch', 'KEMİKLİ ET YEMEKLERİ', '250 g', 105.00),
        (v_period_id, 'lunch', 'KEMİKSİZ ET YEMEKLERİ', '250 g', 105.00),
        (v_period_id, 'lunch', 'PİDELİ VEYA 1/2 EKMEKLİ ET DÖNER', '250 g', 115.00),
        (v_period_id, 'lunch', 'LAVAŞ VEYA 1/2 EKMEKLİ ET TANTUNİ', '200 g', 99.00),
        (v_period_id, 'lunch', 'IZGARA KÖFTELER', '200 g', 94.00),
        (v_period_id, 'lunch', 'SULU SALÇALI ETLİ YEMEKLER VE TERBİYELİ SEBZELİ KÖFTELER', '250 g', 80.00),
        (v_period_id, 'lunch', 'KEMİKSİZ IZGARA/KIZARTMA TAVUK YEMEKLERİ', '200 g', 80.00),
        (v_period_id, 'lunch', 'KEMİKSİZ TAVUK YEMEKLERİ', '250 g', 80.00),
        (v_period_id, 'lunch', 'KEMİKLİ TAVUK YEMEKLERİ', '250 g', 80.00),
        (v_period_id, 'lunch', 'PİDELİ VEYA 1/2 EKMEKLİ TAVUK DÖNER', '250 g', 80.00),
        (v_period_id, 'lunch', 'LAVAŞ VEYA 1/2 EKMEKLİ TAVUK TANTUNİ', '200 g', 75.00),
        (v_period_id, 'lunch', 'DANA CİĞER', '150 g', 105.00),
        (v_period_id, 'lunch', 'MERCİMEKLİ KÖFTE - KISIR', '150 g', 30.00),
        (v_period_id, 'lunch', 'ÇİĞ KÖFTE', '150 g', 36.00),
        (v_period_id, 'lunch', 'SALATA-I', '150 g', 23.00),
        (v_period_id, 'lunch', 'SALATA-II', '100 g', 20.00),
        (v_period_id, 'lunch', 'SALATA-III', '50 g', 17.00),
        (v_period_id, 'lunch', 'TURŞU', '80 g', 12.00),
        (v_period_id, 'lunch', 'YOĞURT', '120 g', 16.00),
        (v_period_id, 'lunch', 'CACIK', '150 g', 17.00),
        (v_period_id, 'lunch', 'MEZELER', '100 g', 23.00),
        (v_period_id, 'lunch', 'KOMPOSTO - HOŞAF ÇEŞİTLERİ', '200 g', 17.00),
        (v_period_id, 'lunch', 'İÇLİ KÖFTE', '120 g', 33.00),
        (v_period_id, 'lunch', 'PATATES KIZARTMASI-KAVURMASI-SALATASI-KÖFTESİ-YUMURTALI PATATES', '150 g', 35.00),
        (v_period_id, 'lunch', 'BAKLAVA-KADAYIF (CEVİZLİ-FINDIKLI)', '100 g', 39.00),
        (v_period_id, 'lunch', 'BAKLAVA-KADAYIF (FISTIKLI)', '100 g', 43.00),
        (v_period_id, 'lunch', 'AŞURE', '150 g', 35.00),
        (v_period_id, 'lunch', 'HELVA TATLISI ÇEŞİTLERİ', '100 g', 33.00),
        (v_period_id, 'lunch', 'HAMUR TATLILARI', '100 g', 33.00),
        (v_period_id, 'lunch', 'SÜTLÜ TATLILAR', '150 g', 33.00),
        (v_period_id, 'lunch', 'YAŞ PASTA', '120 g', 35.00),
        (v_period_id, 'lunch', 'KURU PASTA', '150 g', 33.00),
        (v_period_id, 'lunch', 'MEYVELİ TATLILAR', '150 g', 36.00),
        (v_period_id, 'lunch', 'KUŞBAŞILI VEYA SUCUKLU PİDE', '250 g', 92.00),
        (v_period_id, 'lunch', 'KARIŞIK PİDE', '250 g', 92.00),
        (v_period_id, 'lunch', 'KAŞARLI PİDE', '250 g', 82.00),
        (v_period_id, 'lunch', 'YUMURTALI-BEYAZ PEYNİRLİ PİDE', '250 g', 80.00),
        (v_period_id, 'lunch', 'KIYMALI PİDE', '250 g', 85.00),
        (v_period_id, 'lunch', 'TAVUKLU PİDE', '250 g', 80.00),
        (v_period_id, 'lunch', 'LAHMACUN', '150 g', 55.00),
        (v_period_id, 'lunch', 'MEYVE ÇEŞİTLERİ', '1 Adet', 15.00),
        (v_period_id, 'lunch', 'EKMEK', '1/4 Adet', 3.50),
        (v_period_id, 'lunch', 'GLUTENSİZ ROLL EKMEK', '1 Paket', 15.00),
        (v_period_id, 'lunch', 'AYRAN', '200 ml', 8.00),
        (v_period_id, 'lunch', 'SU', '500 ml', 5.00),
        (v_period_id, 'lunch', 'MEYVE SUYU', '200 ml', 15.00),
        (v_period_id, 'lunch', 'GAZOZ-KOLA-FANTA-SOĞUK ÇAY', '330 ml', 22.00),
        (v_period_id, 'lunch', 'ŞALGAM', '330 ml', 12.00)
        ON CONFLICT (pricing_period_id, meal_type, category_name) DO UPDATE 
        SET portion_amount = EXCLUDED.portion_amount, price = EXCLUDED.price;

        -- ==========================================
        -- 3. AKŞAM YEMEĞİ GRAMAJ VE FİYAT LİSTESİ
        -- ==========================================
        INSERT INTO meal_category_prices (pricing_period_id, meal_type, category_name, portion_amount, price) VALUES
        (v_period_id, 'dinner', 'ÇORBA ÇEŞİTLERİ', '250 g', 30.00),
        (v_period_id, 'dinner', 'PİRİNÇ PİLAVI ÇEŞİTLERİ', '150 g', 35.00),
        (v_period_id, 'dinner', 'BULGUR PİLAVI ÇEŞİTLERİ', '200 g', 35.00),
        (v_period_id, 'dinner', 'MAKARNA ÇEŞİTLERİ', '200 g', 35.00),
        (v_period_id, 'dinner', 'FIRIN MAKARNA', '150 g', 35.00),
        (v_period_id, 'dinner', 'MANTI', '250 g', 40.00),
        (v_period_id, 'dinner', 'BÖREK ÇEŞİTLERİ', '120 g', 32.00),
        (v_period_id, 'dinner', 'ETLİ BAKLAGİLLER', '200 g', 80.00),
        (v_period_id, 'dinner', 'ETSİZ BAKLAGİLLER', '200 g', 53.00),
        (v_period_id, 'dinner', 'ETLİ SEBZE YEMEKLERİ', '200 g', 80.00),
        (v_period_id, 'dinner', 'ETSİZ SEBZE YEMEKLERİ', '200 g', 53.00),
        (v_period_id, 'dinner', 'ETLİ DOLMA VE SARMALAR', '200 g', 80.00),
        (v_period_id, 'dinner', 'ETSİZ DOLMA VE SARMALAR', '200 g', 53.00),
        (v_period_id, 'dinner', 'KEMİKLİ ET YEMEKLERİ', '250 g', 105.00),
        (v_period_id, 'dinner', 'KEMİKSİZ ET YEMEKLERİ', '250 g', 105.00),
        (v_period_id, 'dinner', 'PİDELİ VEYA 1/2 EKMEKLİ ET DÖNER', '250 g', 115.00),
        (v_period_id, 'dinner', 'LAVAŞ VEYA 1/2 EKMEKLİ ET TANTUNİ', '200 g', 99.00),
        (v_period_id, 'dinner', 'IZGARA KÖFTELER', '200 g', 94.00),
        (v_period_id, 'dinner', 'SULU SALÇALI ETLİ YEMEKLER VE TERBİYELİ SEBZELİ KÖFTELER', '250 g', 80.00),
        (v_period_id, 'dinner', 'KEMİKSİZ IZGARA/KIZARTMA TAVUK YEMEKLERİ', '200 g', 80.00),
        (v_period_id, 'dinner', 'KEMİKSİZ TAVUK YEMEKLERİ', '250 g', 80.00),
        (v_period_id, 'dinner', 'KEMİKLİ TAVUK YEMEKLERİ', '250 g', 80.00),
        (v_period_id, 'dinner', 'PİDELİ VEYA 1/2 EKMEKLİ TAVUK DÖNER', '250 g', 80.00),
        (v_period_id, 'dinner', 'LAVAŞ VEYA 1/2 EKMEKLİ TAVUK TANTUNİ', '200 g', 73.00),
        (v_period_id, 'dinner', 'DANA CİĞER', '200 g', 94.00),
        (v_period_id, 'dinner', 'MERCİMEKLİ KÖFTE - KISIR', '200 g', 30.00),
        (v_period_id, 'dinner', 'ÇİĞ KÖFTE', '200 g', 36.00),
        (v_period_id, 'dinner', 'SALATA-I', '150 g', 23.00),
        (v_period_id, 'dinner', 'SALATA-II', '150 g', 20.00),
        (v_period_id, 'dinner', 'SALATA-III', '150 g', 17.00),
        (v_period_id, 'dinner', 'TURŞU', '100 g', 12.00),
        (v_period_id, 'dinner', 'YOĞURT', '150 g', 16.00),
        (v_period_id, 'dinner', 'CACIK', '150 g', 17.00),
        (v_period_id, 'dinner', 'MEZELER', '150 g', 23.00),
        (v_period_id, 'dinner', 'KOMPOSTO - HOŞAF ÇEŞİTLERİ', '200 g', 17.00),
        (v_period_id, 'dinner', 'İÇLİ KÖFTE', '100 g', 40.00),
        (v_period_id, 'dinner', 'PATATES KIZARTMASI-KAVURMASI-SALATASI-KÖFTESİ-YUMURTALI PATATES', '150 g', 35.00),
        (v_period_id, 'dinner', 'BAKLAVA-KADAYIF (CEVİZLİ-FINDIKLI)', '100 g', 39.00),
        (v_period_id, 'dinner', 'BAKLAVA-KADAYIF (FISTIKLI)', '100 g', 43.00),
        (v_period_id, 'dinner', 'AŞURE', '150 g', 35.00),
        (v_period_id, 'dinner', 'HELVA TATLISI ÇEŞİTLERİ', '100 g', 33.00),
        (v_period_id, 'dinner', 'HAMUR TATLILARI', '100 g', 33.00),
        (v_period_id, 'dinner', 'SÜTLÜ TATLILAR', '150 g', 33.00),
        (v_period_id, 'dinner', 'YAŞ PASTA', '120 g', 35.00),
        (v_period_id, 'dinner', 'KURU PASTA', '150 g', 33.00),
        (v_period_id, 'dinner', 'MEYVELİ TATLILAR', '150 g', 36.00),
        (v_period_id, 'dinner', 'KUŞBAŞILI VEYA SUCUKLU PİDE', '250 g', 92.00),
        (v_period_id, 'dinner', 'KARIŞIK PİDE', '250 g', 92.00),
        (v_period_id, 'dinner', 'KAŞARLI PİDE', '250 g', 82.00),
        (v_period_id, 'dinner', 'YUMURTALI-BEYAZ PEYNİRLİ PİDE', '250 g', 80.00),
        (v_period_id, 'dinner', 'KIYMALI PİDE', '250 g', 85.00),
        (v_period_id, 'dinner', 'TAVUKLU PİDE', '250 g', 80.00),
        (v_period_id, 'dinner', 'LAHMACUN', '150 g', 55.00),
        (v_period_id, 'dinner', 'MEYVE ÇEŞİTLERİ', '1 Adet', 15.00),
        (v_period_id, 'dinner', 'EKMEK', '1/4 Adet', 3.50),
        (v_period_id, 'dinner', 'GLUTENSİZ ROLL EKMEK', '1 Paket', 15.00),
        (v_period_id, 'dinner', 'AYRAN', '200 ml', 8.00),
        (v_period_id, 'dinner', 'SU', '500 ml', 5.00),
        (v_period_id, 'dinner', 'MEYVE SUYU', '200 ml', 15.00),
        (v_period_id, 'dinner', 'GAZOZ-KOLA-FANTA-SOĞUK ÇAY', '330 ml', 22.00),
        (v_period_id, 'dinner', 'ŞALGAM', '330 ml', 12.00)
        ON CONFLICT (pricing_period_id, meal_type, category_name) DO UPDATE 
        SET portion_amount = EXCLUDED.portion_amount, price = EXCLUDED.price;

    END IF;
END$$;

COMMIT;
