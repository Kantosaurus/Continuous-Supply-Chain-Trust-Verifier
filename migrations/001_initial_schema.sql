-- Supply Chain Trust Verifier - Initial Schema
-- This migration creates the core tables for the multi-tenant platform.

-- Enable required extensions
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pg_trgm";  -- For fuzzy text search

-- ============================================================================
-- TENANTS
-- ============================================================================

CREATE TABLE tenants (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(255) NOT NULL,
    slug VARCHAR(100) NOT NULL UNIQUE,
    plan JSONB NOT NULL DEFAULT '{"type": "free", "project_limit": 5}',
    settings JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_tenants_slug ON tenants(slug);

-- ============================================================================
-- USERS
-- ============================================================================

CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    email VARCHAR(255) NOT NULL,
    name VARCHAR(255),
    role VARCHAR(50) NOT NULL DEFAULT 'member',
    api_key_hash VARCHAR(255),
    last_login_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(tenant_id, email)
);

CREATE INDEX idx_users_tenant ON users(tenant_id);
CREATE INDEX idx_users_api_key ON users(api_key_hash) WHERE api_key_hash IS NOT NULL;

-- ============================================================================
-- POLICIES
-- ============================================================================

CREATE TABLE policies (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    rules JSONB NOT NULL DEFAULT '[]',
    severity_overrides JSONB DEFAULT '[]',
    is_default BOOLEAN DEFAULT false,
    enabled BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(tenant_id, name)
);

CREATE INDEX idx_policies_tenant ON policies(tenant_id);
CREATE INDEX idx_policies_default ON policies(tenant_id, is_default) WHERE is_default = true;

-- ============================================================================
-- PROJECTS
-- ============================================================================

CREATE TABLE projects (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    repository_url VARCHAR(500),
    default_branch VARCHAR(100) DEFAULT 'main',
    ecosystems VARCHAR(50)[] NOT NULL DEFAULT '{}',
    scan_schedule JSONB NOT NULL DEFAULT '{"type": "daily", "hour": 2}',
    policy_id UUID REFERENCES policies(id) ON DELETE SET NULL,
    last_scan_at TIMESTAMPTZ,
    status VARCHAR(50) NOT NULL DEFAULT 'unknown',
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(tenant_id, name)
);

CREATE INDEX idx_projects_tenant ON projects(tenant_id);
CREATE INDEX idx_projects_status ON projects(status);
CREATE INDEX idx_projects_last_scan ON projects(last_scan_at);

-- ============================================================================
-- DEPENDENCIES
-- ============================================================================

CREATE TABLE dependencies (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    project_id UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    package_name VARCHAR(500) NOT NULL,
    ecosystem VARCHAR(50) NOT NULL,
    version_constraint VARCHAR(100),
    resolved_version VARCHAR(100) NOT NULL,
    is_direct BOOLEAN NOT NULL DEFAULT true,
    is_dev_dependency BOOLEAN NOT NULL DEFAULT false,
    depth INTEGER NOT NULL DEFAULT 0,
    parent_id UUID REFERENCES dependencies(id) ON DELETE SET NULL,
    hash_sha256 VARCHAR(64),
    hash_sha512 VARCHAR(128),
    signature_status VARCHAR(50) DEFAULT 'unknown',
    provenance_status VARCHAR(50) DEFAULT 'unknown',
    provenance_details JSONB,
    first_seen_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_verified_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(project_id, package_name, ecosystem, resolved_version)
);

CREATE INDEX idx_dependencies_project ON dependencies(project_id);
CREATE INDEX idx_dependencies_tenant ON dependencies(tenant_id);
CREATE INDEX idx_dependencies_package ON dependencies(package_name, ecosystem);
CREATE INDEX idx_dependencies_ecosystem ON dependencies(ecosystem);

-- ============================================================================
-- PACKAGES (shared cache across tenants)
-- ============================================================================

CREATE TABLE packages (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    ecosystem VARCHAR(50) NOT NULL,
    name VARCHAR(500) NOT NULL,
    normalized_name VARCHAR(500) NOT NULL,
    description TEXT,
    homepage VARCHAR(500),
    repository VARCHAR(500),
    popularity_rank INTEGER,
    is_popular BOOLEAN DEFAULT false,
    maintainers JSONB DEFAULT '[]',
    first_published TIMESTAMPTZ,
    last_updated TIMESTAMPTZ,
    metadata JSONB DEFAULT '{}',
    cached_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(ecosystem, name)
);

