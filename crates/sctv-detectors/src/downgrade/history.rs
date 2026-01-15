//! Version history tracking for downgrade detection.
//!
//! Maintains a record of package versions across scans to detect
//! unexpected version changes.

use chrono::{DateTime, Utc};
use parking_lot::RwLock;
use sctv_core::{PackageEcosystem, ProjectId};
use semver::Version;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Key for identifying a package in version history.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct VersionHistoryKey {
    pub project_id: ProjectId,
    pub ecosystem: PackageEcosystem,
    pub package_name: String,
}

impl VersionHistoryKey {
    /// Creates a new version history key.
    #[must_use]
    pub fn new(project_id: ProjectId, ecosystem: PackageEcosystem, package_name: String) -> Self {
        Self {
            project_id,
            ecosystem,
            package_name,
        }
    }
}

/// A single version record in the history.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionRecord {
    /// The version that was recorded.
    pub version: Version,
    /// When this version was first seen.
    pub first_seen_at: DateTime<Utc>,
    /// When this version was last seen.
    pub last_seen_at: DateTime<Utc>,
    /// Number of times this version has been seen.
    pub scan_count: u32,
    /// The scan ID where this was first recorded.
    pub first_scan_id: Option<String>,
}

impl VersionRecord {
    /// Creates a new version record.
    #[must_use]
    pub fn new(version: Version) -> Self {
        let now = Utc::now();
        Self {
            version,
            first_seen_at: now,
            last_seen_at: now,
            scan_count: 1,
            first_scan_id: None,
        }
    }

    /// Updates the record with a new scan.
    pub fn update(&mut self) {
        self.last_seen_at = Utc::now();
        self.scan_count += 1;
    }
}

/// History of versions for a specific package.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PackageVersionHistory {
    /// All versions seen for this package, ordered by first seen time.
    pub versions: Vec<VersionRecord>,
    /// The highest version ever seen.
    pub max_version: Option<Version>,
    /// The current (most recently seen) version.
    pub current_version: Option<Version>,
}

impl PackageVersionHistory {
    /// Creates a new empty history.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Records a new version observation.
    pub fn record(&mut self, version: &Version) {
        // Check if we've seen this version before
        if let Some(record) = self.versions.iter_mut().find(|r| &r.version == version) {
            record.update();
        } else {
            self.versions.push(VersionRecord::new(version.clone()));
        }

        // Update max version
        if self.max_version.as_ref().map_or(true, |max| version > max) {
            self.max_version = Some(version.clone());
        }

        // Update current version
        self.current_version = Some(version.clone());
    }

    /// Gets the latest (most recently seen) version.
    #[must_use]
    pub fn latest(&self) -> Option<&Version> {
        self.current_version.as_ref()
    }

    /// Gets the highest version ever recorded.
    #[must_use]
    pub fn max(&self) -> Option<&Version> {
        self.max_version.as_ref()
    }

    /// Checks if a version would be a downgrade from the max.
    #[must_use]
    pub fn is_downgrade_from_max(&self, version: &Version) -> bool {
        self.max_version.as_ref().map_or(false, |max| version < max)
    }

    /// Checks if a version would be a downgrade from the current.
    #[must_use]
    pub fn is_downgrade_from_current(&self, version: &Version) -> bool {
        self.current_version
            .as_ref()
            .map_or(false, |current| version < current)
    }

    /// Gets the version history in chronological order.
    #[must_use]
    pub fn chronological_history(&self) -> Vec<&VersionRecord> {
        let mut records: Vec<_> = self.versions.iter().collect();
        records.sort_by(|a, b| a.first_seen_at.cmp(&b.first_seen_at));
        records
    }

    /// Gets the version history sorted by version number.
    #[must_use]
    pub fn version_sorted_history(&self) -> Vec<&VersionRecord> {
        let mut records: Vec<_> = self.versions.iter().collect();
        records.sort_by(|a, b| a.version.cmp(&b.version));
        records
    }
}

/// In-memory store for version history.
pub struct VersionHistoryStore {
    histories: RwLock<HashMap<VersionHistoryKey, PackageVersionHistory>>,
}

impl VersionHistoryStore {
    /// Creates a new empty store.
    #[must_use]
    pub fn new() -> Self {
        Self {
            histories: RwLock::new(HashMap::new()),
        }
    }

    /// Records a version for a package.
    pub fn record_version(&self, key: &VersionHistoryKey, version: &Version) {
        let mut histories = self.histories.write();
        histories
            .entry(key.clone())
            .or_insert_with(PackageVersionHistory::new)
            .record(version);
    }

    /// Gets the latest version for a package.
    #[must_use]
    pub fn get_latest_version(&self, key: &VersionHistoryKey) -> Option<Version> {
        self.histories
            .read()
            .get(key)
            .and_then(|h| h.latest().cloned())
    }

    /// Gets the maximum version ever recorded for a package.
    #[must_use]
    pub fn get_max_version(&self, key: &VersionHistoryKey) -> Option<Version> {
        self.histories.read().get(key).and_then(|h| h.max().cloned())
    }

    /// Gets the full history for a package.
    #[must_use]
    pub fn get_history(&self, key: &VersionHistoryKey) -> Option<PackageVersionHistory> {
        self.histories.read().get(key).cloned()
    }

    /// Checks if a version would be a downgrade.
    #[must_use]
    pub fn is_downgrade(&self, key: &VersionHistoryKey, version: &Version) -> bool {
        self.histories
            .read()
            .get(key)
            .map_or(false, |h| h.is_downgrade_from_current(version))
    }

