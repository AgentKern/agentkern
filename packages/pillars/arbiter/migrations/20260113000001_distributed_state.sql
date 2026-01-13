-- Arbiter Distributed State Schema
-- Per ARCHITECTURE.md: Replaces in-memory HashMap with Postgres persistence

-- Distributed Locks Table
CREATE TABLE IF NOT EXISTS arbiter_locks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    resource VARCHAR(255) NOT NULL UNIQUE,
    locked_by VARCHAR(255) NOT NULL,
    priority INT NOT NULL DEFAULT 0,
    lock_type VARCHAR(50) NOT NULL DEFAULT 'write',
    acquired_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Index for lock lookups and expiry checks
CREATE INDEX idx_arbiter_locks_resource ON arbiter_locks(resource);
CREATE INDEX idx_arbiter_locks_expires ON arbiter_locks(expires_at);

-- Distributed Queue Table
CREATE TABLE IF NOT EXISTS arbiter_queue (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    agent_id VARCHAR(255) NOT NULL,
    resource VARCHAR(255) NOT NULL,
    priority INT NOT NULL DEFAULT 0,
    operation VARCHAR(50) NOT NULL DEFAULT 'write',
    expected_duration_ms BIGINT NOT NULL DEFAULT 5000,
    intent TEXT,
    enqueued_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(agent_id, resource) -- One request per agent per resource
);

-- Index for queue priority ordering
CREATE INDEX idx_arbiter_queue_resource_priority ON arbiter_queue(resource, priority DESC, enqueued_at ASC);
