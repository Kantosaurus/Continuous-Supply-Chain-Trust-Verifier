//! Integration tests for notification channels using wiremock.

use sctv_core::Severity;
use sctv_notifications::{
    channels::{
        NotificationChannel, PagerDutyChannel, PagerDutyConfig, SlackChannel, SlackConfig,
        TeamsChannel, TeamsConfig, WebhookChannel, WebhookConfig, WebhookMethod,
    },
    error::NotificationError,
    types::{Notification, NotificationContext},
};
use wiremock::{
    matchers::{header, method, path},
    Mock, MockServer, ResponseTemplate,
};

/// Creates a test notification with full context.
fn create_test_notification(severity: Severity) -> Notification {
    Notification::new(
        severity,
        "Security Alert: Typosquatting Detected",
        "Package 'lodash-utils' appears to be a typosquatting attempt targeting 'lodash'.",
    )
    .with_context(
        NotificationContext::new()
            .with_project("my-application")
            .with_package("lodash-utils", "1.0.0")
            .with_dashboard_url("https://sctv.example.com/alerts/123")
            .with_remediation("Remove the suspicious package and verify your dependencies."),
    )
}

// =============================================================================
// Slack Channel Tests
// =============================================================================

mod slack {
    use super::*;

    #[tokio::test]
    async fn test_successful_delivery() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/"))
            .and(header("content-type", "application/json"))
            .respond_with(ResponseTemplate::new(200).set_body_string("ok"))
            .expect(1)
            .mount(&mock_server)
            .await;

        let config = SlackConfig::builder()
            .webhook_url(mock_server.uri())
            .channel("#security-alerts")
            .username("SCTV Bot")
            .icon_emoji(":shield:")
            .enabled(true)
            .build();

        let channel = SlackChannel::new(config);
        let notification = create_test_notification(Severity::High);

        let result = channel.send(&notification).await.unwrap();

        assert!(result.success);
        assert!(result.duration_ms > 0);
    }

    #[tokio::test]
    async fn test_rate_limiting() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .respond_with(
                ResponseTemplate::new(429)
                    .insert_header("Retry-After", "120")
                    .set_body_string("rate limited"),
            )
            .expect(1)
            .mount(&mock_server)
            .await;

        let config = SlackConfig::builder()
            .webhook_url(mock_server.uri())
            .enabled(true)
            .build();

        let channel = SlackChannel::new(config);
        let notification = create_test_notification(Severity::Medium);

        let result = channel.send(&notification).await;

        assert!(matches!(
            result,
            Err(NotificationError::RateLimited {
                retry_after_secs: 120
            })
        ));
    }

    #[tokio::test]
    async fn test_server_error() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(500).set_body_string("internal error"))
            .expect(1)
            .mount(&mock_server)
            .await;

        let config = SlackConfig::builder()
            .webhook_url(mock_server.uri())
            .enabled(true)
            .build();

        let channel = SlackChannel::new(config);
        let notification = create_test_notification(Severity::Critical);

        let result = channel.send(&notification).await.unwrap();

        assert!(!result.success);
        assert!(result.error.is_some());
        assert!(result.error.unwrap().contains("500"));
    }

    #[tokio::test]
    async fn test_disabled_channel() {
        let config = SlackConfig::builder()
            .webhook_url("https://hooks.slack.com/test")
            .enabled(false)
            .build();

        let channel = SlackChannel::new(config);
        let notification = create_test_notification(Severity::Info);

        let result = channel.send(&notification).await;

        assert!(matches!(result, Err(NotificationError::ChannelDisabled)));
    }

    #[tokio::test]
    async fn test_payload_structure() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(header("content-type", "application/json"))
            .respond_with(ResponseTemplate::new(200).set_body_string("ok"))
            .expect(1)
            .mount(&mock_server)
            .await;

        let config = SlackConfig::builder()
            .webhook_url(mock_server.uri())
            .channel("#alerts")
            .username("Test Bot")
            .enabled(true)
            .build();

        let channel = SlackChannel::new(config);
        let notification = create_test_notification(Severity::High);

        let result = channel.send(&notification).await.unwrap();
        assert!(result.success);

        // Verify the mock was called (payload was sent correctly)
        let requests = mock_server.received_requests().await.unwrap();
        assert_eq!(requests.len(), 1);

        let body: serde_json::Value = serde_json::from_slice(&requests[0].body).unwrap();
        assert_eq!(body["channel"], "#alerts");
        assert_eq!(body["username"], "Test Bot");
        assert!(body["attachments"].is_array());
        assert_eq!(body["attachments"][0]["color"], "#fd7e14"); // High severity
    }
}

