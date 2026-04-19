//! User repository implementation.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sctv_core::traits::{RepositoryError, RepositoryResult, UserRepository};
use sctv_core::{TenantId, User, UserId, UserRole};
use sqlx::{PgPool, Row};

/// PostgreSQL implementation of the user repository.
pub struct PgUserRepository {
    pool: PgPool,
}

impl PgUserRepository {
    /// Creates a new user repository.
    #[must_use]
    pub const fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    fn row_to_user(row: &sqlx::postgres::PgRow) -> RepositoryResult<User> {
        let id: uuid::Uuid = row.get("id");
        let tenant_id: uuid::Uuid = row.get("tenant_id");
        let email: String = row.get("email");
        let name: Option<String> = row.get("name");
        let role_str: String = row.get("role");
        let api_key_hash: Option<String> = row.get("api_key_hash");
        let last_login_at: Option<DateTime<Utc>> = row.get("last_login_at");
        let created_at: DateTime<Utc> = row.get("created_at");
        let updated_at: DateTime<Utc> = row.get("updated_at");

        let role = role_str.parse::<UserRole>().unwrap_or(UserRole::Member);

        Ok(User {
            id: UserId(id),
            tenant_id: TenantId(tenant_id),
            email,
            name,
            role,
            api_key_hash,
            last_login_at,
            created_at,
            updated_at,
        })
    }
}

#[async_trait]
impl UserRepository for PgUserRepository {
    async fn find_by_id(&self, id: UserId) -> RepositoryResult<Option<User>> {
        let record = sqlx::query(
            r#"
            SELECT id, tenant_id, email, name, role, api_key_hash,
                   last_login_at, created_at, updated_at
            FROM users WHERE id = $1
            "#,
        )
        .bind(id.0)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

        match record {
            Some(row) => Ok(Some(Self::row_to_user(&row)?)),
            None => Ok(None),
        }
    }

    async fn find_by_email(
        &self,
        tenant_id: TenantId,
        email: &str,
    ) -> RepositoryResult<Option<User>> {
        let record = sqlx::query(
            r#"
            SELECT id, tenant_id, email, name, role, api_key_hash,
                   last_login_at, created_at, updated_at
            FROM users WHERE tenant_id = $1 AND email = $2
            "#,
        )
        .bind(tenant_id.0)
        .bind(email)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

        match record {
            Some(row) => Ok(Some(Self::row_to_user(&row)?)),
            None => Ok(None),
        }
    }

    async fn find_by_api_key(&self, api_key_hash: &str) -> RepositoryResult<Option<User>> {
        let record = sqlx::query(
            r#"
            SELECT id, tenant_id, email, name, role, api_key_hash,
                   last_login_at, created_at, updated_at
            FROM users WHERE api_key_hash = $1
            "#,
        )
        .bind(api_key_hash)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

        match record {
            Some(row) => Ok(Some(Self::row_to_user(&row)?)),
            None => Ok(None),
        }
    }

    async fn find_by_tenant(&self, tenant_id: TenantId) -> RepositoryResult<Vec<User>> {
        let records = sqlx::query(
            r#"
            SELECT id, tenant_id, email, name, role, api_key_hash,
                   last_login_at, created_at, updated_at
            FROM users
            WHERE tenant_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(tenant_id.0)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

        records.iter().map(Self::row_to_user).collect()
    }

    async fn create(&self, user: &User) -> RepositoryResult<()> {
        sqlx::query(
            r#"
            INSERT INTO users (
                id, tenant_id, email, name, role, api_key_hash,
                last_login_at, created_at, updated_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            "#,
        )
        .bind(user.id.0)
        .bind(user.tenant_id.0)
        .bind(&user.email)
        .bind(&user.name)
        .bind(user.role.to_string())
        .bind(&user.api_key_hash)
        .bind(user.last_login_at)
        .bind(user.created_at)
        .bind(user.updated_at)
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

    async fn update(&self, user: &User) -> RepositoryResult<()> {
        let result = sqlx::query(
            r#"
            UPDATE users SET
                email = $2, name = $3, role = $4, api_key_hash = $5,
                last_login_at = $6, updated_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(user.id.0)
        .bind(&user.email)
        .bind(&user.name)
        .bind(user.role.to_string())
        .bind(&user.api_key_hash)
        .bind(user.last_login_at)
        .execute(&self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(RepositoryError::NotFound);
        }

        Ok(())
    }

    async fn delete(&self, id: UserId) -> RepositoryResult<()> {
        let result = sqlx::query("DELETE FROM users WHERE id = $1")
            .bind(id.0)
            .execute(&self.pool)
            .await
            .map_err(|e| RepositoryError::Database(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(RepositoryError::NotFound);
        }

        Ok(())
    }

    async fn update_last_login(&self, id: UserId) -> RepositoryResult<()> {
        let result =
            sqlx::query("UPDATE users SET last_login_at = NOW(), updated_at = NOW() WHERE id = $1")
                .bind(id.0)
                .execute(&self.pool)
                .await
                .map_err(|e| RepositoryError::Database(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(RepositoryError::NotFound);
        }

        Ok(())
    }

    async fn count_by_tenant(&self, tenant_id: TenantId) -> RepositoryResult<u32> {
        let record = sqlx::query("SELECT COUNT(*) as count FROM users WHERE tenant_id = $1")
            .bind(tenant_id.0)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| RepositoryError::Database(e.to_string()))?;

        let count: i64 = record.get("count");
        Ok(count as u32)
    }
}
