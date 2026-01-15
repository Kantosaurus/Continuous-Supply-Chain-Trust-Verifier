-- Supply Chain Trust Verifier - Row Level Security
-- This migration enables multi-tenant isolation using PostgreSQL RLS.

-- ============================================================================
-- Enable Row Level Security on tenant-scoped tables
-- ============================================================================

ALTER TABLE users ENABLE ROW LEVEL SECURITY;
ALTER TABLE projects ENABLE ROW LEVEL SECURITY;
ALTER TABLE dependencies ENABLE ROW LEVEL SECURITY;
ALTER TABLE policies ENABLE ROW LEVEL SECURITY;
ALTER TABLE alerts ENABLE ROW LEVEL SECURITY;
ALTER TABLE sboms ENABLE ROW LEVEL SECURITY;
ALTER TABLE jobs ENABLE ROW LEVEL SECURITY;
ALTER TABLE audit_logs ENABLE ROW LEVEL SECURITY;

-- ============================================================================
-- Create RLS policies for tenant isolation
-- ============================================================================

-- Users can only see users in their tenant
CREATE POLICY tenant_isolation_users ON users
    USING (tenant_id = current_setting('app.current_tenant_id', true)::UUID);

-- Projects can only be accessed within tenant
CREATE POLICY tenant_isolation_projects ON projects
    USING (tenant_id = current_setting('app.current_tenant_id', true)::UUID);

-- Dependencies are isolated by tenant
CREATE POLICY tenant_isolation_dependencies ON dependencies
    USING (tenant_id = current_setting('app.current_tenant_id', true)::UUID);

-- Policies are tenant-specific
CREATE POLICY tenant_isolation_policies ON policies
    USING (tenant_id = current_setting('app.current_tenant_id', true)::UUID);

-- Alerts are isolated by tenant
CREATE POLICY tenant_isolation_alerts ON alerts
    USING (tenant_id = current_setting('app.current_tenant_id', true)::UUID);

-- SBOMs are tenant-specific
CREATE POLICY tenant_isolation_sboms ON sboms
    USING (tenant_id = current_setting('app.current_tenant_id', true)::UUID);

-- Jobs can be tenant-specific or global (NULL tenant_id)
CREATE POLICY tenant_isolation_jobs ON jobs
    USING (
        tenant_id IS NULL
        OR tenant_id = current_setting('app.current_tenant_id', true)::UUID
    );

-- Audit logs are tenant-specific
CREATE POLICY tenant_isolation_audit ON audit_logs
    USING (tenant_id = current_setting('app.current_tenant_id', true)::UUID);

-- ============================================================================
-- Note: Packages and package_versions are NOT tenant-isolated
-- They are shared cache tables used by all tenants
-- ============================================================================

-- ============================================================================
-- Create a helper function to set the current tenant
-- ============================================================================

CREATE OR REPLACE FUNCTION set_current_tenant(tenant_id UUID)
RETURNS void AS $$
BEGIN
    PERFORM set_config('app.current_tenant_id', tenant_id::TEXT, true);
END;
$$ LANGUAGE plpgsql;

-- ============================================================================
-- Create a role for the application with RLS enabled
-- ============================================================================

-- Note: In production, create a specific role:
-- CREATE ROLE app_user;
-- GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA public TO app_user;
-- ALTER DEFAULT PRIVILEGES IN SCHEMA public GRANT SELECT, INSERT, UPDATE, DELETE ON TABLES TO app_user;
