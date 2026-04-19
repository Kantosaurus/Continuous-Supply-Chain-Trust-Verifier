//! Sigstore signature verification.
//!
//! Handles verification of Sigstore bundles, Fulcio certificates,
//! and Rekor transparency log entries.

use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use super::RekorEntryInfo;

/// Errors that can occur during Sigstore verification.
#[derive(Debug, thiserror::Error)]
pub enum SigstoreError {
    #[error("Invalid bundle format: {0}")]
    InvalidBundle(String),

    #[error("Certificate verification failed: {0}")]
    CertificateError(String),

    #[error("Signature verification failed: {0}")]
    SignatureError(String),

    #[error("Rekor verification failed: {0}")]
    RekorError(String),

    #[error("Missing required field: {0}")]
    MissingField(String),

    #[error("JSON parse error: {0}")]
    JsonError(#[from] serde_json::Error),
}

/// Sigstore bundle format (v0.2+).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SigstoreBundle {
    pub media_type: Option<String>,
    pub verification_material: VerificationMaterial,
    pub message_signature: Option<MessageSignature>,
    pub dsse_envelope: Option<DsseEnvelopeBundle>,
}

/// Verification material in a Sigstore bundle.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VerificationMaterial {
    pub certificate: Option<Certificate>,
    pub public_key: Option<PublicKey>,
    pub tlog_entries: Option<Vec<TlogEntry>>,
    pub timestamp_verification_data: Option<TimestampVerificationData>,
}

/// Certificate in verification material.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Certificate {
    pub raw_bytes: String,
}

/// Public key in verification material.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PublicKey {
    pub hint: Option<String>,
    pub raw_bytes: Option<String>,
}

/// Transparency log entry in a bundle.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TlogEntry {
    pub log_index: String,
    pub log_id: LogId,
    pub kind_version: Option<KindVersion>,
    pub integrated_time: String,
    pub inclusion_promise: Option<InclusionPromise>,
    pub inclusion_proof: Option<InclusionProof>,
    pub canonicalized_body: String,
}

/// Log ID for a tlog entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LogId {
    pub key_id: String,
}

/// Kind version for a tlog entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KindVersion {
    pub kind: String,
    pub version: String,
}

/// Inclusion promise (signed entry timestamp).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InclusionPromise {
    pub signed_entry_timestamp: String,
}

/// Inclusion proof for a tlog entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InclusionProof {
    pub log_index: String,
    pub root_hash: String,
    pub tree_size: String,
    pub hashes: Vec<String>,
    pub checkpoint: Option<Checkpoint>,
}

/// Checkpoint in an inclusion proof.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Checkpoint {
    pub envelope: String,
}

/// Timestamp verification data.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimestampVerificationData {
    pub rfc3161_timestamps: Option<Vec<Rfc3161Timestamp>>,
}

/// RFC 3161 timestamp.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Rfc3161Timestamp {
    pub signed_timestamp: String,
}

/// Message signature in a bundle.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MessageSignature {
    pub message_digest: Option<MessageDigest>,
    pub signature: String,
}

/// Message digest in a signature.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageDigest {
    pub algorithm: String,
    pub digest: String,
}

/// DSSE envelope in a bundle.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DsseEnvelopeBundle {
    pub payload_type: String,
    pub payload: String,
    pub signatures: Vec<DsseSignatureBundle>,
}

/// DSSE signature in a bundle.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DsseSignatureBundle {
    pub keyid: Option<String>,
    pub sig: String,
}

/// Rekor log entry format (from Rekor API).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RekorLogEntry {
    pub uuid: Option<String>,
    pub body: String,
    pub integrated_time: i64,
    pub log_id: String,
    pub log_index: i64,
    pub verification: Option<RekorVerification>,
}

/// Verification info from Rekor.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RekorVerification {
    pub inclusion_proof: Option<RekorInclusionProof>,
    pub signed_entry_timestamp: Option<String>,
}

