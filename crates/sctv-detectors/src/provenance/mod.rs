//! Build provenance verification engine.
//!
//! Verifies SLSA provenance attestations and Sigstore signatures to ensure
//! packages have trustworthy build provenance.
//!
//! # Features
//!
//! - SLSA provenance attestation parsing and validation
//! - Sigstore bundle verification
//! - Rekor transparency log verification
//! - Builder identity validation
//! - Source repository verification

mod attestation;
mod sigstore;
mod verification;

use async_trait::async_trait;
use sctv_core::{
    Alert, AlertType, Dependency, PackageEcosystem, ProvenanceFailureDetails, ProvenanceStatus,
    Severity,
};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

use crate::{DetectionResult, Detector, DetectorResult};

pub use attestation::*;
pub use sigstore::*;
pub use verification::*;

/// Configuration for the provenance detector.
#[derive(Debug, Clone)]
pub struct ProvenanceConfig {
    /// Minimum required SLSA level for packages.
    pub minimum_slsa_level: u8,
    /// Whether to require Sigstore signatures.
    pub require_sigstore: bool,
    /// Whether to verify Rekor transparency log entries.
    pub verify_rekor: bool,
    /// Trusted builder IDs for SLSA provenance.
    pub trusted_builders: HashSet<String>,
    /// Whether to allow packages without provenance (will generate warnings).
    pub allow_missing_provenance: bool,
    /// Ecosystems that support provenance verification.
    pub supported_ecosystems: HashSet<PackageEcosystem>,
}

impl Default for ProvenanceConfig {
    fn default() -> Self {
        let mut trusted_builders = HashSet::new();
        // GitHub Actions builders
        trusted_builders
            .insert("https://github.com/slsa-framework/slsa-github-generator".to_string());
        trusted_builders.insert("https://github.com/actions/runner".to_string());
        // npm provenance builders
        trusted_builders.insert("https://github.com/npm/cli".to_string());
        // PyPI trusted publishers
        trusted_builders.insert("https://github.com/pypa/gh-action-pypi-publish".to_string());

        let mut supported_ecosystems = HashSet::new();
        supported_ecosystems.insert(PackageEcosystem::Npm);
        supported_ecosystems.insert(PackageEcosystem::PyPi);
        supported_ecosystems.insert(PackageEcosystem::Cargo);

        Self {
            minimum_slsa_level: 1,
            require_sigstore: false,
            verify_rekor: true,
            trusted_builders,
            allow_missing_provenance: true,
            supported_ecosystems,
        }
    }
}

/// Result of provenance verification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvenanceVerificationResult {
    /// Whether provenance was found.
    pub has_provenance: bool,
    /// The verified SLSA level.
    pub slsa_level: Option<u8>,
    /// The builder ID from the attestation.
    pub builder_id: Option<String>,
    /// Whether the builder is trusted.
    pub builder_trusted: bool,
    /// Source repository URI.
    pub source_uri: Option<String>,
    /// Source commit digest.
    pub source_digest: Option<String>,
    /// Sigstore verification status.
    pub sigstore_verified: bool,
    /// Rekor log entry details.
    pub rekor_entry: Option<RekorEntryInfo>,
    /// List of verification errors.
    pub errors: Vec<String>,
    /// List of verification warnings.
    pub warnings: Vec<String>,
}

impl ProvenanceVerificationResult {
    /// Creates a result indicating no provenance was found.
    #[must_use]
    pub fn no_provenance() -> Self {
        Self {
            has_provenance: false,
            slsa_level: None,
            builder_id: None,
            builder_trusted: false,
            source_uri: None,
            source_digest: None,
            sigstore_verified: false,
            rekor_entry: None,
            errors: vec!["No provenance attestation found".to_string()],
            warnings: Vec::new(),
        }
    }

    /// Creates a result with verification failure.
    #[must_use]
    pub const fn verification_failed(errors: Vec<String>) -> Self {
        Self {
            has_provenance: true,
            slsa_level: Some(0),
            builder_id: None,
            builder_trusted: false,
            source_uri: None,
            source_digest: None,
            sigstore_verified: false,
            rekor_entry: None,
            errors,
            warnings: Vec::new(),
        }
    }

