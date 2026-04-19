//! SCTV Worker Binary
//!
//! This is the entry point for running the Supply Chain Trust Verifier background worker.
//! It processes jobs from the PostgreSQL-backed job queue.

use sctv_worker::{WorkerPoolConfig, WorkerServiceBuilder, WorkerServiceConfig};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

/// Environment variable names for configuration.
mod env_vars {
    pub const DATABASE_URL: &str = "DATABASE_URL";
    pub const WORKER_COUNT: &str = "SCTV_WORKER_COUNT";
    pub const POLL_INTERVAL_MS: &str = "SCTV_POLL_INTERVAL_MS";
    pub const LOG_FORMAT: &str = "SCTV_LOG_FORMAT";
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing/logging
    init_tracing();

    tracing::info!("Starting SCTV Worker");

    // Load configuration from environment
    let config = load_config();

    // Initialize database connection
    let database_url = std::env::var(env_vars::DATABASE_URL)
        .unwrap_or_else(|_| "postgres://sctv:sctv@localhost:5432/sctv".to_string());

    let worker_count = config.pool.worker_count;

    tracing::info!("Connecting to database...");
    let db_pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(worker_count as u32 + 5)
        .connect(&database_url)
        .await?;

    tracing::info!("Database connection established");

    // Build the worker service
    let mut service = WorkerServiceBuilder::new()
        .with_db_pool(db_pool)
        .with_config(config)
        .build()?;

    tracing::info!("Worker service initialized with {} workers", worker_count);

    // Register signal handlers for graceful shutdown
    let shutdown_signal = shutdown_signal();

    // Start processing jobs
    service.start()?;

    tracing::info!("Worker pool started, waiting for jobs...");

    // Wait for shutdown signal
    shutdown_signal.await;

    tracing::info!("Shutdown signal received, stopping workers...");

    // Request graceful shutdown
    if let Err(e) = service.stop().await {
        tracing::error!("Error stopping worker service: {}", e);
    }

    tracing::info!("Worker pool stopped");

    Ok(())
}

/// Initialize tracing/logging based on environment configuration.
fn init_tracing() {
    let log_format = std::env::var(env_vars::LOG_FORMAT).unwrap_or_else(|_| "pretty".to_string());

    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,sctv_worker=debug"));

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

/// Load worker configuration from environment variables.
fn load_config() -> WorkerServiceConfig {
    let worker_count: usize = std::env::var(env_vars::WORKER_COUNT)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(4);

    let poll_interval_ms: u64 = std::env::var(env_vars::POLL_INTERVAL_MS)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(1000);

    let pool_config = WorkerPoolConfig::new()
        .with_workers(worker_count)
        .with_poll_interval(poll_interval_ms);

    WorkerServiceConfig::new().with_pool(pool_config)
}

/// Create a future that completes on SIGTERM or SIGINT.
async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("Failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        () = ctrl_c => {},
        () = terminate => {},
    }
}
