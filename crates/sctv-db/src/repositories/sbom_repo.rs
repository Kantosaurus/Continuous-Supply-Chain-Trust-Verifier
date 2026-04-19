//! SBOM repository implementation.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sctv_core::traits::{RepositoryError, RepositoryResult, SbomRepository};
use sctv_core::{ProjectId, Sbom, SbomFormat, SbomId, TenantId};
use sqlx::{PgPool, Row};

/// `PostgreSQL` implementation of the SBOM repository.
pub struct PgSbomRepository {
    pool: PgPool,
}

impl PgSbomRepository {
    /// Creates a new SBOM repository.
    #[must_use]
    pub const fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    fn row_to_sbom(row: &sqlx::postgres::PgRow) -> RepositoryResult<Sbom> {
        let id: uuid::Uuid = row.get("id");
        let project_id: uuid::Uuid = row.get("project_id");
        let tenant_id: uuid::Uuid = row.get("tenant_id");
        let format_str: String = row.get("format");
        let format_version: String = row.get("format_version");
        let content: serde_json::Value = row.get("content");
        let generated_at: DateTime<Utc> = row.get("generated_at");
        let scan_id: Option<uuid::Uuid> = row.get("scan_id");

        let format = format_str.parse::<SbomFormat>().map_err(|_| {
            RepositoryError::InvalidData(format!("Unknown SBOM format: {format_str}"))
        })?;

        Ok(Sbom {
            id: SbomId(id),
            project_id: ProjectId(project_id),
            tenant_id: TenantId(tenant_id),
            format,
            format_version,
            content,
            generated_at,
            scan_id,
        })
    }

    const fn format_to_str(format: SbomFormat) -> &'static str {
        match format {
            SbomFormat::CycloneDx => "cyclonedx",
            SbomFormat::Spdx => "spdx",
        }
    }
}

#[async_trait]
impl SbomRepository for PgSbomRepository {
    async fn find_by_id(&self, id: SbomId) -> RepositoryResult<Option<Sbom>> {
        let record = sqlx::query(
            r"
            SELECT id, project_id, tenant_id, format, format_version,
                   content, generated_at, scan_id
            FROM sboms WHERE id = $1
            ",
        )
        .bind(id.0)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

        match record {
            Some(row) => Ok(Some(Self::row_to_sbom(&row)?)),
            None => Ok(None),
        }
    }

    async fn find_by_project(&self, project_id: ProjectId) -> RepositoryResult<Vec<Sbom>> {
        let records = sqlx::query(
            r"
            SELECT id, project_id, tenant_id, format, format_version,
                   content, generated_at, scan_id
            FROM sboms
            WHERE project_id = $1
            ORDER BY generated_at DESC
            ",
        )
        .bind(project_id.0)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

        records.iter().map(Self::row_to_sbom).collect()
    }

    async fn find_latest(&self, project_id: ProjectId) -> RepositoryResult<Option<Sbom>> {
        let record = sqlx::query(
            r"
            SELECT id, project_id, tenant_id, format, format_version,
                   content, generated_at, scan_id
            FROM sboms
            WHERE project_id = $1
            ORDER BY generated_at DESC
            LIMIT 1
            ",
        )
        .bind(project_id.0)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

        match record {
            Some(row) => Ok(Some(Self::row_to_sbom(&row)?)),
            None => Ok(None),
        }
    }

    async fn find_latest_by_format(
        &self,
        project_id: ProjectId,
        format: SbomFormat,
    ) -> RepositoryResult<Option<Sbom>> {
        let record = sqlx::query(
            r"
            SELECT id, project_id, tenant_id, format, format_version,
                   content, generated_at, scan_id
            FROM sboms
            WHERE project_id = $1 AND format = $2
            ORDER BY generated_at DESC
            LIMIT 1
            ",
        )
        .bind(project_id.0)
        .bind(Self::format_to_str(format))
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

        match record {
            Some(row) => Ok(Some(Self::row_to_sbom(&row)?)),
            None => Ok(None),
        }
    }

    async fn create(&self, sbom: &Sbom) -> RepositoryResult<()> {
        sqlx::query(
            r"
            INSERT INTO sboms (
                id, project_id, tenant_id, format, format_version,
                content, generated_at, scan_id
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            ",
        )
        .bind(sbom.id.0)
        .bind(sbom.project_id.0)
        .bind(sbom.tenant_id.0)
        .bind(Self::format_to_str(sbom.format))
        .bind(&sbom.format_version)
        .bind(&sbom.content)
        .bind(sbom.generated_at)
        .bind(sbom.scan_id)
        .execute(&self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

        Ok(())
    }

    async fn delete(&self, id: SbomId) -> RepositoryResult<()> {
        let result = sqlx::query("DELETE FROM sboms WHERE id = $1")
            .bind(id.0)
            .execute(&self.pool)
            .await
            .map_err(|e| RepositoryError::Database(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(RepositoryError::NotFound);
        }

        Ok(())
    }

    async fn cleanup_old_sboms(
        &self,
        project_id: ProjectId,
        keep_count: u32,
    ) -> RepositoryResult<u32> {
        // Delete SBOMs older than the most recent `keep_count` for this project
        let result = sqlx::query(
            r"
            DELETE FROM sboms
            WHERE project_id = $1
              AND id NOT IN (
                  SELECT id FROM sboms
                  WHERE project_id = $1
                  ORDER BY generated_at DESC
                  LIMIT $2
              )
            ",
        )
        .bind(project_id.0)
        .bind(i64::from(keep_count))
        .execute(&self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

        Ok(result.rows_affected() as u32)
    }
}
