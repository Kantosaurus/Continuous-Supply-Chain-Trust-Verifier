//! Tampering detection engine.
//!
//! Verifies package integrity by comparing hashes against registry values
//! and detecting cryptographic signature mismatches.

use async_trait::async_trait;
use bytes::Bytes;
use sctv_core::{
    Alert, AlertType, Dependency, HashAlgorithm, PackageChecksums, PackageEcosystem, Severity,
    TamperingDetails,
};
use sha2::{Digest, Sha256, Sha512};
use std::collections::HashMap;
use std::sync::Arc;

use crate::{DetectionResult, Detector, DetectorError, DetectorResult};

/// Configuration for tampering detection.
#[derive(Debug, Clone)]
pub struct TamperingConfig {
    /// Whether to download packages for verification (expensive).
    pub download_for_verification: bool,
    /// Whether to verify signatures when available.
    pub verify_signatures: bool,
    /// Trusted registries per ecosystem.
    pub trusted_registries: HashMap<PackageEcosystem, Vec<String>>,
    /// Minimum hash algorithms required for high-confidence verification.
    pub required_algorithms: Vec<HashAlgorithm>,
}

impl Default for TamperingConfig {
    fn default() -> Self {
        let mut trusted = HashMap::new();
        trusted.insert(
            PackageEcosystem::Npm,
            vec!["https://registry.npmjs.org".to_string()],
        );
        trusted.insert(PackageEcosystem::PyPi, vec!["https://pypi.org".to_string()]);
        trusted.insert(
            PackageEcosystem::Maven,
            vec!["https://repo1.maven.org/maven2".to_string()],
        );
        trusted.insert(
            PackageEcosystem::Cargo,
            vec!["https://crates.io".to_string()],
        );

        Self {
            download_for_verification: false,
            verify_signatures: true,
            trusted_registries: trusted,
            required_algorithms: vec![HashAlgorithm::Sha256],
        }
    }
}

/// Tampering detector result details.
#[derive(Debug, Clone)]
pub struct TamperingFinding {
    /// Type of tampering detected.
    pub finding_type: TamperingType,
    /// Algorithm that detected the mismatch.
    pub algorithm: Option<HashAlgorithm>,
    /// Expected hash value.
    pub expected_hash: Option<String>,
    /// Actual computed hash value.
    pub actual_hash: Option<String>,
    /// Source of the expected value.
    pub source: TamperingSource,
    /// Confidence score (0.0 to 1.0).
    pub confidence: f64,
}

/// Types of tampering detected.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TamperingType {
    /// Hash mismatch detected.
    HashMismatch,
    /// Signature validation failed.
    SignatureInvalid,
    /// Signature missing when expected.
    SignatureMissing,
    /// Checksum file missing from registry.
    ChecksumMissing,
    /// Package modified after initial verification.
    ModifiedSinceVerification,
    /// Package from untrusted source.
    UntrustedSource,
}

impl TamperingType {
    /// Returns the severity for this tampering type.
    #[must_use]
    pub const fn severity(&self) -> Severity {
        match self {
            Self::HashMismatch => Severity::Critical,
            Self::SignatureInvalid => Severity::Critical,
            Self::SignatureMissing => Severity::Medium,
            Self::ChecksumMissing => Severity::Low,
            Self::ModifiedSinceVerification => Severity::Critical,
            Self::UntrustedSource => Severity::High,
        }
    }

    /// Returns a descriptive name.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::HashMismatch => "hash_mismatch",
            Self::SignatureInvalid => "signature_invalid",
            Self::SignatureMissing => "signature_missing",
            Self::ChecksumMissing => "checksum_missing",
            Self::ModifiedSinceVerification => "modified_since_verification",
            Self::UntrustedSource => "untrusted_source",
        }
    }
}

/// Source of the expected hash value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TamperingSource {
    /// Hash from package registry.
    Registry(String),
    /// Hash from lock file.
    LockFile,
    /// Hash from previous verification.
    PreviousVerification,
    /// Hash from SBOM.
    Sbom,
}

impl TamperingSource {
    fn as_str(&self) -> String {
        match self {
            Self::Registry(url) => format!("registry:{}", url),
            Self::LockFile => "lock_file".to_string(),
            Self::PreviousVerification => "previous_verification".to_string(),
            Self::Sbom => "sbom".to_string(),
        }
    }
}

