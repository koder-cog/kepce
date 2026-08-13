-- 0002_create_pricing_periods.sql
-- Dönemsel fiyatlandırma tabloları ve yemek kalori tahmini kolonu

CREATE TABLE IF NOT EXISTS pricing_periods (
    id SERIAL PRIMARY KEY,
    city_slug VARCHAR(255) NOT NULL REFERENCES cities(slug) ON DELETE CASCADE,
    period_start DATE NOT NULL,
    period_end DATE NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(city_slug, period_start, period_end)
);

CREATE TABLE IF NOT EXISTS meal_category_prices (
    id SERIAL PRIMARY KEY,
    pricing_period_id INTEGER NOT NULL REFERENCES pricing_periods(id) ON DELETE CASCADE,
    meal_type VARCHAR(50) NOT NULL, -- 'breakfast', 'lunch', 'dinner'
    category_name VARCHAR(255) NOT NULL,
    portion_amount VARCHAR(100),
    price NUMERIC(10,2) NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(pricing_period_id, meal_type, category_name)
);

ALTER TABLE dishes ADD COLUMN IF NOT EXISTS estimated_calories INTEGER;
