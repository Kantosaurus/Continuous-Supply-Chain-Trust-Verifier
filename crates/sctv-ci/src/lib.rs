//! # SCTV CI
//!
//! CI/CD integrations and SARIF output for Supply Chain Trust Verifier.
//!
//! Provides SARIF (Static Analysis Results Interchange Format) output
//! compatible with GitHub Code Scanning, GitLab SAST, and other CI tools.

use sctv_core::{Alert, AlertType, PackageEcosystem, Severity};
use serde::Serialize;
use std::collections::HashMap;

/// SARIF report format version 2.1.0 for CI/CD integration.
#[derive(Debug, Clone, Serialize)]
pub struct SarifReport {
    /// JSON schema for SARIF.
    #[serde(rename = "$schema")]
    pub schema: String,
    /// SARIF specification version.
    pub version: String,
    /// Analysis runs.
    pub runs: Vec<SarifRun>,
}

impl SarifReport {
    /// Create a SARIF report from alerts.
    pub fn from_alerts(alerts: &[Alert]) -> Self {
        let results: Vec<SarifResult> = alerts
            .iter()
            .map(|alert| SarifResult::from_alert(alert))
            .collect();

        // Collect unique rules
        let mut rules_map: HashMap<String, SarifRule> = HashMap::new();
        for alert in alerts {
            let rule_id = alert.alert_type.type_name().to_string();
            if !rules_map.contains_key(&rule_id) {
                rules_map.insert(
                    rule_id.clone(),
                    SarifRule::from_alert_type(&alert.alert_type),
                );
            }
        }
        let rules: Vec<SarifRule> = rules_map.into_values().collect();

        Self {
            schema: "https://json.schemastore.org/sarif-2.1.0.json".to_string(),
            version: "2.1.0".to_string(),
            runs: vec![SarifRun {
                tool: SarifTool {
                    driver: SarifDriver {
                        name: "Supply Chain Trust Verifier".to_string(),
                        semantic_version: env!("CARGO_PKG_VERSION").to_string(),
                        information_uri: "https://github.com/example/supply-chain-trust-verifier"
                            .to_string(),
                        rules,
                    },
                },
                results,
                invocations: vec![SarifInvocation {
                    execution_successful: true,
                    end_time_utc: chrono::Utc::now().to_rfc3339(),
                }],
            }],
        }
    }

    /// Convert to JSON string.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Convert to compact JSON string.
    pub fn to_json_compact(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
}

/// A single analysis run.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SarifRun {
    pub tool: SarifTool,
    pub results: Vec<SarifResult>,
    pub invocations: Vec<SarifInvocation>,
}

/// Tool information.
#[derive(Debug, Clone, Serialize)]
pub struct SarifTool {
    pub driver: SarifDriver,
}

/// Tool driver information.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SarifDriver {
    pub name: String,
    pub semantic_version: String,
    pub information_uri: String,
    pub rules: Vec<SarifRule>,
}

/// Rule definition.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SarifRule {
    pub id: String,
    pub name: String,
    pub short_description: SarifMessage,
    pub full_description: SarifMessage,
    pub help: SarifHelp,
    pub default_configuration: SarifDefaultConfiguration,
    pub properties: SarifRuleProperties,
}

