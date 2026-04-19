//! Job repository implementation.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sctv_core::traits::{JobFilter, JobRepository, RepositoryError, RepositoryResult};
use sctv_core::{Job, JobId, JobPriority, JobStatus, JobType, TenantId};
use sqlx::{PgPool, Row};
use std::collections::HashMap;

/// `PostgreSQL` implementation of the job repository.
pub struct PgJobRepository {
    pool: PgPool,
}

impl PgJobRepository {
    /// Creates a new job repository.
    #[must_use]
    pub const fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    fn row_to_job(row: &sqlx::postgres::PgRow) -> RepositoryResult<Job> {
        let id: uuid::Uuid = row.get("id");
        let tenant_id: Option<uuid::Uuid> = row.get("tenant_id");
        let job_type_str: String = row.get("job_type");
        let status_str: String = row.get("status");
        let priority: i32 = row.get("priority");
        let payload: serde_json::Value = row.get("payload");
        let result: Option<serde_json::Value> = row.get("result");
        let error_message: Option<String> = row.get("error_message");
        let attempts: i32 = row.get("attempts");
        let max_attempts: i32 = row.get("max_attempts");
        let scheduled_at: DateTime<Utc> = row.get("scheduled_at");
        let started_at: Option<DateTime<Utc>> = row.get("started_at");
        let completed_at: Option<DateTime<Utc>> = row.get("completed_at");
        let created_at: DateTime<Utc> = row.get("created_at");

        let status = status_str
            .parse::<JobStatus>()
            .unwrap_or(JobStatus::Pending);

        let job_type: JobType = serde_json::from_value(payload.clone()).map_err(|e| {
            RepositoryError::Serialization(format!(
                "Failed to deserialize job type '{job_type_str}': {e}"
            ))
        })?;

        Ok(Job {
            id: JobId(id),
            tenant_id: tenant_id.map(TenantId),
            job_type,
            status,
            priority: JobPriority::from(priority),
            payload,
            result,
            error_message,
            attempts: attempts as u32,
            max_attempts: max_attempts as u32,
            scheduled_at,
            started_at,
            completed_at,
            created_at,
        })
    }

    const fn status_to_str(status: JobStatus) -> &'static str {
        match status {
            JobStatus::Pending => "pending",
            JobStatus::Running => "running",
            JobStatus::Completed => "completed",
            JobStatus::Failed => "failed",
            JobStatus::Cancelled => "cancelled",
            JobStatus::Scheduled => "scheduled",
        }
    }
}

#[async_trait]
impl JobRepository for PgJobRepository {
    async fn find_by_id(&self, id: JobId) -> RepositoryResult<Option<Job>> {
        let record = sqlx::query(
            r"
            SELECT id, tenant_id, job_type, status, priority, payload, result,
                   error_message, attempts, max_attempts, scheduled_at,
                   started_at, completed_at, created_at
            FROM jobs WHERE id = $1
            ",
        )
        .bind(id.0)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

        match record {
            Some(row) => Ok(Some(Self::row_to_job(&row)?)),
            None => Ok(None),
        }
    }

    async fn find_by_tenant(
        &self,
        tenant_id: Option<TenantId>,
        filter: JobFilter,
        limit: u32,
        offset: u32,
    ) -> RepositoryResult<Vec<Job>> {
        let mut query = String::from(
            r"
            SELECT id, tenant_id, job_type, status, priority, payload, result,
                   error_message, attempts, max_attempts, scheduled_at,
                   started_at, completed_at, created_at
            FROM jobs
            WHERE 1=1
            ",
        );

        let mut param_count = 0;

        if tenant_id.is_some() {
            param_count += 1;
            query.push_str(&format!(" AND tenant_id = ${param_count}"));
        }

        if filter.status.is_some() {
            param_count += 1;
            query.push_str(&format!(" AND status = ANY(${param_count})"));
        }

        if filter.job_type.is_some() {
            param_count += 1;
            query.push_str(&format!(" AND job_type = ANY(${param_count})"));
        }

        query.push_str(&format!(
            " ORDER BY priority DESC, scheduled_at ASC LIMIT ${} OFFSET ${}",
            param_count + 1,
            param_count + 2
        ));

        let mut query_builder = sqlx::query(&query);

        if let Some(tid) = tenant_id {
            query_builder = query_builder.bind(tid.0);
        }

        if let Some(ref statuses) = filter.status {
            let status_strs: Vec<&str> = statuses.iter().map(|s| Self::status_to_str(*s)).collect();
            query_builder = query_builder.bind(status_strs);
        }

        if let Some(ref job_types) = filter.job_type {
            query_builder = query_builder.bind(job_types);
        }

        let records = query_builder
            .bind(i64::from(limit))
            .bind(i64::from(offset))
            .fetch_all(&self.pool)
            .await
            .map_err(|e| RepositoryError::Database(e.to_string()))?;

        records.iter().map(Self::row_to_job).collect()
    }

