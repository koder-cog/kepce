ALTER TABLE users ADD COLUMN IF NOT EXISTS notif_replies BOOLEAN NOT NULL DEFAULT false;
ALTER TABLE users ADD COLUMN IF NOT EXISTS notif_interactions BOOLEAN NOT NULL DEFAULT false;
ALTER TABLE users ADD COLUMN IF NOT EXISTS notif_system BOOLEAN NOT NULL DEFAULT false;
ALTER TABLE users ADD COLUMN IF NOT EXISTS email_newsletter BOOLEAN NOT NULL DEFAULT false;
ALTER TABLE users ADD COLUMN IF NOT EXISTS email_security BOOLEAN NOT NULL DEFAULT false;
ALTER TABLE users ADD COLUMN IF NOT EXISTS email_updates BOOLEAN NOT NULL DEFAULT false;

-- Eğer kolonlar daha önce DEFAULT true ile oluşturulduysa varsayılanları false yap
ALTER TABLE users ALTER COLUMN notif_replies SET DEFAULT false;
ALTER TABLE users ALTER COLUMN notif_interactions SET DEFAULT false;
ALTER TABLE users ALTER COLUMN notif_system SET DEFAULT false;
ALTER TABLE users ALTER COLUMN email_security SET DEFAULT false;
