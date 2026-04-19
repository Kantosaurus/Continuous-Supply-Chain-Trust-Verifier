//! `PagerDuty` notification channel using Events API v2.

use std::time::{Duration, Instant};

use async_trait::async_trait;
use reqwest::Client;
use sctv_core::Severity;
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info, instrument, warn};

use super::NotificationChannel;
use crate::error::{NotificationError, NotificationResult};
use crate::types::{DeliveryResult, Notification};

/// `PagerDuty` Events API v2 endpoint.
const PAGERDUTY_EVENTS_API: &str = "https://events.pagerduty.com/v2/enqueue";

/// Configuration for the `PagerDuty` notification channel.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PagerDutyConfig {
    /// `PagerDuty` routing key (integration key).
    #[serde(skip_serializing)]
    pub routing_key: String,
    /// Source identifier for events (e.g., hostname or service name).
    #[serde(default = "default_source")]
    pub source: String,
    /// Component generating the event.
    pub component: Option<String>,
    /// Logical grouping of components.
    pub group: Option<String>,
    /// Class/type of the event.
    pub class: Option<String>,
    /// Request timeout in seconds.
    #[serde(default = "default_timeout")]
    pub timeout_secs: u64,
    /// Whether the channel is enabled.
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    /// Custom API URL (for testing). Defaults to `PagerDuty` Events API v2.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_url: Option<String>,
}

fn default_source() -> String {
    "supply-chain-trust-verifier".to_string()
}

const fn default_timeout() -> u64 {
    30
}

const fn default_enabled() -> bool {
    true
}

impl Default for PagerDutyConfig {
    fn default() -> Self {
        Self {
            routing_key: String::new(),
            source: default_source(),
            component: None,
            group: None,
            class: None,
            timeout_secs: 30,
            enabled: false,
            api_url: None,
        }
    }
}

impl PagerDutyConfig {
    /// Creates a new `PagerDuty` configuration builder.
    #[must_use]
    pub fn builder() -> PagerDutyConfigBuilder {
        PagerDutyConfigBuilder::default()
    }
}

/// Builder for `PagerDutyConfig`.
#[derive(Debug, Default)]
pub struct PagerDutyConfigBuilder {
    config: PagerDutyConfig,
}

impl PagerDutyConfigBuilder {
    /// Sets the routing key (integration key).
    #[must_use]
    pub fn routing_key(mut self, key: impl Into<String>) -> Self {
        self.config.routing_key = key.into();
        self
    }

    /// Sets the event source.
    #[must_use]
    pub fn source(mut self, source: impl Into<String>) -> Self {
        self.config.source = source.into();
        self
    }

    /// Sets the component.
    #[must_use]
    pub fn component(mut self, component: impl Into<String>) -> Self {
        self.config.component = Some(component.into());
        self
    }

    /// Sets the group.
    #[must_use]
    pub fn group(mut self, group: impl Into<String>) -> Self {
        self.config.group = Some(group.into());
        self
    }

    /// Sets the event class.
    #[must_use]
    pub fn class(mut self, class: impl Into<String>) -> Self {
        self.config.class = Some(class.into());
        self
    }

    /// Sets the request timeout.
    #[must_use]
    pub const fn timeout_secs(mut self, secs: u64) -> Self {
        self.config.timeout_secs = secs;
        self
    }

    /// Sets whether the channel is enabled.
    #[must_use]
    pub const fn enabled(mut self, enabled: bool) -> Self {
        self.config.enabled = enabled;
        self
    }

    /// Sets a custom API URL (for testing).
    #[must_use]
    pub fn api_url(mut self, url: impl Into<String>) -> Self {
        self.config.api_url = Some(url.into());
        self
    }

    /// Builds the configuration.
    #[must_use]
    pub fn build(self) -> PagerDutyConfig {
        self.config
    }
}

/// `PagerDuty` event action type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
#[allow(dead_code)] // Acknowledge and Resolve are part of the API for future use
pub enum EventAction {
    /// Trigger a new alert.
    Trigger,
    /// Acknowledge an existing alert.
    Acknowledge,
    /// Resolve an existing alert.
    Resolve,
}

/// `PagerDuty` event severity.
#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "lowercase")]
enum PagerDutySeverity {
    Critical,
    Error,
    Warning,
    Info,
}

impl From<Severity> for PagerDutySeverity {
    fn from(severity: Severity) -> Self {
        match severity {
            Severity::Critical => Self::Critical,
            Severity::High => Self::Error,
            Severity::Medium => Self::Warning,
            Severity::Low | Severity::Info => Self::Info,
        }
    }
}

/// `PagerDuty` Events API v2 payload.
#[derive(Debug, Serialize)]
struct PagerDutyEvent {
    routing_key: String,
    event_action: EventAction,
    #[serde(skip_serializing_if = "Option::is_none")]
    dedup_key: Option<String>,
    payload: EventPayload,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    links: Vec<EventLink>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    images: Vec<EventImage>,
}

