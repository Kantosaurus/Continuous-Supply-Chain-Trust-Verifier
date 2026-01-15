//! npm registry client implementation with full functionality.

use async_trait::async_trait;
use bytes::Bytes;
use reqwest::Client;
use sctv_core::{
    Package, PackageChecksums, PackageDependency, PackageEcosystem, PackageId, PackageVersion,
};
use semver::Version;
use sha2::{Digest, Sha256, Sha512};
use std::sync::Arc;
use std::time::Duration;
use url::Url;

use super::models::*;
use crate::{
    PackageMetadata, RegistryCache, RegistryClient, RegistryError, RegistryResult, VersionMetadata,
};

/// npm registry client with caching and hash verification.
pub struct NpmClient {
    http: Client,
    base_url: Url,
    cache: Arc<RegistryCache>,
}

impl NpmClient {
    /// Default npm registry URL.
    pub const DEFAULT_REGISTRY: &'static str = "https://registry.npmjs.org";

    /// Creates a new npm client with default settings.
    #[must_use]
    pub fn new() -> Self {
        Self::with_config(Self::DEFAULT_REGISTRY, Arc::new(RegistryCache::new()))
    }

    /// Creates a client with custom registry URL and cache.
    #[must_use]
    pub fn with_config(registry_url: &str, cache: Arc<RegistryCache>) -> Self {
        let http = Client::builder()
            .timeout(Duration::from_secs(30))
            .gzip(true)
            .user_agent("sctv-registry-client/0.1.0")
            .build()
            .expect("Failed to create HTTP client");

        Self {
            http,
            base_url: Url::parse(registry_url).expect("Invalid registry URL"),
            cache,
        }
    }

    /// Fetches abbreviated package metadata (faster, smaller response).
    async fn fetch_abbreviated(&self, name: &str) -> RegistryResult<NpmAbbreviatedPackage> {
        let url = self
            .base_url
            .join(&encode_package_name(name))
            .map_err(|e| RegistryError::Parse(e.to_string()))?;

        tracing::debug!("Fetching abbreviated metadata for {} from {}", name, url);

        let response = self
            .http
            .get(url)
            .header("Accept", "application/vnd.npm.install-v1+json")
            .send()
            .await?;

        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Err(RegistryError::PackageNotFound(name.to_string()));
        }

        if response.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
            return Err(RegistryError::RateLimited);
        }

        if !response.status().is_success() {
            return Err(RegistryError::Unavailable(format!(
                "Registry returned status {}",
                response.status()
            )));
        }

        response
            .json()
            .await
            .map_err(|e| RegistryError::Parse(e.to_string()))
    }

    /// Fetches full package document with all versions.
    async fn fetch_full(&self, name: &str) -> RegistryResult<NpmPackageDocument> {
        let url = self
            .base_url
            .join(&encode_package_name(name))
            .map_err(|e| RegistryError::Parse(e.to_string()))?;

        tracing::debug!("Fetching full metadata for {} from {}", name, url);

        let response = self.http.get(url).send().await?;

        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Err(RegistryError::PackageNotFound(name.to_string()));
        }

        if response.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
            return Err(RegistryError::RateLimited);
        }

        if !response.status().is_success() {
            return Err(RegistryError::Unavailable(format!(
                "Registry returned status {}",
                response.status()
            )));
        }

        response
            .json()
            .await
            .map_err(|e| RegistryError::Parse(e.to_string()))
    }

    /// Fetches metadata for a specific version.
    async fn fetch_version(&self, name: &str, version: &str) -> RegistryResult<NpmVersionResponse> {
        let url = self
            .base_url
            .join(&format!("{}/{}", encode_package_name(name), version))
            .map_err(|e| RegistryError::Parse(e.to_string()))?;

        tracing::debug!("Fetching version {}@{} from {}", name, version, url);

        let response = self.http.get(url).send().await?;

        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Err(RegistryError::VersionNotFound(
                name.to_string(),
                version.to_string(),
            ));
        }

        if response.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
            return Err(RegistryError::RateLimited);
        }

        response
            .json()
            .await
            .map_err(|e| RegistryError::Parse(e.to_string()))
    }

    /// Verifies the integrity of downloaded package bytes.
    pub fn verify_integrity(&self, bytes: &Bytes, expected: &PackageChecksums) -> IntegrityResult {
        let mut result = IntegrityResult {
            sha256_match: None,
            sha512_match: None,
            integrity_match: None,
            computed_sha256: None,
            computed_sha512: None,
        };

        // Compute SHA-256
        let sha256_hash = {
            let mut hasher = Sha256::new();
            hasher.update(bytes);
            hex::encode(hasher.finalize())
        };
        result.computed_sha256 = Some(sha256_hash.clone());

        // Compute SHA-512
        let sha512_hash = {
            let mut hasher = Sha512::new();
            hasher.update(bytes);
            hex::encode(hasher.finalize())
        };
        result.computed_sha512 = Some(sha512_hash.clone());

        // Check against expected values
        if let Some(expected_sha256) = &expected.sha256 {
            result.sha256_match = Some(expected_sha256.to_lowercase() == sha256_hash);
        }

        if let Some(expected_sha512) = &expected.sha512 {
            result.sha512_match = Some(expected_sha512.to_lowercase() == sha512_hash);
        }

        // Check npm integrity field (base64 encoded sha512)
        if let Some(integrity) = &expected.integrity {
            if let Some(hash) = parse_integrity_hash(integrity) {
                let computed_base64 = {
                    let mut hasher = Sha512::new();
                    hasher.update(bytes);
                    base64::Engine::encode(
                        &base64::engine::general_purpose::STANDARD,
                        hasher.finalize(),
                    )
                };
                result.integrity_match = Some(hash == computed_base64);
            }
        }

        result
    }

    /// Converts npm dependencies map to package dependencies.
    fn parse_dependencies(deps: Option<&std::collections::HashMap<String, String>>) -> Vec<PackageDependency> {
        deps.map(|d| {
            d.iter()
                .map(|(name, constraint)| PackageDependency {
                    name: name.clone(),
                    version_constraint: constraint.clone(),
                    is_optional: false,
                    is_dev: false,
                })
                .collect()
        })
        .unwrap_or_default()
    }
}

