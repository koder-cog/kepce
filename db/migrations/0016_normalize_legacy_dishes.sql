-- 0016_normalize_legacy_dishes.sql
-- Normalize existing legacy dish names and abbreviations in database

BEGIN;

-- 1. Standardize liquid and volume units
UPDATE dishes SET name = '500 ml Su' WHERE name ILIKE '500%ml%su' OR name = '500 ml. su';
UPDATE dishes SET name = '200 ml Ayran' WHERE name ILIKE '200%ml%ayran' OR name = '200 ml. ayran';
UPDATE dishes SET name = '200 ml Süt' WHERE name ILIKE '200%ml%süt' OR name = '200 ml. süt';
UPDATE dishes SET name = '200 ml Meyve Suyu' WHERE name ILIKE '200%ml%meyve%suyu' OR name = '200 ml. meyve suyu';
UPDATE dishes SET name = '330 ml Şalgam' WHERE name ILIKE '330%ml%şalgam' OR name = '330 ml. şalgam';

-- 2. Standardize Pilav abbreviations
UPDATE dishes SET name = 'Pirinç Pilavı' WHERE name IN ('Pirinç P.', 'Pirinç P', 'Pırınç P.', 'Pırınç P');
UPDATE dishes SET name = 'Bulgur Pilavı' WHERE name IN ('Bulgur P.', 'Bulgur P');
UPDATE dishes SET name = 'Sebzeli Bulgur Pilavı' WHERE name IN ('Sebzeli Bulgur P.', 'Sebzeli Bulgur P');
UPDATE dishes SET name = 'Salçalı Bulgur Pilavı' WHERE name IN ('Salçalı Bulgur P.', 'Salçalı Bulgur P');
UPDATE dishes SET name = 'Şehriyeli Bulgur Pilavı' WHERE name IN ('Şeh. Bulgur P.', 'Şeh. Bulgur P', 'Şeh. Bulgur', 'Şehriyeli Bulgur P.');

-- 3. Standardize Soup (Çorba) abbreviations
UPDATE dishes SET name = 'Mercimek Çorbası' WHERE name IN ('Mercimek Ç.', 'Mercimek Ç');
UPDATE dishes SET name = 'Ezogelin Çorbası' WHERE name IN ('Ezogelin Ç.', 'Ezogelin Ç');
UPDATE dishes SET name = 'Domates Çorbası' WHERE name IN ('Domates Ç.', 'Domates Ç');
UPDATE dishes SET name = 'Yayla Çorbası' WHERE name IN ('Yayla Ç.', 'Yayla Ç');
UPDATE dishes SET name = 'Tarhana Çorbası' WHERE name IN ('Tarhana Ç.', 'Tarhana Ç');
UPDATE dishes SET name = 'Düğün Çorbası' WHERE name IN ('Düğün Ç.', 'Düğün Ç');
UPDATE dishes SET name = 'Şehriye Çorbası' WHERE name IN ('Şehriye Ç.', 'Şehriye Ç');

-- 4. Standardize Dish/Vegetable abbreviations
UPDATE dishes SET name = 'Taze Fasulye Yemeği' WHERE name IN ('Taze Fasulye Y.', 'Taze Fasulye Y');
UPDATE dishes SET name = 'Etsiz Nohut Yemeği' WHERE name IN ('Etsiz Nohut Y.', 'Etsiz Nohut Y');
UPDATE dishes SET name = 'Kuru Fasulye Yemeği' WHERE name IN ('Kurufasulye Y.', 'Kuru Fasulye Y.');
UPDATE dishes SET name = 'Patates Kızartması' WHERE name IN ('Patates Kız.', 'Patates Kız');
UPDATE dishes SET name = 'Karışık Kızartma' WHERE name IN ('Karışık Kız.', 'Karışık Kız');
UPDATE dishes SET name = 'Zeytinyağlı Pırasa' WHERE name IN ('Z.yağlı Pırasa', 'Z. yağlı Pırasa');

-- 5. Standardize Orphaned Olive / Adjectives
UPDATE dishes SET name = 'Siyah Zeytin' WHERE name = 'Siyah' AND (category = 'ZEYTİN' OR category IS NULL);

-- 6. Bread standardizations
UPDATE dishes SET name = 'Glutensiz Roll Ekmek' WHERE name ILIKE 'glutensiz roll%';

-- Sync dish_aliases if names matched
UPDATE dish_aliases a
SET name = d.name
FROM dishes d
WHERE a.dish_id = d.id AND a.name != d.name;

COMMIT;
