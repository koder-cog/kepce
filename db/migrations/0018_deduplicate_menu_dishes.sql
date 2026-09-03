-- 0018_deduplicate_menu_dishes.sql
-- Two-stage safe deduplication:
-- 1. Consolidate metadata (amount, calories) from duplicate rows into the primary row (COALESCE/MAX).
-- 2. Delete redundant duplicate rows safely without data loss.

BEGIN;

-- ==============================================================================
-- 1. DEDUPLICATE SAME DISH_ID UNDER DIFFERENT ALIASES IN SAME MENU & PACKAGE
-- ==============================================================================

-- A. Metadata Konsolidasyonu
UPDATE menu_dishes target
SET 
    amount = COALESCE(NULLIF(target.amount, ''), src.max_amount),
    calories = COALESCE(target.calories, src.max_calories)
FROM (
    SELECT 
        md.menu_id, md.package_name, da.dish_id,
        MAX(NULLIF(md.amount, '')) as max_amount,
        MAX(md.calories) as max_calories
    FROM menu_dishes md
    JOIN dish_aliases da ON md.dish_alias_id = da.id
    GROUP BY md.menu_id, md.package_name, da.dish_id
    HAVING count(*) > 1
) src,
dish_aliases da_target
WHERE target.dish_alias_id = da_target.id
  AND target.menu_id = src.menu_id 
  AND target.package_name = src.package_name 
  AND da_target.dish_id = src.dish_id;

-- B. Mükerrer Satırları Temizle
WITH ranked_same_dish AS (
    SELECT 
        md.id,
        ROW_NUMBER() OVER (
            PARTITION BY md.menu_id, md.package_name, da.dish_id 
            ORDER BY 
                (md.amount IS NOT NULL AND md.amount != '') DESC,
                (md.calories IS NOT NULL) DESC,
                md.id ASC
        ) as rn
    FROM menu_dishes md
    JOIN dish_aliases da ON md.dish_alias_id = da.id
)
DELETE FROM menu_dishes WHERE id IN (SELECT id FROM ranked_same_dish WHERE rn > 1);

-- ==============================================================================
-- 2. DEDUPLICATE COLLIDING PRIMARY SLOTS (ORDER_INDEX) IN SAME MENU & PACKAGE
-- ==============================================================================

-- A. Slot Metadata Konsolidasyonu
UPDATE menu_dishes target
SET 
    amount = COALESCE(NULLIF(target.amount, ''), src.max_amount),
    calories = COALESCE(target.calories, src.max_calories)
FROM (
    SELECT 
        menu_id, package_name, order_index,
        MAX(NULLIF(amount, '')) as max_amount,
        MAX(calories) as max_calories
    FROM menu_dishes
    WHERE is_alternative = false
    GROUP BY menu_id, package_name, order_index
    HAVING count(*) > 1
) src
WHERE target.menu_id = src.menu_id 
  AND target.package_name = src.package_name 
  AND target.order_index = src.order_index
  AND target.is_alternative = false;

-- B. Çakışan Boş/Fazla Slot Satırlarını Temizle (is_alternative = true alternatiflerine dokunma)
WITH ranked_slots AS (
    SELECT 
        id,
        ROW_NUMBER() OVER (
            PARTITION BY menu_id, package_name, order_index 
            ORDER BY 
                (amount IS NOT NULL AND amount != '') DESC,
                (calories IS NOT NULL) DESC,
                id ASC
        ) as rn
    FROM menu_dishes
    WHERE is_alternative = false
)
DELETE FROM menu_dishes WHERE id IN (SELECT id FROM ranked_slots WHERE rn > 1);

COMMIT;
