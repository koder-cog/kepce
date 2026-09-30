-- 0031_normalize_dishes_and_soups.sql
-- Generative normalization of soups, numerical portion parentheses, and volume standardizations.
-- Safely merges duplicates using merge_or_rename_dish pattern while preserving raw aliases.

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
        SELECT id, name FROM dishes 
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

DO $$
DECLARE
    rec RECORD;
    target_name TEXT;
BEGIN
    -- 1. Generative Soup Normalization: '% Ç.' or '% Ç'
    FOR rec IN
        SELECT DISTINCT name FROM dishes WHERE name ~* '\s+ç\.?$'
    LOOP
        target_name := regexp_replace(rec.name, '(?i)\s+ç\.?$', ' Çorbası');
        PERFORM merge_or_rename_dish(ARRAY[rec.name], target_name);
    END LOOP;

    -- 2. Portion Parentheses Normalization: e.g. '% (50 g)', '% (120 gr)'
    -- Preserves descriptive parentheses like '(Yoğurt+Sos)' or '(Göbek Marul+Limon)'
    FOR rec IN
        SELECT DISTINCT name FROM dishes WHERE name ~* '\s*\(\s*\d+(?:[.,]\d+)?\s*(?:g|gr|kg|ml|lt|l|adet|porsiyon)\s*\)'
    LOOP
        target_name := trim(regexp_replace(rec.name, '(?i)\s*\(\s*\d+(?:[.,]\d+)?\s*(?:g|gr|kg|ml|lt|l|adet|porsiyon)\s*\)', '', 'g'));
        IF target_name <> rec.name AND length(target_name) > 0 THEN
            PERFORM merge_or_rename_dish(ARRAY[rec.name], target_name);
        END IF;
    END LOOP;

    -- 3. Drink Volume Standardizations
    FOR rec IN
        SELECT DISTINCT name FROM dishes WHERE name ~* '^(?:500\s*ml\.?|0[.,]5\s*(?:l|lt)\.?)\s*su$'
    LOOP
        PERFORM merge_or_rename_dish(ARRAY[rec.name], '500 ml Su');
    END LOOP;

    FOR rec IN
        SELECT DISTINCT name FROM dishes WHERE name ~* '^(?:200\s*ml\.?|0[.,]2\s*(?:l|lt)\.?)\s*ayran$'
    LOOP
        PERFORM merge_or_rename_dish(ARRAY[rec.name], '200 ml Ayran');
    END LOOP;
END $$;

COMMIT;
