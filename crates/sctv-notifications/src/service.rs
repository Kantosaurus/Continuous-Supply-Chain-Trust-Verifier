//! Notification service for coordinating delivery across channels.

use std::collections::HashMap;
use std::sync::Arc;

use sctv_core::{NotificationChannelConfig, NotificationChannelType, Severity};
use serde::{Deserialize, Serialize};
use tracing::{error, info, instrument, warn};

use crate::channels::{
    EmailChannel, EmailConfig, NotificationChannel, PagerDutyChannel, PagerDutyConfig,
    SlackChannel, SlackConfig, TeamsChannel, TeamsConfig, WebhookChannel, WebhookConfig,
};
use crate::error::NotificationResult;
use crate::types::{DeliveryResult, Notification};

/// Configuration for the notification service.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NotificationServiceConfig {
    /// Whether to send notifications in parallel.
    #[serde(default = "default_parallel")]
    pub parallel_delivery: bool,
    /// Whether to continue sending to remaining channels on failure.
    #[serde(default = "default_continue_on_failure")]
    pub continue_on_failure: bool,
    /// Default minimum severity for notifications.
    #[serde(default)]
    pub default_min_severity: Option<Severity>,
}

fn default_parallel() -> bool {
    true
}

fn default_continue_on_failure() -> bool {
    true
}

impl NotificationServiceConfig {
    /// Creates a new configuration builder.
    #[must_use]
    pub fn builder() -> NotificationServiceConfigBuilder {
        NotificationServiceConfigBuilder::default()
    }
}

/// Builder for `NotificationServiceConfig`.
#[derive(Debug, Default)]
pub struct NotificationServiceConfigBuilder {
    config: NotificationServiceConfig,
}

impl NotificationServiceConfigBuilder {
    /// Sets whether to deliver in parallel.
    #[must_use]
    pub fn parallel_delivery(mut self, parallel: bool) -> Self {
        self.config.parallel_delivery = parallel;
        self
    }

    /// Sets whether to continue on failure.
    #[must_use]
    pub fn continue_on_failure(mut self, continue_on_failure: bool) -> Self {
        self.config.continue_on_failure = continue_on_failure;
        self
    }

    /// Sets the default minimum severity.
    #[must_use]
    pub fn default_min_severity(mut self, severity: Severity) -> Self {
        self.config.default_min_severity = Some(severity);
        self
    }

    /// Builds the configuration.
    #[must_use]
    pub fn build(self) -> NotificationServiceConfig {
        self.config
    }
}

/// Result of sending notifications to multiple channels.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiChannelResult {
    /// Results keyed by channel name.
    pub results: HashMap<String, DeliveryResult>,
    /// Total number of successful deliveries.
    pub successful: usize,
    /// Total number of failed deliveries.
    pub failed: usize,
    /// Total number of skipped channels (disabled or filtered by severity).
    pub skipped: usize,
}

impl MultiChannelResult {
    /// Creates a new empty result.
    fn new() -> Self {
        Self {
            results: HashMap::new(),
            successful: 0,
            failed: 0,
            skipped: 0,
        }
    }

    /// Adds a result for a channel.
    fn add_result(&mut self, channel_name: String, result: DeliveryResult) {
        if result.success {
            self.successful += 1;
        } else {
            self.failed += 1;
        }
        self.results.insert(channel_name, result);
    }

    /// Marks a channel as skipped.
    fn add_skipped(&mut self, channel_name: String, reason: &str) {
        self.skipped += 1;
        self.results.insert(
            channel_name,
            DeliveryResult::failure(0, format!("Skipped: {reason}")),
        );
    }

    /// Returns true if all deliveries were successful.
    #[must_use]
    pub fn all_successful(&self) -> bool {
        self.failed == 0
    }

    /// Returns true if at least one delivery was successful.
    #[must_use]
    pub fn any_successful(&self) -> bool {
        self.successful > 0
    }
}