/// Callback trait for fetching registry checksums.
#[async_trait]
pub trait RegistryHashProvider: Send + Sync {
    /// Fetches expected checksums for a package version from registry.
    async fn get_checksums(
        &self,
        ecosystem: PackageEcosystem,
        name: &str,
        version: &str,
    ) -> Result<PackageChecksums, DetectorError>;

    /// Downloads the package content.
    async fn download_package(
        &self,
        ecosystem: PackageEcosystem,
        name: &str,
        version: &str,
    ) -> Result<Bytes, DetectorError>;
}

/// Hash tampering detector.
pub struct TamperingDetector {
    config: TamperingConfig,
    registry_provider: Option<Arc<dyn RegistryHashProvider>>,
}

impl TamperingDetector {
    /// Creates a new tampering detector with default config.
    #[must_use]
    pub fn new() -> Self {
        Self {
            config: TamperingConfig::default(),
            registry_provider: None,
        }
    }

    /// Creates a tampering detector with custom config.
    #[must_use]
    pub fn with_config(config: TamperingConfig) -> Self {
        Self {
            config,
            registry_provider: None,
        }
    }

    /// Sets the registry hash provider.
    #[must_use]
    pub fn with_registry_provider(mut self, provider: Arc<dyn RegistryHashProvider>) -> Self {
        self.registry_provider = Some(provider);
        self
    }

    /// Computes SHA-256 hash of bytes.
    pub fn compute_sha256(bytes: &Bytes) -> String {
        let mut hasher = Sha256::new();
        hasher.update(bytes);
        hex::encode(hasher.finalize())
    }

    /// Computes SHA-512 hash of bytes.
    pub fn compute_sha512(bytes: &Bytes) -> String {
        let mut hasher = Sha512::new();
        hasher.update(bytes);
        hex::encode(hasher.finalize())
    }

    /// Verifies a hash matches the expected value.
    pub fn verify_hash(
        computed: &str,
        expected: &str,
        algorithm: HashAlgorithm,
        source: TamperingSource,
    ) -> Option<TamperingFinding> {
        let computed_lower = computed.to_lowercase();
        let expected_lower = expected.to_lowercase();

        if computed_lower != expected_lower {
            Some(TamperingFinding {
                finding_type: TamperingType::HashMismatch,
                algorithm: Some(algorithm),
                expected_hash: Some(expected.to_string()),
                actual_hash: Some(computed.to_string()),
                source,
                confidence: 1.0,
            })
        } else {
            None
        }
    }

    /// Verifies dependency against stored checksums.
    async fn verify_stored_hashes(&self, dependency: &Dependency) -> Vec<TamperingFinding> {
        let mut findings = Vec::new();

        // If no provider, we can only check stored vs stored
        if self.registry_provider.is_none() {
            return findings;
        }

        let provider = self.registry_provider.as_ref().unwrap();

        // Get expected checksums from registry
        let registry_checksums = match provider
            .get_checksums(
                dependency.ecosystem,
                &dependency.package_name,
                &dependency.resolved_version.to_string(),
            )
            .await
        {
            Ok(c) => c,
            Err(_) => return findings,
        };

        let registry_url = self
            .config
            .trusted_registries
            .get(&dependency.ecosystem)
            .and_then(|urls| urls.first())
            .cloned()
            .unwrap_or_else(|| "unknown".to_string());

        // Compare SHA-256
        if let (Some(stored), Some(expected)) = (
            &dependency.integrity.hash_sha256,
            &registry_checksums.sha256,
        ) {
            if let Some(finding) = Self::verify_hash(
                stored,
                expected,
                HashAlgorithm::Sha256,
                TamperingSource::Registry(registry_url.clone()),
            ) {
                findings.push(finding);
            }
        }

        // Compare SHA-512
        if let (Some(stored), Some(expected)) = (
            &dependency.integrity.hash_sha512,
            &registry_checksums.sha512,
        ) {
            if let Some(finding) = Self::verify_hash(
                stored,
                expected,
                HashAlgorithm::Sha512,
                TamperingSource::Registry(registry_url),
            ) {
                findings.push(finding);
            }
        }

        // Check for missing checksums when they should exist
        if dependency.integrity.hash_sha256.is_none()
            && registry_checksums.sha256.is_some()
            && self
                .config
                .required_algorithms
                .contains(&HashAlgorithm::Sha256)
        {
            findings.push(TamperingFinding {
                finding_type: TamperingType::ChecksumMissing,
                algorithm: Some(HashAlgorithm::Sha256),
                expected_hash: registry_checksums.sha256,
                actual_hash: None,
                source: TamperingSource::LockFile,
                confidence: 0.5,
            });
        }

        findings
    }

