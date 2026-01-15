//! GraphQL schema and resolvers.

use async_graphql::{
    Context, EmptySubscription, InputObject, Object, Result, Schema, SimpleObject, ID,
};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::{extract::State, routing::post, Extension, Router};
use chrono::{DateTime, Utc};
use sctv_core::traits::AlertFilter;
use sctv_core::{AlertStatus, PackageEcosystem, ProjectStatus, Severity};
use std::sync::Arc;
use uuid::Uuid;

use crate::AppState;

// ==================== Context Data ====================

/// GraphQL context data containing state and user info.
pub struct GqlContext {
    pub state: Arc<AppState>,
    pub tenant_id: Option<sctv_core::TenantId>,
    pub user_id: Option<Uuid>,
}

// ==================== Query Root ====================

/// GraphQL query root.
pub struct QueryRoot;

#[Object]
impl QueryRoot {
    /// Get all projects for the current tenant.
    async fn projects(
        &self,
        ctx: &Context<'_>,
        #[graphql(default = 1)] page: i32,
        #[graphql(default = 20)] per_page: i32,
    ) -> Result<Vec<Project>> {
        let gql_ctx = ctx.data::<GqlContext>()?;
        let state = &gql_ctx.state;

        let tenant_id = gql_ctx
            .tenant_id
            .ok_or_else(|| async_graphql::Error::new("Authentication required"))?;

        let project_repo = state
            .project_repo()
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;

        let alert_repo = state
            .alert_repo()
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;

        let dep_repo = state
            .dependency_repo()
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;

        let projects = project_repo
            .find_by_tenant(tenant_id)
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;

        // Apply pagination
        let offset = ((page - 1) * per_page).max(0) as usize;
        let limit = per_page as usize;

        let mut result = Vec::new();
        for project in projects.into_iter().skip(offset).take(limit) {
            let deps = dep_repo.find_by_project(project.id).await.unwrap_or_default();
            let alert_count = alert_repo.count_open_by_project(project.id).await.unwrap_or(0);

            result.push(Project {
                id: ID::from(project.id.0.to_string()),
                name: project.name,
                description: project.description,
                repository_url: project.repository_url.map(|u| u.to_string()),
                status: project.status,
                is_active: true,
                dependency_count: deps.len() as i32,
                alert_count: alert_count as i32,
                last_scan_at: project.last_scan_at,
                created_at: project.created_at,
            });
        }

        Ok(result)
    }

    /// Get a project by ID.
    async fn project(&self, ctx: &Context<'_>, id: ID) -> Result<Option<Project>> {
        let gql_ctx = ctx.data::<GqlContext>()?;
        let state = &gql_ctx.state;

        let tenant_id = gql_ctx
            .tenant_id
            .ok_or_else(|| async_graphql::Error::new("Authentication required"))?;

        let project_repo = state
            .project_repo()
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;

        let alert_repo = state
            .alert_repo()
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;

        let dep_repo = state
            .dependency_repo()
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;

        let uuid = Uuid::parse_str(&id).map_err(|_| async_graphql::Error::new("Invalid ID"))?;

        let project = match project_repo
            .find_by_id(sctv_core::ProjectId(uuid))
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?
        {
            Some(p) => p,
            None => return Ok(None),
        };

        // Verify tenant access
        if project.tenant_id != tenant_id {
            return Err(async_graphql::Error::new("Access denied"));
        }

        let deps = dep_repo.find_by_project(project.id).await.unwrap_or_default();
        let alert_count = alert_repo.count_open_by_project(project.id).await.unwrap_or(0);

        Ok(Some(Project {
            id: ID::from(project.id.0.to_string()),
            name: project.name,
            description: project.description,
            repository_url: project.repository_url.map(|u| u.to_string()),
            status: project.status,
            is_active: true,
            dependency_count: deps.len() as i32,
            alert_count: alert_count as i32,
            last_scan_at: project.last_scan_at,
            created_at: project.created_at,
        }))
    }

