//! REST API request and response models.

use chrono::{DateTime, Utc};
use sctv_core::{AlertStatus, PackageEcosystem, ProjectStatus, Severity};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Pagination parameters.
#[derive(Debug, Clone, Deserialize)]
pub struct PaginationParams {
    #[serde(default = "default_page")]
    pub page: u32,
    #[serde(default = "default_per_page")]
    pub per_page: u32,
}

fn default_page() -> u32 {
    1
}

fn default_per_page() -> u32 {
    20
}

impl PaginationParams {
    pub fn offset(&self) -> u32 {
        (self.page.saturating_sub(1)) * self.per_page
    }
}

/// Paginated response wrapper.
#[derive(Debug, Serialize)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub pagination: PaginationInfo,
}

/// Pagination info.
#[derive(Debug, Serialize)]
pub struct PaginationInfo {
    pub page: u32,
    pub per_page: u32,
    pub total_items: u64,
    pub total_pages: u32,
}

// ==================== Projects ====================

/// Request to create a new project.
#[derive(Debug, Deserialize)]
pub struct CreateProjectRequest {
    pub name: String,
    pub description: Option<String>,
    pub repository_url: Option<String>,
    pub default_branch: Option<String>,
    pub ecosystems: Option<Vec<PackageEcosystem>>,
}

/// Request to update a project.
#[derive(Debug, Deserialize)]
pub struct UpdateProjectRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub repository_url: Option<String>,
    pub default_branch: Option<String>,
    pub is_active: Option<bool>,
}

/// Project response.
#[derive(Debug, Serialize)]
pub struct ProjectResponse {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub repository_url: Option<String>,
    pub default_branch: String,
    pub status: ProjectStatus,
    pub is_active: bool,
    pub dependency_count: u32,
    pub alert_count: u32,
    pub last_scan_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Project list filters.
#[derive(Debug, Deserialize)]
pub struct ProjectFilters {
    pub status: Option<ProjectStatus>,
    pub is_active: Option<bool>,
    pub search: Option<String>,
}

// ==================== Alerts ====================

/// Alert response.
#[derive(Debug, Serialize)]
pub struct AlertResponse {
    pub id: Uuid,
    pub project_id: Uuid,
    pub project_name: Option<String>,
    pub alert_type: String,
    pub severity: Severity,
    pub title: String,
    pub description: String,
    pub status: AlertStatus,
    pub dependency_name: Option<String>,
    pub dependency_version: Option<String>,
    pub created_at: DateTime<Utc>,
    pub acknowledged_at: Option<DateTime<Utc>>,
    pub resolved_at: Option<DateTime<Utc>>,
}

/// Alert list filters.
#[derive(Debug, Deserialize)]
pub struct AlertFilters {
    pub project_id: Option<Uuid>,
    pub severity: Option<Severity>,
    pub status: Option<AlertStatus>,
    pub alert_type: Option<String>,
}

/// Request to acknowledge an alert.
#[derive(Debug, Deserialize)]
pub struct AcknowledgeAlertRequest {
    pub notes: Option<String>,
}

/// Request to resolve an alert.
#[derive(Debug, Deserialize)]
pub struct ResolveAlertRequest {
    pub action_taken: String,
    pub new_version: Option<String>,
    pub notes: Option<String>,
}

/// Request to suppress an alert.
#[derive(Debug, Deserialize)]
pub struct SuppressAlertRequest {
    pub until: Option<DateTime<Utc>>,
    pub reason: Option<String>,
}

// ==================== Policies ====================

/// Request to create a policy.
#[derive(Debug, Deserialize)]
pub struct CreatePolicyRequest {
    pub name: String,
    pub description: Option<String>,
    pub rules: Vec<PolicyRuleRequest>,
    pub severity: Severity,
    pub is_enabled: Option<bool>,
}

/// Request to update a policy.
#[derive(Debug, Deserialize)]
pub struct UpdatePolicyRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub rules: Option<Vec<PolicyRuleRequest>>,
    pub severity: Option<Severity>,
    pub is_enabled: Option<bool>,
}

