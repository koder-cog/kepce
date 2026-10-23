-- db/scripts/manual/0036_drop_plain_identity_columns.sql
-- ==============================================================================
-- DİKKAT: Bu betik ASLA otomatik migration mekanizması ile çalıştırılmamalıdır!
-- Yalnızca 'tools/migrate-identity' aracı çalıştırılıp açık e-posta sayısı kesinlikle
-- 0 olarak teyit edildikten sonra podman stop kepce-api esnasında manuel tetiklenir.
-- ==============================================================================

BEGIN;

-- 1. users tablosundaki açık kişisel verileri ve eski KVKK yurt dışı rıza alanlarını kaldır
ALTER TABLE users
    DROP COLUMN IF EXISTS email,
    DROP COLUMN IF EXISTS google_id,
    DROP COLUMN IF EXISTS avatar_url,
    DROP COLUMN IF EXISTS bio,
    DROP COLUMN IF EXISTS consent_cross_border,
    DROP COLUMN IF EXISTS consent_cross_border_at,
    DROP COLUMN IF EXISTS consent_deadline_at;

-- 2. Eski rıza takip indeksini kaldır
DROP INDEX IF EXISTS idx_users_cross_border_pending;

-- 3. user_sessions tablosundaki açık IP ve User Agent alanlarını kaldır
ALTER TABLE user_sessions
    DROP COLUMN IF EXISTS ip_address,
    DROP COLUMN IF EXISTS user_agent;

COMMIT;

-- ==============================================================================
-- BİLGİ: İşlem tamamlandıktan sonra PostgreSQL oturumunda bağımsız olarak
-- aşağıdaki komutlar koşturulmalıdır:
--   psql -d kepce -c "VACUUM FULL users;"
--   psql -d kepce -c "VACUUM FULL user_sessions;"
--   psql -d kepce -c "SELECT pg_switch_wal(); CHECKPOINT;"
-- ==============================================================================
