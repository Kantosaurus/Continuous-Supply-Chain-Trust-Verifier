//! Audit log domain model for security and compliance tracking.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::net::IpAddr;
use uuid::Uuid;

use super::{TenantId, UserId};

/// Unique identifier for an audit log entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AuditLogId(pub Uuid);

impl AuditLogId {
    /// Creates a new random audit log ID.
    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for AuditLogId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for AuditLogId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Category of audit action.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditAction {
    // Authentication actions
    Login,
    Logout,
    LoginFailed,
    ApiKeyCreated,
    ApiKeyRevoked,

    // Resource creation
    Created,
    Updated,
    Deleted,

    // Project actions
    ProjectScanned,
    PolicyApplied,

    // Alert actions
    AlertAcknowledged,
    AlertResolved,
    AlertSuppressed,

    // Admin actions
    UserInvited,
    UserRemoved,
    RoleChanged,
    SettingsUpdated,

    // Export/Import
    DataExported,
    SbomGenerated,
}

impl std::fmt::Display for AuditAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Login => "login",
            Self::Logout => "logout",
            Self::LoginFailed => "login_failed",
            Self::ApiKeyCreated => "api_key_created",
            Self::ApiKeyRevoked => "api_key_revoked",
            Self::Created => "created",
            Self::Updated => "updated",
            Self::Deleted => "deleted",
            Self::ProjectScanned => "project_scanned",
            Self::PolicyApplied => "policy_applied",
            Self::AlertAcknowledged => "alert_acknowledged",
            Self::AlertResolved => "alert_resolved",
            Self::AlertSuppressed => "alert_suppressed",
            Self::UserInvited => "user_invited",
            Self::UserRemoved => "user_removed",
            Self::RoleChanged => "role_changed",
            Self::SettingsUpdated => "settings_updated",
            Self::DataExported => "data_exported",
            Self::SbomGenerated => "sbom_generated",
        };
        write!(f, "{s}")
    }
}

impl std::str::FromStr for AuditAction {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "login" => Ok(Self::Login),
            "logout" => Ok(Self::Logout),
            "login_failed" => Ok(Self::LoginFailed),
            "api_key_created" => Ok(Self::ApiKeyCreated),
            "api_key_revoked" => Ok(Self::ApiKeyRevoked),
            "created" => Ok(Self::Created),
            "updated" => Ok(Self::Updated),
            "deleted" => Ok(Self::Deleted),
            "project_scanned" => Ok(Self::ProjectScanned),
            "policy_applied" => Ok(Self::PolicyApplied),
            "alert_acknowledged" => Ok(Self::AlertAcknowledged),
            "alert_resolved" => Ok(Self::AlertResolved),
            "alert_suppressed" => Ok(Self::AlertSuppressed),
            "user_invited" => Ok(Self::UserInvited),
            "user_removed" => Ok(Self::UserRemoved),
            "role_changed" => Ok(Self::RoleChanged),
            "settings_updated" => Ok(Self::SettingsUpdated),
            "data_exported" => Ok(Self::DataExported),
            "sbom_generated" => Ok(Self::SbomGenerated),
            _ => Err(format!("Unknown audit action: {s}")),
        }
    }
}

/// Type of resource affected by the action.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ResourceType {
    Tenant,
    User,
    Project,
    Policy,
    Alert,
    Sbom,
    ApiKey,
    Settings,
}

impl std::fmt::Display for ResourceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Tenant => write!(f, "tenant"),
            Self::User => write!(f, "user"),
            Self::Project => write!(f, "project"),
            Self::Policy => write!(f, "policy"),
            Self::Alert => write!(f, "alert"),
            Self::Sbom => write!(f, "sbom"),
            Self::ApiKey => write!(f, "api_key"),
            Self::Settings => write!(f, "settings"),
        }
    }
}

