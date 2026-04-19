//! Downgrade attack detection engine.
//!
//! Detects unexpected version downgrades that may indicate supply chain attacks
//! where an attacker replaces a newer, patched version with an older, vulnerable one.
//!
//! # Features
//!
//! - Version history tracking across scans
//! - Semantic versioning analysis
//! - Lock file comparison
//! - Registry version timeline verification
//! - Configurable downgrade policies

mod history;
mod policy;

use async_trait::async_trait;
use sctv_core::{Alert, AlertType, Dependency, DowngradeDetails, Severity};
use semver::Version;
use serde::{Deserialize, Serialize};

use crate::{DetectionResult, Detector, DetectorResult};

pub use history::*;
pub use policy::*;

/// Configuration for the downgrade detector.
#[derive(Debug, Clone)]
pub struct DowngradeConfig {
    /// Whether to allow any patch-level downgrades (e.g., 1.0.2 -> 1.0.1).
    pub allow_patch_downgrades: bool,
    /// Whether to allow minor version downgrades (e.g., 1.2.0 -> 1.1.0).
    pub allow_minor_downgrades: bool,
    /// Whether to check lock file version against resolved version.
    pub check_lock_file: bool,
    /// Minimum severity for downgrades.
    pub minimum_severity: DowngradeSeverity,
    /// Packages to exclude from downgrade detection.
    pub excluded_packages: Vec<String>,
    /// Maximum age (in days) for a downgrade to be considered suspicious.
    pub suspicious_downgrade_age_days: u32,
}

impl Default for DowngradeConfig {
    fn default() -> Self {
        Self {
            allow_patch_downgrades: false,
            allow_minor_downgrades: false,
            check_lock_file: true,
            minimum_severity: DowngradeSeverity::Minor,
            excluded_packages: Vec::new(),
            suspicious_downgrade_age_days: 30,
        }
    }
}

/// Severity classification for downgrades.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum DowngradeSeverity {
    /// Pre-release version changes.
    Prerelease,
    /// Patch version downgrade (e.g., 1.0.2 -> 1.0.1).
    Patch,
    /// Minor version downgrade (e.g., 1.2.0 -> 1.1.0).
    Minor,
    /// Major version downgrade (e.g., 2.0.0 -> 1.0.0).
    Major,
}

impl DowngradeSeverity {
    /// Converts to alert severity.
    #[must_use]
    pub const fn to_alert_severity(self) -> Severity {
        match self {
            Self::Prerelease => Severity::Low,
            Self::Patch => Severity::Medium,
            Self::Minor => Severity::High,
            Self::Major => Severity::Critical,
        }
    }
}

/// Result of a downgrade analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DowngradeAnalysisResult {
    /// Whether a downgrade was detected.
    pub is_downgrade: bool,
    /// The previous version (if known).
    pub previous_version: Option<String>,
    /// The current version.
    pub current_version: String,
    /// Version from the lock file (if different).
    pub lock_file_version: Option<String>,
    /// Severity of the downgrade.
    pub severity: Option<DowngradeSeverity>,
    /// Whether the downgrade is considered suspicious.
    pub is_suspicious: bool,
    /// Reason for the downgrade being suspicious.
    pub suspicious_reason: Option<String>,
    /// Days since the downgrade occurred.
    pub days_since_downgrade: Option<u32>,
}

impl DowngradeAnalysisResult {
    /// Creates a result indicating no downgrade.
    #[must_use]
    pub fn no_downgrade(current_version: &Version) -> Self {
        Self {
            is_downgrade: false,
            previous_version: None,
            current_version: current_version.to_string(),
            lock_file_version: None,
            severity: None,
            is_suspicious: false,
            suspicious_reason: None,
            days_since_downgrade: None,
        }
    }

    /// Creates a result indicating a downgrade was detected.
    #[must_use]
    pub fn downgrade_detected(
        previous: &Version,
        current: &Version,
        severity: DowngradeSeverity,
    ) -> Self {
        Self {
            is_downgrade: true,
            previous_version: Some(previous.to_string()),
            current_version: current.to_string(),
            lock_file_version: None,
            severity: Some(severity),
            is_suspicious: false,
            suspicious_reason: None,
            days_since_downgrade: None,
        }
    }
}

