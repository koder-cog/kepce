-- ============================================================================
-- 0036_zero_knowledge_identity_v2.sql
-- USERS TABLOSU GÖÇÜ (NICKNAME VE RECOVERY KEY)
-- ============================================================================

-- 1. Mevcut 'username' sütununu 'nickname' olarak taşı/adlandır:
DO $$
BEGIN
    IF EXISTS (
        SELECT 1 FROM information_schema.columns 
        WHERE table_name = 'users' AND column_name = 'username'
    ) THEN
        IF NOT EXISTS (
            SELECT 1 FROM information_schema.columns 
            WHERE table_name = 'users' AND column_name = 'nickname'
        ) THEN
            ALTER TABLE users RENAME COLUMN username TO nickname;
        ELSE
            UPDATE users SET nickname = username WHERE nickname IS NULL;
            ALTER TABLE users DROP COLUMN IF EXISTS username;
        END IF;
    END IF;
END $$;

-- 2. Yeni profil ve kurtarma alanlarını kur
ALTER TABLE users
    ADD COLUMN IF NOT EXISTS nickname VARCHAR(50),
    ADD COLUMN IF NOT EXISTS recovery_key_hash VARCHAR(255),
    ADD COLUMN IF NOT EXISTS is_verified BOOLEAN NOT NULL DEFAULT false,
    ADD COLUMN IF NOT EXISTS avatar_url VARCHAR(512),
    ADD COLUMN IF NOT EXISTS report_trust NUMERIC(5,2) NOT NULL DEFAULT 1.0;

-- nickname sütununu NOT NULL yap
ALTER TABLE users ALTER COLUMN nickname SET NOT NULL;

-- 3. Eski kişisel veri, OAuth, rıza ve e-posta tercih sütunlarını kaldır
-- (Not: avatar_url korunmaktadır)
ALTER TABLE users
    DROP COLUMN IF EXISTS username,
    DROP COLUMN IF EXISTS email,
    DROP COLUMN IF EXISTS email_hash,
    DROP COLUMN IF EXISTS auth_hash,
    DROP COLUMN IF EXISTS google_id,
    DROP COLUMN IF EXISTS bio,
    DROP COLUMN IF EXISTS email_newsletter,
    DROP COLUMN IF EXISTS email_security,
    DROP COLUMN IF EXISTS email_updates,
    DROP COLUMN IF EXISTS consent_cross_border,
    DROP COLUMN IF EXISTS consent_cross_border_at,
    DROP COLUMN IF EXISTS consent_deadline_at;

-- 4. Henüz kurtarma kodu onaylamamış mevcut hesapları onaysız duruma al
UPDATE users
SET is_verified = false
WHERE recovery_key_hash IS NULL;

-- 5. İndeksler
DROP INDEX IF EXISTS idx_users_email_hash;
DROP INDEX IF EXISTS idx_users_auth_hash;
DROP INDEX IF EXISTS idx_users_cross_border_pending;
CREATE UNIQUE INDEX IF NOT EXISTS idx_users_nickname ON users(nickname);

-- 6. Sabit diskte tutulmayacak olan eski traffic_logs tablosunu temizle
DROP TABLE IF EXISTS traffic_logs CASCADE;