impl Default for NpmClient {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl RegistryClient for NpmClient {
    fn ecosystem(&self) -> PackageEcosystem {
        PackageEcosystem::Npm
    }

    fn base_url(&self) -> &Url {
        &self.base_url
    }

    async fn get_package(&self, name: &str) -> RegistryResult<PackageMetadata> {
        // Check cache first
        if let Some(cached) = self.cache.get_package(PackageEcosystem::Npm, name) {
            tracing::debug!("Cache hit for package {}", name);
            return Ok(cached);
        }

        let npm_pkg = self.fetch_full(name).await?;

        let versions: Vec<String> = npm_pkg.versions.keys().cloned().collect();
        let latest = npm_pkg.dist_tags.get("latest").cloned();

        // Parse repository URL
        let repository = npm_pkg.repository.as_ref().and_then(|r| {
            let url_str = r.url();
            // Clean up git:// URLs
            let cleaned = url_str
                .trim_start_matches("git+")
                .trim_end_matches(".git")
                .replace("git://", "https://")
                .replace("ssh://git@", "https://");
            Url::parse(&cleaned).ok()
        });

        // Parse homepage
        let homepage = npm_pkg.homepage.as_ref().and_then(|h| Url::parse(h).ok());

        // Extract maintainers
        let maintainers = npm_pkg
            .maintainers
            .as_ref()
            .map(|m| m.iter().filter_map(|m| m.name.clone()).collect())
            .unwrap_or_default();

        // Parse first published time
        let first_published = npm_pkg
            .time
            .as_ref()
            .and_then(|t| t.get("created"))
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&chrono::Utc));

