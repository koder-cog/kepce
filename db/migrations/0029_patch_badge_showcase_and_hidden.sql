-- 0029_patch_badge_showcase_and_hidden.sql
-- Rozet kademeleri, gizli rozetler ve profil vitrini (pinned_badges) alanları

ALTER TABLE badges 
    ADD COLUMN IF NOT EXISTS is_hidden BOOLEAN NOT NULL DEFAULT false,
    ADD COLUMN IF NOT EXISTS tier VARCHAR(20) NOT NULL DEFAULT 'other';

ALTER TABLE users
    ADD COLUMN IF NOT EXISTS pinned_badges VARCHAR(50)[] DEFAULT '{}';

CREATE INDEX IF NOT EXISTS idx_badges_tier ON badges(tier);
