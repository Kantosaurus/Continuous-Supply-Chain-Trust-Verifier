//! Repository pattern traits for data access.

use async_trait::async_trait;
use thiserror::Error;

use crate::domain::{
    Alert, AlertId, AlertStatus, ApiKey, ApiKeyId, AuditLog, AuditLogFilter, AuditLogId,
    Dependency, DependencyId, Job, JobId, JobStatus, Package, PackageEcosystem, PackageId, Policy,
    PolicyId, Project, ProjectId, Sbom, SbomFormat, SbomId, Severity, Tenant, TenantId, User,
    UserId,
};

/// Errors that can occur during repository operations.
#[derive(Debug, Error)]
pub enum RepositoryError {
    #[error("Entity not found")]
    NotFound,

    #[error("Entity already exists")]
    AlreadyExists,

    #[error("Database error: {0}")]
    Database(String),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Invalid data: {0}")]
    InvalidData(String),
}

/// Result type for repository operations.
pub type RepositoryResult<T> = Result<T, RepositoryError>;

/// Repository for tenant operations.
#[async_trait]
pub trait TenantRepository: Send + Sync {
    /// Finds a tenant by ID.
    async fn find_by_id(&self, id: TenantId) -> RepositoryResult<Option<Tenant>>;

    /// Finds a tenant by slug.
    async fn find_by_slug(&self, slug: &str) -> RepositoryResult<Option<Tenant>>;

    /// Creates a new tenant.
    async fn create(&self, tenant: &Tenant) -> RepositoryResult<()>;

    /// Updates an existing tenant.
    async fn update(&self, tenant: &Tenant) -> RepositoryResult<()>;

    /// Deletes a tenant.
    async fn delete(&self, id: TenantId) -> RepositoryResult<()>;

    /// Lists all tenants.
    async fn list(&self, limit: u32, offset: u32) -> RepositoryResult<Vec<Tenant>>;
}

/// Repository for project operations.
#[async_trait]
pub trait ProjectRepository: Send + Sync {
    /// Finds a project by ID.
    async fn find_by_id(&self, id: ProjectId) -> RepositoryResult<Option<Project>>;

    /// Finds projects by tenant.
    async fn find_by_tenant(&self, tenant_id: TenantId) -> RepositoryResult<Vec<Project>>;

    /// Creates a new project.
    async fn create(&self, project: &Project) -> RepositoryResult<()>;

    /// Updates an existing project.
    async fn update(&self, project: &Project) -> RepositoryResult<()>;

    /// Deletes a project.
    async fn delete(&self, id: ProjectId) -> RepositoryResult<()>;

    /// Lists projects that need scanning.
    async fn find_due_for_scan(&self) -> RepositoryResult<Vec<Project>>;

    /// Counts projects for a tenant.
    async fn count_by_tenant(&self, tenant_id: TenantId) -> RepositoryResult<u32>;
}

/// Repository for dependency operations.
#[async_trait]
pub trait DependencyRepository: Send + Sync {
    /// Finds a dependency by ID.
    async fn find_by_id(&self, id: DependencyId) -> RepositoryResult<Option<Dependency>>;

    /// Finds dependencies for a project.
    async fn find_by_project(&self, project_id: ProjectId) -> RepositoryResult<Vec<Dependency>>;

    /// Finds direct dependencies for a project.
    async fn find_direct_by_project(
        &self,
        project_id: ProjectId,
    ) -> RepositoryResult<Vec<Dependency>>;

    /// Creates a new dependency.
    async fn create(&self, dependency: &Dependency) -> RepositoryResult<()>;

    /// Creates multiple dependencies in batch.
    async fn create_batch(&self, dependencies: &[Dependency]) -> RepositoryResult<()>;

    /// Updates an existing dependency.
    async fn update(&self, dependency: &Dependency) -> RepositoryResult<()>;

    /// Deletes a dependency.
    async fn delete(&self, id: DependencyId) -> RepositoryResult<()>;

    /// Deletes all dependencies for a project.
    async fn delete_by_project(&self, project_id: ProjectId) -> RepositoryResult<u32>;

    /// Finds a dependency by package name and version.
    async fn find_by_package(
        &self,
        project_id: ProjectId,
        ecosystem: PackageEcosystem,
        package_name: &str,
        version: &str,
    ) -> RepositoryResult<Option<Dependency>>;
}

/// Repository for package operations.
#[async_trait]
pub trait PackageRepository: Send + Sync {
    /// Finds a package by ID.
    async fn find_by_id(&self, id: PackageId) -> RepositoryResult<Option<Package>>;

    /// Finds a package by ecosystem and name.
    async fn find_by_name(
        &self,
        ecosystem: PackageEcosystem,
        name: &str,
    ) -> RepositoryResult<Option<Package>>;

