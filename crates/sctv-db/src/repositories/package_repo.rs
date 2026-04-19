//! Package repository implementation.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sctv_core::traits::{PackageRepository, RepositoryError, RepositoryResult};
use sctv_core::{Package, PackageEcosystem, PackageId};
use sqlx::{PgPool, Row};
use url::Url;

/// `PostgreSQL` implementation of the package repository.
pub struct PgPackageRepository {
    pool: PgPool,
}

impl PgPackageRepository {
    /// Creates a new package repository.
    #[must_use]
    pub const fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    fn row_to_package(row: &sqlx::postgres::PgRow) -> RepositoryResult<Package> {
        let id: uuid::Uuid = row.get("id");
        let ecosystem: String = row.get("ecosystem");
        let name: String = row.get("name");
        let normalized_name: String = row.get("normalized_name");
        let description: Option<String> = row.get("description");
        let homepage: Option<String> = row.get("homepage");
        let repository: Option<String> = row.get("repository");
        let popularity_rank: Option<i32> = row.get("popularity_rank");
        let is_popular: bool = row.get("is_popular");
        let maintainers: serde_json::Value = row.get("maintainers");
        let first_published: Option<DateTime<Utc>> = row.get("first_published");
        let last_updated: Option<DateTime<Utc>> = row.get("last_updated");
        let cached_at: DateTime<Utc> = row.get("cached_at");

        let ecosystem: PackageEcosystem = ecosystem
            .parse()
            .map_err(|_| RepositoryError::InvalidData(format!("Invalid ecosystem: {ecosystem}")))?;

        let homepage = homepage.and_then(|u| Url::parse(&u).ok());
        let repository = repository.and_then(|u| Url::parse(&u).ok());

        let maintainers: Vec<String> = serde_json::from_value(maintainers)
            .map_err(|e| RepositoryError::Serialization(e.to_string()))?;

        Ok(Package {
            id: PackageId(id),
            ecosystem,
            name,
            normalized_name,
            description,
            homepage,
            repository,
            popularity_rank: popularity_rank.map(|r| r as u32),
            is_popular,
            maintainers,
            first_published,
            last_updated,
            cached_at,
        })
    }
}

#[async_trait]
impl PackageRepository for PgPackageRepository {
    async fn find_by_id(&self, id: PackageId) -> RepositoryResult<Option<Package>> {
        let record = sqlx::query(
            r"
            SELECT id, ecosystem, name, normalized_name, description, homepage, repository,
                   popularity_rank, is_popular, maintainers, first_published, last_updated, cached_at
            FROM packages WHERE id = $1
            ",
        )
        .bind(id.0)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

        match record {
            Some(row) => Ok(Some(Self::row_to_package(&row)?)),
            None => Ok(None),
        }
    }

    async fn find_by_name(
        &self,
        ecosystem: PackageEcosystem,
        name: &str,
    ) -> RepositoryResult<Option<Package>> {
        let record = sqlx::query(
            r"
            SELECT id, ecosystem, name, normalized_name, description, homepage, repository,
                   popularity_rank, is_popular, maintainers, first_published, last_updated, cached_at
            FROM packages
            WHERE ecosystem = $1 AND name = $2
            ",
        )
        .bind(ecosystem.to_string().to_lowercase())
        .bind(name)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

        match record {
            Some(row) => Ok(Some(Self::row_to_package(&row)?)),
            None => Ok(None),
        }
    }