// =============================================================================
// Teams Channel Tests
// =============================================================================

mod teams {
    use super::*;

    #[tokio::test]
    async fn test_successful_delivery() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(header("content-type", "application/json"))
            .respond_with(ResponseTemplate::new(200).set_body_string("1"))
            .expect(1)
            .mount(&mock_server)
            .await;

        let config = TeamsConfig::builder()
            .webhook_url(mock_server.uri())
            .enabled(true)
            .build();

        let channel = TeamsChannel::new(config);
        let notification = create_test_notification(Severity::Critical);

        let result = channel.send(&notification).await.unwrap();

        assert!(result.success);
        assert!(result.duration_ms > 0);
    }

    #[tokio::test]
    async fn test_rate_limiting() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .respond_with(
                ResponseTemplate::new(429)
                    .insert_header("Retry-After", "60")
                    .set_body_string("throttled"),
            )
            .expect(1)
            .mount(&mock_server)
            .await;

        let config = TeamsConfig::builder()
            .webhook_url(mock_server.uri())
            .enabled(true)
            .build();

        let channel = TeamsChannel::new(config);
        let notification = create_test_notification(Severity::High);

        let result = channel.send(&notification).await;

        assert!(matches!(
            result,
            Err(NotificationError::RateLimited {
                retry_after_secs: 60
            })
        ));
    }

    #[tokio::test]
    async fn test_adaptive_card_payload() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(200).set_body_string("1"))
            .expect(1)
            .mount(&mock_server)
            .await;

        let config = TeamsConfig::builder()
            .webhook_url(mock_server.uri())
            .enabled(true)
            .build();

        let channel = TeamsChannel::new(config);
        let notification = create_test_notification(Severity::Medium);

        let result = channel.send(&notification).await.unwrap();
        assert!(result.success);

        let requests = mock_server.received_requests().await.unwrap();
        assert_eq!(requests.len(), 1);

        let body: serde_json::Value = serde_json::from_slice(&requests[0].body).unwrap();
        assert_eq!(body["type"], "message");
        assert!(body["attachments"].is_array());
        assert_eq!(
            body["attachments"][0]["contentType"],
            "application/vnd.microsoft.card.adaptive"
        );
        assert_eq!(body["attachments"][0]["content"]["type"], "AdaptiveCard");
        assert_eq!(body["attachments"][0]["content"]["version"], "1.4");
    }

    #[tokio::test]
    async fn test_disabled_channel() {
        let config = TeamsConfig::builder()
            .webhook_url("https://outlook.office.com/webhook/test")
            .enabled(false)
            .build();

        let channel = TeamsChannel::new(config);
        let notification = create_test_notification(Severity::Low);

        let result = channel.send(&notification).await;

        assert!(matches!(result, Err(NotificationError::ChannelDisabled)));
    }
}

// =============================================================================
// PagerDuty Channel Tests
// =============================================================================

mod pagerduty {
    use super::*;

    #[tokio::test]
    async fn test_successful_trigger() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/v2/enqueue"))
            .and(header("content-type", "application/json"))
            .respond_with(ResponseTemplate::new(202).set_body_json(serde_json::json!({
                "status": "success",
                "message": "Event processed",
                "dedup_key": "test-dedup-key-123"
            })))
            .expect(1)
            .mount(&mock_server)
            .await;

        let config = PagerDutyConfig::builder()
            .routing_key("12345678901234567890123456789012")
            .source("sctv-test")
            .component("api-server")
            .api_url(format!("{}/v2/enqueue", mock_server.uri()))
            .enabled(true)
            .build();

        let channel = PagerDutyChannel::new(config);
        let notification = create_test_notification(Severity::Critical);

        let result = channel.send(&notification).await.unwrap();

