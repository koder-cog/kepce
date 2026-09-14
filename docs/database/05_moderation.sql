-- ==============================================================================
-- KEPÇE VERİTABANI DOKÜMANTASYONU: 05_MODERATION (Şikayetler, Bildirimler ve İletişim)
-- ==============================================================================
-- Bu dosya içerik şikayetleri (reports), iletişim mesajları (contact_messages),
-- uygulama içi bildirimler ve Web Push abonelik tablolarını tanımlar.
-- ==============================================================================

-- 1. Tipler ve Numaralandırmalar (Enums)
CREATE TYPE report_status_enum AS ENUM ('pending', 'resolved', 'dismissed');

-- 2. Şikayetler (Reports - Menü, Yorum veya Kullanıcı Şikayetleri)
CREATE TABLE reports (
    id SERIAL PRIMARY KEY,
    reporter_id UUID REFERENCES users(id) ON DELETE CASCADE,
    reported_comment_id UUID REFERENCES comments(id) ON DELETE CASCADE,
    reported_user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    menu_id INTEGER REFERENCES menus(id) ON DELETE CASCADE,
    reason TEXT,
    type VARCHAR(50),
    description TEXT,
    status report_status_enum NOT NULL DEFAULT 'pending',
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    resolved_at TIMESTAMPTZ
);

-- 3. İletişim Formu Mesajları (Contact Messages)
CREATE TABLE contact_messages (
    id SERIAL PRIMARY KEY,
    user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    email VARCHAR(255) NOT NULL,
    category VARCHAR(50) NOT NULL,
    subject VARCHAR(150) NOT NULL,
    message TEXT NOT NULL,
    status report_status_enum NOT NULL DEFAULT 'pending',
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    resolved_at TIMESTAMPTZ
);

-- 4. Uygulama İçi Bildirimler (In-App Notifications)
CREATE TABLE notifications (
    id SERIAL PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    type VARCHAR(50) NOT NULL,
    title VARCHAR(255) NOT NULL,
    message TEXT NOT NULL,
    is_read BOOLEAN NOT NULL DEFAULT false,
    action_label VARCHAR(100),
    action_href VARCHAR(255),
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- 5. Tarayıcı Web Push Abonelikleri (Web Push Subscriptions)
CREATE TABLE push_subscriptions (
    id SERIAL PRIMARY KEY,
    user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    city_id INTEGER REFERENCES cities(id) ON DELETE SET NULL,
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