    async fn upsert(&self, package: &Package) -> RepositoryResult<()> {
        let maintainers = serde_json::to_value(&package.maintainers)
            .map_err(|e| RepositoryError::Serialization(e.to_string()))?;

        sqlx::query(
            r"
            INSERT INTO packages (
                id, ecosystem, name, normalized_name, description, homepage, repository,
                popularity_rank, is_popular, maintainers, first_published, last_updated, cached_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
            ON CONFLICT (ecosystem, name)
            DO UPDATE SET
                normalized_name = EXCLUDED.normalized_name,
                description = EXCLUDED.description,
                homepage = EXCLUDED.homepage,
                repository = EXCLUDED.repository,
                popularity_rank = EXCLUDED.popularity_rank,
                is_popular = EXCLUDED.is_popular,
                maintainers = EXCLUDED.maintainers,
                first_published = COALESCE(packages.first_published, EXCLUDED.first_published),
                last_updated = EXCLUDED.last_updated,
                cached_at = EXCLUDED.cached_at
            ",
        )
        .bind(package.id.0)
        .bind(package.ecosystem.to_string().to_lowercase())
        .bind(&package.name)
        .bind(&package.normalized_name)
        .bind(&package.description)
        .bind(package.homepage.as_ref().map(url::Url::as_str))
        .bind(package.repository.as_ref().map(url::Url::as_str))
        .bind(package.popularity_rank.map(|r| r as i32))
        .bind(package.is_popular)
        .bind(maintainers)
        .bind(package.first_published)
        .bind(package.last_updated)
        .bind(package.cached_at)
        .execute(&self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

        Ok(())
    }

    async fn find_popular(
        &self,
        ecosystem: PackageEcosystem,
        limit: u32,
    ) -> RepositoryResult<Vec<Package>> {
        let records = sqlx::query(
            r"
            SELECT id, ecosystem, name, normalized_name, description, homepage, repository,
                   popularity_rank, is_popular, maintainers, first_published, last_updated, cached_at
            FROM packages
            WHERE ecosystem = $1 AND is_popular = true
            ORDER BY popularity_rank ASC NULLS LAST
            LIMIT $2
            ",
        )
        .bind(ecosystem.to_string().to_lowercase())
        .bind(i64::from(limit))
        .fetch_all(&self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

        records.iter().map(Self::row_to_package).collect()
    }

    async fn search_by_name(
        &self,
        ecosystem: PackageEcosystem,
        prefix: &str,
        limit: u32,
    ) -> RepositoryResult<Vec<Package>> {
        // Use the pg_trgm extension for fuzzy matching
        let records = sqlx::query(
            r"
            SELECT id, ecosystem, name, normalized_name, description, homepage, repository,
                   popularity_rank, is_popular, maintainers, first_published, last_updated, cached_at
            FROM packages
            WHERE ecosystem = $1 AND (
                name ILIKE $2 || '%'
                OR normalized_name ILIKE $2 || '%'
                OR name % $2
            )
            ORDER BY
                CASE WHEN name ILIKE $2 || '%' THEN 0 ELSE 1 END,
                similarity(name, $2) DESC,
                popularity_rank ASC NULLS LAST
            LIMIT $3
            ",
        )
        .bind(ecosystem.to_string().to_lowercase())
        .bind(prefix)
        .bind(i64::from(limit))
        .fetch_all(&self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

        records.iter().map(Self::row_to_package).collect()
    }
}

/// Repository for package version operations.
pub struct PgPackageVersionRepository {
    pool: PgPool,
}

impl PgPackageVersionRepository {
    /// Creates a new package version repository.
    #[must_use]
    pub const fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Finds a specific version of a package.
    pub async fn find_version(
        &self,
        package_id: PackageId,
        version: &str,
    ) -> RepositoryResult<Option<sctv_core::PackageVersion>> {
        let record = sqlx::query(
            r"
            SELECT package_id, version, published_at, yanked, deprecated, deprecation_message,
                   checksums, download_url, size_bytes, attestations, dependencies, cached_at
            FROM package_versions
            WHERE package_id = $1 AND version = $2
            ",
        )
        .bind(package_id.0)
        .bind(version)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

        match record {
            Some(row) => {
                let package_id: uuid::Uuid = row.get("package_id");
                let version_str: String = row.get("version");
                let published_at: Option<DateTime<Utc>> = row.get("published_at");
                let yanked: bool = row.get("yanked");
                let deprecated: bool = row.get("deprecated");
                let deprecation_message: Option<String> = row.get("deprecation_message");
                let checksums: serde_json::Value = row.get("checksums");
                let download_url: Option<String> = row.get("download_url");
                let size_bytes: Option<i64> = row.get("size_bytes");
                let attestations: serde_json::Value = row.get("attestations");
                let dependencies: serde_json::Value = row.get("dependencies");
                let cached_at: DateTime<Utc> = row.get("cached_at");

                let version = semver::Version::parse(&version_str)
                    .map_err(|e| RepositoryError::InvalidData(format!("Invalid version: {e}")))?;

                let checksums: sctv_core::PackageChecksums = serde_json::from_value(checksums)
                    .map_err(|e| RepositoryError::Serialization(e.to_string()))?;

                let download_url = download_url.and_then(|u| Url::parse(&u).ok());

                let attestations: Vec<sctv_core::Attestation> =
                    serde_json::from_value(attestations)
                        .map_err(|e| RepositoryError::Serialization(e.to_string()))?;

                let dependencies: Vec<sctv_core::PackageDependency> =
                    serde_json::from_value(dependencies)
                        .map_err(|e| RepositoryError::Serialization(e.to_string()))?;

                Ok(Some(sctv_core::PackageVersion {
                    package_id: PackageId(package_id),
                    version,
                    published_at,
                    yanked,
                    deprecated,
                    deprecation_message,
                    checksums,
                    download_url,
                    size_bytes: size_bytes.map(|s| s as u64),
                    attestations,
                    dependencies,
                    cached_at,
                }))
            }
            None => Ok(None),
        }
    }

    /// Upserts a package version.
    pub async fn upsert(&self, version: &sctv_core::PackageVersion) -> RepositoryResult<()> {
        let checksums = serde_json::to_value(&version.checksums)
            .map_err(|e| RepositoryError::Serialization(e.to_string()))?;

        let attestations = serde_json::to_value(&version.attestations)
            .map_err(|e| RepositoryError::Serialization(e.to_string()))?;

        let dependencies = serde_json::to_value(&version.dependencies)
            .map_err(|e| RepositoryError::Serialization(e.to_string()))?;

        sqlx::query(
            r"
            INSERT INTO package_versions (
                package_id, version, published_at, yanked, deprecated, deprecation_message,
                checksums, download_url, size_bytes, attestations, dependencies, cached_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            ON CONFLICT (package_id, version)
            DO UPDATE SET
                published_at = COALESCE(package_versions.published_at, EXCLUDED.published_at),
                yanked = EXCLUDED.yanked,
                deprecated = EXCLUDED.deprecated,
                deprecation_message = EXCLUDED.deprecation_message,
                checksums = EXCLUDED.checksums,
                download_url = EXCLUDED.download_url,
                size_bytes = EXCLUDED.size_bytes,
                attestations = EXCLUDED.attestations,
                dependencies = EXCLUDED.dependencies,
                cached_at = EXCLUDED.cached_at
            ",
        )
        .bind(version.package_id.0)
        .bind(version.version.to_string())
        .bind(version.published_at)
        .bind(version.yanked)
        .bind(version.deprecated)
        .bind(&version.deprecation_message)
        .bind(checksums)
        .bind(version.download_url.as_ref().map(url::Url::as_str))
        .bind(version.size_bytes.map(|s| s as i64))
        .bind(attestations)
        .bind(dependencies)
        .bind(version.cached_at)
        .execute(&self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

        Ok(())
    }

    /// Lists all versions for a package.
    pub async fn list_versions(
        &self,
        package_id: PackageId,
    ) -> RepositoryResult<Vec<sctv_core::PackageVersion>> {
        let records = sqlx::query(
            r"
            SELECT package_id, version, published_at, yanked, deprecated, deprecation_message,
                   checksums, download_url, size_bytes, attestations, dependencies, cached_at
            FROM package_versions
            WHERE package_id = $1
            ORDER BY published_at DESC NULLS LAST
            ",
        )
        .bind(package_id.0)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

        let mut versions = Vec::new();
        for row in records {
            let package_id: uuid::Uuid = row.get("package_id");
            let version_str: String = row.get("version");
            let published_at: Option<DateTime<Utc>> = row.get("published_at");
            let yanked: bool = row.get("yanked");
            let deprecated: bool = row.get("deprecated");
            let deprecation_message: Option<String> = row.get("deprecation_message");
            let checksums: serde_json::Value = row.get("checksums");
            let download_url: Option<String> = row.get("download_url");
            let size_bytes: Option<i64> = row.get("size_bytes");
            let attestations: serde_json::Value = row.get("attestations");
            let dependencies: serde_json::Value = row.get("dependencies");
            let cached_at: DateTime<Utc> = row.get("cached_at");

            if let Ok(version) = semver::Version::parse(&version_str) {
                let checksums: sctv_core::PackageChecksums = serde_json::from_value(checksums)
                    .map_err(|e| RepositoryError::Serialization(e.to_string()))?;

                let download_url = download_url.and_then(|u| Url::parse(&u).ok());

                let attestations: Vec<sctv_core::Attestation> =
                    serde_json::from_value(attestations)
                        .map_err(|e| RepositoryError::Serialization(e.to_string()))?;

                let dependencies: Vec<sctv_core::PackageDependency> =
                    serde_json::from_value(dependencies)
                        .map_err(|e| RepositoryError::Serialization(e.to_string()))?;

                versions.push(sctv_core::PackageVersion {
                    package_id: PackageId(package_id),
                    version,
                    published_at,
                    yanked,
                    deprecated,
                    deprecation_message,
                    checksums,
                    download_url,
                    size_bytes: size_bytes.map(|s| s as u64),
                    attestations,
                    dependencies,
                    cached_at,
                });
            }
        }

        Ok(versions)
    }
}
