//! Alert domain model for security findings and threats.

use chrono::{DateTime, Utc};
use semver::Version;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[cfg(feature = "graphql")]
use async_graphql::Enum;

use super::{DependencyId, HashAlgorithm, PackageEcosystem, ProjectId, Severity, TenantId};

/// Unique identifier for an alert.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AlertId(pub Uuid);

impl AlertId {
    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for AlertId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for AlertId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A security alert representing a detected threat or policy violation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    pub id: AlertId,
    pub tenant_id: TenantId,
    pub project_id: ProjectId,
    pub dependency_id: Option<DependencyId>,
    pub alert_type: AlertType,
    pub severity: Severity,
    pub title: String,
    pub description: String,
    pub status: AlertStatus,
    pub remediation: Option<Remediation>,
    pub metadata: AlertMetadata,
    pub created_at: DateTime<Utc>,
    pub acknowledged_at: Option<DateTime<Utc>>,
    pub acknowledged_by: Option<Uuid>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub resolved_by: Option<Uuid>,
}

impl Alert {
    /// Creates a new alert.
    #[must_use]
    // alert_type is cloned into self.alert_type and also used for default_severity();
    // changing to &AlertType would break callers in other crates (sctv-detectors, sctv-ci).
    #[allow(clippy::needless_pass_by_value)]
    pub fn new(
        tenant_id: TenantId,
        project_id: ProjectId,
        alert_type: AlertType,
        title: String,
        description: String,
    ) -> Self {
        Self {
            id: AlertId::new(),
            tenant_id,
            project_id,
            dependency_id: None,
            alert_type: alert_type.clone(),
            severity: alert_type.default_severity(),
            title,
            description,
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

    /// Acknowledges the alert.
    pub fn acknowledge(&mut self, user_id: Uuid) {
        self.status = AlertStatus::Acknowledged;
        self.acknowledged_at = Some(Utc::now());
        self.acknowledged_by = Some(user_id);
    }

    /// Marks the alert as being investigated.
    pub const fn start_investigation(&mut self) {
        self.status = AlertStatus::Investigating;
    }

    /// Marks the alert as a false positive.
    pub fn mark_false_positive(&mut self, user_id: Uuid, reason: String) {
        self.status = AlertStatus::FalsePositive;
        self.resolved_at = Some(Utc::now());
        self.resolved_by = Some(user_id);
        self.metadata.false_positive_reason = Some(reason);
    }

    /// Resolves the alert.
    pub fn resolve(&mut self, user_id: Uuid, resolution: Remediation) {
        self.status = AlertStatus::Resolved;
        self.resolved_at = Some(Utc::now());
        self.resolved_by = Some(user_id);
        self.remediation = Some(resolution);
    }

    /// Suppresses the alert.
    pub const fn suppress(&mut self, until: Option<DateTime<Utc>>) {
        self.status = AlertStatus::Suppressed;
        self.metadata.suppressed_until = until;
    }

    /// Checks if the alert is open (not resolved or suppressed).
    #[must_use]
    pub const fn is_open(&self) -> bool {
        matches!(
            self.status,
            AlertStatus::Open | AlertStatus::Acknowledged | AlertStatus::Investigating
        )
    }
}

/// Types of security alerts.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AlertType {
    /// Package hash does not match expected value.
    DependencyTampering(TamperingDetails),

    /// Package version was downgraded unexpectedly.
    DowngradeAttack(DowngradeDetails),

    /// Package name is similar to a popular package.
    Typosquatting(TyposquattingDetails),

    /// Build provenance verification failed.
    ProvenanceFailure(ProvenanceFailureDetails),

    /// Policy rule was violated.
    PolicyViolation(PolicyViolationDetails),

    /// Package was recently published (potentially malicious).
    NewPackage(NewPackageDetails),

    /// Maintainer account may be compromised.
    SuspiciousMaintainer(MaintainerDetails),
}

impl AlertType {
    /// Returns the default severity for this alert type.
    #[must_use]
    pub const fn default_severity(&self) -> Severity {
        match self {
            Self::DependencyTampering(_) | Self::Typosquatting(_) => Severity::Critical,
            Self::DowngradeAttack(_) | Self::SuspiciousMaintainer(_) => Severity::High,
            Self::ProvenanceFailure(_) | Self::NewPackage(_) => Severity::Medium,
            Self::PolicyViolation(details) => details.rule_severity,
        }
    }

