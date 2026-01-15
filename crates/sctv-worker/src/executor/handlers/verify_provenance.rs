//! Verify provenance job executor.
//!
//! This executor verifies SLSA, Sigstore, and in-toto attestations for packages.

use async_trait::async_trait;
use std::time::Instant;

use sctv_core::{
    Alert, AlertId, AlertMetadata, AlertStatus, AlertType,
    ProvenanceFailureDetails, ProvenanceStatus, Remediation, Severity,
};
use sctv_core::traits::{AlertRepository, DependencyRepository};
use sctv_db::repositories::{PgAlertRepository, PgDependencyRepository};

use crate::error::{WorkerError, WorkerResult};
use crate::executor::{ExecutionContext, JobExecutor};
use crate::jobs::{
    Job, JobPayload, JobResult, JobType, ProvenanceVerificationStatus, SigstoreDetails,
    VerifyProvenancePayload, VerifyProvenanceResult,
};

/// Executor for verifying package provenance.
pub struct VerifyProvenanceExecutor;

impl VerifyProvenanceExecutor {
    /// Creates a new verify provenance executor.
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    async fn execute_verify(
        &self,
        payload: &VerifyProvenancePayload,
        ctx: &ExecutionContext,
    ) -> WorkerResult<VerifyProvenanceResult> {
        let start = Instant::now();

        let dependency_repo = PgDependencyRepository::new(ctx.db_pool.clone());
        let alert_repo = PgAlertRepository::new(ctx.db_pool.clone());

        tracing::info!(
            package = %payload.package_name,
            version = %payload.version,
            ecosystem = ?payload.ecosystem,
            "Starting provenance verification"
        );

        let mut slsa_status = None;
        let mut slsa_level = None;
        let mut sigstore_status = None;
        let mut sigstore_details = None;
        let mut intoto_status = None;
        let mut alert_created = false;

        // Verify SLSA attestations
        if payload.verify_slsa {
            let (status, level) = self.verify_slsa(payload, ctx).await?;
            slsa_status = Some(status);
            slsa_level = level;
        }

        // Verify Sigstore signatures
        if payload.verify_sigstore {
            let (status, details) = self.verify_sigstore(payload, ctx).await?;
            sigstore_status = Some(status);
            sigstore_details = details;
        }

        // Verify in-toto attestations
        if payload.verify_intoto {
            let status = self.verify_intoto(payload, ctx).await?;
            intoto_status = Some(status);
        }

        // Determine overall provenance status
        let overall_status = self.determine_overall_status(
            slsa_status.as_ref(),
            sigstore_status.as_ref(),
            intoto_status.as_ref(),
        );

        // Update dependency provenance status
        if let Ok(Some(mut dep)) = dependency_repo.find_by_id(payload.dependency_id).await {
            dep.integrity.provenance_status = overall_status;
            if let Err(e) = dependency_repo.update(&dep).await {
                tracing::warn!(error = %e, "Failed to update dependency provenance status");
            }
        }

        // Create alert if verification failed
        if overall_status == ProvenanceStatus::Failed {
            let alert = self.create_failure_alert(payload, &slsa_status, &sigstore_status)?;
            if let Err(e) = alert_repo.create(&alert).await {
                tracing::warn!(error = %e, "Failed to create provenance alert");
            } else {
                alert_created = true;
            }
        }

        let duration = start.elapsed();

        tracing::info!(
            package = %payload.package_name,
            version = %payload.version,
            slsa_status = ?slsa_status,
            slsa_level = ?slsa_level,
            sigstore_status = ?sigstore_status,
            intoto_status = ?intoto_status,
            alert_created = alert_created,
            duration_ms = duration.as_millis(),
            "Provenance verification completed"
        );

        Ok(VerifyProvenanceResult {
            slsa_status,
            slsa_level,
            sigstore_status,
            sigstore_details,
            intoto_status,
            alert_created,
        })
    }

    async fn verify_slsa(
        &self,
        payload: &VerifyProvenancePayload,
        _ctx: &ExecutionContext,
    ) -> WorkerResult<(ProvenanceVerificationStatus, Option<u8>)> {
        // In a real implementation, this would:
        // 1. Fetch attestations from the registry or transparency log
        // 2. Verify the SLSA provenance attestation
        // 3. Determine the SLSA level based on the attestation

        tracing::debug!(
            package = %payload.package_name,
            "Checking SLSA attestations"
        );

        // For now, return NoAttestations as we don't have real verification
        // In production, this would use the sigstore crate or similar
        Ok((ProvenanceVerificationStatus::NoAttestations, None))
    }

