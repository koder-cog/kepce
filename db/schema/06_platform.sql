-- ==============================================================================
-- KEPÇE VERİTABANI DOKÜMANTASYONU: 06_PLATFORM (Geliştirici API ve Sistem Olayları)
-- ==============================================================================
-- Bu dosya açık veri platformu geliştirici anahtarlarını, kullanım kotalarını,
-- projeleri ve sistem durum / kesinti (incidents) kayıtlarını tanımlar.
-- ==============================================================================

-- 1. Geliştirici Projeleri (Developer Projects)
CREATE TABLE projects (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- 2. API Anahtarları (API Keys - SHA-256 Hashlenmiş)
CREATE TABLE api_keys (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    key_hash VARCHAR(255) NOT NULL UNIQUE,
    key_prefix VARCHAR(10) NOT NULL,
    name VARCHAR(255) NOT NULL,
    tier VARCHAR(50) NOT NULL DEFAULT 'free',
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- 3. Günlük API Kota ve Kullanım Kayıtları (API Usage Logs)
CREATE TABLE api_usage_logs (
    id SERIAL PRIMARY KEY,
    api_key_id UUID NOT NULL REFERENCES api_keys(id) ON DELETE CASCADE,
    date DATE NOT NULL DEFAULT CURRENT_DATE,
    requests INTEGER NOT NULL DEFAULT 0,
    errors INTEGER NOT NULL DEFAULT 0,
    UNIQUE(api_key_id, date)
);

-- 4. Sistem Durum ve Kesinti Olayları (System Incidents)
CREATE TABLE system_incidents (
    id SERIAL PRIMARY KEY,
    component VARCHAR(100) NOT NULL,
    title VARCHAR(255) NOT NULL,
    message TEXT NOT NULL,
    status VARCHAR(50) NOT NULL DEFAULT 'investigating',
    impact VARCHAR(50) NOT NULL DEFAULT 'yavas',
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    resolved_at TIMESTAMPTZ
);
