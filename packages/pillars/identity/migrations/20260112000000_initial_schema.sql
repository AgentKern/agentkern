-- Enable UUID extension
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Agent Records Table
CREATE TABLE IF NOT EXISTS agent_records (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    namespace TEXT NOT NULL DEFAULT 'default',
    version TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'active',
    budget JSONB NOT NULL DEFAULT '{}',
    usage JSONB NOT NULL DEFAULT '{}',
    reputation JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_active_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    terminated_at TIMESTAMPTZ,
    termination_reason TEXT
);

CREATE INDEX IF NOT EXISTS idx_agent_records_namespace ON agent_records(namespace);

-- Verification Keys Table
CREATE TABLE IF NOT EXISTS verification_keys (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    principal_id TEXT NOT NULL,
    namespace TEXT NOT NULL DEFAULT 'default',
    credential_id TEXT NOT NULL,
    public_key TEXT NOT NULL, -- PEM or JWK
    algorithm TEXT NOT NULL DEFAULT 'ES256',
    format TEXT NOT NULL DEFAULT 'pem',
    active BOOLEAN NOT NULL DEFAULT TRUE,
    expires_at TIMESTAMPTZ,
    last_used_at TIMESTAMPTZ,
    usage_count INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_verification_keys_unique 
    ON verification_keys(principal_id, credential_id, namespace);
CREATE INDEX IF NOT EXISTS idx_verification_keys_principal_id ON verification_keys(principal_id);

-- WebAuthn Credentials Table
CREATE TABLE IF NOT EXISTS webauthn_credentials (
    id VARCHAR(512) PRIMARY KEY,
    principal_id VARCHAR(255) NOT NULL,
    credential_public_key BYTEA NOT NULL,
    webauthn_user_id VARCHAR(128) NOT NULL,
    counter INTEGER NOT NULL DEFAULT 0,
    credential_device_type VARCHAR(20) NOT NULL,
    credential_backed_up BOOLEAN NOT NULL DEFAULT FALSE,
    transports TEXT[],
    aaguid VARCHAR(36),
    device_name VARCHAR(255),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_used_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    revoked_at TIMESTAMPTZ,
    revocation_reason TEXT
);

CREATE INDEX IF NOT EXISTS idx_webauthn_credentials_principal_id ON webauthn_credentials(principal_id);

-- WebAuthn Challenges Table (ephemeral, short-lived)
CREATE TABLE IF NOT EXISTS webauthn_challenges (
    principal_id VARCHAR(255) PRIMARY KEY,
    challenge VARCHAR(128) NOT NULL,
    flow_type VARCHAR(20) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL,
    consumed BOOLEAN NOT NULL DEFAULT FALSE
);

CREATE INDEX IF NOT EXISTS idx_webauthn_challenges_expires_at ON webauthn_challenges(expires_at);

-- Audit Events Table
CREATE TABLE IF NOT EXISTS audit_events (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    event_type VARCHAR(100) NOT NULL,
    actor_id VARCHAR(255),
    actor_type VARCHAR(50),
    target_id VARCHAR(255),
    target_type VARCHAR(50),
    action VARCHAR(100) NOT NULL,
    outcome VARCHAR(20) NOT NULL,
    details JSONB,
    ip_address INET,
    user_agent TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_audit_events_created_at ON audit_events(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_audit_events_actor_id ON audit_events(actor_id);
CREATE INDEX IF NOT EXISTS idx_audit_events_event_type ON audit_events(event_type);

-- System Config Table (for kill switch, feature flags, etc.)
CREATE TABLE IF NOT EXISTS system_config (
    key VARCHAR(255) PRIMARY KEY,
    value TEXT NOT NULL,
    description TEXT,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
