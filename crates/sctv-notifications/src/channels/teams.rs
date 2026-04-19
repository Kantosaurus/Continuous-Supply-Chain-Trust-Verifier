//! Microsoft Teams notification channel using webhooks.

use std::time::{Duration, Instant};

use async_trait::async_trait;
use reqwest::Client;
use sctv_core::Severity;
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info, instrument, warn};
use url::Url;

use super::NotificationChannel;
use crate::error::{NotificationError, NotificationResult};
use crate::types::{DeliveryResult, Notification};

/// Configuration for the Microsoft Teams notification channel.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamsConfig {
    /// Teams incoming webhook URL.
    #[serde(skip_serializing)]
    pub webhook_url: String,
    /// Request timeout in seconds.
    #[serde(default = "default_timeout")]
    pub timeout_secs: u64,
    /// Whether the channel is enabled.
    #[serde(default = "default_enabled")]
    pub enabled: bool,
}

const fn default_timeout() -> u64 {
    30
}

const fn default_enabled() -> bool {
    true
}

impl Default for TeamsConfig {
    fn default() -> Self {
        Self {
            webhook_url: String::new(),
            timeout_secs: 30,
            enabled: false,
        }
    }
}

impl TeamsConfig {
    /// Creates a new Teams configuration builder.
    #[must_use]
    pub fn builder() -> TeamsConfigBuilder {
        TeamsConfigBuilder::default()
    }
}

/// Builder for `TeamsConfig`.
#[derive(Debug, Default)]
pub struct TeamsConfigBuilder {
    config: TeamsConfig,
}

impl TeamsConfigBuilder {
    /// Sets the webhook URL.
    #[must_use]
    pub fn webhook_url(mut self, url: impl Into<String>) -> Self {
        self.config.webhook_url = url.into();
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

    /// Builds the configuration.
    #[must_use]
    pub fn build(self) -> TeamsConfig {
        self.config
    }
}

/// Teams Adaptive Card message payload.
#[derive(Debug, Serialize)]
struct TeamsMessage {
    #[serde(rename = "type")]
    message_type: &'static str,
    attachments: Vec<TeamsAttachment>,
}

/// Teams attachment containing an Adaptive Card.
#[derive(Debug, Serialize)]
struct TeamsAttachment {
    #[serde(rename = "contentType")]
    content_type: &'static str,
    content: AdaptiveCard,
}

/// Adaptive Card structure for Teams.
#[derive(Debug, Serialize)]
struct AdaptiveCard {
    #[serde(rename = "$schema")]
    schema: &'static str,
    #[serde(rename = "type")]
    card_type: &'static str,
    version: &'static str,
    body: Vec<CardElement>,
    actions: Vec<CardAction>,
}

/// Element in an Adaptive Card body.
#[derive(Debug, Serialize)]
#[serde(tag = "type")]
enum CardElement {
    TextBlock {
        text: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        size: Option<&'static str>,
        #[serde(skip_serializing_if = "Option::is_none")]
        weight: Option<&'static str>,
        #[serde(skip_serializing_if = "Option::is_none")]
        color: Option<&'static str>,
        #[serde(skip_serializing_if = "Option::is_none")]
        wrap: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        spacing: Option<&'static str>,
    },
    FactSet {
        facts: Vec<Fact>,
    },
    Container {
        items: Vec<Self>,
        #[serde(skip_serializing_if = "Option::is_none")]
        style: Option<&'static str>,
    },
}

/// Fact in a `FactSet`.
#[derive(Debug, Serialize)]
struct Fact {
    title: String,
    value: String,
}

/// Action button in an Adaptive Card.
#[derive(Debug, Serialize)]
#[serde(tag = "type")]
enum CardAction {
    #[serde(rename = "Action.OpenUrl")]
    OpenUrl { title: String, url: String },
}

/// Microsoft Teams notification channel using webhooks.
pub struct TeamsChannel {
    config: TeamsConfig,
    client: Client,
}

impl TeamsChannel {
    /// Creates a new Teams channel with the given configuration.
    #[must_use]
    pub fn new(config: TeamsConfig) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(config.timeout_secs))
            .build()
            .expect("Failed to create HTTP client");

