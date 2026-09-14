-- ==============================================================================
-- KEPÇE VERİTABANI DOKÜMANTASYONU: 02_IDENTITY (Kullanıcı, Yetki ve Oturumlar)
-- ==============================================================================
-- Bu dosya kullanıcı kimlik doğrulama, oturum yönetimi, kullanıcı tercihleri
-- ve rozet/başarım tablolarını tanımlar.
-- ==============================================================================

-- 1. Tipler ve Numaralandırmalar (Enums)
CREATE TYPE user_role_enum AS ENUM ('user', 'admin', 'system_bot');
CREATE TYPE account_status_enum AS ENUM ('active', 'suspended', 'banned');

-- 2. Kullanıcılar (Users)
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    username VARCHAR(50) NOT NULL UNIQUE,
    email VARCHAR(255) NOT NULL UNIQUE,
    password_hash VARCHAR(255) NOT NULL,
    role user_role_enum NOT NULL DEFAULT 'user',
    account_status account_status_enum NOT NULL DEFAULT 'active',
    karma_score INTEGER NOT NULL DEFAULT 0,
    is_verified BOOLEAN NOT NULL DEFAULT false,
    avatar_url VARCHAR(512),
    bio TEXT,
    default_city_slug VARCHAR(255) REFERENCES cities(slug) ON DELETE SET NULL,
    level INTEGER NOT NULL DEFAULT 1,
    level_progress INTEGER NOT NULL DEFAULT 0,
    google_id VARCHAR(255) UNIQUE,
    token_version INTEGER NOT NULL DEFAULT 0,
    opt_out_statistics BOOLEAN NOT NULL DEFAULT false,
    breakfast_notification_enabled BOOLEAN NOT NULL DEFAULT true,
    lunch_notification_enabled BOOLEAN NOT NULL DEFAULT true,
    dinner_notification_enabled BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- 3. Kullanıcı Oturumları (User Sessions - Stateful Refresh/Auth)
CREATE TABLE user_sessions (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    ip_address VARCHAR(45),
    user_agent TEXT,
    last_used_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- 4. Kullanılmış / İptal Edilmiş Belirteçler (Blacklisted / Used Tokens)
CREATE TABLE used_tokens (
    id VARCHAR(255) PRIMARY KEY,
    token_hash VARCHAR(255) NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- 5. Kullanıcı Favori Yemekleri (User Favorites)
CREATE TABLE user_favorites (
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    dish_id INTEGER NOT NULL REFERENCES dishes(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (user_id, dish_id)
);

-- 6. Sabitlenmiş Yemekler (User Pinned Dishes)
CREATE TABLE user_pinned_dishes (
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    dish_id INTEGER NOT NULL REFERENCES dishes(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (user_id, dish_id)
);

-- 7. Engellenen Kullanıcılar (User Blocks)
CREATE TABLE user_blocks (
    blocker_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    blocked_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (blocker_id, blocked_id)
);

-- 8. Kullanıcı Uyarıları (User Warnings)
CREATE TABLE user_warnings (
    id SERIAL PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    message TEXT NOT NULL,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- 9. Rozetler (Badges)
CREATE TABLE badges (
    id SERIAL PRIMARY KEY,
    name VARCHAR(100) NOT NULL UNIQUE,
    slug VARCHAR(100) UNIQUE,
    description TEXT,
    icon_url VARCHAR(255),
    category VARCHAR(50) DEFAULT 'general',
    sort_order INTEGER DEFAULT 0,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- 10. Kullanıcı Rozetleri (User Badges)
CREATE TABLE user_badges (
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    badge_id INTEGER NOT NULL REFERENCES badges(id) ON DELETE CASCADE,
    awarded_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (user_id, badge_id)
);
