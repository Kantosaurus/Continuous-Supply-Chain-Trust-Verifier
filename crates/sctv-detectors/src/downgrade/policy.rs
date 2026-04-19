//! Downgrade policy configuration and evaluation.
//!
//! Defines rules for when downgrades should be allowed or flagged.

use sctv_core::PackageEcosystem;
use semver::Version;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

use super::DowngradeSeverity;

/// Policy for handling version downgrades.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DowngradePolicy {
    /// Name of the policy.
    pub name: String,
    /// Description of what this policy does.
    pub description: Option<String>,
    /// Rules for evaluating downgrades.
    pub rules: Vec<DowngradeRule>,
    /// Default action when no rules match.
    pub default_action: DowngradeAction,
}

impl Default for DowngradePolicy {
    fn default() -> Self {
        Self {
            name: "Default".to_string(),
            description: Some("Default downgrade policy".to_string()),
            rules: vec![
                // Always block major downgrades
                DowngradeRule {
                    name: "block-major".to_string(),
                    condition: DowngradeCondition::SeverityAtLeast(DowngradeSeverity::Major),
                    action: DowngradeAction::Block,
                    package_filter: None,
                    ecosystem_filter: None,
                },
                // Alert on minor downgrades
                DowngradeRule {
                    name: "alert-minor".to_string(),
                    condition: DowngradeCondition::SeverityAtLeast(DowngradeSeverity::Minor),
                    action: DowngradeAction::Alert,
                    package_filter: None,
                    ecosystem_filter: None,
                },
            ],
            default_action: DowngradeAction::Warn,
        }
    }
}

impl DowngradePolicy {
    /// Creates a strict policy that blocks all downgrades.
    #[must_use]
    pub fn strict() -> Self {
        Self {
            name: "Strict".to_string(),
            description: Some("Block all version downgrades".to_string()),
            rules: vec![DowngradeRule {
                name: "block-all".to_string(),
                condition: DowngradeCondition::Always,
                action: DowngradeAction::Block,
                package_filter: None,
                ecosystem_filter: None,
            }],
            default_action: DowngradeAction::Block,
        }
    }

    /// Creates a permissive policy that only warns on downgrades.
    #[must_use]
    pub fn permissive() -> Self {
        Self {
            name: "Permissive".to_string(),
            description: Some("Warn on downgrades but don't block".to_string()),
            rules: vec![DowngradeRule {
                name: "alert-major".to_string(),
                condition: DowngradeCondition::SeverityAtLeast(DowngradeSeverity::Major),
                action: DowngradeAction::Alert,
                package_filter: None,
                ecosystem_filter: None,
            }],
            default_action: DowngradeAction::Warn,
        }
    }

    /// Evaluates a downgrade against this policy.
    pub fn evaluate(
        &self,
        package_name: &str,
        ecosystem: PackageEcosystem,
        previous: &Version,
        current: &Version,
        severity: DowngradeSeverity,
    ) -> PolicyEvaluation {
        let context = DowngradeContext {
            package_name: package_name.to_string(),
            ecosystem,
            previous_version: previous.clone(),
            current_version: current.clone(),
            severity,
        };

        // Find the first matching rule
        for rule in &self.rules {
            if rule.matches(&context) {
                return PolicyEvaluation {
                    action: rule.action,
                    matched_rule: Some(rule.name.clone()),
                    reason: Some(format!(
                        "Rule '{}' matched: {:?}",
                        rule.name, rule.condition
                    )),
                };
            }
        }

        // No rule matched, use default action
        PolicyEvaluation {
            action: self.default_action,
            matched_rule: None,
            reason: Some("No rules matched, using default action".to_string()),
        }
    }
}

/// A single rule in a downgrade policy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DowngradeRule {
    /// Name of the rule.
    pub name: String,
    /// Condition that must be met for this rule to apply.
    pub condition: DowngradeCondition,
    /// Action to take when the rule matches.
    pub action: DowngradeAction,
    /// Optional filter by package name patterns.
    pub package_filter: Option<PackageFilter>,
    /// Optional filter by ecosystem.
    pub ecosystem_filter: Option<HashSet<PackageEcosystem>>,
}

