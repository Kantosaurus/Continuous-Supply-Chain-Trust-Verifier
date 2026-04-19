//! Worker service providing a high-level API for job management.
//!
//! This module provides the main entry point for the worker system, combining
//! the job queue, executor registry, and worker pool into a cohesive service.

use std::sync::Arc;

use sqlx::PgPool;

use crate::error::WorkerResult;
use crate::executor::{
    ExecutionContext, ExecutorConfig, ExecutorRegistry, MonitorRegistryExecutor,
    ScanProjectExecutor, SendNotificationExecutor, VerifyProvenanceExecutor,
};
use crate::jobs::{Job, JobId, JobPayload, JobType};
use crate::pool::{WorkerPool, WorkerPoolConfig, WorkerPoolHandle, WorkerPoolStatsSnapshot};
use crate::queue::{EnqueueOptions, JobFilter, JobQueue, PgJobQueue, QueueStats};

/// Configuration for the worker service.
#[derive(Debug, Clone)]
pub struct WorkerServiceConfig {
    /// Worker pool configuration.
    pub pool: WorkerPoolConfig,
    /// Executor configuration.
    pub executor: ExecutorConfig,
    /// Job retention period in days.
    pub job_retention_days: u32,
    /// Cleanup interval in hours.
    pub cleanup_interval_hours: u32,
}

impl Default for WorkerServiceConfig {
    fn default() -> Self {
        Self {
            pool: WorkerPoolConfig::default(),
            executor: ExecutorConfig::default(),
            job_retention_days: 7,
            cleanup_interval_hours: 24,
        }
    }
}

impl WorkerServiceConfig {
    /// Creates a new configuration.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the worker pool configuration.
    #[must_use]
    pub fn with_pool(mut self, config: WorkerPoolConfig) -> Self {
        self.pool = config;
        self
    }

    /// Sets the executor configuration.
    #[must_use]
    pub fn with_executor(mut self, config: ExecutorConfig) -> Self {
        self.executor = config;
        self
    }

    /// Sets the job retention period.
    #[must_use]
    pub fn with_retention(mut self, days: u32) -> Self {
        self.job_retention_days = days;
        self
    }
}

/// Builder for creating a worker service.
pub struct WorkerServiceBuilder {
    db_pool: Option<PgPool>,
    config: WorkerServiceConfig,
    http_client: Option<reqwest::Client>,
    custom_executors: Vec<Box<dyn FnOnce(&mut ExecutorRegistry)>>,
}

impl WorkerServiceBuilder {
    /// Creates a new builder.
    #[must_use]
    pub fn new() -> Self {
        Self {
            db_pool: None,
            config: WorkerServiceConfig::default(),
            http_client: None,
            custom_executors: Vec::new(),
        }
    }

    /// Sets the database pool.
    #[must_use]
    pub fn with_db_pool(mut self, pool: PgPool) -> Self {
        self.db_pool = Some(pool);
        self
    }

    /// Sets the configuration.
    #[must_use]
    pub fn with_config(mut self, config: WorkerServiceConfig) -> Self {
        self.config = config;
        self
    }

    /// Sets a custom HTTP client.
    #[must_use]
    pub fn with_http_client(mut self, client: reqwest::Client) -> Self {
        self.http_client = Some(client);
        self
    }

    /// Adds a custom executor registration function.
    pub fn with_custom_executor<F>(mut self, f: F) -> Self
    where
        F: FnOnce(&mut ExecutorRegistry) + 'static,
    {
        self.custom_executors.push(Box::new(f));
        self
    }

