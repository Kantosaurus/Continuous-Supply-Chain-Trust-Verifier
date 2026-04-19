//! Job payload and result types for each job type.
//!
//! This module contains the specific data structures for each type of background job.

use sctv_core::{AlertId, DependencyId, PackageEcosystem, ProjectId, Severity, TenantId};
use serde::{Deserialize, Serialize};

// ============================================================================
// SCAN PROJECT
// ============================================================================

/// Payload for scanning a project's dependencies.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanProjectPayload {
    /// ID of the project to scan.
    pub project_id: ProjectId,
    /// Tenant that owns the project.
    pub tenant_id: TenantId,
    /// Ecosystems to scan (empty means all configured ecosystems).
    pub ecosystems: Vec<PackageEcosystem>,
    /// Whether to perform a full scan (ignores cache).
    pub full_scan: bool,
}

impl ScanProjectPayload {
    /// Creates a new scan project payload.
    #[must_use]
    pub const fn new(project_id: ProjectId, tenant_id: TenantId) -> Self {
        Self {
            project_id,
            tenant_id,
            ecosystems: Vec::new(),
            full_scan: false,
        }
    }

    /// Specifies ecosystems to scan.
    #[must_use]
    pub fn with_ecosystems(mut self, ecosystems: Vec<PackageEcosystem>) -> Self {
        self.ecosystems = ecosystems;
        self
    }

    /// Enables full scan mode (ignores cache).
    #[must_use]
    pub const fn full_scan(mut self) -> Self {
        self.full_scan = true;
        self
    }
}

/// Result of a project scan.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanProjectResult {
    /// Number of dependencies found.
    pub dependencies_found: u32,
    /// Number of alerts created.
    pub alerts_created: u32,
    /// Time taken in milliseconds.
    pub scan_duration_ms: u64,
}

// ============================================================================
// MONITOR REGISTRY
// ============================================================================

/// Payload for monitoring a package registry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitorRegistryPayload {
    /// The ecosystem/registry to monitor.
    pub ecosystem: PackageEcosystem,
    /// Specific packages to check (empty means all watched packages).
    pub packages: Vec<String>,
    /// Whether to check for new versions.
    pub check_new_versions: bool,
    /// Whether to check for package removals/yanks.
    pub check_removals: bool,
    /// Whether to check maintainer changes.
    pub check_maintainer_changes: bool,
}

impl MonitorRegistryPayload {
    /// Creates a new monitor registry payload.
    #[must_use]
    pub const fn new(ecosystem: PackageEcosystem) -> Self {
        Self {
            ecosystem,
            packages: Vec::new(),
            check_new_versions: true,
            check_removals: true,
            check_maintainer_changes: true,
        }
    }

    /// Specifies packages to monitor.
    #[must_use]
    pub fn with_packages(mut self, packages: Vec<String>) -> Self {
        self.packages = packages;
        self
    }

    /// Configures what to check.
    #[must_use]
    pub const fn check_only(
        mut self,
        new_versions: bool,
        removals: bool,
        maintainer_changes: bool,
    ) -> Self {
        self.check_new_versions = new_versions;
        self.check_removals = removals;
        self.check_maintainer_changes = maintainer_changes;
        self
    }
}

/// Result of monitoring a registry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitorRegistryResult {
    /// Number of packages checked.
    pub packages_checked: u32,
    /// Number of new versions detected.
    pub new_versions_detected: u32,
    /// Number of removed/yanked packages detected.
    pub removals_detected: u32,
    /// Number of maintainer changes detected.
    pub maintainer_changes_detected: u32,
    /// Number of alerts created.
    pub alerts_created: u32,
}

// ============================================================================
// VERIFY PROVENANCE
// ============================================================================

/// Payload for verifying package provenance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyProvenancePayload {
    /// Dependency to verify.
    pub dependency_id: DependencyId,
    /// Tenant that owns the dependency.
    pub tenant_id: TenantId,
    /// Package ecosystem.
    pub ecosystem: PackageEcosystem,
    /// Package name.
    pub package_name: String,
    /// Package version.
    pub version: String,
    /// Whether to verify SLSA attestations.
    pub verify_slsa: bool,
    /// Whether to verify Sigstore signatures.
    pub verify_sigstore: bool,
    /// Whether to verify in-toto attestations.
    pub verify_intoto: bool,
}

impl VerifyProvenancePayload {
    /// Creates a new verify provenance payload.
    #[must_use]
    pub const fn new(
        dependency_id: DependencyId,
        tenant_id: TenantId,
        ecosystem: PackageEcosystem,
        package_name: String,
        version: String,
    ) -> Self {
        Self {
            dependency_id,
            tenant_id,
            ecosystem,
            package_name,
            version,
            verify_slsa: true,
            verify_sigstore: true,
            verify_intoto: true,
        }
    }

    /// Configures which verification methods to use.
    #[must_use]
    pub const fn verify_only(mut self, slsa: bool, sigstore: bool, intoto: bool) -> Self {
        self.verify_slsa = slsa;
        self.verify_sigstore = sigstore;
        self.verify_intoto = intoto;
        self
    }
}

