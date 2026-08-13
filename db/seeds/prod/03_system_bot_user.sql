-- 03_system_bot_user.sql
-- Sistem Bot Kullanıcısı Seed Verisi

BEGIN;

-- Kepçe Bot (system_bot)
INSERT INTO users (id, username, email, password_hash, role, is_verified, account_status, karma_score, level, level_progress)
SELECT gen_random_uuid(), 'kepce_bot', 'bot@kepce.org', gen_random_uuid()::text, 'system_bot', true, 'active', 1000, 100, 0
WHERE NOT EXISTS (
    SELECT 1 FROM users WHERE username = 'kepce_bot' OR email = 'bot@kepce.org'
);

COMMIT;
