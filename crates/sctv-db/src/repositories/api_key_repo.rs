//! API key repository implementation.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sctv_core::traits::{ApiKeyRepository, RepositoryError, RepositoryResult};
use sctv_core::{ApiKey, ApiKeyId, TenantId};
use sqlx::{PgPool, Row};

/// `PostgreSQL` implementation of the API key repository.
pub struct PgApiKeyRepository {
    pool: PgPool,
}

impl PgApiKeyRepository {
    #[must_use]
    pub const fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    // Result<> is intentional: callers use this fn pointer with .map(...).transpose()
    // and .collect::<Result<_, _>>(), so the signature must match RepositoryResult<T>.
    #[allow(clippy::unnecessary_wraps)]
    fn row_to_api_key(row: &sqlx::postgres::PgRow) -> RepositoryResult<ApiKey> {
        let id: uuid::Uuid = row.get("id");
        let tenant_id: uuid::Uuid = row.get("tenant_id");
        let name: String = row.get("name");
        let key_hash: String = row.get("key_hash");
        let scopes: Vec<String> = row.get("scopes");
        let created_at: DateTime<Utc> = row.get("created_at");
        let last_used_at: Option<DateTime<Utc>> = row.get("last_used_at");
        let expires_at: Option<DateTime<Utc>> = row.get("expires_at");
        let revoked_at: Option<DateTime<Utc>> = row.get("revoked_at");

        Ok(ApiKey {
            id: ApiKeyId(id),
            tenant_id: TenantId(tenant_id),
            name,
            key_hash,
            scopes,
            created_at,
            last_used_at,
            expires_at,
            revoked_at,
        })
    }
}

#[async_trait]
impl ApiKeyRepository for PgApiKeyRepository {
    async fn find_active_by_hash(&self, key_hash: &str) -> RepositoryResult<Option<ApiKey>> {
        let record = sqlx::query(
            r"
            SELECT id, tenant_id, name, key_hash, scopes, created_at,
                   last_used_at, expires_at, revoked_at
            FROM api_keys
            WHERE key_hash = $1
              AND revoked_at IS NULL
              AND (expires_at IS NULL OR expires_at > NOW())
            ",
        )
        .bind(key_hash)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

        record.as_ref().map(Self::row_to_api_key).transpose()
    }

    async fn find_by_id(&self, id: ApiKeyId) -> RepositoryResult<Option<ApiKey>> {
        let record = sqlx::query(
            r"
            SELECT id, tenant_id, name, key_hash, scopes, created_at,
                   last_used_at, expires_at, revoked_at
            FROM api_keys
            WHERE id = $1
            ",
        )
        .bind(id.0)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

        record.as_ref().map(Self::row_to_api_key).transpose()
    }

    async fn list_active_by_tenant(&self, tenant_id: TenantId) -> RepositoryResult<Vec<ApiKey>> {
        let records = sqlx::query(
            r"
            SELECT id, tenant_id, name, key_hash, scopes, created_at,
                   last_used_at, expires_at, revoked_at
            FROM api_keys
            WHERE tenant_id = $1 AND revoked_at IS NULL
            ORDER BY created_at DESC
            ",
        )
        .bind(tenant_id.0)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

        records.iter().map(Self::row_to_api_key).collect()
    }

    async fn create(&self, key: &ApiKey) -> RepositoryResult<()> {
        sqlx::query(
            r"
            INSERT INTO api_keys (
                id, tenant_id, name, key_hash, scopes,
                created_at, last_used_at, expires_at, revoked_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            ",
        )
        .bind(key.id.0)
        .bind(key.tenant_id.0)
        .bind(&key.name)
        .bind(&key.key_hash)
        .bind(&key.scopes)
        .bind(key.created_at)
        .bind(key.last_used_at)
        .bind(key.expires_at)
        .bind(key.revoked_at)
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

    async fn revoke(&self, id: ApiKeyId) -> RepositoryResult<()> {
        let result = sqlx::query(
            "UPDATE api_keys SET revoked_at = NOW() WHERE id = $1 AND revoked_at IS NULL",
        )
        .bind(id.0)
        .execute(&self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(RepositoryError::NotFound);
        }
        Ok(())
    }

    async fn touch_last_used(&self, id: ApiKeyId) -> RepositoryResult<()> {
        sqlx::query("UPDATE api_keys SET last_used_at = NOW() WHERE id = $1")
            .bind(id.0)
            .execute(&self.pool)
            .await
            .map_err(|e| RepositoryError::Database(e.to_string()))?;
        Ok(())
    }
}
