//! Package domain model representing cached package metadata.

use chrono::{DateTime, Utc};
use semver::Version;
use serde::{Deserialize, Serialize};
use url::Url;
use uuid::Uuid;

use super::{Attestation, PackageEcosystem};

/// Unique identifier for a package.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PackageId(pub Uuid);

impl PackageId {
    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for PackageId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for PackageId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Cached metadata for a package from a registry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Package {
    pub id: PackageId,
    pub ecosystem: PackageEcosystem,
    pub name: String,
    pub normalized_name: String,
    pub description: Option<String>,
    pub homepage: Option<Url>,
    pub repository: Option<Url>,
    pub popularity_rank: Option<u32>,
    pub is_popular: bool,
    pub maintainers: Vec<String>,
    pub first_published: Option<DateTime<Utc>>,
    pub last_updated: Option<DateTime<Utc>>,
    pub cached_at: DateTime<Utc>,
}

impl Package {
    /// Creates a new package with the given name and ecosystem.
    #[must_use]
    pub fn new(ecosystem: PackageEcosystem, name: String) -> Self {
        let normalized = normalize_package_name(&name);
        Self {
            id: PackageId::new(),
            ecosystem,
            name,
            normalized_name: normalized,
            description: None,
            homepage: None,
            repository: None,
            popularity_rank: None,
            is_popular: false,
            maintainers: Vec::new(),
            first_published: None,
            last_updated: None,
            cached_at: Utc::now(),
        }
    }

    /// Checks if the cached data is stale and should be refreshed.
    #[must_use]
    pub fn is_stale(&self, max_age: chrono::Duration) -> bool {
        Utc::now() - self.cached_at > max_age
    }

    /// Returns the age of the package since first publication.
    #[must_use]
    pub fn age_days(&self) -> Option<i64> {
        self.first_published
            .map(|published| (Utc::now() - published).num_days())
    }
}

/// Metadata for a specific version of a package.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageVersion {
    pub package_id: PackageId,
    pub version: Version,
    pub published_at: Option<DateTime<Utc>>,
    pub yanked: bool,
    pub deprecated: bool,
    pub deprecation_message: Option<String>,
    pub checksums: PackageChecksums,
    pub download_url: Option<Url>,
    pub size_bytes: Option<u64>,
    pub attestations: Vec<Attestation>,
    pub dependencies: Vec<PackageDependency>,
    pub cached_at: DateTime<Utc>,
}

impl PackageVersion {
    /// Creates a new package version.
    #[must_use]
    pub fn new(package_id: PackageId, version: Version) -> Self {
        Self {
            package_id,
            version,
            published_at: None,
            yanked: false,
            deprecated: false,
            deprecation_message: None,
            checksums: PackageChecksums::default(),
            download_url: None,
            size_bytes: None,
            attestations: Vec::new(),
            dependencies: Vec::new(),
            cached_at: Utc::now(),
        }
    }

    /// Returns the age in days since this version was published.
    #[must_use]
    pub fn age_days(&self) -> Option<i64> {
        self.published_at
            .map(|published| (Utc::now() - published).num_days())
    }

    /// Checks if this version is considered new (published within N days).
    #[must_use]
    pub fn is_new(&self, threshold_days: i64) -> bool {
        self.age_days().map_or(false, |age| age < threshold_days)
    }
}

/// Checksums for integrity verification.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PackageChecksums {
    pub sha256: Option<String>,
    pub sha512: Option<String>,
    pub integrity: Option<String>,
}

impl PackageChecksums {
    /// Checks if any checksum is available.
    #[must_use]
    pub fn has_any(&self) -> bool {
        self.sha256.is_some() || self.sha512.is_some() || self.integrity.is_some()
    }
}

/// A dependency declared by a package.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageDependency {
    pub name: String,
    pub version_constraint: String,
    pub is_optional: bool,
    pub is_dev: bool,
}

/// Normalizes a package name for comparison.
/// Handles different naming conventions across ecosystems.
#[must_use]
pub fn normalize_package_name(name: &str) -> String {
    name.to_lowercase()
        .replace('_', "-")
        .replace('.', "-")
        .trim_start_matches('@')
        .replace('/', "-")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_package_name() {
        assert_eq!(normalize_package_name("My_Package"), "my-package");
        assert_eq!(normalize_package_name("my.package"), "my-package");
        assert_eq!(normalize_package_name("@scope/package"), "scope-package");
        assert_eq!(normalize_package_name("UPPERCASE"), "uppercase");
    }
}