impl DowngradeRule {
    /// Checks if this rule matches the given context.
    pub fn matches(&self, context: &DowngradeContext) -> bool {
        // Check package filter
        if let Some(filter) = &self.package_filter {
            if !filter.matches(&context.package_name) {
                return false;
            }
        }

        // Check ecosystem filter
        if let Some(ecosystems) = &self.ecosystem_filter {
            if !ecosystems.contains(&context.ecosystem) {
                return false;
            }
        }

        // Check condition
        self.condition.matches(context)
    }
}

/// Conditions for when a rule applies.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DowngradeCondition {
    /// Always matches.
    Always,
    /// Never matches.
    Never,
    /// Matches if severity is at least the specified level.
    SeverityAtLeast(DowngradeSeverity),
    /// Matches if severity is exactly the specified level.
    SeverityExact(DowngradeSeverity),
    /// Matches if the version gap is at least N.
    VersionGapAtLeast(u64),
    /// Combines multiple conditions with AND.
    All(Vec<DowngradeCondition>),
    /// Combines multiple conditions with OR.
    Any(Vec<DowngradeCondition>),
    /// Negates a condition.
    Not(Box<DowngradeCondition>),
}

impl DowngradeCondition {
    /// Checks if this condition matches the given context.
    pub fn matches(&self, context: &DowngradeContext) -> bool {
        match self {
            Self::Always => true,
            Self::Never => false,
            Self::SeverityAtLeast(min) => context.severity >= *min,
            Self::SeverityExact(exact) => context.severity == *exact,
            Self::VersionGapAtLeast(min) => {
                let gap = Self::calculate_version_gap(
                    &context.previous_version,
                    &context.current_version,
                );
                gap >= *min
            }
            Self::All(conditions) => conditions.iter().all(|c| c.matches(context)),
            Self::Any(conditions) => conditions.iter().any(|c| c.matches(context)),
            Self::Not(condition) => !condition.matches(context),
        }
    }

    /// Calculates the version gap between two versions.
    fn calculate_version_gap(previous: &Version, current: &Version) -> u64 {
        let prev_score = (previous.major * 10000) + (previous.minor * 100) + previous.patch;
        let curr_score = (current.major * 10000) + (current.minor * 100) + current.patch;
        prev_score.saturating_sub(curr_score)
    }
}

/// Action to take when a rule matches.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DowngradeAction {
    /// Allow the downgrade without any notification.
    Allow,
    /// Log a warning but allow the downgrade.
    Warn,
    /// Create an alert for the downgrade.
    Alert,
    /// Block the downgrade (fail the check).
    Block,
}

/// Filter for package names.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageFilter {
    /// Patterns to include (if empty, include all).
    pub include: Vec<String>,
    /// Patterns to exclude.
    pub exclude: Vec<String>,
}

impl PackageFilter {
    /// Checks if a package name matches this filter.
    pub fn matches(&self, package_name: &str) -> bool {
        // Check exclusions first
        for pattern in &self.exclude {
            if Self::pattern_matches(pattern, package_name) {
                return false;
            }
        }

        // If no include patterns, include everything
        if self.include.is_empty() {
            return true;
        }

        // Check inclusions
        for pattern in &self.include {
            if Self::pattern_matches(pattern, package_name) {
                return true;
            }
        }

        false
    }

    /// Simple pattern matching (supports * wildcard).
    fn pattern_matches(pattern: &str, name: &str) -> bool {
        if pattern == "*" {
            return true;
        }

        if pattern.ends_with('*') {
            let prefix = &pattern[..pattern.len() - 1];
            return name.starts_with(prefix);
        }

        if pattern.starts_with('*') {
            let suffix = &pattern[1..];
            return name.ends_with(suffix);
        }

        pattern == name
    }
}

