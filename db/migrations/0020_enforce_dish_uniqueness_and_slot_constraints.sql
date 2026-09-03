-- 0020_enforce_dish_uniqueness_and_slot_constraints.sql
-- 1. Merge case-insensitive duplicates in dishes (e.g. "Kasap Köfte+Patates" vs "Kasap Köfte+patates")
-- 2. Consolidate metadata (amount/calories) and delete newly exposed duplicates in menu_dishes
-- 3. Add UNIQUE INDEX on LOWER(TRIM(name)) in dishes
-- 4. Add UNIQUE INDEX on primary slots (menu_id, package_name, order_index WHERE is_alternative = false)
-- 5. Add trigger preventing any duplicate dish_id from entering the same course slot in menu_dishes

BEGIN;

-- 1. Deduplicate case variants in dishes
DO $$
DECLARE
    rec RECORD;
    target_id INT;
    dup_id INT;
BEGIN
    FOR rec IN 
        SELECT LOWER(TRIM(name)) as lower_name, 
               array_agg(id ORDER BY (category IS NOT NULL) DESC, id ASC) as ids
        FROM dishes
        GROUP BY LOWER(TRIM(name))
        HAVING count(*) > 1
    LOOP
        target_id := rec.ids[1];
        
        FOR i IN 2..array_length(rec.ids, 1) LOOP
            dup_id := rec.ids[i];
            
            UPDATE dish_aliases SET dish_id = target_id WHERE dish_id = dup_id;
            UPDATE comments SET dish_id = target_id WHERE dish_id = dup_id;

            INSERT INTO user_favorites (user_id, dish_id, created_at)
            SELECT user_id, target_id, created_at FROM user_favorites WHERE dish_id = dup_id
            ON CONFLICT (user_id, dish_id) DO NOTHING;
            DELETE FROM user_favorites WHERE dish_id = dup_id;

            INSERT INTO user_pinned_dishes (user_id, dish_id, created_at)
            SELECT user_id, target_id, created_at FROM user_pinned_dishes WHERE dish_id = dup_id
            ON CONFLICT (user_id, dish_id) DO NOTHING;
            DELETE FROM user_pinned_dishes WHERE dish_id = dup_id;

            UPDATE dishes SET parent_id = target_id WHERE parent_id = dup_id;
            DELETE FROM dishes WHERE id = dup_id;
        END LOOP;
    END LOOP;
END;
$$;

-- 2. Consolidate metadata & delete any newly discovered duplicates in menu_dishes
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

-- 3. Create unique index on dishes to prevent any case-variant duplicates
CREATE UNIQUE INDEX IF NOT EXISTS uq_dishes_lower_name ON dishes (LOWER(TRIM(name)));

-- 4. Create unique index on primary slot in menu_dishes
CREATE UNIQUE INDEX IF NOT EXISTS uq_menu_dishes_primary_slot ON menu_dishes (menu_id, package_name, order_index) WHERE (is_alternative = false);

-- 5. Create Trigger for slot dish uniqueness
CREATE OR REPLACE FUNCTION trg_check_menu_dishes_unique_dish_per_slot()
RETURNS TRIGGER AS $$
DECLARE
    new_dish_id INT;
    existing_count INT;
BEGIN
    SELECT dish_id INTO new_dish_id FROM dish_aliases WHERE id = NEW.dish_alias_id;
    
    IF new_dish_id IS NOT NULL THEN
        SELECT COUNT(*) INTO existing_count
        FROM menu_dishes md
        JOIN dish_aliases da ON md.dish_alias_id = da.id
        WHERE md.menu_id = NEW.menu_id
          AND md.package_name = NEW.package_name
          AND md.order_index = NEW.order_index
          AND da.dish_id = new_dish_id
          AND (TG_OP = 'INSERT' OR md.id != NEW.id);
          
        IF existing_count > 0 THEN
            RAISE EXCEPTION 'Bu menü yuvasında (%) aynı yemek zaten kayıtlı (dish_id: %)', NEW.order_index, new_dish_id;
        END IF;
    END IF;
    
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS trg_menu_dishes_unique_dish ON menu_dishes;
CREATE TRIGGER trg_menu_dishes_unique_dish
BEFORE INSERT OR UPDATE ON menu_dishes
FOR EACH ROW
EXECUTE FUNCTION trg_check_menu_dishes_unique_dish_per_slot();

COMMIT;