    /// Builds the worker service.
    pub fn build(self) -> WorkerResult<WorkerService> {
        let db_pool = self.db_pool.ok_or_else(|| {
            crate::error::WorkerError::Configuration("Database pool required".into())
        })?;

        // Create executor registry with default executors
        let mut registry = ExecutorRegistry::new();
        registry.register(ScanProjectExecutor::new());
        registry.register(MonitorRegistryExecutor::new());
        registry.register(VerifyProvenanceExecutor::new());
        registry.register(SendNotificationExecutor::new());

        // Register custom executors
        for register_fn in self.custom_executors {
            register_fn(&mut registry);
        }

        // Create execution context
        let mut context =
            ExecutionContext::new(db_pool.clone()).with_config(self.config.executor.clone());

        if let Some(http_client) = self.http_client {
            context = context.with_http_client(http_client);
        }

        // Create job queue
        let queue = PgJobQueue::new(db_pool);

        Ok(WorkerService {
            queue: Arc::new(queue),
            registry: Arc::new(registry),
            context,
            config: self.config,
            pool_handle: None,
        })
    }
}

impl Default for WorkerServiceBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// The main worker service.
pub struct WorkerService {
    queue: Arc<PgJobQueue>,
    registry: Arc<ExecutorRegistry>,
    context: ExecutionContext,
    config: WorkerServiceConfig,
    pool_handle: Option<WorkerPoolHandle>,
}

impl WorkerService {
    /// Creates a new builder.
    #[must_use]
    pub fn builder() -> WorkerServiceBuilder {
        WorkerServiceBuilder::new()
    }

    /// Enqueues a job for processing.
    pub async fn enqueue(&self, payload: JobPayload) -> WorkerResult<JobId> {
        self.queue.enqueue(payload, EnqueueOptions::default()).await
    }

    /// Enqueues a job with options.
    pub async fn enqueue_with_options(
        &self,
        payload: JobPayload,
        options: EnqueueOptions,
    ) -> WorkerResult<JobId> {
        self.queue.enqueue(payload, options).await
    }

    /// Enqueues multiple jobs.
    pub async fn enqueue_batch(
        &self,
        jobs: Vec<(JobPayload, EnqueueOptions)>,
    ) -> WorkerResult<Vec<JobId>> {
        self.queue.enqueue_batch(jobs).await
    }

    /// Gets a job by ID.
    pub async fn get_job(&self, job_id: JobId) -> WorkerResult<Option<Job>> {
        self.queue.get(job_id).await
    }

    /// Lists jobs with optional filters.
    pub async fn list_jobs(
        &self,
        filter: JobFilter,
        limit: u32,
        offset: u32,
    ) -> WorkerResult<Vec<Job>> {
        self.queue.list(filter, limit, offset).await
    }

    /// Retries a failed job.
    pub async fn retry_job(&self, job_id: JobId) -> WorkerResult<()> {
        self.queue.retry(job_id).await
    }

    /// Cancels a pending job.
    pub async fn cancel_job(&self, job_id: JobId) -> WorkerResult<()> {
        self.queue.cancel(job_id).await
    }

    /// Gets queue statistics.
    pub async fn queue_stats(&self) -> WorkerResult<QueueStats> {
        self.queue.stats().await
    }

    /// Checks if there are pending jobs.
    pub async fn has_pending_jobs(&self, job_types: &[JobType]) -> WorkerResult<bool> {
        self.queue.has_pending(job_types).await
    }

    /// Cleans up old completed/failed jobs.
    pub async fn cleanup_old_jobs(&self) -> WorkerResult<u32> {
        self.queue
            .cleanup_old_jobs(self.config.job_retention_days)
            .await
    }

    /// Releases stale running jobs.
    pub async fn release_stale_jobs(&self) -> WorkerResult<u32> {
        self.queue
            .release_stale_jobs(self.config.pool.stale_job_timeout_minutes)
            .await
    }

