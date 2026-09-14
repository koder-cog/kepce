-- 0025_disable_dormant_rls.sql
-- Disable dormant Row Level Security (RLS) on tables where policies
-- relied on PostgREST/Supabase session settings (request.jwt.claim.sub).
-- Kepçe backend (Rust/Axum) handles all authorization and access control
-- at the application layer.

ALTER TABLE users DISABLE ROW LEVEL SECURITY;
ALTER TABLE comments DISABLE ROW LEVEL SECURITY;
ALTER TABLE user_favorites DISABLE ROW LEVEL SECURITY;
ALTER TABLE user_pinned_dishes DISABLE ROW LEVEL SECURITY;
ALTER TABLE api_keys DISABLE ROW LEVEL SECURITY;
ALTER TABLE projects DISABLE ROW LEVEL SECURITY;