    /// Downloads and verifies package content.
    async fn verify_by_download(&self, dependency: &Dependency) -> Vec<TamperingFinding> {
        let mut findings = Vec::new();

        if !self.config.download_for_verification {
            return findings;
        }

        let provider = match &self.registry_provider {
            Some(p) => p,
            None => return findings,
        };

        // Download package
        let bytes = match provider
            .download_package(
                dependency.ecosystem,
                &dependency.package_name,
                &dependency.resolved_version.to_string(),
            )
            .await
        {
            Ok(b) => b,
            Err(_) => return findings,
        };

        // Compute hashes
        let computed_sha256 = Self::compute_sha256(&bytes);
        let computed_sha512 = Self::compute_sha512(&bytes);

        // Verify against stored values
        if let Some(stored) = &dependency.integrity.hash_sha256 {
            if let Some(finding) = Self::verify_hash(
                &computed_sha256,
                stored,
                HashAlgorithm::Sha256,
                TamperingSource::PreviousVerification,
            ) {
                findings.push(finding);
            }
        }

        if let Some(stored) = &dependency.integrity.hash_sha512 {
            if let Some(finding) = Self::verify_hash(
                &computed_sha512,
                stored,
                HashAlgorithm::Sha512,
                TamperingSource::PreviousVerification,
            ) {
                findings.push(finding);
            }
        }

        findings
    }

    /// Converts findings to detection results.
    fn findings_to_results(findings: Vec<TamperingFinding>) -> Vec<DetectionResult> {
        findings
            .into_iter()
            .map(|f| {
                DetectionResult::detected(
                    f.confidence,
                    f.finding_type.name(),
                    serde_json::json!({
                        "type": f.finding_type.name(),
                        "algorithm": f.algorithm.map(|a| format!("{a:?}")),
                        "expected_hash": f.expected_hash,
                        "actual_hash": f.actual_hash,
                        "source": f.source.as_str(),
                        "severity": format!("{:?}", f.finding_type.severity()),
                    }),
                )
            })
            .collect()
    }
}