        assert!(result.success);
        assert!(result.response.is_some());

        let response = result.response.unwrap();
        assert_eq!(response["status"], "success");
        assert_eq!(response["dedup_key"], "test-dedup-key-123");
    }

    #[tokio::test]
    async fn test_rate_limiting() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .respond_with(
                ResponseTemplate::new(429)
                    .insert_header("Retry-After", "30")
                    .set_body_string("rate limited"),
            )
            .expect(1)
            .mount(&mock_server)
            .await;

        let config = PagerDutyConfig::builder()
            .routing_key("12345678901234567890123456789012")
            .api_url(format!("{}/v2/enqueue", mock_server.uri()))
            .enabled(true)
            .build();

        let channel = PagerDutyChannel::new(config);
        let notification = create_test_notification(Severity::High);

        let result = channel.send(&notification).await;

        assert!(matches!(
            result,
            Err(NotificationError::RateLimited {
                retry_after_secs: 30
            })
        ));
    }

    #[tokio::test]
    async fn test_invalid_event_error() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(400).set_body_json(serde_json::json!({
                "status": "invalid event",
                "message": "Missing required field: routing_key"
            })))
            .expect(1)
            .mount(&mock_server)
            .await;

        let config = PagerDutyConfig::builder()
            .routing_key("12345678901234567890123456789012")
            .api_url(format!("{}/v2/enqueue", mock_server.uri()))
            .enabled(true)
            .build();

        let channel = PagerDutyChannel::new(config);
        let notification = create_test_notification(Severity::Medium);

        let result = channel.send(&notification).await;

        assert!(matches!(result, Err(NotificationError::InvalidConfig(_))));
    }

    #[tokio::test]
    async fn test_event_payload_structure() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(202).set_body_json(serde_json::json!({
                "status": "success",
                "message": "Event processed",
                "dedup_key": "key"
            })))
            .expect(1)
            .mount(&mock_server)
            .await;

        let config = PagerDutyConfig::builder()
            .routing_key("12345678901234567890123456789012")
            .source("test-source")
            .component("test-component")
            .group("test-group")
            .api_url(format!("{}/v2/enqueue", mock_server.uri()))
            .enabled(true)
            .build();

        let channel = PagerDutyChannel::new(config);
        let notification = create_test_notification(Severity::Critical);

        let result = channel.send(&notification).await.unwrap();
        assert!(result.success);

        let requests = mock_server.received_requests().await.unwrap();
        assert_eq!(requests.len(), 1);

        let body: serde_json::Value = serde_json::from_slice(&requests[0].body).unwrap();
        assert_eq!(body["routing_key"], "12345678901234567890123456789012");
        assert_eq!(body["event_action"], "trigger");
        assert!(body["dedup_key"].is_string());
        assert_eq!(body["payload"]["source"], "test-source");
        assert_eq!(body["payload"]["severity"], "critical");
        assert!(body["payload"]["summary"]
            .as_str()
            .unwrap()
            .contains("Critical"));
        assert!(body["payload"]["custom_details"]["project"].is_string());
        assert!(body["links"].is_array());
    }

    #[tokio::test]
    async fn test_severity_mapping() {
        let mock_server = MockServer::start().await;

        // Test all severity levels
        let severities = [
            (Severity::Critical, "critical"),
            (Severity::High, "error"),
            (Severity::Medium, "warning"),
            (Severity::Low, "info"),
            (Severity::Info, "info"),
        ];

        for (severity, expected_pd_severity) in severities {
            Mock::given(method("POST"))
                .respond_with(ResponseTemplate::new(202).set_body_json(serde_json::json!({
                    "status": "success",
                    "message": "Event processed",
                    "dedup_key": "key"
                })))
                .expect(1)
                .mount(&mock_server)
                .await;

            let config = PagerDutyConfig::builder()
                .routing_key("12345678901234567890123456789012")
                .api_url(format!("{}/v2/enqueue", mock_server.uri()))
                .enabled(true)
                .build();

            let channel = PagerDutyChannel::new(config);
            let notification = Notification::new(severity, "Test", "Test message");

            channel.send(&notification).await.unwrap();

            let requests = mock_server.received_requests().await.unwrap();
            let last_request = requests.last().unwrap();
            let body: serde_json::Value = serde_json::from_slice(&last_request.body).unwrap();

            assert_eq!(
                body["payload"]["severity"], expected_pd_severity,
                "Severity {:?} should map to {}",
                severity, expected_pd_severity
            );

            mock_server.reset().await;
        }
    }
}

