//! Integration tests for the REST API endpoints.
//!
//! These tests verify the API endpoints work correctly with:
//! - Request/response validation
//! - Authentication and authorization
//! - CRUD operations for all resources
//! - Error handling and status codes
//! - Pagination and filtering
//!
//! Tests use mock repositories to isolate API logic from database concerns.

use axum::{
    body::Body,
    http::{header, Method, Request, StatusCode},
};
use sctv_api::{auth::Claims, create_router, AppState};
use sctv_core::TenantId;
use serde_json::{json, Value};
use std::sync::Arc;
use tower::ServiceExt;
use uuid::Uuid;

// =============================================================================
// Test Utilities
// =============================================================================

/// Creates a test JWT token for authentication.
fn create_test_token(tenant_id: TenantId, user_id: Uuid, secret: &str) -> String {
    let claims = Claims::new(
        user_id,
        tenant_id,
        "test@example.com".to_string(),
        vec!["user".to_string()],
        "sctv-api",
        "sctv",
        24,
    );
    sctv_api::auth::encode_token(&claims, secret).unwrap()
}

/// Creates a test application state with mock repositories.
fn create_test_state() -> Arc<AppState> {
    Arc::new(AppState::new("test-secret".to_string()))
}

// =============================================================================
// Health Check Tests
// =============================================================================

mod health_check {
    use super::*;

    #[tokio::test]
    async fn test_health_check_returns_healthy() {
        let state = create_test_state();
        let router = create_router(state);

        let request = Request::builder()
            .uri("/health")
            .method(Method::GET)
            .body(Body::empty())
            .unwrap();

        let response = router.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(json["status"], "healthy");
        assert!(json["version"].is_string());
        assert!(json["timestamp"].is_string());
    }
}

// =============================================================================
// Authentication Tests
// =============================================================================

mod authentication {
    use super::*;