    /// Get all alerts with optional filters.
    async fn alerts(
        &self,
        ctx: &Context<'_>,
        project_id: Option<ID>,
        severity: Option<Severity>,
        status: Option<AlertStatus>,
        #[graphql(default = 1)] page: i32,
        #[graphql(default = 20)] per_page: i32,
    ) -> Result<Vec<Alert>> {
        let gql_ctx = ctx.data::<GqlContext>()?;
        let state = &gql_ctx.state;

        let tenant_id = gql_ctx
            .tenant_id
            .ok_or_else(|| async_graphql::Error::new("Authentication required"))?;

        let alert_repo = state
            .alert_repo()
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;

        // Build filter
        let filter = AlertFilter {
            project_id: project_id
                .and_then(|id| Uuid::parse_str(&id).ok())
                .map(sctv_core::ProjectId),
            status: status.map(|s| vec![s]),
            severity: severity.map(|s| vec![s]),
            alert_type: None,
        };

        let offset = ((page - 1) * per_page).max(0) as u32;

        let alerts = alert_repo
            .find_with_filter(tenant_id, filter, per_page as u32, offset)
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;

        Ok(alerts
            .into_iter()
            .map(|a| {
                let (dep_name, dep_version) = extract_dependency_info(&a.alert_type);
                Alert {
                    id: ID::from(a.id.0.to_string()),
                    project_id: ID::from(a.project_id.0.to_string()),
                    alert_type: a.alert_type.type_name().to_string(),
                    severity: a.severity,
                    title: a.title,
                    description: a.description,
                    status: a.status,
                    dependency_name: dep_name,
                    dependency_version: dep_version,
                    created_at: a.created_at,
                    acknowledged_at: a.acknowledged_at,
                    resolved_at: a.resolved_at,
                }
            })
            .collect())
    }

    /// Get an alert by ID.
    async fn alert(&self, ctx: &Context<'_>, id: ID) -> Result<Option<Alert>> {
        let gql_ctx = ctx.data::<GqlContext>()?;
        let state = &gql_ctx.state;

        let tenant_id = gql_ctx
            .tenant_id
            .ok_or_else(|| async_graphql::Error::new("Authentication required"))?;

        let alert_repo = state
            .alert_repo()
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;

        let uuid = Uuid::parse_str(&id).map_err(|_| async_graphql::Error::new("Invalid ID"))?;

        let alert = match alert_repo
            .find_by_id(sctv_core::AlertId(uuid))
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?
        {
            Some(a) => a,
            None => return Ok(None),
        };

        // Verify tenant access
        if alert.tenant_id != tenant_id {
            return Err(async_graphql::Error::new("Access denied"));
        }

        let (dep_name, dep_version) = extract_dependency_info(&alert.alert_type);

        Ok(Some(Alert {
            id: ID::from(alert.id.0.to_string()),
            project_id: ID::from(alert.project_id.0.to_string()),
            alert_type: alert.alert_type.type_name().to_string(),
            severity: alert.severity,
            title: alert.title,
            description: alert.description,
            status: alert.status,
            dependency_name: dep_name,
            dependency_version: dep_version,
            created_at: alert.created_at,
            acknowledged_at: alert.acknowledged_at,
            resolved_at: alert.resolved_at,
        }))
    }

