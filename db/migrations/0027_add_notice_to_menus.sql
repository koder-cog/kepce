-- Menülere günün özel uyarısını saklamak için notice sütunu eklenmesi
ALTER TABLE menus ADD COLUMN IF NOT EXISTS notice TEXT;
