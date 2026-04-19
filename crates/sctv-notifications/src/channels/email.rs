//! Email notification channel using SMTP.

use std::fmt::Write as _;
use std::time::Instant;

use async_trait::async_trait;
use lettre::message::{header::ContentType, Mailbox};
use lettre::transport::smtp::authentication::Credentials;
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info, instrument};

use super::NotificationChannel;
use crate::error::{NotificationError, NotificationResult};
use crate::types::{DeliveryResult, Notification};

/// Configuration for the email notification channel.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailConfig {
    /// SMTP server hostname.
    pub smtp_host: String,
    /// SMTP server port (typically 587 for TLS, 465 for SSL).
    pub smtp_port: u16,
    /// Username for SMTP authentication.
    pub smtp_username: String,
    /// Password for SMTP authentication.
    #[serde(skip_serializing)]
    pub smtp_password: String,
    /// Sender email address.
    pub from_address: String,
    /// Sender display name.
    pub from_name: String,
    /// Recipient email addresses.
    pub to_addresses: Vec<String>,
    /// Whether TLS is required.
    #[serde(default = "default_tls")]
    pub use_tls: bool,
    /// Whether the channel is enabled.
    #[serde(default = "default_enabled")]
    pub enabled: bool,
}

const fn default_tls() -> bool {
    true
}

const fn default_enabled() -> bool {
    true
}

impl Default for EmailConfig {
    fn default() -> Self {
        Self {
            smtp_host: "smtp.example.com".to_string(),
            smtp_port: 587,
            smtp_username: String::new(),
            smtp_password: String::new(),
            from_address: "noreply@example.com".to_string(),
            from_name: "Supply Chain Trust Verifier".to_string(),
            to_addresses: Vec::new(),
            use_tls: true,
            enabled: false,
        }
    }
}

impl EmailConfig {
    /// Creates a new email configuration builder.
    #[must_use]
    pub fn builder() -> EmailConfigBuilder {
        EmailConfigBuilder::default()
    }
}

/// Builder for `EmailConfig`.
#[derive(Debug, Default)]
pub struct EmailConfigBuilder {
    config: EmailConfig,
}

impl EmailConfigBuilder {
    /// Sets the SMTP server hostname.
    #[must_use]
    pub fn smtp_host(mut self, host: impl Into<String>) -> Self {
        self.config.smtp_host = host.into();
        self
    }

    /// Sets the SMTP server port.
    #[must_use]
    pub const fn smtp_port(mut self, port: u16) -> Self {
        self.config.smtp_port = port;
        self
    }

    /// Sets the SMTP credentials.
    #[must_use]
    pub fn credentials(mut self, username: impl Into<String>, password: impl Into<String>) -> Self {
        self.config.smtp_username = username.into();
        self.config.smtp_password = password.into();
        self
    }

    /// Sets the sender address and name.
    #[must_use]
    pub fn from(mut self, address: impl Into<String>, name: impl Into<String>) -> Self {
        self.config.from_address = address.into();
        self.config.from_name = name.into();
        self
    }

    /// Adds a recipient address.
    #[must_use]
    pub fn to(mut self, address: impl Into<String>) -> Self {
        self.config.to_addresses.push(address.into());
        self
    }

    /// Adds multiple recipient addresses.
    // Builder methods consume `self` to enable method chaining; the `to_*` naming
    // convention for builder setters is idiomatic here even on non-Copy types.
    #[allow(clippy::wrong_self_convention)]
    #[must_use]
    pub fn to_many(mut self, addresses: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.config
            .to_addresses
            .extend(addresses.into_iter().map(Into::into));
        self
    }

    /// Sets whether to use TLS.
    #[must_use]
    pub const fn use_tls(mut self, use_tls: bool) -> Self {
        self.config.use_tls = use_tls;
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
    pub fn build(self) -> EmailConfig {
        self.config
    }
}

/// Email notification channel using SMTP.
pub struct EmailChannel {
    config: EmailConfig,
}

impl EmailChannel {
    /// Creates a new email channel with the given configuration.
    #[must_use]
    pub const fn new(config: EmailConfig) -> Self {
        Self { config }
    }

    /// Creates the SMTP transport.
    fn create_transport(&self) -> NotificationResult<AsyncSmtpTransport<Tokio1Executor>> {
        let creds = Credentials::new(
            self.config.smtp_username.clone(),
            self.config.smtp_password.clone(),
        );

        let transport = if self.config.use_tls {
            AsyncSmtpTransport::<Tokio1Executor>::relay(&self.config.smtp_host)
                .map_err(|e| NotificationError::SmtpTransport(e.to_string()))?
                .credentials(creds)
                .port(self.config.smtp_port)
                .build()
        } else {
            AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(&self.config.smtp_host)
                .credentials(creds)
                .port(self.config.smtp_port)
                .build()
        };

        Ok(transport)
    }

