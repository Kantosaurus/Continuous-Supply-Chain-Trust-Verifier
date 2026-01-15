//! Error types for the worker crate.

use thiserror::Error;

/// Errors that can occur during worker operations.
#[derive(Debug, Error)]
pub enum WorkerError {
    /// Database error.
    #[error("Database error: {0}")]
    Database(String),

    /// Job not found.
    #[error("Job not found: {0}")]
    JobNotFound(String),

    /// Invalid job status.
    #[error("Invalid job status: {0}")]
    InvalidJobStatus(String),

    /// Invalid job type.
    #[error("Invalid job type: {0}")]
    InvalidJobType(String),

    /// Invalid job priority.
    #[error("Invalid job priority: {0}")]
    InvalidJobPriority(i32),

    /// Serialization error.
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Job execution error.
    #[error("Job execution error: {0}")]
    Execution(String),

    /// Worker pool error.
    #[error("Worker pool error: {0}")]
    Pool(String),

    /// Queue is shutting down.
    #[error("Queue is shutting down")]
    Shutdown,

    /// Timeout waiting for a job.
    #[error("Timeout waiting for job")]
    Timeout,

    /// Configuration error.
    #[error("Configuration error: {0}")]
    Configuration(String),

    /// Registry client error.
    #[error("Registry client error: {0}")]
    Registry(String),

    /// Detector error.
    #[error("Detector error: {0}")]
    Detector(String),

    /// Notification error.
    #[error("Notification error: {0}")]
    Notification(String),
}

impl From<sqlx::Error> for WorkerError {
    fn from(err: sqlx::Error) -> Self {
        Self::Database(err.to_string())
    }
}

impl From<serde_json::Error> for WorkerError {
    fn from(err: serde_json::Error) -> Self {
        Self::Serialization(err.to_string())
    }
}

/// Result type for worker operations.
pub type WorkerResult<T> = Result<T, WorkerError>;
