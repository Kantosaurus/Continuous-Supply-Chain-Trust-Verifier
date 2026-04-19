//! Send notification job executor.
//!
//! This executor handles sending notifications through various channels
//! using the `sctv-notifications` crate.

use async_trait::async_trait;
use std::time::Instant;

use sctv_notifications::{
    channels::NotificationChannel as ChannelTrait, EmailChannel, EmailConfig, Notification,
    NotificationContext, PagerDutyChannel, PagerDutyConfig, SlackChannel, SlackConfig,
    TeamsChannel, TeamsConfig, WebhookChannel, WebhookConfig,
};

use crate::error::{WorkerError, WorkerResult};
use crate::executor::{ExecutionContext, JobExecutor};
use crate::jobs::{
    Job, JobPayload, JobResult, JobType, NotificationChannel, SendNotificationPayload,
    SendNotificationResult,
};

/// Executor for sending notifications.
///
/// This executor uses the `sctv-notifications` crate to deliver alerts
/// through various channels (Email, Slack, Teams, `PagerDuty`, Webhook).
pub struct SendNotificationExecutor;

impl SendNotificationExecutor {
    /// Creates a new send notification executor.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Converts a job payload to a notification.
    fn create_notification(payload: &SendNotificationPayload) -> Notification {
        let mut context = NotificationContext::new();

        if let Some(project) = &payload.context.project_name {
            context = context.with_project(project);
        }

        if let Some(package) = &payload.context.package_name {
            if let Some(version) = &payload.context.package_version {
                context = context.with_package(package, version);
            } else {
                context.package_name = Some(package.clone());
            }
        }

        if let Some(url) = &payload.context.dashboard_url {
            context = context.with_dashboard_url(url);
        }

        if let Some(remediation) = &payload.context.remediation {
            context = context.with_remediation(remediation);
        }

        // Set alert type from the alert_id for tracking
        context.alert_type = Some(format!("alert_{}", payload.alert_id));

        Notification::new(payload.severity, &payload.title, &payload.description)
            .with_context(context)
    }

    /// Executes the notification send operation.
    async fn execute_send(
        &self,
        payload: &SendNotificationPayload,
        _ctx: &ExecutionContext,
    ) -> WorkerResult<SendNotificationResult> {
        let start = Instant::now();

        tracing::info!(
            alert_id = %payload.alert_id,
            channel = ?payload.channel,
            severity = ?payload.severity,
            "Sending notification"
        );

        let notification = Self::create_notification(payload);

        let result = match payload.channel {
            NotificationChannel::Email => self.send_email(payload, &notification).await,
            NotificationChannel::Slack => self.send_slack(payload, &notification).await,
            NotificationChannel::Teams => self.send_teams(payload, &notification).await,
            NotificationChannel::PagerDuty => self.send_pagerduty(payload, &notification).await,
            NotificationChannel::Webhook => self.send_webhook(payload, &notification).await,
        };

        let duration = start.elapsed();

        match result {
            Ok(response) => {
                tracing::info!(
                    alert_id = %payload.alert_id,
                    channel = ?payload.channel,
                    duration_ms = duration.as_millis(),
                    "Notification sent successfully"
                );
                Ok(SendNotificationResult {
                    sent: true,
                    response: Some(response),
                    send_duration_ms: duration.as_millis() as u64,
                })
            }
            Err(e) => {
                tracing::error!(
                    alert_id = %payload.alert_id,
                    channel = ?payload.channel,
                    error = %e,
                    duration_ms = duration.as_millis(),
                    "Failed to send notification"
                );
                // Return error result instead of failing the job for transient errors
                Ok(SendNotificationResult {
                    sent: false,
                    response: Some(serde_json::json!({
                        "error": e.to_string(),
                        "channel": format!("{:?}", payload.channel),
                    })),
                    send_duration_ms: duration.as_millis() as u64,
                })
            }
        }
    }

