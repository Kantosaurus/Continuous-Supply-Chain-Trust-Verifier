//! Background job domain model for async task processing.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{ProjectId, TenantId};

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

/// Type of background job to execute.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum JobType {
    /// Scan a project for dependencies and threats.
    ScanProject { project_id: Uuid },
    /// Monitor a package registry for changes.
    MonitorRegistry {
        ecosystem: String,
        package_names: Vec<String>,
    },
    /// Verify provenance attestations for a package.
    VerifyProvenance {
        ecosystem: String,
        package_name: String,
        version: String,
    },
    /// Send a notification alert.
    SendNotification {
        alert_id: Uuid,
        channel_type: String,
    },
    /// Generate an SBOM for a project.
    GenerateSbom { project_id: Uuid, format: String },
    /// Cleanup old scan data.
    Cleanup { older_than_days: u32 },
}

impl JobType {
    /// Returns the type name as a string.
    #[must_use]
    pub const fn type_name(&self) -> &'static str {
        match self {
            Self::ScanProject { .. } => "scan_project",
            Self::MonitorRegistry { .. } => "monitor_registry",
            Self::VerifyProvenance { .. } => "verify_provenance",
            Self::SendNotification { .. } => "send_notification",
            Self::GenerateSbom { .. } => "generate_sbom",
            Self::Cleanup { .. } => "cleanup",
        }
    }
}

/// Status of a background job.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum JobStatus {
    /// Job is waiting to be processed.
    Pending,
    /// Job is currently being executed.
    Running,
    /// Job completed successfully.
    Completed,
    /// Job failed after exhausting retries.
    Failed,
    /// Job was cancelled.
    Cancelled,
    /// Job is scheduled for future execution.
    Scheduled,
}

impl Default for JobStatus {
    fn default() -> Self {
        Self::Pending
    }
}

impl std::fmt::Display for JobStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pending => write!(f, "pending"),
            Self::Running => write!(f, "running"),
            Self::Completed => write!(f, "completed"),
            Self::Failed => write!(f, "failed"),
            Self::Cancelled => write!(f, "cancelled"),
            Self::Scheduled => write!(f, "scheduled"),
        }
    }
}

impl std::str::FromStr for JobStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "pending" => Ok(Self::Pending),
            "running" => Ok(Self::Running),
            "completed" => Ok(Self::Completed),
            "failed" => Ok(Self::Failed),
            "cancelled" => Ok(Self::Cancelled),
            "scheduled" => Ok(Self::Scheduled),
            _ => Err(format!("Unknown job status: {}", s)),
        }
    }
}

/// Priority levels for job scheduling.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum JobPriority {
    /// Low priority, processed when queue is idle.
    Low = 0,
    /// Normal priority for regular operations.
    Normal = 5,
    /// High priority for time-sensitive tasks.
    High = 10,
    /// Critical priority for security alerts.
    Critical = 15,
}

impl Default for JobPriority {
    fn default() -> Self {
        Self::Normal
    }
}

impl From<i32> for JobPriority {
    fn from(value: i32) -> Self {
        match value {
            0..=2 => Self::Low,
            3..=7 => Self::Normal,
            8..=12 => Self::High,
            _ => Self::Critical,
        }
    }
}

impl From<JobPriority> for i32 {
    fn from(priority: JobPriority) -> Self {
        priority as i32
    }
}

/// A background job for async processing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    pub id: JobId,
    pub tenant_id: Option<TenantId>,
    pub job_type: JobType,
    pub status: JobStatus,
    pub priority: JobPriority,
    pub payload: serde_json::Value,
    pub result: Option<serde_json::Value>,
    pub error_message: Option<String>,
    pub attempts: u32,
    pub max_attempts: u32,
    pub scheduled_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

impl Job {
    /// Creates a new job with the given type.
    #[must_use]
    pub fn new(tenant_id: Option<TenantId>, job_type: JobType) -> Self {
        let now = Utc::now();
        let payload = serde_json::to_value(&job_type).unwrap_or_default();
        Self {
            id: JobId::new(),
            tenant_id,
            job_type,
            status: JobStatus::Pending,
            priority: JobPriority::default(),
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

    /// Creates a job with a specific priority.
    #[must_use]
    pub fn with_priority(mut self, priority: JobPriority) -> Self {
        self.priority = priority;
        self
    }

    /// Creates a job scheduled for future execution.
    #[must_use]
    pub fn scheduled_for(mut self, scheduled_at: DateTime<Utc>) -> Self {
        self.scheduled_at = scheduled_at;
        self.status = JobStatus::Scheduled;
        self
    }

    /// Checks if the job can be retried.
    #[must_use]
    pub const fn can_retry(&self) -> bool {
        self.attempts < self.max_attempts
    }

    /// Checks if the job is in a terminal state.
    #[must_use]
    pub const fn is_terminal(&self) -> bool {
        matches!(
            self.status,
            JobStatus::Completed | JobStatus::Failed | JobStatus::Cancelled
        )
    }

    /// Marks the job as started.
    pub fn mark_started(&mut self) {
        self.status = JobStatus::Running;
        self.started_at = Some(Utc::now());
        self.attempts += 1;
    }

    /// Marks the job as completed with a result.
    pub fn mark_completed(&mut self, result: serde_json::Value) {
        self.status = JobStatus::Completed;
        self.completed_at = Some(Utc::now());
        self.result = Some(result);
    }

    /// Marks the job as failed with an error message.
    pub fn mark_failed(&mut self, error: String) {
        self.status = JobStatus::Failed;
        self.completed_at = Some(Utc::now());
        self.error_message = Some(error);
    }

    /// Creates a scan project job.
    #[must_use]
    pub fn scan_project(tenant_id: TenantId, project_id: ProjectId) -> Self {
        Self::new(
            Some(tenant_id),
            JobType::ScanProject {
                project_id: project_id.0,
            },
        )
        .with_priority(JobPriority::Normal)
    }
}
