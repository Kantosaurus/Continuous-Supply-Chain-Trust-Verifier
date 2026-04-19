//! Alert repository implementation.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sctv_core::traits::{AlertFilter, AlertRepository, RepositoryError, RepositoryResult};
use sctv_core::{
    Alert, AlertId, AlertMetadata, AlertStatus, AlertType, DependencyId, ProjectId, Remediation,
    Severity, TenantId,
};
use sqlx::{PgPool, Row};
use std::collections::HashMap;

/// `PostgreSQL` implementation of the alert repository.
pub struct PgAlertRepository {
    pool: PgPool,
}

impl PgAlertRepository {
    /// Creates a new alert repository.
    #[must_use]
    pub const fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    fn row_to_alert(row: &sqlx::postgres::PgRow) -> RepositoryResult<Alert> {
        let id: uuid::Uuid = row.get("id");
        let tenant_id: uuid::Uuid = row.get("tenant_id");
        let project_id: uuid::Uuid = row.get("project_id");
        let dependency_id: Option<uuid::Uuid> = row.get("dependency_id");
        let alert_type_str: String = row.get("alert_type");
        let alert_details: serde_json::Value = row.get("alert_details");
        let severity_str: String = row.get("severity");
        let title: String = row.get("title");
        let description: Option<String> = row.get("description");
        let status_str: String = row.get("status");
        let remediation_json: Option<serde_json::Value> = row.get("remediation");
        let metadata_json: serde_json::Value = row.get("metadata");
        let created_at: DateTime<Utc> = row.get("created_at");
        let acknowledged_at: Option<DateTime<Utc>> = row.get("acknowledged_at");
        let acknowledged_by: Option<uuid::Uuid> = row.get("acknowledged_by");
        let resolved_at: Option<DateTime<Utc>> = row.get("resolved_at");
        let resolved_by: Option<uuid::Uuid> = row.get("resolved_by");

        let severity = match severity_str.as_str() {
            "critical" => Severity::Critical,
            "high" => Severity::High,
            "medium" => Severity::Medium,
            "low" => Severity::Low,
            _ => Severity::Info,
        };

        let status = match status_str.as_str() {
            "open" => AlertStatus::Open,
            "acknowledged" => AlertStatus::Acknowledged,
            "investigating" => AlertStatus::Investigating,
            "false_positive" => AlertStatus::FalsePositive,
            "resolved" => AlertStatus::Resolved,
            "suppressed" => AlertStatus::Suppressed,
            _ => AlertStatus::Open,
        };

        // Parse alert type from type string and details JSON
        let alert_type = Self::parse_alert_type(&alert_type_str, alert_details)?;

        let remediation: Option<Remediation> =
            remediation_json.and_then(|v| serde_json::from_value(v).ok());

        let metadata: AlertMetadata = serde_json::from_value(metadata_json)
            .map_err(|e| RepositoryError::Serialization(e.to_string()))?;

        Ok(Alert {
            id: AlertId(id),
            tenant_id: TenantId(tenant_id),
            project_id: ProjectId(project_id),
            dependency_id: dependency_id.map(DependencyId),
            alert_type,
            severity,
            title,
            description: description.unwrap_or_default(),
            status,
            remediation,
            metadata,
            created_at,
            acknowledged_at,
            acknowledged_by,
            resolved_at,
            resolved_by,
        })
    }