    async fn create(&self, job: &Job) -> RepositoryResult<()> {
        let payload = serde_json::to_value(&job.job_type)
            .map_err(|e| RepositoryError::Serialization(e.to_string()))?;

        sqlx::query(
            r"
            INSERT INTO jobs (
                id, tenant_id, job_type, status, priority, payload, result,
                error_message, attempts, max_attempts, scheduled_at,
                started_at, completed_at, created_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
            ",
        )
        .bind(job.id.0)
        .bind(job.tenant_id.map(|t| t.0))
        .bind(job.job_type.type_name())
        .bind(Self::status_to_str(job.status))
        .bind(i32::from(job.priority))
        .bind(payload)
        .bind(&job.result)
        .bind(&job.error_message)
        .bind(job.attempts as i32)
        .bind(job.max_attempts as i32)
        .bind(job.scheduled_at)
        .bind(job.started_at)
        .bind(job.completed_at)
        .bind(job.created_at)
        .execute(&self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

        Ok(())
    }

    async fn update(&self, job: &Job) -> RepositoryResult<()> {
        let payload = serde_json::to_value(&job.job_type)
            .map_err(|e| RepositoryError::Serialization(e.to_string()))?;

        let result = sqlx::query(
            r"
            UPDATE jobs SET
                status = $2, priority = $3, payload = $4, result = $5,
                error_message = $6, attempts = $7, scheduled_at = $8,
                started_at = $9, completed_at = $10
            WHERE id = $1
            ",
        )
        .bind(job.id.0)
        .bind(Self::status_to_str(job.status))
        .bind(i32::from(job.priority))
        .bind(payload)
        .bind(&job.result)
        .bind(&job.error_message)
        .bind(job.attempts as i32)
        .bind(job.scheduled_at)
        .bind(job.started_at)
        .bind(job.completed_at)
        .execute(&self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(RepositoryError::NotFound);
        }

        Ok(())
    }

    async fn claim_next(&self) -> RepositoryResult<Option<Job>> {
        // Use a transaction with row locking to atomically claim a job
        let record = sqlx::query(
            r"
            UPDATE jobs SET
                status = 'running',
                started_at = NOW(),
                attempts = attempts + 1
            WHERE id = (
                SELECT id FROM jobs
                WHERE status IN ('pending', 'scheduled')
                  AND scheduled_at <= NOW()
                ORDER BY priority DESC, scheduled_at ASC
                LIMIT 1
                FOR UPDATE SKIP LOCKED
            )
            RETURNING id, tenant_id, job_type, status, priority, payload, result,
                      error_message, attempts, max_attempts, scheduled_at,
                      started_at, completed_at, created_at
            ",
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

        match record {
            Some(row) => Ok(Some(Self::row_to_job(&row)?)),
            None => Ok(None),
        }
    }

    async fn find_due_jobs(&self, limit: u32) -> RepositoryResult<Vec<Job>> {
        let records = sqlx::query(
            r"
            SELECT id, tenant_id, job_type, status, priority, payload, result,
                   error_message, attempts, max_attempts, scheduled_at,
                   started_at, completed_at, created_at
            FROM jobs
            WHERE status IN ('pending', 'scheduled')
              AND scheduled_at <= NOW()
            ORDER BY priority DESC, scheduled_at ASC
            LIMIT $1
            ",
        )
        .bind(i64::from(limit))
        .fetch_all(&self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

        records.iter().map(Self::row_to_job).collect()
    }

    async fn find_stale_jobs(&self, older_than_seconds: u32) -> RepositoryResult<Vec<Job>> {
        let records = sqlx::query(
            r"
            SELECT id, tenant_id, job_type, status, priority, payload, result,
                   error_message, attempts, max_attempts, scheduled_at,
                   started_at, completed_at, created_at
            FROM jobs
            WHERE status = 'running'
              AND started_at < NOW() - INTERVAL '1 second' * $1
            ",
        )
        .bind(i64::from(older_than_seconds))
        .fetch_all(&self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

        records.iter().map(Self::row_to_job).collect()
    }

    async fn cleanup_old_jobs(&self, older_than_days: u32) -> RepositoryResult<u32> {
        // Same rationale as audit_log_repo::cleanup_old_logs: a retention
        // of 0 days is a config mistake, not a request to purge every
        // completed job. No-op protects against that.
        if older_than_days == 0 {
            return Ok(0);
        }

        let result = sqlx::query(
            r"
            DELETE FROM jobs
            WHERE status IN ('completed', 'failed', 'cancelled')
              AND completed_at < NOW() - INTERVAL '1 day' * $1
            ",
        )
        .bind(i64::from(older_than_days))
        .execute(&self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

        Ok(result.rows_affected() as u32)
    }

    async fn count_by_status(&self) -> RepositoryResult<HashMap<JobStatus, u32>> {
        let records = sqlx::query(
            r"
            SELECT status, COUNT(*) as count
            FROM jobs
            GROUP BY status
            ",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

        let mut result = HashMap::new();
        for row in records {
            let status_str: String = row.get("status");
            let count: i64 = row.get("count");

            if let Ok(status) = status_str.parse::<JobStatus>() {
                result.insert(status, count as u32);
            }
        }

        Ok(result)
    }
}
