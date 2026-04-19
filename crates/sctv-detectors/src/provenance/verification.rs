//! Provenance verification orchestration.
//!
//! Coordinates attestation parsing and Sigstore verification to produce
//! a comprehensive verification result.

use base64::Engine;
use sctv_core::{Dependency, PackageEcosystem, ProvenanceDetails};
use tracing::debug;

use super::{AttestationParser, ProvenanceConfig, ProvenanceVerificationResult, SigstoreVerifier};
use crate::DetectorResult;

/// Orchestrates provenance verification.
pub struct ProvenanceVerifier {
    config: ProvenanceConfig,
    sigstore: SigstoreVerifier,
}

impl ProvenanceVerifier {
    /// Creates a new verifier with default configuration.
    #[must_use]
    pub fn new() -> Self {
        Self {
            config: ProvenanceConfig::default(),
            sigstore: SigstoreVerifier::new(),
        }
    }

    /// Creates a verifier with custom configuration.
    #[must_use]
    pub const fn with_config(config: ProvenanceConfig) -> Self {
        Self {
            sigstore: SigstoreVerifier::with_settings(config.verify_rekor, None),
            config,
        }
    }

    /// Verifies provenance for a dependency.
    ///
    /// # Errors
    ///
    /// Returns an error if the underlying Sigstore verification or bundle parsing fails.
    pub fn verify(&self, dependency: &Dependency) -> DetectorResult<ProvenanceVerificationResult> {
        debug!(
            package = %dependency.package_name,
            version = %dependency.resolved_version,
            ecosystem = ?dependency.ecosystem,
            "Starting provenance verification"
        );

        // Check existing provenance data on the dependency
        let existing_status = &dependency.integrity.provenance_status;
        let existing_details = &dependency.integrity.provenance_details;

        // If we already have verified provenance, use it
        if let Some(level) = existing_status.level() {
            return Ok(self.build_result_from_existing(level, existing_details.as_ref()));
        }

        // Attempt to fetch and verify provenance based on ecosystem
        match dependency.ecosystem {
            PackageEcosystem::Npm => Self::verify_npm_provenance(dependency),
            PackageEcosystem::PyPi => Self::verify_pypi_provenance(dependency),
            PackageEcosystem::Cargo => Self::verify_cargo_provenance(dependency),
            _ => Ok(ProvenanceVerificationResult::no_provenance()),
        }
    }

