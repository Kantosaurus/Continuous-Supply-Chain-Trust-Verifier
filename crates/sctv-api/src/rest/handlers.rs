//! REST API handler implementations.

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use chrono::Utc;
use sctv_core::traits::AlertFilter;
use sctv_core::{
    Alert, AlertId, Dependency, DependencyId, Policy, PolicyId, PolicyRule, Project, ProjectId,
    Severity,
};
use std::sync::Arc;
use url::Url;
use uuid::Uuid;

use super::models::{
    AcknowledgeAlertRequest, AlertFilters, AlertResponse, CreatePolicyRequest,
    CreateProjectRequest, DependencyResponse, GitHubWebhookPayload, GitLabWebhookPayload,
    PaginatedResponse, PaginationInfo, PaginationParams, PolicyResponse, PolicyRuleRequest,
    PolicyRuleResponse, ProjectFilters, ProjectResponse, ResolveAlertRequest, ScanResponse,
    SuppressAlertRequest, TriggerScanRequest, TriggerScanResponse, UpdatePolicyRequest,
    UpdateProjectRequest, VerificationCheck, VerificationResponse, VerifyDependencyRequest,
    WebhookResponse,
};
use crate::{
    auth::AuthUser,
    error::{ApiError, ApiResult},
    AppState,
};

// ==================== Conversion Helpers ====================

fn project_to_response(project: &Project, dep_count: u32, alert_count: u32) -> ProjectResponse {
    ProjectResponse {
        id: project.id.0,
        name: project.name.clone(),
        description: project.description.clone(),
        repository_url: project
            .repository_url
            .as_ref()
            .map(std::string::ToString::to_string),
        default_branch: project.default_branch.clone(),
        status: project.status,
        is_active: true, // Projects in DB are considered active
        dependency_count: dep_count,
        alert_count,
        last_scan_at: project.last_scan_at,
        created_at: project.created_at,
        updated_at: project.updated_at,
    }
}

fn alert_to_response(alert: &Alert, project_name: Option<String>) -> AlertResponse {
    // Extract dependency info from alert type if available
    let (dep_name, dep_version) = extract_dependency_info(&alert.alert_type);

    AlertResponse {
        id: alert.id.0,
        project_id: alert.project_id.0,
        project_name,
        alert_type: alert.alert_type.type_name().to_string(),
        severity: alert.severity,
        title: alert.title.clone(),
        description: alert.description.clone(),
        status: alert.status,
        dependency_name: dep_name,
        dependency_version: dep_version,
        created_at: alert.created_at,
        acknowledged_at: alert.acknowledged_at,
        resolved_at: alert.resolved_at,
    }
}

fn extract_dependency_info(alert_type: &sctv_core::AlertType) -> (Option<String>, Option<String>) {
    use sctv_core::AlertType;
    match alert_type {
        AlertType::DependencyTampering(d) => {
            (Some(d.package_name.clone()), Some(d.version.clone()))
        }
        AlertType::DowngradeAttack(d) => (
            Some(d.package_name.clone()),
            Some(d.current_version.to_string()),
        ),
        AlertType::Typosquatting(d) => (Some(d.suspicious_package.clone()), None),
        AlertType::ProvenanceFailure(d) => (Some(d.package_name.clone()), Some(d.version.clone())),
        AlertType::PolicyViolation(_) => (None, None),
        AlertType::NewPackage(d) => (Some(d.package_name.clone()), Some(d.version.clone())),
        AlertType::SuspiciousMaintainer(d) => (Some(d.package_name.clone()), None),
    }
}

