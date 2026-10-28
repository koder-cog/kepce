-- ============================================================================
-- 0037_case_insensitive_nickname_index.sql
-- BÜYÜK/KÜÇÜK HARF DUYARSIZ (CASE-INSENSITIVE) RUMUZ BENZERSİZLİK İNDEKSİ
-- ============================================================================

-- Kullanıcı adlarının küçük harfe normalize edilmiş benzersizlik garantisi:
CREATE UNIQUE INDEX IF NOT EXISTS idx_users_nickname_lower ON users (LOWER(nickname));