// =============================================================================
// Webhook Channel Tests
// =============================================================================

mod webhook {
    use super::*;

    #[tokio::test]
    async fn test_successful_post_delivery() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(header("content-type", "application/json"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(serde_json::json!({ "received": true })),
            )
            .expect(1)
            .mount(&mock_server)
            .await;

        let config = WebhookConfig::builder()
            .url(mock_server.uri())
            .method(WebhookMethod::Post)
            .enabled(true)
            .build();

        let channel = WebhookChannel::new(config);
        let notification = create_test_notification(Severity::High);

        let result = channel.send(&notification).await.unwrap();

        assert!(result.success);
        assert!(result.duration_ms > 0);
    }

    #[tokio::test]
    async fn test_bearer_auth() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(header("authorization", "Bearer secret-token-123"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({})))
            .expect(1)
            .mount(&mock_server)
            .await;

        let config = WebhookConfig::builder()
            .url(mock_server.uri())
            .bearer_auth("secret-token-123")
            .enabled(true)
            .build();

        let channel = WebhookChannel::new(config);
        let notification = create_test_notification(Severity::Medium);

        let result = channel.send(&notification).await.unwrap();

        assert!(result.success);
    }

    #[tokio::test]
    async fn test_api_key_auth() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(header("X-Api-Key", "my-api-key"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({})))
            .expect(1)
            .mount(&mock_server)
            .await;

        let config = WebhookConfig::builder()
            .url(mock_server.uri())
            .api_key_auth("X-Api-Key", "my-api-key")
            .enabled(true)
            .build();

        let channel = WebhookChannel::new(config);
        let notification = create_test_notification(Severity::Low);

        let result = channel.send(&notification).await.unwrap();

        assert!(result.success);
    }

    #[tokio::test]
    async fn test_custom_headers() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(header("X-Custom-Header", "custom-value"))
            .and(header("X-Another-Header", "another-value"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({})))
            .expect(1)
            .mount(&mock_server)
            .await;

        let config = WebhookConfig::builder()
            .url(mock_server.uri())
            .header("X-Custom-Header", "custom-value")
            .header("X-Another-Header", "another-value")
            .enabled(true)
            .build();

        let channel = WebhookChannel::new(config);
        let notification = create_test_notification(Severity::Info);

        let result = channel.send(&notification).await.unwrap();

        assert!(result.success);
    }

    #[tokio::test]
    async fn test_retry_on_failure() {
        let mock_server = MockServer::start().await;

        // All requests fail with 503
        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(503).set_body_string("service unavailable"))
            .mount(&mock_server)
            .await;

        let config = WebhookConfig::builder()
            .url(mock_server.uri())
            .max_retries(2)
            .retry_delay_ms(10) // Short delay for tests
            .enabled(true)
            .build();

        let channel = WebhookChannel::new(config);
        let notification = create_test_notification(Severity::High);

        let result = channel.send(&notification).await.unwrap();

        // After retries exhausted, we should get the failure result
        assert!(!result.success);
        assert!(result.error.is_some());
    }

    #[tokio::test]
    async fn test_rate_limiting() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .respond_with(
                ResponseTemplate::new(429)
                    .insert_header("Retry-After", "90")
                    .set_body_string("too many requests"),
            )
            .mount(&mock_server)
            .await;

        let config = WebhookConfig::builder()
            .url(mock_server.uri())
            .max_retries(0) // No retries
            .enabled(true)
            .build();

        let channel = WebhookChannel::new(config);
        let notification = create_test_notification(Severity::Critical);

        // Webhook channel converts rate limit errors to failures after retry exhaustion
        let result = channel.send(&notification).await.unwrap();

        assert!(!result.success);
        assert!(result.error.is_some());
        // Error message is "rate limit exceeded, retry after X seconds"
        assert!(result.error.unwrap().contains("rate limit"));
    }

    #[tokio::test]
    async fn test_custom_payload_template() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({})))
            .expect(1)
            .mount(&mock_server)
            .await;

        let custom_template = serde_json::json!({
            "custom_field": "custom_value",
            "alert": {
                "source": "sctv",
                "priority": "high"
            }
        });

        let config = WebhookConfig::builder()
            .url(mock_server.uri())
            .payload_template(custom_template.clone())
            .enabled(true)
            .build();

        let channel = WebhookChannel::new(config);
        let notification = create_test_notification(Severity::High);

        let result = channel.send(&notification).await.unwrap();
        assert!(result.success);

        let requests = mock_server.received_requests().await.unwrap();
        assert_eq!(requests.len(), 1);

        let body: serde_json::Value = serde_json::from_slice(&requests[0].body).unwrap();
        assert_eq!(body, custom_template);
    }

    #[tokio::test]
    async fn test_default_payload_structure() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({})))
            .expect(1)
            .mount(&mock_server)
            .await;

        let config = WebhookConfig::builder()
            .url(mock_server.uri())
            .enabled(true)
            .build();

        let channel = WebhookChannel::new(config);
        let notification = create_test_notification(Severity::High);

        let result = channel.send(&notification).await.unwrap();
        assert!(result.success);

        let requests = mock_server.received_requests().await.unwrap();
        assert_eq!(requests.len(), 1);

        let body: serde_json::Value = serde_json::from_slice(&requests[0].body).unwrap();
        assert_eq!(body["event_type"], "supply_chain_alert");
        assert_eq!(body["severity"], "high");
        assert!(body["id"].is_string());
        assert!(body["title"].is_string());
        assert!(body["message"].is_string());
        assert!(body["timestamp"].is_string());
        assert_eq!(body["context"]["project_name"], "my-application");
        assert_eq!(body["context"]["package_name"], "lodash-utils");
        assert_eq!(body["context"]["package_version"], "1.0.0");
    }

    #[tokio::test]
    async fn test_put_method() {
        let mock_server = MockServer::start().await;

        Mock::given(method("PUT"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({})))
            .expect(1)
            .mount(&mock_server)
            .await;

        let config = WebhookConfig::builder()
            .url(mock_server.uri())
            .method(WebhookMethod::Put)
            .enabled(true)
            .build();

        let channel = WebhookChannel::new(config);
        let notification = create_test_notification(Severity::Medium);

        let result = channel.send(&notification).await.unwrap();

        assert!(result.success);
    }

    #[tokio::test]
    async fn test_disabled_channel() {
        let config = WebhookConfig::builder()
            .url("https://api.example.com/webhook")
            .enabled(false)
            .build();

        let channel = WebhookChannel::new(config);
        let notification = create_test_notification(Severity::Info);

        let result = channel.send(&notification).await;

        assert!(matches!(result, Err(NotificationError::ChannelDisabled)));
    }

    #[tokio::test]
    async fn test_hmac_signature() {
        use sctv_notifications::channels::HmacAlgorithm;

        let mock_server = MockServer::start().await;

        // Just verify the request is made - we'll check the header after
        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({})))
            .expect(1)
            .mount(&mock_server)
            .await;

        let config = WebhookConfig::builder()
            .url(mock_server.uri())
            .hmac_auth("X-Signature", "my-secret-key", HmacAlgorithm::Sha256)
            .enabled(true)
            .build();

        let channel = WebhookChannel::new(config);
        let notification = create_test_notification(Severity::High);

        let result = channel.send(&notification).await.unwrap();

        assert!(result.success);

        // Verify the signature header was sent
        let requests = mock_server.received_requests().await.unwrap();
        let sig_header = requests[0]
            .headers
            .get("X-Signature")
            .expect("X-Signature header should be present");
        assert!(sig_header.to_str().unwrap().starts_with("sha256="));
    }

    #[tokio::test]
    async fn test_hmac_sha1_signature() {
        use sctv_notifications::channels::HmacAlgorithm;

        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({})))
            .expect(1)
            .mount(&mock_server)
            .await;

        let config = WebhookConfig::builder()
            .url(mock_server.uri())
            .hmac_auth("X-Hub-Signature", "webhook-secret", HmacAlgorithm::Sha1)
            .enabled(true)
            .build();

        let channel = WebhookChannel::new(config);
        let notification = create_test_notification(Severity::Medium);

        let result = channel.send(&notification).await.unwrap();

        assert!(result.success);

        // Verify the signature header was sent with sha1
        let requests = mock_server.received_requests().await.unwrap();
        let sig_header = requests[0]
            .headers
            .get("X-Hub-Signature")
            .expect("X-Hub-Signature header should be present");
        assert!(sig_header.to_str().unwrap().starts_with("sha1="));
    }
}

