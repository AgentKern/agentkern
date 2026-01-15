-- Migration: Intent Paths
-- Purpose: Persist agent goal progression and execution history
-- Date: 2026-01-15

CREATE TABLE IF NOT EXISTS intent_paths (
    id UUID PRIMARY KEY,
    agent_id TEXT NOT NULL,
    original_intent TEXT NOT NULL,
    intent_embedding REAL[], -- Optional pgvector: plain array for now (f32)
    current_step INTEGER NOT NULL DEFAULT 0,
    expected_steps INTEGER NOT NULL DEFAULT 0,
    history JSONB NOT NULL DEFAULT '[]'::jsonb, -- Store list of IntentStep as JSON
    drift_detected BOOLEAN NOT NULL DEFAULT FALSE,
    drift_score INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Index for faster agent lookups
CREATE INDEX IF NOT EXISTS idx_intent_paths_agent_id ON intent_paths(agent_id);

-- Index for finding drifted agents (governance queries)
CREATE INDEX IF NOT EXISTS idx_intent_paths_drift ON intent_paths(drift_score) WHERE drift_score > 0;