    /// Builds a verification result from existing provenance data.
    fn build_result_from_existing(
        &self,
        level: u8,
        details: Option<&ProvenanceDetails>,
    ) -> ProvenanceVerificationResult {
        let builder_id = details.and_then(|d| d.builder_id.clone());
        let builder_trusted = builder_id
            .as_ref()
            .is_some_and(|id| self.config.trusted_builders.contains(id));

        ProvenanceVerificationResult {
            has_provenance: true,
            slsa_level: Some(level),
            builder_id,
            builder_trusted,
            source_uri: details.and_then(|d| d.source_uri.clone()),
            source_digest: details.and_then(|d| d.source_digest.clone()),
            sigstore_verified: level >= 2, // Assume verified if level >= 2
            rekor_entry: None,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    /// Verifies provenance for an npm package.
    #[allow(clippy::unnecessary_wraps)]
    fn verify_npm_provenance(
        dependency: &Dependency,
    ) -> DetectorResult<ProvenanceVerificationResult> {
        // npm packages can have provenance attached via Sigstore
        // In a real implementation, we would:
        // 1. Fetch the package metadata from npm registry
        // 2. Extract the attestations from the manifest
        // 3. Verify the Sigstore bundle
        // 4. Parse and validate the SLSA provenance

        debug!(
            package = %dependency.package_name,
            "Checking npm provenance attestations"
        );

        // For now, simulate the verification flow
        // A real implementation would make HTTP requests to the npm registry
        Ok(ProvenanceVerificationResult::no_provenance())
    }

    /// Verifies provenance for a `PyPI` package.
    #[allow(clippy::unnecessary_wraps)]
    fn verify_pypi_provenance(
        dependency: &Dependency,
    ) -> DetectorResult<ProvenanceVerificationResult> {
        // PyPI packages can have provenance via PEP 740 (attestations)
        // In a real implementation, we would:
        // 1. Fetch the package attestations from PyPI
        // 2. Verify the Sigstore bundle
        // 3. Parse and validate the SLSA provenance

        debug!(
            package = %dependency.package_name,
            "Checking PyPI attestations"
        );

        Ok(ProvenanceVerificationResult::no_provenance())
    }

    /// Verifies provenance for a Cargo crate.
    #[allow(clippy::unnecessary_wraps)]
    fn verify_cargo_provenance(
        dependency: &Dependency,
    ) -> DetectorResult<ProvenanceVerificationResult> {
        // Cargo crates don't have native provenance support yet
        // But some crates publish attestations separately

        debug!(
            package = %dependency.package_name,
            "Checking Cargo crate provenance"
        );

        Ok(ProvenanceVerificationResult::no_provenance())
    }

    /// Verifies a Sigstore bundle and extracts provenance.
    ///
    /// # Errors
    ///
    /// Returns an error if DSSE envelope extraction or SLSA provenance parsing fails
    /// at the structural level (soft failures are returned as verification results).
    pub fn verify_sigstore_bundle(
        &self,
        bundle_data: &[u8],
    ) -> DetectorResult<ProvenanceVerificationResult> {
        // Parse the bundle
        let bundle = match SigstoreVerifier::parse_bundle(bundle_data) {
            Ok(b) => b,
            Err(e) => {
                return Ok(ProvenanceVerificationResult::verification_failed(vec![
                    format!("Failed to parse Sigstore bundle: {e}"),
                ]));
            }
        };

        // Verify the bundle
        let verification = match self.sigstore.verify_bundle(&bundle) {
            Ok(v) => v,
            Err(e) => {
                return Ok(ProvenanceVerificationResult::verification_failed(vec![
                    format!("Sigstore verification failed: {e}"),
                ]));
            }
        };

        // Extract the DSSE envelope and parse provenance
        let dsse = bundle.dsse_envelope.as_ref().ok_or_else(|| {
            crate::DetectorError::AnalysisFailed("No DSSE envelope in bundle".to_string())
        })?;

        // Decode and parse the payload
        let payload_bytes = base64::engine::general_purpose::STANDARD
            .decode(&dsse.payload)
            .map_err(|e| {
                crate::DetectorError::AnalysisFailed(format!("Base64 decode error: {e}"))
            })?;

        let statement: super::attestation::InTotoStatement = serde_json::from_slice(&payload_bytes)
            .map_err(|e| crate::DetectorError::AnalysisFailed(format!("JSON parse error: {e}")))?;

        // Parse the SLSA provenance
        let parsed = match AttestationParser::parse_slsa_provenance(&statement) {
            Ok(p) => p,
            Err(e) => {
                return Ok(ProvenanceVerificationResult::verification_failed(vec![
                    format!("Failed to parse SLSA provenance: {e}"),
                ]));
            }
        };

        // Determine SLSA level
        let slsa_level = Self::determine_slsa_level(&verification, &parsed);

        // Check if builder is trusted
        let builder_trusted = self.config.trusted_builders.contains(&parsed.builder_id);

        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        if !verification.is_verified() {
            errors.push("Sigstore signature verification failed".to_string());
        }

        if !builder_trusted && !self.config.trusted_builders.is_empty() {
            warnings.push(format!(
                "Builder '{}' is not in the trusted builders list",
                parsed.builder_id
            ));
        }

        Ok(ProvenanceVerificationResult {
            has_provenance: true,
            slsa_level: Some(slsa_level),
            builder_id: Some(parsed.builder_id),
            builder_trusted,
            source_uri: parsed.source_uri,
            source_digest: parsed.source_digest,
            sigstore_verified: verification.is_verified(),
            rekor_entry: verification.rekor_entry,
            errors,
            warnings,
        })
    }

    /// Determines the SLSA level based on verification results.
    fn determine_slsa_level(
        verification: &super::sigstore::BundleVerificationResult,
        parsed: &super::attestation::ParsedProvenance,
    ) -> u8 {
        // SLSA Level criteria:
        // Level 0: No provenance
        // Level 1: Provenance exists, shows how package was built
        // Level 2: Hosted build service, signed provenance
        // Level 3: Non-falsifiable provenance, isolated build

        let mut level: u8 = 1; // Has provenance = at least level 1

        // Level 2 requirements:
        // - Authenticated provenance (signed)
        // - Hosted build service
        let is_hosted_builder = Self::is_hosted_build_service(&parsed.builder_id);
        let is_signed = verification.signature_verified;

        if is_hosted_builder && is_signed {
            level = 2;
        }

        // Level 3 requirements:
        // - Level 2 requirements
        // - Non-falsifiable (inclusion proof in transparency log)
        // - Source tracked
        if level >= 2 && verification.inclusion_verified && parsed.source_digest.is_some() {
            level = 3;
        }

        level
    }

    /// Checks if a builder ID represents a hosted build service.
    fn is_hosted_build_service(builder_id: &str) -> bool {
        let hosted_patterns = [
            "github.com/",
            "gitlab.com/",
            "cloud.google.com/",
            "cloudbuild",
            "actions/runner",
            "slsa-framework",
        ];

        hosted_patterns.iter().any(|p| builder_id.contains(p))
    }
}

impl Default for ProvenanceVerifier {
    fn default() -> Self {
        Self::new()
    }
}

/// Represents a provenance attestation fetched from a registry.
#[derive(Debug, Clone)]
pub struct FetchedAttestation {
    pub bundle_type: AttestationBundleType,
    pub data: Vec<u8>,
}

/// Types of attestation bundles.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttestationBundleType {
    /// Sigstore bundle format.
    SigstoreBundle,
    /// DSSE envelope with in-toto statement.
    DsseEnvelope,
    /// Raw in-toto statement (unsigned).
    InTotoStatement,
}

/// Trait for fetching attestations from package registries.
#[async_trait::async_trait]
pub trait AttestationFetcher: Send + Sync {
    /// Fetches attestations for a package.
    async fn fetch_attestations(
        &self,
        ecosystem: PackageEcosystem,
        package_name: &str,
        version: &str,
    ) -> Result<Vec<FetchedAttestation>, Box<dyn std::error::Error + Send + Sync>>;
}

/// Mock attestation fetcher for testing.
#[derive(Default)]
pub struct MockAttestationFetcher {
    attestations: std::sync::RwLock<Vec<FetchedAttestation>>,
}

impl MockAttestationFetcher {
    /// Creates a new mock fetcher.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds an attestation to return.
    pub fn add_attestation(&self, attestation: FetchedAttestation) {
        self.attestations
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .push(attestation);
    }
}

#[async_trait::async_trait]
impl AttestationFetcher for MockAttestationFetcher {
    async fn fetch_attestations(
        &self,
        _ecosystem: PackageEcosystem,
        _package_name: &str,
        _version: &str,
    ) -> Result<Vec<FetchedAttestation>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(self
            .attestations
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone())
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
    fn test_verifier_creation() {
        let verifier = ProvenanceVerifier::new();
        assert!(!verifier.config.trusted_builders.is_empty());
    }

