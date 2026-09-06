-- 0022_enhance_badges_schema.sql
-- Rozet ve kullanıcı rozet şemasını genişletme

ALTER TABLE badges 
    ADD COLUMN IF NOT EXISTS slug VARCHAR(50) UNIQUE,
    ADD COLUMN IF NOT EXISTS category VARCHAR(30) NOT NULL DEFAULT 'sadakat',
    ADD COLUMN IF NOT EXISTS karma_reward INT NOT NULL DEFAULT 0,
    ADD COLUMN IF NOT EXISTS icon VARCHAR(50) NOT NULL DEFAULT 'starFilled',
    ADD COLUMN IF NOT EXISTS is_repeatable BOOLEAN NOT NULL DEFAULT false;

ALTER TABLE user_badges
    ADD COLUMN IF NOT EXISTS count INT NOT NULL DEFAULT 1;

CREATE INDEX IF NOT EXISTS idx_badges_slug ON badges(slug);
CREATE INDEX IF NOT EXISTS idx_badges_category ON badges(category);
