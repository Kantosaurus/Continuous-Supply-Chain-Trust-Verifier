//! Tenant domain model for multi-tenant isolation.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::PackageEcosystem;

/// Unique identifier for a tenant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TenantId(pub Uuid);

impl TenantId {
    /// Creates a new random tenant ID.
    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Creates a tenant ID from a slug (for subdomain-based routing).
    #[must_use]
    pub fn from_slug(_slug: &str) -> Option<Self> {
        // In a real implementation, this would look up the tenant by slug
        None
    }
}

impl Default for TenantId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for TenantId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A tenant organization in the multi-tenant system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tenant {
    pub id: TenantId,
    pub name: String,
    pub slug: String,
    pub plan: TenantPlan,
    pub settings: TenantSettings,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Tenant {
    /// Creates a new tenant with the given name.
    #[must_use]
    pub fn new(name: String, slug: String) -> Self {
        let now = Utc::now();
        Self {
            id: TenantId::new(),
            name,
            slug,
            plan: TenantPlan::Free { project_limit: 5 },
            settings: TenantSettings::default(),
            created_at: now,
            updated_at: now,
        }
    }

    /// Checks if the tenant is active and can use the service.
    #[must_use]
    pub const fn is_active(&self) -> bool {
        true // Could check for suspended status, expired trials, etc.
    }

    /// Returns the maximum number of projects allowed for this tenant.
    #[must_use]
    pub const fn project_limit(&self) -> u32 {
        match &self.plan {
            TenantPlan::Free { project_limit }
            | TenantPlan::Team { project_limit, .. }
            | TenantPlan::Enterprise { project_limit, .. } => *project_limit,
        }
    }
}

/// Subscription plan for a tenant.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum TenantPlan {
    Free {
        project_limit: u32,
    },
    Team {
        project_limit: u32,
        members_limit: u32,
    },
    Enterprise {
        project_limit: u32,
        custom_integrations: bool,
    },
}

/// Tenant-specific settings and preferences.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TenantSettings {
    /// Default policy ID to apply to new projects.
    pub default_policy_id: Option<Uuid>,

    /// Notification channel configurations.
    pub notification_channels: Vec<NotificationChannelConfig>,

    /// Package ecosystems enabled for this tenant.
    pub allowed_ecosystems: Vec<PackageEcosystem>,

    /// Whether to enable continuous monitoring.
    pub continuous_monitoring: bool,

    /// Webhook secret for GitHub/GitLab integrations.
    pub webhook_secret: Option<String>,
}

/// Configuration for a notification channel.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationChannelConfig {
    pub channel_type: NotificationChannelType,
    pub enabled: bool,
    pub min_severity: super::Severity,
    pub config: serde_json::Value,
}

/// Types of notification channels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NotificationChannelType {
    Email,
    Slack,
    Teams,
    Webhook,
    PagerDuty,
}
