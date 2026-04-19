//! # SCTV API
//!
//! REST and GraphQL API server for Supply Chain Trust Verifier.
//!
//! This crate provides the HTTP API layer for the SCTV platform,
//! including REST endpoints, GraphQL schema, and authentication.

pub mod auth;
pub mod error;
pub mod graphql;
pub mod rest;
pub mod state;

use axum::{routing::get, Json, Router};
use serde::Serialize;
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

pub use error::{ApiError, ApiResult};
pub use state::AppState;

/// Server configuration.
#[derive(Debug, Clone)]
pub struct ServerConfig {
    /// Address to bind to.
    pub bind_addr: SocketAddr,
    /// JWT secret for authentication.
    pub jwt_secret: String,
    /// Enable CORS (for development).
    ///
    /// When `true`, a permissive CORS layer is installed on the router
    /// (any origin, any method, any header). When `false`, no CORS layer
    /// is installed — responses will not contain CORS headers and browsers
    /// will reject cross-origin requests. Production deployments should
    /// set this to `false` unless the API is fronted by a gateway that
    /// adds its own CORS policy.
    pub enable_cors: bool,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            bind_addr: SocketAddr::from(([127, 0, 0, 1], 3000)),
            jwt_secret: "development-secret-change-in-production".to_string(),
            enable_cors: true,
        }
    }
}

/// Creates the main API router.
///
/// The `config` argument controls router-level behavior such as whether
/// a CORS layer is installed. When `config.enable_cors` is `false`, no
/// CORS layer is attached and the router will not emit CORS headers.
pub fn create_router(state: Arc<AppState>, config: &ServerConfig) -> Router {
    let api_routes = Router::new()
        // Health check
        .route("/health", get(health_check))
        // REST API v1
        .nest("/api/v1", rest::routes(state.clone()))
        // GraphQL endpoint
        .nest("/graphql", graphql::routes(state.clone()));

    let router = api_routes
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    if config.enable_cors {
        let cors = CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any);
        router.layer(cors)
    } else {
        router
    }
}

/// Health check response.
#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: &'static str,
    pub version: &'static str,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Health check endpoint.
async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "healthy",
        version: env!("CARGO_PKG_VERSION"),
        timestamp: chrono::Utc::now(),
    })
}

/// Starts the API server.
///
/// # Errors
///
/// Returns an [`std::io::Error`] if binding to the address or serving fails.
pub async fn run_server(config: ServerConfig, state: Arc<AppState>) -> std::io::Result<()> {
    let router = create_router(state, &config);

    tracing::info!("Starting SCTV API server on {}", config.bind_addr);

    let listener = tokio::net::TcpListener::bind(config.bind_addr).await?;
    axum::serve(listener, router).await
}

#[cfg(test)]
mod tests {
    use super::{create_router, AppState, ServerConfig};
    use axum::{
        body::Body,
        http::{Method, Request, StatusCode},
    };
    use std::sync::Arc;
    use tower::ServiceExt;

    fn test_state() -> Arc<AppState> {
        Arc::new(AppState::new("test-secret".to_string()))
    }

    fn cors_enabled_config() -> ServerConfig {
        ServerConfig {
            enable_cors: true,
            ..ServerConfig::default()
        }
    }

    fn cors_disabled_config() -> ServerConfig {
        ServerConfig {
            enable_cors: false,
            ..ServerConfig::default()
        }
    }

    /// With `enable_cors = true`, a preflight OPTIONS request is handled by
    /// the CORS layer and the response includes `access-control-allow-origin`.
    #[tokio::test]
    async fn cors_enabled_returns_cors_headers_on_preflight() {
        let router = create_router(test_state(), &cors_enabled_config());

        let request = Request::builder()
            .uri("/api/v1/projects")
            .method(Method::OPTIONS)
            .header("Origin", "http://localhost:3000")
            .header("Access-Control-Request-Method", "GET")
            .body(Body::empty())
            .unwrap();

        let response = router.oneshot(request).await.unwrap();

        // tower-http CORS replies to preflight with 200 OK.
        assert_eq!(response.status(), StatusCode::OK);
        assert!(
            response
                .headers()
                .contains_key("access-control-allow-origin"),
            "expected access-control-allow-origin header when CORS is enabled, \
             got headers: {:?}",
            response.headers()
        );
    }

    /// With `enable_cors = true`, simple requests also get the
    /// `access-control-allow-origin` header echoed back.
    #[tokio::test]
    async fn cors_enabled_sets_allow_origin_on_simple_get() {
        let router = create_router(test_state(), &cors_enabled_config());

        let request = Request::builder()
            .uri("/health")
            .method(Method::GET)
            .header("Origin", "http://localhost:3000")
            .body(Body::empty())
            .unwrap();

        let response = router.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert!(
            response
                .headers()
                .contains_key("access-control-allow-origin"),
            "expected access-control-allow-origin on simple GET when CORS enabled"
        );
    }

    /// With `enable_cors = false`, no CORS layer is installed. A preflight
    /// OPTIONS request is not intercepted (falls through to method-not-allowed
    /// or similar) and the response MUST NOT contain CORS headers.
    #[tokio::test]
    async fn cors_disabled_has_no_cors_headers_on_preflight() {
        let router = create_router(test_state(), &cors_disabled_config());

        let request = Request::builder()
            .uri("/api/v1/projects")
            .method(Method::OPTIONS)
            .header("Origin", "http://localhost:3000")
            .header("Access-Control-Request-Method", "GET")
            .body(Body::empty())
            .unwrap();

        let response = router.oneshot(request).await.unwrap();

        assert!(
            !response
                .headers()
                .contains_key("access-control-allow-origin"),
            "expected NO access-control-allow-origin header when CORS disabled, \
             got headers: {:?}",
            response.headers()
        );
        assert!(
            !response
                .headers()
                .contains_key("access-control-allow-methods"),
            "expected NO access-control-allow-methods header when CORS disabled"
        );
    }

    /// With `enable_cors = false`, a normal GET /health request succeeds but
    /// carries no CORS headers.
    #[tokio::test]
    async fn cors_disabled_has_no_cors_headers_on_get() {
        let router = create_router(test_state(), &cors_disabled_config());

        let request = Request::builder()
            .uri("/health")
            .method(Method::GET)
            .header("Origin", "http://localhost:3000")
            .body(Body::empty())
            .unwrap();

        let response = router.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert!(
            !response
                .headers()
                .contains_key("access-control-allow-origin"),
            "expected NO access-control-allow-origin on GET /health when CORS disabled"
        );
    }
}
