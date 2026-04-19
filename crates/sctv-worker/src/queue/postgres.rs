//! `PostgreSQL` implementation of the job queue.
//!
//! This implementation uses `SELECT FOR UPDATE SKIP LOCKED` for efficient
//! concurrent job claiming without blocking.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{PgPool, Row};
use std::collections::HashMap;

use crate::error::{WorkerError, WorkerResult};
use crate::jobs::{Job, JobId, JobPayload, JobPriority, JobResult, JobStatus, JobType};
use crate::queue::{EnqueueOptions, JobFilter, JobQueue, QueueStats};

/// PostgreSQL-backed job queue.
pub struct PgJobQueue {
    pool: PgPool,
}

impl PgJobQueue {
    /// Creates a new `PostgreSQL` job queue.
    #[must_use]
    pub const fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Converts a database row to a Job.
    fn row_to_job(row: &sqlx::postgres::PgRow) -> WorkerResult<Job> {
        let id: uuid::Uuid = row.get("id");
        let tenant_id: Option<uuid::Uuid> = row.get("tenant_id");
        let job_type_str: String = row.get("job_type");
        let status_str: String = row.get("status");
        let priority: i32 = row.get("priority");
        let payload_json: serde_json::Value = row.get("payload");
        let result_json: Option<serde_json::Value> = row.get("result");
        let error_message: Option<String> = row.get("error_message");
        let attempts: i32 = row.get("attempts");
        let max_attempts: i32 = row.get("max_attempts");
        let scheduled_at: DateTime<Utc> = row.get("scheduled_at");
        let started_at: Option<DateTime<Utc>> = row.get("started_at");
        let completed_at: Option<DateTime<Utc>> = row.get("completed_at");
        let created_at: DateTime<Utc> = row.get("created_at");

        let job_type = JobType::from_str(&job_type_str)?;
        let status = JobStatus::from_str(&status_str)?;
        let priority = JobPriority::try_from(priority)?;

        let payload: JobPayload = serde_json::from_value(payload_json)?;
        let result: Option<JobResult> = result_json.map(serde_json::from_value).transpose()?;

        Ok(Job {
            id: JobId(id),
            tenant_id: tenant_id.map(sctv_core::TenantId),
            job_type,
            status,
            priority,
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
}

#[async_trait]
impl JobQueue for PgJobQueue {
    async fn enqueue(&self, payload: JobPayload, options: EnqueueOptions) -> WorkerResult<JobId> {
        let job_id = JobId::new();
        let job_type = payload.job_type();
        let now = Utc::now();

        let priority: i32 = options.priority.unwrap_or_default().into();
        let max_attempts = options.max_attempts.unwrap_or(3) as i32;
        let scheduled_at = options.scheduled_at.unwrap_or(now);
        let status = if options.scheduled_at.is_some() && options.scheduled_at.unwrap() > now {
            JobStatus::Scheduled
        } else {
            JobStatus::Pending
        };

        let payload_json = serde_json::to_value(&payload)?;

        sqlx::query(
            r"
            INSERT INTO jobs (
                id, tenant_id, job_type, status, priority, payload,
                attempts, max_attempts, scheduled_at, created_at
            ) VALUES ($1, $2, $3, $4, $5, $6, 0, $7, $8, $9)
            ",
        )
        .bind(job_id.0)
        .bind(options.tenant_id.map(|t| t.0))
        .bind(job_type.as_str())
        .bind(status.as_str())
        .bind(priority)
        .bind(payload_json)
        .bind(max_attempts)
        .bind(scheduled_at)
        .bind(now)
        .execute(&self.pool)
        .await?;

        tracing::debug!(job_id = %job_id, job_type = ?job_type, "Job enqueued");

        Ok(job_id)
    }

    async fn enqueue_batch(
        &self,
        jobs: Vec<(JobPayload, EnqueueOptions)>,
    ) -> WorkerResult<Vec<JobId>> {
        if jobs.is_empty() {
            return Ok(Vec::new());
        }

        let mut tx = self.pool.begin().await?;
        let mut job_ids = Vec::with_capacity(jobs.len());
        let now = Utc::now();

        for (payload, options) in jobs {
            let job_id = JobId::new();
            let job_type = payload.job_type();

            let priority: i32 = options.priority.unwrap_or_default().into();
            let max_attempts = options.max_attempts.unwrap_or(3) as i32;
            let scheduled_at = options.scheduled_at.unwrap_or(now);
            let status = if options.scheduled_at.is_some() && options.scheduled_at.unwrap() > now {
                JobStatus::Scheduled
            } else {
                JobStatus::Pending
            };

            let payload_json = serde_json::to_value(&payload)?;

            sqlx::query(
                r"
                INSERT INTO jobs (
                    id, tenant_id, job_type, status, priority, payload,
                    attempts, max_attempts, scheduled_at, created_at
                ) VALUES ($1, $2, $3, $4, $5, $6, 0, $7, $8, $9)
                ",
            )
            .bind(job_id.0)
            .bind(options.tenant_id.map(|t| t.0))
            .bind(job_type.as_str())
            .bind(status.as_str())
            .bind(priority)
            .bind(payload_json)
            .bind(max_attempts)
            .bind(scheduled_at)
            .bind(now)
            .execute(&mut *tx)
            .await?;

            job_ids.push(job_id);
        }

        tx.commit().await?;

        tracing::debug!(count = job_ids.len(), "Batch of jobs enqueued");

        Ok(job_ids)
    }

    async fn claim_next(&self, job_types: &[JobType]) -> WorkerResult<Option<Job>> {
        let job_type_strs: Vec<&str> = job_types.iter().map(JobType::as_str).collect();
        let now = Utc::now();

        // Use SELECT FOR UPDATE SKIP LOCKED to claim a job without blocking
        let row = sqlx::query(
            r"
            WITH next_job AS (
                SELECT id FROM jobs
                WHERE status IN ('pending', 'scheduled')
                  AND job_type = ANY($1)
                  AND scheduled_at <= $2
                ORDER BY priority DESC, scheduled_at ASC
                LIMIT 1
                FOR UPDATE SKIP LOCKED
            )
            UPDATE jobs
            SET status = 'running', started_at = $3, attempts = attempts + 1
            FROM next_job
            WHERE jobs.id = next_job.id
            RETURNING jobs.id, jobs.tenant_id, jobs.job_type, jobs.status,
                      jobs.priority, jobs.payload, jobs.result, jobs.error_message,
                      jobs.attempts, jobs.max_attempts, jobs.scheduled_at,
                      jobs.started_at, jobs.completed_at, jobs.created_at
            ",
        )
        .bind(&job_type_strs)
        .bind(now)
        .bind(now)
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(row) => {
                let job = Self::row_to_job(&row)?;
                tracing::debug!(job_id = %job.id, job_type = ?job.job_type, "Job claimed");
                Ok(Some(job))
            }
            None => Ok(None),
        }
    }

    async fn claim_batch(&self, job_types: &[JobType], limit: u32) -> WorkerResult<Vec<Job>> {
        let job_type_strs: Vec<&str> = job_types.iter().map(JobType::as_str).collect();
        let now = Utc::now();

        let rows = sqlx::query(
            r"
            WITH next_jobs AS (
                SELECT id FROM jobs
                WHERE status IN ('pending', 'scheduled')
                  AND job_type = ANY($1)
                  AND scheduled_at <= $2
                ORDER BY priority DESC, scheduled_at ASC
                LIMIT $3
                FOR UPDATE SKIP LOCKED
            )
            UPDATE jobs
            SET status = 'running', started_at = $4, attempts = attempts + 1
            FROM next_jobs
            WHERE jobs.id = next_jobs.id
            RETURNING jobs.id, jobs.tenant_id, jobs.job_type, jobs.status,
                      jobs.priority, jobs.payload, jobs.result, jobs.error_message,
                      jobs.attempts, jobs.max_attempts, jobs.scheduled_at,
                      jobs.started_at, jobs.completed_at, jobs.created_at
            ",
        )
        .bind(&job_type_strs)
        .bind(now)
        .bind(i64::from(limit))
        .bind(now)
        .fetch_all(&self.pool)
        .await?;

        let jobs: WorkerResult<Vec<Job>> = rows.iter().map(Self::row_to_job).collect();
        let jobs = jobs?;

        if !jobs.is_empty() {
            tracing::debug!(count = jobs.len(), "Batch of jobs claimed");
        }

        Ok(jobs)
    }

    async fn complete(&self, job_id: JobId, result: JobResult) -> WorkerResult<()> {
        let result_json = serde_json::to_value(&result)?;
        let now = Utc::now();

        let rows_affected = sqlx::query(
            r"
            UPDATE jobs
            SET status = 'completed', result = $2, completed_at = $3
            WHERE id = $1 AND status = 'running'
            ",
        )
        .bind(job_id.0)
        .bind(result_json)
        .bind(now)
        .execute(&self.pool)
        .await?
        .rows_affected();

        if rows_affected == 0 {
            return Err(WorkerError::JobNotFound(job_id.to_string()));
        }

        tracing::debug!(job_id = %job_id, "Job completed");

        Ok(())
    }

    async fn fail(&self, job_id: JobId, error: &str) -> WorkerResult<()> {
        let now = Utc::now();

        // Check if job can be retried
        let row = sqlx::query(
            "SELECT attempts, max_attempts FROM jobs WHERE id = $1 AND status = 'running'",
        )
        .bind(job_id.0)
        .fetch_optional(&self.pool)
        .await?;

        let (attempts, max_attempts): (i32, i32) = match row {
            Some(row) => (row.get("attempts"), row.get("max_attempts")),
            None => return Err(WorkerError::JobNotFound(job_id.to_string())),
        };

        let new_status = if attempts < max_attempts {
            "pending"
        } else {
            "failed"
        };

        sqlx::query(
            r"
            UPDATE jobs
            SET status = $2, error_message = $3, completed_at = $4
            WHERE id = $1
            ",
        )
        .bind(job_id.0)
        .bind(new_status)
        .bind(error)
        .bind(now)
        .execute(&self.pool)
        .await?;

        tracing::debug!(
            job_id = %job_id,
            status = new_status,
            attempts = attempts,
            max_attempts = max_attempts,
            "Job failed"
        );

        Ok(())
    }

    async fn retry(&self, job_id: JobId) -> WorkerResult<()> {
        let rows_affected = sqlx::query(
            r"
            UPDATE jobs
            SET status = 'pending', started_at = NULL, completed_at = NULL, error_message = NULL
            WHERE id = $1 AND status IN ('failed', 'cancelled')
            ",
        )
        .bind(job_id.0)
        .execute(&self.pool)
        .await?
        .rows_affected();

        if rows_affected == 0 {
            return Err(WorkerError::JobNotFound(job_id.to_string()));
        }

        tracing::debug!(job_id = %job_id, "Job queued for retry");

        Ok(())
    }

    async fn cancel(&self, job_id: JobId) -> WorkerResult<()> {
        let now = Utc::now();

        let rows_affected = sqlx::query(
            r"
            UPDATE jobs
            SET status = 'cancelled', completed_at = $2
            WHERE id = $1 AND status IN ('pending', 'scheduled')
            ",
        )
        .bind(job_id.0)
        .bind(now)
        .execute(&self.pool)
        .await?
        .rows_affected();

        if rows_affected == 0 {
            return Err(WorkerError::JobNotFound(job_id.to_string()));
        }

        tracing::debug!(job_id = %job_id, "Job cancelled");

        Ok(())
    }

    async fn get(&self, job_id: JobId) -> WorkerResult<Option<Job>> {
        let row = sqlx::query(
            r"
            SELECT id, tenant_id, job_type, status, priority, payload, result,
                   error_message, attempts, max_attempts, scheduled_at,
                   started_at, completed_at, created_at
            FROM jobs WHERE id = $1
            ",
        )
        .bind(job_id.0)
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(row) => Ok(Some(Self::row_to_job(&row)?)),
            None => Ok(None),
        }
    }

    async fn list(&self, filter: JobFilter, limit: u32, offset: u32) -> WorkerResult<Vec<Job>> {
        // Build dynamic query based on filters
        let mut query = String::from(
            r"
            SELECT id, tenant_id, job_type, status, priority, payload, result,
                   error_message, attempts, max_attempts, scheduled_at,
                   started_at, completed_at, created_at
            FROM jobs WHERE 1=1
            ",
        );

        let mut param_count = 0;

        if filter.status.is_some() {
            param_count += 1;
            query.push_str(&format!(" AND status = ANY(${param_count})"));
        }

        if filter.job_type.is_some() {
            param_count += 1;
            query.push_str(&format!(" AND job_type = ANY(${param_count})"));
        }

        if filter.tenant_id.is_some() {
            param_count += 1;
            query.push_str(&format!(" AND tenant_id = ${param_count}"));
        }

        if filter.min_priority.is_some() {
            param_count += 1;
            query.push_str(&format!(" AND priority >= ${param_count}"));
        }

        if filter.scheduled_before.is_some() {
            param_count += 1;
            query.push_str(&format!(" AND scheduled_at <= ${param_count}"));
        }

        if filter.created_after.is_some() {
            param_count += 1;
            query.push_str(&format!(" AND created_at >= ${param_count}"));
        }

        query.push_str(&format!(
            " ORDER BY priority DESC, created_at DESC LIMIT ${} OFFSET ${}",
            param_count + 1,
            param_count + 2
        ));

        let mut query_builder = sqlx::query(&query);

        if let Some(ref statuses) = filter.status {
            let status_strs: Vec<&str> = statuses.iter().map(JobStatus::as_str).collect();
            query_builder = query_builder.bind(status_strs);
        }

        if let Some(ref job_types) = filter.job_type {
            let type_strs: Vec<&str> = job_types.iter().map(JobType::as_str).collect();
            query_builder = query_builder.bind(type_strs);
        }

        if let Some(tenant_id) = filter.tenant_id {
            query_builder = query_builder.bind(tenant_id.0);
        }

        if let Some(min_priority) = filter.min_priority {
            let priority: i32 = min_priority.into();
            query_builder = query_builder.bind(priority);
        }

        if let Some(scheduled_before) = filter.scheduled_before {
            query_builder = query_builder.bind(scheduled_before);
        }

        if let Some(created_after) = filter.created_after {
            query_builder = query_builder.bind(created_after);
        }

        let rows = query_builder
            .bind(i64::from(limit))
            .bind(i64::from(offset))
            .fetch_all(&self.pool)
            .await?;

        rows.iter().map(Self::row_to_job).collect()
    }

    async fn stats(&self) -> WorkerResult<QueueStats> {
        // Get counts by status
        let status_counts = sqlx::query(
            r"
            SELECT status, COUNT(*) as count
            FROM jobs
            GROUP BY status
            ",
        )
        .fetch_all(&self.pool)
        .await?;

        let mut stats = QueueStats::default();

        for row in &status_counts {
            let status: String = row.get("status");
            let count: i64 = row.get("count");

            match status.as_str() {
                "pending" => stats.pending = count as u64,
                "running" => stats.running = count as u64,
                "completed" => stats.completed = count as u64,
                "failed" => stats.failed = count as u64,
                "scheduled" => stats.scheduled = count as u64,
                _ => {}
            }
        }

        // Get counts by job type (for pending/running only)
        let type_counts = sqlx::query(
            r"
            SELECT job_type, COUNT(*) as count
            FROM jobs
            WHERE status IN ('pending', 'running', 'scheduled')
            GROUP BY job_type
            ",
        )
        .fetch_all(&self.pool)
        .await?;

        let mut by_type = HashMap::new();
        for row in type_counts {
            let job_type: String = row.get("job_type");
            let count: i64 = row.get("count");
            by_type.insert(job_type, count as u64);
        }
        stats.by_type = by_type;

        Ok(stats)
    }

    async fn release_stale_jobs(&self, timeout_minutes: u32) -> WorkerResult<u32> {
        let cutoff = Utc::now() - chrono::Duration::minutes(i64::from(timeout_minutes));

        let rows_affected = sqlx::query(
            r"
            UPDATE jobs
            SET status = 'pending', started_at = NULL
            WHERE status = 'running' AND started_at < $1
            ",
        )
        .bind(cutoff)
        .execute(&self.pool)
        .await?
        .rows_affected();

        if rows_affected > 0 {
            tracing::info!(
                count = rows_affected,
                timeout_minutes = timeout_minutes,
                "Released stale jobs"
            );
        }

        Ok(rows_affected as u32)
    }

    async fn cleanup_old_jobs(&self, retention_days: u32) -> WorkerResult<u32> {
        let cutoff = Utc::now() - chrono::Duration::days(i64::from(retention_days));

        let rows_affected = sqlx::query(
            r"
            DELETE FROM jobs
            WHERE status IN ('completed', 'failed', 'cancelled')
              AND completed_at < $1
            ",
        )
        .bind(cutoff)
        .execute(&self.pool)
        .await?
        .rows_affected();

        if rows_affected > 0 {
            tracing::info!(
                count = rows_affected,
                retention_days = retention_days,
                "Cleaned up old jobs"
            );
        }

        Ok(rows_affected as u32)
    }

    async fn has_pending(&self, job_types: &[JobType]) -> WorkerResult<bool> {
        let job_type_strs: Vec<&str> = job_types.iter().map(JobType::as_str).collect();
        let now = Utc::now();

        let row = sqlx::query(
            r"
            SELECT EXISTS(
                SELECT 1 FROM jobs
                WHERE status IN ('pending', 'scheduled')
                  AND job_type = ANY($1)
                  AND scheduled_at <= $2
            ) as has_pending
            ",
        )
        .bind(&job_type_strs)
        .bind(now)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.get("has_pending"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: These tests require a running PostgreSQL database.
    // In a real implementation, you'd use testcontainers or a test database.

    #[test]
    fn test_job_status_roundtrip() {
        let statuses = vec![
            JobStatus::Pending,
            JobStatus::Running,
            JobStatus::Completed,
            JobStatus::Failed,
            JobStatus::Cancelled,
            JobStatus::Scheduled,
        ];

        for status in statuses {
            let str_repr = status.as_str();
            let parsed = JobStatus::from_str(str_repr).unwrap();
            assert_eq!(status, parsed);
        }
    }

    #[test]
    fn test_job_type_roundtrip() {
        let types = vec![
            JobType::ScanProject,
            JobType::MonitorRegistry,
            JobType::VerifyProvenance,
            JobType::SendNotification,
        ];

        for job_type in types {
            let str_repr = job_type.as_str();
            let parsed = JobType::from_str(str_repr).unwrap();
            assert_eq!(job_type, parsed);
        }
    }

    #[test]
    fn test_job_priority_conversions() {
        let priorities = vec![
            (JobPriority::Low, 0),
            (JobPriority::Normal, 5),
            (JobPriority::High, 10),
            (JobPriority::Critical, 15),
        ];

        for (priority, expected_int) in priorities {
            let int_val: i32 = priority.into();
            assert_eq!(int_val, expected_int);

            let parsed = JobPriority::try_from(expected_int).unwrap();
            assert_eq!(priority, parsed);
        }
    }
}