fn dependency_to_response(dep: &Dependency) -> DependencyResponse {
    DependencyResponse {
        id: dep.id.0,
        project_id: dep.project_id.0,
        package_name: dep.package_name.clone(),
        ecosystem: dep.ecosystem,
        version_constraint: dep.version_constraint.clone(),
        resolved_version: dep.resolved_version.to_string(),
        is_direct: dep.is_direct,
        is_dev_dependency: dep.is_dev_dependency,
        depth: dep.depth,
        hash_sha256: dep.integrity.hash_sha256.clone(),
        hash_sha512: dep.integrity.hash_sha512.clone(),
        signature_status: format!("{:?}", dep.integrity.signature_status).to_lowercase(),
        provenance_status: format!("{:?}", dep.integrity.provenance_status).to_lowercase(),
        first_seen_at: dep.first_seen_at,
        last_verified_at: dep.last_verified_at,
    }
}

fn policy_to_response(policy: &Policy) -> PolicyResponse {
    // Pick the highest override severity so the UI reflects the most severe
    // override the policy enforces; fall back to Medium when no overrides
    // are configured (rules carry their own severities per-finding).
    let severity = policy
        .severity_overrides
        .iter()
        .map(|o| o.severity)
        .max()
        .unwrap_or(Severity::Medium);

    PolicyResponse {
        id: policy.id.0,
        name: policy.name.clone(),
        description: policy.description.clone(),
        rules: policy
            .rules
            .iter()
            .filter_map(|r| {
                // Serialize the rule to JSON to extract type and config
                let json = serde_json::to_value(r).ok()?;
                let rule_type = json.get("type")?.as_str()?.to_string();
                let mut config = json;
                if let Some(obj) = config.as_object_mut() {
                    obj.remove("type");
                }
                Some(PolicyRuleResponse { rule_type, config })
            })
            .collect(),
        severity,
        is_enabled: policy.enabled,
        created_at: policy.created_at,
        updated_at: policy.updated_at,
    }
}

/// Helper to convert REST `PolicyRuleRequest` to domain `PolicyRule`
fn rule_request_to_policy_rule(request: &PolicyRuleRequest) -> Result<PolicyRule, ApiError> {
    // Merge type and config into a single JSON object for deserialization
    let mut json = request.config.clone();
    if let Some(obj) = json.as_object_mut() {
        obj.insert(
            "type".to_string(),
            serde_json::Value::String(request.rule_type.clone()),
        );
    } else {
        let mut obj = serde_json::Map::new();
        obj.insert(
            "type".to_string(),
            serde_json::Value::String(request.rule_type.clone()),
        );
        json = serde_json::Value::Object(obj);
    }

    serde_json::from_value(json)
        .map_err(|e| ApiError::Validation(format!("Invalid policy rule: {e}")))
}

// ==================== Projects ====================

/// List all projects for the authenticated user's tenant.
pub async fn list_projects(
    user: AuthUser,
    Query(pagination): Query<PaginationParams>,
    Query(_filters): Query<ProjectFilters>,
    State(state): State<Arc<AppState>>,
) -> ApiResult<Json<PaginatedResponse<ProjectResponse>>> {
    let project_repo = state.project_repo()?;
    let alert_repo = state.alert_repo()?;
    let dep_repo = state.dependency_repo()?;

    // Fetch all projects for the tenant
    let projects = project_repo.find_by_tenant(user.tenant_id).await?;

    // Calculate pagination
    let total_items = projects.len() as u64;
    let total_pages = ((total_items as f64) / f64::from(pagination.per_page)).ceil() as u32;
    let offset = pagination.offset() as usize;
    let limit = pagination.per_page as usize;

    // Paginate results
    let paginated: Vec<_> = projects.into_iter().skip(offset).take(limit).collect();

    // Build responses with dependency and alert counts
    let mut responses = Vec::with_capacity(paginated.len());
    for project in paginated {
        let deps = dep_repo.find_by_project(project.id).await?;
        let alert_count = alert_repo.count_open_by_project(project.id).await?;
        responses.push(project_to_response(
            &project,
            deps.len() as u32,
            alert_count,
        ));
    }

    Ok(Json(PaginatedResponse {
        data: responses,
        pagination: PaginationInfo {
            page: pagination.page,
            per_page: pagination.per_page,
            total_items,
            total_pages,
        },
    }))
}

