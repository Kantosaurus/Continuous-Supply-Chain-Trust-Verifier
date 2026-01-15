//! Application state shared across handlers.

use sctv_core::traits::{
    AlertRepository, DependencyRepository, PolicyRepository, ProjectRepository,
};
use std::sync::Arc;

/// Repository container holding all repository instances.
pub struct Repositories {
    pub projects: Arc<dyn ProjectRepository>,
    pub alerts: Arc<dyn AlertRepository>,
    pub dependencies: Arc<dyn DependencyRepository>,
    pub policies: Arc<dyn PolicyRepository>,
}

impl Repositories {
    /// Creates a new repository container with PostgreSQL implementations.
    pub fn new_pg(pool: sqlx::PgPool) -> Self {
        use sctv_db::{
            PgAlertRepository, PgDependencyRepository, PgPolicyRepository, PgProjectRepository,
        };
        Self {
            projects: Arc::new(PgProjectRepository::new(pool.clone())),
            alerts: Arc::new(PgAlertRepository::new(pool.clone())),
            dependencies: Arc::new(PgDependencyRepository::new(pool.clone())),
            policies: Arc::new(PgPolicyRepository::new(pool)),
        }
    }
}

/// Application state shared across all API handlers.
pub struct AppState {
    /// JWT signing secret.
    pub jwt_secret: String,
    /// JWT issuer.
    pub jwt_issuer: String,
    /// JWT audience.
    pub jwt_audience: String,
    /// Database pool (optional - for when DB is connected).
    pub db_pool: Option<sqlx::PgPool>,
    /// Repository instances (optional - available when DB is connected).
    pub repositories: Option<Repositories>,
}

impl AppState {
    /// Creates a new application state without database.
    pub fn new(jwt_secret: String) -> Self {
        Self {
            jwt_secret,
            jwt_issuer: "sctv-api".to_string(),
            jwt_audience: "sctv".to_string(),
            db_pool: None,
            repositories: None,
        }
    }

    /// Creates application state with database connection.
    pub fn with_database(jwt_secret: String, pool: sqlx::PgPool) -> Self {
        let repositories = Repositories::new_pg(pool.clone());
        Self {
            jwt_secret,
            jwt_issuer: "sctv-api".to_string(),
            jwt_audience: "sctv".to_string(),
            db_pool: Some(pool),
            repositories: Some(repositories),
        }
    }

    /// Returns a reference to the database pool, if available.
    pub fn pool(&self) -> Option<&sqlx::PgPool> {
        self.db_pool.as_ref()
    }

    /// Returns a reference to the repositories, if available.
    pub fn repos(&self) -> Option<&Repositories> {
        self.repositories.as_ref()
    }

    /// Returns the project repository or an error if not available.
    pub fn project_repo(&self) -> Result<&dyn ProjectRepository, crate::ApiError> {
        self.repositories
            .as_ref()
            .map(|r| r.projects.as_ref())
            .ok_or_else(|| crate::ApiError::ServiceUnavailable("Database not configured".into()))
    }

    /// Returns the alert repository or an error if not available.
    pub fn alert_repo(&self) -> Result<&dyn AlertRepository, crate::ApiError> {
        self.repositories
            .as_ref()
            .map(|r| r.alerts.as_ref())
            .ok_or_else(|| crate::ApiError::ServiceUnavailable("Database not configured".into()))
    }

    /// Returns the dependency repository or an error if not available.
    pub fn dependency_repo(&self) -> Result<&dyn DependencyRepository, crate::ApiError> {
        self.repositories
            .as_ref()
            .map(|r| r.dependencies.as_ref())
            .ok_or_else(|| crate::ApiError::ServiceUnavailable("Database not configured".into()))
    }

    /// Returns the policy repository or an error if not available.
    pub fn policy_repo(&self) -> Result<&dyn PolicyRepository, crate::ApiError> {
        self.repositories
            .as_ref()
            .map(|r| r.policies.as_ref())
            .ok_or_else(|| crate::ApiError::ServiceUnavailable("Database not configured".into()))
    }
}