    async fn verify_sigstore(
        &self,
        payload: &VerifyProvenancePayload,
        _ctx: &ExecutionContext,
    ) -> WorkerResult<(ProvenanceVerificationStatus, Option<SigstoreDetails>)> {
        // In a real implementation, this would:
        // 1. Query Rekor transparency log for the package
        // 2. Verify the Sigstore signature
        // 3. Extract certificate details

        tracing::debug!(
            package = %payload.package_name,
            "Checking Sigstore signatures"
        );

        // For now, return NoAttestations as we don't have real verification
        Ok((ProvenanceVerificationStatus::NoAttestations, None))
    }

    async fn verify_intoto(
        &self,
        payload: &VerifyProvenancePayload,
        _ctx: &ExecutionContext,
    ) -> WorkerResult<ProvenanceVerificationStatus> {
        // In a real implementation, this would:
        // 1. Fetch in-toto link metadata
        // 2. Verify the supply chain layout
        // 3. Check all required steps were performed

        tracing::debug!(
            package = %payload.package_name,
            "Checking in-toto attestations"
        );

        // For now, return NoAttestations as we don't have real verification
        Ok(ProvenanceVerificationStatus::NoAttestations)
    }

    fn determine_overall_status(
        &self,
        slsa: Option<&ProvenanceVerificationStatus>,
        sigstore: Option<&ProvenanceVerificationStatus>,
        intoto: Option<&ProvenanceVerificationStatus>,
    ) -> ProvenanceStatus {
        let statuses = [slsa, sigstore, intoto];

        // If any verification failed, overall status is failed
        if statuses
            .iter()
            .any(|s| matches!(s, Some(ProvenanceVerificationStatus::Failed)))
        {
            return ProvenanceStatus::Failed;
        }

        // If any verification passed, overall status is verified (SLSA Level 1)
        if statuses
            .iter()
            .any(|s| matches!(s, Some(ProvenanceVerificationStatus::Verified)))
        {
            return ProvenanceStatus::SlsaLevel1;
        }

        // If all checks have no attestations, status is unknown
        ProvenanceStatus::Unknown
    }

    fn create_failure_alert(
        &self,
        payload: &VerifyProvenancePayload,
        slsa_status: &Option<ProvenanceVerificationStatus>,
        sigstore_status: &Option<ProvenanceVerificationStatus>,
    ) -> WorkerResult<Alert> {
        let mut attestation_errors = Vec::new();

        if matches!(slsa_status, Some(ProvenanceVerificationStatus::Failed)) {
            attestation_errors.push("SLSA attestation verification failed".to_string());
        }
        if matches!(sigstore_status, Some(ProvenanceVerificationStatus::Failed)) {
            attestation_errors.push("Sigstore signature verification failed".to_string());
        }

        let details = ProvenanceFailureDetails {
            package_name: payload.package_name.clone(),
            ecosystem: payload.ecosystem,
            version: payload.version.clone(),
            expected_slsa_level: 1,
            actual_slsa_level: None,
            attestation_errors,
        };

        // Note: We need a project_id for the alert, but it's not in the payload.
        // In a real implementation, we'd look this up from the dependency.
        // For now, we'll create a placeholder project ID.
        let project_id = sctv_core::ProjectId::new();

        Ok(Alert {
            id: AlertId::new(),
            tenant_id: payload.tenant_id,
            project_id,
            dependency_id: Some(payload.dependency_id),
            alert_type: AlertType::ProvenanceFailure(details.clone()),
            severity: Severity::High,
            title: format!(
                "Provenance verification failed for {}@{}",
                payload.package_name, payload.version
            ),
            description: format!(
                "The provenance verification for '{}@{}' failed. \
                 This could indicate that the package was not built from its claimed source. \
                 Errors: {}",
                payload.package_name,
                payload.version,
                details.attestation_errors.join(", ")
            ),
            status: AlertStatus::Open,
            remediation: Some(Remediation {
                action_taken: "Investigate the package provenance manually.".to_string(),
                new_version: None,
                notes: Some("Check the package's build provenance on the registry".to_string()),
            }),
            metadata: AlertMetadata::default(),
            created_at: chrono::Utc::now(),
            acknowledged_at: None,
            acknowledged_by: None,
            resolved_at: None,
            resolved_by: None,
        })
    }
}

impl Default for VerifyProvenanceExecutor {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl JobExecutor for VerifyProvenanceExecutor {
    fn handles(&self) -> Vec<JobType> {
        vec![JobType::VerifyProvenance]
    }

    async fn execute(&self, job: &Job, ctx: &ExecutionContext) -> WorkerResult<JobResult> {
        let payload = match &job.payload {
            JobPayload::VerifyProvenance(p) => p,
            _ => {
                return Err(WorkerError::Execution(
                    "Invalid payload type for VerifyProvenance".into(),
                ))
            }
        };

        let result = self.execute_verify(payload, ctx).await?;
        Ok(JobResult::VerifyProvenance(result))
    }

    fn default_timeout_secs(&self) -> u64 {
        120 // 2 minutes for provenance verification
    }
}
