//! # SCTV API
//!
//! REST and GraphQL API server for Supply Chain Trust Verifier.
//!
//! This crate provides the HTTP API layer for the SCTV platform,
//! including REST endpoints, GraphQL schema, and authentication.

pub mod auth;
pub mod error;
pub mod graphql;
pub mod middleware;
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
    pub enable_cors: bool,
    /// Enable GraphQL playground.
    pub enable_graphql_playground: bool,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            bind_addr: SocketAddr::from(([127, 0, 0, 1], 3000)),
            jwt_secret: "development-secret-change-in-production".to_string(),
            enable_cors: true,
            enable_graphql_playground: true,
        }
    }
}

/// Creates the main API router.
pub fn create_router(state: Arc<AppState>) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let api_routes = Router::new()
        // Health check
        .route("/health", get(health_check))
        // REST API v1
        .nest("/api/v1", rest::routes(state.clone()))
        // GraphQL endpoint
        .nest("/graphql", graphql::routes(state.clone()));

    api_routes
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .with_state(state)
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
pub async fn run_server(config: ServerConfig, state: Arc<AppState>) -> std::io::Result<()> {
    let router = create_router(state);

    tracing::info!("Starting SCTV API server on {}", config.bind_addr);

    let listener = tokio::net::TcpListener::bind(config.bind_addr).await?;
    axum::serve(listener, router).await
}
