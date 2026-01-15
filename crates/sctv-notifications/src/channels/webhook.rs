//! Generic webhook notification channel.

use std::collections::HashMap;
use std::time::{Duration, Instant};

use async_trait::async_trait;
use reqwest::{header, Client, Method};
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info, instrument, warn};
use url::Url;

use super::NotificationChannel;
use crate::error::{NotificationError, NotificationResult};
use crate::types::{DeliveryResult, Notification};

/// HTTP method for webhook requests.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum WebhookMethod {
    #[default]
    Post,
    Put,
    Patch,
}

impl From<WebhookMethod> for Method {
    fn from(method: WebhookMethod) -> Self {
        match method {
            WebhookMethod::Post => Method::POST,
            WebhookMethod::Put => Method::PUT,
            WebhookMethod::Patch => Method::PATCH,
        }
    }
}

/// Authentication method for webhooks.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WebhookAuth {
    /// No authentication.
    None,
    /// Bearer token authentication.
    Bearer {
        #[serde(skip_serializing)]
        token: String,
    },
    /// Basic authentication.
    Basic {
        username: String,
        #[serde(skip_serializing)]
        password: String,
    },
    /// API key in header.
    ApiKey {
        header_name: String,
        #[serde(skip_serializing)]
        api_key: String,
    },
    /// HMAC signature in header.
    HmacSignature {
        header_name: String,
        #[serde(skip_serializing)]
        secret: String,
        algorithm: HmacAlgorithm,
    },
}

impl Default for WebhookAuth {
    fn default() -> Self {
        Self::None
    }
}

/// HMAC algorithm for signature-based auth.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HmacAlgorithm {
    #[default]
    Sha256,
    Sha1,
}

/// Configuration for the generic webhook channel.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookConfig {
    /// Target webhook URL.
    pub url: String,
    /// HTTP method to use.
    #[serde(default)]
    pub method: WebhookMethod,
    /// Authentication configuration.
    #[serde(default)]
    pub auth: WebhookAuth,
    /// Additional headers to include.
    #[serde(default)]
    pub headers: HashMap<String, String>,
    /// Request timeout in seconds.
    #[serde(default = "default_timeout")]
    pub timeout_secs: u64,
    /// Number of retry attempts on failure.
    #[serde(default = "default_retries")]
    pub max_retries: u32,
    /// Delay between retries in milliseconds.
    #[serde(default = "default_retry_delay")]
    pub retry_delay_ms: u64,
    /// Custom payload template (JSON). Uses notification data if not set.
    pub payload_template: Option<serde_json::Value>,
    /// Whether the channel is enabled.
    #[serde(default = "default_enabled")]
    pub enabled: bool,
}

fn default_timeout() -> u64 {
    30
}

fn default_retries() -> u32 {
    3
}

fn default_retry_delay() -> u64 {
    1000
}

fn default_enabled() -> bool {
    true
}

impl Default for WebhookConfig {
    fn default() -> Self {
        Self {
            url: String::new(),
            method: WebhookMethod::default(),
            auth: WebhookAuth::default(),
            headers: HashMap::new(),
            timeout_secs: 30,
            max_retries: 3,
            retry_delay_ms: 1000,
            payload_template: None,
            enabled: false,
        }
    }
}

impl WebhookConfig {
    /// Creates a new webhook configuration builder.
    #[must_use]
    pub fn builder() -> WebhookConfigBuilder {
        WebhookConfigBuilder::default()
    }
}

/// Builder for `WebhookConfig`.
#[derive(Debug, Default)]
pub struct WebhookConfigBuilder {
    config: WebhookConfig,
}

impl WebhookConfigBuilder {
    /// Sets the webhook URL.
    #[must_use]
    pub fn url(mut self, url: impl Into<String>) -> Self {
        self.config.url = url.into();
        self
    }

    /// Sets the HTTP method.
    #[must_use]
    pub fn method(mut self, method: WebhookMethod) -> Self {
        self.config.method = method;
        self
    }

    /// Sets bearer token authentication.
    #[must_use]
    pub fn bearer_auth(mut self, token: impl Into<String>) -> Self {
        self.config.auth = WebhookAuth::Bearer {
            token: token.into(),
        };
        self
    }

    /// Sets basic authentication.
    #[must_use]
    pub fn basic_auth(mut self, username: impl Into<String>, password: impl Into<String>) -> Self {
        self.config.auth = WebhookAuth::Basic {
            username: username.into(),
            password: password.into(),
        };
        self
    }

    /// Sets API key authentication.
    #[must_use]
    pub fn api_key_auth(
        mut self,
        header_name: impl Into<String>,
        api_key: impl Into<String>,
    ) -> Self {
        self.config.auth = WebhookAuth::ApiKey {
            header_name: header_name.into(),
            api_key: api_key.into(),
        };
        self
    }

    /// Sets HMAC signature authentication.
    #[must_use]
    pub fn hmac_auth(
        mut self,
        header_name: impl Into<String>,
        secret: impl Into<String>,
        algorithm: HmacAlgorithm,
    ) -> Self {
        self.config.auth = WebhookAuth::HmacSignature {
            header_name: header_name.into(),
            secret: secret.into(),
            algorithm,
        };
        self
    }

