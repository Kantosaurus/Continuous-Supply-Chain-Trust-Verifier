//! Attestation domain model for build provenance verification.

use std::collections::BTreeMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// An attestation providing evidence about a software artifact.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attestation {
    pub id: Uuid,
    pub attestation_type: AttestationType,
    pub predicate_type: String,
    pub subject: AttestationSubject,
    pub issuer: Option<String>,
    pub signature: AttestationSignature,
    pub raw_payload: serde_json::Value,
    pub verified_at: DateTime<Utc>,
}

impl Attestation {
    /// Creates a new attestation.
    #[must_use]
    pub fn new(
        attestation_type: AttestationType,
        predicate_type: String,
        subject: AttestationSubject,
        signature: AttestationSignature,
        raw_payload: serde_json::Value,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            attestation_type,
            predicate_type,
            subject,
            issuer: None,
            signature,
            raw_payload,
            verified_at: Utc::now(),
        }
    }

    /// Checks if this attestation has been verified.
    #[must_use]
    pub const fn is_verified(&self) -> bool {
        self.signature.verified
    }
}

/// Types of attestations.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AttestationType {
    /// SLSA Provenance attestation.
    SlsaProvenance,
    /// in-toto attestation.
    InToto,
    /// Sigstore bundle.
    SigstoreBundle,
    /// Custom attestation type.
    Custom(String),
}

impl AttestationType {
    /// Returns the predicate type URI for SLSA provenance.
    pub const SLSA_PROVENANCE_V1: &'static str =
        "https://slsa.dev/provenance/v1";
    pub const SLSA_PROVENANCE_V02: &'static str =
        "https://slsa.dev/provenance/v0.2";
}

/// Subject of an attestation (the artifact being attested).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttestationSubject {
    pub name: String,
    pub digest: BTreeMap<String, String>,
}

impl AttestationSubject {
    /// Creates a new subject with a single digest.
    #[must_use]
    pub fn new(name: String, algorithm: &str, digest: String) -> Self {
        let mut digests = BTreeMap::new();
        digests.insert(algorithm.to_string(), digest);
        Self {
            name,
            digest: digests,
        }
    }

    /// Gets the SHA-256 digest if available.
    #[must_use]
    pub fn sha256(&self) -> Option<&str> {
        self.digest.get("sha256").map(String::as_str)
    }

    /// Gets the SHA-512 digest if available.
    #[must_use]
    pub fn sha512(&self) -> Option<&str> {
        self.digest.get("sha512").map(String::as_str)
    }
}

/// Signature information for an attestation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttestationSignature {
    pub keyid: Option<String>,
    pub sig: String,
    pub verified: bool,
    pub certificate_chain: Option<Vec<String>>,
    pub transparency_log_entry: Option<TransparencyLogEntry>,
}

impl AttestationSignature {
    /// Creates a new unverified signature.
    #[must_use]
    pub fn new(sig: String) -> Self {
        Self {
            keyid: None,
            sig,
            verified: false,
            certificate_chain: None,
            transparency_log_entry: None,
        }
    }

    /// Marks the signature as verified.
    pub fn mark_verified(&mut self) {
        self.verified = true;
    }
}

/// Entry in a transparency log (e.g., Rekor).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransparencyLogEntry {
    pub log_index: u64,
    pub log_id: String,
    pub integrated_time: DateTime<Utc>,
    pub inclusion_proof: Option<InclusionProof>,
}

/// Inclusion proof for a transparency log entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InclusionProof {
    pub log_index: u64,
    pub root_hash: String,
    pub tree_size: u64,
    pub hashes: Vec<String>,
}

/// SLSA Provenance predicate (v1.0).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlsaProvenancePredicate {
    pub build_definition: BuildDefinition,
    pub run_details: RunDetails,
}

/// Build definition from SLSA provenance.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BuildDefinition {
    pub build_type: String,
    pub external_parameters: serde_json::Value,
    pub internal_parameters: Option<serde_json::Value>,
    pub resolved_dependencies: Option<Vec<ResourceDescriptor>>,
}

/// Run details from SLSA provenance.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunDetails {
    pub builder: BuilderInfo,
    pub metadata: Option<BuildMetadata>,
    pub byproducts: Option<Vec<ResourceDescriptor>>,
}

/// Builder information from SLSA provenance.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BuilderInfo {
    pub id: String,
    pub version: Option<BTreeMap<String, String>>,
    pub builder_dependencies: Option<Vec<ResourceDescriptor>>,
}

/// Build metadata from SLSA provenance.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BuildMetadata {
    pub invocation_id: Option<String>,
    pub started_on: Option<DateTime<Utc>>,
    pub finished_on: Option<DateTime<Utc>>,
}

/// Resource descriptor for dependencies and artifacts.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourceDescriptor {
    pub uri: Option<String>,
    pub digest: Option<BTreeMap<String, String>>,
    pub name: Option<String>,
    pub download_location: Option<String>,
    pub media_type: Option<String>,
    pub content: Option<String>,
    pub annotations: Option<BTreeMap<String, serde_json::Value>>,
}