/// Downgrade attack detector.
pub struct DowngradeDetector {
    config: DowngradeConfig,
    history_store: VersionHistoryStore,
}

impl DowngradeDetector {
    /// Creates a new detector with default configuration.
    #[must_use]
    pub fn new() -> Self {
        Self {
            config: DowngradeConfig::default(),
            history_store: VersionHistoryStore::new(),
        }
    }

    /// Creates a detector with custom configuration.
    #[must_use]
    pub fn with_config(config: DowngradeConfig) -> Self {
        Self {
            config,
            history_store: VersionHistoryStore::new(),
        }
    }

    /// Creates a detector with custom configuration and history store.
    #[must_use]
    pub const fn with_config_and_store(
        config: DowngradeConfig,
        store: VersionHistoryStore,
    ) -> Self {
        Self {
            config,
            history_store: store,
        }
    }

    /// Analyzes a dependency for potential downgrade attacks.
    pub fn analyze_downgrade(&self, dependency: &Dependency) -> DowngradeAnalysisResult {
        // Check if package is excluded
        if self.is_excluded(&dependency.package_name) {
            return DowngradeAnalysisResult::no_downgrade(&dependency.resolved_version);
        }

        // Get version history for this dependency
        let key = VersionHistoryKey {
            project_id: dependency.project_id,
            ecosystem: dependency.ecosystem,
            package_name: dependency.package_name.clone(),
        };

        let previous_version = self.history_store.get_latest_version(&key);

        // Compare versions
        let result = previous_version.map_or_else(
            || DowngradeAnalysisResult::no_downgrade(&dependency.resolved_version),
            |prev| self.compare_versions(&prev, &dependency.resolved_version),
        );

        // Update history with current version
        self.history_store
            .record_version(&key, &dependency.resolved_version);

        // Enhance result with additional analysis
        Self::enhance_result(result, dependency)
    }

    /// Compares two versions and determines if there's a downgrade.
    fn compare_versions(&self, previous: &Version, current: &Version) -> DowngradeAnalysisResult {
        // Current version is same or newer - no downgrade
        if current >= previous {
            return DowngradeAnalysisResult::no_downgrade(current);
        }

        // Determine downgrade severity
        let severity = Self::determine_severity(previous, current);

        // Check if this severity level should be reported
        if severity < self.config.minimum_severity {
            return DowngradeAnalysisResult::no_downgrade(current);
        }

        // Check policy allowances
        if self.is_allowed_downgrade(previous, current, severity) {
            return DowngradeAnalysisResult::no_downgrade(current);
        }

        DowngradeAnalysisResult::downgrade_detected(previous, current, severity)
    }

    /// Determines the severity of a version downgrade.
    const fn determine_severity(previous: &Version, current: &Version) -> DowngradeSeverity {
        if previous.major != current.major {
            DowngradeSeverity::Major
        } else if previous.minor != current.minor {
            DowngradeSeverity::Minor
        } else if previous.patch != current.patch {
            DowngradeSeverity::Patch
        } else {
            // Only pre-release differs
            DowngradeSeverity::Prerelease
        }
    }

    /// Checks if a downgrade is allowed by configuration.
    const fn is_allowed_downgrade(
        &self,
        _previous: &Version,
        _current: &Version,
        severity: DowngradeSeverity,
    ) -> bool {
        match severity {
            DowngradeSeverity::Patch => self.config.allow_patch_downgrades,
            DowngradeSeverity::Minor => self.config.allow_minor_downgrades,
            DowngradeSeverity::Major => false, // Never allow major downgrades
            DowngradeSeverity::Prerelease => true, // Pre-release changes are usually ok
        }
    }

    /// Checks if a package is in the exclusion list.
    fn is_excluded(&self, package_name: &str) -> bool {
        self.config
            .excluded_packages
            .iter()
            .any(|excluded| excluded == package_name || package_name.starts_with(excluded))
    }

