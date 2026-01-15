//! REST API handlers.

mod handlers;
mod models;

use axum::{
    routing::{delete, get, post, put},
    Router,
};
use std::sync::Arc;

use crate::AppState;

/// Creates the REST API router.
pub fn routes(_state: Arc<AppState>) -> Router<Arc<AppState>> {
    Router::new()
        // Projects
        .route("/projects", get(handlers::list_projects))
        .route("/projects", post(handlers::create_project))
        .route("/projects/{id}", get(handlers::get_project))
        .route("/projects/{id}", put(handlers::update_project))
        .route("/projects/{id}", delete(handlers::delete_project))
        .route("/projects/{id}/scan", post(handlers::trigger_scan))
        .route("/projects/{id}/dependencies", get(handlers::list_project_dependencies))
        // Alerts
        .route("/alerts", get(handlers::list_alerts))
        .route("/alerts/{id}", get(handlers::get_alert))
        .route("/alerts/{id}/acknowledge", post(handlers::acknowledge_alert))
        .route("/alerts/{id}/resolve", post(handlers::resolve_alert))
        .route("/alerts/{id}/suppress", post(handlers::suppress_alert))
        // Policies
        .route("/policies", get(handlers::list_policies))
        .route("/policies", post(handlers::create_policy))
        .route("/policies/{id}", get(handlers::get_policy))
        .route("/policies/{id}", put(handlers::update_policy))
        .route("/policies/{id}", delete(handlers::delete_policy))
        // Dependencies
        .route("/dependencies/{id}", get(handlers::get_dependency))
        .route("/dependencies/{id}/verify", post(handlers::verify_dependency))
        // Scans
        .route("/scans", get(handlers::list_scans))
        .route("/scans/{id}", get(handlers::get_scan))
        // Webhooks
        .route("/webhooks/github", post(handlers::github_webhook))
        .route("/webhooks/gitlab", post(handlers::gitlab_webhook))
}

pub use handlers::*;
pub use models::*;
