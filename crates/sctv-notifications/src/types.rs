//! Common types for the notification system.

use chrono::{DateTime, Utc};
use sctv_core::Severity;
use serde::{Deserialize, Serialize};

/// A notification message to be delivered through a channel.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    /// Unique identifier for the notification.
    pub id: String,
    /// Alert severity level.
    pub severity: Severity,
    /// Notification title/subject.
    pub title: String,
    /// Main notification message.
    pub message: String,
    /// Additional context for the notification.
    pub context: NotificationContext,
    /// When the notification was created.
    pub created_at: DateTime<Utc>,
}

impl Notification {
    /// Creates a new notification.
    #[must_use]
    pub fn new(severity: Severity, title: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            severity,
            title: title.into(),
            message: message.into(),
            context: NotificationContext::default(),
            created_at: Utc::now(),
        }
    }

    /// Adds context to the notification.
    #[must_use]
    pub fn with_context(mut self, context: NotificationContext) -> Self {
        self.context = context;
        self
    }
}

/// Additional context information for a notification.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NotificationContext {
    /// Project name if applicable.
    pub project_name: Option<String>,
    /// Package name if applicable.
    pub package_name: Option<String>,
    /// Package version if applicable.
    pub package_version: Option<String>,
    /// URL to view more details.
    pub dashboard_url: Option<String>,
    /// Suggested remediation steps.
    pub remediation: Option<String>,
    /// Alert type identifier.
    pub alert_type: Option<String>,
    /// Additional metadata.
    #[serde(default)]
    pub metadata: serde_json::Value,
}

impl NotificationContext {
    /// Creates a new empty context.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the project name.
    #[must_use]
    pub fn with_project(mut self, name: impl Into<String>) -> Self {
        self.project_name = Some(name.into());
        self
    }

    /// Sets the package information.
    #[must_use]
    pub fn with_package(mut self, name: impl Into<String>, version: impl Into<String>) -> Self {
        self.package_name = Some(name.into());
        self.package_version = Some(version.into());
        self
    }

    /// Sets the dashboard URL.
    #[must_use]
    pub fn with_dashboard_url(mut self, url: impl Into<String>) -> Self {
        self.dashboard_url = Some(url.into());
        self
    }

    /// Sets the remediation advice.
    #[must_use]
    pub fn with_remediation(mut self, remediation: impl Into<String>) -> Self {
        self.remediation = Some(remediation.into());
        self
    }
}

/// Result of a notification delivery attempt.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeliveryResult {
    /// Whether the delivery was successful.
    pub success: bool,
    /// Channel-specific response data.
    pub response: Option<serde_json::Value>,
    /// Error message if delivery failed.
    pub error: Option<String>,
    /// Time taken to deliver in milliseconds.
    pub duration_ms: u64,
    /// Timestamp of the delivery attempt.
    pub timestamp: DateTime<Utc>,
}

impl DeliveryResult {
    /// Creates a successful delivery result.
    #[must_use]
    pub fn success(duration_ms: u64) -> Self {
        Self {
            success: true,
            response: None,
            error: None,
            duration_ms,
            timestamp: Utc::now(),
        }
    }

    /// Creates a successful delivery result with response data.
    #[must_use]
    pub fn success_with_response(duration_ms: u64, response: serde_json::Value) -> Self {
        Self {
            success: true,
            response: Some(response),
            error: None,
            duration_ms,
            timestamp: Utc::now(),
        }
    }

    /// Creates a failed delivery result.
    #[must_use]
    pub fn failure(duration_ms: u64, error: impl Into<String>) -> Self {
        Self {
            success: false,
            response: None,
            error: Some(error.into()),
            duration_ms,
            timestamp: Utc::now(),
        }
    }
}
