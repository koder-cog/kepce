-- ==============================================================================
-- KEPÇE VERİTABANI DOKÜMANTASYONU: 07_INDEXES (Tetikleyiciler, Kısıtlar ve İndeksler)
-- ==============================================================================
-- Bu dosya Kepçe veritabanı genelindeki veri bütünlüğü tetikleyicilerini,
-- kısmi (partial) tekillik indekslerini, yabancı anahtar (FK) kapsama indekslerini
-- ve zaman serisi / arama performans indekslerini tanımlar.
-- ==============================================================================

-- ==============================================================================
-- 1. BÜTÜNLÜK TETİKLEYİCİLERİ (INTEGRITY TRIGGERS)
-- ==============================================================================

-- Aynı menü yuvasında aynı yemeğin (farklı alias ile dahi olsa) yinelenmesini önler
CREATE OR REPLACE FUNCTION trg_check_menu_dishes_unique_dish_per_slot()
RETURNS TRIGGER AS $$
DECLARE
    new_dish_id INT;
    existing_count INT;
BEGIN
    SELECT dish_id INTO new_dish_id FROM dish_aliases WHERE id = NEW.dish_alias_id;
    
    IF new_dish_id IS NOT NULL THEN
        SELECT COUNT(*) INTO existing_count
        FROM menu_dishes md
        JOIN dish_aliases da ON md.dish_alias_id = da.id
        WHERE md.menu_id = NEW.menu_id
          AND md.package_name = NEW.package_name
          AND md.order_index = NEW.order_index
          AND da.dish_id = new_dish_id
          AND (TG_OP = 'INSERT' OR md.id != NEW.id);
          
        IF existing_count > 0 THEN
            RAISE EXCEPTION 'Bu menü yuvasında (%) aynı yemek zaten kayıtlı (dish_id: %)', NEW.order_index, new_dish_id;
        END IF;
    END IF;
    
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS trg_menu_dishes_unique_dish ON menu_dishes;
CREATE TRIGGER trg_menu_dishes_unique_dish
BEFORE INSERT OR UPDATE ON menu_dishes
FOR EACH ROW
EXECUTE FUNCTION trg_check_menu_dishes_unique_dish_per_slot();

-- ==============================================================================
-- 2. ÖZEL VE KISMİ TEKİLLİK İNDEKSLERİ (SPECIAL & PARTIAL UNIQUE INDEXES)
-- ==============================================================================

-- Yemek isimlerinde büyük/küçük harf duyarsız benzersizlik
CREATE UNIQUE INDEX IF NOT EXISTS uq_dishes_lower_name ON dishes (LOWER(TRIM(name)));

-- Bir menü paketindeki her yuvada (order_index) yalnızca tek bir asıl yemek olabilir (alternatifler hariç)
CREATE UNIQUE INDEX IF NOT EXISTS uq_menu_dishes_primary_slot ON menu_dishes (menu_id, package_name, order_index) WHERE (is_alternative = false);

-- ==============================================================================
-- 3. YABANCI ANAHTAR VE PERFORMANS İNDEKSLERİ
-- ==============================================================================

-- 3.1. Temel Varlıklar (Core Domain)
CREATE INDEX IF NOT EXISTS idx_dishes_parent_id ON dishes (parent_id);
CREATE INDEX IF NOT EXISTS idx_dishes_category ON dishes (category);
CREATE INDEX IF NOT EXISTS idx_dish_aliases_dish_id ON dish_aliases (dish_id);
CREATE INDEX IF NOT EXISTS idx_dish_tags_tag_id ON dish_tags (tag_id);

-- 3.2. Menüler ve Fiyatlandırma (Menus & Pricing Domain)
CREATE INDEX IF NOT EXISTS idx_menu_dishes_menu_id ON menu_dishes (menu_id);
CREATE INDEX IF NOT EXISTS idx_menu_dishes_dish_alias_id ON menu_dishes (dish_alias_id);
CREATE INDEX IF NOT EXISTS idx_menus_serve_date ON menus (serve_date);
CREATE INDEX IF NOT EXISTS idx_menus_city_id ON menus (city_id);
CREATE INDEX IF NOT EXISTS idx_menus_status ON menus (status);
CREATE INDEX IF NOT EXISTS idx_menus_submitted_by ON menus (submitted_by);
CREATE INDEX IF NOT EXISTS idx_menu_history_city_serve_date ON menu_history (city_id, serve_date);
CREATE INDEX IF NOT EXISTS idx_menu_history_source_type ON menu_history (source_type);
CREATE INDEX IF NOT EXISTS idx_menu_submissions_city_slug ON menu_submissions (city_slug);
CREATE INDEX IF NOT EXISTS idx_menu_submissions_status ON menu_submissions (status);
CREATE INDEX IF NOT EXISTS idx_pricing_periods_city_slug ON pricing_periods (city_slug);
CREATE INDEX IF NOT EXISTS idx_meal_category_prices_period_id ON meal_category_prices (pricing_period_id);