    /// Sends notification via email channel.
    async fn send_email(
        &self,
        payload: &SendNotificationPayload,
        notification: &Notification,
    ) -> WorkerResult<serde_json::Value> {
        // Parse email configuration from channel_config
        let config: EmailConfig = serde_json::from_value(payload.channel_config.clone())
            .map_err(|e| WorkerError::Notification(format!("Invalid email configuration: {e}")))?;

        // Ensure the channel is enabled for this send
        let config = EmailConfig {
            enabled: true,
            ..config
        };

        let channel = EmailChannel::new(config);

        // Validate configuration
        channel.validate().await.map_err(|e| {
            WorkerError::Notification(format!("Email configuration validation failed: {e}"))
        })?;

        let result = channel
            .send(notification)
            .await
            .map_err(|e| WorkerError::Notification(format!("Email delivery failed: {e}")))?;

        if result.success {
            Ok(serde_json::json!({
                "status": "sent",
                "channel": "email",
                "duration_ms": result.duration_ms,
                "response": result.response,
            }))
        } else {
            Err(WorkerError::Notification(
                result
                    .error
                    .unwrap_or_else(|| "Unknown email error".to_string()),
            ))
        }
    }

    /// Sends notification via Slack webhook.
    async fn send_slack(
        &self,
        payload: &SendNotificationPayload,
        notification: &Notification,
    ) -> WorkerResult<serde_json::Value> {
        // Parse Slack configuration
        let config: SlackConfig = serde_json::from_value(payload.channel_config.clone())
            .map_err(|e| WorkerError::Notification(format!("Invalid Slack configuration: {e}")))?;

        let config = SlackConfig {
            enabled: true,
            ..config
        };

        let channel = SlackChannel::new(config);

        channel.validate().await.map_err(|e| {
            WorkerError::Notification(format!("Slack configuration validation failed: {e}"))
        })?;

        let result = channel
            .send(notification)
            .await
            .map_err(|e| WorkerError::Notification(format!("Slack delivery failed: {e}")))?;

        if result.success {
            Ok(serde_json::json!({
                "status": "sent",
                "channel": "slack",
                "duration_ms": result.duration_ms,
                "response": result.response,
            }))
        } else {
            Err(WorkerError::Notification(
                result
                    .error
                    .unwrap_or_else(|| "Unknown Slack error".to_string()),
            ))
        }
    }

    /// Sends notification via Microsoft Teams webhook.
    async fn send_teams(
        &self,
        payload: &SendNotificationPayload,
        notification: &Notification,
    ) -> WorkerResult<serde_json::Value> {
        // Parse Teams configuration
        let config: TeamsConfig = serde_json::from_value(payload.channel_config.clone())
            .map_err(|e| WorkerError::Notification(format!("Invalid Teams configuration: {e}")))?;

        let config = TeamsConfig {
            enabled: true,
            ..config
        };

        let channel = TeamsChannel::new(config);

        channel.validate().await.map_err(|e| {
            WorkerError::Notification(format!("Teams configuration validation failed: {e}"))
        })?;

        let result = channel
            .send(notification)
            .await
            .map_err(|e| WorkerError::Notification(format!("Teams delivery failed: {e}")))?;

        if result.success {
            Ok(serde_json::json!({
                "status": "sent",
                "channel": "teams",
                "duration_ms": result.duration_ms,
                "response": result.response,
            }))
        } else {
            Err(WorkerError::Notification(
                result
                    .error
                    .unwrap_or_else(|| "Unknown Teams error".to_string()),
            ))
        }
    }

    /// Sends notification via `PagerDuty` Events API.
    async fn send_pagerduty(
        &self,
        payload: &SendNotificationPayload,
        notification: &Notification,
    ) -> WorkerResult<serde_json::Value> {
        // Parse PagerDuty configuration
        let config: PagerDutyConfig = serde_json::from_value(payload.channel_config.clone())
            .map_err(|e| {
                WorkerError::Notification(format!("Invalid PagerDuty configuration: {e}"))
            })?;

        let config = PagerDutyConfig {
            enabled: true,
            ..config
        };

        let channel = PagerDutyChannel::new(config);

        channel.validate().await.map_err(|e| {
            WorkerError::Notification(format!("PagerDuty configuration validation failed: {e}"))
        })?;

        let result = channel
            .send(notification)
            .await
            .map_err(|e| WorkerError::Notification(format!("PagerDuty delivery failed: {e}")))?;

        if result.success {
            Ok(serde_json::json!({
                "status": "sent",
                "channel": "pagerduty",
                "duration_ms": result.duration_ms,
                "response": result.response,
            }))
        } else {
            Err(WorkerError::Notification(
                result
                    .error
                    .unwrap_or_else(|| "Unknown PagerDuty error".to_string()),
            ))
        }
    }