        // Parse last modified time
        let last_updated = npm_pkg
            .time
            .as_ref()
            .and_then(|t| t.get("modified"))
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&chrono::Utc));

        let package = Package {
            id: PackageId::new(),
            ecosystem: PackageEcosystem::Npm,
            name: npm_pkg.name.clone(),
            normalized_name: sctv_core::normalize_package_name(&npm_pkg.name),
            description: npm_pkg.description,
            homepage,
            repository,
            popularity_rank: None,
            is_popular: false,
            maintainers,
            first_published,
            last_updated,
            cached_at: chrono::Utc::now(),
        };

        let metadata = PackageMetadata {
            package,
            available_versions: versions,
            latest_version: latest,
        };

        self.cache
            .set_package(PackageEcosystem::Npm, name, metadata.clone());

        Ok(metadata)
    }

    async fn get_version(&self, name: &str, version: &str) -> RegistryResult<VersionMetadata> {
        // Check cache first
        if let Some(cached) = self.cache.get_version(PackageEcosystem::Npm, name, version) {
            tracing::debug!("Cache hit for {}@{}", name, version);
            return Ok(cached);
        }

        let npm_version = self.fetch_version(name, version).await?;

        let parsed_version = Version::parse(&npm_version.version)
            .map_err(|e| RegistryError::Parse(format!("Invalid version: {e}")))?;

        let checksums = PackageChecksums {
            sha256: None, // npm doesn't provide SHA256 by default
            sha512: npm_version.dist.shasum.clone(),
            integrity: npm_version.dist.integrity.clone(),
        };

        let download_url = npm_version
            .dist
            .tarball
            .as_ref()
            .and_then(|t| Url::parse(t).ok());

        // Parse dependencies
        let mut dependencies = Self::parse_dependencies(npm_version.dependencies.as_ref());

        // Add dev dependencies
        let dev_deps = npm_version.dev_dependencies.as_ref().map(|d| {
            d.iter()
                .map(|(name, constraint)| PackageDependency {
                    name: name.clone(),
                    version_constraint: constraint.clone(),
                    is_optional: false,
                    is_dev: true,
                })
                .collect::<Vec<_>>()
        });
        if let Some(dev) = dev_deps {
            dependencies.extend(dev);
        }

        let package_version = PackageVersion {
            package_id: PackageId::new(),
            version: parsed_version,
            published_at: None, // Would need to fetch full package to get this
            yanked: npm_version.deprecated.is_some(),
            deprecated: npm_version.deprecated.is_some(),
            deprecation_message: npm_version.deprecated,
            checksums,
            download_url: download_url.clone(),
            size_bytes: npm_version.dist.unpacked_size,
            attestations: Vec::new(),
            dependencies,
            cached_at: chrono::Utc::now(),
        };

        let metadata = VersionMetadata {
            version: package_version,
            download_url,
        };

        self.cache
            .set_version(PackageEcosystem::Npm, name, version, metadata.clone());

        Ok(metadata)
    }

    async fn download_package(&self, name: &str, version: &str) -> RegistryResult<Bytes> {
        let url = self.get_download_url(name, version).await?;

        tracing::debug!("Downloading {}@{} from {}", name, version, url);

        let response = self.http.get(url).send().await?;

        if !response.status().is_success() {
            return Err(RegistryError::Unavailable(format!(
                "Download failed with status {}",
                response.status()
            )));
        }

        Ok(response.bytes().await?)
    }

    async fn list_popular(&self, limit: usize) -> RegistryResult<Vec<String>> {
        // npm doesn't have a direct API for popular packages.
        // Return a curated list of well-known packages.
        let popular = vec![
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
            "rxjs",
            "ramda",
            "inquirer",
            "yargs",
            "glob",
            "minimist",
            "semver",
            "dotenv",
            "cross-env",
            "rimraf",
            "mkdirp",
            "node-fetch",
            "request",
            "body-parser",
            "cors",
            "mongoose",
            "mysql",
            "pg",
            "redis",
            "socket.io",
            "ws",
            "nodemon",
            "pm2",
            "babel-core",
            "@babel/core",
            "postcss",
            "autoprefixer",
            "tailwindcss",
            "styled-components",
        ];

        Ok(popular.into_iter().take(limit).map(String::from).collect())
    }

    async fn package_exists(&self, name: &str) -> RegistryResult<bool> {
        match self.fetch_abbreviated(name).await {
            Ok(_) => Ok(true),
            Err(RegistryError::PackageNotFound(_)) => Ok(false),
            Err(e) => Err(e),
        }
    }

    async fn get_download_url(&self, name: &str, version: &str) -> RegistryResult<Url> {
        let version_meta = self.get_version(name, version).await?;

        version_meta.download_url.ok_or_else(|| {
            RegistryError::Unavailable(format!("No download URL for {name}@{version}"))
        })
    }
}

/// Result of integrity verification.
#[derive(Debug, Clone)]
pub struct IntegrityResult {
    pub sha256_match: Option<bool>,
    pub sha512_match: Option<bool>,
    pub integrity_match: Option<bool>,
    pub computed_sha256: Option<String>,
    pub computed_sha512: Option<String>,
}

impl IntegrityResult {
    /// Returns true if all available checks passed.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        let checks = [self.sha256_match, self.sha512_match, self.integrity_match];
        checks
            .iter()
            .filter_map(|c| *c)
            .all(|matched| matched)
    }

    /// Returns true if any check failed.
    #[must_use]
    pub fn has_failure(&self) -> bool {
        let checks = [self.sha256_match, self.sha512_match, self.integrity_match];
        checks.iter().filter_map(|c| *c).any(|matched| !matched)
    }
}

/// Encodes a package name for use in URLs (handles scoped packages).
fn encode_package_name(name: &str) -> String {
    if name.starts_with('@') {
        // Scoped packages need URL encoding: @scope/package -> @scope%2Fpackage
        name.replace('/', "%2F")
    } else {
        name.to_string()
    }
}

/// Parses the hash from an npm integrity string (e.g., "sha512-abc123...").
fn parse_integrity_hash(integrity: &str) -> Option<String> {
    let parts: Vec<&str> = integrity.split('-').collect();
    if parts.len() == 2 {
        Some(parts[1].to_string())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_package_name() {
        assert_eq!(encode_package_name("lodash"), "lodash");
        assert_eq!(encode_package_name("@babel/core"), "@babel%2Fcore");
        assert_eq!(encode_package_name("@types/node"), "@types%2Fnode");
    }

    #[test]
    fn test_parse_integrity_hash() {
        let integrity = "sha512-abc123def456";
        assert_eq!(parse_integrity_hash(integrity), Some("abc123def456".to_string()));

        let invalid = "invalid";
        assert_eq!(parse_integrity_hash(invalid), None);
    }
}
