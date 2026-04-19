-- ============================================================================
-- Migration 005 — API keys
-- ============================================================================
-- Stores server-issued API keys used for programmatic access. The raw key
-- value is never persisted; only its sha256 digest is stored. Lookup is by
-- digest, and comparison must be constant-time on the application side.
-- ============================================================================

CREATE TABLE IF NOT EXISTS api_keys (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    key_hash TEXT NOT NULL UNIQUE,
    scopes TEXT[] NOT NULL DEFAULT ARRAY[]::TEXT[],
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_used_at TIMESTAMPTZ,
    expires_at TIMESTAMPTZ,
    revoked_at TIMESTAMPTZ
);

-- Only active (non-revoked) keys participate in auth lookups.
CREATE INDEX IF NOT EXISTS idx_api_keys_key_hash_active
    ON api_keys(key_hash)
    WHERE revoked_at IS NULL;

CREATE INDEX IF NOT EXISTS idx_api_keys_tenant_id
    ON api_keys(tenant_id);
