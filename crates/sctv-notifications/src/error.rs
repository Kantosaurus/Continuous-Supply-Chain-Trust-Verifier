//! Error types for the notification system.

use thiserror::Error;

/// Errors that can occur during notification delivery.
#[derive(Debug, Error)]
pub enum NotificationError {
    /// Failed to send email notification.
    #[error("email delivery failed: {0}")]
    EmailDelivery(String),

    /// SMTP transport error.
    #[error("SMTP transport error: {0}")]
    SmtpTransport(String),

    /// Failed to send webhook notification.
    #[error("webhook delivery failed: {0}")]
    WebhookDelivery(String),

    /// Invalid webhook URL.
    #[error("invalid webhook URL: {0}")]
    InvalidWebhookUrl(String),

    /// HTTP request failed.
    #[error("HTTP request failed: {0}")]
    HttpRequest(String),

    /// Failed to serialize notification payload.
    #[error("serialization error: {0}")]
    Serialization(String),

    /// Invalid channel configuration.
    #[error("invalid channel configuration: {0}")]
    InvalidConfig(String),

    /// Channel is disabled.
    #[error("notification channel is disabled")]
    ChannelDisabled,

    /// Rate limit exceeded.
    #[error("rate limit exceeded, retry after {retry_after_secs} seconds")]
    RateLimited { retry_after_secs: u64 },

    /// Authentication failed.
    #[error("authentication failed: {0}")]
    AuthenticationFailed(String),

    /// Timeout waiting for response.
    #[error("request timed out after {timeout_secs} seconds")]
    Timeout { timeout_secs: u64 },
}

impl From<reqwest::Error> for NotificationError {
    fn from(err: reqwest::Error) -> Self {
        if err.is_timeout() {
            Self::Timeout { timeout_secs: 30 }
        } else if err.is_request() {
            Self::HttpRequest(err.to_string())
        } else {
            Self::WebhookDelivery(err.to_string())
        }
    }
}

impl From<serde_json::Error> for NotificationError {
    fn from(err: serde_json::Error) -> Self {
        Self::Serialization(err.to_string())
    }
}

impl From<url::ParseError> for NotificationError {
    fn from(err: url::ParseError) -> Self {
        Self::InvalidWebhookUrl(err.to_string())
    }
}

/// Result type for notification operations.
pub type NotificationResult<T> = Result<T, NotificationError>;
