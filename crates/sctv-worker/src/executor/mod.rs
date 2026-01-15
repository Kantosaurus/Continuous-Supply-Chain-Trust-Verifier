//! Job executor trait and implementations.
//!
//! This module defines the `JobExecutor` trait that all job handlers must implement,
//! as well as the registry for managing executors.

mod handlers;
mod registry;

pub use handlers::*;
pub use registry::ExecutorRegistry;

use async_trait::async_trait;
use std::sync::Arc;

use crate::error::WorkerResult;
use crate::jobs::{Job, JobResult, JobType};

/// Context provided to job executors during execution.
#[derive(Clone)]
pub struct ExecutionContext {
    /// Database pool for database operations.
    pub db_pool: sqlx::PgPool,
    /// HTTP client for external API calls.
    pub http_client: reqwest::Client,
    /// Configuration for the executor.
    pub config: ExecutorConfig,
}

impl ExecutionContext {
    /// Creates a new execution context.
    #[must_use]
    pub fn new(db_pool: sqlx::PgPool) -> Self {
        Self {
            db_pool,
            http_client: reqwest::Client::new(),
            config: ExecutorConfig::default(),
        }
    }

    /// Sets the HTTP client.
    #[must_use]
    pub fn with_http_client(mut self, client: reqwest::Client) -> Self {
        self.http_client = client;
        self
    }

    /// Sets the executor configuration.
    #[must_use]
    pub fn with_config(mut self, config: ExecutorConfig) -> Self {
        self.config = config;
        self
    }
}

/// Configuration for job executors.
#[derive(Debug, Clone)]
pub struct ExecutorConfig {
    /// Timeout for HTTP requests (in seconds).
    pub http_timeout_secs: u64,
    /// Maximum number of concurrent HTTP requests.
    pub max_concurrent_requests: u32,
    /// Whether to skip SSL verification (for testing only).
    pub skip_ssl_verification: bool,
    /// Dashboard base URL for notification links.
    pub dashboard_base_url: Option<String>,
}

impl Default for ExecutorConfig {
    fn default() -> Self {
        Self {
            http_timeout_secs: 30,
            max_concurrent_requests: 10,
            skip_ssl_verification: false,
            dashboard_base_url: None,
        }
    }
}

/// Trait for executing jobs.
///
/// Each job type should have an implementation of this trait that handles
/// the specific job logic.
#[async_trait]
pub trait JobExecutor: Send + Sync {
    /// Returns the job types this executor can handle.
    fn handles(&self) -> Vec<JobType>;

    /// Executes a job and returns the result.
    ///
    /// # Arguments
    ///
    /// * `job` - The job to execute.
    /// * `ctx` - Execution context with shared resources.
    ///
    /// # Returns
    ///
    /// The job result on success, or an error on failure.
    async fn execute(&self, job: &Job, ctx: &ExecutionContext) -> WorkerResult<JobResult>;

    /// Called before job execution (optional hook).
    async fn before_execute(&self, _job: &Job, _ctx: &ExecutionContext) -> WorkerResult<()> {
        Ok(())
    }

    /// Called after job execution (optional hook).
    async fn after_execute(
        &self,
        _job: &Job,
        _result: &JobResult,
        _ctx: &ExecutionContext,
    ) -> WorkerResult<()> {
        Ok(())
    }

    /// Returns the default timeout for this executor (in seconds).
    fn default_timeout_secs(&self) -> u64 {
        300 // 5 minutes default
    }
}

/// Type alias for a boxed executor.
pub type BoxedExecutor = Arc<dyn JobExecutor>;