    /// Checks if the verification passed all requirements.
    #[must_use]
    pub fn is_valid(&self, config: &ProvenanceConfig) -> bool {
        if !self.has_provenance {
            return config.allow_missing_provenance;
        }

        let level_ok = self
            .slsa_level
            .is_some_and(|l| l >= config.minimum_slsa_level);
        let sigstore_ok = !config.require_sigstore || self.sigstore_verified;
        let builder_ok = self.builder_trusted || config.trusted_builders.is_empty();

        level_ok && sigstore_ok && builder_ok && self.errors.is_empty()
    }
}

/// Information about a Rekor transparency log entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RekorEntryInfo {
    pub log_index: u64,
    pub log_id: String,
    pub integrated_time: i64,
    pub inclusion_verified: bool,
}

/// SLSA/Sigstore provenance verifier.
pub struct ProvenanceDetector {
    config: ProvenanceConfig,
    verifier: ProvenanceVerifier,
}

impl ProvenanceDetector {
    /// Creates a new detector with default configuration.
    #[must_use]
    pub fn new() -> Self {
        Self {
            config: ProvenanceConfig::default(),
            verifier: ProvenanceVerifier::new(),
        }
    }

    /// Creates a detector with custom configuration.
    #[must_use]
    pub fn with_config(config: ProvenanceConfig) -> Self {
        Self {
            verifier: ProvenanceVerifier::with_config(config.clone()),
            config,
        }
    }

    /// Verifies provenance for a dependency.
    pub async fn verify_provenance(
        &self,
        dependency: &Dependency,
    ) -> DetectorResult<ProvenanceVerificationResult> {
        // Check if ecosystem supports provenance
        if !self
            .config
            .supported_ecosystems
            .contains(&dependency.ecosystem)
        {
            return Ok(ProvenanceVerificationResult {
                has_provenance: false,
                slsa_level: None,
                builder_id: None,
                builder_trusted: false,
                source_uri: None,
                source_digest: None,
                sigstore_verified: false,
                rekor_entry: None,
                errors: Vec::new(),
                warnings: vec![format!(
                    "Provenance verification not supported for {:?} ecosystem",
                    dependency.ecosystem
                )],
            });
        }

        // Check existing provenance status on the dependency
        let provenance_status = &dependency.integrity.provenance_status;
        let provenance_details = &dependency.integrity.provenance_details;

        match provenance_status {
            ProvenanceStatus::Unknown => {
                // Need to fetch and verify provenance
                self.verifier.verify(dependency).await
            }
            ProvenanceStatus::Failed => {
                Ok(ProvenanceVerificationResult::verification_failed(vec![
                    "Previous provenance verification failed".to_string(),
                ]))
            }
            _ => {
                // Already have provenance status, build result from it
                let level = provenance_status.level();
                let details = provenance_details.as_ref();

                Ok(ProvenanceVerificationResult {
                    has_provenance: level.is_some(),
                    slsa_level: level,
                    builder_id: details.and_then(|d| d.builder_id.clone()),
                    builder_trusted: details
                        .and_then(|d| d.builder_id.as_ref())
                        .is_some_and(|id| self.config.trusted_builders.contains(id)),
                    source_uri: details.and_then(|d| d.source_uri.clone()),
                    source_digest: details.and_then(|d| d.source_digest.clone()),
                    sigstore_verified: dependency.integrity.signature_status
                        == sctv_core::SignatureStatus::Verified,
                    rekor_entry: None,
                    errors: Vec::new(),
                    warnings: Vec::new(),
                })
            }
        }
    }