/// Channel entry with configuration for filtering.
struct ChannelEntry {
    channel: Arc<dyn NotificationChannel>,
    min_severity: Severity,
    enabled: bool,
}

/// Service for coordinating notification delivery across multiple channels.
pub struct NotificationService {
    config: NotificationServiceConfig,
    channels: Vec<ChannelEntry>,
}

impl NotificationService {
    /// Creates a new notification service builder.
    #[must_use]
    pub fn builder() -> NotificationServiceBuilder {
        NotificationServiceBuilder::new()
    }

    /// Creates a notification service from tenant channel configurations.
    ///
    /// This is the primary way to construct a service from stored configurations.
    #[must_use]
    pub fn from_channel_configs(
        configs: &[NotificationChannelConfig],
        service_config: NotificationServiceConfig,
    ) -> Self {
        let mut builder = Self::builder().config(service_config);

        for channel_config in configs {
            if !channel_config.enabled {
                continue;
            }

            match channel_config.channel_type {
                NotificationChannelType::Email => {
                    if let Ok(email_config) =
                        serde_json::from_value::<EmailConfig>(channel_config.config.clone())
                    {
                        let mut config = email_config;
                        config.enabled = true;
                        builder = builder.email(config, channel_config.min_severity);
                    } else {
                        warn!("Failed to parse email channel configuration");
                    }
                }
                NotificationChannelType::Slack => {
                    if let Ok(slack_config) =
                        serde_json::from_value::<SlackConfig>(channel_config.config.clone())
                    {
                        let mut config = slack_config;
                        config.enabled = true;
                        builder = builder.slack(config, channel_config.min_severity);
                    } else {
                        warn!("Failed to parse Slack channel configuration");
                    }
                }
                NotificationChannelType::Webhook => {
                    if let Ok(webhook_config) =
                        serde_json::from_value::<WebhookConfig>(channel_config.config.clone())
                    {
                        let mut config = webhook_config;
                        config.enabled = true;
                        builder = builder.webhook(config, channel_config.min_severity);
                    } else {
                        warn!("Failed to parse webhook channel configuration");
                    }
                }
                NotificationChannelType::Teams => {
                    if let Ok(teams_config) =
                        serde_json::from_value::<TeamsConfig>(channel_config.config.clone())
                    {
                        let mut config = teams_config;
                        config.enabled = true;
                        builder = builder.teams(config, channel_config.min_severity);
                    } else {
                        warn!("Failed to parse Teams channel configuration");
                    }
                }
                NotificationChannelType::PagerDuty => {
                    if let Ok(pagerduty_config) =
                        serde_json::from_value::<PagerDutyConfig>(channel_config.config.clone())
                    {
                        let mut config = pagerduty_config;
                        config.enabled = true;
                        builder = builder.pagerduty(config, channel_config.min_severity);
                    } else {
                        warn!("Failed to parse PagerDuty channel configuration");
                    }
                }
            }
        }

        builder.build()
    }

    /// Sends a notification to all configured channels.
    #[instrument(skip(self, notification), fields(notification_id = %notification.id))]
    pub async fn send(&self, notification: &Notification) -> MultiChannelResult {
        let mut result = MultiChannelResult::new();

        if self.channels.is_empty() {
            info!("No notification channels configured");
            return result;
        }

        info!(
            severity = %notification.severity,
            title = %notification.title,
            channel_count = self.channels.len(),
            "Sending notification to channels"
        );

        if self.config.parallel_delivery {
            self.send_parallel(notification, &mut result).await;
        } else {
            self.send_sequential(notification, &mut result).await;
        }

        info!(
            successful = result.successful,
            failed = result.failed,
            skipped = result.skipped,
            "Notification delivery complete"
        );

        result
    }

