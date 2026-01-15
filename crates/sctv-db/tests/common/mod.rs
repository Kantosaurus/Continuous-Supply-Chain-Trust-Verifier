//! Common test utilities for database integration tests.

use sqlx::PgPool;
use std::sync::Once;
use testcontainers::{runners::AsyncRunner, ContainerAsync, ImageExt};
use testcontainers::core::IntoContainerPort;

static INIT: Once = Once::new();

/// Initialize tracing for tests (only once).
pub fn init_tracing() {
    INIT.call_once(|| {
        tracing_subscriber::fmt()
            .with_env_filter("error")
            .with_test_writer()
            .try_init()
            .ok();
    });
}

/// PostgreSQL container image for testing.
#[derive(Debug, Clone)]
pub struct PostgresImage {
    env_vars: Vec<(String, String)>,
}

impl Default for PostgresImage {
    fn default() -> Self {
        Self {
            env_vars: vec![
                ("POSTGRES_USER".to_string(), "test".to_string()),
                ("POSTGRES_PASSWORD".to_string(), "test".to_string()),
                ("POSTGRES_DB".to_string(), "sctv_test".to_string()),
            ],
        }
    }
}

impl testcontainers::Image for PostgresImage {
    fn name(&self) -> &str {
        "postgres"
    }

    fn tag(&self) -> &str {
        "16-alpine"
    }

    fn ready_conditions(&self) -> Vec<testcontainers::core::WaitFor> {
        vec![testcontainers::core::WaitFor::message_on_stderr(
            "database system is ready to accept connections",
        )]
    }

    fn env_vars(
        &self,
    ) -> impl IntoIterator<Item = (impl Into<std::borrow::Cow<'_, str>>, impl Into<std::borrow::Cow<'_, str>>)> {
        self.env_vars
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_str()))
    }

    fn expose_ports(&self) -> &[testcontainers::core::ContainerPort] {
        &[]
    }
}

/// Test database context holding the container and pool.
pub struct TestDb {
    #[allow(dead_code)]
    container: ContainerAsync<PostgresImage>,
    pub pool: PgPool,
}

impl TestDb {
    /// Creates a new test database with migrations applied.
    pub async fn new() -> Self {
        init_tracing();

        let container = PostgresImage::default()
            .with_mapped_port(5432, 5432.tcp())
            .start()
            .await
            .expect("Failed to start PostgreSQL container");

        let host_port = container
            .get_host_port_ipv4(5432)
            .await
            .expect("Failed to get host port");

        let database_url = format!(
            "postgres://test:test@127.0.0.1:{}/sctv_test",
            host_port
        );

        // Wait a bit for PostgreSQL to be fully ready
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;

        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await
            .expect("Failed to connect to test database");

        // Run migrations
        sqlx::migrate!("../../migrations")
            .run(&pool)
            .await
            .expect("Failed to run migrations");

        Self { container, pool }
    }
}

/// Creates a test tenant for use in tests.
pub fn create_test_tenant() -> sctv_core::Tenant {
    sctv_core::Tenant::new(
        format!("Test Tenant {}", uuid::Uuid::new_v4()),
        format!("test-{}", uuid::Uuid::new_v4().to_string().split('-').next().unwrap()),
    )
}

/// Creates a test user for use in tests.
pub fn create_test_user(tenant_id: sctv_core::TenantId) -> sctv_core::User {
    sctv_core::User::new(
        tenant_id,
        format!("test-{}@example.com", uuid::Uuid::new_v4()),
    )
}

/// Creates a test project for use in tests.
pub fn create_test_project(tenant_id: sctv_core::TenantId) -> sctv_core::Project {
    sctv_core::Project::new(
        tenant_id,
        format!("test-project-{}", uuid::Uuid::new_v4()),
    )
}
