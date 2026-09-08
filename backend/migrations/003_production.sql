-- Production-ready миграции для Чистовик

-- Idempotency keys (защита от дублей)
CREATE TABLE IF NOT EXISTS idempotency_keys (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id),
    key VARCHAR(255) NOT NULL,
    response_body JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(user_id, key)
);

CREATE INDEX IF NOT EXISTS idx_idempotency_keys_user ON idempotency_keys(user_id);
CREATE INDEX IF NOT EXISTS idx_idempotency_keys_created ON idempotency_keys(created_at);

-- Upload sessions (для chunked upload)
CREATE TABLE IF NOT EXISTS upload_sessions (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id),
    filename VARCHAR(255) NOT NULL,
    total_size BIGINT NOT NULL,
    content_type VARCHAR(100) NOT NULL,
    temp_path TEXT NOT NULL,
    file_path TEXT,
    file_hash VARCHAR(64),
    status VARCHAR(20) NOT NULL DEFAULT 'pending',
    total_chunks INTEGER NOT NULL DEFAULT 0,
    uploaded_chunks INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_upload_sessions_user ON upload_sessions(user_id);
CREATE INDEX IF NOT EXISTS idx_upload_sessions_status ON upload_sessions(status);

-- Spam checks (антиспам)
CREATE TABLE IF NOT EXISTS spam_checks (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id),
    content_type VARCHAR(50) NOT NULL,
    content_hash VARCHAR(64) NOT NULL,
    score DOUBLE PRECISION NOT NULL,
    is_spam BOOLEAN NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_spam_checks_user ON spam_checks(user_id);
CREATE INDEX IF NOT EXISTS idx_spam_checks_created ON spam_checks(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_spam_checks_hash ON spam_checks(content_hash);

-- Security alerts (аномалии)
CREATE TABLE IF NOT EXISTS security_alerts (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id),
    alert_type VARCHAR(100) NOT NULL,
    risk_score DOUBLE PRECISION NOT NULL,
    alerts JSONB NOT NULL DEFAULT '[]',
    ip_address VARCHAR(45),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_security_alerts_user ON security_alerts(user_id);
CREATE INDEX IF NOT EXISTS idx_security_alerts_created ON security_alerts(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_security_alerts_type ON security_alerts(alert_type);

-- Quotas (квоты пользователей)
CREATE TABLE IF NOT EXISTS user_quotas (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id) UNIQUE,
    max_storage_bytes BIGINT NOT NULL DEFAULT 10737418240, -- 10 GB
    max_contents INTEGER NOT NULL DEFAULT 1000,
    max_file_size BIGINT NOT NULL DEFAULT 2147483648, -- 2 GB
    max_daily_uploads INTEGER NOT NULL DEFAULT 50,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_user_quotas_user ON user_quotas(user_id);

-- Backup history (история бэкапов)
CREATE TABLE IF NOT EXISTS backup_history (
    id UUID PRIMARY KEY,
    filename VARCHAR(255) NOT NULL,
    file_path TEXT NOT NULL,
    size_bytes BIGINT NOT NULL,
    hash VARCHAR(64) NOT NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'completed',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_backup_history_created ON backup_history(created_at DESC);