impl SarifRule {
    fn from_alert_type(alert_type: &AlertType) -> Self {
        let (id, name, short_desc, full_desc, help_text, tags) = match alert_type {
            AlertType::DependencyTampering(_) => (
                "dependency_tampering",
                "Dependency Tampering",
                "Package hash mismatch detected",
                "The hash of a downloaded package does not match the expected hash from the registry. This could indicate package tampering or a supply chain attack.",
                "Verify the package source and consider using lock files with integrity hashes.",
                vec!["security", "supply-chain", "integrity"],
            ),
            AlertType::DowngradeAttack(_) => (
                "downgrade_attack",
                "Downgrade Attack",
                "Package version downgrade detected",
                "A dependency has been downgraded to a lower version than previously resolved. This could be an attempt to reintroduce known vulnerabilities.",
                "Review the version change and ensure it's intentional. Consider using version pinning.",
                vec!["security", "supply-chain", "versioning"],
            ),
            AlertType::Typosquatting(_) => (
                "typosquatting",
                "Typosquatting",
                "Potential typosquatting package detected",
                "A package name is suspiciously similar to a popular package, which may indicate a typosquatting attack attempting to inject malicious code.",
                "Verify the package name is correct and review the package contents before use.",
                vec!["security", "supply-chain", "typosquatting"],
            ),
            AlertType::ProvenanceFailure(_) => (
                "provenance_failure",
                "Provenance Verification Failed",
                "Build provenance verification failed",
                "The package's build provenance attestation could not be verified. This may indicate the package was not built in a trusted environment.",
                "Consider using packages with verified SLSA provenance.",
                vec!["security", "supply-chain", "provenance", "slsa"],
            ),
            AlertType::PolicyViolation(_) => (
                "policy_violation",
                "Policy Violation",
                "Security policy violation detected",
                "A dependency violates one or more security policies defined for this project.",
                "Review the policy violation and update the dependency or policy as appropriate.",
                vec!["security", "policy", "compliance"],
            ),
            AlertType::NewPackage(_) => (
                "new_package",
                "New Package",
                "Recently published package detected",
                "A recently published package has been added as a dependency. New packages may not have undergone sufficient community review.",
                "Review new packages carefully before including them in your project.",
                vec!["security", "supply-chain", "review"],
            ),
            AlertType::SuspiciousMaintainer(_) => (
                "suspicious_maintainer",
                "Suspicious Maintainer Activity",
                "Suspicious maintainer activity detected",
                "Unusual activity has been detected from a package maintainer, which may indicate account compromise.",
                "Verify the maintainer's identity and review recent changes to the package.",
                vec!["security", "supply-chain", "maintainer"],
            ),
        };

        Self {
            id: id.to_string(),
            name: name.to_string(),
            short_description: SarifMessage {
                text: short_desc.to_string(),
            },
            full_description: SarifMessage {
                text: full_desc.to_string(),
            },
            help: SarifHelp {
                text: help_text.to_string(),
                markdown: format!("**Recommendation:** {}", help_text),
            },
            default_configuration: SarifDefaultConfiguration {
                level: match alert_type.default_severity() {
                    Severity::Critical | Severity::High => "error".to_string(),
                    Severity::Medium => "warning".to_string(),
                    _ => "note".to_string(),
                },
            },
            properties: SarifRuleProperties {
                tags: tags.into_iter().map(String::from).collect(),
                precision: "high".to_string(),
                security_severity: match alert_type.default_severity() {
                    Severity::Critical => "9.0".to_string(),
                    Severity::High => "7.0".to_string(),
                    Severity::Medium => "5.0".to_string(),
                    Severity::Low => "3.0".to_string(),
                    Severity::Info => "1.0".to_string(),
                },
            },
        }
    }
}

/// Rule default configuration.
#[derive(Debug, Clone, Serialize)]
pub struct SarifDefaultConfiguration {
    pub level: String,
}

/// Rule properties.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SarifRuleProperties {
    pub tags: Vec<String>,
    pub precision: String,
    pub security_severity: String,
}

/// Help information.
#[derive(Debug, Clone, Serialize)]
pub struct SarifHelp {
    pub text: String,
    pub markdown: String,
}

/// Analysis result (finding).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SarifResult {
    pub rule_id: String,
    pub rule_index: Option<usize>,
    pub level: String,
    pub message: SarifMessage,
    pub locations: Vec<SarifLocation>,
    pub partial_fingerprints: SarifPartialFingerprints,
    pub properties: SarifResultProperties,
}

impl SarifResult {
    fn from_alert(alert: &Alert) -> Self {
        let (package_name, ecosystem, version) = match &alert.alert_type {
            AlertType::DependencyTampering(d) => {
                (d.package_name.clone(), d.ecosystem, d.version.clone())
            }
            AlertType::DowngradeAttack(d) => (
                d.package_name.clone(),
                d.ecosystem,
                d.current_version.to_string(),
            ),
            AlertType::Typosquatting(d) => {
                (d.suspicious_package.clone(), d.ecosystem, String::new())
            }
            AlertType::ProvenanceFailure(d) => {
                (d.package_name.clone(), d.ecosystem, d.version.clone())
            }
            AlertType::PolicyViolation(d) => {
                (d.policy_name.clone(), PackageEcosystem::Npm, String::new())
            }
            AlertType::NewPackage(d) => (d.package_name.clone(), d.ecosystem, d.version.clone()),
            AlertType::SuspiciousMaintainer(d) => {
                (d.package_name.clone(), d.ecosystem, String::new())
            }
        };

        Self {
            rule_id: alert.alert_type.type_name().to_string(),
            rule_index: None,
            level: match alert.severity {
                Severity::Critical | Severity::High => "error",
                Severity::Medium => "warning",
                _ => "note",
            }
            .to_string(),
            message: SarifMessage {
                text: format!("{}: {}", alert.title, alert.description),
            },
            locations: vec![SarifLocation {
                physical_location: SarifPhysicalLocation {
                    artifact_location: SarifArtifactLocation {
                        uri: format!("pkg:{}/{}@{}", ecosystem.purl_type(), package_name, version),
                        uri_base_id: Some("PACKAGE_ROOT".to_string()),
                    },
                    region: None,
                },
                logical_locations: vec![SarifLogicalLocation {
                    name: package_name.clone(),
                    kind: "package".to_string(),
                    fully_qualified_name: format!("{}:{}", ecosystem.purl_type(), package_name),
                }],
            }],
            partial_fingerprints: SarifPartialFingerprints {
                primary_location_line_hash: format!(
                    "{:x}",
                    md5_hash(&format!("{}:{}:{}", package_name, ecosystem, version))
                ),
            },
            properties: SarifResultProperties {
                alert_id: alert.id.to_string(),
                ecosystem: ecosystem.to_string(),
                package_name,
                version: if version.is_empty() {
                    None
                } else {
                    Some(version)
                },
                created_at: alert.created_at.to_rfc3339(),
            },
        }
    }
}

