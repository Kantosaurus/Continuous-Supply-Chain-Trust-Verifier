//! Background job processor for Supply Chain Trust Verifier.
//!
//! This crate provides a robust, PostgreSQL-backed job queue system for processing
//! background tasks such as project scanning, registry monitoring, provenance
//! verification, and notification delivery.
//!
//! # Architecture
//!
//! The worker system is composed of several key components:
//!
//! - **Job Queue** (`queue`): PostgreSQL-backed queue using `SELECT FOR UPDATE SKIP LOCKED`
//!   for efficient concurrent job claiming without blocking.
//!
//! - **Job Types** (`jobs`): Definitions for different job types and their payloads:
//!   - `ScanProject`: Scan a project's dependencies for security threats
//!   - `MonitorRegistry`: Monitor package registries for changes
//!   - `VerifyProvenance`: Verify SLSA/Sigstore attestations
//!   - `SendNotification`: Send alerts via various channels
//!
//! - **Executors** (`executor`): Trait-based job handlers with a registry pattern.
//!
//! - **Worker Pool** (`pool`): Configurable Tokio-based worker pool for concurrent processing.
//!
//! - **Service** (`service`): High-level API combining all components.
//!
//! # Usage
//!
//! ```ignore
//! use sctv_worker::{WorkerService, WorkerServiceConfig, WorkerPoolConfig};
//!
//! // Create the worker service
//! let service = WorkerService::builder()
//!     .with_db_pool(db_pool)
//!     .with_config(WorkerServiceConfig::new()
//!         .with_pool(WorkerPoolConfig::new().with_workers(4)))
//!     .build()?;
//!
//! // Enqueue a job
//! let job_id = service.scan_project(project_id, tenant_id).await?;
//!
//! // Start the worker pool
//! service.start()?;
//!
//! // ... later, gracefully stop
//! service.stop().await?;
//! ```
//!
//! # Job Lifecycle
//!
//! 1. Jobs are enqueued with a status of `Pending` (or `Scheduled` for future execution)
//! 2. Workers claim jobs using `SELECT FOR UPDATE SKIP LOCKED`, setting status to `Running`
//! 3. After execution, jobs are marked as `Completed` or `Failed`
//! 4. Failed jobs are automatically retried up to `max_attempts` times
//! 5. Old completed/failed jobs are periodically cleaned up
//!
//! # Reliability Features
//!
//! - **Atomic job claiming**: Uses `PostgreSQL`'s `SKIP LOCKED` for safe concurrent access
//! - **Automatic retries**: Configurable retry count with exponential backoff potential
//! - **Stale job recovery**: Automatically releases jobs that appear stuck
//! - **Graceful shutdown**: Waits for running jobs to complete before stopping
//! - **Job prioritization**: High priority jobs are processed first

pub mod error;
pub mod executor;
pub mod jobs;
pub mod pool;
pub mod queue;
pub mod service;

// Re-export commonly used types at the crate root
pub use error::{WorkerError, WorkerResult};
pub use executor::{
    BoxedExecutor, ExecutionContext, ExecutorConfig, ExecutorRegistry, JobExecutor,
    MonitorRegistryExecutor, ScanProjectExecutor, SendNotificationExecutor,
    VerifyProvenanceExecutor,
};
pub use jobs::{
    Job,
    JobId,
    JobPayload,
    JobPriority,
    JobResult,
    JobStatus,
    JobType,
    // Payloads
    MonitorRegistryPayload,
    MonitorRegistryResult,
    NotificationChannel,
    NotificationContext,
    ProvenanceVerificationStatus,
    ScanProjectPayload,
    ScanProjectResult,
    SendNotificationPayload,
    SendNotificationResult,
    SigstoreDetails,
    VerifyProvenancePayload,
    VerifyProvenanceResult,
};
pub use pool::{
    WorkerPool, WorkerPoolConfig, WorkerPoolHandle, WorkerPoolStats, WorkerPoolStatsSnapshot,
};
pub use queue::{EnqueueOptions, JobFilter, JobQueue, PgJobQueue, QueueStats};
pub use service::{WorkerService, WorkerServiceBuilder, WorkerServiceConfig};