    /// Sends notifications to channels in parallel.
    async fn send_parallel(&self, notification: &Notification, result: &mut MultiChannelResult) {
        let futures: Vec<_> = self
            .channels
            .iter()
            .filter_map(|entry| {
                if !self.should_send(entry, notification.severity) {
                    return None;
                }

                let channel = Arc::clone(&entry.channel);
                let notif = notification.clone();

                Some(async move {
                    let channel_name = channel.name().to_string();
                    let delivery_result = channel.send(&notif).await;
                    (channel_name, delivery_result)
                })
            })
            .collect();

        // Track skipped channels
        for entry in &self.channels {
            if !self.should_send(entry, notification.severity) {
                let reason = if !entry.enabled {
                    "channel disabled"
                } else {
                    "severity below threshold"
                };
                result.add_skipped(entry.channel.name().to_string(), reason);
            }
        }

        let results = futures::future::join_all(futures).await;

        for (channel_name, delivery_result) in results {
            match delivery_result {
                Ok(dr) => result.add_result(channel_name, dr),
                Err(e) => {
                    let error_msg = e.to_string();
                    error!(channel = %channel_name, error = %error_msg, "Channel delivery failed");
                    result.add_result(channel_name, DeliveryResult::failure(0, error_msg));
                }
            }
        }
    }

    /// Sends notifications to channels sequentially.
    async fn send_sequential(
        &self,
        notification: &Notification,
        result: &mut MultiChannelResult,
    ) {
        for entry in &self.channels {
            let channel_name = entry.channel.name().to_string();

            if !self.should_send(entry, notification.severity) {
                let reason = if !entry.enabled {
                    "channel disabled"
                } else {
                    "severity below threshold"
                };
                result.add_skipped(channel_name, reason);
                continue;
            }

            match entry.channel.send(notification).await {
                Ok(dr) => {
                    let success = dr.success;
                    result.add_result(channel_name, dr);

                    if !success && !self.config.continue_on_failure {
                        warn!("Stopping delivery due to failure");
                        break;
                    }
                }
                Err(e) => {
                    error!(channel = %channel_name, error = %e, "Channel delivery failed");
                    result.add_result(
                        channel_name.clone(),
                        DeliveryResult::failure(0, e.to_string()),
                    );

                    if !self.config.continue_on_failure {
                        warn!("Stopping delivery due to error");
                        break;
                    }
                }
            }
        }
    }

    /// Checks if a notification should be sent to a channel.
    fn should_send(&self, entry: &ChannelEntry, severity: Severity) -> bool {
        if !entry.enabled || !entry.channel.is_enabled() {
            return false;
        }

        // Check severity threshold
        severity >= entry.min_severity
    }

    /// Validates all channel configurations.
    pub async fn validate_all(&self) -> HashMap<String, NotificationResult<()>> {
        let mut results = HashMap::new();

        for entry in &self.channels {
            let channel_name = entry.channel.name().to_string();
            let validation = entry.channel.validate().await;
            results.insert(channel_name, validation);
        }

        results
    }

    /// Returns the number of configured channels.
    #[must_use]
    pub fn channel_count(&self) -> usize {
        self.channels.len()
    }

    /// Returns the number of enabled channels.
    #[must_use]
    pub fn enabled_channel_count(&self) -> usize {
        self.channels
            .iter()
            .filter(|e| e.enabled && e.channel.is_enabled())
            .count()
    }
}

/// Builder for `NotificationService`.
pub struct NotificationServiceBuilder {
    config: NotificationServiceConfig,
    channels: Vec<ChannelEntry>,
}

impl NotificationServiceBuilder {
    /// Creates a new builder.
    fn new() -> Self {
        Self {
            config: NotificationServiceConfig::default(),
            channels: Vec::new(),
        }
    }

    /// Sets the service configuration.
    #[must_use]
    pub fn config(mut self, config: NotificationServiceConfig) -> Self {
        self.config = config;
        self
    }

    /// Adds an email channel.
    #[must_use]
    pub fn email(mut self, config: EmailConfig, min_severity: Severity) -> Self {
        let enabled = config.enabled;
        self.channels.push(ChannelEntry {
            channel: Arc::new(EmailChannel::new(config)),
            min_severity,
            enabled,
        });
        self
    }

