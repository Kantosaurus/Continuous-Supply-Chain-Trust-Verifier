//! SLSA attestation parsing and validation.
//!
//! Handles parsing of in-toto attestations and SLSA provenance predicates.

use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use sctv_core::{Attestation, AttestationSignature, AttestationSubject, AttestationType};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Errors that can occur during attestation parsing.
#[derive(Debug, thiserror::Error)]
pub enum AttestationError {
    #[error("Invalid attestation format: {0}")]
    InvalidFormat(String),

    #[error("Unsupported attestation type: {0}")]
    UnsupportedType(String),

    #[error("Invalid predicate: {0}")]
    InvalidPredicate(String),

    #[error("Missing required field: {0}")]
    MissingField(String),

    #[error("Base64 decode error: {0}")]
    Base64Error(#[from] base64::DecodeError),

    #[error("JSON parse error: {0}")]
    JsonError(#[from] serde_json::Error),
}

/// In-toto statement envelope (DSSE format).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DsseEnvelope {
    #[serde(rename = "payloadType")]
    pub payload_type: String,
    pub payload: String,
    pub signatures: Vec<DsseSignature>,
}

/// Signature in a DSSE envelope.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DsseSignature {
    pub keyid: Option<String>,
    pub sig: String,
}

/// In-toto statement structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InTotoStatement {
    #[serde(rename = "_type")]
    pub statement_type: String,
    pub subject: Vec<InTotoSubject>,
    #[serde(rename = "predicateType")]
    pub predicate_type: String,
    pub predicate: serde_json::Value,
}

/// Subject in an in-toto statement.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InTotoSubject {
    pub name: String,
    pub digest: BTreeMap<String, String>,
}

/// SLSA Provenance predicate v0.2.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SlsaProvenanceV02 {
    pub builder: BuilderV02,
    pub build_type: String,
    pub invocation: Option<InvocationV02>,
    pub build_config: Option<serde_json::Value>,
    pub metadata: Option<MetadataV02>,
    pub materials: Option<Vec<MaterialV02>>,
}

/// Builder info in SLSA v0.2.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuilderV02 {
    pub id: String,
}

/// Invocation info in SLSA v0.2.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InvocationV02 {
    pub config_source: Option<ConfigSourceV02>,
    pub parameters: Option<serde_json::Value>,
    pub environment: Option<serde_json::Value>,
}

/// Config source in SLSA v0.2.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigSourceV02 {
    pub uri: Option<String>,
    pub digest: Option<BTreeMap<String, String>>,
    pub entry_point: Option<String>,
}

/// Metadata in SLSA v0.2.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MetadataV02 {
    pub build_invocation_id: Option<String>,
    pub build_started_on: Option<String>,
    pub build_finished_on: Option<String>,
    pub completeness: Option<CompletenessV02>,
    pub reproducible: Option<bool>,
}

/// Completeness info in SLSA v0.2.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletenessV02 {
    pub parameters: Option<bool>,
    pub environment: Option<bool>,
    pub materials: Option<bool>,
}

/// Material (dependency) in SLSA v0.2.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaterialV02 {
    pub uri: Option<String>,
    pub digest: Option<BTreeMap<String, String>>,
}

/// SLSA Provenance predicate v1.0.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SlsaProvenanceV1 {
    pub build_definition: BuildDefinitionV1,
    pub run_details: RunDetailsV1,
}

/// Build definition in SLSA v1.0.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BuildDefinitionV1 {
    pub build_type: String,
    pub external_parameters: serde_json::Value,
    pub internal_parameters: Option<serde_json::Value>,
    pub resolved_dependencies: Option<Vec<ResourceDescriptorV1>>,
}

/// Run details in SLSA v1.0.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunDetailsV1 {
    pub builder: BuilderV1,
    pub metadata: Option<BuildMetadataV1>,
    pub byproducts: Option<Vec<ResourceDescriptorV1>>,
}

/// Builder info in SLSA v1.0.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BuilderV1 {
    pub id: String,
    pub version: Option<BTreeMap<String, String>>,
    pub builder_dependencies: Option<Vec<ResourceDescriptorV1>>,
}

/// Build metadata in SLSA v1.0.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BuildMetadataV1 {
    pub invocation_id: Option<String>,
    pub started_on: Option<String>,
    pub finished_on: Option<String>,
}

