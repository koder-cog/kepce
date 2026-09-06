-- 0023_fix_cankiri_slug.sql
-- Çankırı ilinin slug yazım hatasını düzeltme (canakri -> cankiri)

UPDATE cities SET slug = 'cankiri' WHERE slug = 'canakri';