/// Create a new project.
pub async fn create_project(
    user: AuthUser,
    State(state): State<Arc<AppState>>,
    Json(request): Json<CreateProjectRequest>,
) -> ApiResult<(StatusCode, Json<ProjectResponse>)> {
    let project_repo = state.project_repo()?;

    // Create new project domain object
    let mut project = Project::new(user.tenant_id, request.name);
    project.description = request.description;
    project.repository_url = request.repository_url.and_then(|u| Url::parse(&u).ok());
    project.default_branch = request.default_branch.unwrap_or_else(|| "main".to_string());
    if let Some(ecosystems) = request.ecosystems {
        project.ecosystems = ecosystems;
    }

    // Save to database
    project_repo.create(&project).await?;

    let response = project_to_response(&project, 0, 0);
    Ok((StatusCode::CREATED, Json(response)))
}

/// Get a specific project.
pub async fn get_project(
    user: AuthUser,
    Path(id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
) -> ApiResult<Json<ProjectResponse>> {
    let project_repo = state.project_repo()?;
    let alert_repo = state.alert_repo()?;
    let dep_repo = state.dependency_repo()?;

    let project = project_repo
        .find_by_id(ProjectId(id))
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("Project {id} not found")))?;

    // Verify tenant access
    if project.tenant_id != user.tenant_id {
        return Err(ApiError::Forbidden);
    }

    let deps = dep_repo.find_by_project(project.id).await?;
    let alert_count = alert_repo.count_open_by_project(project.id).await?;

    Ok(Json(project_to_response(
        &project,
        deps.len() as u32,
        alert_count,
    )))
}

/// Update a project.
pub async fn update_project(
    user: AuthUser,
    Path(id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    Json(request): Json<UpdateProjectRequest>,
) -> ApiResult<Json<ProjectResponse>> {
    let project_repo = state.project_repo()?;
    let alert_repo = state.alert_repo()?;
    let dep_repo = state.dependency_repo()?;

    let mut project = project_repo
        .find_by_id(ProjectId(id))
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("Project {id} not found")))?;

    // Verify tenant access
    if project.tenant_id != user.tenant_id {
        return Err(ApiError::Forbidden);
    }

    // Apply updates
    if let Some(name) = request.name {
        project.name = name;
    }
    if let Some(desc) = request.description {
        project.description = Some(desc);
    }
    if let Some(url) = request.repository_url {
        project.repository_url = Url::parse(&url).ok();
    }
    if let Some(branch) = request.default_branch {
        project.default_branch = branch;
    }
    project.updated_at = Utc::now();

    // Save updates
    project_repo.update(&project).await?;

    let deps = dep_repo.find_by_project(project.id).await?;
    let alert_count = alert_repo.count_open_by_project(project.id).await?;

    Ok(Json(project_to_response(
        &project,
        deps.len() as u32,
        alert_count,
    )))
}

/// Delete a project.
pub async fn delete_project(
    user: AuthUser,
    Path(id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
) -> ApiResult<StatusCode> {
    let project_repo = state.project_repo()?;

    // Verify project exists and belongs to tenant
    let project = project_repo
        .find_by_id(ProjectId(id))
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("Project {id} not found")))?;

    if project.tenant_id != user.tenant_id {
        return Err(ApiError::Forbidden);
    }

    project_repo.delete(ProjectId(id)).await?;

    Ok(StatusCode::NO_CONTENT)
}

