//! Policy repository implementation.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sctv_core::traits::{PolicyRepository, RepositoryError, RepositoryResult};
use sctv_core::{Policy, PolicyId, PolicyRule, SeverityOverride, TenantId};
use sqlx::{PgPool, Row};

/// `PostgreSQL` implementation of the policy repository.
pub struct PgPolicyRepository {
    pool: PgPool,
}

impl PgPolicyRepository {
    /// Creates a new policy repository.
    #[must_use]
    pub const fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    fn row_to_policy(row: &sqlx::postgres::PgRow) -> RepositoryResult<Policy> {
        let id: uuid::Uuid = row.get("id");
        let tenant_id: uuid::Uuid = row.get("tenant_id");
        let name: String = row.get("name");
        let description: Option<String> = row.get("description");
        let rules: serde_json::Value = row.get("rules");
        let severity_overrides: Option<serde_json::Value> = row.get("severity_overrides");
        let is_default: bool = row.get("is_default");
        let enabled: bool = row.get("enabled");
        let created_at: DateTime<Utc> = row.get("created_at");
        let updated_at: DateTime<Utc> = row.get("updated_at");

        let rules: Vec<PolicyRule> = serde_json::from_value(rules)
            .map_err(|e| RepositoryError::Serialization(e.to_string()))?;

        let severity_overrides: Vec<SeverityOverride> = match severity_overrides {
            Some(v) => serde_json::from_value(v)
                .map_err(|e| RepositoryError::Serialization(e.to_string()))?,
            None => Vec::new(),
        };

        Ok(Policy {
            id: PolicyId(id),
            tenant_id: TenantId(tenant_id),
            name,
            description,
            rules,
            severity_overrides,
            is_default,
            enabled,
            created_at,
            updated_at,
        })
    }
}

#[async_trait]
impl PolicyRepository for PgPolicyRepository {
    async fn find_by_id(&self, id: PolicyId) -> RepositoryResult<Option<Policy>> {
        let record = sqlx::query(
            r"
            SELECT id, tenant_id, name, description, rules, severity_overrides,
                   is_default, enabled, created_at, updated_at
            FROM policies WHERE id = $1
            ",
        )
        .bind(id.0)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

        match record {
            Some(row) => Ok(Some(Self::row_to_policy(&row)?)),
            None => Ok(None),
        }
    }

    async fn find_by_tenant(&self, tenant_id: TenantId) -> RepositoryResult<Vec<Policy>> {
        let records = sqlx::query(
            r"
            SELECT id, tenant_id, name, description, rules, severity_overrides,
                   is_default, enabled, created_at, updated_at
            FROM policies
            WHERE tenant_id = $1
            ORDER BY is_default DESC, name ASC
            ",
        )
        .bind(tenant_id.0)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

        records.iter().map(Self::row_to_policy).collect()
    }

    async fn find_default(&self, tenant_id: TenantId) -> RepositoryResult<Option<Policy>> {
        let record = sqlx::query(
            r"
            SELECT id, tenant_id, name, description, rules, severity_overrides,
                   is_default, enabled, created_at, updated_at
            FROM policies
            WHERE tenant_id = $1 AND is_default = true
            LIMIT 1
            ",
        )
        .bind(tenant_id.0)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

        match record {
            Some(row) => Ok(Some(Self::row_to_policy(&row)?)),
            None => Ok(None),
        }
    }

    async fn create(&self, policy: &Policy) -> RepositoryResult<()> {
        let rules = serde_json::to_value(&policy.rules)
            .map_err(|e| RepositoryError::Serialization(e.to_string()))?;

        let severity_overrides = serde_json::to_value(&policy.severity_overrides)
            .map_err(|e| RepositoryError::Serialization(e.to_string()))?;

        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| RepositoryError::Database(e.to_string()))?;

        if policy.is_default {
            sqlx::query(
                "UPDATE policies SET is_default = false WHERE tenant_id = $1 AND is_default = true",
            )
            .bind(policy.tenant_id.0)
            .execute(&mut *tx)
            .await
            .map_err(|e| RepositoryError::Database(e.to_string()))?;
        }

        sqlx::query(
            r"
            INSERT INTO policies (
                id, tenant_id, name, description, rules, severity_overrides,
                is_default, enabled, created_at, updated_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            ",
        )
        .bind(policy.id.0)
        .bind(policy.tenant_id.0)
        .bind(&policy.name)
        .bind(&policy.description)
        .bind(rules)
        .bind(severity_overrides)
        .bind(policy.is_default)
        .bind(policy.enabled)
        .bind(policy.created_at)
        .bind(policy.updated_at)
        .execute(&mut *tx)
        .await
        .map_err(|e| {
            if e.to_string().contains("duplicate key") {
                RepositoryError::AlreadyExists
            } else {
                RepositoryError::Database(e.to_string())
            }
        })?;

        tx.commit()
            .await
            .map_err(|e| RepositoryError::Database(e.to_string()))?;

        Ok(())
    }

    async fn update(&self, policy: &Policy) -> RepositoryResult<()> {
        let rules = serde_json::to_value(&policy.rules)
            .map_err(|e| RepositoryError::Serialization(e.to_string()))?;

        let severity_overrides = serde_json::to_value(&policy.severity_overrides)
            .map_err(|e| RepositoryError::Serialization(e.to_string()))?;

        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| RepositoryError::Database(e.to_string()))?;

        if policy.is_default {
            sqlx::query(
                "UPDATE policies SET is_default = false WHERE tenant_id = $1 AND is_default = true AND id != $2",
            )
            .bind(policy.tenant_id.0)
            .bind(policy.id.0)
            .execute(&mut *tx)
            .await
            .map_err(|e| RepositoryError::Database(e.to_string()))?;
        }

        let result = sqlx::query(
            r"
            UPDATE policies SET
                name = $2, description = $3, rules = $4, severity_overrides = $5,
                is_default = $6, enabled = $7, updated_at = NOW()
            WHERE id = $1
            ",
        )
        .bind(policy.id.0)
        .bind(&policy.name)
        .bind(&policy.description)
        .bind(rules)
        .bind(severity_overrides)
        .bind(policy.is_default)
        .bind(policy.enabled)
        .execute(&mut *tx)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(RepositoryError::NotFound);
        }

        tx.commit()
            .await
            .map_err(|e| RepositoryError::Database(e.to_string()))?;

        Ok(())
    }

    async fn delete(&self, id: PolicyId) -> RepositoryResult<()> {
        let result = sqlx::query("DELETE FROM policies WHERE id = $1")
            .bind(id.0)
            .execute(&self.pool)
            .await
            .map_err(|e| RepositoryError::Database(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(RepositoryError::NotFound);
        }

        Ok(())
    }

    async fn set_default(&self, tenant_id: TenantId, policy_id: PolicyId) -> RepositoryResult<()> {
        // Start a transaction
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| RepositoryError::Database(e.to_string()))?;

        // Unset current default
        sqlx::query(
            "UPDATE policies SET is_default = false WHERE tenant_id = $1 AND is_default = true",
        )
        .bind(tenant_id.0)
        .execute(&mut *tx)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

        // Set new default
        let result = sqlx::query(
            "UPDATE policies SET is_default = true, updated_at = NOW() WHERE id = $1 AND tenant_id = $2",
        )
        .bind(policy_id.0)
        .bind(tenant_id.0)
        .execute(&mut *tx)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(RepositoryError::NotFound);
        }

        tx.commit()
            .await
            .map_err(|e| RepositoryError::Database(e.to_string()))?;

        Ok(())
    }
}