-- 3.3. Kimlik ve Tercihler (Identity & Preferences Domain)
CREATE INDEX IF NOT EXISTS idx_user_sessions_user_id ON user_sessions (user_id);
CREATE INDEX IF NOT EXISTS idx_user_sessions_expires_at ON user_sessions (expires_at);
CREATE INDEX IF NOT EXISTS idx_used_tokens_expires_at ON used_tokens (expires_at);
CREATE INDEX IF NOT EXISTS idx_user_favorites_dish_id ON user_favorites (dish_id);
CREATE INDEX IF NOT EXISTS idx_user_pinned_dishes_dish_id ON user_pinned_dishes (dish_id);
CREATE INDEX IF NOT EXISTS idx_badges_slug ON badges (slug);
CREATE INDEX IF NOT EXISTS idx_badges_category ON badges (category);
CREATE INDEX IF NOT EXISTS idx_user_badges_badge_id ON user_badges (badge_id);

-- 3.4. Sosyal Etkileşim (Social Domain)
CREATE INDEX IF NOT EXISTS idx_comments_menu_id ON comments (menu_id);
CREATE INDEX IF NOT EXISTS idx_comments_user_id ON comments (user_id);
CREATE INDEX IF NOT EXISTS idx_comments_dish_id ON comments (dish_id);
CREATE INDEX IF NOT EXISTS idx_comments_parent_id ON comments (parent_id);
CREATE INDEX IF NOT EXISTS idx_vote_reactions_comment_id ON vote_reactions (comment_id);
CREATE INDEX IF NOT EXISTS idx_vote_reactions_user_id ON vote_reactions (user_id);
CREATE INDEX IF NOT EXISTS idx_menu_votes_menu_id ON menu_votes (menu_id);
CREATE INDEX IF NOT EXISTS idx_menu_votes_user_id ON menu_votes (user_id);
CREATE INDEX IF NOT EXISTS idx_dish_votes_dish_id ON dish_votes (dish_id);
CREATE INDEX IF NOT EXISTS idx_dish_votes_menu_id ON dish_votes (menu_id);
CREATE INDEX IF NOT EXISTS idx_dish_votes_user_id ON dish_votes (user_id);

-- 3.5. Moderasyon ve Bildirimler (Moderation Domain)
CREATE INDEX IF NOT EXISTS idx_reports_status ON reports (status);
CREATE INDEX IF NOT EXISTS idx_reports_reporter_id ON reports (reporter_id);
CREATE INDEX IF NOT EXISTS idx_reports_reported_user_id ON reports (reported_user_id);
CREATE INDEX IF NOT EXISTS idx_reports_reported_comment_id ON reports (reported_comment_id);
CREATE INDEX IF NOT EXISTS idx_reports_menu_id ON reports (menu_id);
CREATE INDEX IF NOT EXISTS idx_contact_messages_status ON contact_messages (status);
CREATE INDEX IF NOT EXISTS idx_contact_messages_user_id ON contact_messages (user_id);
CREATE INDEX IF NOT EXISTS idx_notifications_user_id_created ON notifications (user_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_push_subs_user_id ON push_subscriptions (user_id);
CREATE INDEX IF NOT EXISTS idx_push_subs_city_id ON push_subscriptions (city_id);
CREATE INDEX IF NOT EXISTS idx_push_subs_endpoint ON push_subscriptions (endpoint);

-- 3.6. Platform Servisleri (Platform Domain)
CREATE INDEX IF NOT EXISTS idx_api_keys_project_id ON api_keys (project_id);
CREATE INDEX IF NOT EXISTS idx_api_keys_user_id ON api_keys (user_id);
CREATE INDEX IF NOT EXISTS idx_api_usage_logs_key_date ON api_usage_logs (api_key_id, date);
