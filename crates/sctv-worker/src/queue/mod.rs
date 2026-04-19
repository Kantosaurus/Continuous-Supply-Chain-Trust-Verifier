//! Job queue implementation using `PostgreSQL`.
//!
//! This module provides a reliable job queue backed by `PostgreSQL`, using
//! `SELECT FOR UPDATE SKIP LOCKED` for efficient concurrent job claiming.

mod postgres;

pub use postgres::PgJobQueue;

use async_trait::async_trait;
use chrono::{DateTime, Utc};

use crate::error::WorkerResult;
use crate::jobs::{Job, JobId, JobPayload, JobPriority, JobResult, JobStatus, JobType};

/// Options for enqueueing a job.
#[derive(Debug, Clone, Default)]
pub struct EnqueueOptions {
    /// Priority for the job.
    pub priority: Option<JobPriority>,
    /// Maximum retry attempts.
    pub max_attempts: Option<u32>,
    /// When to execute the job (default: now).
    pub scheduled_at: Option<DateTime<Utc>>,
    /// Tenant ID for tenant-scoped jobs.
    pub tenant_id: Option<sctv_core::TenantId>,
}

impl EnqueueOptions {
    /// Creates new enqueue options.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the priority.
    #[must_use]
    pub const fn with_priority(mut self, priority: JobPriority) -> Self {
        self.priority = Some(priority);
        self
    }

    /// Sets the maximum attempts.
    #[must_use]
    pub const fn with_max_attempts(mut self, max: u32) -> Self {
        self.max_attempts = Some(max);
        self
    }

    /// Schedules the job for a specific time.
    #[must_use]
    pub const fn scheduled_at(mut self, time: DateTime<Utc>) -> Self {
        self.scheduled_at = Some(time);
        self
    }

    /// Sets the tenant ID.
    #[must_use]
    pub const fn with_tenant(mut self, tenant_id: sctv_core::TenantId) -> Self {
        self.tenant_id = Some(tenant_id);
        self
    }
}

/// Filter options for querying jobs.
#[derive(Debug, Clone, Default)]
pub struct JobFilter {
    /// Filter by status.
    pub status: Option<Vec<JobStatus>>,
    /// Filter by job type.
    pub job_type: Option<Vec<JobType>>,
    /// Filter by tenant.
    pub tenant_id: Option<sctv_core::TenantId>,
    /// Filter by minimum priority.
    pub min_priority: Option<JobPriority>,
    /// Filter by scheduled time (before).
    pub scheduled_before: Option<DateTime<Utc>>,
    /// Filter by created time (after).
    pub created_after: Option<DateTime<Utc>>,
}

/// Statistics about the job queue.
#[derive(Debug, Clone, Default)]
pub struct QueueStats {
    /// Number of pending jobs.
    pub pending: u64,
    /// Number of running jobs.
    pub running: u64,
    /// Number of completed jobs (in retention period).
    pub completed: u64,
    /// Number of failed jobs (in retention period).
    pub failed: u64,
    /// Number of scheduled jobs.
    pub scheduled: u64,
    /// Breakdown by job type.
    pub by_type: std::collections::HashMap<String, u64>,
}

/// Trait for job queue implementations.
#[async_trait]
pub trait JobQueue: Send + Sync {
    /// Enqueues a new job.
    async fn enqueue(&self, payload: JobPayload, options: EnqueueOptions) -> WorkerResult<JobId>;

    /// Enqueues multiple jobs in a batch.
    async fn enqueue_batch(
        &self,
        jobs: Vec<(JobPayload, EnqueueOptions)>,
    ) -> WorkerResult<Vec<JobId>>;

    /// Claims the next available job for processing.
    ///
    /// Uses `SELECT FOR UPDATE SKIP LOCKED` to safely claim a job
    /// without blocking other workers.
    async fn claim_next(&self, job_types: &[JobType]) -> WorkerResult<Option<Job>>;

    /// Claims multiple jobs at once (for batch processing).
    async fn claim_batch(&self, job_types: &[JobType], limit: u32) -> WorkerResult<Vec<Job>>;

    /// Marks a job as completed with a result.
    async fn complete(&self, job_id: JobId, result: JobResult) -> WorkerResult<()>;

    /// Marks a job as failed with an error message.
    async fn fail(&self, job_id: JobId, error: &str) -> WorkerResult<()>;

    /// Retries a failed job (resets status to pending).
    async fn retry(&self, job_id: JobId) -> WorkerResult<()>;

    /// Cancels a job.
    async fn cancel(&self, job_id: JobId) -> WorkerResult<()>;

    /// Gets a job by ID.
    async fn get(&self, job_id: JobId) -> WorkerResult<Option<Job>>;

    /// Lists jobs with optional filters.
    async fn list(&self, filter: JobFilter, limit: u32, offset: u32) -> WorkerResult<Vec<Job>>;

    /// Gets queue statistics.
    async fn stats(&self) -> WorkerResult<QueueStats>;

    /// Releases stale running jobs (for crash recovery).
    ///
    /// Jobs that have been running for longer than `timeout_minutes` are
    /// reset to pending status.
    async fn release_stale_jobs(&self, timeout_minutes: u32) -> WorkerResult<u32>;

    /// Cleans up old completed/failed jobs.
    async fn cleanup_old_jobs(&self, retention_days: u32) -> WorkerResult<u32>;

    /// Checks if there are pending jobs.
    async fn has_pending(&self, job_types: &[JobType]) -> WorkerResult<bool>;
}