    /// Get dependencies for a project.
    async fn dependencies(
        &self,
        ctx: &Context<'_>,
        project_id: ID,
        ecosystem: Option<PackageEcosystem>,
        is_direct: Option<bool>,
        #[graphql(default = 1)] page: i32,
        #[graphql(default = 50)] per_page: i32,
    ) -> Result<Vec<Dependency>> {
        let gql_ctx = ctx.data::<GqlContext>()?;
        let state = &gql_ctx.state;

        let tenant_id = gql_ctx
            .tenant_id
            .ok_or_else(|| async_graphql::Error::new("Authentication required"))?;

        let project_repo = state
            .project_repo()
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;

        let dep_repo = state
            .dependency_repo()
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;

        let uuid =
            Uuid::parse_str(&project_id).map_err(|_| async_graphql::Error::new("Invalid ID"))?;

        // Verify project access
        let project = project_repo
            .find_by_id(sctv_core::ProjectId(uuid))
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?
            .ok_or_else(|| async_graphql::Error::new("Project not found"))?;

        if project.tenant_id != tenant_id {
            return Err(async_graphql::Error::new("Access denied"));
        }

        // Fetch dependencies
        let dependencies = if is_direct == Some(true) {
            dep_repo
                .find_direct_by_project(sctv_core::ProjectId(uuid))
                .await
        } else {
            dep_repo
                .find_by_project(sctv_core::ProjectId(uuid))
                .await
        }
        .map_err(|e| async_graphql::Error::new(e.to_string()))?;

        // Apply filters and pagination
        let offset = ((page - 1) * per_page).max(0) as usize;
        let limit = per_page as usize;

        let filtered: Vec<_> = dependencies
            .into_iter()
            .filter(|d| ecosystem.map_or(true, |e| d.ecosystem == e))
            .filter(|d| is_direct.map_or(true, |direct| d.is_direct == direct))
            .skip(offset)
            .take(limit)
            .map(|d| Dependency {
                id: ID::from(d.id.0.to_string()),
                project_id: ID::from(d.project_id.0.to_string()),
                package_name: d.package_name,
                ecosystem: d.ecosystem,
                version_constraint: d.version_constraint,
                resolved_version: d.resolved_version.to_string(),
                is_direct: d.is_direct,
                is_dev_dependency: d.is_dev_dependency,
                depth: d.depth as i32,
                hash_sha256: d.integrity.hash_sha256,
                signature_verified: matches!(
                    d.integrity.signature_status,
                    sctv_core::SignatureStatus::Verified
                ),
                provenance_level: match d.integrity.provenance_status {
                    sctv_core::ProvenanceStatus::SlsaLevel0 => Some(0),
                    sctv_core::ProvenanceStatus::SlsaLevel1 => Some(1),
                    sctv_core::ProvenanceStatus::SlsaLevel2 => Some(2),
                    sctv_core::ProvenanceStatus::SlsaLevel3 => Some(3),
                    _ => None,
                },
                first_seen_at: d.first_seen_at,
                last_verified_at: d.last_verified_at,
            })
            .collect();

        Ok(filtered)
    }

    /// Get all policies for the current tenant.
    async fn policies(&self, ctx: &Context<'_>) -> Result<Vec<Policy>> {
        let gql_ctx = ctx.data::<GqlContext>()?;
        let state = &gql_ctx.state;

        let tenant_id = gql_ctx
            .tenant_id
            .ok_or_else(|| async_graphql::Error::new("Authentication required"))?;

        let policy_repo = state
            .policy_repo()
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;

        let policies = policy_repo
            .find_by_tenant(tenant_id)
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;

        Ok(policies
            .into_iter()
            .map(|p| Policy {
                id: ID::from(p.id.0.to_string()),
                name: p.name,
                description: p.description,
                severity: Severity::High, // Default display severity
                is_enabled: p.enabled,
                created_at: p.created_at,
                updated_at: p.updated_at,
            })
            .collect())
    }
}

// ==================== Mutation Root ====================

/// GraphQL mutation root.
pub struct MutationRoot;

#[Object]
impl MutationRoot {
    /// Create a new project.
    async fn create_project(&self, ctx: &Context<'_>, input: CreateProjectInput) -> Result<Project> {
        let gql_ctx = ctx.data::<GqlContext>()?;
        let state = &gql_ctx.state;

        let tenant_id = gql_ctx
            .tenant_id
            .ok_or_else(|| async_graphql::Error::new("Authentication required"))?;

        let project_repo = state
            .project_repo()
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;

        let mut project = sctv_core::Project::new(tenant_id, input.name);
        project.description = input.description;
        project.repository_url = input.repository_url.and_then(|u| url::Url::parse(&u).ok());

        project_repo
            .create(&project)
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;

        Ok(Project {
            id: ID::from(project.id.0.to_string()),
            name: project.name,
            description: project.description,
            repository_url: project.repository_url.map(|u| u.to_string()),
            status: ProjectStatus::Unknown,
            is_active: true,
            dependency_count: 0,
            alert_count: 0,
            last_scan_at: None,
            created_at: project.created_at,
        })
    }

