-- Migration 0013: Menus ve Menu Dishes tablolarına kalori ve porsiyon alanlarını ekleme
ALTER TABLE menus ADD COLUMN IF NOT EXISTS calorie_range_min INTEGER;
ALTER TABLE menus ADD COLUMN IF NOT EXISTS calorie_range_max INTEGER;

ALTER TABLE menu_dishes ADD COLUMN IF NOT EXISTS amount VARCHAR(100);
ALTER TABLE menu_dishes ADD COLUMN IF NOT EXISTS calories INTEGER;
