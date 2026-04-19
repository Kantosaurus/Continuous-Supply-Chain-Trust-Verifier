//! Shared helpers used by multiple CLI subcommands.

use chrono::Utc;
use sctv_core::{
    Alert, AlertId, AlertMetadata, AlertStatus, AlertType, PackageEcosystem, ProjectId, Severity,
    TenantId, TyposquattingDetails,
};
use sctv_detectors::typosquatting::{Confidence, TyposquatCandidate};

/// Converts a typosquat candidate into an Alert suitable for SARIF emission.
///
/// The CLI has no tenant/project context, so synthetic IDs are used; the SARIF
/// emitter does not surface them to end users.
#[must_use]
pub fn typosquat_to_alert(candidate: &TyposquatCandidate, ecosystem: PackageEcosystem) -> Alert {
    let severity = match candidate.confidence {
        Confidence::High => Severity::High,
        Confidence::Medium => Severity::Medium,
        Confidence::Low => Severity::Low,
    };

    Alert {
        id: AlertId::default(),
        tenant_id: TenantId::new(),
        project_id: ProjectId::new(),
        dependency_id: None,
        alert_type: AlertType::Typosquatting(TyposquattingDetails {
            suspicious_package: candidate.suspicious_name.clone(),
            ecosystem,
            similar_popular_package: candidate.popular_name.clone(),
            similarity_score: candidate.similarity_score,
            detection_method: candidate.detection_method,
            popular_package_downloads: None,
        }),
        severity,
        title: format!(
            "Possible typosquatting of '{}'",
            candidate.popular_name
        ),
        description: format!(
            "Package '{}' looks similar to popular package '{}' (score {:.2}, method {:?}).",
            candidate.suspicious_name,
            candidate.popular_name,
            candidate.similarity_score,
            candidate.detection_method
        ),
        status: AlertStatus::Open,
        remediation: None,
        metadata: AlertMetadata::default(),
        created_at: Utc::now(),
        acknowledged_at: None,
        acknowledged_by: None,
        resolved_at: None,
        resolved_by: None,
    }
}

/// Emits SARIF JSON for the given alerts to stdout.
pub fn emit_sarif(alerts: &[Alert]) -> anyhow::Result<()> {
    let report = sctv_ci::SarifReport::from_alerts(alerts);
    let json = report.to_json()?;
    println!("{json}");
    Ok(())
}
