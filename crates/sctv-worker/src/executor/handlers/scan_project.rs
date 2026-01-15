//! Scan project job executor.
//!
//! This executor handles scanning a project's dependencies for security threats.

use async_trait::async_trait;
use std::time::Instant;

use sctv_core::{
    Alert, AlertId, AlertMetadata, AlertStatus, AlertType, Dependency, HashAlgorithm,
    Remediation, Severity, SignatureStatus, TamperingDetails, TyposquattingDetails,
};
use sctv_core::traits::{AlertRepository, DependencyRepository, ProjectRepository};
use sctv_db::repositories::{PgAlertRepository, PgDependencyRepository, PgProjectRepository};

use crate::error::{WorkerError, WorkerResult};
use crate::executor::{ExecutionContext, JobExecutor};
use crate::jobs::{
    Job, JobPayload, JobResult, JobType, ScanProjectPayload, ScanProjectResult,
};

/// Executor for scanning project dependencies.
pub struct ScanProjectExecutor;

impl ScanProjectExecutor {
    /// Creates a new scan project executor.
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    async fn execute_scan(
        &self,
        payload: &ScanProjectPayload,
        ctx: &ExecutionContext,
    ) -> WorkerResult<ScanProjectResult> {
        let start = Instant::now();

        let project_repo = PgProjectRepository::new(ctx.db_pool.clone());
        let dependency_repo = PgDependencyRepository::new(ctx.db_pool.clone());
        let alert_repo = PgAlertRepository::new(ctx.db_pool.clone());

        // Fetch the project
        let project = project_repo
            .find_by_id(payload.project_id)
            .await
            .map_err(|e| WorkerError::Execution(format!("Failed to fetch project: {}", e)))?
            .ok_or_else(|| WorkerError::Execution("Project not found".into()))?;

        tracing::info!(
            project_id = %payload.project_id,
            project_name = %project.name,
            "Starting project scan"
        );

        // Determine which ecosystems to scan
        let ecosystems = if payload.ecosystems.is_empty() {
            project.ecosystems.clone()
        } else {
            payload.ecosystems.clone()
        };

        // Fetch existing dependencies
        let existing_deps = dependency_repo
            .find_by_project(payload.project_id)
            .await
            .map_err(|e| WorkerError::Execution(format!("Failed to fetch dependencies: {}", e)))?;

        tracing::debug!(
            count = existing_deps.len(),
            "Found existing dependencies"
        );

        let mut alerts_created = 0u32;
        let dependencies_found = existing_deps.len() as u32;

        // Run detectors on dependencies
        for dep in &existing_deps {
            // Check for typosquatting
            if let Some(alert) = self.check_typosquatting(dep, payload).await? {
                if let Err(e) = alert_repo.create(&alert).await {
                    tracing::warn!(error = %e, "Failed to create typosquatting alert");
                } else {
                    alerts_created += 1;
                }
            }

            // Check for tampering (if hashes available)
            if let Some(alert) = self.check_tampering(dep, payload).await? {
                if let Err(e) = alert_repo.create(&alert).await {
                    tracing::warn!(error = %e, "Failed to create tampering alert");
                } else {
                    alerts_created += 1;
                }
            }
        }

        // Update project last_scan_at
        let mut updated_project = project.clone();
        updated_project.last_scan_at = Some(chrono::Utc::now());
        if let Err(e) = project_repo.update(&updated_project).await {
            tracing::warn!(error = %e, "Failed to update project last_scan_at");
        }

        let duration = start.elapsed();

        tracing::info!(
            project_id = %payload.project_id,
            dependencies_found = dependencies_found,
            alerts_created = alerts_created,
            duration_ms = duration.as_millis(),
            "Project scan completed"
        );

        Ok(ScanProjectResult {
            dependencies_found,
            alerts_created,
            scan_duration_ms: duration.as_millis() as u64,
        })
    }

