-- 0033_add_cross_border_consent_fields.sql
-- KVKK Madde 9 Yurt Dışı Sunucu Barındırma Rızası ve Mühlet Yönetimi

ALTER TABLE users
    ADD COLUMN IF NOT EXISTS consent_cross_border BOOLEAN NOT NULL DEFAULT FALSE,
    ADD COLUMN IF NOT EXISTS consent_cross_border_at TIMESTAMP WITH TIME ZONE NULL,
    ADD COLUMN IF NOT EXISTS consent_deadline_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT (CURRENT_TIMESTAMP + INTERVAL '30 days');

-- Halihazırda kayıtlı kullanıcılara 30 günlük mühlet başlangıcı
UPDATE users
SET consent_deadline_at = CURRENT_TIMESTAMP + INTERVAL '30 days'
WHERE consent_cross_border = FALSE;

CREATE INDEX IF NOT EXISTS idx_users_cross_border_pending
    ON users (consent_deadline_at)
    WHERE consent_cross_border = FALSE;
