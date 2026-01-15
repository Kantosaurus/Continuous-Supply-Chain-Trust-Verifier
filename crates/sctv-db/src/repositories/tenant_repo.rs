//! Tenant repository implementation.

use async_trait::async_trait;
use sctv_core::traits::{RepositoryError, RepositoryResult, TenantRepository};
use sctv_core::{Tenant, TenantId};
use sqlx::{PgPool, Row};

/// PostgreSQL implementation of the tenant repository.
pub struct PgTenantRepository {
    pool: PgPool,
}

impl PgTenantRepository {
    /// Creates a new tenant repository.
    #[must_use]
    pub const fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl TenantRepository for PgTenantRepository {
    async fn find_by_id(&self, id: TenantId) -> RepositoryResult<Option<Tenant>> {
        let record = sqlx::query(
            "SELECT id, name, slug, plan, settings, created_at, updated_at FROM tenants WHERE id = $1"
        )
        .bind(id.0)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

        match record {
            Some(row) => {
                let id: uuid::Uuid = row.get("id");
                let name: String = row.get("name");
                let slug: String = row.get("slug");
                let plan: serde_json::Value = row.get("plan");
                let settings: serde_json::Value = row.get("settings");
                let created_at = row.get("created_at");
                let updated_at = row.get("updated_at");

                Ok(Some(Tenant {
                    id: TenantId(id),
                    name,
                    slug,
                    plan: serde_json::from_value(plan)
                        .map_err(|e| RepositoryError::Serialization(e.to_string()))?,
                    settings: serde_json::from_value(settings)
                        .map_err(|e| RepositoryError::Serialization(e.to_string()))?,
                    created_at,
                    updated_at,
                }))
            }
            None => Ok(None),
        }
    }

    async fn find_by_slug(&self, slug: &str) -> RepositoryResult<Option<Tenant>> {
        let record = sqlx::query(
            "SELECT id, name, slug, plan, settings, created_at, updated_at FROM tenants WHERE slug = $1"
        )
        .bind(slug)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

        match record {
            Some(row) => {
                let id: uuid::Uuid = row.get("id");
                let name: String = row.get("name");
                let slug: String = row.get("slug");
                let plan: serde_json::Value = row.get("plan");
                let settings: serde_json::Value = row.get("settings");
                let created_at = row.get("created_at");
                let updated_at = row.get("updated_at");

                Ok(Some(Tenant {
                    id: TenantId(id),
                    name,
                    slug,
                    plan: serde_json::from_value(plan)
                        .map_err(|e| RepositoryError::Serialization(e.to_string()))?,
                    settings: serde_json::from_value(settings)
                        .map_err(|e| RepositoryError::Serialization(e.to_string()))?,
                    created_at,
                    updated_at,
                }))
            }
            None => Ok(None),
        }
    }

    async fn create(&self, tenant: &Tenant) -> RepositoryResult<()> {
        let plan = serde_json::to_value(&tenant.plan)
            .map_err(|e| RepositoryError::Serialization(e.to_string()))?;
        let settings = serde_json::to_value(&tenant.settings)
            .map_err(|e| RepositoryError::Serialization(e.to_string()))?;

        sqlx::query(
            "INSERT INTO tenants (id, name, slug, plan, settings, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7)"
        )
        .bind(tenant.id.0)
        .bind(&tenant.name)
        .bind(&tenant.slug)
        .bind(plan)
        .bind(settings)
        .bind(tenant.created_at)
        .bind(tenant.updated_at)
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

    async fn update(&self, tenant: &Tenant) -> RepositoryResult<()> {
        let plan = serde_json::to_value(&tenant.plan)
            .map_err(|e| RepositoryError::Serialization(e.to_string()))?;
        let settings = serde_json::to_value(&tenant.settings)
            .map_err(|e| RepositoryError::Serialization(e.to_string()))?;

        let result = sqlx::query(
            "UPDATE tenants SET name = $2, slug = $3, plan = $4, settings = $5, updated_at = NOW() WHERE id = $1"
        )
        .bind(tenant.id.0)
        .bind(&tenant.name)
        .bind(&tenant.slug)
        .bind(plan)
        .bind(settings)
        .execute(&self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(RepositoryError::NotFound);
        }

        Ok(())
    }

    async fn delete(&self, id: TenantId) -> RepositoryResult<()> {
        let result = sqlx::query("DELETE FROM tenants WHERE id = $1")
            .bind(id.0)
            .execute(&self.pool)
            .await
            .map_err(|e| RepositoryError::Database(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(RepositoryError::NotFound);
        }

        Ok(())
    }

    async fn list(&self, limit: u32, offset: u32) -> RepositoryResult<Vec<Tenant>> {
        let records = sqlx::query(
            "SELECT id, name, slug, plan, settings, created_at, updated_at FROM tenants ORDER BY created_at DESC LIMIT $1 OFFSET $2"
        )
        .bind(i64::from(limit))
        .bind(i64::from(offset))
        .fetch_all(&self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

        records
            .into_iter()
            .map(|row| {
                let id: uuid::Uuid = row.get("id");
                let name: String = row.get("name");
                let slug: String = row.get("slug");
                let plan: serde_json::Value = row.get("plan");
                let settings: serde_json::Value = row.get("settings");
                let created_at = row.get("created_at");
                let updated_at = row.get("updated_at");

                Ok(Tenant {
                    id: TenantId(id),
                    name,
                    slug,
                    plan: serde_json::from_value(plan)
                        .map_err(|e| RepositoryError::Serialization(e.to_string()))?,
                    settings: serde_json::from_value(settings)
                        .map_err(|e| RepositoryError::Serialization(e.to_string()))?,
                    created_at,
                    updated_at,
                })
            })
            .collect()
    }
}