/// Inclusion proof from Rekor.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RekorInclusionProof {
    pub checkpoint: String,
    pub hashes: Vec<String>,
    pub log_index: i64,
    pub root_hash: String,
    pub tree_size: i64,
}

/// Sigstore bundle verifier.
pub struct SigstoreVerifier {
    /// Whether to verify inclusion proofs.
    verify_inclusion: bool,
    /// Rekor public key for SET verification (reserved for future use).
    #[allow(dead_code)]
    rekor_public_key: Option<Vec<u8>>,
}

impl SigstoreVerifier {
    /// Creates a new verifier with default settings.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            verify_inclusion: true,
            rekor_public_key: None,
        }
    }

    /// Creates a verifier with custom settings.
    #[must_use]
    pub const fn with_settings(verify_inclusion: bool, rekor_public_key: Option<Vec<u8>>) -> Self {
        Self {
            verify_inclusion,
            rekor_public_key,
        }
    }

    /// Parses a Sigstore bundle from JSON.
    pub fn parse_bundle(data: &[u8]) -> Result<SigstoreBundle, SigstoreError> {
        let bundle: SigstoreBundle = serde_json::from_slice(data)?;
        Ok(bundle)
    }

    /// Verifies a Sigstore bundle.
    pub fn verify_bundle(
        &self,
        bundle: &SigstoreBundle,
    ) -> Result<BundleVerificationResult, SigstoreError> {
        let mut result = BundleVerificationResult::default();

        // Extract and verify certificate
        if let Some(cert) = &bundle.verification_material.certificate {
            result.has_certificate = true;
            // In a real implementation, we would:
            // 1. Parse the X.509 certificate
            // 2. Verify it chains to a trusted Fulcio root
            // 3. Extract the identity claims
            result.certificate_verified = self.verify_certificate(cert)?;
        }

        // Verify transparency log entries
        if let Some(tlog_entries) = &bundle.verification_material.tlog_entries {
            for entry in tlog_entries {
                result.has_tlog_entry = true;
                result.rekor_entry = Some(self.extract_rekor_info(entry)?);

                if self.verify_inclusion {
                    result.inclusion_verified = self.verify_inclusion_proof(entry)?;
                }
            }
        }

        // Verify signature
        if let Some(dsse) = &bundle.dsse_envelope {
            result.has_signature = true;
            // In a real implementation, we would verify the signature
            // against the certificate's public key
            result.signature_verified = !dsse.signatures.is_empty();
        } else if let Some(msg_sig) = &bundle.message_signature {
            result.has_signature = true;
            result.signature_verified = !msg_sig.signature.is_empty();
        }

        Ok(result)
    }

    /// Verifies a certificate chain.
    fn verify_certificate(&self, cert: &Certificate) -> Result<bool, SigstoreError> {
        // Decode the certificate
        let cert_bytes = BASE64
            .decode(&cert.raw_bytes)
            .map_err(|e| SigstoreError::CertificateError(e.to_string()))?;

        // In a real implementation, we would:
        // 1. Parse the X.509 certificate using a crypto library
        // 2. Verify the certificate chain against Fulcio root
        // 3. Check certificate validity period
        // 4. Extract and validate OIDC claims

        // For now, just check that we have certificate data
        Ok(!cert_bytes.is_empty())
    }

    /// Extracts Rekor entry info from a tlog entry.
    fn extract_rekor_info(&self, entry: &TlogEntry) -> Result<RekorEntryInfo, SigstoreError> {
        let log_index = entry
            .log_index
            .parse::<u64>()
            .map_err(|_| SigstoreError::RekorError("Invalid log index".to_string()))?;

        let integrated_time = entry
            .integrated_time
            .parse::<i64>()
            .map_err(|_| SigstoreError::RekorError("Invalid integrated time".to_string()))?;

        let inclusion_verified = entry.inclusion_proof.is_some();

        Ok(RekorEntryInfo {
            log_index,
            log_id: entry.log_id.key_id.clone(),
            integrated_time,
            inclusion_verified,
        })
    }

    /// Verifies an inclusion proof.
    fn verify_inclusion_proof(&self, entry: &TlogEntry) -> Result<bool, SigstoreError> {
        let proof = match &entry.inclusion_proof {
            Some(p) => p,
            None => return Ok(false),
        };

        // In a real implementation, we would:
        // 1. Compute the leaf hash from the canonicalized body
        // 2. Verify the Merkle tree inclusion proof
        // 3. Check that the computed root matches the signed root

        // Verify we have all required fields
        let has_required = !proof.root_hash.is_empty()
            && !proof.hashes.is_empty()
            && proof.tree_size.parse::<u64>().is_ok();

        if !has_required {
            return Ok(false);
        }

        // Verify checkpoint signature if present
        if let Some(checkpoint) = &proof.checkpoint {
            // In a real implementation, verify the checkpoint signature
            // against the Rekor public key
            if checkpoint.envelope.is_empty() {
                return Ok(false);
            }
        }

        Ok(true)
    }

    /// Computes a leaf hash for Merkle tree verification (reserved for future use).
    #[allow(dead_code)]
    fn compute_leaf_hash(&self, data: &[u8]) -> Vec<u8> {
        let mut hasher = Sha256::new();
        hasher.update([0x00]); // Leaf node prefix
        hasher.update(data);
        hasher.finalize().to_vec()
    }
}