    /// Update a project.
    async fn update_project(
        &self,
        ctx: &Context<'_>,
        id: ID,
        input: UpdateProjectInput,
    ) -> Result<Option<Project>> {
        let gql_ctx = ctx.data::<GqlContext>()?;
        let state = &gql_ctx.state;

        let tenant_id = gql_ctx
            .tenant_id
            .ok_or_else(|| async_graphql::Error::new("Authentication required"))?;

        let project_repo = state
            .project_repo()
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;

        let alert_repo = state
            .alert_repo()
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;

        let dep_repo = state
            .dependency_repo()
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;

        let uuid = Uuid::parse_str(&id).map_err(|_| async_graphql::Error::new("Invalid ID"))?;

        let mut project = match project_repo
            .find_by_id(sctv_core::ProjectId(uuid))
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?
        {
            Some(p) => p,
            None => return Ok(None),
        };

        // Verify tenant access
        if project.tenant_id != tenant_id {
            return Err(async_graphql::Error::new("Access denied"));
        }

        // Apply updates
        if let Some(name) = input.name {
            project.name = name;
        }
        if let Some(desc) = input.description {
            project.description = Some(desc);
        }
        if let Some(url) = input.repository_url {
            project.repository_url = url::Url::parse(&url).ok();
        }
        project.updated_at = Utc::now();

        project_repo
            .update(&project)
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;

        let deps = dep_repo.find_by_project(project.id).await.unwrap_or_default();
        let alert_count = alert_repo.count_open_by_project(project.id).await.unwrap_or(0);

        Ok(Some(Project {
            id: ID::from(project.id.0.to_string()),
            name: project.name,
            description: project.description,
            repository_url: project.repository_url.map(|u| u.to_string()),
            status: project.status,
            is_active: true,
            dependency_count: deps.len() as i32,
            alert_count: alert_count as i32,
            last_scan_at: project.last_scan_at,
            created_at: project.created_at,
        }))
    }

    /// Delete a project.
    async fn delete_project(&self, ctx: &Context<'_>, id: ID) -> Result<bool> {
        let gql_ctx = ctx.data::<GqlContext>()?;
        let state = &gql_ctx.state;

        let tenant_id = gql_ctx
            .tenant_id
            .ok_or_else(|| async_graphql::Error::new("Authentication required"))?;

        let project_repo = state
            .project_repo()
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;

        let uuid = Uuid::parse_str(&id).map_err(|_| async_graphql::Error::new("Invalid ID"))?;

        // Verify project exists and belongs to tenant
        let project = match project_repo
            .find_by_id(sctv_core::ProjectId(uuid))
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?
        {
            Some(p) => p,
            None => return Ok(false),
        };

        if project.tenant_id != tenant_id {
            return Err(async_graphql::Error::new("Access denied"));
        }

        project_repo
            .delete(sctv_core::ProjectId(uuid))
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;

        Ok(true)
    }

    /// Trigger a scan for a project.
    async fn trigger_scan(
        &self,
        ctx: &Context<'_>,
        project_id: ID,
        _full_scan: Option<bool>,
    ) -> Result<Scan> {
        let gql_ctx = ctx.data::<GqlContext>()?;
        let state = &gql_ctx.state;

        let tenant_id = gql_ctx
            .tenant_id
            .ok_or_else(|| async_graphql::Error::new("Authentication required"))?;

        let project_repo = state
            .project_repo()
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;

        let uuid =
            Uuid::parse_str(&project_id).map_err(|_| async_graphql::Error::new("Invalid ID"))?;

        // Verify project exists and belongs to tenant
        let project = project_repo
            .find_by_id(sctv_core::ProjectId(uuid))
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?
            .ok_or_else(|| async_graphql::Error::new("Project not found"))?;

        if project.tenant_id != tenant_id {
            return Err(async_graphql::Error::new("Access denied"));
        }

        // In a full implementation, this would enqueue a scan job
        Ok(Scan {
            id: ID::from(Uuid::new_v4().to_string()),
            project_id,
            status: "queued".to_string(),
            started_at: Utc::now(),
            completed_at: None,
            dependencies_found: 0,
            alerts_created: 0,
            error_message: None,
        })
    }

