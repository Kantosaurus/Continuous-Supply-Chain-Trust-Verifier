//! Worker pool for concurrent job processing.
//!
//! This module provides a configurable worker pool that spawns multiple Tokio tasks
//! to process jobs concurrently from the queue.

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::{broadcast, Semaphore};
use tokio::task::JoinHandle;

use crate::error::{WorkerError, WorkerResult};
use crate::executor::{ExecutionContext, ExecutorRegistry};
use crate::jobs::JobType;
use crate::queue::JobQueue;

/// Configuration for the worker pool.
#[derive(Debug, Clone)]
pub struct WorkerPoolConfig {
    /// Number of concurrent workers.
    pub worker_count: usize,
    /// Job types to process (empty means all registered types).
    pub job_types: Vec<JobType>,
    /// Polling interval when no jobs are available (in milliseconds).
    pub poll_interval_ms: u64,
    /// Maximum time a job can run before being considered stuck (in minutes).
    pub stale_job_timeout_minutes: u32,
    /// How often to check for stale jobs (in seconds).
    pub stale_check_interval_secs: u64,
    /// Enable graceful shutdown (wait for running jobs to complete).
    pub graceful_shutdown: bool,
    /// Maximum time to wait for graceful shutdown (in seconds).
    pub shutdown_timeout_secs: u64,
}

impl Default for WorkerPoolConfig {
    fn default() -> Self {
        Self {
            worker_count: 4,
            job_types: Vec::new(),
            poll_interval_ms: 1000,
            stale_job_timeout_minutes: 30,
            stale_check_interval_secs: 60,
            graceful_shutdown: true,
            shutdown_timeout_secs: 30,
        }
    }
}

impl WorkerPoolConfig {
    /// Creates a new worker pool configuration.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the number of workers.
    #[must_use]
    pub fn with_workers(mut self, count: usize) -> Self {
        self.worker_count = count;
        self
    }

    /// Sets the job types to process.
    #[must_use]
    pub fn with_job_types(mut self, types: Vec<JobType>) -> Self {
        self.job_types = types;
        self
    }

    /// Sets the polling interval.
    #[must_use]
    pub fn with_poll_interval(mut self, ms: u64) -> Self {
        self.poll_interval_ms = ms;
        self
    }
}

/// Statistics for the worker pool.
#[derive(Debug, Default)]
pub struct WorkerPoolStats {
    /// Total jobs processed.
    pub jobs_processed: AtomicU64,
    /// Total jobs completed successfully.
    pub jobs_completed: AtomicU64,
    /// Total jobs failed.
    pub jobs_failed: AtomicU64,
    /// Currently running jobs.
    pub jobs_running: AtomicU64,
}

impl WorkerPoolStats {
    /// Returns a snapshot of the current statistics.
    pub fn snapshot(&self) -> WorkerPoolStatsSnapshot {
        WorkerPoolStatsSnapshot {
            jobs_processed: self.jobs_processed.load(Ordering::Relaxed),
            jobs_completed: self.jobs_completed.load(Ordering::Relaxed),
            jobs_failed: self.jobs_failed.load(Ordering::Relaxed),
            jobs_running: self.jobs_running.load(Ordering::Relaxed),
        }
    }
}

/// A snapshot of worker pool statistics.
#[derive(Debug, Clone)]
pub struct WorkerPoolStatsSnapshot {
    pub jobs_processed: u64,
    pub jobs_completed: u64,
    pub jobs_failed: u64,
    pub jobs_running: u64,
}

/// A worker pool that processes jobs concurrently.
pub struct WorkerPool<Q: JobQueue> {
    queue: Arc<Q>,
    executors: Arc<ExecutorRegistry>,
    context: ExecutionContext,
    config: WorkerPoolConfig,
    stats: Arc<WorkerPoolStats>,
    shutdown: Arc<AtomicBool>,
    shutdown_tx: broadcast::Sender<()>,
}