    /// Determines the SLSA level based on attestation contents.
    #[allow(dead_code)]
    fn determine_slsa_level(&self, result: &ProvenanceVerificationResult) -> u8 {
        // SLSA Level requirements:
        // Level 1: Documentation of build process
        // Level 2: Hosted build platform with authenticated provenance
        // Level 3: Hardened builds with non-falsifiable provenance

        if !result.has_provenance {
            return 0;
        }

        let mut level = 1; // Has provenance = at least level 1

        // Level 2: Hosted build with authenticated provenance
        if result.builder_trusted && result.sigstore_verified {
            level = 2;
        }

        // Level 3: Rekor transparency log with inclusion proof
        if level >= 2
            && result
                .rekor_entry
                .as_ref()
                .is_some_and(|e| e.inclusion_verified)
            && result.source_digest.is_some()
        {
            level = 3;
        }

        level
    }
}

impl Default for ProvenanceDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Detector for ProvenanceDetector {
    fn detector_type(&self) -> &'static str {
        "provenance"
    }

    async fn analyze(&self, dependency: &Dependency) -> DetectorResult<Vec<DetectionResult>> {
        let verification_result = self.verify_provenance(dependency).await?;
        let mut results = Vec::new();

        // Check if provenance meets requirements
        let is_valid = verification_result.is_valid(&self.config);

        if !is_valid {
            let slsa_level = verification_result.slsa_level.unwrap_or(0);
            let required_level = self.config.minimum_slsa_level;

            // Determine confidence based on what failed
            let confidence = if !verification_result.has_provenance {
                0.7 // Medium confidence - missing provenance
            } else if slsa_level < required_level {
                0.85 // High confidence - level doesn't meet requirements
            } else if !verification_result.errors.is_empty() {
                0.9 // Very high confidence - verification errors
            } else {
                0.6 // Lower confidence - warnings only
            };

            results.push(DetectionResult::detected(
                confidence,
                "slsa_provenance",
                serde_json::to_value(&verification_result).unwrap_or_default(),
            ));
        }

        // Also flag if builder is not trusted
        if verification_result.has_provenance
            && !verification_result.builder_trusted
            && !self.config.trusted_builders.is_empty()
        {
            results.push(DetectionResult::detected(
                0.6,
                "untrusted_builder",
                serde_json::json!({
                    "builder_id": verification_result.builder_id,
                    "trusted_builders": self.config.trusted_builders.iter().collect::<Vec<_>>(),
                }),
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
                if result.method == "slsa_provenance" {
                    let verification: ProvenanceVerificationResult =
                        serde_json::from_value(result.details.clone()).ok()?;

                    let details = ProvenanceFailureDetails {
                        package_name: dependency.package_name.clone(),
                        ecosystem: dependency.ecosystem,
                        version: dependency.resolved_version.to_string(),
                        expected_slsa_level: self.config.minimum_slsa_level,
                        actual_slsa_level: verification.slsa_level,
                        attestation_errors: verification.errors.clone(),
                    };

                    let title = if verification.has_provenance {
                        format!(
                            "SLSA provenance verification failed for {}@{}",
                            dependency.package_name, dependency.resolved_version
                        )
                    } else {
                        format!(
                            "Missing provenance for {}@{}",
                            dependency.package_name, dependency.resolved_version
                        )
                    };

                    let description = if verification.has_provenance {
                        format!(
                            "Package '{}' version {} has SLSA level {} but requires level {}. \
                             Errors: {}",
                            dependency.package_name,
                            dependency.resolved_version,
                            verification.slsa_level.unwrap_or(0),
                            self.config.minimum_slsa_level,
                            verification.errors.join(", ")
                        )
                    } else {
                        format!(
                            "Package '{}' version {} does not have SLSA provenance attestation. \
                             Required minimum level: {}. This means the build process cannot be \
                             verified and the package may not be from a trusted source.",
                            dependency.package_name,
                            dependency.resolved_version,
                            self.config.minimum_slsa_level
                        )
                    };

                    let mut alert = Alert::new(
                        dependency.tenant_id,
                        dependency.project_id,
                        AlertType::ProvenanceFailure(details),
                        title,
                        description,
                    );
                    alert.dependency_id = Some(dependency.id);

                    // Override severity based on missing vs insufficient
                    if !verification.has_provenance && self.config.allow_missing_provenance {
                        alert.severity = Severity::Medium;
                    }

                    Some(alert)
                } else if result.method == "untrusted_builder" {
                    let builder_id: Option<String> = result
                        .details
                        .get("builder_id")
                        .and_then(|v| v.as_str())
                        .map(String::from);

                    let details = ProvenanceFailureDetails {
                        package_name: dependency.package_name.clone(),
                        ecosystem: dependency.ecosystem,
                        version: dependency.resolved_version.to_string(),
                        expected_slsa_level: self.config.minimum_slsa_level,
                        actual_slsa_level: dependency.integrity.provenance_status.level(),
                        attestation_errors: vec![format!(
                            "Builder '{}' is not in the trusted builders list",
                            builder_id.as_deref().unwrap_or("unknown")
                        )],
                    };

                    let mut alert = Alert::new(
                        dependency.tenant_id,
                        dependency.project_id,
                        AlertType::ProvenanceFailure(details),
                        format!(
                            "Untrusted builder for {}@{}",
                            dependency.package_name, dependency.resolved_version
                        ),
                        format!(
                            "Package '{}' version {} was built by '{}' which is not in the \
                             list of trusted builders. This may indicate the package was built \
                             by an unauthorized build system.",
                            dependency.package_name,
                            dependency.resolved_version,
                            builder_id.as_deref().unwrap_or("unknown")
                        ),
                    );
                    alert.dependency_id = Some(dependency.id);
                    alert.severity = Severity::Medium;

                    Some(alert)
                } else {
                    None
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sctv_core::{ProjectId, TenantId};
    use semver::Version;

    fn create_test_dependency() -> Dependency {
        Dependency::new(
            ProjectId::new(),
            TenantId::new(),
            "test-package".to_string(),
            PackageEcosystem::Npm,
            "^1.0.0".to_string(),
            Version::new(1, 0, 0),
        )
    }

    #[test]
    fn test_no_provenance_result() {
        let result = ProvenanceVerificationResult::no_provenance();
        assert!(!result.has_provenance);
        assert!(result.slsa_level.is_none());
        assert!(!result.errors.is_empty());
    }

    #[test]
    fn test_config_defaults() {
        let config = ProvenanceConfig::default();
        assert_eq!(config.minimum_slsa_level, 1);
        assert!(!config.require_sigstore);
        assert!(config.verify_rekor);
        assert!(!config.trusted_builders.is_empty());
    }

    #[tokio::test]
    async fn test_unsupported_ecosystem() {
        let mut config = ProvenanceConfig::default();
        config.supported_ecosystems.clear();
        config.supported_ecosystems.insert(PackageEcosystem::Npm);

        let detector = ProvenanceDetector::with_config(config);
        let mut dependency = create_test_dependency();
        dependency.ecosystem = PackageEcosystem::Maven;

        let result = detector.verify_provenance(&dependency).await.unwrap();
        assert!(!result.has_provenance);
        assert!(!result.warnings.is_empty());
    }

    #[test]
    fn test_result_validity_check() {
        let config = ProvenanceConfig::default();

        // Valid result
        let valid_result = ProvenanceVerificationResult {
            has_provenance: true,
            slsa_level: Some(2),
            builder_id: Some("https://github.com/slsa-framework/slsa-github-generator".to_string()),
            builder_trusted: true,
            source_uri: Some("https://github.com/test/repo".to_string()),
            source_digest: Some("abc123".to_string()),
            sigstore_verified: false,
            rekor_entry: None,
            errors: Vec::new(),
            warnings: Vec::new(),
        };
        assert!(valid_result.is_valid(&config));

        // Invalid - level too low
        let mut invalid_level = valid_result;
        invalid_level.slsa_level = Some(0);
        assert!(!invalid_level.is_valid(&config));

        // Missing provenance with allow_missing = true
        let no_provenance = ProvenanceVerificationResult::no_provenance();
        assert!(no_provenance.is_valid(&config));

        // Missing provenance with allow_missing = false
        let mut strict_config = config;
        strict_config.allow_missing_provenance = false;
        assert!(!no_provenance.is_valid(&strict_config));
    }
}