    /// Enhances the analysis result with additional checks.
    fn enhance_result(
        mut result: DowngradeAnalysisResult,
        dependency: &Dependency,
    ) -> DowngradeAnalysisResult {
        if !result.is_downgrade {
            return result;
        }

        // Check for suspicious patterns
        if let Some(prev_str) = &result.previous_version {
            if let Ok(prev) = Version::parse(prev_str) {
                // Suspicious: Major or minor downgrade with significant version gap
                let version_gap = Self::calculate_version_gap(&prev, &dependency.resolved_version);
                if version_gap > 5 {
                    result.is_suspicious = true;
                    result.suspicious_reason =
                        Some(format!("Large version gap of {version_gap} versions"));
                }

                // Suspicious: Downgrade crosses a major security release boundary
                // (This would require CVE data in a real implementation)
            }
        }

        result
    }

    /// Calculates the "gap" between two versions (rough estimate).
    const fn calculate_version_gap(previous: &Version, current: &Version) -> u64 {
        let prev_score = (previous.major * 10000) + (previous.minor * 100) + previous.patch;
        let curr_score = (current.major * 10000) + (current.minor * 100) + current.patch;
        prev_score.saturating_sub(curr_score)
    }

    /// Gets a reference to the history store.
    #[must_use]
    pub const fn history_store(&self) -> &VersionHistoryStore {
        &self.history_store
    }
}

