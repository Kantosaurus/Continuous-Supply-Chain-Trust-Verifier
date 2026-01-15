# Database Schema

**Version:** 0.1.0

Complete documentation of the SCTV PostgreSQL database schema.

---

## Table of Contents

- [Overview](#overview)
- [Schema Diagram](#schema-diagram)
- [Tables](#tables)
- [Indexes](#indexes)
- [Constraints and Relationships](#constraints-and-relationships)
- [Multi-Tenancy](#multi-tenancy)
- [Migrations](#migrations)
- [Performance Considerations](#performance-considerations)

---

## Overview

SCTV uses PostgreSQL 14+ with the following extensions:

- **uuid-ossp** - UUID generation
- **pg_trgm** - Fuzzy text search for typosquatting detection

### Design Principles

1. **Multi-tenant isolation** - All user data scoped by `tenant_id`
2. **Normalized schema** - Reduce redundancy, ensure consistency
3. **Audit trail** - Track all changes with timestamps
4. **Soft deletes** - Preserve historical data where appropriate
5. **JSONB for flexibility** - Store semi-structured data efficiently

---

## Schema Diagram

```
┌─────────────┐
│  tenants    │
│─────────────│
│ id (PK)     │◄──┐
│ name        │   │
│ slug (UK)   │   │
│ plan        │   │
│ settings    │   │
└─────────────┘   │
                  │ FK: tenant_id
                  │
    ┌─────────────┴──────────┬─────────────────────┬──────────────┐
    │                        │                     │              │
┌───┴─────────┐      ┌───────┴──────┐      ┌──────┴──────┐  ┌───┴──────┐
│   users     │      │   policies   │      │  projects   │  │   jobs   │
│─────────────│      │──────────────│      │─────────────│  │──────────│
│ id (PK)     │      │ id (PK)      │◄─┐   │ id (PK)     │  │ id (PK)  │
│ tenant_id   │      │ tenant_id    │  │   │ tenant_id   │  │ tenant_id│
│ email       │      │ name         │  │   │ name        │  │ job_type │
│ role        │      │ rules        │  │   │ repository  │  │ status   │
└─────────────┘      │ enabled      │  │   │ ecosystems  │  │ payload  │
                     └──────────────┘  │   │ policy_id ──┘  └──────────┘
                                       │   │ status      │
                                       │   └──────┬──────┘
                                       │          │
                                       │          │ FK: project_id
                                       │          │
                     ┌─────────────────┴──────────┴─────┬──────────────┐
                     │                                   │              │
              ┌──────┴──────────┐              ┌────────┴───────┐  ┌───┴──────┐
              │  dependencies   │              │    alerts      │  │  sboms   │
              │─────────────────│              │────────────────│  │──────────│
              │ id (PK)         │              │ id (PK)        │  │ id (PK)  │
              │ project_id      │              │ tenant_id      │  │ project  │
              │ tenant_id       │              │ project_id     │  │ format   │
              │ package_name    │              │ dependency_id  │  │ content  │
              │ ecosystem       │              │ alert_type     │  └──────────┘
              │ version         │              │ severity       │
              │ hash_sha256     │              │ status         │
              │ provenance      │              └────────────────┘
              └─────────────────┘

┌──────────────────┐           ┌────────────────────┐
│    packages      │           │ package_versions   │
│  (global cache)  │           │  (global cache)    │
│──────────────────│           │────────────────────│
│ id (PK)          │◄──────────│ id (PK)            │
│ ecosystem        │ FK        │ package_id         │
│ name (UK)        │           │ version (UK)       │
│ normalized_name  │           │ published_at       │
│ is_popular       │           │ checksums          │
│ maintainers      │           │ attestations       │
└──────────────────┘           └────────────────────┘

┌──────────────────┐
│   audit_logs     │
│──────────────────│
│ id (PK)          │
│ tenant_id        │
│ user_id          │
│ action           │
│ resource_type    │
│ resource_id      │
│ details          │
│ ip_address       │
└──────────────────┘
```

---

## Tables

### tenants

Multi-tenant organizations. All user data is scoped to a tenant.

```sql
CREATE TABLE tenants (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(255) NOT NULL,
    slug VARCHAR(100) NOT NULL UNIQUE,
    plan JSONB NOT NULL DEFAULT '{"type": "free", "project_limit": 5}',
    settings JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

**Fields:**
- `id` - Unique tenant identifier
- `name` - Organization display name
- `slug` - URL-safe identifier (e.g., "acme-corp")
- `plan` - Subscription plan details (type, limits)
- `settings` - Tenant-wide configuration (notifications, etc.)
- `created_at` - Tenant creation timestamp
- `updated_at` - Last modification timestamp

**Plan JSONB Structure:**
```json
{
  "type": "enterprise",
  "project_limit": 100,
  "user_limit": 50,
  "scan_frequency": "hourly",
  "retention_days": 365
}
```

**Settings JSONB Structure:**
```json
{
  "notification_channels": [
    {
      "type": "slack",
      "enabled": true,
      "webhook_url": "https://hooks.slack.com/...",
      "min_severity": "high"
    }
  ],
  "default_policy_id": "uuid",
  "allow_public_sboms": false
}
```

---

### users

User accounts associated with tenants.

```sql
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
```

**Fields:**
- `id` - Unique user identifier
- `tenant_id` - Associated tenant
- `email` - User email (unique per tenant)
- `name` - Display name
- `role` - User role: `admin`, `member`, `viewer`, `api_only`
- `api_key_hash` - Hashed API key for programmatic access
- `last_login_at` - Last login timestamp
- `created_at` - Account creation timestamp
- `updated_at` - Last modification timestamp

**Roles:**
- `admin` - Full access, can manage users and settings
- `member` - Can manage projects, alerts, policies
- `viewer` - Read-only access
- `api_only` - Programmatic access only (no UI login)

---

### policies

Security policies defining rules for projects.

```sql
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
```

**Fields:**
- `id` - Unique policy identifier
- `tenant_id` - Associated tenant
- `name` - Policy name
- `description` - Policy description
- `rules` - Array of policy rules (JSONB)
- `severity_overrides` - Package-specific severity adjustments
- `is_default` - Whether this is the default policy for new projects
- `enabled` - Whether this policy is active
- `created_at` - Policy creation timestamp
- `updated_at` - Last modification timestamp

**Rules JSONB Structure:**
```json
[
  {
    "type": "BlockDeprecated",
    "severity": "high"
  },
  {
    "type": "RequireProvenance",
    "min_slsa_level": 2,
    "apply_to": "direct"
  },
  {
    "type": "BlockPackageAge",
    "min_age_days": 30,
    "severity": "medium",
    "exemptions": ["@myorg/*"]
  }
]
```

---

### projects

Software projects being monitored for supply chain threats.

```sql
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
```

**Fields:**
- `id` - Unique project identifier
- `tenant_id` - Associated tenant
- `name` - Project name
- `description` - Project description
- `repository_url` - Git repository URL (optional)
- `default_branch` - Default branch name
- `ecosystems` - Array of package ecosystems (npm, pypi, maven, etc.)
- `scan_schedule` - Scan frequency configuration
- `policy_id` - Associated security policy (nullable)
- `last_scan_at` - Last successful scan timestamp
- `status` - Project health: `healthy`, `at_risk`, `vulnerable`, `unknown`
- `metadata` - Additional project metadata
- `created_at` - Project creation timestamp
- `updated_at` - Last modification timestamp

**Scan Schedule JSONB:**
```json
{
  "type": "daily",
  "hour": 2,
  "timezone": "UTC"
}
// or
{
  "type": "cron",
  "expression": "0 2 * * *"
}
// or
{
  "type": "manual"
}
```

**Status Values:**
- `healthy` - No critical/high alerts
- `at_risk` - Has medium severity alerts
- `vulnerable` - Has critical/high severity alerts
- `unknown` - Never scanned or scan failed

---

### dependencies

Discovered dependencies from project scans.

```sql
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
```

**Fields:**
- `id` - Unique dependency identifier
- `project_id` - Associated project
- `tenant_id` - Associated tenant (for isolation)
- `package_name` - Package name (e.g., "lodash")
- `ecosystem` - Package ecosystem (npm, pypi, maven, etc.)
- `version_constraint` - Version specifier from manifest (e.g., "^4.17.0")
- `resolved_version` - Actual resolved version (e.g., "4.17.21")
- `is_direct` - Whether this is a direct dependency
- `is_dev_dependency` - Whether this is a dev/test dependency
- `depth` - Dependency tree depth (0 = direct, 1 = transitive, etc.)
- `parent_id` - Parent dependency in tree (nullable for direct deps)
- `hash_sha256` - SHA-256 hash of package artifact
- `hash_sha512` - SHA-512 hash of package artifact
- `signature_status` - Signature verification status
- `provenance_status` - SLSA provenance status
- `provenance_details` - Detailed provenance information (JSONB)
- `first_seen_at` - When dependency was first discovered
- `last_verified_at` - Last integrity verification timestamp

**Signature Status Values:**
- `verified` - Valid signature found
- `invalid` - Signature verification failed
- `missing` - No signature available
- `unknown` - Not yet checked

**Provenance Status Values:**
- `verified` - Valid SLSA provenance
- `invalid` - Provenance verification failed
- `missing` - No provenance attestation
- `unknown` - Not yet checked

**Provenance Details JSONB:**
```json
{
  "slsa_level": 2,
  "builder": {
    "id": "https://github.com/actions",
    "version": "v1"
  },
  "build_type": "https://github.com/slsa-framework/slsa-github-generator/generic@v1",
  "invocation": {
    "config_source": {
      "uri": "git+https://github.com/user/repo@refs/heads/main",
      "digest": {"sha1": "abc123"}
    }
  },
  "materials": [],
  "verified_at": "2026-01-15T10:30:00Z"
}
```

---

### packages

Global package metadata cache (shared across tenants).

```sql
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
```

**Fields:**
- `id` - Unique package identifier
- `ecosystem` - Package ecosystem
- `name` - Package name
- `normalized_name` - Normalized name for comparison (lowercase, no special chars)
- `description` - Package description
- `homepage` - Project homepage URL
- `repository` - Source repository URL
- `popularity_rank` - Download/popularity ranking (lower is more popular)
- `is_popular` - Whether package is in top 1000
- `maintainers` - Array of maintainer information
- `first_published` - First publication date
- `last_updated` - Last update timestamp
- `metadata` - Additional ecosystem-specific metadata
- `cached_at` - When this data was cached

**Maintainers JSONB:**
```json
[
  {
    "username": "johndoe",
    "email": "john@example.com",
    "added_at": "2020-01-01T00:00:00Z"
  }
]
```

---

### package_versions

Individual package version information (global cache).

```sql
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
```

**Fields:**
- `id` - Unique version identifier
- `package_id` - Associated package
- `version` - Version string (e.g., "4.17.21")
- `published_at` - Publication timestamp
- `yanked` - Whether version was yanked/removed
- `deprecated` - Whether version is deprecated
- `deprecation_message` - Deprecation notice
- `checksums` - Hash checksums (JSONB)
- `download_url` - Package download URL
- `size_bytes` - Package size in bytes
- `attestations` - SLSA/Sigstore attestations (JSONB array)
- `dependencies` - Version dependencies (JSONB array)
- `cached_at` - Cache timestamp

**Checksums JSONB:**
```json
{
  "sha1": "abc123...",
  "sha256": "def456...",
  "sha512": "ghi789..."
}
```

**Attestations JSONB:**
```json
[
  {
    "type": "slsa_provenance",
    "version": "v0.2",
    "predicate_type": "https://slsa.dev/provenance/v0.2",
    "bundle": "base64-encoded-bundle"
  }
]
```

---

### alerts

Security alerts generated by threat detectors.

```sql
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
```

**Fields:**
- `id` - Unique alert identifier
- `tenant_id` - Associated tenant
- `project_id` - Associated project
- `dependency_id` - Associated dependency (nullable)
- `alert_type` - Type of threat detected
- `alert_details` - Type-specific details (JSONB)
- `severity` - Alert severity level
- `title` - Alert title
- `description` - Detailed description
- `status` - Alert status
- `remediation` - Recommended remediation steps
- `metadata` - Additional metadata
- `created_at` - Alert creation timestamp
- `acknowledged_at` - When alert was acknowledged
- `acknowledged_by` - User who acknowledged
- `resolved_at` - When alert was resolved
- `resolved_by` - User who resolved

**Alert Types:**
- `dependency_tampering` - Hash mismatch
- `downgrade_attack` - Suspicious version downgrade
- `typosquatting` - Name similarity attack
- `provenance_failure` - Missing/invalid provenance
- `policy_violation` - Policy rule violation
- `new_package` - Recently published package
- `suspicious_maintainer` - Unusual maintainer activity

**Severity Values:**
- `critical` - Immediate action required
- `high` - Urgent attention needed
- `medium` - Should be addressed
- `low` - Informational
- `info` - Reference information

**Status Values:**
- `open` - New, unaddressed alert
- `acknowledged` - Team is aware, investigating
- `resolved` - Issue has been fixed
- `suppressed` - False positive or accepted risk

---

### sboms

Generated Software Bill of Materials.

```sql
CREATE TABLE sboms (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    project_id UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    format VARCHAR(50) NOT NULL,
    format_version VARCHAR(20) NOT NULL,
    content JSONB NOT NULL,
    generated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    scan_id UUID
);
```

**Fields:**
- `id` - Unique SBOM identifier
- `project_id` - Associated project
- `tenant_id` - Associated tenant
- `format` - SBOM format: `cyclonedx`, `spdx`
- `format_version` - Format version (e.g., "1.5", "2.3")
- `content` - Complete SBOM document (JSONB)
- `generated_at` - Generation timestamp
- `scan_id` - Associated scan job ID (nullable)

---

### jobs

Background job queue.

```sql
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
```

**Fields:**
- `id` - Unique job identifier
- `tenant_id` - Associated tenant (nullable for system jobs)
- `job_type` - Job type: `scan_project`, `verify_provenance`, `monitor_registry`, `send_notification`
- `status` - Job status
- `priority` - Job priority (higher = more important)
- `payload` - Job input data (JSONB)
- `result` - Job output data (JSONB, nullable)
- `error_message` - Error message if failed
- `attempts` - Number of execution attempts
- `max_attempts` - Maximum retry attempts
- `scheduled_at` - When job should run
- `started_at` - When job execution started
- `completed_at` - When job finished
- `created_at` - Job creation timestamp

**Status Values:**
- `pending` - Waiting to be claimed
- `running` - Currently executing
- `completed` - Successfully finished
- `failed` - Execution failed (after retries)
- `scheduled` - Waiting for scheduled time

---

### audit_logs

Complete audit trail of user actions.

```sql
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
```

**Fields:**
- `id` - Unique log entry identifier
- `tenant_id` - Associated tenant
- `user_id` - User who performed action (nullable for system actions)
- `action` - Action performed
- `resource_type` - Type of resource affected
- `resource_id` - Identifier of affected resource
- `details` - Action-specific details (JSONB)
- `ip_address` - Client IP address
- `user_agent` - Client user agent
- `created_at` - Action timestamp

**Action Values:**
- `create`, `update`, `delete` - CRUD operations
- `scan`, `acknowledge_alert`, `resolve_alert` - Security actions
- `login`, `logout`, `api_key_created` - Authentication
- `policy_created`, `policy_updated` - Policy management

---

## Indexes

### Performance Indexes

```sql
-- Tenant lookups
CREATE INDEX idx_tenants_slug ON tenants(slug);

-- User lookups
CREATE INDEX idx_users_tenant ON users(tenant_id);
CREATE INDEX idx_users_api_key ON users(api_key_hash) WHERE api_key_hash IS NOT NULL;

-- Policy lookups
CREATE INDEX idx_policies_tenant ON policies(tenant_id);
CREATE INDEX idx_policies_default ON policies(tenant_id, is_default) WHERE is_default = true;

-- Project lookups
CREATE INDEX idx_projects_tenant ON projects(tenant_id);
CREATE INDEX idx_projects_status ON projects(status);
CREATE INDEX idx_projects_last_scan ON projects(last_scan_at);

-- Dependency lookups
CREATE INDEX idx_dependencies_project ON dependencies(project_id);
CREATE INDEX idx_dependencies_tenant ON dependencies(tenant_id);
CREATE INDEX idx_dependencies_package ON dependencies(package_name, ecosystem);
CREATE INDEX idx_dependencies_ecosystem ON dependencies(ecosystem);

-- Package cache lookups
CREATE INDEX idx_packages_ecosystem ON packages(ecosystem);
CREATE INDEX idx_packages_normalized ON packages(normalized_name);
CREATE INDEX idx_packages_popular ON packages(ecosystem, is_popular) WHERE is_popular = true;
CREATE INDEX idx_packages_name_trgm ON packages USING gin(name gin_trgm_ops);

-- Alert lookups
CREATE INDEX idx_alerts_tenant ON alerts(tenant_id);
CREATE INDEX idx_alerts_project ON alerts(project_id);
CREATE INDEX idx_alerts_status ON alerts(status);
CREATE INDEX idx_alerts_severity ON alerts(severity);
CREATE INDEX idx_alerts_created ON alerts(created_at DESC);
CREATE INDEX idx_alerts_type ON alerts(alert_type);

-- Job queue indexes
CREATE INDEX idx_jobs_status ON jobs(status) WHERE status IN ('pending', 'running');
CREATE INDEX idx_jobs_scheduled ON jobs(scheduled_at) WHERE status = 'pending';
CREATE INDEX idx_jobs_tenant ON jobs(tenant_id);
CREATE INDEX idx_jobs_type ON jobs(job_type);

-- Audit log indexes
CREATE INDEX idx_audit_tenant ON audit_logs(tenant_id);
CREATE INDEX idx_audit_created ON audit_logs(created_at DESC);
CREATE INDEX idx_audit_resource ON audit_logs(resource_type, resource_id);
```

---

## Constraints and Relationships

### Foreign Key Constraints

```sql
-- All user data cascades on tenant deletion
users.tenant_id → tenants.id ON DELETE CASCADE
policies.tenant_id → tenants.id ON DELETE CASCADE
projects.tenant_id → tenants.id ON DELETE CASCADE
dependencies.tenant_id → tenants.id ON DELETE CASCADE
alerts.tenant_id → tenants.id ON DELETE CASCADE
sboms.tenant_id → tenants.id ON DELETE CASCADE

-- Project data cascades on project deletion
dependencies.project_id → projects.id ON DELETE CASCADE
alerts.project_id → projects.id ON DELETE CASCADE
sboms.project_id → projects.id ON DELETE CASCADE

-- Policy reference is nullable (SET NULL on deletion)
projects.policy_id → policies.id ON DELETE SET NULL

-- Package cache relationships
package_versions.package_id → packages.id ON DELETE CASCADE
```

### Unique Constraints

```sql
-- Prevent duplicate tenant slugs
UNIQUE(slug) ON tenants

-- One email per tenant
UNIQUE(tenant_id, email) ON users

-- One policy name per tenant
UNIQUE(tenant_id, name) ON policies

-- One project name per tenant
UNIQUE(tenant_id, name) ON projects

-- One dependency version per project
UNIQUE(project_id, package_name, ecosystem, resolved_version) ON dependencies

-- One package per ecosystem
UNIQUE(ecosystem, name) ON packages

-- One version per package
UNIQUE(package_id, version) ON package_versions
```

---

## Multi-Tenancy

### Row-Level Security (RLS)

While SCTV implements tenant isolation at the application layer, you can also enable PostgreSQL RLS for defense-in-depth:

```sql
-- Enable RLS on tenant-scoped tables
ALTER TABLE users ENABLE ROW LEVEL SECURITY;
ALTER TABLE projects ENABLE ROW LEVEL SECURITY;
ALTER TABLE dependencies ENABLE ROW LEVEL SECURITY;
ALTER TABLE alerts ENABLE ROW LEVEL SECURITY;
ALTER TABLE sboms ENABLE ROW LEVEL SECURITY;
ALTER TABLE policies ENABLE ROW LEVEL SECURITY;

-- Create RLS policies
CREATE POLICY tenant_isolation ON users
    USING (tenant_id = current_setting('app.current_tenant_id')::uuid);

CREATE POLICY tenant_isolation ON projects
    USING (tenant_id = current_setting('app.current_tenant_id')::uuid);

-- Repeat for other tables...
```

### Application-Level Isolation

All repository methods automatically filter by `tenant_id`:

```rust
// Example: ProjectRepository
async fn list_by_tenant(&self, tenant_id: &TenantId) -> Result<Vec<Project>> {
    sqlx::query_as!(
        ProjectModel,
        "SELECT * FROM projects WHERE tenant_id = $1 ORDER BY name",
        tenant_id.as_uuid()
    )
    .fetch_all(&self.pool)
    .await
}
```

---

## Migrations

### Migration Files

Migrations are located in the `migrations/` directory and are applied using SQLx:

```
migrations/
├── 001_initial_schema.sql
├── 002_add_sbom_support.sql
├── 003_add_audit_logs.sql
└── 004_add_provenance_fields.sql
```

### Running Migrations

```bash
# Apply all pending migrations
sqlx migrate run

# Revert last migration
sqlx migrate revert

# Check migration status
sqlx migrate info
```

### Migration Best Practices

1. **Never modify existing migrations** - Create new migrations instead
2. **Test migrations** - Test both up and down migrations
3. **Use transactions** - Wrap DDL in transactions where possible
4. **Document changes** - Add comments explaining the purpose
5. **Consider data** - Plan for data migration, not just schema

---

## Performance Considerations

### Query Optimization

1. **Use prepared statements** - SQLx automatically prepares queries
2. **Leverage indexes** - All foreign keys and common queries are indexed
3. **Limit result sets** - Always use `LIMIT` and pagination
4. **Use JSONB efficiently** - Index JSONB fields when needed:
   ```sql
   CREATE INDEX idx_policy_rules ON policies USING gin(rules);
   ```

### Connection Pooling

Configure appropriate pool size based on workload:

```toml
[database]
max_connections = 50
min_connections = 10
idle_timeout_seconds = 600
```

### Vacuum and Maintenance

Schedule regular maintenance:

```sql
-- Auto-vacuum configuration
ALTER TABLE dependencies SET (autovacuum_vacuum_scale_factor = 0.1);
ALTER TABLE alerts SET (autovacuum_vacuum_scale_factor = 0.1);

-- Analyze tables for query planner
ANALYZE projects;
ANALYZE dependencies;
ANALYZE alerts;
```

### Partitioning (Future)

For high-volume deployments, consider partitioning large tables:

```sql
-- Partition alerts by created_at (monthly)
CREATE TABLE alerts_2026_01 PARTITION OF alerts
    FOR VALUES FROM ('2026-01-01') TO ('2026-02-01');

-- Partition audit_logs by created_at (monthly)
CREATE TABLE audit_logs_2026_01 PARTITION OF audit_logs
    FOR VALUES FROM ('2026-01-01') TO ('2026-02-01');
```

---

## Next Steps

- [Data Flow Documentation](data-flow.md) - Understand data flow patterns
- [Performance Tuning](../operations/performance.md) - Optimize database performance
- [Backup Strategy](../operations/backup.md) - Database backup and recovery
