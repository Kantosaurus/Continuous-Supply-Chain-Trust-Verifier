//! SCTV API Server Binary
//!
//! This is the entry point for running the Supply Chain Trust Verifier API server.
//! It reads configuration from environment variables and starts the HTTP server.

use sctv_api::{run_server, AppState, ServerConfig};
use std::net::SocketAddr;
use std::sync::Arc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

/// Environment variable names for configuration.
mod env_vars {
    pub const BIND_HOST: &str = "SCTV_HOST";
    pub const BIND_PORT: &str = "SCTV_PORT";
    pub const JWT_SECRET: &str = "SCTV_JWT_SECRET";
    pub const DATABASE_URL: &str = "DATABASE_URL";
    pub const ENABLE_CORS: &str = "SCTV_ENABLE_CORS";
    pub const ENABLE_PLAYGROUND: &str = "SCTV_ENABLE_GRAPHQL_PLAYGROUND";
    pub const LOG_FORMAT: &str = "SCTV_LOG_FORMAT";
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing/logging
    init_tracing();

    tracing::info!("Starting SCTV API Server");

    // Load configuration from environment
    let config = load_config()?;

    // Initialize database connection
    let database_url = std::env::var(env_vars::DATABASE_URL)
        .unwrap_or_else(|_| "postgres://sctv:sctv@localhost:5432/sctv".to_string());

    tracing::info!("Connecting to database...");
    let db_pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await?;

    tracing::info!("Database connection established");

    // Run migrations in production
    tracing::info!("Running database migrations...");
    sqlx::migrate!("../../migrations").run(&db_pool).await?;
    tracing::info!("Migrations completed");

    // Create application state
    let state = Arc::new(AppState::with_database(config.jwt_secret.clone(), db_pool));

    // Start the server
    tracing::info!("Server listening on {}", config.bind_addr);
    run_server(config, state).await?;

    Ok(())
}

/// Initialize tracing/logging based on environment configuration.
fn init_tracing() {
    let log_format = std::env::var(env_vars::LOG_FORMAT).unwrap_or_else(|_| "pretty".to_string());

    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,sctv_api=debug,tower_http=debug"));

    match log_format.as_str() {
        "json" => {
            tracing_subscriber::registry()
                .with(env_filter)
                .with(tracing_subscriber::fmt::layer().json())
                .init();
        }
        _ => {
            tracing_subscriber::registry()
                .with(env_filter)
                .with(tracing_subscriber::fmt::layer().pretty())
                .init();
        }
    }
}

/// Load server configuration from environment variables.
fn load_config() -> anyhow::Result<ServerConfig> {
    let host = std::env::var(env_vars::BIND_HOST).unwrap_or_else(|_| "0.0.0.0".to_string());

    let port: u16 = std::env::var(env_vars::BIND_PORT)
        .unwrap_or_else(|_| "3000".to_string())
        .parse()
        .unwrap_or(3000);

    let bind_addr: SocketAddr = format!("{host}:{port}").parse()?;

    let jwt_secret = std::env::var(env_vars::JWT_SECRET)
        .unwrap_or_else(|_| {
            tracing::warn!(
                "JWT_SECRET not set, using default. THIS IS INSECURE FOR PRODUCTION!"
            );
            "development-secret-change-in-production".to_string()
        });

    let enable_cors = std::env::var(env_vars::ENABLE_CORS)
        .map(|v| v.to_lowercase() == "true" || v == "1")
        .unwrap_or(true);

    let enable_graphql_playground = std::env::var(env_vars::ENABLE_PLAYGROUND)
        .map(|v| v.to_lowercase() == "true" || v == "1")
        .unwrap_or(true);

    Ok(ServerConfig {
        bind_addr,
        jwt_secret,
        enable_cors,
        enable_graphql_playground,
    })
}