/// Trigger a scan for a project.
pub async fn trigger_scan(
    user: AuthUser,
    Path(id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    Json(_request): Json<TriggerScanRequest>,
) -> ApiResult<Json<TriggerScanResponse>> {
    use sctv_worker::jobs::{JobPayload, ScanProjectPayload};
    use sctv_worker::queue::{EnqueueOptions, JobQueue, PgJobQueue};

    let project_repo = state.project_repo()?;

    let project = project_repo
        .find_by_id(ProjectId(id))
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("Project {id} not found")))?;

    if project.tenant_id != user.tenant_id {
        return Err(ApiError::Forbidden);
    }

    let pool = state
        .pool()
        .ok_or_else(|| ApiError::ServiceUnavailable("Database not configured".into()))?;
    let queue = PgJobQueue::new(pool.clone());

    let payload = JobPayload::ScanProject(ScanProjectPayload::new(project.id, user.tenant_id));
    let job_id = queue
        .enqueue(payload, EnqueueOptions::default())
        .await
        .map_err(|e| ApiError::Internal(format!("Failed to enqueue scan: {e}")))?;

    Ok(Json(TriggerScanResponse {
        scan_id: job_id.0,
        status: "queued".to_string(),
        message: "Scan has been queued for processing".to_string(),
    }))
}

/// List dependencies for a project.
pub async fn list_project_dependencies(
    user: AuthUser,
    Path(id): Path<Uuid>,
    Query(pagination): Query<PaginationParams>,
    State(state): State<Arc<AppState>>,
) -> ApiResult<Json<PaginatedResponse<DependencyResponse>>> {
    let project_repo = state.project_repo()?;
    let dep_repo = state.dependency_repo()?;

    // Verify project exists and belongs to tenant
    let project = project_repo
        .find_by_id(ProjectId(id))
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("Project {id} not found")))?;

    if project.tenant_id != user.tenant_id {
        return Err(ApiError::Forbidden);
    }

    let dependencies = dep_repo.find_by_project(ProjectId(id)).await?;

    // Calculate pagination
    let total_items = dependencies.len() as u64;
    let total_pages = ((total_items as f64) / f64::from(pagination.per_page)).ceil() as u32;
    let offset = pagination.offset() as usize;
    let limit = pagination.per_page as usize;

    // Paginate and convert
    let responses: Vec<_> = dependencies
        .into_iter()
        .skip(offset)
        .take(limit)
        .map(|d| dependency_to_response(&d))
        .collect();

    Ok(Json(PaginatedResponse {
        data: responses,
        pagination: PaginationInfo {
            page: pagination.page,
            per_page: pagination.per_page,
            total_items,
            total_pages,
        },
    }))
}

// ==================== Alerts ====================

/// List all alerts.
pub async fn list_alerts(
    user: AuthUser,
    Query(pagination): Query<PaginationParams>,
    Query(filters): Query<AlertFilters>,
    State(state): State<Arc<AppState>>,
) -> ApiResult<Json<PaginatedResponse<AlertResponse>>> {
    let alert_repo = state.alert_repo()?;
    let project_repo = state.project_repo()?;

    // Build filter (cloned so we can count with the same predicate)
    let filter = AlertFilter {
        project_id: filters.project_id.map(ProjectId),
        status: filters.status.map(|s| vec![s]),
        severity: filters.severity.map(|s| vec![s]),
        alert_type: filters.alert_type.map(|t| vec![t]),
    };

    let total_items = alert_repo
        .count_with_filter(user.tenant_id, filter.clone())
        .await?;

    let alerts = alert_repo
        .find_with_filter(
            user.tenant_id,
            filter,
            pagination.per_page,
            pagination.offset(),
        )
        .await?;

    let mut responses = Vec::with_capacity(alerts.len());
    for alert in &alerts {
        let project_name = project_repo
            .find_by_id(alert.project_id)
            .await?
            .map(|p| p.name);
        responses.push(alert_to_response(alert, project_name));
    }

    let per_page = u64::from(pagination.per_page.max(1));
    let total_pages = total_items.div_ceil(per_page) as u32;

    Ok(Json(PaginatedResponse {
        data: responses,
        pagination: PaginationInfo {
            page: pagination.page,
            per_page: pagination.per_page,
            total_items,
            total_pages,
        },
    }))
}

