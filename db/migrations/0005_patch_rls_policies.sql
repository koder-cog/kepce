-- Row Level Security (RLS) Policies
-- This migration enables RLS on sensitive tables to ensure defense-in-depth data isolation.

-- 1. Enable RLS on tables
ALTER TABLE users ENABLE ROW LEVEL SECURITY;
ALTER TABLE comments ENABLE ROW LEVEL SECURITY;
ALTER TABLE user_favorites ENABLE ROW LEVEL SECURITY;
ALTER TABLE user_pinned_dishes ENABLE ROW LEVEL SECURITY;
ALTER TABLE api_keys ENABLE ROW LEVEL SECURITY;
ALTER TABLE projects ENABLE ROW LEVEL SECURITY;

-- 2. Define Policies (Drop if exists first to allow re-run safety)
DO $$
BEGIN
    DROP POLICY IF EXISTS "Public profiles are viewable by everyone" ON users;
    CREATE POLICY "Public profiles are viewable by everyone" ON users FOR SELECT USING (true);

    DROP POLICY IF EXISTS "Users can update own profile" ON users;
    CREATE POLICY "Users can update own profile" ON users FOR UPDATE USING (id = current_setting('request.jwt.claim.sub', true)::uuid);

    DROP POLICY IF EXISTS "Comments are viewable by everyone" ON comments;
    CREATE POLICY "Comments are viewable by everyone" ON comments FOR SELECT USING (true);

    DROP POLICY IF EXISTS "Users can insert own comments" ON comments;
    CREATE POLICY "Users can insert own comments" ON comments FOR INSERT WITH CHECK (user_id = current_setting('request.jwt.claim.sub', true)::uuid);

    DROP POLICY IF EXISTS "Users can update own comments" ON comments;
    CREATE POLICY "Users can update own comments" ON comments FOR UPDATE USING (user_id = current_setting('request.jwt.claim.sub', true)::uuid);

    DROP POLICY IF EXISTS "Users can delete own comments" ON comments;
    CREATE POLICY "Users can delete own comments" ON comments FOR DELETE USING (user_id = current_setting('request.jwt.claim.sub', true)::uuid);

    DROP POLICY IF EXISTS "Users can view own favorites" ON user_favorites;
    DROP POLICY IF EXISTS "Favorites are viewable by everyone" ON user_favorites;
    CREATE POLICY "Favorites are viewable by everyone" ON user_favorites FOR SELECT USING (true);

    DROP POLICY IF EXISTS "Users can manage own favorites" ON user_favorites;
    CREATE POLICY "Users can manage own favorites" ON user_favorites FOR ALL USING (user_id = current_setting('request.jwt.claim.sub', true)::uuid);

    DROP POLICY IF EXISTS "Users can view own pinned dishes" ON user_pinned_dishes;
    DROP POLICY IF EXISTS "Pinned dishes are viewable by everyone" ON user_pinned_dishes;
    CREATE POLICY "Pinned dishes are viewable by everyone" ON user_pinned_dishes FOR SELECT USING (true);

    DROP POLICY IF EXISTS "Users can manage own pinned dishes" ON user_pinned_dishes;
    CREATE POLICY "Users can manage own pinned dishes" ON user_pinned_dishes FOR ALL USING (user_id = current_setting('request.jwt.claim.sub', true)::uuid);

    DROP POLICY IF EXISTS "Users can view own projects" ON projects;
    CREATE POLICY "Users can view own projects" ON projects FOR SELECT USING (user_id = current_setting('request.jwt.claim.sub', true)::uuid);

    DROP POLICY IF EXISTS "Users can manage own projects" ON projects;
    CREATE POLICY "Users can manage own projects" ON projects FOR ALL USING (user_id = current_setting('request.jwt.claim.sub', true)::uuid);

    DROP POLICY IF EXISTS "Users can view own api keys" ON api_keys;
    CREATE POLICY "Users can view own api keys" ON api_keys FOR SELECT USING (user_id = current_setting('request.jwt.claim.sub', true)::uuid);

    DROP POLICY IF EXISTS "Users can manage own api keys" ON api_keys;
    CREATE POLICY "Users can manage own api keys" ON api_keys FOR ALL USING (user_id = current_setting('request.jwt.claim.sub', true)::uuid);
END$$;
