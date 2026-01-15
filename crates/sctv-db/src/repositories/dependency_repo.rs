//! Dependency repository implementation.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sctv_core::traits::{DependencyRepository, RepositoryError, RepositoryResult};
use sctv_core::{
    Dependency, DependencyId, DependencyIntegrity, PackageEcosystem, ProjectId,
    ProvenanceDetails, ProvenanceStatus, SignatureStatus, TenantId,
};
use semver::Version;
use sqlx::{PgPool, Row};

/// PostgreSQL implementation of the dependency repository.
pub struct PgDependencyRepository {
    pool: PgPool,
}

impl PgDependencyRepository {
    /// Creates a new dependency repository.
    #[must_use]
    pub const fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    fn row_to_dependency(row: &sqlx::postgres::PgRow) -> RepositoryResult<Dependency> {
        let id: uuid::Uuid = row.get("id");
        let project_id: uuid::Uuid = row.get("project_id");
        let tenant_id: uuid::Uuid = row.get("tenant_id");
        let package_name: String = row.get("package_name");
        let ecosystem: String = row.get("ecosystem");
        let version_constraint: Option<String> = row.get("version_constraint");
        let resolved_version: String = row.get("resolved_version");
        let is_direct: bool = row.get("is_direct");
        let is_dev_dependency: bool = row.get("is_dev_dependency");
        let depth: i32 = row.get("depth");
        let parent_id: Option<uuid::Uuid> = row.get("parent_id");
        let hash_sha256: Option<String> = row.get("hash_sha256");
        let hash_sha512: Option<String> = row.get("hash_sha512");
        let signature_status: String = row.get("signature_status");
        let provenance_status: String = row.get("provenance_status");
        let provenance_details: Option<serde_json::Value> = row.get("provenance_details");
        let first_seen_at: DateTime<Utc> = row.get("first_seen_at");
        let last_verified_at: DateTime<Utc> = row.get("last_verified_at");

        let ecosystem: PackageEcosystem = ecosystem
            .parse()
            .map_err(|_| RepositoryError::InvalidData(format!("Invalid ecosystem: {}", ecosystem)))?;

        let resolved_version = Version::parse(&resolved_version)
            .map_err(|e| RepositoryError::InvalidData(format!("Invalid version: {}", e)))?;

        let signature_status = match signature_status.as_str() {
            "verified" => SignatureStatus::Verified,
            "invalid" => SignatureStatus::Invalid,
            "missing" => SignatureStatus::Missing,
            _ => SignatureStatus::Unknown,
        };

        let provenance_status = match provenance_status.as_str() {
            "slsa_level0" => ProvenanceStatus::SlsaLevel0,
            "slsa_level1" => ProvenanceStatus::SlsaLevel1,
            "slsa_level2" => ProvenanceStatus::SlsaLevel2,
            "slsa_level3" => ProvenanceStatus::SlsaLevel3,
            "failed" => ProvenanceStatus::Failed,
            _ => ProvenanceStatus::Unknown,
        };

        let provenance_details: Option<ProvenanceDetails> = provenance_details
            .and_then(|v| serde_json::from_value(v).ok());

        let integrity = DependencyIntegrity {
            hash_sha256,
            hash_sha512,
            signature_status,
            provenance_status,
            provenance_details,
        };

        Ok(Dependency {
            id: DependencyId(id),
            project_id: ProjectId(project_id),
            tenant_id: TenantId(tenant_id),
            package_name,
            ecosystem,
            version_constraint: version_constraint.unwrap_or_default(),
            resolved_version,
            is_direct,
            is_dev_dependency,
            depth: depth as u32,
            parent_id: parent_id.map(DependencyId),
            integrity,
            first_seen_at,
            last_verified_at,
        })
    }

    fn signature_status_to_str(status: SignatureStatus) -> &'static str {
        match status {
            SignatureStatus::Verified => "verified",
            SignatureStatus::Invalid => "invalid",
            SignatureStatus::Missing => "missing",
            SignatureStatus::Unknown => "unknown",
        }
    }

    fn provenance_status_to_str(status: ProvenanceStatus) -> &'static str {
        match status {
            ProvenanceStatus::SlsaLevel0 => "slsa_level0",
            ProvenanceStatus::SlsaLevel1 => "slsa_level1",
            ProvenanceStatus::SlsaLevel2 => "slsa_level2",
            ProvenanceStatus::SlsaLevel3 => "slsa_level3",
            ProvenanceStatus::Failed => "failed",
            ProvenanceStatus::Unknown => "unknown",
        }
    }
}