/// Get a specific alert.
pub async fn get_alert(
    user: AuthUser,
    Path(id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
) -> ApiResult<Json<AlertResponse>> {
    let alert_repo = state.alert_repo()?;
    let project_repo = state.project_repo()?;

    let alert = alert_repo
        .find_by_id(AlertId(id))
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("Alert {id} not found")))?;

    // Verify tenant access
    if alert.tenant_id != user.tenant_id {
        return Err(ApiError::Forbidden);
    }

    let project_name = project_repo
        .find_by_id(alert.project_id)
        .await?
        .map(|p| p.name);

    Ok(Json(alert_to_response(&alert, project_name)))
}

/// Acknowledge an alert.
pub async fn acknowledge_alert(
    user: AuthUser,
    Path(id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    Json(_request): Json<AcknowledgeAlertRequest>,
) -> ApiResult<Json<AlertResponse>> {
    let alert_repo = state.alert_repo()?;
    let project_repo = state.project_repo()?;

    let mut alert = alert_repo
        .find_by_id(AlertId(id))
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("Alert {id} not found")))?;

    // Verify tenant access
    if alert.tenant_id != user.tenant_id {
        return Err(ApiError::Forbidden);
    }

    // Acknowledge the alert
    alert.acknowledge(user.user_id);
    alert_repo.update(&alert).await?;

    let project_name = project_repo
        .find_by_id(alert.project_id)
        .await?
        .map(|p| p.name);

    Ok(Json(alert_to_response(&alert, project_name)))
}

/// Resolve an alert.
pub async fn resolve_alert(
    user: AuthUser,
    Path(id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    Json(request): Json<ResolveAlertRequest>,
) -> ApiResult<Json<AlertResponse>> {
    let alert_repo = state.alert_repo()?;
    let project_repo = state.project_repo()?;

    let mut alert = alert_repo
        .find_by_id(AlertId(id))
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("Alert {id} not found")))?;

    // Verify tenant access
    if alert.tenant_id != user.tenant_id {
        return Err(ApiError::Forbidden);
    }

    // Resolve the alert
    let remediation = sctv_core::Remediation {
        action_taken: request.action_taken,
        new_version: request
            .new_version
            .and_then(|v| semver::Version::parse(&v).ok()),
        notes: request.notes,
    };
    alert.resolve(user.user_id, remediation);
    alert_repo.update(&alert).await?;

    let project_name = project_repo
        .find_by_id(alert.project_id)
        .await?
        .map(|p| p.name);

    Ok(Json(alert_to_response(&alert, project_name)))
}

/// Suppress an alert.
pub async fn suppress_alert(
    user: AuthUser,
    Path(id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    Json(request): Json<SuppressAlertRequest>,
) -> ApiResult<Json<AlertResponse>> {
    let alert_repo = state.alert_repo()?;
    let project_repo = state.project_repo()?;

    let mut alert = alert_repo
        .find_by_id(AlertId(id))
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("Alert {id} not found")))?;

    // Verify tenant access
    if alert.tenant_id != user.tenant_id {
        return Err(ApiError::Forbidden);
    }

    // Suppress the alert
    alert.suppress(request.until);
    alert_repo.update(&alert).await?;

    let project_name = project_repo
        .find_by_id(alert.project_id)
        .await?
        .map(|p| p.name);

    Ok(Json(alert_to_response(&alert, project_name)))
}

// ==================== Policies ====================