impl Default for TamperingDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Detector for TamperingDetector {
    fn detector_type(&self) -> &'static str {
        "tampering"
    }

    async fn analyze(&self, dependency: &Dependency) -> DetectorResult<Vec<DetectionResult>> {
        let mut all_findings = Vec::new();

        // Verify against stored hashes
        let stored_findings = self.verify_stored_hashes(dependency).await;
        all_findings.extend(stored_findings);

        // Optionally download and verify
        let download_findings = self.verify_by_download(dependency).await;
        all_findings.extend(download_findings);

        // Check signature status
        match dependency.integrity.signature_status {
            sctv_core::SignatureStatus::Invalid => {
                all_findings.push(TamperingFinding {
                    finding_type: TamperingType::SignatureInvalid,
                    algorithm: None,
                    expected_hash: None,
                    actual_hash: None,
                    source: TamperingSource::Registry("unknown".to_string()),
                    confidence: 1.0,
                });
            }
            sctv_core::SignatureStatus::Missing if self.config.verify_signatures => {
                // Only flag as finding if we expect signatures
                all_findings.push(TamperingFinding {
                    finding_type: TamperingType::SignatureMissing,
                    algorithm: None,
                    expected_hash: None,
                    actual_hash: None,
                    source: TamperingSource::Registry("unknown".to_string()),
                    confidence: 0.3,
                });
            }
            _ => {}
        }

        if all_findings.is_empty() {
            Ok(vec![DetectionResult::not_detected()])
        } else {
            Ok(Self::findings_to_results(all_findings))
        }
    }

    fn create_alerts(&self, dependency: &Dependency, results: &[DetectionResult]) -> Vec<Alert> {
        results
            .iter()
            .filter(|r| r.detected)
            .filter_map(|result| {
                let details_value = &result.details;

                let algorithm = details_value
                    .get("algorithm")
                    .and_then(|v| v.as_str())
                    .and_then(|s| match s {
                        "Sha256" => Some(HashAlgorithm::Sha256),
                        "Sha512" => Some(HashAlgorithm::Sha512),
                        _ => None,
                    })
                    .unwrap_or(HashAlgorithm::Sha256);

                let expected_hash = details_value
                    .get("expected_hash")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown")
                    .to_string();

                let actual_hash = details_value
                    .get("actual_hash")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown")
                    .to_string();

                let source = details_value
                    .get("source")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown")
                    .to_string();

                let tampering_details = TamperingDetails {
                    package_name: dependency.package_name.clone(),
                    ecosystem: dependency.ecosystem,
                    version: dependency.resolved_version.to_string(),
                    expected_hash,
                    actual_hash,
                    algorithm,
                    registry_source: source,
                };

                let alert_type = AlertType::DependencyTampering(tampering_details.clone());

                let title = format!(
                    "Tampering detected in {}@{}",
                    dependency.package_name, dependency.resolved_version
                );

                let description = format!(
                    "Hash mismatch detected for {} version {}. Expected {} hash: {}, but found: {}. \
                    This may indicate the package has been tampered with.",
                    dependency.package_name,
                    dependency.resolved_version,
                    format!("{:?}", algorithm).to_lowercase(),
                    tampering_details.expected_hash,
                    tampering_details.actual_hash,
                );

                let mut alert = Alert::new(
                    dependency.tenant_id,
                    dependency.project_id,
                    alert_type,
                    title,
                    description,
                );
                alert.dependency_id = Some(dependency.id);

                Some(alert)
            })
            .collect()
    }
}

/// Verifier that can compare hashes from multiple sources.
pub struct IntegrityVerifier {
    /// Expected checksums from various sources.
    pub registry_checksums: Option<PackageChecksums>,
    pub lockfile_checksums: Option<PackageChecksums>,
    pub sbom_checksums: Option<PackageChecksums>,
    pub computed_checksums: Option<PackageChecksums>,
}

impl IntegrityVerifier {
    /// Creates a new verifier.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            registry_checksums: None,
            lockfile_checksums: None,
            sbom_checksums: None,
            computed_checksums: None,
        }
    }

    /// Verifies all sources match and returns any mismatches.
    pub fn verify_all(&self) -> Vec<IntegrityMismatch> {
        let mut mismatches = Vec::new();

        // Compare registry vs computed
        if let (Some(registry), Some(computed)) =
            (&self.registry_checksums, &self.computed_checksums)
        {
            if let (Some(r_sha256), Some(c_sha256)) = (&registry.sha256, &computed.sha256) {
                if r_sha256.to_lowercase() != c_sha256.to_lowercase() {
                    mismatches.push(IntegrityMismatch {
                        source_a: "registry".to_string(),
                        source_b: "computed".to_string(),
                        algorithm: HashAlgorithm::Sha256,
                        hash_a: r_sha256.clone(),
                        hash_b: c_sha256.clone(),
                    });
                }
            }
        }

        // Compare lockfile vs computed
        if let (Some(lockfile), Some(computed)) =
            (&self.lockfile_checksums, &self.computed_checksums)
        {
            if let (Some(l_sha256), Some(c_sha256)) = (&lockfile.sha256, &computed.sha256) {
                if l_sha256.to_lowercase() != c_sha256.to_lowercase() {
                    mismatches.push(IntegrityMismatch {
                        source_a: "lockfile".to_string(),
                        source_b: "computed".to_string(),
                        algorithm: HashAlgorithm::Sha256,
                        hash_a: l_sha256.clone(),
                        hash_b: c_sha256.clone(),
                    });
                }
            }
        }

        // Compare registry vs lockfile
        if let (Some(registry), Some(lockfile)) =
            (&self.registry_checksums, &self.lockfile_checksums)
        {
            if let (Some(r_sha256), Some(l_sha256)) = (&registry.sha256, &lockfile.sha256) {
                if r_sha256.to_lowercase() != l_sha256.to_lowercase() {
                    mismatches.push(IntegrityMismatch {
                        source_a: "registry".to_string(),
                        source_b: "lockfile".to_string(),
                        algorithm: HashAlgorithm::Sha256,
                        hash_a: r_sha256.clone(),
                        hash_b: l_sha256.clone(),
                    });
                }
            }
        }

        mismatches
    }

    /// Returns true if all sources match.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.verify_all().is_empty()
    }
}

