//! Typosquatting detection engine.
//!
//! Detects packages with names suspiciously similar to popular packages,
//! which may indicate a typosquatting attack.

use async_trait::async_trait;
use parking_lot::RwLock;
use sctv_core::{
    normalize_package_name, Alert, AlertType, Dependency, PackageEcosystem, TyposquattingDetails,
    TyposquattingMethod,
};
use serde::Serialize;
use std::collections::HashMap;
use strsim::{damerau_levenshtein, jaro_winkler};

use crate::{DetectionResult, Detector, DetectorResult};

/// Configuration for the typosquatting detector.
#[derive(Debug, Clone)]
pub struct TyposquattingConfig {
    /// Maximum Levenshtein distance to consider a match.
    pub levenshtein_threshold: usize,
    /// Minimum Jaro-Winkler similarity to consider a match.
    pub jaro_winkler_threshold: f64,
    /// Whether to check for phonetic similarity.
    pub check_phonetic: bool,
    /// Whether to check for keyboard proximity typos.
    pub check_keyboard_distance: bool,
    /// Minimum package name length to check.
    pub min_name_length: usize,
}

impl Default for TyposquattingConfig {
    fn default() -> Self {
        Self {
            levenshtein_threshold: 2,
            jaro_winkler_threshold: 0.85,
            check_phonetic: true,
            check_keyboard_distance: true,
            min_name_length: 4,
        }
    }
}

/// Store for popular package names by ecosystem.
pub struct PopularPackagesStore {
    packages: RwLock<HashMap<PackageEcosystem, Vec<String>>>,
}

impl Default for PopularPackagesStore {
    fn default() -> Self {
        Self::new()
    }
}

impl PopularPackagesStore {
    /// Creates a new store with default popular packages.
    // The length comes entirely from static package-name list literals; extracting
    // them would add indirection without improving readability.
    #[allow(clippy::too_many_lines)]
    #[must_use]
    pub fn new() -> Self {
        let mut packages = HashMap::new();

        // npm popular packages
        packages.insert(
            PackageEcosystem::Npm,
            vec![
                "lodash",
                "chalk",
                "react",
                "express",
                "axios",
                "moment",
                "uuid",
                "commander",
                "debug",
                "fs-extra",
                "async",
                "request",
                "underscore",
                "bluebird",
                "webpack",
                "typescript",
                "jest",
                "mocha",
                "eslint",
                "prettier",
                "next",
                "vue",
                "angular",
                "rxjs",
                "ramda",
                "inquirer",
                "yargs",
                "glob",
                "minimist",
                "semver",
                "dotenv",
                "cross-env",
            ]
            .into_iter()
            .map(String::from)
            .collect(),
        );

        // PyPI popular packages
        packages.insert(
            PackageEcosystem::PyPi,
            vec![
                "requests",
                "numpy",
                "pandas",
                "boto3",
                "django",
                "flask",
                "pytest",
                "pyyaml",
                "cryptography",
                "pillow",
                "setuptools",
                "pip",
                "wheel",
                "urllib3",
                "certifi",
                "idna",
                "chardet",
                "six",
                "python-dateutil",
                "pytz",
                "packaging",
                "attrs",
                "click",
                "jinja2",
                "markupsafe",
            ]
            .into_iter()
            .map(String::from)
            .collect(),
        );

        // Cargo popular packages
        packages.insert(
            PackageEcosystem::Cargo,
            vec![
                "serde",
                "tokio",
                "rand",
                "clap",
                "log",
                "regex",
                "chrono",
                "reqwest",
                "futures",
                "hyper",
                "lazy_static",
                "serde_json",
                "thiserror",
                "anyhow",
                "tracing",
                "bytes",
                "syn",
                "quote",
                "proc-macro2",
                "itertools",
            ]
            .into_iter()
            .map(String::from)
            .collect(),
        );

        Self {
            packages: RwLock::new(packages),
        }
    }

    /// Gets popular packages for an ecosystem.
    pub fn get(&self, ecosystem: PackageEcosystem) -> Vec<String> {
        self.packages
            .read()
            .get(&ecosystem)
            .cloned()
            .unwrap_or_default()
    }

    /// Updates the popular packages for an ecosystem.
    pub fn set(&self, ecosystem: PackageEcosystem, packages: Vec<String>) {
        self.packages.write().insert(ecosystem, packages);
    }
}

/// Global popular packages store.
static POPULAR_PACKAGES: std::sync::LazyLock<PopularPackagesStore> =
    std::sync::LazyLock::new(PopularPackagesStore::new);

/// A potential typosquatting candidate.
#[derive(Debug, Clone, Serialize, serde::Deserialize)]
pub struct TyposquatCandidate {
    pub suspicious_name: String,
    pub popular_name: String,
    pub similarity_score: f64,
    pub detection_method: TyposquattingMethod,
    pub confidence: Confidence,
}

