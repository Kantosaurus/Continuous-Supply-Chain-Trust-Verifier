//! Notification channels.
//!
//! This module provides implementations for various notification delivery channels:
//! - [`EmailChannel`] - SMTP-based email delivery
//! - [`SlackChannel`] - Slack webhook integration
//! - [`TeamsChannel`] - Microsoft Teams webhook integration
//! - [`PagerDutyChannel`] - `PagerDuty` Events API v2 integration
//! - [`WebhookChannel`] - Generic HTTP webhook support

mod email;
mod pagerduty;
mod slack;
mod teams;
mod webhook;

pub use email::{EmailChannel, EmailConfig};
pub use pagerduty::{PagerDutyChannel, PagerDutyConfig};
pub use slack::{SlackChannel, SlackConfig};
pub use teams::{TeamsChannel, TeamsConfig};
pub use webhook::{HmacAlgorithm, WebhookAuth, WebhookChannel, WebhookConfig, WebhookMethod};

use async_trait::async_trait;

use crate::error::NotificationResult;
use crate::types::{DeliveryResult, Notification};

/// Trait for notification delivery channels.
///
/// Each channel implementation handles the specifics of delivering
/// notifications to a particular service (email, Slack, webhooks, etc.).
#[async_trait]
pub trait NotificationChannel: Send + Sync {
    /// Returns the name of this channel for logging purposes.
    fn name(&self) -> &'static str;

    /// Checks if this channel is currently enabled.
    fn is_enabled(&self) -> bool;

    /// Delivers a notification through this channel.
    ///
    /// # Errors
    ///
    /// Returns an error if the delivery fails due to network issues,
    /// authentication problems, or invalid configuration.
    async fn send(&self, notification: &Notification) -> NotificationResult<DeliveryResult>;

    /// Validates the channel configuration.
    ///
    /// This can be used to verify credentials or connectivity before
    /// attempting to send notifications.
    async fn validate(&self) -> NotificationResult<()> {
        Ok(())
    }
}