    /// Returns the type name as a string.
    #[must_use]
    pub const fn type_name(&self) -> &'static str {
        match self {
            Self::DependencyTampering(_) => "dependency_tampering",
            Self::DowngradeAttack(_) => "downgrade_attack",
            Self::Typosquatting(_) => "typosquatting",
            Self::ProvenanceFailure(_) => "provenance_failure",
            Self::PolicyViolation(_) => "policy_violation",
            Self::NewPackage(_) => "new_package",
            Self::SuspiciousMaintainer(_) => "suspicious_maintainer",
        }
    }
}

/// Details for tampering alerts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TamperingDetails {
    pub package_name: String,
    pub ecosystem: PackageEcosystem,
    pub version: String,
    pub expected_hash: String,
    pub actual_hash: String,
    pub algorithm: HashAlgorithm,
    pub registry_source: String,
}

/// Details for downgrade attack alerts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DowngradeDetails {
    pub package_name: String,
    pub ecosystem: PackageEcosystem,
    pub previous_version: Version,
    pub current_version: Version,
    pub lock_file_version: Option<Version>,
}

/// Details for typosquatting alerts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TyposquattingDetails {
    pub suspicious_package: String,
    pub ecosystem: PackageEcosystem,
    pub similar_popular_package: String,
    pub similarity_score: f64,
    pub detection_method: TyposquattingMethod,
    pub popular_package_downloads: Option<u64>,
}

/// Methods for detecting typosquatting.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TyposquattingMethod {
    Levenshtein,
    DamerauLevenshtein,
    JaroWinkler,
    Phonetic,
    KeyboardDistance,
    Combosquatting,
}

/// Details for provenance failure alerts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvenanceFailureDetails {
    pub package_name: String,
    pub ecosystem: PackageEcosystem,
    pub version: String,
    pub expected_slsa_level: u8,
    pub actual_slsa_level: Option<u8>,
    pub attestation_errors: Vec<String>,
}

/// Details for policy violation alerts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyViolationDetails {
    pub policy_name: String,
    pub rule_type: String,
    pub rule_severity: Severity,
    pub violation_details: String,
}

/// Details for new package alerts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewPackageDetails {
    pub package_name: String,
    pub ecosystem: PackageEcosystem,
    pub version: String,
    pub published_at: DateTime<Utc>,
    pub age_days: u32,
    pub threshold_days: u32,
}

/// Details for suspicious maintainer alerts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaintainerDetails {
    pub package_name: String,
    pub ecosystem: PackageEcosystem,
    pub maintainer_name: String,
    pub reason: String,
}

/// Current status of an alert.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "graphql", derive(Enum))]
#[serde(rename_all = "lowercase")]
pub enum AlertStatus {
    Open,
    Acknowledged,
    Investigating,
    FalsePositive,
    Resolved,
    Suppressed,
}

/// Remediation information for resolved alerts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Remediation {
    pub action_taken: String,
    pub new_version: Option<Version>,
    pub notes: Option<String>,
}

/// Additional metadata for alerts.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AlertMetadata {
    pub scan_id: Option<Uuid>,
    pub first_detected_at: Option<DateTime<Utc>>,
    pub occurrence_count: u32,
    pub false_positive_reason: Option<String>,
    pub suppressed_until: Option<DateTime<Utc>>,
    pub related_alerts: Vec<AlertId>,
    pub external_references: Vec<String>,
}
