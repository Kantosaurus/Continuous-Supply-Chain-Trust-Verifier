//! Retry with exponential backoff for transient HTTP failures.
//!
//! Intended for wrapping `reqwest::Client::*().send()` calls in registry
//! clients. Retries on connect/timeout errors and 5xx / 429 responses;
//! 4xx responses are returned to the caller untouched so they can decide
//! whether the error is recoverable (e.g. 404 → PackageNotFound).

use reqwest::{Response, StatusCode};
use std::future::Future;
use std::time::Duration;

/// Configuration for HTTP retry behavior.
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Number of additional attempts after the first one (total attempts = 1 + max_retries).
    pub max_retries: u32,
    /// Initial backoff delay.
    pub initial_delay: Duration,
    /// Cap on the exponentially-growing delay.
    pub max_delay: Duration,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(5),
        }
    }
}

/// Runs the given HTTP send closure with exponential backoff on transient
/// failures. Returns the first non-retryable outcome (success or 4xx).
pub async fn retry_http<F, Fut>(config: &RetryConfig, mut op: F) -> Result<Response, reqwest::Error>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<Response, reqwest::Error>>,
{
    let mut delay = config.initial_delay;
    let mut last_err: Option<reqwest::Error> = None;

    for attempt in 0..=config.max_retries {
        if attempt > 0 {
            tracing::debug!(
                attempt,
                delay_ms = delay.as_millis() as u64,
                "Retrying HTTP request"
            );
            tokio::time::sleep(delay).await;
            delay = (delay * 2).min(config.max_delay);
        }

        match op().await {
            Ok(resp) => {
                let status = resp.status();
                if status.is_server_error() || status == StatusCode::TOO_MANY_REQUESTS {
                    tracing::warn!(
                        status = %status,
                        attempt,
                        "Registry returned retryable status; will retry"
                    );
                    // Drop response; caller will see the final status or the last error.
                    continue;
                }
                return Ok(resp);
            }
            Err(e) => {
                if is_transient(&e) {
                    tracing::warn!(error = %e, attempt, "Transient HTTP error; will retry");
                    last_err = Some(e);
                    continue;
                }
                return Err(e);
            }
        }
    }

    // Exhausted retries. Return the last error if we have one, else run one
    // final send to surface whatever the server is saying.
    if let Some(e) = last_err {
        return Err(e);
    }
    op().await
}

/// Returns true if the reqwest error is worth retrying (connect/timeout).
fn is_transient(err: &reqwest::Error) -> bool {
    err.is_timeout() || err.is_connect()
}
