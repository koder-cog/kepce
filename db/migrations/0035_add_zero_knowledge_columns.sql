-- 0035_add_zero_knowledge_columns.sql
-- users tablosu hash sütunları ve 5651 trafik günlüğü
-- Bu betik yalnızca DDL ekleme işlemlerini yapar; var olan açık verileri silmez.

-- 1. users tablosuna kör indeks ve takma ad sütunları eklenir
ALTER TABLE users
    ALTER COLUMN email DROP NOT NULL,
    ADD COLUMN IF NOT EXISTS email_hash VARCHAR(64) NULL,
    ADD COLUMN IF NOT EXISTS auth_hash VARCHAR(64) NULL,
    ADD COLUMN IF NOT EXISTS nickname VARCHAR(50) NULL,
    ADD COLUMN IF NOT EXISTS report_trust NUMERIC(5,2) NOT NULL DEFAULT 1.0;

-- 2. Kör indeks ve kimlik doğrulama hash indeksleri kurulur
CREATE UNIQUE INDEX IF NOT EXISTS idx_users_email_hash
    ON users (email_hash)
    WHERE email_hash IS NOT NULL;

CREATE UNIQUE INDEX IF NOT EXISTS idx_users_auth_hash
    ON users (auth_hash)
    WHERE auth_hash IS NOT NULL;

-- 3. user_sessions tablosuna alt ağ karma sütunu eklenir
ALTER TABLE user_sessions
    ADD COLUMN IF NOT EXISTS subnet_hash VARCHAR(64) NULL;

-- 4. 5651 sayılı kanun gereği RAM'de X25519 ile mühürlenen trafik kayıt tablosu
CREATE TABLE IF NOT EXISTS traffic_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    key_id INTEGER NOT NULL DEFAULT 1,
    service_type VARCHAR(32) NOT NULL DEFAULT 'comment',
    reference_id UUID NULL,
    encrypted_payload TEXT NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

ALTER TABLE traffic_logs
    ADD COLUMN IF NOT EXISTS service_type VARCHAR(32) NOT NULL DEFAULT 'comment',
    ADD COLUMN IF NOT EXISTS reference_id UUID NULL;

CREATE INDEX IF NOT EXISTS idx_traffic_logs_created_at
    ON traffic_logs (created_at);

CREATE INDEX IF NOT EXISTS idx_traffic_logs_reference_id
    ON traffic_logs (reference_id);