impl Default for IntegrityVerifier {
    fn default() -> Self {
        Self::new()
    }
}

/// Represents a mismatch between two integrity sources.
#[derive(Debug, Clone)]
pub struct IntegrityMismatch {
    pub source_a: String,
    pub source_b: String,
    pub algorithm: HashAlgorithm,
    pub hash_a: String,
    pub hash_b: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use sctv_core::{DependencyIntegrity, ProjectId, TenantId};
    use semver::Version;

    fn create_test_dependency() -> Dependency {
        let mut dep = Dependency::new(
            ProjectId::new(),
            TenantId::new(),
            "test-package".to_string(),
            PackageEcosystem::Npm,
            "^1.0.0".to_string(),
            Version::new(1, 0, 0),
        );
        dep.integrity = DependencyIntegrity {
            hash_sha256: Some("abc123".to_string()),
            hash_sha512: None,
            signature_status: sctv_core::SignatureStatus::Unknown,
            provenance_status: sctv_core::ProvenanceStatus::Unknown,
            provenance_details: None,
        };
        dep
    }

    #[test]
    fn test_verify_hash_match() {
        let result = TamperingDetector::verify_hash(
            "abc123",
            "ABC123",
            HashAlgorithm::Sha256,
            TamperingSource::Registry("test".to_string()),
        );
        assert!(result.is_none());
    }

    #[test]
    fn test_verify_hash_mismatch() {
        let result = TamperingDetector::verify_hash(
            "abc123",
            "def456",
            HashAlgorithm::Sha256,
            TamperingSource::Registry("test".to_string()),
        );
        assert!(result.is_some());
        let finding = result.unwrap();
        assert_eq!(finding.finding_type, TamperingType::HashMismatch);
        assert_eq!(finding.confidence, 1.0);
    }

    #[test]
    fn test_compute_sha256() {
        let bytes = Bytes::from("test content");
        let hash = TamperingDetector::compute_sha256(&bytes);
        assert!(!hash.is_empty());
        assert_eq!(hash.len(), 64); // SHA-256 produces 32 bytes = 64 hex chars
    }

    #[test]
    fn test_integrity_verifier() {
        let mut verifier = IntegrityVerifier::new();

        verifier.registry_checksums = Some(PackageChecksums {
            sha1: None,
            sha256: Some("abc123".to_string()),
            sha512: None,
            integrity: None,
        });

        verifier.computed_checksums = Some(PackageChecksums {
            sha1: None,
            sha256: Some("abc123".to_string()),
            sha512: None,
            integrity: None,
        });

        assert!(verifier.is_valid());
    }

    #[test]
    fn test_integrity_verifier_mismatch() {
        let mut verifier = IntegrityVerifier::new();

        verifier.registry_checksums = Some(PackageChecksums {
            sha1: None,
            sha256: Some("abc123".to_string()),
            sha512: None,
            integrity: None,
        });

        verifier.computed_checksums = Some(PackageChecksums {
            sha1: None,
            sha256: Some("def456".to_string()),
            sha512: None,
            integrity: None,
        });

        let mismatches = verifier.verify_all();
        assert_eq!(mismatches.len(), 1);
        assert_eq!(mismatches[0].source_a, "registry");
        assert_eq!(mismatches[0].source_b, "computed");
    }

    #[tokio::test]
    async fn test_detector_no_findings() {
        let detector = TamperingDetector::new();
        let dependency = create_test_dependency();

        let results = detector.analyze(&dependency).await.unwrap();
        // Without a registry provider, only signature status is checked
        assert!(!results.is_empty());
    }
}