impl<Q: JobQueue + 'static> WorkerPool<Q> {
    /// Creates a new worker pool.
    pub fn new(
        queue: Q,
        executors: ExecutorRegistry,
        context: ExecutionContext,
        config: WorkerPoolConfig,
    ) -> Self {
        let (shutdown_tx, _) = broadcast::channel(1);
        Self {
            queue: Arc::new(queue),
            executors: Arc::new(executors),
            context,
            config,
            stats: Arc::new(WorkerPoolStats::default()),
            shutdown: Arc::new(AtomicBool::new(false)),
            shutdown_tx,
        }
    }

    /// Returns the current statistics.
    pub fn stats(&self) -> WorkerPoolStatsSnapshot {
        self.stats.snapshot()
    }

    /// Starts the worker pool and returns a handle.
    pub fn start(self) -> WorkerPoolHandle {
        let worker_count = self.config.worker_count;
        let stats = self.stats.clone();
        let shutdown = self.shutdown.clone();
        let shutdown_tx = self.shutdown_tx.clone();

        // Determine which job types to process
        let job_types: Vec<JobType> = if self.config.job_types.is_empty() {
            self.executors.registered_types()
        } else {
            self.config.job_types.clone()
        };

        tracing::info!(
            worker_count = worker_count,
            job_types = ?job_types,
            "Starting worker pool"
        );

        let pool = Arc::new(self);
        let mut worker_handles = Vec::with_capacity(worker_count);

        // Spawn worker tasks
        for worker_id in 0..worker_count {
            let pool = pool.clone();
            let job_types = job_types.clone();
            let mut shutdown_rx = pool.shutdown_tx.subscribe();

            let handle = tokio::spawn(async move {
                tracing::debug!(worker_id = worker_id, "Worker started");

                loop {
                    // Check for shutdown
                    if pool.shutdown.load(Ordering::Relaxed) {
                        tracing::debug!(worker_id = worker_id, "Worker shutting down");
                        break;
                    }

                    // Try to claim a job
                    match pool.queue.claim_next(&job_types).await {
                        Ok(Some(job)) => {
                            let job_id = job.id;
                            let job_type = job.job_type.clone();

                            pool.stats.jobs_running.fetch_add(1, Ordering::Relaxed);
                            pool.stats.jobs_processed.fetch_add(1, Ordering::Relaxed);

                            tracing::debug!(
                                worker_id = worker_id,
                                job_id = %job_id,
                                job_type = ?job_type,
                                "Processing job"
                            );

                            // Execute the job
                            match pool.executors.execute(&job, &pool.context).await {
                                Ok(result) => {
                                    if let Err(e) = pool.queue.complete(job_id, result).await {
                                        tracing::error!(
                                            worker_id = worker_id,
                                            job_id = %job_id,
                                            error = %e,
                                            "Failed to mark job as completed"
                                        );
                                    }
                                    pool.stats.jobs_completed.fetch_add(1, Ordering::Relaxed);
                                }
                                Err(e) => {
                                    tracing::error!(
                                        worker_id = worker_id,
                                        job_id = %job_id,
                                        error = %e,
                                        "Job execution failed"
                                    );
                                    if let Err(e) =
                                        pool.queue.fail(job_id, &e.to_string()).await
                                    {
                                        tracing::error!(
                                            worker_id = worker_id,
                                            job_id = %job_id,
                                            error = %e,
                                            "Failed to mark job as failed"
                                        );
                                    }
                                    pool.stats.jobs_failed.fetch_add(1, Ordering::Relaxed);
                                }
                            }

                            pool.stats.jobs_running.fetch_sub(1, Ordering::Relaxed);
                        }
                        Ok(None) => {
                            // No jobs available, wait before polling again
                            tokio::select! {
                                _ = tokio::time::sleep(Duration::from_millis(
                                    pool.config.poll_interval_ms
                                )) => {}
                                _ = shutdown_rx.recv() => {
                                    tracing::debug!(
                                        worker_id = worker_id,
                                        "Worker received shutdown signal"
                                    );
                                    break;
                                }
                            }
                        }
                        Err(e) => {
                            tracing::error!(
                                worker_id = worker_id,
                                error = %e,
                                "Failed to claim job"
                            );
                            // Wait before retrying
                            tokio::time::sleep(Duration::from_secs(1)).await;
                        }
                    }
                }

                tracing::debug!(worker_id = worker_id, "Worker stopped");
            });

            worker_handles.push(handle);
        }

        // Spawn stale job checker
        let stale_pool = pool.clone();
        let stale_job_types = job_types.clone();
        let stale_handle = tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(
                stale_pool.config.stale_check_interval_secs,
            ));

            loop {
                interval.tick().await;

                if stale_pool.shutdown.load(Ordering::Relaxed) {
                    break;
                }

                match stale_pool
                    .queue
                    .release_stale_jobs(stale_pool.config.stale_job_timeout_minutes)
                    .await
                {
                    Ok(count) if count > 0 => {
                        tracing::warn!(count = count, "Released stale jobs");
                    }
                    Err(e) => {
                        tracing::error!(error = %e, "Failed to release stale jobs");
                    }
                    _ => {}
                }
            }
        });

        WorkerPoolHandle {
            worker_handles,
            stale_handle,
            stats,
            shutdown,
            shutdown_tx,
            config: pool.config.clone(),
        }
    }
}

