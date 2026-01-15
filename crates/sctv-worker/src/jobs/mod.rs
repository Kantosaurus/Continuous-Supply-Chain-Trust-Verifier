//! Job type definitions for background processing.
//!
//! This module defines the different types of jobs that can be queued and processed
//! by the worker pool. Each job type has its own payload structure and processing logic.

mod payloads;

pub use payloads::*;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::WorkerError;

/// Unique identifier for a job.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct JobId(pub Uuid);

impl JobId {
    /// Creates a new random job ID.
    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for JobId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for JobId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Status of a job in the queue.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum JobStatus {
    /// Job is waiting to be processed.
    Pending,
    /// Job is currently being processed by a worker.
    Running,
    /// Job completed successfully.
    Completed,
    /// Job failed after all retry attempts.
    Failed,
    /// Job was cancelled before completion.
    Cancelled,
    /// Job is scheduled for a future time.
    Scheduled,
}

impl JobStatus {
    /// Returns the database string representation.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Running => "running",
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
            Self::Scheduled => "scheduled",
        }
    }

    /// Parses from database string.
    pub fn from_str(s: &str) -> Result<Self, WorkerError> {
        match s {
            "pending" => Ok(Self::Pending),
            "running" => Ok(Self::Running),
            "completed" => Ok(Self::Completed),
            "failed" => Ok(Self::Failed),
            "cancelled" => Ok(Self::Cancelled),
            "scheduled" => Ok(Self::Scheduled),
            _ => Err(WorkerError::InvalidJobStatus(s.to_string())),
        }
    }
}

/// Type of job to be executed.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JobType {
    /// Scan a project's dependencies for threats.
    ScanProject,
    /// Monitor a registry for changes to watched packages.
    MonitorRegistry,
    /// Verify provenance attestations for a package.
    VerifyProvenance,
    /// Send a notification about an alert.
    SendNotification,
}

impl JobType {
    /// Returns the database string representation.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ScanProject => "scan_project",
            Self::MonitorRegistry => "monitor_registry",
            Self::VerifyProvenance => "verify_provenance",
            Self::SendNotification => "send_notification",
        }
    }

    /// Parses from database string.
    pub fn from_str(s: &str) -> Result<Self, WorkerError> {
        match s {
            "scan_project" => Ok(Self::ScanProject),
            "monitor_registry" => Ok(Self::MonitorRegistry),
            "verify_provenance" => Ok(Self::VerifyProvenance),
            "send_notification" => Ok(Self::SendNotification),
            _ => Err(WorkerError::InvalidJobType(s.to_string())),
        }
    }
}

/// The payload for a job, containing type-specific data.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum JobPayload {
    /// Payload for scanning a project.
    ScanProject(ScanProjectPayload),
    /// Payload for monitoring a registry.
    MonitorRegistry(MonitorRegistryPayload),
    /// Payload for verifying provenance.
    VerifyProvenance(VerifyProvenancePayload),
    /// Payload for sending a notification.
    SendNotification(SendNotificationPayload),
}

impl JobPayload {
    /// Returns the job type for this payload.
    #[must_use]
    pub fn job_type(&self) -> JobType {
        match self {
            Self::ScanProject(_) => JobType::ScanProject,
            Self::MonitorRegistry(_) => JobType::MonitorRegistry,
            Self::VerifyProvenance(_) => JobType::VerifyProvenance,
            Self::SendNotification(_) => JobType::SendNotification,
        }
    }
}

/// Result of job execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum JobResult {
    /// Result of a scan project job.
    ScanProject(ScanProjectResult),
    /// Result of a monitor registry job.
    MonitorRegistry(MonitorRegistryResult),
    /// Result of a verify provenance job.
    VerifyProvenance(VerifyProvenanceResult),
    /// Result of a send notification job.
    SendNotification(SendNotificationResult),
}

/// Priority levels for jobs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum JobPriority {
    /// Low priority - processed when no higher priority jobs exist.
    Low = 0,
    /// Normal priority - default for most jobs.
    Normal = 5,
    /// High priority - processed before normal priority.
    High = 10,
    /// Critical priority - processed immediately.
    Critical = 15,
}

impl Default for JobPriority {
    fn default() -> Self {
        Self::Normal
    }
}

impl From<JobPriority> for i32 {
    fn from(priority: JobPriority) -> Self {
        priority as i32
    }
}

impl TryFrom<i32> for JobPriority {
    type Error = WorkerError;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Low),
            5 => Ok(Self::Normal),
            10 => Ok(Self::High),
            15 => Ok(Self::Critical),
            _ => Err(WorkerError::InvalidJobPriority(value)),
        }
    }
}

/// A job record representing work to be done.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    /// Unique identifier for this job.
    pub id: JobId,
    /// Tenant that owns this job (if tenant-specific).
    pub tenant_id: Option<sctv_core::TenantId>,
    /// Type of job.
    pub job_type: JobType,
    /// Current status of the job.
    pub status: JobStatus,
    /// Priority level.
    pub priority: JobPriority,
    /// Job-specific payload data.
    pub payload: JobPayload,
    /// Result of job execution (if completed).
    pub result: Option<JobResult>,
    /// Error message if job failed.
    pub error_message: Option<String>,
    /// Number of execution attempts.
    pub attempts: u32,
    /// Maximum number of retry attempts.
    pub max_attempts: u32,
    /// When the job should be executed.
    pub scheduled_at: DateTime<Utc>,
    /// When execution started.
    pub started_at: Option<DateTime<Utc>>,
    /// When execution completed.
    pub completed_at: Option<DateTime<Utc>>,
    /// When the job was created.
    pub created_at: DateTime<Utc>,
}

