-- ============================================================================
-- Migration 004 — Foreign-key indexes
-- ============================================================================
-- PostgreSQL does not automatically index FK columns on the referencing side.
-- Without these indexes, joins and cascade deletes on the parent tables fall
-- back to sequential scans, which becomes expensive as the tables grow.
-- ============================================================================

CREATE INDEX IF NOT EXISTS idx_alerts_acknowledged_by
    ON alerts(acknowledged_by)
    WHERE acknowledged_by IS NOT NULL;

CREATE INDEX IF NOT EXISTS idx_alerts_resolved_by
    ON alerts(resolved_by)
    WHERE resolved_by IS NOT NULL;

CREATE INDEX IF NOT EXISTS idx_projects_policy_id
    ON projects(policy_id)
    WHERE policy_id IS NOT NULL;

CREATE INDEX IF NOT EXISTS idx_dependencies_parent_id
    ON dependencies(parent_id)
    WHERE parent_id IS NOT NULL;
