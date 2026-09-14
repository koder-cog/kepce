-- ==============================================================================
-- KEPÇE VERİTABANI DOKÜMANTASYONU: 01_CORE (Temel Varlıklar)
-- ==============================================================================
-- Bu dosya Kepçe'nin temel coğrafi ve yemek varlıklarını tanımlar.
-- ==============================================================================

-- 1. Şehirler
CREATE TABLE cities (
    id SERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    slug VARCHAR(255) NOT NULL UNIQUE
);

-- 2. Ana Yemekler (Master Dish Catalog)
CREATE TABLE dishes (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    category VARCHAR(100),
    parent_id INTEGER REFERENCES dishes(id) ON DELETE SET NULL,
    is_celiac BOOLEAN NOT NULL DEFAULT false,
    is_vegan BOOLEAN NOT NULL DEFAULT false,
    is_vegetarian BOOLEAN NOT NULL DEFAULT false,
    estimated_calories INTEGER,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- 3. Yemek Takma Adları (Scraper ve Kaynak Eşleştirmeleri)
CREATE TABLE dish_aliases (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    dish_id INTEGER REFERENCES dishes(id) ON DELETE SET NULL,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- 4. Etiketler
CREATE TABLE tags (
    id SERIAL PRIMARY KEY,
    name VARCHAR(100) NOT NULL UNIQUE,
    slug VARCHAR(100) NOT NULL UNIQUE,
    category VARCHAR(50) NOT NULL DEFAULT 'general',
    sort_order INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- 5. Yemek - Etiket Çoka-Çok İlişkisi
CREATE TABLE dish_tags (
    dish_id INTEGER NOT NULL REFERENCES dishes(id) ON DELETE CASCADE,
    tag_id INTEGER NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (dish_id, tag_id)
);