    fn parse_alert_type(type_str: &str, details: serde_json::Value) -> RepositoryResult<AlertType> {
        match type_str {
            "dependency_tampering" => {
                let details = serde_json::from_value(details)
                    .map_err(|e| RepositoryError::Serialization(e.to_string()))?;
                Ok(AlertType::DependencyTampering(details))
            }
            "downgrade_attack" => {
                let details = serde_json::from_value(details)
                    .map_err(|e| RepositoryError::Serialization(e.to_string()))?;
                Ok(AlertType::DowngradeAttack(details))
            }
            "typosquatting" => {
                let details = serde_json::from_value(details)
                    .map_err(|e| RepositoryError::Serialization(e.to_string()))?;
                Ok(AlertType::Typosquatting(details))
            }
            "provenance_failure" => {
                let details = serde_json::from_value(details)
                    .map_err(|e| RepositoryError::Serialization(e.to_string()))?;
                Ok(AlertType::ProvenanceFailure(details))
            }
            "policy_violation" => {
                let details = serde_json::from_value(details)
                    .map_err(|e| RepositoryError::Serialization(e.to_string()))?;
                Ok(AlertType::PolicyViolation(details))
            }
            "new_package" => {
                let details = serde_json::from_value(details)
                    .map_err(|e| RepositoryError::Serialization(e.to_string()))?;
                Ok(AlertType::NewPackage(details))
            }
            "suspicious_maintainer" => {
                let details = serde_json::from_value(details)
                    .map_err(|e| RepositoryError::Serialization(e.to_string()))?;
                Ok(AlertType::SuspiciousMaintainer(details))
            }
            _ => Err(RepositoryError::InvalidData(format!(
                "Unknown alert type: {type_str}"
            ))),
        }
    }

    const fn severity_to_str(severity: Severity) -> &'static str {
        match severity {
            Severity::Critical => "critical",
            Severity::High => "high",
            Severity::Medium => "medium",
            Severity::Low => "low",
            Severity::Info => "info",
        }
    }

    const fn status_to_str(status: AlertStatus) -> &'static str {
        match status {
            AlertStatus::Open => "open",
            AlertStatus::Acknowledged => "acknowledged",
            AlertStatus::Investigating => "investigating",
            AlertStatus::FalsePositive => "false_positive",
            AlertStatus::Resolved => "resolved",
            AlertStatus::Suppressed => "suppressed",
        }
    }

    fn alert_type_details(alert_type: &AlertType) -> RepositoryResult<serde_json::Value> {
        let value = match alert_type {
            AlertType::DependencyTampering(d) => serde_json::to_value(d),
            AlertType::DowngradeAttack(d) => serde_json::to_value(d),
            AlertType::Typosquatting(d) => serde_json::to_value(d),
            AlertType::ProvenanceFailure(d) => serde_json::to_value(d),
            AlertType::PolicyViolation(d) => serde_json::to_value(d),
            AlertType::NewPackage(d) => serde_json::to_value(d),
            AlertType::SuspiciousMaintainer(d) => serde_json::to_value(d),
        };
        value.map_err(|e| RepositoryError::Serialization(e.to_string()))
    }
}

#[async_trait]
impl AlertRepository for PgAlertRepository {
    async fn find_by_id(&self, id: AlertId) -> RepositoryResult<Option<Alert>> {
        let record = sqlx::query(
            r"
            SELECT id, tenant_id, project_id, dependency_id, alert_type, alert_details,
                   severity, title, description, status, remediation, metadata,
                   created_at, acknowledged_at, acknowledged_by, resolved_at, resolved_by
            FROM alerts WHERE id = $1
            ",
        )
        .bind(id.0)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

        match record {
            Some(row) => Ok(Some(Self::row_to_alert(&row)?)),
            None => Ok(None),
        }
    }

    async fn find_by_project(&self, project_id: ProjectId) -> RepositoryResult<Vec<Alert>> {
        let records = sqlx::query(
            r"
            SELECT id, tenant_id, project_id, dependency_id, alert_type, alert_details,
                   severity, title, description, status, remediation, metadata,
                   created_at, acknowledged_at, acknowledged_by, resolved_at, resolved_by
            FROM alerts
            WHERE project_id = $1
            ORDER BY created_at DESC
            ",
        )
        .bind(project_id.0)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

        records.iter().map(Self::row_to_alert).collect()
    }

