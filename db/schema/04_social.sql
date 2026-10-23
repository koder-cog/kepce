-- ==============================================================================
-- KEPÇE VERİTABANI DOKÜMANTASYONU: 04_SOCIAL (Yorumlar, Oylar ve Etkileşimler)
-- ==============================================================================
-- Bu dosya menü ve yemek bazlı kullanıcı yorumlarını, silinme durumlarını,
-- oylama (upvote/downvote veya duygu analizi) mekanizmalarını tanımlar.
-- ==============================================================================

-- 1. Tipler ve Numaralandırmalar (Enums)
CREATE TYPE sentiment_enum AS ENUM ('positive', 'negative', 'neutral');
CREATE TYPE reaction_type_enum AS ENUM ('upvote', 'downvote');

-- 2. Yorumlar (Comments - Hiyerarşik ve Yemek Bağlantılı)
CREATE TABLE comments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    menu_id INTEGER NOT NULL REFERENCES menus(id) ON DELETE CASCADE,
    dish_id INTEGER REFERENCES dishes(id) ON DELETE CASCADE,
    parent_id UUID REFERENCES comments(id) ON DELETE CASCADE,
    content TEXT,
    sentiment sentiment_enum NOT NULL DEFAULT 'neutral',
    is_tabldot BOOLEAN NOT NULL DEFAULT false,
    is_deleted BOOLEAN NOT NULL DEFAULT false,
    reason VARCHAR(255),
    deletion_type VARCHAR(50),
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- 3. Yorum Tepkileri (Comment Reactions / Upvotes & Downvotes)
CREATE TABLE vote_reactions (
    id SERIAL PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    comment_id UUID NOT NULL REFERENCES comments(id) ON DELETE CASCADE,
    reaction_type reaction_type_enum NOT NULL,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(user_id, comment_id)
);

-- 4. Menü Oyları (Menu Level Sentiment Votes)
CREATE TABLE menu_votes (
    id SERIAL PRIMARY KEY,
    menu_id INTEGER NOT NULL REFERENCES menus(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    sentiment sentiment_enum NOT NULL,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(menu_id, user_id)
);

-- 5. Yemek Oyları (Dish Level Sentiment Votes)
CREATE TABLE dish_votes (
    id SERIAL PRIMARY KEY,
    dish_id INTEGER NOT NULL REFERENCES dishes(id) ON DELETE CASCADE,
    menu_id INTEGER NOT NULL REFERENCES menus(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    sentiment sentiment_enum NOT NULL,
    is_explicit BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(dish_id, menu_id, user_id)
);
