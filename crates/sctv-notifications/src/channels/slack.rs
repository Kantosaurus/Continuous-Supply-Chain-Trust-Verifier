//! Slack notification channel using webhooks.

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

/// Configuration for the Slack notification channel.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlackConfig {
    /// Slack incoming webhook URL.
    #[serde(skip_serializing)]
    pub webhook_url: String,
    /// Channel to post to (overrides webhook default).
    pub channel: Option<String>,
    /// Username to display (overrides webhook default).
    pub username: Option<String>,
    /// Icon emoji to use (e.g., ":warning:").
    pub icon_emoji: Option<String>,
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

impl Default for SlackConfig {
    fn default() -> Self {
        Self {
            webhook_url: String::new(),
            channel: None,
            username: Some("Supply Chain Trust Verifier".to_string()),
            icon_emoji: Some(":shield:".to_string()),
            timeout_secs: 30,
            enabled: false,
        }
    }
}

impl SlackConfig {
    /// Creates a new Slack configuration builder.
    #[must_use]
    pub fn builder() -> SlackConfigBuilder {
        SlackConfigBuilder::default()
    }
}

/// Builder for `SlackConfig`.
#[derive(Debug, Default)]
pub struct SlackConfigBuilder {
    config: SlackConfig,
}

impl SlackConfigBuilder {
    /// Sets the webhook URL.
    #[must_use]
    pub fn webhook_url(mut self, url: impl Into<String>) -> Self {
        self.config.webhook_url = url.into();
        self
    }

    /// Sets the channel to post to.
    #[must_use]
    pub fn channel(mut self, channel: impl Into<String>) -> Self {
        self.config.channel = Some(channel.into());
        self
    }

    /// Sets the username to display.
    #[must_use]
    pub fn username(mut self, username: impl Into<String>) -> Self {
        self.config.username = Some(username.into());
        self
    }

    /// Sets the icon emoji.
    #[must_use]
    pub fn icon_emoji(mut self, emoji: impl Into<String>) -> Self {
        self.config.icon_emoji = Some(emoji.into());
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
    pub fn build(self) -> SlackConfig {
        self.config
    }
}

/// Slack message payload.
#[derive(Debug, Serialize)]
struct SlackMessage {
    #[serde(skip_serializing_if = "Option::is_none")]
    channel: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    icon_emoji: Option<String>,
    attachments: Vec<SlackAttachment>,
}

/// Slack attachment for rich formatting.
#[derive(Debug, Serialize)]
struct SlackAttachment {
    color: String,
    title: String,
    text: String,
    fields: Vec<SlackField>,
    footer: String,
    ts: i64,
}

/// Slack attachment field.
#[derive(Debug, Serialize)]
struct SlackField {
    title: String,
    value: String,
    short: bool,
}

/// Slack notification channel using webhooks.
pub struct SlackChannel {
    config: SlackConfig,
    client: Client,
}

impl SlackChannel {
    /// Creates a new Slack channel with the given configuration.
    #[must_use]
    pub fn new(config: SlackConfig) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(config.timeout_secs))
            .build()
            .expect("Failed to create HTTP client");

