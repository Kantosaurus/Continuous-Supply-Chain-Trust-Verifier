//! Dependency domain model representing package dependencies.

use chrono::{DateTime, Utc};
use semver::Version;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{HashAlgorithm, PackageEcosystem, ProjectId, TenantId};

/// Unique identifier for a dependency.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DependencyId(pub Uuid);

impl DependencyId {
    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for DependencyId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for DependencyId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A dependency of a project.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dependency {
    pub id: DependencyId,
    pub project_id: ProjectId,
    pub tenant_id: TenantId,
    pub package_name: String,
    pub ecosystem: PackageEcosystem,
    pub version_constraint: String,
    pub resolved_version: Version,
    pub is_direct: bool,
    pub is_dev_dependency: bool,
    pub depth: u32,
    pub parent_id: Option<DependencyId>,
    pub integrity: DependencyIntegrity,
    pub first_seen_at: DateTime<Utc>,
    pub last_verified_at: DateTime<Utc>,
}

impl Dependency {
    /// Creates a new dependency.
    #[must_use]
    pub fn new(
        project_id: ProjectId,
        tenant_id: TenantId,
        package_name: String,
        ecosystem: PackageEcosystem,
        version_constraint: String,
        resolved_version: Version,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: DependencyId::new(),
            project_id,
            tenant_id,
            package_name,
            ecosystem,
            version_constraint,
            resolved_version,
            is_direct: true,
            is_dev_dependency: false,
            depth: 0,
            parent_id: None,
            integrity: DependencyIntegrity::default(),
            first_seen_at: now,
            last_verified_at: now,
        }
    }

    /// Creates a Package URL (purl) for this dependency.
    #[must_use]
    pub fn purl(&self) -> String {
        format!(
            "pkg:{}/{}@{}",
            self.ecosystem.purl_type(),
            self.package_name,
            self.resolved_version
        )
    }

    /// Checks if this dependency has been verified.
    #[must_use]
    pub const fn is_verified(&self) -> bool {
        matches!(self.integrity.signature_status, SignatureStatus::Verified)
            || self.integrity.hash_sha256.is_some()
    }
}

/// Integrity information for a dependency.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DependencyIntegrity {
    pub hash_sha256: Option<String>,
    pub hash_sha512: Option<String>,
    pub signature_status: SignatureStatus,
    pub provenance_status: ProvenanceStatus,
    pub provenance_details: Option<ProvenanceDetails>,
}

impl DependencyIntegrity {
    /// Sets the hash for the given algorithm.
    pub fn set_hash(&mut self, algorithm: HashAlgorithm, hash: String) {
        match algorithm {
            HashAlgorithm::Sha256 => self.hash_sha256 = Some(hash),
            HashAlgorithm::Sha512 => self.hash_sha512 = Some(hash),
            HashAlgorithm::Blake3 => {} // Not stored currently
        }
    }

    /// Gets the hash for the given algorithm.
    #[must_use]
    pub fn get_hash(&self, algorithm: HashAlgorithm) -> Option<&str> {
        match algorithm {
            HashAlgorithm::Sha256 => self.hash_sha256.as_deref(),
            HashAlgorithm::Sha512 => self.hash_sha512.as_deref(),
            HashAlgorithm::Blake3 => None,
        }
    }
}

/// Status of cryptographic signature verification.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SignatureStatus {
    /// Signature verified successfully.
    Verified,
    /// Signature verification failed.
    Invalid,
    /// No signature present.
    Missing,
    /// Signature status not yet checked.
    #[default]
    Unknown,
}

/// Status of build provenance verification.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProvenanceStatus {
    /// SLSA Level 0 - No provenance.
    SlsaLevel0,
    /// SLSA Level 1 - Build process documented.
    SlsaLevel1,
    /// SLSA Level 2 - Hosted build platform.
    SlsaLevel2,
    /// SLSA Level 3 - Hardened builds.
    SlsaLevel3,
    /// Provenance status not determined.
    #[default]
    Unknown,
    /// Provenance verification failed.
    Failed,
}

impl ProvenanceStatus {
    /// Returns the SLSA level as a number.
    #[must_use]
    pub const fn level(&self) -> Option<u8> {
        match self {
            Self::SlsaLevel0 => Some(0),
            Self::SlsaLevel1 => Some(1),
            Self::SlsaLevel2 => Some(2),
            Self::SlsaLevel3 => Some(3),
            Self::Unknown | Self::Failed => None,
        }
    }

    /// Creates a provenance status from a SLSA level.
    #[must_use]
    pub const fn from_level(level: u8) -> Self {
        match level {
            0 => Self::SlsaLevel0,
            1 => Self::SlsaLevel1,
            2 => Self::SlsaLevel2,
            3 => Self::SlsaLevel3,
            _ => Self::Unknown,
        }
    }
}

/// Details about build provenance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvenanceDetails {
    pub builder_id: Option<String>,
    pub source_uri: Option<String>,
    pub source_digest: Option<String>,
    pub build_invocation_id: Option<String>,
    pub attestation_time: Option<DateTime<Utc>>,
}