#[async_trait]
impl DependencyRepository for PgDependencyRepository {
    async fn find_by_id(&self, id: DependencyId) -> RepositoryResult<Option<Dependency>> {
        let record = sqlx::query(
            r#"
            SELECT id, project_id, tenant_id, package_name, ecosystem, version_constraint,
                   resolved_version, is_direct, is_dev_dependency, depth, parent_id,
                   hash_sha256, hash_sha512, signature_status, provenance_status,
                   provenance_details, first_seen_at, last_verified_at
            FROM dependencies WHERE id = $1
            "#,
        )
        .bind(id.0)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

        match record {
            Some(row) => Ok(Some(Self::row_to_dependency(&row)?)),
            None => Ok(None),
        }
    }

    async fn find_by_project(&self, project_id: ProjectId) -> RepositoryResult<Vec<Dependency>> {
        let records = sqlx::query(
            r#"
            SELECT id, project_id, tenant_id, package_name, ecosystem, version_constraint,
                   resolved_version, is_direct, is_dev_dependency, depth, parent_id,
                   hash_sha256, hash_sha512, signature_status, provenance_status,
                   provenance_details, first_seen_at, last_verified_at
            FROM dependencies
            WHERE project_id = $1
            ORDER BY is_direct DESC, depth ASC, package_name ASC
            "#,
        )
        .bind(project_id.0)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

        records.iter().map(Self::row_to_dependency).collect()
    }

    async fn find_direct_by_project(
        &self,
        project_id: ProjectId,
    ) -> RepositoryResult<Vec<Dependency>> {
        let records = sqlx::query(
            r#"
            SELECT id, project_id, tenant_id, package_name, ecosystem, version_constraint,
                   resolved_version, is_direct, is_dev_dependency, depth, parent_id,
                   hash_sha256, hash_sha512, signature_status, provenance_status,
                   provenance_details, first_seen_at, last_verified_at
            FROM dependencies
            WHERE project_id = $1 AND is_direct = true
            ORDER BY package_name ASC
            "#,
        )
        .bind(project_id.0)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

        records.iter().map(Self::row_to_dependency).collect()
    }

