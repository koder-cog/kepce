-- 0019_normalize_suffix_and_abbreviations.sql
-- Comprehensive normalization of Çorba/Çorbası/Ç., Pilav/Pilavı/P., Salata/Salatası, Yemek/Yemeği
-- and safe deduplication of menu dishes.

BEGIN;

CREATE OR REPLACE FUNCTION merge_or_rename_dish(source_names TEXT[], target_name TEXT)
RETURNS VOID AS $$
DECLARE
    target_rec RECORD;
    source_rec RECORD;
BEGIN
    -- 1. Check if target dish exists
    SELECT id INTO target_rec FROM dishes WHERE name = target_name;

    -- 2. Iterate through each source dish that matches source_names and is NOT the target
    FOR source_rec IN 
        SELECT id FROM dishes 
        WHERE (name = ANY(source_names) OR name ILIKE ANY(source_names)) AND name != target_name
    LOOP
        IF target_rec.id IS NOT NULL THEN
            -- Target dish exists: Safely merge source into target
            UPDATE dish_aliases SET dish_id = target_rec.id WHERE dish_id = source_rec.id;
            UPDATE comments SET dish_id = target_rec.id WHERE dish_id = source_rec.id;

            -- Merge user_favorites
            INSERT INTO user_favorites (user_id, dish_id, created_at)
            SELECT user_id, target_rec.id, created_at FROM user_favorites WHERE dish_id = source_rec.id
            ON CONFLICT (user_id, dish_id) DO NOTHING;
            DELETE FROM user_favorites WHERE dish_id = source_rec.id;

            -- Merge user_pinned_dishes
            INSERT INTO user_pinned_dishes (user_id, dish_id, created_at)
            SELECT user_id, target_rec.id, created_at FROM user_pinned_dishes WHERE dish_id = source_rec.id
            ON CONFLICT (user_id, dish_id) DO NOTHING;
            DELETE FROM user_pinned_dishes WHERE dish_id = source_rec.id;

            -- Merge parent_id
            UPDATE dishes SET parent_id = target_rec.id WHERE parent_id = source_rec.id;

            -- Delete obsolete source record
            DELETE FROM dishes WHERE id = source_rec.id;
        ELSE
            -- Target dish does not exist: Rename first source dish to target_name
            UPDATE dishes SET name = target_name WHERE id = source_rec.id;
            SELECT id INTO target_rec FROM dishes WHERE id = source_rec.id;
        END IF;
    END LOOP;
END;
$$ LANGUAGE plpgsql;