    /// Adds a custom header.
    #[must_use]
    pub fn header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.config.headers.insert(name.into(), value.into());
        self
    }

    /// Sets the request timeout.
    #[must_use]
    pub fn timeout_secs(mut self, secs: u64) -> Self {
        self.config.timeout_secs = secs;
        self
    }

    /// Sets the maximum number of retries.
    #[must_use]
    pub fn max_retries(mut self, retries: u32) -> Self {
        self.config.max_retries = retries;
        self
    }

    /// Sets the retry delay.
    #[must_use]
    pub fn retry_delay_ms(mut self, delay_ms: u64) -> Self {
        self.config.retry_delay_ms = delay_ms;
        self
    }

    /// Sets a custom payload template.
    #[must_use]
    pub fn payload_template(mut self, template: serde_json::Value) -> Self {
        self.config.payload_template = Some(template);
        self
    }

    /// Sets whether the channel is enabled.
    #[must_use]
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.config.enabled = enabled;
        self
    }

    /// Builds the configuration.
    #[must_use]
    pub fn build(self) -> WebhookConfig {
        self.config
    }
}

/// Webhook payload sent to the target URL.
#[derive(Debug, Serialize)]
pub struct WebhookPayload {
    /// Notification ID.
    pub id: String,
    /// Event type.
    pub event_type: String,
    /// Severity level.
    pub severity: String,
    /// Notification title.
    pub title: String,
    /// Notification message.
    pub message: String,
    /// Timestamp in ISO 8601 format.
    pub timestamp: String,
    /// Additional context.
    pub context: WebhookContext,
}

/// Context data in webhook payload.
#[derive(Debug, Serialize)]
pub struct WebhookContext {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub package_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub package_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dashboard_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remediation: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alert_type: Option<String>,
    #[serde(skip_serializing_if = "serde_json::Value::is_null")]
    pub metadata: serde_json::Value,
}

/// Generic webhook notification channel.
pub struct WebhookChannel {
    config: WebhookConfig,
    client: Client,
}

impl WebhookChannel {
    /// Creates a new webhook channel with the given configuration.
    #[must_use]
    pub fn new(config: WebhookConfig) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(config.timeout_secs))
            .build()
            .expect("Failed to create HTTP client");

        Self { config, client }
    }

    /// Builds the webhook payload.
    fn build_payload(&self, notification: &Notification) -> serde_json::Value {
        if let Some(template) = &self.config.payload_template {
            return template.clone();
        }

        let payload = WebhookPayload {
            id: notification.id.clone(),
            event_type: "supply_chain_alert".to_string(),
            severity: notification.severity.to_string().to_lowercase(),
            title: notification.title.clone(),
            message: notification.message.clone(),
            timestamp: notification.created_at.to_rfc3339(),
            context: WebhookContext {
                project_name: notification.context.project_name.clone(),
                package_name: notification.context.package_name.clone(),
                package_version: notification.context.package_version.clone(),
                dashboard_url: notification.context.dashboard_url.clone(),
                remediation: notification.context.remediation.clone(),
                alert_type: notification.context.alert_type.clone(),
                metadata: notification.context.metadata.clone(),
            },
        };

        serde_json::to_value(payload).expect("Failed to serialize payload")
    }

    /// Computes HMAC signature for the payload.
    fn compute_signature(&self, payload: &[u8], secret: &str, algorithm: HmacAlgorithm) -> String {
        use hmac::{Hmac, Mac};
        use sha1::Sha1;
        use sha2::Sha256;

        match algorithm {
            HmacAlgorithm::Sha256 => {
                type HmacSha256 = Hmac<Sha256>;

                let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
                    .expect("HMAC can take key of any size");
                mac.update(payload);
                let result = mac.finalize();
                format!("sha256={}", hex::encode(result.into_bytes()))
            }
            HmacAlgorithm::Sha1 => {
                type HmacSha1 = Hmac<Sha1>;

                let mut mac = HmacSha1::new_from_slice(secret.as_bytes())
                    .expect("HMAC can take key of any size");
                mac.update(payload);
                let result = mac.finalize();
                format!("sha1={}", hex::encode(result.into_bytes()))
            }
        }
    }

    /// Sends the webhook request with retries.
    async fn send_with_retry(
        &self,
        notification: &Notification,
    ) -> NotificationResult<DeliveryResult> {
        let payload = self.build_payload(notification);
        let payload_bytes = serde_json::to_vec(&payload)?;

        let mut last_error = None;

        for attempt in 0..=self.config.max_retries {
            if attempt > 0 {
                debug!(
                    notification_id = %notification.id,
                    attempt,
                    "Retrying webhook request"
                );
                tokio::time::sleep(Duration::from_millis(
                    self.config.retry_delay_ms * u64::from(attempt),
                ))
                .await;
            }

            let start = Instant::now();

            match self.send_request(&payload_bytes).await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    warn!(
                        notification_id = %notification.id,
                        attempt,
                        error = %e,
                        "Webhook request failed"
                    );
                    last_error = Some((e, start.elapsed().as_millis() as u64));
                }
            }
        }

        let (error, duration_ms) = last_error.expect("At least one attempt was made");
        Ok(DeliveryResult::failure(duration_ms, error.to_string()))
    }

    /// Sends a single webhook request.
    async fn send_request(&self, payload: &[u8]) -> NotificationResult<DeliveryResult> {
        let start = Instant::now();

        let mut request = self
            .client
            .request(self.config.method.into(), &self.config.url)
            .header(header::CONTENT_TYPE, "application/json");

        // Apply authentication
        request = match &self.config.auth {
            WebhookAuth::None => request,
            WebhookAuth::Bearer { token } => request.bearer_auth(token),
            WebhookAuth::Basic { username, password } => request.basic_auth(username, Some(password)),
            WebhookAuth::ApiKey {
                header_name,
                api_key,
            } => request.header(header_name, api_key),
            WebhookAuth::HmacSignature {
                header_name,
                secret,
                algorithm,
            } => {
                let signature = self.compute_signature(payload, secret, *algorithm);
                request.header(header_name, signature)
            }
        };

        // Apply custom headers
        for (name, value) in &self.config.headers {
            request = request.header(name, value);
        }

        let response = request.body(payload.to_vec()).send().await?;

        let duration_ms = start.elapsed().as_millis() as u64;
        let status = response.status();

        if status.is_success() {
            let body: serde_json::Value = response
                .json()
                .await
                .unwrap_or(serde_json::Value::Null);

            Ok(DeliveryResult::success_with_response(
                duration_ms,
                serde_json::json!({
                    "status": status.as_u16(),
                    "body": body,
                }),
            ))
        } else if status.as_u16() == 429 {
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
            Err(NotificationError::WebhookDelivery(format!(
                "HTTP {status}: {body}"
            )))
        }
    }
}

