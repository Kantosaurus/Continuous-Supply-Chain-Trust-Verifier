//! Custom middleware for the API server.

use axum::{extract::Request, middleware::Next, response::Response};
use std::time::Instant;

/// Logging middleware that records request duration.
pub async fn logging_middleware(request: Request, next: Next) -> Response {
    let method = request.method().clone();
    let uri = request.uri().clone();
    let start = Instant::now();

    let response = next.run(request).await;

    let duration = start.elapsed();
    let status = response.status();

    tracing::info!(
        method = %method,
        uri = %uri,
        status = %status.as_u16(),
        duration_ms = %duration.as_millis(),
        "Request completed"
    );

    response
}

/// Rate limiting state (placeholder - use a proper rate limiter in production).
pub struct RateLimiter {
    // Field is stored for future use; the check() method is a stub that doesn't yet use it.
    #[allow(dead_code)]
    requests_per_minute: u32,
}

impl RateLimiter {
    #[must_use]
    pub const fn new(requests_per_minute: u32) -> Self {
        Self {
            requests_per_minute,
        }
    }

    /// Checks if a request should be rate limited.
    #[must_use]
    pub const fn check(&self, _client_id: &str) -> bool {
        // In a real implementation, track requests per client
        // and return true if rate limit exceeded
        false
    }
}