/// Resource descriptor in SLSA v1.0.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourceDescriptorV1 {
    pub uri: Option<String>,
    pub digest: Option<BTreeMap<String, String>>,
    pub name: Option<String>,
    pub download_location: Option<String>,
    pub media_type: Option<String>,
    pub content: Option<String>,
    pub annotations: Option<BTreeMap<String, serde_json::Value>>,
}

/// Parser for SLSA attestations.
pub struct AttestationParser;

impl AttestationParser {
    /// Parses a DSSE envelope from JSON.
    pub fn parse_dsse(data: &[u8]) -> Result<DsseEnvelope, AttestationError> {
        let envelope: DsseEnvelope = serde_json::from_slice(data)?;
        Ok(envelope)
    }

    /// Extracts the in-toto statement from a DSSE envelope.
    pub fn extract_statement(envelope: &DsseEnvelope) -> Result<InTotoStatement, AttestationError> {
        if envelope.payload_type != "application/vnd.in-toto+json" {
            return Err(AttestationError::UnsupportedType(format!(
                "Expected in-toto payload type, got: {}",
                envelope.payload_type
            )));
        }

        let payload_bytes = BASE64.decode(&envelope.payload)?;
        let statement: InTotoStatement = serde_json::from_slice(&payload_bytes)?;

        // Validate statement type
        if statement.statement_type != "https://in-toto.io/Statement/v0.1"
            && statement.statement_type != "https://in-toto.io/Statement/v1"
        {
            return Err(AttestationError::UnsupportedType(format!(
                "Unsupported statement type: {}",
                statement.statement_type
            )));
        }

        Ok(statement)
    }

    /// Parses SLSA provenance from an in-toto statement.
    pub fn parse_slsa_provenance(
        statement: &InTotoStatement,
    ) -> Result<ParsedProvenance, AttestationError> {
        match statement.predicate_type.as_str() {
            "https://slsa.dev/provenance/v0.2" => {
                let predicate: SlsaProvenanceV02 =
                    serde_json::from_value(statement.predicate.clone())?;
                Ok(Self::convert_v02_to_parsed(&predicate, statement))
            }
            "https://slsa.dev/provenance/v1" => {
                let predicate: SlsaProvenanceV1 =
                    serde_json::from_value(statement.predicate.clone())?;
                Ok(Self::convert_v1_to_parsed(&predicate, statement))
            }
            other => Err(AttestationError::UnsupportedType(format!(
                "Unsupported predicate type: {other}"
            ))),
        }
    }

    /// Converts SLSA v0.2 predicate to parsed format.
    fn convert_v02_to_parsed(
        predicate: &SlsaProvenanceV02,
        statement: &InTotoStatement,
    ) -> ParsedProvenance {
        let source_uri = predicate
            .invocation
            .as_ref()
            .and_then(|i| i.config_source.as_ref())
            .and_then(|c| c.uri.clone());

        let source_digest = predicate
            .invocation
            .as_ref()
            .and_then(|i| i.config_source.as_ref())
            .and_then(|c| c.digest.as_ref())
            .and_then(|d| d.get("sha1").or_else(|| d.get("sha256")).cloned());

        let invocation_id = predicate
            .metadata
            .as_ref()
            .and_then(|m| m.build_invocation_id.clone());

        ParsedProvenance {
            predicate_type: statement.predicate_type.clone(),
            builder_id: predicate.builder.id.clone(),
            build_type: predicate.build_type.clone(),
            source_uri,
            source_digest,
            invocation_id,
            subjects: statement
                .subject
                .iter()
                .map(|s| ParsedSubject {
                    name: s.name.clone(),
                    digests: s.digest.clone(),
                })
                .collect(),
            materials: predicate
                .materials
                .as_ref()
                .map(|m| {
                    m.iter()
                        .map(|mat| ParsedMaterial {
                            uri: mat.uri.clone(),
                            digest: mat.digest.clone(),
                        })
                        .collect()
                })
                .unwrap_or_default(),
        }
    }