        Self { config, client }
    }

    /// Returns the color for the given severity level.
    const fn severity_color(severity: Severity) -> &'static str {
        match severity {
            Severity::Critical => "#dc3545", // Red
            Severity::High => "#fd7e14",     // Orange
            Severity::Medium => "#ffc107",   // Yellow
            Severity::Low => "#17a2b8",      // Cyan
            Severity::Info => "#6c757d",     // Gray
        }
    }

    /// Returns the emoji for the given severity level.
    const fn severity_emoji(severity: Severity) -> &'static str {
        match severity {
            Severity::Critical => ":rotating_light:",
            Severity::High => ":warning:",
            Severity::Medium => ":large_orange_diamond:",
            Severity::Low => ":information_source:",
            Severity::Info => ":speech_balloon:",
        }
    }

    /// Builds the Slack message payload.
    fn build_message(&self, notification: &Notification) -> SlackMessage {
        let mut fields = Vec::new();

        fields.push(SlackField {
            title: "Severity".to_string(),
            value: format!(
                "{} {}",
                Self::severity_emoji(notification.severity),
                notification.severity
            ),
            short: true,
        });

        if let Some(project) = &notification.context.project_name {
            fields.push(SlackField {
                title: "Project".to_string(),
                value: project.clone(),
                short: true,
            });
        }

        if let Some(package) = &notification.context.package_name {
            let mut value = package.clone();
            if let Some(version) = &notification.context.package_version {
                value.push('@');
                value.push_str(version);
            }
            fields.push(SlackField {
                title: "Package".to_string(),
                value,
                short: true,
            });
        }

        if let Some(alert_type) = &notification.context.alert_type {
            fields.push(SlackField {
                title: "Alert Type".to_string(),
                value: alert_type.clone(),
                short: true,
            });
        }

        let mut text = notification.message.clone();

        if let Some(remediation) = &notification.context.remediation {
            text.push_str(&format!("\n\n*Remediation:*\n{remediation}"));
        }

        if let Some(url) = &notification.context.dashboard_url {
            text.push_str(&format!("\n\n<{url}|View in Dashboard>"));
        }

        let attachment = SlackAttachment {
            color: Self::severity_color(notification.severity).to_string(),
            title: notification.title.clone(),
            text,
            fields,
            footer: "Supply Chain Trust Verifier".to_string(),
            ts: notification.created_at.timestamp(),
        };

        SlackMessage {
            channel: self.config.channel.clone(),
            username: self.config.username.clone(),
            icon_emoji: self.config.icon_emoji.clone(),
            attachments: vec![attachment],
        }
    }
}

#[async_trait]
impl NotificationChannel for SlackChannel {
    fn name(&self) -> &'static str {
        "slack"
    }

    fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    #[instrument(skip(self, notification), fields(channel = "slack"))]
    async fn send(&self, notification: &Notification) -> NotificationResult<DeliveryResult> {
        if !self.is_enabled() {
            return Err(NotificationError::ChannelDisabled);
        }

        let start = Instant::now();

        debug!(
            notification_id = %notification.id,
            "Sending Slack notification"
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
                "Slack notification sent successfully"
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
                "Slack rate limit exceeded"
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
                "Failed to send Slack notification"
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

        // Ensure it's a Slack webhook URL
        if !url.host_str().is_some_and(|h| h.contains("slack.com")) {
            warn!(
                "Webhook URL does not appear to be a Slack URL: {}",
                url.host_str().unwrap_or("unknown")
            );
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::NotificationContext;

    #[test]
    fn test_slack_config_builder() {
        let config = SlackConfig::builder()
            .webhook_url("https://hooks.slack.com/services/xxx/yyy/zzz")
            .channel("#security-alerts")
            .username("SCTV Bot")
            .icon_emoji(":lock:")
            .enabled(true)
            .build();

        assert_eq!(config.channel, Some("#security-alerts".to_string()));
        assert_eq!(config.username, Some("SCTV Bot".to_string()));
        assert!(config.enabled);
    }

    #[test]
    fn test_build_message() {
        let config = SlackConfig::builder()
            .webhook_url("https://hooks.slack.com/services/xxx/yyy/zzz")
            .channel("#alerts")
            .enabled(true)
            .build();

        let channel = SlackChannel::new(config);

        let notification = Notification::new(
            Severity::High,
            "Typosquatting Detected",
            "Package 'lodash-utils' may be a typosquatting attempt.",
        )
        .with_context(
            NotificationContext::new()
                .with_project("my-app")
                .with_package("lodash-utils", "1.0.0"),
        );

        let message = channel.build_message(&notification);

        assert_eq!(message.channel, Some("#alerts".to_string()));
        assert_eq!(message.attachments.len(), 1);
        assert_eq!(message.attachments[0].color, "#fd7e14"); // High severity orange
    }

    #[test]
    fn test_severity_colors() {
        assert_eq!(SlackChannel::severity_color(Severity::Critical), "#dc3545");
        assert_eq!(SlackChannel::severity_color(Severity::High), "#fd7e14");
        assert_eq!(SlackChannel::severity_color(Severity::Medium), "#ffc107");
        assert_eq!(SlackChannel::severity_color(Severity::Low), "#17a2b8");
        assert_eq!(SlackChannel::severity_color(Severity::Info), "#6c757d");
    }
}