    /// Acknowledge an alert.
    async fn acknowledge_alert(
        &self,
        ctx: &Context<'_>,
        id: ID,
        _notes: Option<String>,
    ) -> Result<Option<Alert>> {
        let gql_ctx = ctx.data::<GqlContext>()?;
        let state = &gql_ctx.state;

        let tenant_id = gql_ctx
            .tenant_id
            .ok_or_else(|| async_graphql::Error::new("Authentication required"))?;

        let user_id = gql_ctx
            .user_id
            .ok_or_else(|| async_graphql::Error::new("Authentication required"))?;

        let alert_repo = state
            .alert_repo()
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;

        let uuid = Uuid::parse_str(&id).map_err(|_| async_graphql::Error::new("Invalid ID"))?;

        let mut alert = match alert_repo
            .find_by_id(sctv_core::AlertId(uuid))
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?
        {
            Some(a) => a,
            None => return Ok(None),
        };

        // Verify tenant access
        if alert.tenant_id != tenant_id {
            return Err(async_graphql::Error::new("Access denied"));
        }

        alert.acknowledge(user_id);

        alert_repo
            .update(&alert)
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;

        let (dep_name, dep_version) = extract_dependency_info(&alert.alert_type);

        Ok(Some(Alert {
            id: ID::from(alert.id.0.to_string()),
            project_id: ID::from(alert.project_id.0.to_string()),
            alert_type: alert.alert_type.type_name().to_string(),
            severity: alert.severity,
            title: alert.title,
            description: alert.description,
            status: alert.status,
            dependency_name: dep_name,
            dependency_version: dep_version,
            created_at: alert.created_at,
            acknowledged_at: alert.acknowledged_at,
            resolved_at: alert.resolved_at,
        }))
    }

    /// Resolve an alert.
    async fn resolve_alert(
        &self,
        ctx: &Context<'_>,
        id: ID,
        action_taken: String,
        new_version: Option<String>,
    ) -> Result<Option<Alert>> {
        let gql_ctx = ctx.data::<GqlContext>()?;
        let state = &gql_ctx.state;

        let tenant_id = gql_ctx
            .tenant_id
            .ok_or_else(|| async_graphql::Error::new("Authentication required"))?;

        let user_id = gql_ctx
            .user_id
            .ok_or_else(|| async_graphql::Error::new("Authentication required"))?;

        let alert_repo = state
            .alert_repo()
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;

        let uuid = Uuid::parse_str(&id).map_err(|_| async_graphql::Error::new("Invalid ID"))?;

        let mut alert = match alert_repo
            .find_by_id(sctv_core::AlertId(uuid))
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?
        {
            Some(a) => a,
            None => return Ok(None),
        };

        // Verify tenant access
        if alert.tenant_id != tenant_id {
            return Err(async_graphql::Error::new("Access denied"));
        }

        let remediation = sctv_core::Remediation {
            action_taken,
            new_version: new_version.and_then(|v| semver::Version::parse(&v).ok()),
            notes: None,
        };
        alert.resolve(user_id, remediation);

        alert_repo
            .update(&alert)
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;

        let (dep_name, dep_version) = extract_dependency_info(&alert.alert_type);

        Ok(Some(Alert {
            id: ID::from(alert.id.0.to_string()),
            project_id: ID::from(alert.project_id.0.to_string()),
            alert_type: alert.alert_type.type_name().to_string(),
            severity: alert.severity,
            title: alert.title,
            description: alert.description,
            status: alert.status,
            dependency_name: dep_name,
            dependency_version: dep_version,
            created_at: alert.created_at,
            acknowledged_at: alert.acknowledged_at,
            resolved_at: alert.resolved_at,
        }))
    }

    /// Create a policy.
    async fn create_policy(&self, ctx: &Context<'_>, input: CreatePolicyInput) -> Result<Policy> {
        let gql_ctx = ctx.data::<GqlContext>()?;
        let state = &gql_ctx.state;

        let tenant_id = gql_ctx
            .tenant_id
            .ok_or_else(|| async_graphql::Error::new("Authentication required"))?;

        let policy_repo = state
            .policy_repo()
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;

        let now = Utc::now();
        let policy = sctv_core::Policy {
            id: sctv_core::PolicyId::new(),
            tenant_id,
            name: input.name,
            description: input.description,
            rules: Vec::new(), // Rules would need to be added via separate input
            severity_overrides: Vec::new(),
            is_default: false,
            enabled: input.is_enabled.unwrap_or(true),
            created_at: now,
            updated_at: now,
        };

        policy_repo
            .create(&policy)
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;

        Ok(Policy {
            id: ID::from(policy.id.0.to_string()),
            name: policy.name,
            description: policy.description,
            severity: input.severity,
            is_enabled: policy.enabled,
            created_at: policy.created_at,
            updated_at: policy.updated_at,
        })
    }
}

// ==================== Helper Functions ====================