    #[test]
    fn test_is_hosted_build_service() {
        let _verifier = ProvenanceVerifier::new();

        assert!(ProvenanceVerifier::is_hosted_build_service(
            "https://github.com/slsa-framework/slsa-github-generator"
        ));
        assert!(ProvenanceVerifier::is_hosted_build_service(
            "https://gitlab.com/runner"
        ));
        assert!(ProvenanceVerifier::is_hosted_build_service(
            "https://cloud.google.com/build"
        ));
        assert!(!ProvenanceVerifier::is_hosted_build_service(
            "local-builder"
        ));
    }

    #[test]
    fn test_verify_returns_no_provenance_for_unknown() {
        let verifier = ProvenanceVerifier::new();
        let dependency = create_test_dependency();

        let result = verifier.verify(&dependency).unwrap();
        assert!(!result.has_provenance);
    }

    #[test]
    fn test_build_result_from_existing() {
        let mut config = ProvenanceConfig::default();
        config.trusted_builders.insert("test-builder".to_string());

        let verifier = ProvenanceVerifier::with_config(config);

        let details = Some(ProvenanceDetails {
            builder_id: Some("test-builder".to_string()),
            source_uri: Some("https://github.com/test/repo".to_string()),
            source_digest: Some("abc123".to_string()),
            build_invocation_id: None,
            attestation_time: None,
        });

        let result = verifier.build_result_from_existing(2, details.as_ref());

        assert!(result.has_provenance);
        assert_eq!(result.slsa_level, Some(2));
        assert!(result.builder_trusted);
        assert_eq!(
            result.source_uri,
            Some("https://github.com/test/repo".to_string())
        );
    }
}