impl Job {
    /// Creates a new job with the given payload.
    #[must_use]
    pub fn new(payload: JobPayload) -> Self {
        let now = Utc::now();
        Self {
            id: JobId::new(),
            tenant_id: None,
            job_type: payload.job_type(),
            status: JobStatus::Pending,
            priority: JobPriority::Normal,
            payload,
            result: None,
            error_message: None,
            attempts: 0,
            max_attempts: 3,
            scheduled_at: now,
            started_at: None,
            completed_at: None,
            created_at: now,
        }
    }

    /// Sets the tenant ID for this job.
    #[must_use]
    pub fn with_tenant(mut self, tenant_id: sctv_core::TenantId) -> Self {
        self.tenant_id = Some(tenant_id);
        self
    }

    /// Sets the priority for this job.
    #[must_use]
    pub fn with_priority(mut self, priority: JobPriority) -> Self {
        self.priority = priority;
        self
    }

    /// Sets the maximum retry attempts.
    #[must_use]
    pub fn with_max_attempts(mut self, max: u32) -> Self {
        self.max_attempts = max;
        self
    }

    /// Schedules the job for a specific time.
    #[must_use]
    pub fn scheduled_for(mut self, time: DateTime<Utc>) -> Self {
        self.scheduled_at = time;
        self.status = JobStatus::Scheduled;
        self
    }

    /// Checks if the job can be retried.
    #[must_use]
    pub fn can_retry(&self) -> bool {
        self.attempts < self.max_attempts
    }

    /// Marks the job as started.
    pub fn mark_started(&mut self) {
        self.status = JobStatus::Running;
        self.started_at = Some(Utc::now());
        self.attempts += 1;
    }

    /// Marks the job as completed with a result.
    pub fn mark_completed(&mut self, result: JobResult) {
        self.status = JobStatus::Completed;
        self.completed_at = Some(Utc::now());
        self.result = Some(result);
    }

    /// Marks the job as failed with an error message.
    pub fn mark_failed(&mut self, error: String) {
        self.status = if self.can_retry() {
            JobStatus::Pending
        } else {
            JobStatus::Failed
        };
        self.completed_at = Some(Utc::now());
        self.error_message = Some(error);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sctv_core::{PackageEcosystem, ProjectId, TenantId};

    #[test]
    fn test_job_creation() {
        let payload = JobPayload::ScanProject(ScanProjectPayload {
            project_id: ProjectId::new(),
            tenant_id: TenantId::new(),
            ecosystems: vec![PackageEcosystem::Npm],
            full_scan: true,
        });

        let job = Job::new(payload);

        assert_eq!(job.status, JobStatus::Pending);
        assert_eq!(job.job_type, JobType::ScanProject);
        assert_eq!(job.attempts, 0);
        assert_eq!(job.max_attempts, 3);
    }

    #[test]
    fn test_job_builder_pattern() {
        let payload = JobPayload::ScanProject(ScanProjectPayload {
            project_id: ProjectId::new(),
            tenant_id: TenantId::new(),
            ecosystems: vec![PackageEcosystem::Npm],
            full_scan: true,
        });

        let tenant_id = TenantId::new();
        let job = Job::new(payload)
            .with_tenant(tenant_id)
            .with_priority(JobPriority::High)
            .with_max_attempts(5);

        assert_eq!(job.tenant_id, Some(tenant_id));
        assert_eq!(job.priority, JobPriority::High);
        assert_eq!(job.max_attempts, 5);
    }

    #[test]
    fn test_job_status_transitions() {
        let payload = JobPayload::ScanProject(ScanProjectPayload {
            project_id: ProjectId::new(),
            tenant_id: TenantId::new(),
            ecosystems: vec![],
            full_scan: false,
        });

        let mut job = Job::new(payload);
        assert_eq!(job.status, JobStatus::Pending);
        assert!(job.can_retry());

        job.mark_started();
        assert_eq!(job.status, JobStatus::Running);
        assert_eq!(job.attempts, 1);
        assert!(job.started_at.is_some());

        let result = JobResult::ScanProject(ScanProjectResult {
            dependencies_found: 10,
            alerts_created: 2,
            scan_duration_ms: 1500,
        });
        job.mark_completed(result);
        assert_eq!(job.status, JobStatus::Completed);
        assert!(job.completed_at.is_some());
        assert!(job.result.is_some());
    }

    #[test]
    fn test_job_retry_logic() {
        let payload = JobPayload::ScanProject(ScanProjectPayload {
            project_id: ProjectId::new(),
            tenant_id: TenantId::new(),
            ecosystems: vec![],
            full_scan: false,
        });

        let mut job = Job::new(payload).with_max_attempts(2);

        // First failure - should retry
        job.mark_started();
        job.mark_failed("Error 1".to_string());
        assert_eq!(job.status, JobStatus::Pending);
        assert!(job.can_retry());

        // Second failure - no more retries
        job.mark_started();
        job.mark_failed("Error 2".to_string());
        assert_eq!(job.status, JobStatus::Failed);
        assert!(!job.can_retry());
    }
}