// =============================================================================
// Multi-Channel Service Tests
// =============================================================================

mod service {
    use super::*;
    use sctv_notifications::service::NotificationService;

    #[tokio::test]
    async fn test_multi_channel_delivery() {
        let slack_server = MockServer::start().await;
        let webhook_server = MockServer::start().await;

        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(200).set_body_string("ok"))
            .expect(1)
            .mount(&slack_server)
            .await;

        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({})))
            .expect(1)
            .mount(&webhook_server)
            .await;

        let slack_config = SlackConfig::builder()
            .webhook_url(slack_server.uri())
            .enabled(true)
            .build();

        let webhook_config = WebhookConfig::builder()
            .url(webhook_server.uri())
            .enabled(true)
            .build();

        let service = NotificationService::builder()
            .slack(slack_config, Severity::Info)
            .webhook(webhook_config, Severity::Info)
            .build();

        let notification = create_test_notification(Severity::High);
        let results = service.send(&notification).await;

        // Both channels should be in results
        assert_eq!(results.results.len(), 2);
        assert_eq!(results.successful, 2);
        assert_eq!(results.failed, 0);

        // Check individual results
        assert!(results.results.get("slack").unwrap().success);
        assert!(results.results.get("webhook").unwrap().success);
    }

    #[tokio::test]
    async fn test_partial_failure() {
        let slack_server = MockServer::start().await;
        let webhook_server = MockServer::start().await;

        // Slack succeeds
        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(200).set_body_string("ok"))
            .expect(1)
            .mount(&slack_server)
            .await;

        // Webhook fails
        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(500).set_body_string("error"))
            .mount(&webhook_server)
            .await;

        let slack_config = SlackConfig::builder()
            .webhook_url(slack_server.uri())
            .enabled(true)
            .build();

        let webhook_config = WebhookConfig::builder()
            .url(webhook_server.uri())
            .max_retries(0)
            .enabled(true)
            .build();

        let service = NotificationService::builder()
            .slack(slack_config, Severity::Info)
            .webhook(webhook_config, Severity::Info)
            .build();

        let notification = create_test_notification(Severity::Critical);
        let results = service.send(&notification).await;

        assert_eq!(results.results.len(), 2);
        assert_eq!(results.successful, 1);
        assert_eq!(results.failed, 1);

        assert!(results.results.get("slack").unwrap().success);
        assert!(!results.results.get("webhook").unwrap().success);
    }

    #[tokio::test]
    async fn test_severity_filtering() {
        let slack_server = MockServer::start().await;
        let webhook_server = MockServer::start().await;

        // Slack expects to receive request (min_severity = Info)
        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(200).set_body_string("ok"))
            .expect(1)
            .mount(&slack_server)
            .await;

        // Webhook should NOT receive request (min_severity = Critical)
        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({})))
            .expect(0) // Should not be called
            .mount(&webhook_server)
            .await;

        let slack_config = SlackConfig::builder()
            .webhook_url(slack_server.uri())
            .enabled(true)
            .build();

        let webhook_config = WebhookConfig::builder()
            .url(webhook_server.uri())
            .enabled(true)
            .build();

        let service = NotificationService::builder()
            .slack(slack_config, Severity::Info) // Will receive Medium
            .webhook(webhook_config, Severity::Critical) // Will NOT receive Medium
            .build();

        let notification = create_test_notification(Severity::Medium);
        let results = service.send(&notification).await;

        // Slack succeeds, webhook is skipped
        assert_eq!(results.successful, 1);
        assert_eq!(results.skipped, 1);
    }
}
