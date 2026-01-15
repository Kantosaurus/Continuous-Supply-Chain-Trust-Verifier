//! Audit log repository implementation.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sctv_core::traits::{AuditLogRepository, RepositoryError, RepositoryResult};
use sctv_core::{AuditAction, AuditLog, AuditLogFilter, AuditLogId, ResourceType, TenantId, UserId};
use sqlx::{PgPool, Row};
use std::net::IpAddr;

/// PostgreSQL implementation of the audit log repository.
pub struct PgAuditLogRepository {
    pool: PgPool,
}

impl PgAuditLogRepository {
    /// Creates a new audit log repository.
    #[must_use]
    pub const fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    fn row_to_audit_log(row: &sqlx::postgres::PgRow) -> RepositoryResult<AuditLog> {
        let id: uuid::Uuid = row.get("id");
        let tenant_id: uuid::Uuid = row.get("tenant_id");
        let user_id: Option<uuid::Uuid> = row.get("user_id");
        let action_str: String = row.get("action");
        let resource_type_str: String = row.get("resource_type");
        let resource_id: Option<uuid::Uuid> = row.get("resource_id");
        let details: serde_json::Value = row.get("details");
        let ip_address_str: Option<String> = row
            .try_get::<Option<String>, _>("ip_address")
            .ok()
            .flatten();
        let user_agent: Option<String> = row.get("user_agent");
        let created_at: DateTime<Utc> = row.get("created_at");

        let action = action_str
            .parse::<AuditAction>()
            .map_err(|e| RepositoryError::InvalidData(e))?;

        let resource_type = resource_type_str
            .parse::<ResourceType>()
            .map_err(|e| RepositoryError::InvalidData(e))?;

        let ip_address: Option<IpAddr> = ip_address_str.and_then(|s| s.parse().ok());

        Ok(AuditLog {
            id: AuditLogId(id),
            tenant_id: TenantId(tenant_id),
            user_id: user_id.map(UserId),
            action,
            resource_type,
            resource_id,
            details,
            ip_address,
            user_agent,
            created_at,
        })
    }
}

#[async_trait]
impl AuditLogRepository for PgAuditLogRepository {
    async fn find_by_id(&self, id: AuditLogId) -> RepositoryResult<Option<AuditLog>> {
        let record = sqlx::query(
            r#"
            SELECT id, tenant_id, user_id, action, resource_type, resource_id,
                   details, ip_address::text, user_agent, created_at
            FROM audit_logs WHERE id = $1
            "#,
        )
        .bind(id.0)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

        match record {
            Some(row) => Ok(Some(Self::row_to_audit_log(&row)?)),
            None => Ok(None),
        }
    }

    async fn find_by_tenant(
        &self,
        tenant_id: TenantId,
        filter: AuditLogFilter,
        limit: u32,
        offset: u32,
    ) -> RepositoryResult<Vec<AuditLog>> {
        let mut query = String::from(
            r#"
            SELECT id, tenant_id, user_id, action, resource_type, resource_id,
                   details, ip_address::text, user_agent, created_at
            FROM audit_logs
            WHERE tenant_id = $1
            "#,
        );

        let mut param_count = 1;

        if filter.user_id.is_some() {
            param_count += 1;
            query.push_str(&format!(" AND user_id = ${}", param_count));
        }

        if filter.action.is_some() {
            param_count += 1;
            query.push_str(&format!(" AND action = ANY(${})", param_count));
        }

        if filter.resource_type.is_some() {
            param_count += 1;
            query.push_str(&format!(" AND resource_type = ${}", param_count));
        }

        if filter.resource_id.is_some() {
            param_count += 1;
            query.push_str(&format!(" AND resource_id = ${}", param_count));
        }

        if filter.from_date.is_some() {
            param_count += 1;
            query.push_str(&format!(" AND created_at >= ${}", param_count));
        }

        if filter.to_date.is_some() {
            param_count += 1;
            query.push_str(&format!(" AND created_at <= ${}", param_count));
        }

        query.push_str(&format!(
            " ORDER BY created_at DESC LIMIT ${} OFFSET ${}",
            param_count + 1,
            param_count + 2
        ));

        let mut query_builder = sqlx::query(&query).bind(tenant_id.0);

        if let Some(user_id) = filter.user_id {
            query_builder = query_builder.bind(user_id.0);
        }

        if let Some(ref actions) = filter.action {
            let action_strs: Vec<String> = actions.iter().map(|a| a.to_string()).collect();
            query_builder = query_builder.bind(action_strs);
        }

        if let Some(ref resource_type) = filter.resource_type {
            query_builder = query_builder.bind(resource_type.to_string());
        }

        if let Some(resource_id) = filter.resource_id {
            query_builder = query_builder.bind(resource_id);
        }

        if let Some(from_date) = filter.from_date {
            query_builder = query_builder.bind(from_date);
        }

        if let Some(to_date) = filter.to_date {
            query_builder = query_builder.bind(to_date);
        }

        let records = query_builder
            .bind(limit as i64)
            .bind(offset as i64)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| RepositoryError::Database(e.to_string()))?;

        records.iter().map(Self::row_to_audit_log).collect()
    }

    async fn create(&self, audit_log: &AuditLog) -> RepositoryResult<()> {
        sqlx::query(
            r#"
            INSERT INTO audit_logs (
                id, tenant_id, user_id, action, resource_type, resource_id,
                details, ip_address, user_agent, created_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8::inet, $9, $10)
            "#,
        )
        .bind(audit_log.id.0)
        .bind(audit_log.tenant_id.0)
        .bind(audit_log.user_id.map(|u| u.0))
        .bind(audit_log.action.to_string())
        .bind(audit_log.resource_type.to_string())
        .bind(audit_log.resource_id)
        .bind(&audit_log.details)
        .bind(audit_log.ip_address.map(|ip| ip.to_string()))
        .bind(&audit_log.user_agent)
        .bind(audit_log.created_at)
        .execute(&self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

        Ok(())
    }

    async fn cleanup_old_logs(&self, older_than_days: u32) -> RepositoryResult<u32> {
        let result = sqlx::query(
            r#"
            DELETE FROM audit_logs
            WHERE created_at < NOW() - INTERVAL '1 day' * $1
            "#,
        )
        .bind(older_than_days as i64)
        .execute(&self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

        Ok(result.rows_affected() as u32)
    }

    async fn count_by_tenant(&self, tenant_id: TenantId) -> RepositoryResult<u32> {
        let record = sqlx::query("SELECT COUNT(*) as count FROM audit_logs WHERE tenant_id = $1")
            .bind(tenant_id.0)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| RepositoryError::Database(e.to_string()))?;

        let count: i64 = record.get("count");
        Ok(count as u32)
    }
}