    async fn find_with_filter(
        &self,
        tenant_id: TenantId,
        filter: AlertFilter,
        limit: u32,
        offset: u32,
    ) -> RepositoryResult<Vec<Alert>> {
        // Build dynamic query based on filters
        let mut query = String::from(
            r"
            SELECT id, tenant_id, project_id, dependency_id, alert_type, alert_details,
                   severity, title, description, status, remediation, metadata,
                   created_at, acknowledged_at, acknowledged_by, resolved_at, resolved_by
            FROM alerts
            WHERE tenant_id = $1
            ",
        );

        let mut param_count = 1;

        if filter.project_id.is_some() {
            param_count += 1;
            query.push_str(&format!(" AND project_id = ${param_count}"));
        }

        if filter.status.is_some() {
            param_count += 1;
            query.push_str(&format!(" AND status = ANY(${param_count})"));
        }

        if filter.severity.is_some() {
            param_count += 1;
            query.push_str(&format!(" AND severity = ANY(${param_count})"));
        }

        if filter.alert_type.is_some() {
            param_count += 1;
            query.push_str(&format!(" AND alert_type = ANY(${param_count})"));
        }

        query.push_str(&format!(
            " ORDER BY created_at DESC LIMIT ${} OFFSET ${}",
            param_count + 1,
            param_count + 2
        ));

        // Execute with appropriate bindings
        let mut query_builder = sqlx::query(&query).bind(tenant_id.0);

        if let Some(project_id) = filter.project_id {
            query_builder = query_builder.bind(project_id.0);
        }

        if let Some(ref statuses) = filter.status {
            let status_strs: Vec<&str> = statuses.iter().map(|s| Self::status_to_str(*s)).collect();
            query_builder = query_builder.bind(status_strs);
        }

        if let Some(ref severities) = filter.severity {
            let severity_strs: Vec<&str> = severities
                .iter()
                .map(|s| Self::severity_to_str(*s))
                .collect();
            query_builder = query_builder.bind(severity_strs);
        }

        if let Some(ref alert_types) = filter.alert_type {
            query_builder = query_builder.bind(alert_types);
        }

        let records = query_builder
            .bind(i64::from(limit))
            .bind(i64::from(offset))
            .fetch_all(&self.pool)
            .await
            .map_err(|e| RepositoryError::Database(e.to_string()))?;

        records.iter().map(Self::row_to_alert).collect()
    }

    async fn count_with_filter(
        &self,
        tenant_id: TenantId,
        filter: AlertFilter,
    ) -> RepositoryResult<u64> {
        let mut query = String::from("SELECT COUNT(*)::BIGINT FROM alerts WHERE tenant_id = $1");
        let mut param_count = 1;

        if filter.project_id.is_some() {
            param_count += 1;
            query.push_str(&format!(" AND project_id = ${param_count}"));
        }

        if filter.status.is_some() {
            param_count += 1;
            query.push_str(&format!(" AND status = ANY(${param_count})"));
        }

        if filter.severity.is_some() {
            param_count += 1;
            query.push_str(&format!(" AND severity = ANY(${param_count})"));
        }

        if filter.alert_type.is_some() {
            param_count += 1;
            query.push_str(&format!(" AND alert_type = ANY(${param_count})"));
        }

        let mut query_builder = sqlx::query_scalar::<_, i64>(&query).bind(tenant_id.0);

        if let Some(project_id) = filter.project_id {
            query_builder = query_builder.bind(project_id.0);
        }

        if let Some(ref statuses) = filter.status {
            let status_strs: Vec<&str> = statuses.iter().map(|s| Self::status_to_str(*s)).collect();
            query_builder = query_builder.bind(status_strs);
        }

        if let Some(ref severities) = filter.severity {
            let severity_strs: Vec<&str> = severities
                .iter()
                .map(|s| Self::severity_to_str(*s))
                .collect();
            query_builder = query_builder.bind(severity_strs);
        }

        if let Some(ref alert_types) = filter.alert_type {
            query_builder = query_builder.bind(alert_types);
        }

        let total: i64 = query_builder
            .fetch_one(&self.pool)
            .await
            .map_err(|e| RepositoryError::Database(e.to_string()))?;

        Ok(total.max(0) as u64)
    }

