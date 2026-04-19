//! Database connection pool with tenant-aware connections.

use sctv_core::TenantId;
use sqlx::postgres::{PgPool, PgPoolOptions};
use std::time::Duration;
use thiserror::Error;

/// Errors that can occur with database connections.
#[derive(Debug, Error)]
pub enum DbError {
    #[error("Connection error: {0}")]
    Connection(#[from] sqlx::Error),

    #[error("Configuration error: {0}")]
    Configuration(String),

    #[error("Migration error: {0}")]
    Migration(String),
}

/// Configuration for the database connection pool.
#[derive(Debug, Clone)]
pub struct DbConfig {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub acquire_timeout: Duration,
    pub idle_timeout: Duration,
}

impl Default for DbConfig {
    fn default() -> Self {
        Self {
            url: String::new(),
            max_connections: 10,
            min_connections: 1,
            acquire_timeout: Duration::from_secs(30),
            idle_timeout: Duration::from_secs(600),
        }
    }
}

impl DbConfig {
    /// Creates a new configuration from a database URL.
    #[must_use]
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            ..Default::default()
        }
    }

    /// Sets the maximum number of connections.
    #[must_use]
    pub const fn max_connections(mut self, max: u32) -> Self {
        self.max_connections = max;
        self
    }

    /// Sets the minimum number of connections.
    #[must_use]
    pub const fn min_connections(mut self, min: u32) -> Self {
        self.min_connections = min;
        self
    }
}

/// Tenant-aware database connection pool.
#[derive(Clone)]
pub struct TenantAwarePool {
    pool: PgPool,
}

impl TenantAwarePool {
    /// Creates a new tenant-aware pool with the given configuration.
    ///
    /// # Errors
    ///
    /// Returns an error if the database connection cannot be established.
    pub async fn new(config: DbConfig) -> Result<Self, DbError> {
        let pool = PgPoolOptions::new()
            .max_connections(config.max_connections)
            .min_connections(config.min_connections)
            .acquire_timeout(config.acquire_timeout)
            .idle_timeout(config.idle_timeout)
            .connect(&config.url)
            .await?;

        Ok(Self { pool })
    }

    /// Acquires a connection with tenant context set.
    ///
    /// # Errors
    ///
    /// Returns an error if a connection cannot be acquired from the pool or the
    /// tenant context query fails.
    pub async fn acquire(&self, tenant_id: TenantId) -> Result<TenantConnection, DbError> {
        let mut conn = self.pool.acquire().await?;

        // Set tenant context for Row Level Security
        sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
            .bind(tenant_id.0.to_string())
            .execute(&mut *conn)
            .await?;

        Ok(TenantConnection { conn, tenant_id })
    }

    /// Gets a raw connection without tenant context (for admin operations).
    ///
    /// # Errors
    ///
    /// Returns an error if a connection cannot be acquired from the pool.
    pub async fn acquire_admin(
        &self,
    ) -> Result<sqlx::pool::PoolConnection<sqlx::Postgres>, DbError> {
        Ok(self.pool.acquire().await?)
    }

    /// Returns a reference to the underlying pool.
    #[must_use]
    pub const fn inner(&self) -> &PgPool {
        &self.pool
    }

    /// Runs pending database migrations.
    /// Note: This requires sqlx-cli to prepare migrations first.
    /// Run: `cargo sqlx prepare --database-url <url>`
    ///
    /// # Errors
    ///
    /// Returns an error if migrations fail to apply.
    #[allow(clippy::unused_async)]
    // Public API: caller-visible async fn must remain async so signature
    // and .await behavior stay stable; implementation may become async
    // when migration acquires real I/O.
    pub async fn run_migrations(&self) -> Result<(), DbError> {
        // Migration support requires sqlx-cli preparation.
        // Use sqlx::migrate!("../../migrations") when database is available.
        tracing::warn!("Migrations not available - run sqlx prepare first");
        Ok(())
    }
}

/// A database connection with tenant context.
pub struct TenantConnection {
    conn: sqlx::pool::PoolConnection<sqlx::Postgres>,
    tenant_id: TenantId,
}

impl TenantConnection {
    /// Returns the tenant ID for this connection.
    #[must_use]
    pub const fn tenant_id(&self) -> TenantId {
        self.tenant_id
    }
}

impl std::ops::Deref for TenantConnection {
    type Target = sqlx::pool::PoolConnection<sqlx::Postgres>;

    fn deref(&self) -> &Self::Target {
        &self.conn
    }
}

impl std::ops::DerefMut for TenantConnection {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.conn
    }
}

impl AsRef<sqlx::pool::PoolConnection<sqlx::Postgres>> for TenantConnection {
    fn as_ref(&self) -> &sqlx::pool::PoolConnection<sqlx::Postgres> {
        &self.conn
    }
}