impl Default for DowngradeDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Detector for DowngradeDetector {
    fn detector_type(&self) -> &'static str {
        "downgrade"
    }

    async fn analyze(&self, dependency: &Dependency) -> DetectorResult<Vec<DetectionResult>> {
        let analysis = self.analyze_downgrade(dependency);
        let mut results = Vec::new();

        if analysis.is_downgrade {
            let confidence: f64 = match analysis.severity {
                Some(DowngradeSeverity::Major) => 0.95,
                Some(DowngradeSeverity::Minor) => 0.85,
                Some(DowngradeSeverity::Patch) => 0.7,
                Some(DowngradeSeverity::Prerelease) => 0.5,
                None => 0.6,
            };

            // Increase confidence if suspicious
            let final_confidence = if analysis.is_suspicious {
                (confidence + 0.1).min(1.0)
            } else {
                confidence
            };

            results.push(DetectionResult::detected(
                final_confidence,
                "version_downgrade",
                serde_json::to_value(&analysis).unwrap_or_default(),
            ));
        }

        if results.is_empty() {
            results.push(DetectionResult::not_detected());
        }

        Ok(results)
    }

    fn create_alerts(&self, dependency: &Dependency, results: &[DetectionResult]) -> Vec<Alert> {
        results
            .iter()
            .filter(|r| r.detected)
            .filter_map(|result| {
                let analysis: DowngradeAnalysisResult =
                    serde_json::from_value(result.details.clone()).ok()?;

                if !analysis.is_downgrade {
                    return None;
                }

                let previous_version = analysis.previous_version.as_ref()?;
                let previous = Version::parse(previous_version).ok()?;

                let details = DowngradeDetails {
                    package_name: dependency.package_name.clone(),
                    ecosystem: dependency.ecosystem,
                    previous_version: previous.clone(),
                    current_version: dependency.resolved_version.clone(),
                    lock_file_version: analysis
                        .lock_file_version
                        .and_then(|v| Version::parse(&v).ok()),
                };

                let severity_text = match analysis.severity {
                    Some(DowngradeSeverity::Major) => "major",
                    Some(DowngradeSeverity::Minor) => "minor",
                    Some(DowngradeSeverity::Patch) => "patch",
                    Some(DowngradeSeverity::Prerelease) => "pre-release",
                    None => "unknown",
                };

                let mut alert = Alert::new(
                    dependency.tenant_id,
                    dependency.project_id,
                    AlertType::DowngradeAttack(details),
                    format!(
                        "Version downgrade detected: {}@{} -> {}",
                        dependency.package_name, previous, dependency.resolved_version
                    ),
                    format!(
                        "Package '{}' was downgraded from version {} to {}. This is a {} \
                         version downgrade which may indicate a supply chain attack where \
                         an attacker is attempting to replace a patched version with an \
                         older, potentially vulnerable version.{}",
                        dependency.package_name,
                        previous,
                        dependency.resolved_version,
                        severity_text,
                        if analysis.is_suspicious {
                            format!(
                                " This downgrade is considered suspicious: {}",
                                analysis.suspicious_reason.unwrap_or_default()
                            )
                        } else {
                            String::new()
                        }
                    ),
                );

                alert.dependency_id = Some(dependency.id);
                alert.severity = analysis
                    .severity
                    .map_or(Severity::Medium, DowngradeSeverity::to_alert_severity);

                Some(alert)
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sctv_core::{PackageEcosystem, ProjectId, TenantId};

    fn create_test_dependency(version: &str) -> Dependency {
        Dependency::new(
            ProjectId::new(),
            TenantId::new(),
            "test-package".to_string(),
            PackageEcosystem::Npm,
            "^1.0.0".to_string(),
            Version::parse(version).unwrap(),
        )
    }

    #[test]
    fn test_no_downgrade() {
        let detector = DowngradeDetector::new();

        let dep1 = create_test_dependency("1.0.0");
        let result1 = detector.analyze_downgrade(&dep1);
        assert!(!result1.is_downgrade);

        // Same version should not be a downgrade
        let _dep2 = create_test_dependency("1.0.0");
        let dep2_same_key = Dependency::new(
            dep1.project_id,
            dep1.tenant_id,
            "test-package".to_string(),
            PackageEcosystem::Npm,
            "^1.0.0".to_string(),
            Version::parse("1.0.0").unwrap(),
        );
        let result2 = detector.analyze_downgrade(&dep2_same_key);
        assert!(!result2.is_downgrade);
    }

    #[test]
    fn test_upgrade_not_flagged() {
        let detector = DowngradeDetector::new();
        let project_id = ProjectId::new();
        let tenant_id = TenantId::new();

        // First scan with 1.0.0
        let dep1 = Dependency::new(
            project_id,
            tenant_id,
            "test-package".to_string(),
            PackageEcosystem::Npm,
            "^1.0.0".to_string(),
            Version::new(1, 0, 0),
        );
        detector.analyze_downgrade(&dep1);

        // Upgrade to 1.0.1 should not be flagged
        let dep2 = Dependency::new(
            project_id,
            tenant_id,
            "test-package".to_string(),
            PackageEcosystem::Npm,
            "^1.0.0".to_string(),
            Version::new(1, 0, 1),
        );
        let result = detector.analyze_downgrade(&dep2);
        assert!(!result.is_downgrade);
    }

    #[test]
    fn test_patch_downgrade_detected() {
        // Use config that detects patch downgrades
        let config = DowngradeConfig {
            minimum_severity: DowngradeSeverity::Patch,
            ..DowngradeConfig::default()
        };

        let detector = DowngradeDetector::with_config(config);
        let project_id = ProjectId::new();
        let tenant_id = TenantId::new();

        // First scan with 1.0.2
        let dep1 = Dependency::new(
            project_id,
            tenant_id,
            "test-package".to_string(),
            PackageEcosystem::Npm,
            "^1.0.0".to_string(),
            Version::new(1, 0, 2),
        );
        detector.analyze_downgrade(&dep1);

        // Downgrade to 1.0.1
        let dep2 = Dependency::new(
            project_id,
            tenant_id,
            "test-package".to_string(),
            PackageEcosystem::Npm,
            "^1.0.0".to_string(),
            Version::new(1, 0, 1),
        );
        let result = detector.analyze_downgrade(&dep2);
        assert!(result.is_downgrade);
        assert_eq!(result.severity, Some(DowngradeSeverity::Patch));
    }

    #[test]
    fn test_minor_downgrade_detected() {
        let detector = DowngradeDetector::new();
        let project_id = ProjectId::new();
        let tenant_id = TenantId::new();

        // First scan with 1.2.0
        let dep1 = Dependency::new(
            project_id,
            tenant_id,
            "test-package".to_string(),
            PackageEcosystem::Npm,
            "^1.0.0".to_string(),
            Version::new(1, 2, 0),
        );
        detector.analyze_downgrade(&dep1);

        // Downgrade to 1.1.0
        let dep2 = Dependency::new(
            project_id,
            tenant_id,
            "test-package".to_string(),
            PackageEcosystem::Npm,
            "^1.0.0".to_string(),
            Version::new(1, 1, 0),
        );
        let result = detector.analyze_downgrade(&dep2);
        assert!(result.is_downgrade);
        assert_eq!(result.severity, Some(DowngradeSeverity::Minor));
    }

    #[test]
    fn test_major_downgrade_detected() {
        let detector = DowngradeDetector::new();
        let project_id = ProjectId::new();
        let tenant_id = TenantId::new();

        // First scan with 2.0.0
        let dep1 = Dependency::new(
            project_id,
            tenant_id,
            "test-package".to_string(),
            PackageEcosystem::Npm,
            "^1.0.0".to_string(),
            Version::new(2, 0, 0),
        );
        detector.analyze_downgrade(&dep1);

        // Downgrade to 1.0.0
        let dep2 = Dependency::new(
            project_id,
            tenant_id,
            "test-package".to_string(),
            PackageEcosystem::Npm,
            "^1.0.0".to_string(),
            Version::new(1, 0, 0),
        );
        let result = detector.analyze_downgrade(&dep2);
        assert!(result.is_downgrade);
        assert_eq!(result.severity, Some(DowngradeSeverity::Major));
    }

    #[test]
    fn test_excluded_package() {
        let mut config = DowngradeConfig::default();
        config.excluded_packages.push("excluded-pkg".to_string());

        let detector = DowngradeDetector::with_config(config);
        let project_id = ProjectId::new();
        let tenant_id = TenantId::new();

        // First scan with 2.0.0
        let dep1 = Dependency::new(
            project_id,
            tenant_id,
            "excluded-pkg".to_string(),
            PackageEcosystem::Npm,
            "^1.0.0".to_string(),
            Version::new(2, 0, 0),
        );
        detector.analyze_downgrade(&dep1);

        // Downgrade should not be detected for excluded package
        let dep2 = Dependency::new(
            project_id,
            tenant_id,
            "excluded-pkg".to_string(),
            PackageEcosystem::Npm,
            "^1.0.0".to_string(),
            Version::new(1, 0, 0),
        );
        let result = detector.analyze_downgrade(&dep2);
        assert!(!result.is_downgrade);
    }

    #[test]
    fn test_allow_patch_downgrades() {
        let config = DowngradeConfig {
            allow_patch_downgrades: true,
            ..DowngradeConfig::default()
        };

        let detector = DowngradeDetector::with_config(config);
        let project_id = ProjectId::new();
        let tenant_id = TenantId::new();

        // First scan with 1.0.2
        let dep1 = Dependency::new(
            project_id,
            tenant_id,
            "test-package".to_string(),
            PackageEcosystem::Npm,
            "^1.0.0".to_string(),
            Version::new(1, 0, 2),
        );
        detector.analyze_downgrade(&dep1);

        // Patch downgrade should be allowed
        let dep2 = Dependency::new(
            project_id,
            tenant_id,
            "test-package".to_string(),
            PackageEcosystem::Npm,
            "^1.0.0".to_string(),
            Version::new(1, 0, 1),
        );
        let result = detector.analyze_downgrade(&dep2);
        assert!(!result.is_downgrade);
    }

    #[test]
    fn test_severity_conversion() {
        assert_eq!(
            DowngradeSeverity::Major.to_alert_severity(),
            Severity::Critical
        );
        assert_eq!(DowngradeSeverity::Minor.to_alert_severity(), Severity::High);
        assert_eq!(
            DowngradeSeverity::Patch.to_alert_severity(),
            Severity::Medium
        );
        assert_eq!(
            DowngradeSeverity::Prerelease.to_alert_severity(),
            Severity::Low
        );
    }
}