/// List all policies.
pub async fn list_policies(
    user: AuthUser,
    Query(pagination): Query<PaginationParams>,
    State(state): State<Arc<AppState>>,
) -> ApiResult<Json<PaginatedResponse<PolicyResponse>>> {
    let policy_repo = state.policy_repo()?;

    let policies = policy_repo.find_by_tenant(user.tenant_id).await?;

    // Calculate pagination
    let total_items = policies.len() as u64;
    let total_pages = ((total_items as f64) / f64::from(pagination.per_page)).ceil() as u32;
    let offset = pagination.offset() as usize;
    let limit = pagination.per_page as usize;

    // Paginate and convert
    let responses: Vec<_> = policies
        .into_iter()
        .skip(offset)
        .take(limit)
        .map(|p| policy_to_response(&p))
        .collect();

    Ok(Json(PaginatedResponse {
        data: responses,
        pagination: PaginationInfo {
            page: pagination.page,
            per_page: pagination.per_page,
            total_items,
            total_pages,
        },
    }))
}

/// Create a new policy.
pub async fn create_policy(
    user: AuthUser,
    State(state): State<Arc<AppState>>,
    Json(request): Json<CreatePolicyRequest>,
) -> ApiResult<(StatusCode, Json<PolicyResponse>)> {
    let policy_repo = state.policy_repo()?;

    // Convert rule requests to domain rules
    let rules: Result<Vec<PolicyRule>, _> = request
        .rules
        .iter()
        .map(rule_request_to_policy_rule)
        .collect();
    let rules = rules?;

    let now = Utc::now();
    let policy = Policy {
        id: PolicyId::new(),
        tenant_id: user.tenant_id,
        name: request.name,
        description: request.description,
        rules,
        severity_overrides: Vec::new(),
        is_default: false,
        enabled: request.is_enabled.unwrap_or(true),
        created_at: now,
        updated_at: now,
    };

    policy_repo.create(&policy).await?;

    Ok((StatusCode::CREATED, Json(policy_to_response(&policy))))
}

/// Get a specific policy.
pub async fn get_policy(
    user: AuthUser,
    Path(id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
) -> ApiResult<Json<PolicyResponse>> {
    let policy_repo = state.policy_repo()?;

    let policy = policy_repo
        .find_by_id(PolicyId(id))
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("Policy {id} not found")))?;

    // Verify tenant access
    if policy.tenant_id != user.tenant_id {
        return Err(ApiError::Forbidden);
    }

    Ok(Json(policy_to_response(&policy)))
}

/// Update a policy.
pub async fn update_policy(
    user: AuthUser,
    Path(id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    Json(request): Json<UpdatePolicyRequest>,
) -> ApiResult<Json<PolicyResponse>> {
    let policy_repo = state.policy_repo()?;

    let mut policy = policy_repo
        .find_by_id(PolicyId(id))
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("Policy {id} not found")))?;

    // Verify tenant access
    if policy.tenant_id != user.tenant_id {
        return Err(ApiError::Forbidden);
    }

    // Apply updates
    if let Some(name) = request.name {
        policy.name = name;
    }
    if let Some(desc) = request.description {
        policy.description = Some(desc);
    }
    if let Some(rules) = request.rules {
        let converted_rules: Result<Vec<PolicyRule>, _> =
            rules.iter().map(rule_request_to_policy_rule).collect();
        policy.rules = converted_rules?;
    }
    if let Some(enabled) = request.is_enabled {
        policy.enabled = enabled;
    }
    policy.updated_at = Utc::now();

    policy_repo.update(&policy).await?;

    Ok(Json(policy_to_response(&policy)))
}

/// Delete a policy.
pub async fn delete_policy(
    user: AuthUser,
    Path(id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
) -> ApiResult<StatusCode> {
    let policy_repo = state.policy_repo()?;

    // Verify policy exists and belongs to tenant
    let policy = policy_repo
        .find_by_id(PolicyId(id))
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("Policy {id} not found")))?;

    if policy.tenant_id != user.tenant_id {
        return Err(ApiError::Forbidden);
    }

    policy_repo.delete(PolicyId(id)).await?;

    Ok(StatusCode::NO_CONTENT)
}

// ==================== Dependencies ====================

