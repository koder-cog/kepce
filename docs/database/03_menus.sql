-- ==============================================================================
-- KEPÇE VERİTABANI DOKÜMANTASYONU: 03_MENUS (Menüler, Yemek Eşleştirmeleri ve Fiyatlandırma)
-- ==============================================================================
-- Bu dosya KYK yurtlarının günlük tabldot menülerini, servis yuvalarını,
-- menü değişiklik geçmişini, kullanıcı menü gönderimlerini ve dinamik fiyat
-- dönemlerini tanımlar.
-- ==============================================================================

-- 1. Tipler ve Numaralandırmalar (Enums)
CREATE TYPE meal_type_enum AS ENUM ('breakfast', 'lunch', 'dinner');
CREATE TYPE menu_status_enum AS ENUM ('pending', 'approved', 'rejected');

-- 2. Günlük Menüler (Daily Menus)
CREATE TABLE menus (
    id SERIAL PRIMARY KEY,
    city_id INTEGER NOT NULL REFERENCES cities(id) ON DELETE CASCADE,
    serve_date DATE NOT NULL,
    meal_type meal_type_enum NOT NULL,
    merkle_root VARCHAR(64),
    previous_hash VARCHAR(64),
    source_type VARCHAR(50) DEFAULT 'unknown',
    submitted_by UUID REFERENCES users(id) ON DELETE SET NULL,
    status menu_status_enum NOT NULL DEFAULT 'pending',
    bot_commentary TEXT,
    min_calories INTEGER,
    max_calories INTEGER,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(city_id, serve_date, meal_type)
);

-- 3. Menü Tabakları / Yuvaları (Menu Dishes - Slots & Packages)
CREATE TABLE menu_dishes (
    id SERIAL PRIMARY KEY,
    menu_id INTEGER NOT NULL REFERENCES menus(id) ON DELETE CASCADE,
    dish_alias_id INTEGER NOT NULL REFERENCES dish_aliases(id) ON DELETE RESTRICT,
    order_index INTEGER NOT NULL DEFAULT 0,
    is_alternative BOOLEAN NOT NULL DEFAULT false,
    package_name VARCHAR(50) NOT NULL DEFAULT 'NORMAL',
    UNIQUE(menu_id, dish_alias_id, package_name)
);

-- 4. Menü Değişiklik ve Kaynak Geçmişi (Audit / History Log)
CREATE TABLE menu_history (
    id SERIAL PRIMARY KEY,
    city_id INTEGER NOT NULL REFERENCES cities(id) ON DELETE CASCADE,
    serve_date DATE NOT NULL,
    meal_type VARCHAR(50) NOT NULL,
    source_type VARCHAR(50) NOT NULL,
    submitted_by UUID REFERENCES users(id) ON DELETE SET NULL,
    dishes_payload JSONB NOT NULL,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- 5. Topluluk Menü Başvuruları (User Menu Submissions)
CREATE TABLE menu_submissions (
    id SERIAL PRIMARY KEY,
    user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    city_slug VARCHAR(255) NOT NULL REFERENCES cities(slug) ON DELETE CASCADE,
    year INTEGER NOT NULL,
    month INTEGER NOT NULL,
    notes TEXT,
    status VARCHAR(50) NOT NULL DEFAULT 'pending',
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- 6. Fiyatlandırma Dönemleri (Pricing Periods - Şehir / Tarih Bazlı)
CREATE TABLE pricing_periods (
    id SERIAL PRIMARY KEY,
    city_slug VARCHAR(255) NOT NULL REFERENCES cities(slug) ON DELETE CASCADE,
    start_date DATE NOT NULL,
    end_date DATE,
    notes TEXT,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- 7. Kategori Bazlı Öğün Fiyatları (Meal Category Prices)
CREATE TABLE meal_category_prices (
    id SERIAL PRIMARY KEY,
    pricing_period_id INTEGER NOT NULL REFERENCES pricing_periods(id) ON DELETE CASCADE,
    category VARCHAR(100) NOT NULL,
    price NUMERIC(10, 2) NOT NULL,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);