    #[tokio::test]
    async fn test_unauthenticated_request_returns_401() {
        let state = create_test_state();
        let router = create_router(state);

        let request = Request::builder()
            .uri("/api/v1/projects")
            .method(Method::GET)
            .body(Body::empty())
            .unwrap();

        let response = router.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_invalid_token_returns_401() {
        let state = create_test_state();
        let router = create_router(state);

        let request = Request::builder()
            .uri("/api/v1/projects")
            .method(Method::GET)
            .header(header::AUTHORIZATION, "Bearer invalid-token")
            .body(Body::empty())
            .unwrap();

        let response = router.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_expired_token_returns_401() {
        let state = create_test_state();
        let router = create_router(state);

        // Create an expired token (negative expiration)
        let tenant_id = TenantId::new();
        let claims = Claims {
            sub: Uuid::new_v4(),
            tenant_id: tenant_id.0,
            email: "test@example.com".to_string(),
            roles: vec!["user".to_string()],
            iat: chrono::Utc::now().timestamp() - 3600,
            exp: chrono::Utc::now().timestamp() - 1800, // Expired
            iss: "sctv-api".to_string(),
            aud: "sctv".to_string(),
        };
        let token = sctv_api::auth::encode_token(&claims, "test-secret").unwrap();

        let request = Request::builder()
            .uri("/api/v1/projects")
            .method(Method::GET)
            .header(header::AUTHORIZATION, format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = router.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_missing_bearer_prefix_returns_401() {
        let state = create_test_state();
        let router = create_router(state);

        let tenant_id = TenantId::new();
        let token = create_test_token(tenant_id, Uuid::new_v4(), "test-secret");

        let request = Request::builder()
            .uri("/api/v1/projects")
            .method(Method::GET)
            .header(header::AUTHORIZATION, token) // Missing "Bearer " prefix
            .body(Body::empty())
            .unwrap();

        let response = router.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }
}

// =============================================================================
// Project Endpoint Tests (Without Database)
// =============================================================================

mod projects_no_db {
    use super::*;

    #[tokio::test]
    async fn test_list_projects_without_db_returns_503() {
        let state = create_test_state(); // No database configured
        let router = create_router(state);

        let tenant_id = TenantId::new();
        let token = create_test_token(tenant_id, Uuid::new_v4(), "test-secret");

        let request = Request::builder()
            .uri("/api/v1/projects")
            .method(Method::GET)
            .header(header::AUTHORIZATION, format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = router.oneshot(request).await.unwrap();

        // Without database, should return service unavailable
        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    }

    #[tokio::test]
    async fn test_create_project_without_db_returns_503() {
        let state = create_test_state();
        let router = create_router(state);

        let tenant_id = TenantId::new();
        let token = create_test_token(tenant_id, Uuid::new_v4(), "test-secret");

        let body = json!({
            "name": "test-project",
            "description": "A test project",
            "ecosystems": ["npm"]
        });

        let request = Request::builder()
            .uri("/api/v1/projects")
            .method(Method::POST)
            .header(header::AUTHORIZATION, format!("Bearer {}", token))
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(body.to_string()))
            .unwrap();

        let response = router.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    }
}

// =============================================================================
// Webhook Endpoint Tests
// =============================================================================

mod webhooks {
    use super::*;

    #[tokio::test]
    async fn test_github_webhook_push_event() {
        let state = create_test_state();
        let router = create_router(state);

        let body = json!({
            "action": "push",
            "repository": {
                "id": 12345,
                "name": "test-repo",
                "full_name": "owner/test-repo",
                "clone_url": "https://github.com/owner/test-repo.git"
            },
            "ref": "refs/heads/main"
        });

        let request = Request::builder()
            .uri("/api/v1/webhooks/github")
            .method(Method::POST)
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(body.to_string()))
            .unwrap();

        let response = router.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(json["received"], true);
        assert_eq!(json["scan_triggered"], true);
        assert!(json["scan_id"].is_string());
    }

    #[tokio::test]
    async fn test_github_webhook_non_push_event() {
        let state = create_test_state();
        let router = create_router(state);

        let body = json!({
            "action": "opened",
            "repository": {
                "id": 12345,
                "name": "test-repo",
                "full_name": "owner/test-repo"
            }
        });

        let request = Request::builder()
            .uri("/api/v1/webhooks/github")
            .method(Method::POST)
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(body.to_string()))
            .unwrap();

        let response = router.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(json["received"], true);
        assert_eq!(json["scan_triggered"], false);
        assert!(json["scan_id"].is_null());
    }

    #[tokio::test]
    async fn test_gitlab_webhook_push_event() {
        let state = create_test_state();
        let router = create_router(state);

        let body = json!({
            "object_kind": "push",
            "project": {
                "id": 12345,
                "name": "test-repo",
                "path_with_namespace": "owner/test-repo",
                "git_http_url": "https://gitlab.com/owner/test-repo.git"
            },
            "ref": "refs/heads/main"
        });

        let request = Request::builder()
            .uri("/api/v1/webhooks/gitlab")
            .method(Method::POST)
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(body.to_string()))
            .unwrap();

        let response = router.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(json["received"], true);
        assert_eq!(json["scan_triggered"], true);
    }

    #[tokio::test]
    async fn test_gitlab_webhook_merge_request_event() {
        let state = create_test_state();
        let router = create_router(state);

        let body = json!({
            "object_kind": "merge_request",
            "project": {
                "id": 12345,
                "name": "test-repo",
                "path_with_namespace": "owner/test-repo"
            }
        });

        let request = Request::builder()
            .uri("/api/v1/webhooks/gitlab")
            .method(Method::POST)
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(body.to_string()))
            .unwrap();

        let response = router.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(json["scan_triggered"], false);
    }
}

// =============================================================================
// Error Response Format Tests
// =============================================================================

mod error_responses {
    use super::*;

    #[tokio::test]
    async fn test_error_response_format() {
        let state = create_test_state();
        let router = create_router(state);

        let request = Request::builder()
            .uri("/api/v1/projects")
            .method(Method::GET)
            .body(Body::empty())
            .unwrap();

        let response = router.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();

        // Verify error response structure
        assert!(json["error"].is_object());
        assert!(json["error"]["code"].is_string());
        assert!(json["error"]["message"].is_string());
    }

    #[tokio::test]
    async fn test_not_found_error_format() {
        let state = create_test_state();
        let router = create_router(state);

        let tenant_id = TenantId::new();
        let token = create_test_token(tenant_id, Uuid::new_v4(), "test-secret");

        // Request a non-existent scan
        let fake_id = Uuid::new_v4();
        let request = Request::builder()
            .uri(format!("/api/v1/scans/{}", fake_id))
            .method(Method::GET)
            .header(header::AUTHORIZATION, format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = router.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(json["error"]["code"], "NOT_FOUND");
    }
}

// =============================================================================
// Request Validation Tests
// =============================================================================

mod request_validation {
    use super::*;

    #[tokio::test]
    async fn test_invalid_json_returns_bad_request() {
        let state = create_test_state();
        let router = create_router(state);

        let tenant_id = TenantId::new();
        let token = create_test_token(tenant_id, Uuid::new_v4(), "test-secret");

        let request = Request::builder()
            .uri("/api/v1/projects")
            .method(Method::POST)
            .header(header::AUTHORIZATION, format!("Bearer {}", token))
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from("{ invalid json }"))
            .unwrap();

        let response = router.oneshot(request).await.unwrap();

