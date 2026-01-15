//! Domain events that can be emitted by the system.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::{AlertId, AlertType, DependencyId, ProjectId, Severity, TenantId};

/// Base trait for all domain events.
pub trait DomainEvent: Send + Sync {
    /// Returns the event type name.
    fn event_type(&self) -> &'static str;

    /// Returns when the event occurred.
    fn occurred_at(&self) -> DateTime<Utc>;

    /// Returns the tenant this event belongs to.
    fn tenant_id(&self) -> TenantId;
}

/// All possible domain events in the system.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event_type", rename_all = "snake_case")]
pub enum Event {
    ProjectCreated(ProjectCreatedEvent),
    ProjectScanned(ProjectScannedEvent),
    DependencyAdded(DependencyAddedEvent),
    DependencyUpdated(DependencyUpdatedEvent),
    DependencyRemoved(DependencyRemovedEvent),
    AlertCreated(AlertCreatedEvent),
    AlertAcknowledged(AlertAcknowledgedEvent),
    AlertResolved(AlertResolvedEvent),
    PolicyViolationDetected(PolicyViolationDetectedEvent),
}

impl DomainEvent for Event {
    fn event_type(&self) -> &'static str {
        match self {
            Self::ProjectCreated(_) => "project_created",
            Self::ProjectScanned(_) => "project_scanned",
            Self::DependencyAdded(_) => "dependency_added",
            Self::DependencyUpdated(_) => "dependency_updated",
            Self::DependencyRemoved(_) => "dependency_removed",
            Self::AlertCreated(_) => "alert_created",
            Self::AlertAcknowledged(_) => "alert_acknowledged",
            Self::AlertResolved(_) => "alert_resolved",
            Self::PolicyViolationDetected(_) => "policy_violation_detected",
        }
    }

    fn occurred_at(&self) -> DateTime<Utc> {
        match self {
            Self::ProjectCreated(e) => e.occurred_at,
            Self::ProjectScanned(e) => e.occurred_at,
            Self::DependencyAdded(e) => e.occurred_at,
            Self::DependencyUpdated(e) => e.occurred_at,
            Self::DependencyRemoved(e) => e.occurred_at,
            Self::AlertCreated(e) => e.occurred_at,
            Self::AlertAcknowledged(e) => e.occurred_at,
            Self::AlertResolved(e) => e.occurred_at,
            Self::PolicyViolationDetected(e) => e.occurred_at,
        }
    }

    fn tenant_id(&self) -> TenantId {
        match self {
            Self::ProjectCreated(e) => e.tenant_id,
            Self::ProjectScanned(e) => e.tenant_id,
            Self::DependencyAdded(e) => e.tenant_id,
            Self::DependencyUpdated(e) => e.tenant_id,
            Self::DependencyRemoved(e) => e.tenant_id,
            Self::AlertCreated(e) => e.tenant_id,
            Self::AlertAcknowledged(e) => e.tenant_id,
            Self::AlertResolved(e) => e.tenant_id,
            Self::PolicyViolationDetected(e) => e.tenant_id,
        }
    }
}

/// Event emitted when a new project is created.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectCreatedEvent {
    pub tenant_id: TenantId,
    pub project_id: ProjectId,
    pub project_name: String,
    pub occurred_at: DateTime<Utc>,
}

/// Event emitted when a project scan completes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectScannedEvent {
    pub tenant_id: TenantId,
    pub project_id: ProjectId,
    pub scan_id: Uuid,
    pub dependencies_found: u32,
    pub alerts_created: u32,
    pub duration_ms: u64,
    pub occurred_at: DateTime<Utc>,
}

/// Event emitted when a new dependency is discovered.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyAddedEvent {
    pub tenant_id: TenantId,
    pub project_id: ProjectId,
    pub dependency_id: DependencyId,
    pub package_name: String,
    pub version: String,
    pub occurred_at: DateTime<Utc>,
}

/// Event emitted when a dependency version changes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyUpdatedEvent {
    pub tenant_id: TenantId,
    pub project_id: ProjectId,
    pub dependency_id: DependencyId,
    pub package_name: String,
    pub old_version: String,
    pub new_version: String,
    pub occurred_at: DateTime<Utc>,
}

/// Event emitted when a dependency is removed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyRemovedEvent {
    pub tenant_id: TenantId,
    pub project_id: ProjectId,
    pub dependency_id: DependencyId,
    pub package_name: String,
    pub occurred_at: DateTime<Utc>,
}

/// Event emitted when a new alert is created.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertCreatedEvent {
    pub tenant_id: TenantId,
    pub project_id: ProjectId,
    pub alert_id: AlertId,
    pub alert_type: AlertType,
    pub severity: Severity,
    pub title: String,
    pub occurred_at: DateTime<Utc>,
}

/// Event emitted when an alert is acknowledged.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertAcknowledgedEvent {
    pub tenant_id: TenantId,
    pub alert_id: AlertId,
    pub acknowledged_by: Uuid,
    pub occurred_at: DateTime<Utc>,
}

/// Event emitted when an alert is resolved.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertResolvedEvent {
    pub tenant_id: TenantId,
    pub alert_id: AlertId,
    pub resolved_by: Uuid,
    pub resolution: String,
    pub occurred_at: DateTime<Utc>,
}

/// Event emitted when a policy violation is detected.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyViolationDetectedEvent {
    pub tenant_id: TenantId,
    pub project_id: ProjectId,
    pub policy_name: String,
    pub rule_type: String,
    pub package_name: String,
    pub severity: Severity,
    pub occurred_at: DateTime<Utc>,
}