#[async_trait]
impl NotificationChannel for WebhookChannel {
    fn name(&self) -> &'static str {
        "webhook"
    }

    fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    #[instrument(skip(self, notification), fields(channel = "webhook"))]
    async fn send(&self, notification: &Notification) -> NotificationResult<DeliveryResult> {
        if !self.is_enabled() {
            return Err(NotificationError::ChannelDisabled);
        }

        debug!(
            notification_id = %notification.id,
            url = %self.config.url,
            "Sending webhook notification"
        );

        let result = self.send_with_retry(notification).await?;

        if result.success {
            info!(
                notification_id = %notification.id,
                duration_ms = result.duration_ms,
                "Webhook notification sent successfully"
            );
        } else {
            error!(
                notification_id = %notification.id,
                error = ?result.error,
                "Failed to send webhook notification"
            );
        }

        Ok(result)
    }

    async fn validate(&self) -> NotificationResult<()> {
        if self.config.url.is_empty() {
            return Err(NotificationError::InvalidConfig(
                "Webhook URL is required".to_string(),
            ));
        }

        // Validate URL format
        let _url = Url::parse(&self.config.url)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::NotificationContext;
    use sctv_core::Severity;

    #[test]
    fn test_webhook_config_builder() {
        let config = WebhookConfig::builder()
            .url("https://api.example.com/webhooks/alerts")
            .method(WebhookMethod::Post)
            .bearer_auth("secret-token")
            .header("X-Custom-Header", "custom-value")
            .timeout_secs(60)
            .max_retries(5)
            .enabled(true)
            .build();

        assert_eq!(config.url, "https://api.example.com/webhooks/alerts");
        assert_eq!(config.method, WebhookMethod::Post);
        assert!(matches!(config.auth, WebhookAuth::Bearer { .. }));
        assert_eq!(config.headers.get("X-Custom-Header"), Some(&"custom-value".to_string()));
        assert_eq!(config.max_retries, 5);
    }

    #[test]
    fn test_build_payload() {
        let config = WebhookConfig::builder()
            .url("https://api.example.com/webhook")
            .enabled(true)
            .build();

        let channel = WebhookChannel::new(config);

        let notification = Notification::new(
            Severity::Critical,
            "Critical Security Alert",
            "Package tampering detected in production dependency.",
        )
        .with_context(
            NotificationContext::new()
                .with_project("api-server")
                .with_package("axios", "0.21.1"),
        );

        let payload = channel.build_payload(&notification);

        assert_eq!(payload["event_type"], "supply_chain_alert");
        assert_eq!(payload["severity"], "critical");
        assert_eq!(payload["title"], "Critical Security Alert");
        assert_eq!(payload["context"]["project_name"], "api-server");
    }

    #[test]
    fn test_custom_payload_template() {
        let template = serde_json::json!({
            "alert": {
                "type": "security",
                "source": "sctv"
            }
        });

        let config = WebhookConfig::builder()
            .url("https://api.example.com/webhook")
            .payload_template(template.clone())
            .enabled(true)
            .build();

        let channel = WebhookChannel::new(config);

        let notification = Notification::new(Severity::High, "Test", "Test message");
        let payload = channel.build_payload(&notification);

        assert_eq!(payload, template);
    }
}
