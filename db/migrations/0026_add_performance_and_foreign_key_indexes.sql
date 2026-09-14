-- 0026_add_performance_and_foreign_key_indexes.sql
-- Add essential performance, query, and foreign key lookup indexes
-- to eliminate Full Table Scans on critical tables.

-- 1. Comments and interactions
CREATE INDEX IF NOT EXISTS idx_comments_menu_id ON comments (menu_id);
CREATE INDEX IF NOT EXISTS idx_comments_user_id ON comments (user_id);
CREATE INDEX IF NOT EXISTS idx_comments_dish_id ON comments (dish_id);
CREATE INDEX IF NOT EXISTS idx_comments_parent_id ON comments (parent_id);

-- 2. Dish aliases and menu dishes
CREATE INDEX IF NOT EXISTS idx_dish_aliases_dish_id ON dish_aliases (dish_id);
CREATE INDEX IF NOT EXISTS idx_menu_dishes_dish_alias_id ON menu_dishes (dish_alias_id);
CREATE INDEX IF NOT EXISTS idx_menu_dishes_menu_id ON menu_dishes (menu_id);

-- 3. Menus and history lookups
CREATE INDEX IF NOT EXISTS idx_menus_serve_date ON menus (serve_date);
CREATE INDEX IF NOT EXISTS idx_menus_city_id ON menus (city_id);
CREATE INDEX IF NOT EXISTS idx_menus_status ON menus (status);
CREATE INDEX IF NOT EXISTS idx_menus_submitted_by ON menus (submitted_by);
CREATE INDEX IF NOT EXISTS idx_menu_history_city_serve_date ON menu_history (city_id, serve_date);
CREATE INDEX IF NOT EXISTS idx_menu_history_source_type ON menu_history (source_type);

-- 4. User sessions and relations
CREATE INDEX IF NOT EXISTS idx_user_sessions_user_id ON user_sessions (user_id);
CREATE INDEX IF NOT EXISTS idx_user_sessions_expires_at ON user_sessions (expires_at);
CREATE INDEX IF NOT EXISTS idx_user_favorites_dish_id ON user_favorites (dish_id);
CREATE INDEX IF NOT EXISTS idx_user_pinned_dishes_dish_id ON user_pinned_dishes (dish_id);
CREATE INDEX IF NOT EXISTS idx_user_badges_badge_id ON user_badges (badge_id);

-- 5. Dishes hierarchy and categories
CREATE INDEX IF NOT EXISTS idx_dishes_parent_id ON dishes (parent_id);
CREATE INDEX IF NOT EXISTS idx_dishes_category ON dishes (category);

-- 6. Votes and reactions
CREATE INDEX IF NOT EXISTS idx_dish_votes_user_id ON dish_votes (user_id);
CREATE INDEX IF NOT EXISTS idx_menu_votes_user_id ON menu_votes (user_id);
CREATE INDEX IF NOT EXISTS idx_vote_reactions_comment_id ON vote_reactions (comment_id);
CREATE INDEX IF NOT EXISTS idx_vote_reactions_user_id ON vote_reactions (user_id);

-- 7. Moderation and contact
CREATE INDEX IF NOT EXISTS idx_reports_status ON reports (status);
CREATE INDEX IF NOT EXISTS idx_reports_reporter_id ON reports (reporter_id);
CREATE INDEX IF NOT EXISTS idx_reports_reported_user_id ON reports (reported_user_id);
CREATE INDEX IF NOT EXISTS idx_reports_reported_comment_id ON reports (reported_comment_id);
CREATE INDEX IF NOT EXISTS idx_reports_menu_id ON reports (menu_id);
CREATE INDEX IF NOT EXISTS idx_contact_messages_status ON contact_messages (status);
CREATE INDEX IF NOT EXISTS idx_contact_messages_user_id ON contact_messages (user_id);

-- 8. Pricing periods and submissions
CREATE INDEX IF NOT EXISTS idx_pricing_periods_city_slug ON pricing_periods (city_slug);
CREATE INDEX IF NOT EXISTS idx_meal_category_prices_period_id ON meal_category_prices (pricing_period_id);
CREATE INDEX IF NOT EXISTS idx_menu_submissions_city_slug ON menu_submissions (city_slug);
CREATE INDEX IF NOT EXISTS idx_menu_submissions_status ON menu_submissions (status);

-- 9. Developer API and tags
CREATE INDEX IF NOT EXISTS idx_api_keys_project_id ON api_keys (project_id);
CREATE INDEX IF NOT EXISTS idx_api_keys_user_id ON api_keys (user_id);
CREATE INDEX IF NOT EXISTS idx_api_usage_logs_key_date ON api_usage_logs (api_key_id, date);
CREATE INDEX IF NOT EXISTS idx_dish_tags_tag_id ON dish_tags (tag_id);