    /// Creates or updates a package.
    async fn upsert(&self, package: &Package) -> RepositoryResult<()>;

    /// Lists popular packages for an ecosystem.
    async fn find_popular(
        &self,
        ecosystem: PackageEcosystem,
        limit: u32,
    ) -> RepositoryResult<Vec<Package>>;

    /// Searches packages by name prefix.
    async fn search_by_name(
        &self,
        ecosystem: PackageEcosystem,
        prefix: &str,
        limit: u32,
    ) -> RepositoryResult<Vec<Package>>;
}

/// Repository for policy operations.
#[async_trait]
pub trait PolicyRepository: Send + Sync {
    /// Finds a policy by ID.
    async fn find_by_id(&self, id: PolicyId) -> RepositoryResult<Option<Policy>>;

    /// Finds policies for a tenant.
    async fn find_by_tenant(&self, tenant_id: TenantId) -> RepositoryResult<Vec<Policy>>;

    /// Finds the default policy for a tenant.
    async fn find_default(&self, tenant_id: TenantId) -> RepositoryResult<Option<Policy>>;

    /// Creates a new policy.
    async fn create(&self, policy: &Policy) -> RepositoryResult<()>;

    /// Updates an existing policy.
    async fn update(&self, policy: &Policy) -> RepositoryResult<()>;

    /// Deletes a policy.
    async fn delete(&self, id: PolicyId) -> RepositoryResult<()>;

    /// Sets the default policy for a tenant.
    async fn set_default(&self, tenant_id: TenantId, policy_id: PolicyId) -> RepositoryResult<()>;
}

/// Filter options for alert queries.
#[derive(Debug, Clone, Default)]
pub struct AlertFilter {
    pub project_id: Option<ProjectId>,
    pub status: Option<Vec<AlertStatus>>,
    pub severity: Option<Vec<Severity>>,
    pub alert_type: Option<Vec<String>>,
}

/// Repository for alert operations.
#[async_trait]
pub trait AlertRepository: Send + Sync {
    /// Finds an alert by ID.
    async fn find_by_id(&self, id: AlertId) -> RepositoryResult<Option<Alert>>;

    /// Finds alerts for a project.
    async fn find_by_project(&self, project_id: ProjectId) -> RepositoryResult<Vec<Alert>>;

    /// Finds alerts with filters.
    async fn find_with_filter(
        &self,
        tenant_id: TenantId,
        filter: AlertFilter,
        limit: u32,
        offset: u32,
    ) -> RepositoryResult<Vec<Alert>>;

    /// Counts alerts matching the same filter (used for pagination totals).
    async fn count_with_filter(
        &self,
        tenant_id: TenantId,
        filter: AlertFilter,
    ) -> RepositoryResult<u64>;

    /// Creates a new alert.
    async fn create(&self, alert: &Alert) -> RepositoryResult<()>;

    /// Updates an existing alert.
    async fn update(&self, alert: &Alert) -> RepositoryResult<()>;

    /// Counts open alerts for a project.
    async fn count_open_by_project(&self, project_id: ProjectId) -> RepositoryResult<u32>;

    /// Counts alerts by severity for a project.
    async fn count_by_severity(
        &self,
        project_id: ProjectId,
    ) -> RepositoryResult<std::collections::HashMap<Severity, u32>>;
}

/// Repository for user operations.
#[async_trait]
pub trait UserRepository: Send + Sync {
    /// Finds a user by ID.
    async fn find_by_id(&self, id: UserId) -> RepositoryResult<Option<User>>;

    /// Finds a user by email within a tenant.
    async fn find_by_email(&self, tenant_id: TenantId, email: &str) -> RepositoryResult<Option<User>>;

    /// Finds a user by API key hash.
    async fn find_by_api_key(&self, api_key_hash: &str) -> RepositoryResult<Option<User>>;

    /// Finds users for a tenant.
    async fn find_by_tenant(&self, tenant_id: TenantId) -> RepositoryResult<Vec<User>>;

    /// Creates a new user.
    async fn create(&self, user: &User) -> RepositoryResult<()>;

    /// Updates an existing user.
    async fn update(&self, user: &User) -> RepositoryResult<()>;

    /// Deletes a user.
    async fn delete(&self, id: UserId) -> RepositoryResult<()>;

    /// Updates the last login timestamp.
    async fn update_last_login(&self, id: UserId) -> RepositoryResult<()>;

    /// Counts users for a tenant.
    async fn count_by_tenant(&self, tenant_id: TenantId) -> RepositoryResult<u32>;
}

/// Filter options for job queries.
#[derive(Debug, Clone, Default)]
pub struct JobFilter {
    pub status: Option<Vec<JobStatus>>,
    pub job_type: Option<Vec<String>>,
}

