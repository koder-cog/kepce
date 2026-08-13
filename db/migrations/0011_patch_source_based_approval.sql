-- 1. Add submitted_by to menus
ALTER TABLE menus ADD COLUMN IF NOT EXISTS submitted_by UUID;

DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM information_schema.table_constraints WHERE constraint_name = 'fk_menus_submitted_by') THEN
        ALTER TABLE menus ADD CONSTRAINT fk_menus_submitted_by FOREIGN KEY (submitted_by) REFERENCES users(id) ON DELETE SET NULL;
    END IF;
END$$;

-- 2. Create menu_history table
CREATE TABLE IF NOT EXISTS menu_history (
    id SERIAL PRIMARY KEY,
    city_id INTEGER NOT NULL REFERENCES cities(id) ON DELETE CASCADE,
    serve_date DATE NOT NULL,
    meal_type VARCHAR(50) NOT NULL,
    source_type VARCHAR(50) NOT NULL,
    submitted_by UUID,
    dishes_payload JSONB NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM information_schema.table_constraints WHERE constraint_name = 'fk_menu_history_submitted_by') THEN
        ALTER TABLE menu_history ADD CONSTRAINT fk_menu_history_submitted_by FOREIGN KEY (submitted_by) REFERENCES users(id) ON DELETE SET NULL;
    END IF;
END$$;