/// Handle to a running worker pool.
pub struct WorkerPoolHandle {
    worker_handles: Vec<JoinHandle<()>>,
    stale_handle: JoinHandle<()>,
    stats: Arc<WorkerPoolStats>,
    shutdown: Arc<AtomicBool>,
    shutdown_tx: broadcast::Sender<()>,
    config: WorkerPoolConfig,
}

impl WorkerPoolHandle {
    /// Returns the current statistics.
    pub fn stats(&self) -> WorkerPoolStatsSnapshot {
        self.stats.snapshot()
    }

    /// Checks if the pool is running.
    pub fn is_running(&self) -> bool {
        !self.shutdown.load(Ordering::Relaxed)
    }

    /// Initiates graceful shutdown.
    pub fn shutdown(&self) {
        tracing::info!("Initiating worker pool shutdown");
        self.shutdown.store(true, Ordering::Relaxed);
        let _ = self.shutdown_tx.send(());
    }

    /// Waits for all workers to stop.
    pub async fn wait(self) -> WorkerResult<()> {
        if self.config.graceful_shutdown {
            let timeout = Duration::from_secs(self.config.shutdown_timeout_secs);

            match tokio::time::timeout(timeout, async {
                for handle in self.worker_handles {
                    let _ = handle.await;
                }
                let _ = self.stale_handle.await;
            })
            .await
            {
                Ok(_) => {
                    tracing::info!("Worker pool stopped gracefully");
                    Ok(())
                }
                Err(_) => {
                    tracing::warn!("Worker pool shutdown timed out");
                    Err(WorkerError::Timeout)
                }
            }
        } else {
            // Abort all workers immediately
            for handle in self.worker_handles {
                handle.abort();
            }
            self.stale_handle.abort();
            tracing::info!("Worker pool stopped (forced)");
            Ok(())
        }
    }

    /// Initiates shutdown and waits for completion.
    pub async fn stop(self) -> WorkerResult<()> {
        self.shutdown();
        self.wait().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_worker_pool_config_builder() {
        let config = WorkerPoolConfig::new()
            .with_workers(8)
            .with_job_types(vec![JobType::ScanProject, JobType::SendNotification])
            .with_poll_interval(500);

        assert_eq!(config.worker_count, 8);
        assert_eq!(config.job_types.len(), 2);
        assert_eq!(config.poll_interval_ms, 500);
    }

    #[test]
    fn test_worker_pool_stats() {
        let stats = WorkerPoolStats::default();

        stats.jobs_processed.fetch_add(10, Ordering::Relaxed);
        stats.jobs_completed.fetch_add(8, Ordering::Relaxed);
        stats.jobs_failed.fetch_add(2, Ordering::Relaxed);

        let snapshot = stats.snapshot();
        assert_eq!(snapshot.jobs_processed, 10);
        assert_eq!(snapshot.jobs_completed, 8);
        assert_eq!(snapshot.jobs_failed, 2);
    }
}
