-- 0016_normalize_legacy_dishes.sql
-- Normalize existing legacy dish names and merge duplicates safely

BEGIN;

CREATE OR REPLACE FUNCTION merge_or_rename_dish(source_names TEXT[], target_name TEXT)
RETURNS VOID AS $$
DECLARE
    target_rec RECORD;
    source_rec RECORD;
BEGIN
    -- 1. Check if target dish already exists
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

-- 1. Standardize liquid and volume units
SELECT merge_or_rename_dish(ARRAY['500%ml%su', '500 ml. su', '500ml Su', '500 ml su'], '500 ml Su');
SELECT merge_or_rename_dish(ARRAY['200%ml%ayran', '200 ml. ayran', '200ml Ayran', '200 ml ayran'], '200 ml Ayran');
SELECT merge_or_rename_dish(ARRAY['200%ml%süt', '200 ml. süt', '200ml Süt', '200 ml süt'], '200 ml Süt');
SELECT merge_or_rename_dish(ARRAY['200%ml%meyve%suyu', '200 ml. meyve suyu', '200ml Meyve Suyu'], '200 ml Meyve Suyu');
SELECT merge_or_rename_dish(ARRAY['330%ml%şalgam', '330 ml. şalgam', '330ml Şalgam'], '330 ml Şalgam');

-- 2. Standardize Pilav abbreviations
SELECT merge_or_rename_dish(ARRAY['Pirinç P.', 'Pirinç P', 'Pırınç P.', 'Pırınç P'], 'Pirinç Pilavı');
SELECT merge_or_rename_dish(ARRAY['Bulgur P.', 'Bulgur P'], 'Bulgur Pilavı');
SELECT merge_or_rename_dish(ARRAY['Sebzeli Bulgur P.', 'Sebzeli Bulgur P'], 'Sebzeli Bulgur Pilavı');
SELECT merge_or_rename_dish(ARRAY['Salçalı Bulgur P.', 'Salçalı Bulgur P'], 'Salçalı Bulgur Pilavı');
SELECT merge_or_rename_dish(ARRAY['Şeh. Bulgur P.', 'Şeh. Bulgur P', 'Şeh. Bulgur', 'Şehriyeli Bulgur P.'], 'Şehriyeli Bulgur Pilavı');

-- 3. Standardize Soup (Çorba) abbreviations
SELECT merge_or_rename_dish(ARRAY['Mercimek Ç.', 'Mercimek Ç'], 'Mercimek Çorbası');
SELECT merge_or_rename_dish(ARRAY['Ezogelin Ç.', 'Ezogelin Ç'], 'Ezogelin Çorbası');
SELECT merge_or_rename_dish(ARRAY['Domates Ç.', 'Domates Ç'], 'Domates Çorbası');
SELECT merge_or_rename_dish(ARRAY['Yayla Ç.', 'Yayla Ç'], 'Yayla Çorbası');
SELECT merge_or_rename_dish(ARRAY['Tarhana Ç.', 'Tarhana Ç'], 'Tarhana Çorbası');
SELECT merge_or_rename_dish(ARRAY['Düğün Ç.', 'Düğün Ç'], 'Düğün Çorbası');
SELECT merge_or_rename_dish(ARRAY['Şehriye Ç.', 'Şehriye Ç'], 'Şehriye Çorbası');

-- 4. Standardize Dish/Vegetable abbreviations
SELECT merge_or_rename_dish(ARRAY['Taze Fasulye Y.', 'Taze Fasulye Y'], 'Taze Fasulye Yemeği');
SELECT merge_or_rename_dish(ARRAY['Etsiz Nohut Y.', 'Etsiz Nohut Y'], 'Etsiz Nohut Yemeği');
SELECT merge_or_rename_dish(ARRAY['Kurufasulye Y.', 'Kuru Fasulye Y.'], 'Kuru Fasulye Yemeği');
SELECT merge_or_rename_dish(ARRAY['Patates Kız.', 'Patates Kız'], 'Patates Kızartması');
SELECT merge_or_rename_dish(ARRAY['Karışık Kız.', 'Karışık Kız'], 'Karışık Kızartma');
SELECT merge_or_rename_dish(ARRAY['Z.yağlı Pırasa', 'Z. yağlı Pırasa'], 'Zeytinyağlı Pırasa');

-- 5. Standardize Orphaned Olive / Adjectives
SELECT merge_or_rename_dish(ARRAY['Siyah'], 'Siyah Zeytin');

-- 6. Bread standardizations
SELECT merge_or_rename_dish(ARRAY['glutensiz roll%'], 'Glutensiz Roll Ekmek');

-- Clean up helper function
DROP FUNCTION merge_or_rename_dish(TEXT[], TEXT);

COMMIT;