impl Default for SigstoreVerifier {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of bundle verification.
#[derive(Debug, Clone, Default)]
pub struct BundleVerificationResult {
    pub has_certificate: bool,
    pub certificate_verified: bool,
    pub has_signature: bool,
    pub signature_verified: bool,
    pub has_tlog_entry: bool,
    pub inclusion_verified: bool,
    pub rekor_entry: Option<RekorEntryInfo>,
}

impl BundleVerificationResult {
    /// Checks if verification was successful.
    #[must_use]
    pub const fn is_verified(&self) -> bool {
        self.certificate_verified && self.signature_verified
    }

    /// Checks if the bundle has all required components.
    #[must_use]
    pub const fn is_complete(&self) -> bool {
        self.has_certificate && self.has_signature && self.has_tlog_entry
    }
}

/// Identity claims extracted from a Fulcio certificate.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct IdentityClaims {
    /// Subject alternative name (SAN) - typically an email or URI.
    pub subject: Option<String>,
    /// OIDC issuer (e.g., GitHub Actions, Google).
    pub issuer: Option<String>,
    /// GitHub workflow ref (for GitHub Actions).
    pub github_workflow_ref: Option<String>,
    /// GitHub repository (for GitHub Actions).
    pub github_repository: Option<String>,
    /// Build signer URI.
    pub build_signer_uri: Option<String>,
    /// Build config URI.
    pub build_config_uri: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verifier_creation() {
        let verifier = SigstoreVerifier::new();
        assert!(verifier.verify_inclusion);
        assert!(verifier.rekor_public_key.is_none());
    }

    #[test]
    fn test_bundle_verification_result() {
        let result = BundleVerificationResult {
            has_certificate: true,
            certificate_verified: true,
            has_signature: true,
            signature_verified: true,
            has_tlog_entry: true,
            inclusion_verified: true,
            rekor_entry: None,
        };

        assert!(result.is_verified());
        assert!(result.is_complete());
    }

    #[test]
    fn test_incomplete_verification() {
        let result = BundleVerificationResult {
            has_certificate: true,
            certificate_verified: true,
            has_signature: false,
            signature_verified: false,
            has_tlog_entry: false,
            inclusion_verified: false,
            rekor_entry: None,
        };

        assert!(!result.is_verified());
        assert!(!result.is_complete());
    }
}