CREATE INDEX idx_packages_ecosystem ON packages(ecosystem);
CREATE INDEX idx_packages_normalized ON packages(normalized_name);
CREATE INDEX idx_packages_popular ON packages(ecosystem, is_popular) WHERE is_popular = true;
CREATE INDEX idx_packages_name_trgm ON packages USING gin(name gin_trgm_ops);

-- ============================================================================
-- PACKAGE VERSIONS
-- ============================================================================

CREATE TABLE package_versions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    package_id UUID NOT NULL REFERENCES packages(id) ON DELETE CASCADE,
    version VARCHAR(100) NOT NULL,
    published_at TIMESTAMPTZ,
    yanked BOOLEAN DEFAULT false,
    deprecated BOOLEAN DEFAULT false,
    deprecation_message TEXT,
    checksums JSONB DEFAULT '{}',
    download_url VARCHAR(1000),
    size_bytes BIGINT,
    attestations JSONB DEFAULT '[]',
    dependencies JSONB DEFAULT '[]',
    cached_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(package_id, version)
);

CREATE INDEX idx_package_versions_package ON package_versions(package_id);
CREATE INDEX idx_package_versions_published ON package_versions(published_at DESC);

-- ============================================================================
-- ALERTS
-- ============================================================================

CREATE TABLE alerts (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    project_id UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    dependency_id UUID REFERENCES dependencies(id) ON DELETE SET NULL,
    alert_type VARCHAR(100) NOT NULL,
    alert_details JSONB NOT NULL DEFAULT '{}',
    severity VARCHAR(50) NOT NULL,
    title VARCHAR(500) NOT NULL,
    description TEXT,
    status VARCHAR(50) NOT NULL DEFAULT 'open',
    remediation JSONB,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    acknowledged_at TIMESTAMPTZ,
    acknowledged_by UUID REFERENCES users(id) ON DELETE SET NULL,
    resolved_at TIMESTAMPTZ,
    resolved_by UUID REFERENCES users(id) ON DELETE SET NULL
);

CREATE INDEX idx_alerts_tenant ON alerts(tenant_id);
CREATE INDEX idx_alerts_project ON alerts(project_id);
CREATE INDEX idx_alerts_status ON alerts(status);
CREATE INDEX idx_alerts_severity ON alerts(severity);
CREATE INDEX idx_alerts_created ON alerts(created_at DESC);
CREATE INDEX idx_alerts_type ON alerts(alert_type);

-- ============================================================================
-- SBOMS
-- ============================================================================

CREATE TABLE sboms (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    project_id UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    format VARCHAR(50) NOT NULL,  -- 'cyclonedx' or 'spdx'
    format_version VARCHAR(20) NOT NULL,
    content JSONB NOT NULL,
    generated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    scan_id UUID
);

CREATE INDEX idx_sboms_project ON sboms(project_id);
CREATE INDEX idx_sboms_generated ON sboms(generated_at DESC);

-- ============================================================================
-- BACKGROUND JOBS
-- ============================================================================

CREATE TABLE jobs (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID REFERENCES tenants(id) ON DELETE CASCADE,
    job_type VARCHAR(100) NOT NULL,
    status VARCHAR(50) NOT NULL DEFAULT 'pending',
    priority INTEGER DEFAULT 0,
    payload JSONB NOT NULL DEFAULT '{}',
    result JSONB,
    error_message TEXT,
    attempts INTEGER DEFAULT 0,
    max_attempts INTEGER DEFAULT 3,
    scheduled_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_jobs_status ON jobs(status) WHERE status IN ('pending', 'running');
CREATE INDEX idx_jobs_scheduled ON jobs(scheduled_at) WHERE status = 'pending';
CREATE INDEX idx_jobs_tenant ON jobs(tenant_id);
CREATE INDEX idx_jobs_type ON jobs(job_type);

-- ============================================================================
-- AUDIT LOGS
-- ============================================================================

CREATE TABLE audit_logs (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    action VARCHAR(100) NOT NULL,
    resource_type VARCHAR(100) NOT NULL,
    resource_id UUID,
    details JSONB DEFAULT '{}',
    ip_address INET,
    user_agent TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_audit_tenant ON audit_logs(tenant_id);
CREATE INDEX idx_audit_created ON audit_logs(created_at DESC);
CREATE INDEX idx_audit_resource ON audit_logs(resource_type, resource_id);

-- ============================================================================
-- HELPER FUNCTIONS
-- ============================================================================

-- Function to update the updated_at timestamp
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Apply update triggers
CREATE TRIGGER update_tenants_updated_at BEFORE UPDATE ON tenants
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_users_updated_at BEFORE UPDATE ON users
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_policies_updated_at BEFORE UPDATE ON policies
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_projects_updated_at BEFORE UPDATE ON projects
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
