-- Migration: 0017_create_push_subscriptions.sql
-- Web Push bildirim abonelikleri tablosu

CREATE TABLE IF NOT EXISTS push_subscriptions (
    id SERIAL PRIMARY KEY,
    user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    city_id INT REFERENCES cities(id) ON DELETE SET NULL,
    endpoint TEXT NOT NULL UNIQUE,
    p256dh TEXT NOT NULL,
    auth TEXT NOT NULL,
    notif_breakfast_enabled BOOLEAN NOT NULL DEFAULT true,
    notif_breakfast_time VARCHAR(5) NOT NULL DEFAULT '07:30',
    notif_dinner_enabled BOOLEAN NOT NULL DEFAULT true,
    notif_dinner_time VARCHAR(5) NOT NULL DEFAULT '17:00',
    user_agent TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_push_subs_user_id ON push_subscriptions(user_id);
CREATE INDEX IF NOT EXISTS idx_push_subs_city_id ON push_subscriptions(city_id);
CREATE INDEX IF NOT EXISTS idx_push_subs_endpoint ON push_subscriptions(endpoint);