/// Message with text.
#[derive(Debug, Clone, Serialize)]
pub struct SarifMessage {
    pub text: String,
}

/// Result location.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SarifLocation {
    pub physical_location: SarifPhysicalLocation,
    pub logical_locations: Vec<SarifLogicalLocation>,
}

/// Physical location.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SarifPhysicalLocation {
    pub artifact_location: SarifArtifactLocation,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub region: Option<SarifRegion>,
}

/// Artifact location.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SarifArtifactLocation {
    pub uri: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uri_base_id: Option<String>,
}

/// Region in a file.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SarifRegion {
    pub start_line: u32,
    pub start_column: Option<u32>,
    pub end_line: Option<u32>,
    pub end_column: Option<u32>,
}

/// Logical location.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SarifLogicalLocation {
    pub name: String,
    pub kind: String,
    pub fully_qualified_name: String,
}

/// Partial fingerprints for result deduplication.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SarifPartialFingerprints {
    pub primary_location_line_hash: String,
}

/// Result properties.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SarifResultProperties {
    pub alert_id: String,
    pub ecosystem: String,
    pub package_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    pub created_at: String,
}

/// Invocation information.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SarifInvocation {
    pub execution_successful: bool,
    pub end_time_utc: String,
}

/// Simple MD5 hash for fingerprinting (not for security).
fn md5_hash(input: &str) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    input.hash(&mut hasher);
    hasher.finish()
}

/// CI/CD integration configuration.
#[derive(Debug, Clone)]
pub struct CiConfig {
    /// Whether to fail the build on critical alerts.
    pub fail_on_critical: bool,
    /// Whether to fail the build on high severity alerts.
    pub fail_on_high: bool,
    /// Whether to output SARIF format.
    pub output_sarif: bool,
    /// Output file path for SARIF.
    pub sarif_output_path: Option<String>,
}

impl Default for CiConfig {
    fn default() -> Self {
        Self {
            fail_on_critical: true,
            fail_on_high: false,
            output_sarif: true,
            sarif_output_path: None,
        }
    }
}

/// Determines CI exit code based on alerts.
pub fn determine_exit_code(alerts: &[Alert], config: &CiConfig) -> i32 {
    for alert in alerts {
        match alert.severity {
            Severity::Critical if config.fail_on_critical => return 1,
            Severity::High if config.fail_on_high => return 1,
            _ => {}
        }
    }
    0
}

#[cfg(test)]
mod tests {
    use super::*;
    use sctv_core::{AlertType, HashAlgorithm, ProjectId, TamperingDetails, TenantId};

    fn create_test_alert() -> Alert {
        Alert::new(
            TenantId::new(),
            ProjectId::new(),
            AlertType::DependencyTampering(TamperingDetails {
                package_name: "lodash".to_string(),
                ecosystem: PackageEcosystem::Npm,
                version: "4.17.21".to_string(),
                expected_hash: "abc123".to_string(),
                actual_hash: "def456".to_string(),
                algorithm: HashAlgorithm::Sha256,
                registry_source: "https://registry.npmjs.org".to_string(),
            }),
            "Tampering detected in lodash".to_string(),
            "Hash mismatch".to_string(),
        )
    }

    #[test]
    fn test_sarif_report_creation() {
        let alerts = vec![create_test_alert()];
        let report = SarifReport::from_alerts(&alerts);

        assert_eq!(report.version, "2.1.0");
        assert_eq!(report.runs.len(), 1);
        assert_eq!(report.runs[0].results.len(), 1);
    }

    #[test]
    fn test_sarif_json_output() {
        let alerts = vec![create_test_alert()];
        let report = SarifReport::from_alerts(&alerts);
        let json = report.to_json().unwrap();

        assert!(json.contains("dependency_tampering"));
        assert!(json.contains("lodash"));
    }

    #[test]
    fn test_exit_code_critical() {
        let mut alert = create_test_alert();
        alert.severity = Severity::Critical;
        let alerts = vec![alert];

        let config = CiConfig::default();
        assert_eq!(determine_exit_code(&alerts, &config), 1);
    }

    #[test]
    fn test_exit_code_low() {
        let mut alert = create_test_alert();
        alert.severity = Severity::Low;
        let alerts = vec![alert];

        let config = CiConfig::default();
        assert_eq!(determine_exit_code(&alerts, &config), 0);
    }
}
