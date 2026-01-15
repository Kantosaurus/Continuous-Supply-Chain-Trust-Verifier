//! Project repository implementation.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sctv_core::traits::{ProjectRepository, RepositoryError, RepositoryResult};
use sctv_core::{
    PackageEcosystem, PolicyId, Project, ProjectId, ProjectMetadata, ProjectStatus, ScanSchedule,
    TenantId,
};
use sqlx::{PgPool, Row};
use url::Url;

/// PostgreSQL implementation of the project repository.
pub struct PgProjectRepository {
    pool: PgPool,
}

impl PgProjectRepository {
    /// Creates a new project repository.
    #[must_use]
    pub const fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    fn row_to_project(row: &sqlx::postgres::PgRow) -> RepositoryResult<Project> {
        let id: uuid::Uuid = row.get("id");
        let tenant_id: uuid::Uuid = row.get("tenant_id");
        let name: String = row.get("name");
        let description: Option<String> = row.get("description");
        let repository_url: Option<String> = row.get("repository_url");
        let default_branch: String = row.get("default_branch");
        let ecosystems: Vec<String> = row.get("ecosystems");
        let scan_schedule: serde_json::Value = row.get("scan_schedule");
        let policy_id: Option<uuid::Uuid> = row.get("policy_id");
        let last_scan_at: Option<DateTime<Utc>> = row.get("last_scan_at");
        let status: String = row.get("status");
        let metadata: serde_json::Value = row.get("metadata");
        let created_at: DateTime<Utc> = row.get("created_at");
        let updated_at: DateTime<Utc> = row.get("updated_at");

        let repository_url = repository_url
            .and_then(|u| Url::parse(&u).ok());

        let ecosystems: Vec<PackageEcosystem> = ecosystems
            .into_iter()
            .filter_map(|e| e.parse().ok())
            .collect();

        let scan_schedule: ScanSchedule = serde_json::from_value(scan_schedule)
            .map_err(|e| RepositoryError::Serialization(e.to_string()))?;

        let status = match status.as_str() {
            "healthy" => ProjectStatus::Healthy,
            "warning" => ProjectStatus::Warning,
            "critical" => ProjectStatus::Critical,
            _ => ProjectStatus::Unknown,
        };

        let metadata: ProjectMetadata = serde_json::from_value(metadata)
            .unwrap_or_default();

        Ok(Project {
            id: ProjectId(id),
            tenant_id: TenantId(tenant_id),
            name,
            description,
            repository_url,
            default_branch,
            ecosystems,
            scan_schedule,
            policy_id: policy_id.map(PolicyId),
            last_scan_at,
            status,
            metadata,
            created_at,
            updated_at,
        })
    }
}

#[async_trait]
impl ProjectRepository for PgProjectRepository {
    async fn find_by_id(&self, id: ProjectId) -> RepositoryResult<Option<Project>> {
        let record = sqlx::query(
            r#"
            SELECT id, tenant_id, name, description, repository_url, default_branch,
                   ecosystems, scan_schedule, policy_id, last_scan_at, status,
                   metadata, created_at, updated_at
            FROM projects WHERE id = $1
            "#,
        )
        .bind(id.0)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

        match record {
            Some(row) => Ok(Some(Self::row_to_project(&row)?)),
            None => Ok(None),
        }
    }

    async fn find_by_tenant(&self, tenant_id: TenantId) -> RepositoryResult<Vec<Project>> {
        let records = sqlx::query(
            r#"
            SELECT id, tenant_id, name, description, repository_url, default_branch,
                   ecosystems, scan_schedule, policy_id, last_scan_at, status,
                   metadata, created_at, updated_at
            FROM projects WHERE tenant_id = $1
            ORDER BY name ASC
            "#,
        )
        .bind(tenant_id.0)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

        records
            .iter()
            .map(Self::row_to_project)
            .collect()
    }