/// Status of a provenance verification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProvenanceVerificationStatus {
    /// Verification passed.
    Verified,
    /// Verification failed.
    Failed,
    /// No attestations available.
    NoAttestations,
    /// Attestations present but not verifiable.
    Unverifiable,
}

/// Result of provenance verification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyProvenanceResult {
    /// SLSA verification status.
    pub slsa_status: Option<ProvenanceVerificationStatus>,
    /// SLSA level if verified.
    pub slsa_level: Option<u8>,
    /// Sigstore verification status.
    pub sigstore_status: Option<ProvenanceVerificationStatus>,
    /// Sigstore certificate details.
    pub sigstore_details: Option<SigstoreDetails>,
    /// in-toto verification status.
    pub intoto_status: Option<ProvenanceVerificationStatus>,
    /// Whether an alert was created.
    pub alert_created: bool,
}

/// Details from Sigstore verification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SigstoreDetails {
    /// Certificate issuer.
    pub issuer: Option<String>,
    /// Subject identity.
    pub subject: Option<String>,
    /// Transparency log entry.
    pub rekor_entry: Option<String>,
}

// ============================================================================
// SEND NOTIFICATION
// ============================================================================

/// Channel to send notification through.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NotificationChannel {
    /// Email notification.
    Email,
    /// Slack webhook.
    Slack,
    /// Microsoft Teams webhook.
    Teams,
    /// `PagerDuty` alert.
    PagerDuty,
    /// Generic webhook.
    Webhook,
}

/// Payload for sending a notification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendNotificationPayload {
    /// Alert that triggered the notification.
    pub alert_id: AlertId,
    /// Tenant that owns the alert.
    pub tenant_id: TenantId,
    /// Channel to send through.
    pub channel: NotificationChannel,
    /// Channel-specific configuration (e.g., webhook URL, email address).
    pub channel_config: serde_json::Value,
    /// Alert severity.
    pub severity: Severity,
    /// Alert title.
    pub title: String,
    /// Alert description.
    pub description: String,
    /// Additional context data.
    pub context: NotificationContext,
}

/// Context information for notifications.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NotificationContext {
    /// Project name.
    pub project_name: Option<String>,
    /// Package name (if applicable).
    pub package_name: Option<String>,
    /// Package version (if applicable).
    pub package_version: Option<String>,
    /// Link to the alert in the dashboard.
    pub dashboard_url: Option<String>,
    /// Recommended remediation.
    pub remediation: Option<String>,
}

impl SendNotificationPayload {
    /// Creates a new notification payload.
    #[must_use]
    pub fn new(
        alert_id: AlertId,
        tenant_id: TenantId,
        channel: NotificationChannel,
        severity: Severity,
        title: String,
        description: String,
    ) -> Self {
        Self {
            alert_id,
            tenant_id,
            channel,
            channel_config: serde_json::Value::Null,
            severity,
            title,
            description,
            context: NotificationContext::default(),
        }
    }

    /// Sets the channel configuration.
    #[must_use]
    pub fn with_config(mut self, config: serde_json::Value) -> Self {
        self.channel_config = config;
        self
    }

    /// Sets the notification context.
    #[must_use]
    pub fn with_context(mut self, context: NotificationContext) -> Self {
        self.context = context;
        self
    }
}

/// Result of sending a notification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendNotificationResult {
    /// Whether the notification was sent successfully.
    pub sent: bool,
    /// Channel-specific response (e.g., message ID).
    pub response: Option<serde_json::Value>,
    /// Time taken to send in milliseconds.
    pub send_duration_ms: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn test_scan_project_payload_builder() {
        let project_id = ProjectId(Uuid::new_v4());
        let tenant_id = TenantId(Uuid::new_v4());

        let payload = ScanProjectPayload::new(project_id, tenant_id)
            .with_ecosystems(vec![PackageEcosystem::Npm, PackageEcosystem::PyPi])
            .full_scan();

        assert_eq!(payload.project_id, project_id);
        assert_eq!(payload.ecosystems.len(), 2);
        assert!(payload.full_scan);
    }

    #[test]
    fn test_monitor_registry_payload_builder() {
        let payload = MonitorRegistryPayload::new(PackageEcosystem::Npm)
            .with_packages(vec!["lodash".to_string(), "express".to_string()])
            .check_only(true, false, true);

        assert_eq!(payload.ecosystem, PackageEcosystem::Npm);
        assert_eq!(payload.packages.len(), 2);
        assert!(payload.check_new_versions);
        assert!(!payload.check_removals);
        assert!(payload.check_maintainer_changes);
    }

    #[test]
    fn test_notification_payload_serialization() {
        let payload = SendNotificationPayload::new(
            AlertId(Uuid::new_v4()),
            TenantId(Uuid::new_v4()),
            NotificationChannel::Slack,
            Severity::High,
            "Security Alert".to_string(),
            "Potential typosquatting detected".to_string(),
        )
        .with_context(NotificationContext {
            project_name: Some("my-project".to_string()),
            package_name: Some("lodash-utils".to_string()),
            ..Default::default()
        });

        let json = serde_json::to_string(&payload).unwrap();
        let parsed: SendNotificationPayload = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.channel, NotificationChannel::Slack);
        assert_eq!(parsed.context.project_name, Some("my-project".to_string()));
    }
}