/// Policy rule in request.
#[derive(Debug, Deserialize)]
pub struct PolicyRuleRequest {
    pub rule_type: String,
    pub config: serde_json::Value,
}

/// Policy response.
#[derive(Debug, Serialize)]
pub struct PolicyResponse {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub rules: Vec<PolicyRuleResponse>,
    pub severity: Severity,
    pub is_enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Policy rule in response.
#[derive(Debug, Serialize)]
pub struct PolicyRuleResponse {
    pub rule_type: String,
    pub config: serde_json::Value,
}

// ==================== Dependencies ====================

/// Dependency response.
#[derive(Debug, Serialize)]
pub struct DependencyResponse {
    pub id: Uuid,
    pub project_id: Uuid,
    pub package_name: String,
    pub ecosystem: PackageEcosystem,
    pub version_constraint: String,
    pub resolved_version: String,
    pub is_direct: bool,
    pub is_dev_dependency: bool,
    pub depth: u32,
    pub hash_sha256: Option<String>,
    pub hash_sha512: Option<String>,
    pub signature_status: String,
    pub provenance_status: String,
    pub first_seen_at: DateTime<Utc>,
    pub last_verified_at: DateTime<Utc>,
}

/// Request to verify a dependency.
#[derive(Debug, Deserialize)]
pub struct VerifyDependencyRequest {
    pub force_download: Option<bool>,
}

/// Verification result response.
#[derive(Debug, Serialize)]
pub struct VerificationResponse {
    pub dependency_id: Uuid,
    pub is_valid: bool,
    pub checks: Vec<VerificationCheck>,
    pub verified_at: DateTime<Utc>,
}

/// Individual verification check.
#[derive(Debug, Serialize)]
pub struct VerificationCheck {
    pub check_type: String,
    pub passed: bool,
    pub expected: Option<String>,
    pub actual: Option<String>,
    pub message: Option<String>,
}

// ==================== Scans ====================

/// Scan response.
#[derive(Debug, Serialize)]
pub struct ScanResponse {
    pub id: Uuid,
    pub project_id: Uuid,
    pub status: String,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub dependencies_found: u32,
    pub alerts_created: u32,
    pub error_message: Option<String>,
}

/// Request to trigger a scan.
#[derive(Debug, Deserialize)]
pub struct TriggerScanRequest {
    pub full_scan: Option<bool>,
    pub ecosystems: Option<Vec<PackageEcosystem>>,
}

/// Scan trigger response.
#[derive(Debug, Serialize)]
pub struct TriggerScanResponse {
    pub scan_id: Uuid,
    pub status: String,
    pub message: String,
}

// ==================== Webhooks ====================

/// GitHub webhook payload (simplified).
#[derive(Debug, Deserialize)]
pub struct GitHubWebhookPayload {
    pub action: Option<String>,
    pub repository: Option<GitHubRepository>,
    #[serde(rename = "ref")]
    pub git_ref: Option<String>,
}

/// GitHub repository info.
#[derive(Debug, Deserialize)]
pub struct GitHubRepository {
    pub id: u64,
    pub name: String,
    pub full_name: String,
    pub clone_url: Option<String>,
}

/// GitLab webhook payload (simplified).
#[derive(Debug, Deserialize)]
pub struct GitLabWebhookPayload {
    pub object_kind: Option<String>,
    pub project: Option<GitLabProject>,
    #[serde(rename = "ref")]
    pub git_ref: Option<String>,
}

/// GitLab project info.
#[derive(Debug, Deserialize)]
pub struct GitLabProject {
    pub id: u64,
    pub name: String,
    pub path_with_namespace: String,
    pub git_http_url: Option<String>,
}

/// Webhook response.
#[derive(Debug, Serialize)]
pub struct WebhookResponse {
    pub received: bool,
    pub message: String,
    pub scan_triggered: bool,
    pub scan_id: Option<Uuid>,
}