    /// Converts SLSA v1.0 predicate to parsed format.
    fn convert_v1_to_parsed(
        predicate: &SlsaProvenanceV1,
        statement: &InTotoStatement,
    ) -> ParsedProvenance {
        // Extract source info from resolved dependencies
        let source_dep = predicate
            .build_definition
            .resolved_dependencies
            .as_ref()
            .and_then(|deps| deps.first());

        let source_uri = source_dep.and_then(|d| d.uri.clone());
        let source_digest = source_dep
            .and_then(|d| d.digest.as_ref())
            .and_then(|dig| dig.get("sha1").or_else(|| dig.get("gitCommit")).cloned());

        let invocation_id = predicate
            .run_details
            .metadata
            .as_ref()
            .and_then(|m| m.invocation_id.clone());

        ParsedProvenance {
            predicate_type: statement.predicate_type.clone(),
            builder_id: predicate.run_details.builder.id.clone(),
            build_type: predicate.build_definition.build_type.clone(),
            source_uri,
            source_digest,
            invocation_id,
            subjects: statement
                .subject
                .iter()
                .map(|s| ParsedSubject {
                    name: s.name.clone(),
                    digests: s.digest.clone(),
                })
                .collect(),
            materials: predicate
                .build_definition
                .resolved_dependencies
                .as_ref()
                .map(|deps| {
                    deps.iter()
                        .map(|d| ParsedMaterial {
                            uri: d.uri.clone(),
                            digest: d.digest.clone(),
                        })
                        .collect()
                })
                .unwrap_or_default(),
        }
    }

    /// Converts parsed provenance to core domain types.
    pub fn to_attestation(
        envelope: &DsseEnvelope,
        statement: &InTotoStatement,
        _parsed: &ParsedProvenance,
    ) -> Attestation {
        let subject = if let Some(first_subject) = statement.subject.first() {
            AttestationSubject {
                name: first_subject.name.clone(),
                digest: first_subject.digest.clone(),
            }
        } else {
            AttestationSubject {
                name: String::new(),
                digest: BTreeMap::new(),
            }
        };

        let signature = if let Some(sig) = envelope.signatures.first() {
            AttestationSignature {
                keyid: sig.keyid.clone(),
                sig: sig.sig.clone(),
                verified: false,
                certificate_chain: None,
                transparency_log_entry: None,
            }
        } else {
            AttestationSignature::new(String::new())
        };

        let attestation_type = if statement
            .predicate_type
            .starts_with("https://slsa.dev/provenance")
        {
            AttestationType::SlsaProvenance
        } else {
            AttestationType::InToto
        };

        Attestation::new(
            attestation_type,
            statement.predicate_type.clone(),
            subject,
            signature,
            statement.predicate.clone(),
        )
    }
}

/// Parsed provenance data in a normalized format.
#[derive(Debug, Clone)]
pub struct ParsedProvenance {
    pub predicate_type: String,
    pub builder_id: String,
    pub build_type: String,
    pub source_uri: Option<String>,
    pub source_digest: Option<String>,
    pub invocation_id: Option<String>,
    pub subjects: Vec<ParsedSubject>,
    pub materials: Vec<ParsedMaterial>,
}

/// Parsed subject from attestation.
#[derive(Debug, Clone)]
pub struct ParsedSubject {
    pub name: String,
    pub digests: BTreeMap<String, String>,
}

/// Parsed material (dependency) from attestation.
#[derive(Debug, Clone)]
pub struct ParsedMaterial {
    pub uri: Option<String>,
    pub digest: Option<BTreeMap<String, String>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_dsse_envelope() {
        let envelope_json = r#"{
            "payloadType": "application/vnd.in-toto+json",
            "payload": "eyJfdHlwZSI6Imh0dHBzOi8vaW4tdG90by5pby9TdGF0ZW1lbnQvdjAuMSIsInN1YmplY3QiOlt7Im5hbWUiOiJ0ZXN0IiwiZGlnZXN0Ijp7InNoYTI1NiI6ImFiYzEyMyJ9fV0sInByZWRpY2F0ZVR5cGUiOiJodHRwczovL3Nsc2EuZGV2L3Byb3ZlbmFuY2UvdjAuMiIsInByZWRpY2F0ZSI6eyJidWlsZGVyIjp7ImlkIjoidGVzdC1idWlsZGVyIn0sImJ1aWxkVHlwZSI6InRlc3QifX0=",
            "signatures": [{"sig": "test-sig"}]
        }"#;

        let envelope = AttestationParser::parse_dsse(envelope_json.as_bytes()).unwrap();
        assert_eq!(envelope.payload_type, "application/vnd.in-toto+json");
        assert!(!envelope.signatures.is_empty());
    }
}