    async fn create(&self, alert: &Alert) -> RepositoryResult<()> {
        let alert_details = Self::alert_type_details(&alert.alert_type)?;

        let remediation = alert
            .remediation
            .as_ref()
            .map(serde_json::to_value)
            .transpose()
            .map_err(|e| RepositoryError::Serialization(e.to_string()))?;

        let metadata = serde_json::to_value(&alert.metadata)
            .map_err(|e| RepositoryError::Serialization(e.to_string()))?;

        sqlx::query(
            r"
            INSERT INTO alerts (
                id, tenant_id, project_id, dependency_id, alert_type, alert_details,
                severity, title, description, status, remediation, metadata,
                created_at, acknowledged_at, acknowledged_by, resolved_at, resolved_by
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17)
            ",
        )
        .bind(alert.id.0)
        .bind(alert.tenant_id.0)
        .bind(alert.project_id.0)
        .bind(alert.dependency_id.map(|d| d.0))
        .bind(alert.alert_type.type_name())
        .bind(alert_details)
        .bind(Self::severity_to_str(alert.severity))
        .bind(&alert.title)
        .bind(&alert.description)
        .bind(Self::status_to_str(alert.status))
        .bind(remediation)
        .bind(metadata)
        .bind(alert.created_at)
        .bind(alert.acknowledged_at)
        .bind(alert.acknowledged_by)
        .bind(alert.resolved_at)
        .bind(alert.resolved_by)
        .execute(&self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

        Ok(())
    }

    async fn update(&self, alert: &Alert) -> RepositoryResult<()> {
        let alert_details = Self::alert_type_details(&alert.alert_type)?;

        let remediation = alert
            .remediation
            .as_ref()
            .map(serde_json::to_value)
            .transpose()
            .map_err(|e| RepositoryError::Serialization(e.to_string()))?;

        let metadata = serde_json::to_value(&alert.metadata)
            .map_err(|e| RepositoryError::Serialization(e.to_string()))?;

        let result = sqlx::query(
            r"
            UPDATE alerts SET
                alert_type = $2, alert_details = $3, severity = $4, title = $5,
                description = $6, status = $7, remediation = $8, metadata = $9,
                acknowledged_at = $10, acknowledged_by = $11, resolved_at = $12, resolved_by = $13
            WHERE id = $1
            ",
        )
        .bind(alert.id.0)
        .bind(alert.alert_type.type_name())
        .bind(alert_details)
        .bind(Self::severity_to_str(alert.severity))
        .bind(&alert.title)
        .bind(&alert.description)
        .bind(Self::status_to_str(alert.status))
        .bind(remediation)
        .bind(metadata)
        .bind(alert.acknowledged_at)
        .bind(alert.acknowledged_by)
        .bind(alert.resolved_at)
        .bind(alert.resolved_by)
        .execute(&self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(RepositoryError::NotFound);
        }

        Ok(())
    }

    async fn count_open_by_project(&self, project_id: ProjectId) -> RepositoryResult<u32> {
        let record = sqlx::query(
            r"
            SELECT COUNT(*) as count FROM alerts
            WHERE project_id = $1 AND status IN ('open', 'acknowledged', 'investigating')
            ",
        )
        .bind(project_id.0)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

        let count: i64 = record.get("count");
        Ok(count as u32)
    }

    async fn count_by_severity(
        &self,
        project_id: ProjectId,
    ) -> RepositoryResult<HashMap<Severity, u32>> {
        let records = sqlx::query(
            r"
            SELECT severity, COUNT(*) as count FROM alerts
            WHERE project_id = $1 AND status IN ('open', 'acknowledged', 'investigating')
            GROUP BY severity
            ",
        )
        .bind(project_id.0)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

        let mut result = HashMap::new();
        for row in records {
            let severity_str: String = row.get("severity");
            let count: i64 = row.get("count");

            let severity = match severity_str.as_str() {
                "critical" => Severity::Critical,
                "high" => Severity::High,
                "medium" => Severity::Medium,
                "low" => Severity::Low,
                _ => Severity::Info,
            };

            result.insert(severity, count as u32);
        }

        Ok(result)
    }
}