    /// Sends notification via generic webhook.
    async fn send_webhook(
        &self,
        payload: &SendNotificationPayload,
        notification: &Notification,
    ) -> WorkerResult<serde_json::Value> {
        // Parse webhook configuration
        let config: WebhookConfig = serde_json::from_value(payload.channel_config.clone())
            .map_err(|e| {
                WorkerError::Notification(format!("Invalid webhook configuration: {e}"))
            })?;

        let config = WebhookConfig {
            enabled: true,
            ..config
        };

        let channel = WebhookChannel::new(config);

        channel.validate().await.map_err(|e| {
            WorkerError::Notification(format!("Webhook configuration validation failed: {e}"))
        })?;

        let result = channel
            .send(notification)
            .await
            .map_err(|e| WorkerError::Notification(format!("Webhook delivery failed: {e}")))?;

        if result.success {
            Ok(serde_json::json!({
                "status": "sent",
                "channel": "webhook",
                "duration_ms": result.duration_ms,
                "response": result.response,
            }))
        } else {
            Err(WorkerError::Notification(
                result
                    .error
                    .unwrap_or_else(|| "Unknown webhook error".to_string()),
            ))
        }
    }
}

impl Default for SendNotificationExecutor {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl JobExecutor for SendNotificationExecutor {
    fn handles(&self) -> Vec<JobType> {
        vec![JobType::SendNotification]
    }

    async fn execute(&self, job: &Job, ctx: &ExecutionContext) -> WorkerResult<JobResult> {
        let payload = match &job.payload {
            JobPayload::SendNotification(p) => p,
            _ => {
                return Err(WorkerError::Execution(
                    "Invalid payload type for SendNotification".into(),
                ))
            }
        };

        let result = self.execute_send(payload, ctx).await?;
        Ok(JobResult::SendNotification(result))
    }

    fn default_timeout_secs(&self) -> u64 {
        60 // 1 minute for notifications
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sctv_core::{AlertId, Severity, TenantId};
    use uuid::Uuid;

    fn create_test_payload(channel: NotificationChannel) -> SendNotificationPayload {
        SendNotificationPayload {
            alert_id: AlertId(Uuid::new_v4()),
            tenant_id: TenantId(Uuid::new_v4()),
            channel,
            channel_config: serde_json::json!({}),
            severity: Severity::High,
            title: "Test Alert".to_string(),
            description: "This is a test notification".to_string(),
            context: crate::jobs::NotificationContext {
                project_name: Some("test-project".to_string()),
                package_name: Some("lodash".to_string()),
                package_version: Some("4.17.21".to_string()),
                dashboard_url: Some("https://sctv.example.com/alerts/123".to_string()),
                remediation: Some("Update to the latest version".to_string()),
            },
        }
    }

    #[test]
    fn test_create_notification() {
        let payload = create_test_payload(NotificationChannel::Slack);
        let notification = SendNotificationExecutor::create_notification(&payload);

        assert_eq!(notification.severity, Severity::High);
        assert_eq!(notification.title, "Test Alert");
        assert_eq!(notification.message, "This is a test notification");
        assert_eq!(
            notification.context.project_name,
            Some("test-project".to_string())
        );
        assert_eq!(
            notification.context.package_name,
            Some("lodash".to_string())
        );
        assert_eq!(
            notification.context.package_version,
            Some("4.17.21".to_string())
        );
    }

    #[test]
    fn test_executor_handles_job_type() {
        let executor = SendNotificationExecutor::new();
        let handles = executor.handles();

        assert_eq!(handles.len(), 1);
        assert_eq!(handles[0], JobType::SendNotification);
    }

    #[test]
    fn test_default_timeout() {
        let executor = SendNotificationExecutor::new();
        assert_eq!(executor.default_timeout_secs(), 60);
    }
}