    /// Adds a Slack channel.
    #[must_use]
    pub fn slack(mut self, config: SlackConfig, min_severity: Severity) -> Self {
        let enabled = config.enabled;
        self.channels.push(ChannelEntry {
            channel: Arc::new(SlackChannel::new(config)),
            min_severity,
            enabled,
        });
        self
    }

    /// Adds a generic webhook channel.
    #[must_use]
    pub fn webhook(mut self, config: WebhookConfig, min_severity: Severity) -> Self {
        let enabled = config.enabled;
        self.channels.push(ChannelEntry {
            channel: Arc::new(WebhookChannel::new(config)),
            min_severity,
            enabled,
        });
        self
    }

    /// Adds a Microsoft Teams channel.
    #[must_use]
    pub fn teams(mut self, config: TeamsConfig, min_severity: Severity) -> Self {
        let enabled = config.enabled;
        self.channels.push(ChannelEntry {
            channel: Arc::new(TeamsChannel::new(config)),
            min_severity,
            enabled,
        });
        self
    }

    /// Adds a PagerDuty channel.
    #[must_use]
    pub fn pagerduty(mut self, config: PagerDutyConfig, min_severity: Severity) -> Self {
        let enabled = config.enabled;
        self.channels.push(ChannelEntry {
            channel: Arc::new(PagerDutyChannel::new(config)),
            min_severity,
            enabled,
        });
        self
    }

    /// Adds a custom channel implementation.
    #[must_use]
    pub fn custom(
        mut self,
        channel: Arc<dyn NotificationChannel>,
        min_severity: Severity,
        enabled: bool,
    ) -> Self {
        self.channels.push(ChannelEntry {
            channel,
            min_severity,
            enabled,
        });
        self
    }

    /// Builds the notification service.
    #[must_use]
    pub fn build(self) -> NotificationService {
        NotificationService {
            config: self.config,
            channels: self.channels,
        }
    }
}

impl Default for NotificationServiceBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service_config_builder() {
        let config = NotificationServiceConfig::builder()
            .parallel_delivery(false)
            .continue_on_failure(true)
            .default_min_severity(Severity::High)
            .build();

        assert!(!config.parallel_delivery);
        assert!(config.continue_on_failure);
        assert_eq!(config.default_min_severity, Some(Severity::High));
    }

    #[test]
    fn test_multi_channel_result() {
        let mut result = MultiChannelResult::new();

        result.add_result("email".to_string(), DeliveryResult::success(100));
        result.add_result(
            "slack".to_string(),
            DeliveryResult::failure(50, "Connection refused"),
        );
        result.add_skipped("webhook".to_string(), "disabled");

        assert_eq!(result.successful, 1);
        assert_eq!(result.failed, 1);
        assert_eq!(result.skipped, 1);
        assert!(!result.all_successful());
        assert!(result.any_successful());
    }

    #[test]
    fn test_service_builder() {
        let service = NotificationService::builder()
            .config(
                NotificationServiceConfig::builder()
                    .parallel_delivery(true)
                    .build(),
            )
            .slack(
                SlackConfig::builder()
                    .webhook_url("https://hooks.slack.com/test")
                    .enabled(true)
                    .build(),
                Severity::Medium,
            )
            .webhook(
                WebhookConfig::builder()
                    .url("https://api.example.com/webhook")
                    .enabled(true)
                    .build(),
                Severity::High,
            )
            .build();

        assert_eq!(service.channel_count(), 2);
        assert_eq!(service.enabled_channel_count(), 2);
    }

    #[tokio::test]
    async fn test_empty_service() {
        let service = NotificationService::builder().build();

        let notification = Notification::new(Severity::High, "Test", "Test message");
        let result = service.send(&notification).await;

        assert_eq!(result.successful, 0);
        assert_eq!(result.failed, 0);
        assert_eq!(result.skipped, 0);
    }
}
