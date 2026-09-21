-- 0030_add_submission_storage_ref.sql
-- Menü gönderimlerinin karantina dizin referansını (UUID) kalıcı hale getirir.
-- Onay sonrası dosyaların data/menuler altına taşınabilmesi için gereklidir;
-- aksi halde DB kaydından karantina dizinine geri dönüş yolu yoktur.

ALTER TABLE menu_submissions
    ADD COLUMN IF NOT EXISTS storage_ref TEXT;

COMMENT ON COLUMN menu_submissions.storage_ref IS 'Karantina dizin adı (UUID). Onay sonrası dosya taşıma için kullanılır.';