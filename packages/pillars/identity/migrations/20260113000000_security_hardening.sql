-- Security Hardening Migration
-- Adds: agent secrets, immutable audit logs with hash chain

-- =============================================================================
-- 1. Agent Secrets for Authentication
-- =============================================================================

ALTER TABLE agent_records 
ADD COLUMN IF NOT EXISTS secret_hash TEXT;

-- Index for faster login queries
CREATE INDEX IF NOT EXISTS idx_agent_records_id_secret 
    ON agent_records(id) WHERE secret_hash IS NOT NULL;

-- =============================================================================
-- 2. Immutable Audit Log with Hash Chain
-- =============================================================================

-- Add hash chain columns
ALTER TABLE audit_events
ADD COLUMN IF NOT EXISTS previous_hash TEXT,
ADD COLUMN IF NOT EXISTS event_hash TEXT,
ADD COLUMN IF NOT EXISTS signature TEXT;

-- Create index for hash chain integrity checks
CREATE INDEX IF NOT EXISTS idx_audit_events_event_hash 
    ON audit_events(event_hash);

-- Function to compute event hash
CREATE OR REPLACE FUNCTION compute_audit_hash() RETURNS TRIGGER AS $$
DECLARE
    prev_hash TEXT;
    content TEXT;
BEGIN
    -- Get previous event hash (or empty for first event)
    SELECT event_hash INTO prev_hash 
    FROM audit_events 
    ORDER BY created_at DESC, id DESC 
    LIMIT 1;
    
    IF prev_hash IS NULL THEN
        prev_hash := 'genesis';
    END IF;
    
    NEW.previous_hash := prev_hash;
    
    -- Compute hash of event content + previous hash
    content := COALESCE(NEW.event_type, '') || 
               COALESCE(NEW.actor_id, '') || 
               COALESCE(NEW.target_id, '') || 
               COALESCE(NEW.action, '') || 
               COALESCE(NEW.outcome, '') || 
               COALESCE(NEW.details::TEXT, '') ||
               NEW.created_at::TEXT ||
               prev_hash;
    
    NEW.event_hash := encode(sha256(content::BYTEA), 'hex');
    
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Trigger to compute hash on insert
DROP TRIGGER IF EXISTS audit_hash_trigger ON audit_events;
CREATE TRIGGER audit_hash_trigger
    BEFORE INSERT ON audit_events
    FOR EACH ROW
    EXECUTE FUNCTION compute_audit_hash();

-- =============================================================================
-- 3. Immutability Protection
-- =============================================================================

-- Prevent UPDATE on audit_events
CREATE OR REPLACE FUNCTION prevent_audit_update() RETURNS TRIGGER AS $$
BEGIN
    RAISE EXCEPTION 'Audit events are immutable and cannot be updated';
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS prevent_audit_update_trigger ON audit_events;
CREATE TRIGGER prevent_audit_update_trigger
    BEFORE UPDATE ON audit_events
    FOR EACH ROW
    EXECUTE FUNCTION prevent_audit_update();

-- Prevent DELETE on audit_events (except by superuser during maintenance)
CREATE OR REPLACE FUNCTION prevent_audit_delete() RETURNS TRIGGER AS $$
BEGIN
    IF current_setting('agentkern.allow_audit_delete', TRUE) = 'true' THEN
        RETURN OLD;
    END IF;
    RAISE EXCEPTION 'Audit events are immutable and cannot be deleted';
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS prevent_audit_delete_trigger ON audit_events;
CREATE TRIGGER prevent_audit_delete_trigger
    BEFORE DELETE ON audit_events
    FOR EACH ROW
    EXECUTE FUNCTION prevent_audit_delete();

-- =============================================================================
-- 4. Token Blacklist for Revocation
-- =============================================================================

CREATE TABLE IF NOT EXISTS token_blacklist (
    jti TEXT PRIMARY KEY,
    subject TEXT NOT NULL,
    revoked_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL,
    reason TEXT
);

-- Index for cleanup
CREATE INDEX IF NOT EXISTS idx_token_blacklist_expires_at 
    ON token_blacklist(expires_at);

-- Cleanup function for expired blacklist entries
CREATE OR REPLACE FUNCTION cleanup_token_blacklist() RETURNS void AS $$
BEGIN
    DELETE FROM token_blacklist WHERE expires_at < NOW();
END;
$$ LANGUAGE plpgsql;

-- =============================================================================
-- 5. Treasury Tables (Distributed State)
-- =============================================================================

CREATE TABLE IF NOT EXISTS agent_balances (
    agent_id TEXT PRIMARY KEY,
    balance BIGINT NOT NULL DEFAULT 0,
    held BIGINT NOT NULL DEFAULT 0,
    currency TEXT NOT NULL DEFAULT 'VMC',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS pending_transfers (
    transaction_id UUID PRIMARY KEY,
    from_agent TEXT NOT NULL REFERENCES agent_balances(agent_id),
    to_agent TEXT NOT NULL,
    amount BIGINT NOT NULL,
    reference TEXT,
    idempotency_key TEXT UNIQUE,
    status TEXT NOT NULL DEFAULT 'pending',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL DEFAULT (NOW() + INTERVAL '1 hour')
);

CREATE INDEX IF NOT EXISTS idx_pending_transfers_expires 
    ON pending_transfers(expires_at) WHERE status = 'pending';
CREATE INDEX IF NOT EXISTS idx_pending_transfers_idempotency 
    ON pending_transfers(idempotency_key);

CREATE TABLE IF NOT EXISTS completed_transfers (
    transaction_id UUID PRIMARY KEY,
    from_agent TEXT NOT NULL,
    to_agent TEXT NOT NULL,
    amount BIGINT NOT NULL,
    reference TEXT,
    idempotency_key TEXT,
    completed_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_completed_transfers_from 
    ON completed_transfers(from_agent, completed_at DESC);
CREATE INDEX IF NOT EXISTS idx_completed_transfers_to 
    ON completed_transfers(to_agent, completed_at DESC);