/// Payload section of a `PagerDuty` event.
#[derive(Debug, Serialize)]
struct EventPayload {
    summary: String,
    source: String,
    severity: PagerDutySeverity,
    #[serde(skip_serializing_if = "Option::is_none")]
    timestamp: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    component: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    group: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    class: Option<String>,
    #[serde(skip_serializing_if = "std::collections::HashMap::is_empty")]
    custom_details: std::collections::HashMap<String, serde_json::Value>,
}

/// Link in a `PagerDuty` event.
#[derive(Debug, Serialize)]
struct EventLink {
    href: String,
    text: String,
}

/// Image in a `PagerDuty` event.
#[derive(Debug, Serialize)]
struct EventImage {
    src: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    href: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    alt: Option<String>,
}

/// `PagerDuty` API response.
#[derive(Debug, Deserialize)]
struct PagerDutyResponse {
    status: String,
    message: String,
    #[serde(default)]
    dedup_key: Option<String>,
}

/// `PagerDuty` notification channel using Events API v2.
pub struct PagerDutyChannel {
    config: PagerDutyConfig,
    client: Client,
}

impl PagerDutyChannel {
    /// Creates a new `PagerDuty` channel with the given configuration.
    #[must_use]
    pub fn new(config: PagerDutyConfig) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(config.timeout_secs))
            .build()
            .expect("Failed to create HTTP client");

        Self { config, client }
    }

    /// Builds the `PagerDuty` event payload.
    fn build_event(&self, notification: &Notification, action: EventAction) -> PagerDutyEvent {
        let mut custom_details = std::collections::HashMap::new();

        // Add notification context as custom details
        if let Some(project) = &notification.context.project_name {
            custom_details.insert(
                "project".to_string(),
                serde_json::Value::String(project.clone()),
            );
        }

        if let Some(package) = &notification.context.package_name {
            custom_details.insert(
                "package".to_string(),
                serde_json::Value::String(package.clone()),
            );
        }

        if let Some(version) = &notification.context.package_version {
            custom_details.insert(
                "version".to_string(),
                serde_json::Value::String(version.clone()),
            );
        }

        if let Some(alert_type) = &notification.context.alert_type {
            custom_details.insert(
                "alert_type".to_string(),
                serde_json::Value::String(alert_type.clone()),
            );
        }

        if let Some(remediation) = &notification.context.remediation {
            custom_details.insert(
                "remediation".to_string(),
                serde_json::Value::String(remediation.clone()),
            );
        }

        // Add full message as custom detail
        custom_details.insert(
            "message".to_string(),
            serde_json::Value::String(notification.message.clone()),
        );

        custom_details.insert(
            "notification_id".to_string(),
            serde_json::Value::String(notification.id.clone()),
        );

        // Build links
        let mut links = Vec::new();
        if let Some(url) = &notification.context.dashboard_url {
            links.push(EventLink {
                href: url.clone(),
                text: "View in Dashboard".to_string(),
            });
        }

        // Determine component and class
        let component = self
            .config
            .component
            .clone()
            .or_else(|| notification.context.project_name.clone());

        let class = self
            .config
            .class
            .clone()
            .or_else(|| notification.context.alert_type.clone())
            .unwrap_or_else(|| "supply_chain_alert".to_string());

        PagerDutyEvent {
            routing_key: self.config.routing_key.clone(),
            event_action: action,
            dedup_key: Some(notification.id.clone()),
            payload: EventPayload {
                summary: format!("[{}] {}", notification.severity, notification.title),
                source: self.config.source.clone(),
                severity: notification.severity.into(),
                timestamp: Some(notification.created_at.to_rfc3339()),
                component,
                group: self.config.group.clone(),
                class: Some(class),
                custom_details,
            },
            links,
            images: Vec::new(),
        }
    }

    /// Sends a `PagerDuty` event.
    async fn send_event(
        &self,
        notification: &Notification,
        action: EventAction,
    ) -> NotificationResult<DeliveryResult> {
        let start = Instant::now();

        let event = self.build_event(notification, action);

        let api_url = self
            .config
            .api_url
            .as_deref()
            .unwrap_or(PAGERDUTY_EVENTS_API);

        let response = self.client.post(api_url).json(&event).send().await?;

        let duration_ms = start.elapsed().as_millis() as u64;
        let status = response.status();

        if status.is_success() {
            let pd_response: PagerDutyResponse =
                response.json().await.unwrap_or_else(|_| PagerDutyResponse {
                    status: "success".to_string(),
                    message: "Event accepted".to_string(),
                    dedup_key: None,
                });

            info!(
                notification_id = %notification.id,
                duration_ms,
                dedup_key = ?pd_response.dedup_key,
                "PagerDuty event sent successfully"
            );

            Ok(DeliveryResult::success_with_response(
                duration_ms,
                serde_json::json!({
                    "status": pd_response.status,
                    "message": pd_response.message,
                    "dedup_key": pd_response.dedup_key,
                }),
            ))
        } else if status.as_u16() == 429 {
            warn!(
                notification_id = %notification.id,
                "PagerDuty rate limit exceeded"
            );

            let retry_after = response
                .headers()
                .get("Retry-After")
                .and_then(|h| h.to_str().ok())
                .and_then(|s| s.parse().ok())
                .unwrap_or(60);

            Err(NotificationError::RateLimited {
                retry_after_secs: retry_after,
            })
        } else if status.as_u16() == 400 {
            let body = response.text().await.unwrap_or_default();
            error!(
                notification_id = %notification.id,
                status = %status,
                body = %body,
                "Invalid PagerDuty event"
            );

            Err(NotificationError::InvalidConfig(format!(
                "Invalid event: {body}"
            )))
        } else {
            let body = response.text().await.unwrap_or_default();
            error!(
                notification_id = %notification.id,
                status = %status,
                body = %body,
                "Failed to send PagerDuty event"
            );

            Ok(DeliveryResult::failure(
                duration_ms,
                format!("HTTP {status}: {body}"),
            ))
        }
    }
}