        Self { config, client }
    }

    /// Returns the color for the given severity level.
    const fn severity_color(severity: Severity) -> &'static str {
        match severity {
            Severity::Critical => "attention",
            Severity::High => "warning",
            Severity::Medium => "warning",
            Severity::Low => "accent",
            Severity::Info => "default",
        }
    }

    /// Returns the emoji indicator for the given severity level.
    const fn severity_indicator(severity: Severity) -> &'static str {
        match severity {
            Severity::Critical => "\u{1F6A8}",    // 🚨
            Severity::High => "\u{26A0}\u{FE0F}", // ⚠️
            Severity::Medium => "\u{1F536}",      // 🔶
            Severity::Low => "\u{2139}\u{FE0F}",  // ℹ️
            Severity::Info => "\u{1F4AC}",        // 💬
        }
    }

    /// Builds the Teams Adaptive Card message.
    fn build_message(&self, notification: &Notification) -> TeamsMessage {
        let mut body = Vec::new();

        // Header with severity indicator
        body.push(CardElement::TextBlock {
            text: format!(
                "{} {} - {}",
                Self::severity_indicator(notification.severity),
                notification.severity,
                notification.title
            ),
            size: Some("Large"),
            weight: Some("Bolder"),
            color: Some(Self::severity_color(notification.severity)),
            wrap: Some(true),
            spacing: None,
        });

        // Main message
        body.push(CardElement::TextBlock {
            text: notification.message.clone(),
            size: None,
            weight: None,
            color: None,
            wrap: Some(true),
            spacing: Some("Medium"),
        });

        // Facts section
        let mut facts = Vec::new();

        facts.push(Fact {
            title: "Severity".to_string(),
            value: notification.severity.to_string(),
        });

        facts.push(Fact {
            title: "Time".to_string(),
            value: notification
                .created_at
                .format("%Y-%m-%d %H:%M:%S UTC")
                .to_string(),
        });

        if let Some(project) = &notification.context.project_name {
            facts.push(Fact {
                title: "Project".to_string(),
                value: project.clone(),
            });
        }

        if let Some(package) = &notification.context.package_name {
            let mut value = package.clone();
            if let Some(version) = &notification.context.package_version {
                value.push('@');
                value.push_str(version);
            }
            facts.push(Fact {
                title: "Package".to_string(),
                value,
            });
        }

        if let Some(alert_type) = &notification.context.alert_type {
            facts.push(Fact {
                title: "Alert Type".to_string(),
                value: alert_type.clone(),
            });
        }

        body.push(CardElement::FactSet { facts });

        // Remediation if present
        if let Some(remediation) = &notification.context.remediation {
            body.push(CardElement::Container {
                style: Some("emphasis"),
                items: vec![
                    CardElement::TextBlock {
                        text: "Remediation".to_string(),
                        size: None,
                        weight: Some("Bolder"),
                        color: None,
                        wrap: None,
                        spacing: Some("Medium"),
                    },
                    CardElement::TextBlock {
                        text: remediation.clone(),
                        size: None,
                        weight: None,
                        color: None,
                        wrap: Some(true),
                        spacing: Some("Small"),
                    },
                ],
            });
        }

        // Actions
        let mut actions = Vec::new();
        if let Some(url) = &notification.context.dashboard_url {
            actions.push(CardAction::OpenUrl {
                title: "View in Dashboard".to_string(),
                url: url.clone(),
            });
        }

        let card = AdaptiveCard {
            schema: "http://adaptivecards.io/schemas/adaptive-card.json",
            card_type: "AdaptiveCard",
            version: "1.4",
            body,
            actions,
        };

        TeamsMessage {
            message_type: "message",
            attachments: vec![TeamsAttachment {
                content_type: "application/vnd.microsoft.card.adaptive",
                content: card,
            }],
        }
    }
}