    /// Clears all history.
    pub fn clear(&self) {
        self.histories.write().clear();
    }

    /// Clears history for a specific project.
    pub fn clear_project(&self, project_id: ProjectId) {
        self.histories
            .write()
            .retain(|k, _| k.project_id != project_id);
    }

    /// Gets the number of tracked packages.
    #[must_use]
    pub fn len(&self) -> usize {
        self.histories.read().len()
    }

    /// Checks if the store is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.histories.read().is_empty()
    }
}

impl Default for VersionHistoryStore {
    fn default() -> Self {
        Self::new()
    }
}

/// Trait for persisting version history.
#[async_trait::async_trait]
pub trait VersionHistoryPersistence: Send + Sync {
    /// Loads history for a project.
    async fn load(
        &self,
        project_id: ProjectId,
    ) -> Result<HashMap<VersionHistoryKey, PackageVersionHistory>, Box<dyn std::error::Error + Send + Sync>>;

    /// Saves history for a project.
    async fn save(
        &self,
        project_id: ProjectId,
        histories: &HashMap<VersionHistoryKey, PackageVersionHistory>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}

/// In-memory persistence (no actual persistence).
#[derive(Default)]
pub struct MemoryPersistence {
    data: RwLock<HashMap<ProjectId, HashMap<VersionHistoryKey, PackageVersionHistory>>>,
}

impl MemoryPersistence {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait::async_trait]
impl VersionHistoryPersistence for MemoryPersistence {
    async fn load(
        &self,
        project_id: ProjectId,
    ) -> Result<HashMap<VersionHistoryKey, PackageVersionHistory>, Box<dyn std::error::Error + Send + Sync>>
    {
        Ok(self
            .data
            .read()
            .get(&project_id)
            .cloned()
            .unwrap_or_default())
    }

    async fn save(
        &self,
        project_id: ProjectId,
        histories: &HashMap<VersionHistoryKey, PackageVersionHistory>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.data.write().insert(project_id, histories.clone());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_record_creation() {
        let version = Version::new(1, 0, 0);
        let record = VersionRecord::new(version.clone());

        assert_eq!(record.version, version);
        assert_eq!(record.scan_count, 1);
    }

    #[test]
    fn test_version_record_update() {
        let version = Version::new(1, 0, 0);
        let mut record = VersionRecord::new(version);

        record.update();
        assert_eq!(record.scan_count, 2);

        record.update();
        assert_eq!(record.scan_count, 3);
    }

    #[test]
    fn test_package_history_record() {
        let mut history = PackageVersionHistory::new();

        history.record(&Version::new(1, 0, 0));
        assert_eq!(history.latest(), Some(&Version::new(1, 0, 0)));
        assert_eq!(history.max(), Some(&Version::new(1, 0, 0)));

        history.record(&Version::new(1, 1, 0));
        assert_eq!(history.latest(), Some(&Version::new(1, 1, 0)));
        assert_eq!(history.max(), Some(&Version::new(1, 1, 0)));

        // Recording an older version updates current but not max
        history.record(&Version::new(1, 0, 5));
        assert_eq!(history.latest(), Some(&Version::new(1, 0, 5)));
        assert_eq!(history.max(), Some(&Version::new(1, 1, 0)));
    }

    #[test]
    fn test_downgrade_detection() {
        let mut history = PackageVersionHistory::new();

        history.record(&Version::new(1, 0, 0));
        history.record(&Version::new(1, 1, 0));

        assert!(history.is_downgrade_from_max(&Version::new(1, 0, 5)));
        assert!(history.is_downgrade_from_current(&Version::new(1, 0, 5)));
        assert!(!history.is_downgrade_from_max(&Version::new(1, 2, 0)));
    }

    #[test]
    fn test_history_store() {
        let store = VersionHistoryStore::new();
        let key = VersionHistoryKey::new(
            ProjectId::new(),
            PackageEcosystem::Npm,
            "test-pkg".to_string(),
        );

        assert!(store.get_latest_version(&key).is_none());

        store.record_version(&key, &Version::new(1, 0, 0));
        assert_eq!(store.get_latest_version(&key), Some(Version::new(1, 0, 0)));

        store.record_version(&key, &Version::new(1, 1, 0));
        assert_eq!(store.get_latest_version(&key), Some(Version::new(1, 1, 0)));
        assert_eq!(store.get_max_version(&key), Some(Version::new(1, 1, 0)));
    }

    #[test]
    fn test_store_is_downgrade() {
        let store = VersionHistoryStore::new();
        let key = VersionHistoryKey::new(
            ProjectId::new(),
            PackageEcosystem::Npm,
            "test-pkg".to_string(),
        );

        store.record_version(&key, &Version::new(1, 1, 0));

        assert!(store.is_downgrade(&key, &Version::new(1, 0, 0)));
        assert!(!store.is_downgrade(&key, &Version::new(1, 2, 0)));
    }

    #[test]
    fn test_store_clear() {
        let store = VersionHistoryStore::new();
        let key = VersionHistoryKey::new(
            ProjectId::new(),
            PackageEcosystem::Npm,
            "test-pkg".to_string(),
        );

        store.record_version(&key, &Version::new(1, 0, 0));
        assert!(!store.is_empty());

        store.clear();
        assert!(store.is_empty());
    }
}
