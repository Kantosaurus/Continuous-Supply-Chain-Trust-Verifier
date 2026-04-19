//! Alert notification system for the Supply Chain Trust Verifier.
//!
//! This crate provides a modular notification system supporting multiple delivery channels:
//!
//! - **Email** - SMTP-based email notifications via [`channels::EmailChannel`]
//! - **Slack** - Slack webhook integration via [`channels::SlackChannel`]
//! - **Teams** - Microsoft Teams webhook integration via [`channels::TeamsChannel`]
//! - **`PagerDuty`** - `PagerDuty` Events API v2 via [`channels::PagerDutyChannel`]
//! - **Webhook** - Generic HTTP webhook support via [`channels::WebhookChannel`]
//!
//! # Architecture
//!
//! The notification system follows a channel-based architecture where each delivery
//! mechanism implements the [`channels::NotificationChannel`] trait. The
//! [`NotificationService`] coordinates delivery across multiple channels with support
//! for parallel delivery, retry logic, and severity-based filtering.
//!
//! # Example
//!
//! ```rust,no_run
//! use sctv_notifications::{
//!     NotificationService, NotificationServiceConfig, Notification,
//!     channels::{SlackConfig, WebhookConfig},
//! };
//! use sctv_core::Severity;
//!
//! # async fn example() {
//! // Build a notification service with multiple channels
//! let service = NotificationService::builder()
//!     .config(NotificationServiceConfig::builder()
//!         .parallel_delivery(true)
//!         .continue_on_failure(true)
//!         .build())
//!     .slack(
//!         SlackConfig::builder()
//!             .webhook_url("https://hooks.slack.com/services/xxx/yyy/zzz")
//!             .channel("#security-alerts")
//!             .enabled(true)
//!             .build(),
//!         Severity::Medium,
//!     )
//!     .webhook(
//!         WebhookConfig::builder()
//!             .url("https://api.example.com/webhooks/alerts")
//!             .bearer_auth("secret-token")
//!             .enabled(true)
//!             .build(),
//!         Severity::High,
//!     )
//!     .build();
//!
//! // Create and send a notification
//! let notification = Notification::new(
//!     Severity::High,
//!     "Typosquatting Detected",
//!     "Package 'lodash-utils' may be a typosquatting attempt.",
//! );
//!
//! let result = service.send(&notification).await;
//! println!("Sent to {} channels successfully", result.successful);
//! # }
//! ```
//!
//! # Integration with Worker System
//!
//! This crate is designed to integrate with the `sctv-worker` job queue system.
//! Notifications are typically queued as background jobs and processed asynchronously
//! to ensure reliable delivery with automatic retries.

pub mod channels;
pub mod error;
pub mod service;
pub mod types;

// Re-export main types at crate root for convenience
pub use channels::{
    EmailChannel, EmailConfig, NotificationChannel, PagerDutyChannel, PagerDutyConfig,
    SlackChannel, SlackConfig, TeamsChannel, TeamsConfig, WebhookChannel, WebhookConfig,
};
pub use error::{NotificationError, NotificationResult};
pub use service::{MultiChannelResult, NotificationService, NotificationServiceConfig};
pub use types::{DeliveryResult, Notification, NotificationContext};