/// Context for evaluating a downgrade.
#[derive(Debug, Clone)]
pub struct DowngradeContext {
    pub package_name: String,
    pub ecosystem: PackageEcosystem,
    pub previous_version: Version,
    pub current_version: Version,
    pub severity: DowngradeSeverity,
}

/// Result of policy evaluation.
#[derive(Debug, Clone)]
pub struct PolicyEvaluation {
    /// The action to take.
    pub action: DowngradeAction,
    /// Name of the rule that matched (if any).
    pub matched_rule: Option<String>,
    /// Reason for the decision.
    pub reason: Option<String>,
}

impl PolicyEvaluation {
    /// Checks if the downgrade should be blocked.
    #[must_use]
    pub fn should_block(&self) -> bool {
        self.action == DowngradeAction::Block
    }

    /// Checks if an alert should be created.
    #[must_use]
    pub fn should_alert(&self) -> bool {
        matches!(self.action, DowngradeAction::Alert | DowngradeAction::Block)
    }

    /// Checks if a warning should be logged.
    #[must_use]
    pub fn should_warn(&self) -> bool {
        matches!(
            self.action,
            DowngradeAction::Warn | DowngradeAction::Alert | DowngradeAction::Block
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_policy() {
        let policy = DowngradePolicy::default();

        // Major downgrade should be blocked
        let result = policy.evaluate(
            "test-pkg",
            PackageEcosystem::Npm,
            &Version::new(2, 0, 0),
            &Version::new(1, 0, 0),
            DowngradeSeverity::Major,
        );
        assert!(result.should_block());

        // Minor downgrade should alert
        let result = policy.evaluate(
            "test-pkg",
            PackageEcosystem::Npm,
            &Version::new(1, 2, 0),
            &Version::new(1, 1, 0),
            DowngradeSeverity::Minor,
        );
        assert!(result.should_alert());
        assert!(!result.should_block());
    }

    #[test]
    fn test_strict_policy() {
        let policy = DowngradePolicy::strict();

        // All downgrades should be blocked
        let result = policy.evaluate(
            "test-pkg",
            PackageEcosystem::Npm,
            &Version::new(1, 0, 1),
            &Version::new(1, 0, 0),
            DowngradeSeverity::Patch,
        );
        assert!(result.should_block());
    }

    #[test]
    fn test_permissive_policy() {
        let policy = DowngradePolicy::permissive();

        // Patch downgrade should just warn
        let result = policy.evaluate(
            "test-pkg",
            PackageEcosystem::Npm,
            &Version::new(1, 0, 1),
            &Version::new(1, 0, 0),
            DowngradeSeverity::Patch,
        );
        assert!(!result.should_block());
        assert!(!result.should_alert());
        assert!(result.should_warn());
    }

    #[test]
    fn test_package_filter() {
        let filter = PackageFilter {
            include: vec!["lodash".to_string(), "react*".to_string()],
            exclude: vec!["react-native".to_string()],
        };

        assert!(filter.matches("lodash"));
        assert!(filter.matches("react"));
        assert!(filter.matches("react-dom"));
        assert!(!filter.matches("react-native"));
        assert!(!filter.matches("vue"));
    }

    #[test]
    fn test_condition_matching() {
        let context = DowngradeContext {
            package_name: "test".to_string(),
            ecosystem: PackageEcosystem::Npm,
            previous_version: Version::new(2, 0, 0),
            current_version: Version::new(1, 0, 0),
            severity: DowngradeSeverity::Major,
        };

        assert!(DowngradeCondition::Always.matches(&context));
        assert!(!DowngradeCondition::Never.matches(&context));
        assert!(DowngradeCondition::SeverityAtLeast(DowngradeSeverity::Minor).matches(&context));
        assert!(!DowngradeCondition::SeverityExact(DowngradeSeverity::Minor).matches(&context));
        assert!(DowngradeCondition::SeverityExact(DowngradeSeverity::Major).matches(&context));
    }
}
