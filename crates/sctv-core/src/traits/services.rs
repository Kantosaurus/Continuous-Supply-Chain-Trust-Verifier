//! Service trait definitions for the Supply Chain Trust Verifier.

use async_trait::async_trait;
use thiserror::Error;

use crate::domain::{
    Alert, Dependency, Package, PackageEcosystem, PackageVersion, Policy, Project,
};

/// Errors that can occur during service operations.
#[derive(Debug, Error)]
pub enum ServiceError {
    #[error("Repository error: {0}")]
    Repository(#[from] super::RepositoryError),

    #[error("Registry error: {0}")]
    Registry(String),

    #[error("Detection error: {0}")]
    Detection(String),

    #[error("Verification error: {0}")]
    Verification(String),

    #[error("Configuration error: {0}")]
    Configuration(String),

    #[error("Rate limited")]
    RateLimited,

    #[error("Timeout")]
    Timeout,
}

/// Result type for service operations.
pub type ServiceResult<T> = Result<T, ServiceError>;

/// Result of scanning a project for dependencies.
#[derive(Debug, Clone)]
pub struct ScanResult {
    pub dependencies: Vec<Dependency>,
    pub alerts: Vec<Alert>,
    pub duration_ms: u64,
}

/// Service for scanning projects.
#[async_trait]
pub trait ScanService: Send + Sync {
    /// Scans a project for dependencies and threats.
    async fn scan_project(&self, project: &Project) -> ServiceResult<ScanResult>;

    /// Scans dependencies against a policy.
    async fn evaluate_policy(
        &self,
        dependencies: &[Dependency],
        policy: &Policy,
    ) -> ServiceResult<Vec<Alert>>;
}

/// Metadata retrieved from a package registry.
#[derive(Debug, Clone)]
pub struct PackageMetadata {
    pub package: Package,
    pub versions: Vec<PackageVersion>,
}

/// Service for interacting with package registries.
#[async_trait]
pub trait RegistryService: Send + Sync {
    /// Gets the ecosystem this service handles.
    fn ecosystem(&self) -> PackageEcosystem;

    /// Fetches metadata for a package.
    async fn get_package(&self, name: &str) -> ServiceResult<PackageMetadata>;

    /// Fetches a specific version of a package.
    async fn get_version(&self, name: &str, version: &str) -> ServiceResult<PackageVersion>;

    /// Downloads a package for verification.
    async fn download_package(&self, name: &str, version: &str) -> ServiceResult<bytes::Bytes>;

    /// Lists popular packages.
    async fn list_popular(&self, limit: usize) -> ServiceResult<Vec<String>>;

    /// Checks if a package exists.
    async fn package_exists(&self, name: &str) -> ServiceResult<bool>;
}

/// Result of a detection check.
#[derive(Debug, Clone)]
pub struct DetectionResult {
    pub detected: bool,
    pub confidence: f64,
    pub details: serde_json::Value,
}

/// Service for detecting supply chain threats.
#[async_trait]
pub trait DetectorService: Send + Sync {
    /// Returns the detector type name.
    fn detector_type(&self) -> &'static str;

    /// Analyzes a dependency for threats.
    async fn analyze(&self, dependency: &Dependency) -> ServiceResult<Vec<DetectionResult>>;
}

/// Result of hash verification.
#[derive(Debug, Clone)]
pub struct HashVerificationResult {
    pub verified: bool,
    pub expected_hash: Option<String>,
    pub actual_hash: String,
    pub algorithm: String,
}

/// Service for verifying package integrity.
#[async_trait]
pub trait IntegrityService: Send + Sync {
    /// Verifies the hash of a package.
    async fn verify_hash(
        &self,
        ecosystem: PackageEcosystem,
        name: &str,
        version: &str,
    ) -> ServiceResult<HashVerificationResult>;

    /// Verifies the signature of a package.
    async fn verify_signature(
        &self,
        ecosystem: PackageEcosystem,
        name: &str,
        version: &str,
    ) -> ServiceResult<bool>;
}

/// Result of provenance verification.
#[derive(Debug, Clone)]
pub struct ProvenanceResult {
    pub verified: bool,
    pub slsa_level: u8,
    pub builder_id: Option<String>,
    pub source_uri: Option<String>,
    pub errors: Vec<String>,
}

/// Service for verifying build provenance.
#[async_trait]
pub trait ProvenanceService: Send + Sync {
    /// Verifies the provenance of a package.
    async fn verify_provenance(
        &self,
        ecosystem: PackageEcosystem,
        name: &str,
        version: &str,
    ) -> ServiceResult<ProvenanceResult>;
}

/// Typosquatting candidate.
#[derive(Debug, Clone)]
pub struct TyposquatCandidate {
    pub suspicious_name: String,
    pub popular_name: String,
    pub similarity_score: f64,
    pub detection_method: String,
}

/// Service for detecting typosquatting.
#[async_trait]
pub trait TyposquattingService: Send + Sync {
    /// Checks if a package name might be typosquatting.
    async fn check(
        &self,
        ecosystem: PackageEcosystem,
        name: &str,
    ) -> ServiceResult<Vec<TyposquatCandidate>>;
}