    async fn check_typosquatting(
        &self,
        dep: &Dependency,
        payload: &ScanProjectPayload,
    ) -> WorkerResult<Option<Alert>> {
        use sctv_detectors::typosquatting::{Confidence, TyposquattingDetector};

        let detector = TyposquattingDetector::new();
        let candidates = detector.check(dep.ecosystem, &dep.package_name);

        if candidates.is_empty() {
            return Ok(None);
        }

        // Get the highest confidence finding
        let finding = candidates
            .into_iter()
            .max_by_key(|c| match c.confidence {
                Confidence::High => 3,
                Confidence::Medium => 2,
                Confidence::Low => 1,
            });

        if let Some(finding) = finding {
            let severity = match finding.confidence {
                Confidence::High => Severity::High,
                Confidence::Medium => Severity::Medium,
                Confidence::Low => Severity::Low,
            };

            let details = TyposquattingDetails {
                suspicious_package: dep.package_name.clone(),
                ecosystem: dep.ecosystem,
                similar_popular_package: finding.popular_name.clone(),
                similarity_score: finding.similarity_score,
                detection_method: finding.detection_method,
                popular_package_downloads: None,
            };

            let alert = Alert {
                id: AlertId::new(),
                tenant_id: payload.tenant_id,
                project_id: payload.project_id,
                dependency_id: Some(dep.id),
                alert_type: AlertType::Typosquatting(details),
                severity,
                title: format!(
                    "Potential typosquatting: {} similar to {}",
                    dep.package_name, finding.popular_name
                ),
                description: format!(
                    "The package '{}' has a name similar to the popular package '{}'. \
                     This could indicate a typosquatting attack. \
                     Similarity score: {:.2}, Detection method: {:?}",
                    dep.package_name,
                    finding.popular_name,
                    finding.similarity_score,
                    finding.detection_method
                ),
                status: AlertStatus::Open,
                remediation: Some(Remediation {
                    action_taken: format!(
                        "Verify that '{}' is the intended package. If not, replace it with '{}'.",
                        dep.package_name, finding.popular_name
                    ),
                    new_version: None,
                    notes: Some("Review the package source and maintainers".to_string()),
                }),
                metadata: AlertMetadata::default(),
                created_at: chrono::Utc::now(),
                acknowledged_at: None,
                acknowledged_by: None,
                resolved_at: None,
                resolved_by: None,
            };

            return Ok(Some(alert));
        }

        Ok(None)
    }

    async fn check_tampering(
        &self,
        dep: &Dependency,
        payload: &ScanProjectPayload,
    ) -> WorkerResult<Option<Alert>> {
        // Skip if no hash is available or signature is not invalid
        if dep.integrity.signature_status != SignatureStatus::Invalid {
            return Ok(None);
        }

        let details = TamperingDetails {
            package_name: dep.package_name.clone(),
            ecosystem: dep.ecosystem,
            version: dep.resolved_version.to_string(),
            expected_hash: dep.integrity.hash_sha256.clone().unwrap_or_default(),
            actual_hash: String::new(),
            algorithm: HashAlgorithm::Sha256,
            registry_source: "unknown".to_string(),
        };

        let alert = Alert {
            id: AlertId::new(),
            tenant_id: payload.tenant_id,
            project_id: payload.project_id,
            dependency_id: Some(dep.id),
            alert_type: AlertType::DependencyTampering(details),
            severity: Severity::Critical,
            title: format!(
                "Signature verification failed for {}@{}",
                dep.package_name, dep.resolved_version
            ),
            description: format!(
                "The package '{}@{}' has an invalid signature. \
                 This could indicate that the package has been tampered with.",
                dep.package_name, dep.resolved_version
            ),
            status: AlertStatus::Open,
            remediation: Some(Remediation {
                action_taken: "Investigate the package integrity and consider using an alternative.".to_string(),
                new_version: None,
                notes: Some("Verify the package checksum manually".to_string()),
            }),
            metadata: AlertMetadata::default(),
            created_at: chrono::Utc::now(),
            acknowledged_at: None,
            acknowledged_by: None,
            resolved_at: None,
            resolved_by: None,
        };

        Ok(Some(alert))
    }
}

impl Default for ScanProjectExecutor {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl JobExecutor for ScanProjectExecutor {
    fn handles(&self) -> Vec<JobType> {
        vec![JobType::ScanProject]
    }

    async fn execute(&self, job: &Job, ctx: &ExecutionContext) -> WorkerResult<JobResult> {
        let payload = match &job.payload {
            JobPayload::ScanProject(p) => p,
            _ => {
                return Err(WorkerError::Execution(
                    "Invalid payload type for ScanProject".into(),
                ))
            }
        };

        let result = self.execute_scan(payload, ctx).await?;
        Ok(JobResult::ScanProject(result))
    }

    fn default_timeout_secs(&self) -> u64 {
        600 // 10 minutes for full project scans
    }
}
