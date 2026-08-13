-- 0012_create_used_tokens.sql
-- Tek kullanımlık token'ları (şifresiz giriş, e-posta onayı, şifre sıfırlama) kalıcı kılmak ve
-- sunucu yeniden başladığında tekrar kullanım (replay) saldırılarını engellemek için tablo.

CREATE TABLE IF NOT EXISTS used_tokens (
    token_hash VARCHAR(64) PRIMARY KEY,
    token_type VARCHAR(32) NOT NULL,
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    used_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    expires_at TIMESTAMP WITH TIME ZONE NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_used_tokens_expires_at ON used_tokens(expires_at);