    async fn create(&self, project: &Project) -> RepositoryResult<()> {
        let ecosystems: Vec<String> = project
            .ecosystems
            .iter()
            .map(|e| e.to_string().to_lowercase())
            .collect();

        let scan_schedule = serde_json::to_value(&project.scan_schedule)
            .map_err(|e| RepositoryError::Serialization(e.to_string()))?;

        let metadata = serde_json::to_value(&project.metadata)
            .map_err(|e| RepositoryError::Serialization(e.to_string()))?;

        let status = match project.status {
            ProjectStatus::Healthy => "healthy",
            ProjectStatus::Warning => "warning",
            ProjectStatus::Critical => "critical",
            ProjectStatus::Unknown => "unknown",
        };

        sqlx::query(
            r#"
            INSERT INTO projects (
                id, tenant_id, name, description, repository_url, default_branch,
                ecosystems, scan_schedule, policy_id, last_scan_at, status,
                metadata, created_at, updated_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
            "#,
        )
        .bind(project.id.0)
        .bind(project.tenant_id.0)
        .bind(&project.name)
        .bind(&project.description)
        .bind(project.repository_url.as_ref().map(|u| u.as_str()))
        .bind(&project.default_branch)
        .bind(&ecosystems)
        .bind(scan_schedule)
        .bind(project.policy_id.map(|p| p.0))
        .bind(project.last_scan_at)
        .bind(status)
        .bind(metadata)
        .bind(project.created_at)
        .bind(project.updated_at)
        .execute(&self.pool)
        .await
        .map_err(|e| {
            if e.to_string().contains("duplicate key") {
                RepositoryError::AlreadyExists
            } else {
                RepositoryError::Database(e.to_string())
            }
        })?;

        Ok(())
    }

    async fn update(&self, project: &Project) -> RepositoryResult<()> {
        let ecosystems: Vec<String> = project
            .ecosystems
            .iter()
            .map(|e| e.to_string().to_lowercase())
            .collect();

        let scan_schedule = serde_json::to_value(&project.scan_schedule)
            .map_err(|e| RepositoryError::Serialization(e.to_string()))?;

        let metadata = serde_json::to_value(&project.metadata)
            .map_err(|e| RepositoryError::Serialization(e.to_string()))?;

        let status = match project.status {
            ProjectStatus::Healthy => "healthy",
            ProjectStatus::Warning => "warning",
            ProjectStatus::Critical => "critical",
            ProjectStatus::Unknown => "unknown",
        };

        let result = sqlx::query(
            r#"
            UPDATE projects SET
                name = $2, description = $3, repository_url = $4, default_branch = $5,
                ecosystems = $6, scan_schedule = $7, policy_id = $8, last_scan_at = $9,
                status = $10, metadata = $11, updated_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(project.id.0)
        .bind(&project.name)
        .bind(&project.description)
        .bind(project.repository_url.as_ref().map(|u| u.as_str()))
        .bind(&project.default_branch)
        .bind(&ecosystems)
        .bind(scan_schedule)
        .bind(project.policy_id.map(|p| p.0))
        .bind(project.last_scan_at)
        .bind(status)
        .bind(metadata)
        .execute(&self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(RepositoryError::NotFound);
        }

        Ok(())
    }

    async fn delete(&self, id: ProjectId) -> RepositoryResult<()> {
        let result = sqlx::query("DELETE FROM projects WHERE id = $1")
            .bind(id.0)
            .execute(&self.pool)
            .await
            .map_err(|e| RepositoryError::Database(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(RepositoryError::NotFound);
        }

        Ok(())
    }

    async fn find_due_for_scan(&self) -> RepositoryResult<Vec<Project>> {
        // Find projects that haven't been scanned in the last hour
        // or have never been scanned
        let records = sqlx::query(
            r#"
            SELECT id, tenant_id, name, description, repository_url, default_branch,
                   ecosystems, scan_schedule, policy_id, last_scan_at, status,
                   metadata, created_at, updated_at
            FROM projects
            WHERE last_scan_at IS NULL
               OR last_scan_at < NOW() - INTERVAL '1 hour'
            ORDER BY last_scan_at ASC NULLS FIRST
            LIMIT 100
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

        records
            .iter()
            .map(Self::row_to_project)
            .collect()
    }

    async fn count_by_tenant(&self, tenant_id: TenantId) -> RepositoryResult<u32> {
        let record = sqlx::query("SELECT COUNT(*) as count FROM projects WHERE tenant_id = $1")
            .bind(tenant_id.0)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| RepositoryError::Database(e.to_string()))?;

        let count: i64 = record.get("count");
        Ok(count as u32)
    }
}