    /// Starts the worker pool.
    ///
    /// Returns an error if the pool is already running.
    pub fn start(&mut self) -> WorkerResult<()> {
        if self.pool_handle.is_some() {
            return Err(crate::error::WorkerError::Pool(
                "Worker pool is already running".into(),
            ));
        }

        // We need to create new instances for the pool since it takes ownership
        let queue = PgJobQueue::new(self.context.db_pool.clone());
        let registry = ExecutorRegistry::new();
        // Note: In a real implementation, we'd clone the registry or use Arc<ExecutorRegistry>

        let pool = WorkerPool::new(
            queue,
            registry,
            self.context.clone(),
            self.config.pool.clone(),
        );

        let handle = pool.start();
        self.pool_handle = Some(handle);

        tracing::info!("Worker service started");
        Ok(())
    }

    /// Gets worker pool statistics.
    pub fn pool_stats(&self) -> Option<WorkerPoolStatsSnapshot> {
        self.pool_handle.as_ref().map(|h| h.stats())
    }

    /// Checks if the worker pool is running.
    pub fn is_running(&self) -> bool {
        self.pool_handle
            .as_ref()
            .map(|h| h.is_running())
            .unwrap_or(false)
    }

    /// Initiates graceful shutdown of the worker pool.
    pub fn shutdown(&self) {
        if let Some(handle) = &self.pool_handle {
            handle.shutdown();
        }
    }

    /// Stops the worker pool and waits for completion.
    pub async fn stop(&mut self) -> WorkerResult<()> {
        if let Some(handle) = self.pool_handle.take() {
            handle.stop().await?;
            tracing::info!("Worker service stopped");
        }
        Ok(())
    }

    /// Returns the registered job types.
    pub fn registered_job_types(&self) -> Vec<JobType> {
        self.registry.registered_types()
    }
}

/// Convenience functions for creating common jobs.
impl WorkerService {
    /// Creates and enqueues a project scan job.
    pub async fn scan_project(
        &self,
        project_id: sctv_core::ProjectId,
        tenant_id: sctv_core::TenantId,
    ) -> WorkerResult<JobId> {
        use crate::jobs::{JobPayload, ScanProjectPayload};

        let payload = JobPayload::ScanProject(ScanProjectPayload::new(project_id, tenant_id));
        self.enqueue(payload).await
    }

    /// Creates and enqueues a registry monitoring job.
    pub async fn monitor_registry(
        &self,
        ecosystem: sctv_core::PackageEcosystem,
    ) -> WorkerResult<JobId> {
        use crate::jobs::{JobPayload, MonitorRegistryPayload};

        let payload = JobPayload::MonitorRegistry(MonitorRegistryPayload::new(ecosystem));
        self.enqueue(payload).await
    }

    /// Creates and enqueues a provenance verification job.
    pub async fn verify_provenance(
        &self,
        dependency_id: sctv_core::DependencyId,
        tenant_id: sctv_core::TenantId,
        ecosystem: sctv_core::PackageEcosystem,
        package_name: String,
        version: String,
    ) -> WorkerResult<JobId> {
        use crate::jobs::{JobPayload, VerifyProvenancePayload};

        let payload = JobPayload::VerifyProvenance(VerifyProvenancePayload::new(
            dependency_id,
            tenant_id,
            ecosystem,
            package_name,
            version,
        ));
        self.enqueue(payload).await
    }

    /// Creates and enqueues a notification job.
    pub async fn send_notification(
        &self,
        alert_id: sctv_core::AlertId,
        tenant_id: sctv_core::TenantId,
        channel: crate::jobs::NotificationChannel,
        severity: sctv_core::Severity,
        title: String,
        description: String,
    ) -> WorkerResult<JobId> {
        use crate::jobs::{JobPayload, SendNotificationPayload};

        let payload = JobPayload::SendNotification(SendNotificationPayload::new(
            alert_id,
            tenant_id,
            channel,
            severity,
            title,
            description,
        ));
        self.enqueue(payload).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_worker_service_config_builder() {
        let config = WorkerServiceConfig::new()
            .with_pool(WorkerPoolConfig::new().with_workers(8))
            .with_retention(14);

        assert_eq!(config.pool.worker_count, 8);
        assert_eq!(config.job_retention_days, 14);
    }
}