-- 1. SOUPS: Unify Çorba / Çorbası / Ç. / Ç
SELECT merge_or_rename_dish(ARRAY['Mısır Ç.', 'Mısır Ç', 'Mısır Çorba'], 'Mısır Çorbası');
SELECT merge_or_rename_dish(ARRAY['Kremalı Mısır Çorba'], 'Kremalı Mısır Çorbası');
SELECT merge_or_rename_dish(ARRAY['Yoğurtlu Buğday Çorba'], 'Yoğurtlu Buğday Çorbası');
SELECT merge_or_rename_dish(ARRAY['Buğday Çorba'], 'Buğday Çorbası');
SELECT merge_or_rename_dish(ARRAY['Soğan Çorba'], 'Soğan Çorbası');
SELECT merge_or_rename_dish(ARRAY['4 Kaşık Çorba'], '4 Kaşık Çorbası');
SELECT merge_or_rename_dish(ARRAY['Mengen Çorba'], 'Mengen Çorbası');
SELECT merge_or_rename_dish(ARRAY['Arpa Şehriye Çorba'], 'Arpa Şehriye Çorbası');
SELECT merge_or_rename_dish(ARRAY['Kesme Aşı Çorba'], 'Kesme Aşı Çorbası');
SELECT merge_or_rename_dish(ARRAY['Kaşarlı Domates Çorba', 'Kaşarlı Domates Ç.', 'Kaşarlı Domates Ç'], 'Kaşarlı Domates Çorbası');
SELECT merge_or_rename_dish(ARRAY['Salçalı Arpa Şehriye Çorba'], 'Salçalı Arpa Şehriye Çorbası');
SELECT merge_or_rename_dish(ARRAY['Sütlü Brokoli Çorba'], 'Sütlü Brokoli Çorbası');
SELECT merge_or_rename_dish(ARRAY['Lebeniye Çorba'], 'Lebeniye Çorbası');
SELECT merge_or_rename_dish(ARRAY['Soğuk Dövme Çorba'], 'Soğuk Dövme Çorbası');
SELECT merge_or_rename_dish(ARRAY['Etli Düğün Çorba'], 'Etli Düğün Çorbası');
SELECT merge_or_rename_dish(ARRAY['Şehriyeli Tavuk Çorba', 'Şehriyeli Tavuk Ç.', 'Şehriyeli Tavuk Ç'], 'Şehriyeli Tavuk Çorbası');
SELECT merge_or_rename_dish(ARRAY['Köz Sebze Çorba'], 'Köz Sebze Çorbası');
SELECT merge_or_rename_dish(ARRAY['Toyga Çorba', 'Toyga Ç.'], 'Toyga Çorbası');
SELECT merge_or_rename_dish(ARRAY['Havuç Çorba', 'Havuç Ç.', 'Havuç Ç'], 'Havuç Çorbası');
SELECT merge_or_rename_dish(ARRAY['Tarhana Çorba', 'Tarhana Ç'], 'Tarhana Çorbası');
SELECT merge_or_rename_dish(ARRAY['Mercimek Çorba'], 'Mercimek Çorbası');
SELECT merge_or_rename_dish(ARRAY['Sebze Çorba', 'Sebze Ç.'], 'Sebze Çorbası');
SELECT merge_or_rename_dish(ARRAY['Yayla Çorba', 'Yayla Ç'], 'Yayla Çorbası');
SELECT merge_or_rename_dish(ARRAY['Yüksük Çorba', 'Yüksük Ç.'], 'Yüksük Çorbası');
SELECT merge_or_rename_dish(ARRAY['Şehriye Çorba'], 'Şehriye Çorbası');
SELECT merge_or_rename_dish(ARRAY['Erişteli Yeşil Mercimek Çorba', 'Erişteli Yeşil Mercimek Ç.'], 'Erişteli Yeşil Mercimek Çorbası');
SELECT merge_or_rename_dish(ARRAY['Domates Çorba'], 'Domates Çorbası');
SELECT merge_or_rename_dish(ARRAY['Düğün Çorba'], 'Düğün Çorbası');
SELECT merge_or_rename_dish(ARRAY['Mahluta Çorba'], 'Mahluta Çorbası');
SELECT merge_or_rename_dish(ARRAY['Anadolu Çorba'], 'Anadolu Çorbası');
SELECT merge_or_rename_dish(ARRAY['Tutmaç Çorba', 'Tutmaç Ç.', 'Tutmaç Ç'], 'Tutmaç Çorbası');
SELECT merge_or_rename_dish(ARRAY['Kremalı Sebze Çorba', 'Kremalı Sebze Ç.'], 'Kremalı Sebze Çorbası');
SELECT merge_or_rename_dish(ARRAY['Ezogelin Çorba'], 'Ezogelin Çorbası');
SELECT merge_or_rename_dish(ARRAY['Kremalı Brokoli Çorba'], 'Kremalı Brokoli Çorbası');
SELECT merge_or_rename_dish(ARRAY['Kremalı Tavuk Çorba'], 'Kremalı Tavuk Çorbası');
SELECT merge_or_rename_dish(ARRAY['Terbiyeli Sebze Çorba'], 'Terbiyeli Sebze Çorbası');
SELECT merge_or_rename_dish(ARRAY['Brokoli Çorba', 'Brokoli Ç.', 'Brokoli Ç'], 'Brokoli Çorbası');
SELECT merge_or_rename_dish(ARRAY['Çeşmi Nigar Çorba', 'Çeşmi Nigar Ç.'], 'Çeşmi Nigar Çorbası');
SELECT merge_or_rename_dish(ARRAY['Alaca Çorba', 'Alaca Ç.'], 'Alaca Çorbası');
SELECT merge_or_rename_dish(ARRAY['Köz Biber Çorba', 'Köz Biber Ç.'], 'Köz Biber Çorbası');
SELECT merge_or_rename_dish(ARRAY['Ayran Aşı Çorba', 'Ayran Aşı Ç.'], 'Ayran Aşı Çorbası');
SELECT merge_or_rename_dish(ARRAY['Çölyak Tarhana Çorba', 'Çölyak Tarhana Ç.'], 'Çölyak Tarhana Çorbası');
SELECT merge_or_rename_dish(ARRAY['Tavuk Suyu Çorba', 'Tavuk Suyu Ç.'], 'Tavuk Suyu Çorbası');

