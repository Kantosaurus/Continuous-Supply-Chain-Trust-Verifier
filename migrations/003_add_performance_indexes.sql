-- Supply Chain Trust Verifier - Performance Indexes
-- This migration adds additional indexes for query optimization.

-- ============================================================================
-- USERS - Additional indexes for authentication
-- ============================================================================

-- Index for email lookups (useful for login)
CREATE INDEX IF NOT EXISTS idx_users_email ON users(email);

-- ============================================================================
-- JOBS - Indexes for efficient job claiming and scheduling
-- ============================================================================

-- Composite index for job claiming (status + scheduled_at + priority)
CREATE INDEX IF NOT EXISTS idx_jobs_claimable ON jobs(scheduled_at, priority DESC)
    WHERE status IN ('pending', 'scheduled');

-- Index for finding stale running jobs
CREATE INDEX IF NOT EXISTS idx_jobs_running_started ON jobs(started_at)
    WHERE status = 'running';

-- Index for cleanup queries on completed jobs
CREATE INDEX IF NOT EXISTS idx_jobs_completed_at ON jobs(completed_at)
    WHERE status IN ('completed', 'failed', 'cancelled');

-- ============================================================================
-- SBOMS - Indexes for format-based queries
-- ============================================================================

-- Index for finding SBOMs by format
CREATE INDEX IF NOT EXISTS idx_sboms_format ON sboms(project_id, format, generated_at DESC);

-- Index for tenant-based SBOM queries
CREATE INDEX IF NOT EXISTS idx_sboms_tenant ON sboms(tenant_id);

-- ============================================================================
-- AUDIT_LOGS - Indexes for common query patterns
-- ============================================================================

-- Index for user activity queries
CREATE INDEX IF NOT EXISTS idx_audit_user ON audit_logs(user_id, created_at DESC)
    WHERE user_id IS NOT NULL;

-- Index for action type filtering
CREATE INDEX IF NOT EXISTS idx_audit_action ON audit_logs(action);

-- ============================================================================
-- DEPENDENCIES - Additional indexes for integrity verification
-- ============================================================================

-- Index for finding dependencies by signature status
CREATE INDEX IF NOT EXISTS idx_dependencies_signature ON dependencies(signature_status)
    WHERE signature_status != 'unknown';

-- Index for finding dependencies by provenance status
CREATE INDEX IF NOT EXISTS idx_dependencies_provenance ON dependencies(provenance_status)
    WHERE provenance_status != 'unknown';

-- Index for finding dependencies that need reverification
CREATE INDEX IF NOT EXISTS idx_dependencies_last_verified ON dependencies(last_verified_at);

-- ============================================================================
-- ALERTS - Index for dependency-based alert lookup
-- ============================================================================

-- Index for finding alerts by dependency
CREATE INDEX IF NOT EXISTS idx_alerts_dependency ON alerts(dependency_id)
    WHERE dependency_id IS NOT NULL;

-- ============================================================================
-- PACKAGES - Additional search optimization
-- ============================================================================

-- Index for normalized name searches (case-insensitive lookups)
CREATE INDEX IF NOT EXISTS idx_packages_normalized_trgm ON packages
    USING gin(normalized_name gin_trgm_ops);

-- Note: the `dependencies` table has no `updated_at` column (it uses
-- `last_verified_at` to track changes), so no updated_at trigger is needed.

-- ============================================================================
-- COMMENTS - Add table and column documentation
-- ============================================================================

COMMENT ON TABLE tenants IS 'Multi-tenant organizations using the platform';
COMMENT ON TABLE users IS 'User accounts within tenant organizations';
COMMENT ON TABLE projects IS 'Software projects being monitored for supply chain threats';
COMMENT ON TABLE dependencies IS 'Package dependencies tracked for each project';
COMMENT ON TABLE packages IS 'Cached package metadata from registries (shared across tenants)';
COMMENT ON TABLE package_versions IS 'Version-specific package information with integrity data';
COMMENT ON TABLE policies IS 'Security policies defining acceptable dependency criteria';
COMMENT ON TABLE alerts IS 'Security alerts generated from dependency scanning';
COMMENT ON TABLE sboms IS 'Software Bill of Materials documents for projects';
COMMENT ON TABLE jobs IS 'Background job queue for async processing';
COMMENT ON TABLE audit_logs IS 'Security audit trail for compliance and forensics';