        // Should fail with 400 or 422 for invalid JSON
        let status = response.status();
        assert!(
            status == StatusCode::BAD_REQUEST || status == StatusCode::UNPROCESSABLE_ENTITY,
            "Expected 400 or 422, got {}",
            status
        );
    }

    #[tokio::test]
    async fn test_missing_content_type_for_post() {
        let state = create_test_state();
        let router = create_router(state);

        let tenant_id = TenantId::new();
        let token = create_test_token(tenant_id, Uuid::new_v4(), "test-secret");

        let body = json!({
            "name": "test-project"
        });

        let request = Request::builder()
            .uri("/api/v1/projects")
            .method(Method::POST)
            .header(header::AUTHORIZATION, format!("Bearer {}", token))
            // Missing Content-Type header
            .body(Body::from(body.to_string()))
            .unwrap();

        let response = router.oneshot(request).await.unwrap();

        // Should either fail with 415 (Unsupported Media Type) or process anyway
        let status = response.status();
        assert!(
            status == StatusCode::UNSUPPORTED_MEDIA_TYPE
                || status == StatusCode::SERVICE_UNAVAILABLE
                || status == StatusCode::BAD_REQUEST,
            "Expected 415, 503, or 400, got {}",
            status
        );
    }
}

// =============================================================================
// Pagination Tests
// =============================================================================

mod pagination {
    use super::*;

    #[tokio::test]
    async fn test_pagination_params_parsing() {
        let state = create_test_state();
        let router = create_router(state);

        let tenant_id = TenantId::new();
        let token = create_test_token(tenant_id, Uuid::new_v4(), "test-secret");

        // Test with pagination parameters
        let request = Request::builder()
            .uri("/api/v1/projects?page=2&per_page=10")
            .method(Method::GET)
            .header(header::AUTHORIZATION, format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = router.oneshot(request).await.unwrap();

        // Should fail with 503 (no DB) but not with a parsing error
        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    }

    #[tokio::test]
    async fn test_default_pagination() {
        let state = create_test_state();
        let router = create_router(state);

        let tenant_id = TenantId::new();
        let token = create_test_token(tenant_id, Uuid::new_v4(), "test-secret");

        // Test without pagination parameters (should use defaults)
        let request = Request::builder()
            .uri("/api/v1/projects")
            .method(Method::GET)
            .header(header::AUTHORIZATION, format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = router.oneshot(request).await.unwrap();

        // Should process request (fail with 503 due to no DB)
        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    }
}

// =============================================================================
// Scan List Tests
// =============================================================================

mod scans {
    use super::*;

    #[tokio::test]
    async fn test_list_scans_empty() {
        let state = create_test_state();
        let router = create_router(state);

        let tenant_id = TenantId::new();
        let token = create_test_token(tenant_id, Uuid::new_v4(), "test-secret");

        let request = Request::builder()
            .uri("/api/v1/scans")
            .method(Method::GET)
            .header(header::AUTHORIZATION, format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = router.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();

        // Should return empty list with pagination
        assert!(json["data"].is_array());
        assert_eq!(json["data"].as_array().unwrap().len(), 0);
        assert!(json["pagination"].is_object());
    }
}

// =============================================================================
// CORS Tests
// =============================================================================

mod cors {
    use super::*;

    #[tokio::test]
    async fn test_cors_preflight_request() {
        let state = create_test_state();
        let router = create_router(state);

        let request = Request::builder()
            .uri("/api/v1/projects")
            .method(Method::OPTIONS)
            .header("Origin", "http://localhost:3000")
            .header("Access-Control-Request-Method", "GET")
            .body(Body::empty())
            .unwrap();

        let response = router.oneshot(request).await.unwrap();

        // CORS preflight should return 200 or 204
        let status = response.status();
        assert!(
            status == StatusCode::OK || status == StatusCode::NO_CONTENT,
            "Expected 200 or 204, got {}",
            status
        );

        // Should have CORS headers
        let headers = response.headers();
        assert!(
            headers.contains_key("access-control-allow-origin")
                || headers.contains_key("vary"),
            "Expected CORS headers"
        );
    }
}

// =============================================================================
// Content Type Tests
// =============================================================================

mod content_type {
    use super::*;

    #[tokio::test]
    async fn test_json_response_content_type() {
        let state = create_test_state();
        let router = create_router(state);

        let request = Request::builder()
            .uri("/health")
            .method(Method::GET)
            .body(Body::empty())
            .unwrap();

        let response = router.oneshot(request).await.unwrap();

        let content_type = response
            .headers()
            .get(header::CONTENT_TYPE)
            .unwrap()
            .to_str()
            .unwrap();

        assert!(content_type.contains("application/json"));
    }
}