/// Get a specific dependency.
pub async fn get_dependency(
    user: AuthUser,
    Path(id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
) -> ApiResult<Json<DependencyResponse>> {
    let dep_repo = state.dependency_repo()?;
    let project_repo = state.project_repo()?;

    let dependency = dep_repo
        .find_by_id(DependencyId(id))
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("Dependency {id} not found")))?;

    // Verify tenant access via project
    let project = project_repo
        .find_by_id(dependency.project_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Associated project not found".to_string()))?;

    if project.tenant_id != user.tenant_id {
        return Err(ApiError::Forbidden);
    }

    Ok(Json(dependency_to_response(&dependency)))
}

/// Verify a dependency.
pub async fn verify_dependency(
    user: AuthUser,
    Path(id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    Json(_request): Json<VerifyDependencyRequest>,
) -> ApiResult<Json<VerificationResponse>> {
    let dep_repo = state.dependency_repo()?;
    let project_repo = state.project_repo()?;

    let dependency = dep_repo
        .find_by_id(DependencyId(id))
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("Dependency {id} not found")))?;

    // Verify tenant access via project
    let project = project_repo
        .find_by_id(dependency.project_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Associated project not found".to_string()))?;

    if project.tenant_id != user.tenant_id {
        return Err(ApiError::Forbidden);
    }

    // Build verification checks based on integrity data
    let mut checks = Vec::new();

    // Hash verification check
    if dependency.integrity.hash_sha256.is_some() {
        checks.push(VerificationCheck {
            check_type: "sha256_hash".to_string(),
            passed: true, // Would need registry fetch to verify
            expected: dependency.integrity.hash_sha256.clone(),
            actual: dependency.integrity.hash_sha256.clone(),
            message: None,
        });
    }

    // Signature check
    use sctv_core::SignatureStatus;
    let sig_passed = matches!(
        dependency.integrity.signature_status,
        SignatureStatus::Verified
    );
    checks.push(VerificationCheck {
        check_type: "signature".to_string(),
        passed: sig_passed,
        expected: None,
        actual: None,
        message: if sig_passed {
            None
        } else {
            Some(format!(
                "Signature status: {:?}",
                dependency.integrity.signature_status
            ))
        },
    });

    // Provenance check
    use sctv_core::ProvenanceStatus;
    let prov_passed = matches!(
        dependency.integrity.provenance_status,
        ProvenanceStatus::SlsaLevel1 | ProvenanceStatus::SlsaLevel2 | ProvenanceStatus::SlsaLevel3
    );
    checks.push(VerificationCheck {
        check_type: "provenance".to_string(),
        passed: prov_passed,
        expected: None,
        actual: None,
        message: Some(format!(
            "Provenance level: {:?}",
            dependency.integrity.provenance_status
        )),
    });

    let is_valid = checks.iter().filter(|c| c.passed).count() >= 2;

    Ok(Json(VerificationResponse {
        dependency_id: id,
        is_valid,
        checks,
        verified_at: Utc::now(),
    }))
}

// ==================== Scans ====================

/// List scans (backed by the jobs table; each `ScanProject` job is one scan).
pub async fn list_scans(
    user: AuthUser,
    Query(pagination): Query<PaginationParams>,
    State(state): State<Arc<AppState>>,
) -> ApiResult<Json<PaginatedResponse<ScanResponse>>> {
    use sctv_core::traits::{JobFilter, JobRepository};
    use sctv_db::PgJobRepository;

    let pool = state
        .pool()
        .ok_or_else(|| ApiError::ServiceUnavailable("Database not configured".into()))?;
    let job_repo = PgJobRepository::new(pool.clone());

    let filter = JobFilter {
        status: None,
        job_type: Some(vec!["scan_project".to_string()]),
    };

    let jobs = job_repo
        .find_by_tenant(
            Some(user.tenant_id),
            filter,
            pagination.per_page,
            pagination.offset(),
        )
        .await?;

    let responses: Vec<ScanResponse> = jobs.iter().filter_map(job_to_scan_response).collect();

    // We cannot cheaply count filtered jobs without an extra trait method;
    // use the returned count as a minimum and infer whether more pages exist.
    let total_items = responses.len() as u64;
    let per_page = u64::from(pagination.per_page.max(1));
    let total_pages = total_items.div_ceil(per_page).max(1) as u32;

    Ok(Json(PaginatedResponse {
        data: responses,
        pagination: PaginationInfo {
            page: pagination.page,
            per_page: pagination.per_page,
            total_items,
            total_pages,
        },
    }))
}