impl std::str::FromStr for ResourceType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "tenant" => Ok(Self::Tenant),
            "user" => Ok(Self::User),
            "project" => Ok(Self::Project),
            "policy" => Ok(Self::Policy),
            "alert" => Ok(Self::Alert),
            "sbom" => Ok(Self::Sbom),
            "api_key" => Ok(Self::ApiKey),
            "settings" => Ok(Self::Settings),
            _ => Err(format!("Unknown resource type: {s}")),
        }
    }
}

/// An audit log entry for tracking security-relevant events.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLog {
    pub id: AuditLogId,
    pub tenant_id: TenantId,
    pub user_id: Option<UserId>,
    pub action: AuditAction,
    pub resource_type: ResourceType,
    pub resource_id: Option<Uuid>,
    pub details: serde_json::Value,
    pub ip_address: Option<IpAddr>,
    pub user_agent: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl AuditLog {
    /// Creates a new audit log entry.
    #[must_use]
    pub fn new(
        tenant_id: TenantId,
        user_id: Option<UserId>,
        action: AuditAction,
        resource_type: ResourceType,
    ) -> Self {
        Self {
            id: AuditLogId::new(),
            tenant_id,
            user_id,
            action,
            resource_type,
            resource_id: None,
            details: serde_json::Value::Object(serde_json::Map::new()),
            ip_address: None,
            user_agent: None,
            created_at: Utc::now(),
        }
    }

    /// Sets the resource ID.
    #[must_use]
    pub const fn with_resource_id(mut self, resource_id: Uuid) -> Self {
        self.resource_id = Some(resource_id);
        self
    }

    /// Sets additional details.
    #[must_use]
    pub fn with_details(mut self, details: serde_json::Value) -> Self {
        self.details = details;
        self
    }

    /// Sets the request context (IP and user agent).
    #[must_use]
    pub fn with_request_context(mut self, ip_address: IpAddr, user_agent: String) -> Self {
        self.ip_address = Some(ip_address);
        self.user_agent = Some(user_agent);
        self
    }

    /// Creates a login audit entry.
    #[must_use]
    pub fn login(tenant_id: TenantId, user_id: UserId) -> Self {
        Self::new(
            tenant_id,
            Some(user_id),
            AuditAction::Login,
            ResourceType::User,
        )
        .with_resource_id(user_id.0)
    }

    /// Creates a resource creation audit entry.
    #[must_use]
    pub fn created(
        tenant_id: TenantId,
        user_id: UserId,
        resource_type: ResourceType,
        resource_id: Uuid,
    ) -> Self {
        Self::new(
            tenant_id,
            Some(user_id),
            AuditAction::Created,
            resource_type,
        )
        .with_resource_id(resource_id)
    }

    /// Creates a resource update audit entry.
    #[must_use]
    pub fn updated(
        tenant_id: TenantId,
        user_id: UserId,
        resource_type: ResourceType,
        resource_id: Uuid,
        changes: serde_json::Value,
    ) -> Self {
        Self::new(
            tenant_id,
            Some(user_id),
            AuditAction::Updated,
            resource_type,
        )
        .with_resource_id(resource_id)
        .with_details(changes)
    }

    /// Creates a resource deletion audit entry.
    #[must_use]
    pub fn deleted(
        tenant_id: TenantId,
        user_id: UserId,
        resource_type: ResourceType,
        resource_id: Uuid,
    ) -> Self {
        Self::new(
            tenant_id,
            Some(user_id),
            AuditAction::Deleted,
            resource_type,
        )
        .with_resource_id(resource_id)
    }
}

/// Filter options for querying audit logs.
#[derive(Debug, Clone, Default)]
pub struct AuditLogFilter {
    pub user_id: Option<UserId>,
    pub action: Option<Vec<AuditAction>>,
    pub resource_type: Option<ResourceType>,
    pub resource_id: Option<Uuid>,
    pub from_date: Option<DateTime<Utc>>,
    pub to_date: Option<DateTime<Utc>>,
}
