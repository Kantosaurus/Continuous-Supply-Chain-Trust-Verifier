//! Policy domain model for security rules and enforcement.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{HashAlgorithm, PackageEcosystem, Severity, TenantId};

/// Unique identifier for a policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PolicyId(pub Uuid);

impl PolicyId {
    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for PolicyId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for PolicyId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A security policy defining rules for supply chain verification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Policy {
    pub id: PolicyId,
    pub tenant_id: TenantId,
    pub name: String,
    pub description: Option<String>,
    pub rules: Vec<PolicyRule>,
    pub severity_overrides: Vec<SeverityOverride>,
    pub is_default: bool,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Policy {
    /// Creates a new policy with the given name.
    #[must_use]
    pub fn new(tenant_id: TenantId, name: String) -> Self {
        let now = Utc::now();
        Self {
            id: PolicyId::new(),
            tenant_id,
            name,
            description: None,
            rules: Vec::new(),
            severity_overrides: Vec::new(),
            is_default: false,
            enabled: true,
            created_at: now,
            updated_at: now,
        }
    }

    /// Creates a default strict policy.
    #[must_use]
    pub fn default_strict(tenant_id: TenantId) -> Self {
        let mut policy = Self::new(tenant_id, "Strict Security".to_string());
        policy.description = Some("Strict security policy requiring all verifications".to_string());
        policy.rules = vec![
            PolicyRule::RequireHashVerification {
                algorithms: vec![HashAlgorithm::Sha256],
            },
            PolicyRule::BlockTyposquatting { threshold: 0.85 },
            PolicyRule::RequireProvenance { minimum_slsa_level: 1 },
            PolicyRule::EnforceVersionPinning {
                strategy: VersionPinningStrategy::Locked,
            },
        ];
        policy
    }

    /// Creates a permissive policy for getting started.
    #[must_use]
    pub fn default_permissive(tenant_id: TenantId) -> Self {
        let mut policy = Self::new(tenant_id, "Permissive".to_string());
        policy.description = Some("Permissive policy for initial setup".to_string());
        policy.rules = vec![PolicyRule::BlockTyposquatting { threshold: 0.95 }];
        policy
    }

    /// Adds a rule to the policy.
    pub fn add_rule(&mut self, rule: PolicyRule) {
        self.rules.push(rule);
        self.updated_at = Utc::now();
    }

    /// Checks if the policy has a specific rule type.
    #[must_use]
    pub fn has_rule(&self, rule_type: &str) -> bool {
        self.rules.iter().any(|r| r.rule_type() == rule_type)
    }
}

/// Individual policy rules that can be applied.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum PolicyRule {
    /// Require hash verification using specified algorithms.
    RequireHashVerification { algorithms: Vec<HashAlgorithm> },

    /// Require cryptographic signatures from trusted keys.
    RequireSignature { trusted_keys: Vec<String> },

    /// Require SLSA provenance at or above a minimum level.
    RequireProvenance { minimum_slsa_level: u8 },

    /// Block packages with names similar to popular packages.
    BlockTyposquatting { threshold: f64 },

    /// Enforce version pinning strategy.
    EnforceVersionPinning { strategy: VersionPinningStrategy },

    /// Allow only packages matching patterns.
    AllowList { packages: Vec<PackagePattern> },

    /// Block packages matching patterns.
    DenyList { packages: Vec<PackagePattern> },

    /// Require packages to be at least N days old.
    RequireMinimumAge { days: u32 },

    /// Require packages to have at least N maintainers.
    RequireMinimumMaintainers { count: u32 },

    /// Block packages from specific ecosystems.
    BlockEcosystems { ecosystems: Vec<PackageEcosystem> },

    /// Require packages to have a minimum download count.
    RequireMinimumDownloads { count: u64 },
}

impl PolicyRule {
    /// Returns the type name of this rule.
    #[must_use]
    pub const fn rule_type(&self) -> &'static str {
        match self {
            Self::RequireHashVerification { .. } => "require_hash_verification",
            Self::RequireSignature { .. } => "require_signature",
            Self::RequireProvenance { .. } => "require_provenance",
            Self::BlockTyposquatting { .. } => "block_typosquatting",
            Self::EnforceVersionPinning { .. } => "enforce_version_pinning",
            Self::AllowList { .. } => "allow_list",
            Self::DenyList { .. } => "deny_list",
            Self::RequireMinimumAge { .. } => "require_minimum_age",
            Self::RequireMinimumMaintainers { .. } => "require_minimum_maintainers",
            Self::BlockEcosystems { .. } => "block_ecosystems",
            Self::RequireMinimumDownloads { .. } => "require_minimum_downloads",
        }
    }

    /// Returns the default severity for violations of this rule.
    #[must_use]
    pub const fn default_severity(&self) -> Severity {
        match self {
            Self::RequireHashVerification { .. } => Severity::High,
            Self::RequireSignature { .. } => Severity::High,
            Self::RequireProvenance { .. } => Severity::Medium,
            Self::BlockTyposquatting { .. } => Severity::Critical,
            Self::EnforceVersionPinning { .. } => Severity::Medium,
            Self::AllowList { .. } => Severity::High,
            Self::DenyList { .. } => Severity::Critical,
            Self::RequireMinimumAge { .. } => Severity::Medium,
            Self::RequireMinimumMaintainers { .. } => Severity::Low,
            Self::BlockEcosystems { .. } => Severity::High,
            Self::RequireMinimumDownloads { .. } => Severity::Low,
        }
    }
}

/// Version pinning strategies.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum VersionPinningStrategy {
    /// Only exact versions allowed (e.g., "1.2.3").
    Exact,
    /// Must match lock file exactly.
    Locked,
    /// Allow patch updates (e.g., "~1.2.3").
    SemverPatch,
    /// Allow minor updates (e.g., "^1.2.3").
    SemverMinor,
}

/// Pattern for matching packages in allow/deny lists.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackagePattern {
    pub ecosystem: Option<PackageEcosystem>,
    pub name_pattern: String,
    pub version_pattern: Option<String>,
}

impl PackagePattern {
    /// Creates a new pattern for a specific package.
    #[must_use]
    pub fn exact(ecosystem: PackageEcosystem, name: &str) -> Self {
        Self {
            ecosystem: Some(ecosystem),
            name_pattern: name.to_string(),
            version_pattern: None,
        }
    }

    /// Creates a pattern matching any package with the given name prefix.
    #[must_use]
    pub fn prefix(name_prefix: &str) -> Self {
        Self {
            ecosystem: None,
            name_pattern: format!("{name_prefix}*"),
            version_pattern: None,
        }
    }

    /// Checks if a package matches this pattern.
    #[must_use]
    pub fn matches(&self, ecosystem: PackageEcosystem, name: &str) -> bool {
        if let Some(eco) = self.ecosystem {
            if eco != ecosystem {
                return false;
            }
        }

        if self.name_pattern.ends_with('*') {
            let prefix = &self.name_pattern[..self.name_pattern.len() - 1];
            name.starts_with(prefix)
        } else {
            name == self.name_pattern
        }
    }
}

/// Override the severity for specific rule types or packages.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeverityOverride {
    pub rule_type: Option<String>,
    pub package_pattern: Option<PackagePattern>,
    pub severity: Severity,
}