-- 2. PILAF: Unify Pilav / Pilavı / P. / P
SELECT merge_or_rename_dish(ARRAY['Bulgur Pilav', 'Bulgur P.', 'Bulgur P'], 'Bulgur Pilavı');
SELECT merge_or_rename_dish(ARRAY['Pirinç Pilav', 'Pirinç P.', 'Pirinç P'], 'Pirinç Pilavı');
SELECT merge_or_rename_dish(ARRAY['Nohutlu Pirinç Pilav'], 'Nohutlu Pirinç Pilavı');
SELECT merge_or_rename_dish(ARRAY['Meyhane Pilav'], 'Meyhane Pilavı');
SELECT merge_or_rename_dish(ARRAY['Garnitürlü Pirinç Pilav'], 'Garnitürlü Pirinç Pilavı');
SELECT merge_or_rename_dish(ARRAY['Salçalı Bulgur Pilav', 'Salçalı Bulgur P.'], 'Salçalı Bulgur Pilavı');
SELECT merge_or_rename_dish(ARRAY['Sebzeli Bulgur Pilav', 'Sebzeli Bulgur P.'], 'Sebzeli Bulgur Pilavı');

-- 3. SALAD: Unify Salata / Salatası
SELECT merge_or_rename_dish(ARRAY['Coleslaw Salata', 'Koslov Salata', 'Koslov Salatası'], 'Coleslaw Salatası');
SELECT merge_or_rename_dish(ARRAY['Kış Salata'], 'Kış Salatası');
SELECT merge_or_rename_dish(ARRAY['Akdeniz Salata'], 'Akdeniz Salatası');
SELECT merge_or_rename_dish(ARRAY['Havuç-lahana Salata', 'Havuç-Lahana Salata'], 'Havuç-Lahana Salatası');
SELECT merge_or_rename_dish(ARRAY['Havuç Aysberg Salata'], 'Havuç Aysberg Salatası');
SELECT merge_or_rename_dish(ARRAY['Brokoli Salata'], 'Brokoli Salatası');
SELECT merge_or_rename_dish(ARRAY['Marul Salata'], 'Marul Salatası');
SELECT merge_or_rename_dish(ARRAY['Pancar Salata'], 'Pancar Salatası');
SELECT merge_or_rename_dish(ARRAY['Roka Salata'], 'Roka Salatası');
SELECT merge_or_rename_dish(ARRAY['Rus Salata'], 'Rus Salatası');
SELECT merge_or_rename_dish(ARRAY['Bahçe Salata'], 'Bahçe Salatası');
SELECT merge_or_rename_dish(ARRAY['Patates Salata'], 'Patates Salatası');

-- 4. DISHES: Unify Yemek / Yemeği / Y.
SELECT merge_or_rename_dish(ARRAY['Fırında Sebzeli Tavuk Y.'], 'Fırında Sebzeli Tavuk Yemeği');
SELECT merge_or_rename_dish(ARRAY['Taze Fasulye Y.', 'Taze Fasulye Yemek'], 'Taze Fasulye Yemeği');

-- 5. CONSOLIDATE METADATA IN MENU_DISHES (amount, calories)
UPDATE menu_dishes target
SET 
    amount = COALESCE(NULLIF(target.amount, ''), src.max_amount),
    calories = COALESCE(target.calories, src.max_calories)
FROM (
    SELECT 
        md.menu_id, md.package_name, md.order_index, da.dish_id,
        MAX(NULLIF(md.amount, '')) as max_amount,
        MAX(md.calories) as max_calories
    FROM menu_dishes md
    JOIN dish_aliases da ON md.dish_alias_id = da.id
    GROUP BY md.menu_id, md.package_name, md.order_index, da.dish_id
    HAVING count(*) > 1
) src,
dish_aliases da_target
WHERE target.dish_alias_id = da_target.id
  AND target.menu_id = src.menu_id 
  AND target.package_name = src.package_name 
  AND target.order_index = src.order_index
  AND da_target.dish_id = src.dish_id;

-- 6. DELETE REDUNDANT DUPLICATE ROWS
DELETE FROM menu_dishes
WHERE id IN (
    SELECT md.id
    FROM (
        SELECT md.id,
               ROW_NUMBER() OVER (
                   PARTITION BY md.menu_id, md.package_name, md.order_index, da.dish_id
                   ORDER BY 
                       (md.amount IS NOT NULL AND md.amount != '') DESC,
                       md.calories IS NOT NULL DESC,
                       md.id ASC
               ) as rn
        FROM menu_dishes md
        JOIN dish_aliases da ON md.dish_alias_id = da.id
    ) md
    WHERE md.rn > 1
);

-- Clean up helper function
DROP FUNCTION merge_or_rename_dish(TEXT[], TEXT);

COMMIT;