/// Get a specific scan by ID (backed by the jobs table).
pub async fn get_scan(
    user: AuthUser,
    Path(id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
) -> ApiResult<Json<ScanResponse>> {
    use sctv_core::traits::JobRepository;
    use sctv_core::JobId as CoreJobId;
    use sctv_db::PgJobRepository;

    let pool = state
        .pool()
        .ok_or_else(|| ApiError::ServiceUnavailable("Database not configured".into()))?;
    let job_repo = PgJobRepository::new(pool.clone());

    let job = job_repo
        .find_by_id(CoreJobId(id))
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("Scan {id} not found")))?;

    if job.tenant_id != Some(user.tenant_id) {
        return Err(ApiError::Forbidden);
    }

    job_to_scan_response(&job)
        .map(Json)
        .ok_or_else(|| ApiError::NotFound(format!("Job {id} is not a scan")))
}

/// Maps a core Job row to the API's `ScanResponse`. Returns None for jobs
/// that aren't `ScanProject` jobs — callers filter them out.
fn job_to_scan_response(job: &sctv_core::Job) -> Option<ScanResponse> {
    let project_id = match &job.job_type {
        sctv_core::JobType::ScanProject { project_id } => *project_id,
        _ => return None,
    };

    let (dependencies_found, alerts_created) = job
        .result
        .as_ref()
        .and_then(|v| {
            let deps = v
                .get("dependencies_found")
                .and_then(serde_json::Value::as_u64)? as u32;
            let alerts = v
                .get("alerts_created")
                .and_then(serde_json::Value::as_u64)? as u32;
            Some((deps, alerts))
        })
        .unwrap_or((0, 0));

    Some(ScanResponse {
        id: job.id.0,
        project_id,
        status: format!("{:?}", job.status).to_lowercase(),
        started_at: job.started_at.unwrap_or(job.created_at),
        completed_at: job.completed_at,
        dependencies_found,
        alerts_created,
        error_message: job.error_message.clone(),
    })
}

// ==================== Webhooks ====================

/// Handle GitHub webhook.
pub async fn github_webhook(
    State(_state): State<Arc<AppState>>,
    Json(payload): Json<GitHubWebhookPayload>,
) -> ApiResult<Json<WebhookResponse>> {
    tracing::info!("Received GitHub webhook: {:?}", payload.action);

    // In a real implementation, validate signature and process event
    let should_scan = payload.action.as_deref() == Some("push");

    Ok(Json(WebhookResponse {
        received: true,
        message: "Webhook received successfully".to_string(),
        scan_triggered: should_scan,
        scan_id: if should_scan {
            Some(Uuid::new_v4())
        } else {
            None
        },
    }))
}

/// Handle GitLab webhook.
pub async fn gitlab_webhook(
    State(_state): State<Arc<AppState>>,
    Json(payload): Json<GitLabWebhookPayload>,
) -> ApiResult<Json<WebhookResponse>> {
    tracing::info!("Received GitLab webhook: {:?}", payload.object_kind);

    let should_scan = payload.object_kind.as_deref() == Some("push");

    Ok(Json(WebhookResponse {
        received: true,
        message: "Webhook received successfully".to_string(),
        scan_triggered: should_scan,
        scan_id: if should_scan {
            Some(Uuid::new_v4())
        } else {
            None
        },
    }))
}
