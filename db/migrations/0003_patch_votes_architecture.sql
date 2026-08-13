CREATE TABLE IF NOT EXISTS menu_votes (
    id SERIAL PRIMARY KEY,
    menu_id INT NOT NULL REFERENCES menus(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    sentiment sentiment_enum NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(menu_id, user_id)
);

CREATE TABLE IF NOT EXISTS dish_votes (
    id SERIAL PRIMARY KEY,
    dish_id INT NOT NULL REFERENCES dishes(id) ON DELETE CASCADE,
    menu_id INT NOT NULL REFERENCES menus(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    sentiment sentiment_enum NOT NULL,
    is_explicit BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(dish_id, menu_id, user_id)
);

-- Indices for fast querying
CREATE INDEX IF NOT EXISTS idx_menu_votes_menu_id ON menu_votes(menu_id);
CREATE INDEX IF NOT EXISTS idx_dish_votes_dish_id ON dish_votes(dish_id);
CREATE INDEX IF NOT EXISTS idx_dish_votes_menu_id ON dish_votes(menu_id);