#[async_trait]
impl NotificationChannel for TeamsChannel {
    fn name(&self) -> &'static str {
        "teams"
    }

    fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    #[instrument(skip(self, notification), fields(channel = "teams"))]
    async fn send(&self, notification: &Notification) -> NotificationResult<DeliveryResult> {
        if !self.is_enabled() {
            return Err(NotificationError::ChannelDisabled);
        }

        let start = Instant::now();

        debug!(
            notification_id = %notification.id,
            "Sending Teams notification"
        );

        let message = self.build_message(notification);

        let response = self
            .client
            .post(&self.config.webhook_url)
            .json(&message)
            .send()
            .await?;

        let duration_ms = start.elapsed().as_millis() as u64;
        let status = response.status();

        if status.is_success() {
            info!(
                notification_id = %notification.id,
                duration_ms,
                "Teams notification sent successfully"
            );

            Ok(DeliveryResult::success_with_response(
                duration_ms,
                serde_json::json!({
                    "status": status.as_u16(),
                }),
            ))
        } else if status.as_u16() == 429 {
            warn!(
                notification_id = %notification.id,
                "Teams rate limit exceeded"
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
        } else {
            let body = response.text().await.unwrap_or_default();
            error!(
                notification_id = %notification.id,
                status = %status,
                body = %body,
                "Failed to send Teams notification"
            );

            Ok(DeliveryResult::failure(
                duration_ms,
                format!("HTTP {status}: {body}"),
            ))
        }
    }

    async fn validate(&self) -> NotificationResult<()> {
        if self.config.webhook_url.is_empty() {
            return Err(NotificationError::InvalidConfig(
                "Webhook URL is required".to_string(),
            ));
        }

        // Validate URL format
        let url = Url::parse(&self.config.webhook_url)?;

        // Check for Teams webhook URL patterns
        let host = url.host_str().unwrap_or("");
        if !host.contains("webhook.office.com") && !host.contains("microsoft.com") {
            warn!("Webhook URL does not appear to be a Teams URL: {}", host);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::NotificationContext;

    #[test]
    fn test_teams_config_builder() {
        let config = TeamsConfig::builder()
            .webhook_url("https://outlook.office.com/webhook/xxx/IncomingWebhook/yyy/zzz")
            .timeout_secs(60)
            .enabled(true)
            .build();

        assert!(config.webhook_url.contains("webhook"));
        assert_eq!(config.timeout_secs, 60);
        assert!(config.enabled);
    }

    #[test]
    fn test_build_message() {
        let config = TeamsConfig::builder()
            .webhook_url("https://outlook.office.com/webhook/test")
            .enabled(true)
            .build();

        let channel = TeamsChannel::new(config);

        let notification = Notification::new(
            Severity::Critical,
            "Supply Chain Alert",
            "A critical vulnerability was detected in a production dependency.",
        )
        .with_context(
            NotificationContext::new()
                .with_project("api-server")
                .with_package("axios", "0.21.1")
                .with_dashboard_url("https://sctv.example.com/alerts/123"),
        );

        let message = channel.build_message(&notification);

        assert_eq!(message.message_type, "message");
        assert_eq!(message.attachments.len(), 1);
        assert_eq!(
            message.attachments[0].content_type,
            "application/vnd.microsoft.card.adaptive"
        );
        assert!(!message.attachments[0].content.actions.is_empty());
    }

    #[test]
    fn test_severity_indicators() {
        assert_eq!(
            TeamsChannel::severity_color(Severity::Critical),
            "attention"
        );
        assert_eq!(TeamsChannel::severity_color(Severity::High), "warning");
        assert_eq!(TeamsChannel::severity_color(Severity::Info), "default");
    }
}