#[async_trait]
impl NotificationChannel for PagerDutyChannel {
    fn name(&self) -> &'static str {
        "pagerduty"
    }

    fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    #[instrument(skip(self, notification), fields(channel = "pagerduty"))]
    async fn send(&self, notification: &Notification) -> NotificationResult<DeliveryResult> {
        if !self.is_enabled() {
            return Err(NotificationError::ChannelDisabled);
        }

        debug!(
            notification_id = %notification.id,
            severity = %notification.severity,
            "Sending PagerDuty notification"
        );

        // Trigger a new alert
        self.send_event(notification, EventAction::Trigger).await
    }

    async fn validate(&self) -> NotificationResult<()> {
        if self.config.routing_key.is_empty() {
            return Err(NotificationError::InvalidConfig(
                "Routing key is required".to_string(),
            ));
        }

        // PagerDuty routing keys are 32 characters
        if self.config.routing_key.len() != 32 {
            warn!(
                "PagerDuty routing key length ({}) differs from expected (32)",
                self.config.routing_key.len()
            );
        }

        if self.config.source.is_empty() {
            return Err(NotificationError::InvalidConfig(
                "Event source is required".to_string(),
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::NotificationContext;

    #[test]
    fn test_pagerduty_config_builder() {
        let config = PagerDutyConfig::builder()
            .routing_key("12345678901234567890123456789012")
            .source("my-service")
            .component("api")
            .group("production")
            .class("security-alert")
            .timeout_secs(60)
            .enabled(true)
            .build();

        assert_eq!(config.routing_key.len(), 32);
        assert_eq!(config.source, "my-service");
        assert_eq!(config.component, Some("api".to_string()));
        assert_eq!(config.group, Some("production".to_string()));
        assert!(config.enabled);
    }

    #[test]
    fn test_build_event() {
        let config = PagerDutyConfig::builder()
            .routing_key("12345678901234567890123456789012")
            .source("sctv")
            .enabled(true)
            .build();

        let channel = PagerDutyChannel::new(config);

        let notification = Notification::new(
            Severity::Critical,
            "Critical Vulnerability",
            "A critical supply chain vulnerability was detected.",
        )
        .with_context(
            NotificationContext::new()
                .with_project("api-server")
                .with_package("lodash", "4.17.20")
                .with_dashboard_url("https://sctv.example.com/alerts/123"),
        );

        let event = channel.build_event(&notification, EventAction::Trigger);

        assert_eq!(event.event_action, EventAction::Trigger);
        assert!(event.payload.summary.contains("Critical"));
        assert_eq!(event.payload.source, "sctv");
        assert!(matches!(
            event.payload.severity,
            PagerDutySeverity::Critical
        ));
        assert!(!event.links.is_empty());
        assert!(event.payload.custom_details.contains_key("project"));
    }

    #[test]
    fn test_severity_mapping() {
        assert!(matches!(
            PagerDutySeverity::from(Severity::Critical),
            PagerDutySeverity::Critical
        ));
        assert!(matches!(
            PagerDutySeverity::from(Severity::High),
            PagerDutySeverity::Error
        ));
        assert!(matches!(
            PagerDutySeverity::from(Severity::Medium),
            PagerDutySeverity::Warning
        ));
        assert!(matches!(
            PagerDutySeverity::from(Severity::Low),
            PagerDutySeverity::Info
        ));
        assert!(matches!(
            PagerDutySeverity::from(Severity::Info),
            PagerDutySeverity::Info
        ));
    }

    #[test]
    fn test_event_action_serialization() {
        let trigger = serde_json::to_string(&EventAction::Trigger).unwrap();
        let acknowledge = serde_json::to_string(&EventAction::Acknowledge).unwrap();
        let resolve = serde_json::to_string(&EventAction::Resolve).unwrap();

        assert_eq!(trigger, "\"trigger\"");
        assert_eq!(acknowledge, "\"acknowledge\"");
        assert_eq!(resolve, "\"resolve\"");
    }
}
