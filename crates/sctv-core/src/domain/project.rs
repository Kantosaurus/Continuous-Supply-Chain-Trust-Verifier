//! Project domain model representing a monitored repository.

use chrono::{DateTime, Datelike, Timelike, Utc, Weekday};
use serde::{Deserialize, Serialize};
use url::Url;
use uuid::Uuid;

#[cfg(feature = "graphql")]
use async_graphql::Enum;

use super::{PackageEcosystem, PolicyId, TenantId};

/// Unique identifier for a project.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ProjectId(pub Uuid);

impl ProjectId {
    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for ProjectId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for ProjectId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A project representing a monitored repository or application.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: ProjectId,
    pub tenant_id: TenantId,
    pub name: String,
    pub description: Option<String>,
    pub repository_url: Option<Url>,
    pub default_branch: String,
    pub ecosystems: Vec<PackageEcosystem>,
    pub scan_schedule: ScanSchedule,
    pub policy_id: Option<PolicyId>,
    pub last_scan_at: Option<DateTime<Utc>>,
    pub status: ProjectStatus,
    pub metadata: ProjectMetadata,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Project {
    /// Creates a new project with the given name.
    #[must_use]
    pub fn new(tenant_id: TenantId, name: String) -> Self {
        let now = Utc::now();
        Self {
            id: ProjectId::new(),
            tenant_id,
            name,
            description: None,
            repository_url: None,
            default_branch: "main".to_string(),
            ecosystems: Vec::new(),
            scan_schedule: ScanSchedule::Daily { hour: 2 },
            policy_id: None,
            last_scan_at: None,
            status: ProjectStatus::Unknown,
            metadata: ProjectMetadata::default(),
            created_at: now,
            updated_at: now,
        }
    }

    /// Checks if the project should be scanned based on its schedule.
    #[must_use]
    pub fn should_scan_now(&self) -> bool {
        match &self.scan_schedule {
            ScanSchedule::Manual => false,
            ScanSchedule::Hourly => {
                self.last_scan_at
                    .map_or(true, |last| Utc::now() - last >= chrono::Duration::hours(1))
            }
            ScanSchedule::Daily { hour } => {
                let now = Utc::now();
                let current_hour = now.hour();
                if current_hour != u32::from(*hour) {
                    return false;
                }
                self.last_scan_at
                    .map_or(true, |last| Utc::now() - last >= chrono::Duration::hours(20))
            }
            ScanSchedule::Weekly { day, hour } => {
                let now = Utc::now();
                let current_day = now.weekday();
                let current_hour = now.hour();
                if current_day != *day || current_hour != u32::from(*hour) {
                    return false;
                }
                self.last_scan_at
                    .map_or(true, |last| Utc::now() - last >= chrono::Duration::days(6))
            }
            ScanSchedule::OnPush => false, // Triggered externally via webhook
        }
    }

    /// Updates the project status based on alerts.
    pub fn update_status(&mut self, critical_count: u32, high_count: u32, medium_count: u32) {
        self.status = if critical_count > 0 {
            ProjectStatus::Critical
        } else if high_count > 0 {
            ProjectStatus::Warning
        } else if medium_count > 0 {
            ProjectStatus::Warning
        } else {
            ProjectStatus::Healthy
        };
        self.updated_at = Utc::now();
    }
}

/// Schedule for automatic project scanning.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum ScanSchedule {
    /// Manual scans only, no automatic scheduling.
    Manual,
    /// Scan every hour.
    Hourly,
    /// Scan once per day at the specified hour (UTC).
    Daily { hour: u8 },
    /// Scan once per week on the specified day and hour (UTC).
    Weekly { day: Weekday, hour: u8 },
    /// Scan on push events (webhook-triggered).
    OnPush,
}

/// Current health status of a project.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "graphql", derive(Enum))]
#[serde(rename_all = "lowercase")]
pub enum ProjectStatus {
    /// No critical or high severity alerts.
    Healthy,
    /// Has medium or high severity alerts.
    Warning,
    /// Has critical severity alerts.
    Critical,
    /// Status not yet determined.
    Unknown,
}

/// Additional metadata for a project.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProjectMetadata {
    /// Number of direct dependencies.
    pub direct_dependency_count: u32,
    /// Total number of dependencies (including transitive).
    pub total_dependency_count: u32,
    /// Number of open alerts.
    pub open_alert_count: u32,
    /// SLSA level of the project's supply chain.
    pub slsa_level: Option<u8>,
    /// Tags for categorization.
    pub tags: Vec<String>,
}