fn extract_dependency_info(alert_type: &sctv_core::AlertType) -> (Option<String>, Option<String>) {
    use sctv_core::AlertType;
    match alert_type {
        AlertType::DependencyTampering(d) => (Some(d.package_name.clone()), Some(d.version.clone())),
        AlertType::DowngradeAttack(d) => {
            (Some(d.package_name.clone()), Some(d.current_version.to_string()))
        }
        AlertType::Typosquatting(d) => (Some(d.suspicious_package.clone()), None),
        AlertType::ProvenanceFailure(d) => (Some(d.package_name.clone()), Some(d.version.clone())),
        AlertType::PolicyViolation(_) => (None, None),
        AlertType::NewPackage(d) => (Some(d.package_name.clone()), Some(d.version.clone())),
        AlertType::SuspiciousMaintainer(d) => (Some(d.package_name.clone()), None),
    }
}

// ==================== Types ====================

/// Project type.
#[derive(SimpleObject)]
pub struct Project {
    pub id: ID,
    pub name: String,
    pub description: Option<String>,
    pub repository_url: Option<String>,
    pub status: ProjectStatus,
    pub is_active: bool,
    pub dependency_count: i32,
    pub alert_count: i32,
    pub last_scan_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

/// Alert type.
#[derive(SimpleObject)]
pub struct Alert {
    pub id: ID,
    pub project_id: ID,
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

/// Dependency type.
#[derive(SimpleObject)]
pub struct Dependency {
    pub id: ID,
    pub project_id: ID,
    pub package_name: String,
    pub ecosystem: PackageEcosystem,
    pub version_constraint: String,
    pub resolved_version: String,
    pub is_direct: bool,
    pub is_dev_dependency: bool,
    pub depth: i32,
    pub hash_sha256: Option<String>,
    pub signature_verified: bool,
    pub provenance_level: Option<i32>,
    pub first_seen_at: DateTime<Utc>,
    pub last_verified_at: DateTime<Utc>,
}

/// Policy type.
#[derive(SimpleObject)]
pub struct Policy {
    pub id: ID,
    pub name: String,
    pub description: Option<String>,
    pub severity: Severity,
    pub is_enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Scan type.
#[derive(SimpleObject)]
pub struct Scan {
    pub id: ID,
    pub project_id: ID,
    pub status: String,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub dependencies_found: i32,
    pub alerts_created: i32,
    pub error_message: Option<String>,
}

// ==================== Inputs ====================

/// Input for creating a project.
#[derive(InputObject)]
pub struct CreateProjectInput {
    pub name: String,
    pub description: Option<String>,
    pub repository_url: Option<String>,
}

/// Input for updating a project.
#[derive(InputObject)]
pub struct UpdateProjectInput {
    pub name: Option<String>,
    pub description: Option<String>,
    pub repository_url: Option<String>,
    pub is_active: Option<bool>,
}

/// Input for creating a policy.
#[derive(InputObject)]
pub struct CreatePolicyInput {
    pub name: String,
    pub description: Option<String>,
    pub severity: Severity,
    pub is_enabled: Option<bool>,
}

// ==================== Schema ====================

/// The GraphQL schema type.
pub type ApiSchema = Schema<QueryRoot, MutationRoot, EmptySubscription>;

/// Creates the GraphQL schema.
pub fn create_schema(_state: Arc<AppState>) -> ApiSchema {
    Schema::build(QueryRoot, MutationRoot, EmptySubscription).finish()
}

/// GraphQL handler.
async fn graphql_handler(
    State(state): State<Arc<AppState>>,
    Extension(schema): Extension<ApiSchema>,
    headers: axum::http::HeaderMap,
    request: GraphQLRequest,
) -> GraphQLResponse {
    // Extract auth info from headers
    let auth_header = headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok());

    let (tenant_id, user_id) = if let Some(auth) = auth_header {
        if let Some(token) = auth.strip_prefix("Bearer ") {
            match crate::auth::decode_token(token, &state.jwt_secret) {
                Ok(claims) => (Some(claims.tenant_id()), Some(claims.sub)),
                Err(_) => (None, None),
            }
        } else {
            (None, None)
        }
    } else {
        (None, None)
    };

    let gql_ctx = GqlContext {
        state: state.clone(),
        tenant_id,
        user_id,
    };

    schema
        .execute(request.into_inner().data(gql_ctx))
        .await
        .into()
}

/// Creates the GraphQL router.
pub fn routes(state: Arc<AppState>) -> Router<Arc<AppState>> {
    let schema = create_schema(state.clone());

    Router::new()
        .route("/", post(graphql_handler))
        .layer(Extension(schema))
}