/// Repository for background job operations.
#[async_trait]
pub trait JobRepository: Send + Sync {
    /// Finds a job by ID.
    async fn find_by_id(&self, id: JobId) -> RepositoryResult<Option<Job>>;

    /// Finds jobs for a tenant with optional filters.
    async fn find_by_tenant(
        &self,
        tenant_id: Option<TenantId>,
        filter: JobFilter,
        limit: u32,
        offset: u32,
    ) -> RepositoryResult<Vec<Job>>;

    /// Creates a new job.
    async fn create(&self, job: &Job) -> RepositoryResult<()>;

    /// Updates an existing job.
    async fn update(&self, job: &Job) -> RepositoryResult<()>;

    /// Claims the next pending job for processing.
    async fn claim_next(&self) -> RepositoryResult<Option<Job>>;

    /// Finds jobs that are due for execution.
    async fn find_due_jobs(&self, limit: u32) -> RepositoryResult<Vec<Job>>;

    /// Finds stale running jobs (for recovery).
    async fn find_stale_jobs(&self, older_than_seconds: u32) -> RepositoryResult<Vec<Job>>;

    /// Deletes old completed jobs.
    async fn cleanup_old_jobs(&self, older_than_days: u32) -> RepositoryResult<u32>;

    /// Counts jobs by status.
    async fn count_by_status(&self) -> RepositoryResult<std::collections::HashMap<JobStatus, u32>>;
}

/// Repository for SBOM operations.
#[async_trait]
pub trait SbomRepository: Send + Sync {
    /// Finds an SBOM by ID.
    async fn find_by_id(&self, id: SbomId) -> RepositoryResult<Option<Sbom>>;

    /// Finds SBOMs for a project.
    async fn find_by_project(&self, project_id: ProjectId) -> RepositoryResult<Vec<Sbom>>;

    /// Finds the latest SBOM for a project.
    async fn find_latest(&self, project_id: ProjectId) -> RepositoryResult<Option<Sbom>>;

    /// Finds the latest SBOM for a project in a specific format.
    async fn find_latest_by_format(
        &self,
        project_id: ProjectId,
        format: SbomFormat,
    ) -> RepositoryResult<Option<Sbom>>;

    /// Creates a new SBOM.
    async fn create(&self, sbom: &Sbom) -> RepositoryResult<()>;

    /// Deletes an SBOM.
    async fn delete(&self, id: SbomId) -> RepositoryResult<()>;

    /// Deletes old SBOMs for a project, keeping the most recent N.
    async fn cleanup_old_sboms(&self, project_id: ProjectId, keep_count: u32) -> RepositoryResult<u32>;
}

/// Repository for API key operations.
#[async_trait]
pub trait ApiKeyRepository: Send + Sync {
    /// Finds an active API key by its SHA-256 digest. Returns None if no
    /// match exists, is revoked, or is expired.
    async fn find_active_by_hash(&self, key_hash: &str) -> RepositoryResult<Option<ApiKey>>;

    /// Finds an API key by ID.
    async fn find_by_id(&self, id: ApiKeyId) -> RepositoryResult<Option<ApiKey>>;

    /// Lists API keys for a tenant (excluding revoked ones).
    async fn list_active_by_tenant(&self, tenant_id: TenantId) -> RepositoryResult<Vec<ApiKey>>;

    /// Creates a new API key row.
    async fn create(&self, key: &ApiKey) -> RepositoryResult<()>;

    /// Marks an API key revoked (soft-delete). Returns NotFound if the row
    /// doesn't exist or is already revoked.
    async fn revoke(&self, id: ApiKeyId) -> RepositoryResult<()>;

    /// Updates the last_used_at timestamp. Intended to be called on each
    /// successful auth but is best-effort; errors are logged, not propagated.
    async fn touch_last_used(&self, id: ApiKeyId) -> RepositoryResult<()>;
}

/// Repository for audit log operations.
#[async_trait]
pub trait AuditLogRepository: Send + Sync {
    /// Finds an audit log entry by ID.
    async fn find_by_id(&self, id: AuditLogId) -> RepositoryResult<Option<AuditLog>>;

    /// Finds audit logs for a tenant with optional filters.
    async fn find_by_tenant(
        &self,
        tenant_id: TenantId,
        filter: AuditLogFilter,
        limit: u32,
        offset: u32,
    ) -> RepositoryResult<Vec<AuditLog>>;

    /// Creates a new audit log entry.
    async fn create(&self, audit_log: &AuditLog) -> RepositoryResult<()>;

    /// Deletes old audit logs.
    async fn cleanup_old_logs(&self, older_than_days: u32) -> RepositoryResult<u32>;

    /// Counts audit logs for a tenant.
    async fn count_by_tenant(&self, tenant_id: TenantId) -> RepositoryResult<u32>;
}
