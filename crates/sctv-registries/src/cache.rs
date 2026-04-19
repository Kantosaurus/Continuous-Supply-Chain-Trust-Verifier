//! Registry metadata caching.

use chrono::{DateTime, Duration, Utc};
use dashmap::DashMap;
use sctv_core::PackageEcosystem;
use std::sync::Arc;

use crate::{PackageMetadata, VersionMetadata};

/// Cache entry with expiration.
#[derive(Debug, Clone)]
pub struct CacheEntry<T> {
    pub data: T,
    pub cached_at: DateTime<Utc>,
    pub ttl: Duration,
}

impl<T> CacheEntry<T> {
    /// Creates a new cache entry.
    pub fn new(data: T, ttl: Duration) -> Self {
        Self {
            data,
            cached_at: Utc::now(),
            ttl,
        }
    }

    /// Checks if this entry has expired.
    #[must_use]
    pub fn is_expired(&self) -> bool {
        Utc::now() - self.cached_at > self.ttl
    }
}

/// Key for package cache lookups.
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
struct PackageCacheKey {
    ecosystem: PackageEcosystem,
    name: String,
}

/// Key for version cache lookups.
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
struct VersionCacheKey {
    ecosystem: PackageEcosystem,
    name: String,
    version: String,
}

/// In-memory cache for registry metadata.
#[derive(Clone)]
pub struct RegistryCache {
    packages: Arc<DashMap<PackageCacheKey, CacheEntry<PackageMetadata>>>,
    versions: Arc<DashMap<VersionCacheKey, CacheEntry<VersionMetadata>>>,
    package_ttl: Duration,
    version_ttl: Duration,
}

impl Default for RegistryCache {
    fn default() -> Self {
        Self::new()
    }
}

impl RegistryCache {
    /// Default TTL for package metadata (1 hour).
    pub const DEFAULT_PACKAGE_TTL: Duration = Duration::hours(1);

    /// Default TTL for version metadata (24 hours).
    pub const DEFAULT_VERSION_TTL: Duration = Duration::hours(24);

    /// Creates a new registry cache with default TTLs.
    #[must_use]
    pub fn new() -> Self {
        Self {
            packages: Arc::new(DashMap::new()),
            versions: Arc::new(DashMap::new()),
            package_ttl: Self::DEFAULT_PACKAGE_TTL,
            version_ttl: Self::DEFAULT_VERSION_TTL,
        }
    }

    /// Creates a cache with custom TTLs.
    #[must_use]
    pub fn with_ttl(package_ttl: Duration, version_ttl: Duration) -> Self {
        Self {
            packages: Arc::new(DashMap::new()),
            versions: Arc::new(DashMap::new()),
            package_ttl,
            version_ttl,
        }
    }

    /// Gets a cached package if not expired.
    #[must_use]
    pub fn get_package(&self, ecosystem: PackageEcosystem, name: &str) -> Option<PackageMetadata> {
        let key = PackageCacheKey {
            ecosystem,
            name: name.to_string(),
        };

        self.packages.get(&key).and_then(|entry| {
            if entry.is_expired() {
                None
            } else {
                Some(entry.data.clone())
            }
        })
    }

    /// Caches package metadata.
    pub fn set_package(&self, ecosystem: PackageEcosystem, name: &str, metadata: PackageMetadata) {
        let key = PackageCacheKey {
            ecosystem,
            name: name.to_string(),
        };
        let entry = CacheEntry::new(metadata, self.package_ttl);
        self.packages.insert(key, entry);
    }

    /// Gets a cached version if not expired.
    #[must_use]
    pub fn get_version(
        &self,
        ecosystem: PackageEcosystem,
        name: &str,
        version: &str,
    ) -> Option<VersionMetadata> {
        let key = VersionCacheKey {
            ecosystem,
            name: name.to_string(),
            version: version.to_string(),
        };

        self.versions.get(&key).and_then(|entry| {
            if entry.is_expired() {
                None
            } else {
                Some(entry.data.clone())
            }
        })
    }

    /// Caches version metadata.
    pub fn set_version(
        &self,
        ecosystem: PackageEcosystem,
        name: &str,
        version: &str,
        metadata: VersionMetadata,
    ) {
        let key = VersionCacheKey {
            ecosystem,
            name: name.to_string(),
            version: version.to_string(),
        };
        let entry = CacheEntry::new(metadata, self.version_ttl);
        self.versions.insert(key, entry);
    }

    /// Invalidates all cached entries for a package.
    pub fn invalidate_package(&self, ecosystem: PackageEcosystem, name: &str) {
        let key = PackageCacheKey {
            ecosystem,
            name: name.to_string(),
        };
        self.packages.remove(&key);

        // Also remove all version entries for this package
        self.versions
            .retain(|k, _| !(k.ecosystem == ecosystem && k.name == name));
    }

    /// Clears all expired entries.
    pub fn cleanup(&self) {
        self.packages.retain(|_, entry| !entry.is_expired());
        self.versions.retain(|_, entry| !entry.is_expired());
    }

    /// Returns the number of cached packages.
    #[must_use]
    pub fn package_count(&self) -> usize {
        self.packages.len()
    }

    /// Returns the number of cached versions.
    #[must_use]
    pub fn version_count(&self) -> usize {
        self.versions.len()
    }
}