/// Confidence level for detections.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, serde::Deserialize)]
pub enum Confidence {
    Low,
    Medium,
    High,
}

impl Confidence {
    const fn to_score(self) -> f64 {
        match self {
            Self::Low => 0.5,
            Self::Medium => 0.75,
            Self::High => 0.95,
        }
    }
}

/// Typosquatting detector.
pub struct TyposquattingDetector {
    config: TyposquattingConfig,
}

impl TyposquattingDetector {
    /// Creates a new detector with default configuration.
    #[must_use]
    pub fn new() -> Self {
        Self {
            config: TyposquattingConfig::default(),
        }
    }

    /// Creates a detector with custom configuration.
    #[must_use]
    pub const fn with_config(config: TyposquattingConfig) -> Self {
        Self { config }
    }

    /// Checks a package name for potential typosquatting.
    // The usize→f64 casts in this function are for a similarity ratio on package name
    // lengths (< 1000 chars). All values fit exactly in f64 (< 2^53), so no precision
    // is lost.
    #[allow(clippy::cast_precision_loss)]
    pub fn check(&self, ecosystem: PackageEcosystem, name: &str) -> Vec<TyposquatCandidate> {
        if name.len() < self.config.min_name_length {
            return Vec::new();
        }

        let normalized = normalize_package_name(name);
        let popular = POPULAR_PACKAGES.get(ecosystem);
        let mut candidates = Vec::new();

        for popular_name in &popular {
            let popular_normalized = normalize_package_name(popular_name);

            // Skip if it's the same package
            if normalized == popular_normalized {
                continue;
            }

            // Check Damerau-Levenshtein distance
            let distance = damerau_levenshtein(&normalized, &popular_normalized);
            if distance > 0 && distance <= self.config.levenshtein_threshold {
                let max_len = normalized.len().max(popular_normalized.len());
                let similarity = 1.0 - (distance as f64 / max_len as f64);
                let confidence =
                    Self::calculate_confidence(distance, &normalized, &popular_normalized);

                candidates.push(TyposquatCandidate {
                    suspicious_name: name.to_string(),
                    popular_name: popular_name.clone(),
                    similarity_score: similarity,
                    detection_method: TyposquattingMethod::DamerauLevenshtein,
                    confidence,
                });
                continue;
            }

            // Check Jaro-Winkler similarity for longer names
            if normalized.len() >= 6 && popular_normalized.len() >= 6 {
                let similarity = jaro_winkler(&normalized, &popular_normalized);
                if similarity >= self.config.jaro_winkler_threshold && similarity < 1.0 {
                    candidates.push(TyposquatCandidate {
                        suspicious_name: name.to_string(),
                        popular_name: popular_name.clone(),
                        similarity_score: similarity,
                        detection_method: TyposquattingMethod::JaroWinkler,
                        confidence: Confidence::Medium,
                    });
                    continue;
                }
            }

            // Check for combosquatting (word order swapping)
            if Self::is_combosquat(&normalized, &popular_normalized) {
                candidates.push(TyposquatCandidate {
                    suspicious_name: name.to_string(),
                    popular_name: popular_name.clone(),
                    similarity_score: 0.95,
                    detection_method: TyposquattingMethod::Combosquatting,
                    confidence: Confidence::High,
                });
            }
        }

        // Sort by similarity score (highest first). NaN scores compare as Equal to avoid panics.
        candidates.sort_by(|a, b| {
            b.similarity_score
                .partial_cmp(&a.similarity_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        candidates
    }

    /// Calculates confidence based on edit distance and name characteristics.
    fn calculate_confidence(distance: usize, suspicious: &str, popular: &str) -> Confidence {
        // Single character difference is very suspicious
        if distance == 1 {
            // Check if it's a common typo pattern
            if Self::is_common_typo(suspicious, popular) {
                return Confidence::High;
            }
            return Confidence::High;
        }

        // Two character differences
        if distance == 2 {
            // Transposition is common
            if Self::has_transposition(suspicious, popular) {
                return Confidence::High;
            }
            return Confidence::Medium;
        }

        Confidence::Low
    }

    /// Checks if the difference looks like a common typo.
    fn is_common_typo(suspicious: &str, popular: &str) -> bool {
        // Common typo patterns:
        // - Adjacent key typos (e.g., 'a' -> 's')
        // - Missing/extra character
        // - Doubled character

        let s_chars: Vec<char> = suspicious.chars().collect();
        let p_chars: Vec<char> = popular.chars().collect();

        // Check for doubled character
        if s_chars.len() == p_chars.len() + 1 {
            let mut diff_count = 0;
            let mut s_idx = 0;
            let mut p_idx = 0;

            while s_idx < s_chars.len() && p_idx < p_chars.len() {
                if s_chars[s_idx] == p_chars[p_idx] {
                    s_idx += 1;
                    p_idx += 1;
                } else if s_idx + 1 < s_chars.len() && s_chars[s_idx] == s_chars[s_idx + 1] {
                    // Doubled character
                    s_idx += 1;
                    diff_count += 1;
                } else {
                    return false;
                }
            }
            return diff_count == 1;
        }

        true // Default to true for single char difference
    }

    /// Checks if there's a character transposition.
    fn has_transposition(suspicious: &str, popular: &str) -> bool {
        if suspicious.len() != popular.len() {
            return false;
        }

        let s_chars: Vec<char> = suspicious.chars().collect();
        let p_chars: Vec<char> = popular.chars().collect();

        let mut diffs = Vec::new();
        for (i, (s, p)) in s_chars.iter().zip(p_chars.iter()).enumerate() {
            if s != p {
                diffs.push(i);
            }
        }

        // Check if exactly two adjacent positions differ and are swapped
        if diffs.len() == 2 && diffs[1] == diffs[0] + 1 {
            return s_chars[diffs[0]] == p_chars[diffs[1]]
                && s_chars[diffs[1]] == p_chars[diffs[0]];
        }

        false
    }

    /// Checks if names are combosquatting variants.
    fn is_combosquat(suspicious: &str, popular: &str) -> bool {
        let s_parts: Vec<&str> = suspicious.split(['-', '_']).collect();
        let p_parts: Vec<&str> = popular.split(['-', '_']).collect();

        if s_parts.len() != p_parts.len() || s_parts.len() < 2 {
            return false;
        }

        // Check if parts are the same but in different order
        let mut s_sorted = s_parts.clone();
        let mut p_sorted = p_parts.clone();
        s_sorted.sort_unstable();
        p_sorted.sort_unstable();

        s_sorted == p_sorted && s_parts != p_parts
    }
}

impl Default for TyposquattingDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Detector for TyposquattingDetector {
    fn detector_type(&self) -> &'static str {
        "typosquatting"
    }

    async fn analyze(&self, dependency: &Dependency) -> DetectorResult<Vec<DetectionResult>> {
        let candidates = self.check(dependency.ecosystem, &dependency.package_name);

        let results: Vec<DetectionResult> = candidates
            .into_iter()
            .map(|c| {
                DetectionResult::detected(
                    c.confidence.to_score(),
                    &format!("{:?}", c.detection_method),
                    serde_json::to_value(&c).unwrap_or_default(),
                )
            })
            .collect();

        Ok(results)
    }

    fn create_alerts(&self, dependency: &Dependency, results: &[DetectionResult]) -> Vec<Alert> {
        results
            .iter()
            .filter(|r| r.detected)
            .filter_map(|r| {
                let candidate: TyposquatCandidate =
                    serde_json::from_value(r.details.clone()).ok()?;

                let details = TyposquattingDetails {
                    suspicious_package: candidate.suspicious_name.clone(),
                    ecosystem: dependency.ecosystem,
                    similar_popular_package: candidate.popular_name.clone(),
                    similarity_score: candidate.similarity_score,
                    detection_method: candidate.detection_method,
                    popular_package_downloads: None,
                };

                Some(Alert::new(
                    dependency.tenant_id,
                    dependency.project_id,
                    AlertType::Typosquatting(details),
                    format!(
                        "Potential typosquatting: {} similar to {}",
                        candidate.suspicious_name, candidate.popular_name
                    ),
                    format!(
                        "The package '{}' has a name very similar to the popular package '{}'. \
                         This may indicate a typosquatting attack where malicious actors create \
                         packages with names similar to popular ones to trick developers.",
                        candidate.suspicious_name, candidate.popular_name
                    ),
                ))
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_levenshtein_detection() {
        let detector = TyposquattingDetector::new();

        // Test common typosquat: lodash -> lodahs (transposition)
        let candidates = detector.check(PackageEcosystem::Npm, "lodahs");
        assert!(!candidates.is_empty());
        assert_eq!(candidates[0].popular_name, "lodash");

        // Test: reqeusts (typo of requests)
        let candidates = detector.check(PackageEcosystem::PyPi, "reqeusts");
        assert!(!candidates.is_empty());
        assert_eq!(candidates[0].popular_name, "requests");
    }

    #[test]
    fn test_combosquatting() {
        let detector = TyposquattingDetector::new();

        // Test: extra-fs vs fs-extra
        let candidates = detector.check(PackageEcosystem::Npm, "extra-fs");
        assert!(!candidates.is_empty());
        let combosquat = candidates
            .iter()
            .find(|c| c.detection_method == TyposquattingMethod::Combosquatting);
        assert!(combosquat.is_some());
    }

    #[test]
    fn test_no_false_positives() {
        let detector = TyposquattingDetector::new();

        // Exact match should not be flagged
        let candidates = detector.check(PackageEcosystem::Npm, "lodash");
        assert!(candidates.is_empty());

        // Completely different name should not be flagged
        let candidates = detector.check(PackageEcosystem::Npm, "my-unique-package-name");
        assert!(candidates.is_empty());
    }
}