    /// Builds the email message.
    fn build_message(&self, notification: &Notification) -> NotificationResult<Message> {
        let from_mailbox: Mailbox =
            format!("{} <{}>", self.config.from_name, self.config.from_address)
                .parse()
                .map_err(|e: lettre::address::AddressError| {
                    NotificationError::InvalidConfig(format!("Invalid from address: {e}"))
                })?;

        let mut builder = Message::builder().from(from_mailbox).subject(format!(
            "[{}] {}",
            notification.severity, notification.title
        ));

        for to_addr in &self.config.to_addresses {
            let to_mailbox: Mailbox =
                to_addr
                    .parse()
                    .map_err(|e: lettre::address::AddressError| {
                        NotificationError::InvalidConfig(format!(
                            "Invalid to address {to_addr}: {e}"
                        ))
                    })?;
            builder = builder.to(to_mailbox);
        }

        let body = Self::format_body(notification);

        builder
            .header(ContentType::TEXT_PLAIN)
            .body(body)
            .map_err(|e| NotificationError::EmailDelivery(e.to_string()))
    }

    /// Formats the email body.
    fn format_body(notification: &Notification) -> String {
        let mut body = String::new();

        writeln!(body, "Severity: {}", notification.severity)
            .expect("write to String is infallible");
        write!(body, "Time: {}\n\n", notification.created_at)
            .expect("write to String is infallible");
        body.push_str(&notification.message);
        body.push_str("\n\n");

        if let Some(project) = &notification.context.project_name {
            writeln!(body, "Project: {project}").expect("write to String is infallible");
        }

        if let Some(package) = &notification.context.package_name {
            write!(body, "Package: {package}").expect("write to String is infallible");
            if let Some(version) = &notification.context.package_version {
                write!(body, "@{version}").expect("write to String is infallible");
            }
            body.push('\n');
        }

        if let Some(url) = &notification.context.dashboard_url {
            write!(body, "\nView in dashboard: {url}\n").expect("write to String is infallible");
        }

        if let Some(remediation) = &notification.context.remediation {
            write!(body, "\nRemediation:\n{remediation}\n").expect("write to String is infallible");
        }

        body.push_str("\n---\nSupply Chain Trust Verifier\n");

        body
    }
}

#[async_trait]
impl NotificationChannel for EmailChannel {
    fn name(&self) -> &'static str {
        "email"
    }

    fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    #[instrument(skip(self, notification), fields(channel = "email"))]
    async fn send(&self, notification: &Notification) -> NotificationResult<DeliveryResult> {
        if !self.is_enabled() {
            return Err(NotificationError::ChannelDisabled);
        }

        if self.config.to_addresses.is_empty() {
            return Err(NotificationError::InvalidConfig(
                "No recipient addresses configured".to_string(),
            ));
        }

        let start = Instant::now();

        debug!(
            notification_id = %notification.id,
            recipients = ?self.config.to_addresses,
            "Sending email notification"
        );

        let transport = self.create_transport()?;
        let message = self.build_message(notification)?;

        match transport.send(message).await {
            Ok(response) => {
                // as_millis() returns u128; elapsed time in ms will never exceed u64::MAX (~585M years).
                let duration_ms = u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX);
                info!(
                    notification_id = %notification.id,
                    duration_ms,
                    "Email notification sent successfully"
                );

                Ok(DeliveryResult::success_with_response(
                    duration_ms,
                    serde_json::json!({
                        "message": response.message().collect::<Vec<_>>(),
                        "code": response.code().to_string(),
                    }),
                ))
            }
            Err(e) => {
                // as_millis() returns u128; elapsed time in ms will never exceed u64::MAX (~585M years).
                let duration_ms = u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX);
                error!(
                    notification_id = %notification.id,
                    error = %e,
                    "Failed to send email notification"
                );

                Ok(DeliveryResult::failure(duration_ms, e.to_string()))
            }
        }
    }

    async fn validate(&self) -> NotificationResult<()> {
        if self.config.smtp_host.is_empty() {
            return Err(NotificationError::InvalidConfig(
                "SMTP host is required".to_string(),
            ));
        }

        if self.config.from_address.is_empty() {
            return Err(NotificationError::InvalidConfig(
                "From address is required".to_string(),
            ));
        }

        // Validate the from address format
        let _: Mailbox = format!("{} <{}>", self.config.from_name, self.config.from_address)
            .parse()
            .map_err(|e: lettre::address::AddressError| {
                NotificationError::InvalidConfig(format!("Invalid from address: {e}"))
            })?;

        // Validate all to addresses
        for addr in &self.config.to_addresses {
            let _: Mailbox = addr.parse().map_err(|e: lettre::address::AddressError| {
                NotificationError::InvalidConfig(format!("Invalid to address {addr}: {e}"))
            })?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sctv_core::Severity;

    #[test]
    fn test_email_config_builder() {
        let config = EmailConfig::builder()
            .smtp_host("smtp.gmail.com")
            .smtp_port(587)
            .credentials("user@gmail.com", "password")
            .from("alerts@example.com", "SCTV Alerts")
            .to("admin@example.com")
            .to("security@example.com")
            .enabled(true)
            .build();

        assert_eq!(config.smtp_host, "smtp.gmail.com");
        assert_eq!(config.smtp_port, 587);
        assert_eq!(config.to_addresses.len(), 2);
        assert!(config.enabled);
    }

    #[test]
    fn test_format_body() {
        let notification = Notification::new(
            Severity::High,
            "Security Alert",
            "A potential typosquatting attack was detected.",
        )
        .with_context(
            crate::types::NotificationContext::new()
                .with_project("my-project")
                .with_package("lodash-utils", "1.0.0"),
        );

        let body = EmailChannel::format_body(&notification);

        assert!(body.contains("Severity: High"));
        assert!(body.contains("Project: my-project"));
        assert!(body.contains("Package: lodash-utils@1.0.0"));
    }
}