    async fn create(&self, dependency: &Dependency) -> RepositoryResult<()> {
        let provenance_details = dependency
            .integrity
            .provenance_details
            .as_ref()
            .map(|d| serde_json::to_value(d))
            .transpose()
            .map_err(|e| RepositoryError::Serialization(e.to_string()))?;

        sqlx::query(
            r#"
            INSERT INTO dependencies (
                id, project_id, tenant_id, package_name, ecosystem, version_constraint,
                resolved_version, is_direct, is_dev_dependency, depth, parent_id,
                hash_sha256, hash_sha512, signature_status, provenance_status,
                provenance_details, first_seen_at, last_verified_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18)
            "#,
        )
        .bind(dependency.id.0)
        .bind(dependency.project_id.0)
        .bind(dependency.tenant_id.0)
        .bind(&dependency.package_name)
        .bind(dependency.ecosystem.to_string().to_lowercase())
        .bind(if dependency.version_constraint.is_empty() {
            None
        } else {
            Some(&dependency.version_constraint)
        })
        .bind(dependency.resolved_version.to_string())
        .bind(dependency.is_direct)
        .bind(dependency.is_dev_dependency)
        .bind(dependency.depth as i32)
        .bind(dependency.parent_id.map(|p| p.0))
        .bind(&dependency.integrity.hash_sha256)
        .bind(&dependency.integrity.hash_sha512)
        .bind(Self::signature_status_to_str(dependency.integrity.signature_status))
        .bind(Self::provenance_status_to_str(dependency.integrity.provenance_status))
        .bind(provenance_details)
        .bind(dependency.first_seen_at)
        .bind(dependency.last_verified_at)
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

    async fn create_batch(&self, dependencies: &[Dependency]) -> RepositoryResult<()> {
        if dependencies.is_empty() {
            return Ok(());
        }

        // Use a transaction for batch insert
        let mut tx = self.pool.begin().await
            .map_err(|e| RepositoryError::Database(e.to_string()))?;

        for dependency in dependencies {
            let provenance_details = dependency
                .integrity
                .provenance_details
                .as_ref()
                .map(|d| serde_json::to_value(d))
                .transpose()
                .map_err(|e| RepositoryError::Serialization(e.to_string()))?;

            sqlx::query(
                r#"
                INSERT INTO dependencies (
                    id, project_id, tenant_id, package_name, ecosystem, version_constraint,
                    resolved_version, is_direct, is_dev_dependency, depth, parent_id,
                    hash_sha256, hash_sha512, signature_status, provenance_status,
                    provenance_details, first_seen_at, last_verified_at
                ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18)
                ON CONFLICT (project_id, package_name, ecosystem, resolved_version)
                DO UPDATE SET
                    hash_sha256 = EXCLUDED.hash_sha256,
                    hash_sha512 = EXCLUDED.hash_sha512,
                    signature_status = EXCLUDED.signature_status,
                    provenance_status = EXCLUDED.provenance_status,
                    provenance_details = EXCLUDED.provenance_details,
                    last_verified_at = EXCLUDED.last_verified_at
                "#,
            )
            .bind(dependency.id.0)
            .bind(dependency.project_id.0)
            .bind(dependency.tenant_id.0)
            .bind(&dependency.package_name)
            .bind(dependency.ecosystem.to_string().to_lowercase())
            .bind(if dependency.version_constraint.is_empty() {
                None
            } else {
                Some(&dependency.version_constraint)
            })
            .bind(dependency.resolved_version.to_string())
            .bind(dependency.is_direct)
            .bind(dependency.is_dev_dependency)
            .bind(dependency.depth as i32)
            .bind(dependency.parent_id.map(|p| p.0))
            .bind(&dependency.integrity.hash_sha256)
            .bind(&dependency.integrity.hash_sha512)
            .bind(Self::signature_status_to_str(dependency.integrity.signature_status))
            .bind(Self::provenance_status_to_str(dependency.integrity.provenance_status))
            .bind(provenance_details)
            .bind(dependency.first_seen_at)
            .bind(dependency.last_verified_at)
            .execute(&mut *tx)
            .await
            .map_err(|e| RepositoryError::Database(e.to_string()))?;
        }

        tx.commit().await
            .map_err(|e| RepositoryError::Database(e.to_string()))?;

        Ok(())
    }

    async fn update(&self, dependency: &Dependency) -> RepositoryResult<()> {
        let provenance_details = dependency
            .integrity
            .provenance_details
            .as_ref()
            .map(|d| serde_json::to_value(d))
            .transpose()
            .map_err(|e| RepositoryError::Serialization(e.to_string()))?;

        let result = sqlx::query(
            r#"
            UPDATE dependencies SET
                package_name = $2, ecosystem = $3, version_constraint = $4,
                resolved_version = $5, is_direct = $6, is_dev_dependency = $7,
                depth = $8, parent_id = $9, hash_sha256 = $10, hash_sha512 = $11,
                signature_status = $12, provenance_status = $13, provenance_details = $14,
                last_verified_at = $15
            WHERE id = $1
            "#,
        )
        .bind(dependency.id.0)
        .bind(&dependency.package_name)
        .bind(dependency.ecosystem.to_string().to_lowercase())
        .bind(if dependency.version_constraint.is_empty() {
            None
        } else {
            Some(&dependency.version_constraint)
        })
        .bind(dependency.resolved_version.to_string())
        .bind(dependency.is_direct)
        .bind(dependency.is_dev_dependency)
        .bind(dependency.depth as i32)
        .bind(dependency.parent_id.map(|p| p.0))
        .bind(&dependency.integrity.hash_sha256)
        .bind(&dependency.integrity.hash_sha512)
        .bind(Self::signature_status_to_str(dependency.integrity.signature_status))
        .bind(Self::provenance_status_to_str(dependency.integrity.provenance_status))
        .bind(provenance_details)
        .bind(dependency.last_verified_at)
        .execute(&self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(RepositoryError::NotFound);
        }

        Ok(())
    }

    async fn delete(&self, id: DependencyId) -> RepositoryResult<()> {
        let result = sqlx::query("DELETE FROM dependencies WHERE id = $1")
            .bind(id.0)
            .execute(&self.pool)
            .await
            .map_err(|e| RepositoryError::Database(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(RepositoryError::NotFound);
        }

        Ok(())
    }

    async fn delete_by_project(&self, project_id: ProjectId) -> RepositoryResult<u32> {
        let result = sqlx::query("DELETE FROM dependencies WHERE project_id = $1")
            .bind(project_id.0)
            .execute(&self.pool)
            .await
            .map_err(|e| RepositoryError::Database(e.to_string()))?;

        Ok(result.rows_affected() as u32)
    }

    async fn find_by_package(
        &self,
        project_id: ProjectId,
        ecosystem: PackageEcosystem,
        package_name: &str,
        version: &str,
    ) -> RepositoryResult<Option<Dependency>> {
        let record = sqlx::query(
            r#"
            SELECT id, project_id, tenant_id, package_name, ecosystem, version_constraint,
                   resolved_version, is_direct, is_dev_dependency, depth, parent_id,
                   hash_sha256, hash_sha512, signature_status, provenance_status,
                   provenance_details, first_seen_at, last_verified_at
            FROM dependencies
            WHERE project_id = $1 AND ecosystem = $2 AND package_name = $3 AND resolved_version = $4
            "#,
        )
        .bind(project_id.0)
        .bind(ecosystem.to_string().to_lowercase())
        .bind(package_name)
        .bind(version)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

        match record {
            Some(row) => Ok(Some(Self::row_to_dependency(&row)?)),
            None => Ok(None),
        }
    }
}
