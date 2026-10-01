-- 0032_create_takeaway_schema.sql
-- Al Götür paket ve slot seçim mimarisi

CREATE TABLE IF NOT EXISTS takeaway_packages (
    id SERIAL PRIMARY KEY,
    city_slug VARCHAR(255) NOT NULL REFERENCES cities(slug) ON DELETE CASCADE,
    package_name VARCHAR(255) NOT NULL,
    academic_year VARCHAR(50) NOT NULL DEFAULT '2026-2027',
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(city_slug, package_name, academic_year)
);

CREATE TABLE IF NOT EXISTS takeaway_slots (
    id SERIAL PRIMARY KEY,
    package_id INTEGER NOT NULL REFERENCES takeaway_packages(id) ON DELETE CASCADE,
    slot_index INTEGER NOT NULL,
    slot_title VARCHAR(255),
    is_required BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(package_id, slot_index)
);

CREATE TABLE IF NOT EXISTS takeaway_slot_items (
    id SERIAL PRIMARY KEY,
    slot_id INTEGER NOT NULL REFERENCES takeaway_slots(id) ON DELETE CASCADE,
    dish_id INTEGER NOT NULL REFERENCES dishes(id) ON DELETE CASCADE,
    portion_override VARCHAR(255),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(slot_id, dish_id)
);

CREATE INDEX IF NOT EXISTS idx_takeaway_packages_city ON takeaway_packages(city_slug);
CREATE INDEX IF NOT EXISTS idx_takeaway_slots_pkg ON takeaway_slots(package_id);
CREATE INDEX IF NOT EXISTS idx_takeaway_slot_items_slot ON takeaway_slot_items(slot_id);
