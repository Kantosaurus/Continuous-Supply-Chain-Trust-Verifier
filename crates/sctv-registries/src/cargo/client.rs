//! Crates.io registry client implementation.

use async_trait::async_trait;
use bytes::Bytes;
use reqwest::Client;
use sctv_core::{
    normalize_package_name, Package, PackageChecksums, PackageDependency, PackageEcosystem,
    PackageId, PackageVersion,
};
use semver::Version;
use std::sync::Arc;
use std::time::Duration;
use url::Url;

use super::models::*;
use crate::{
    retry_http, PackageMetadata, RegistryCache, RegistryClient, RegistryError, RegistryResult,
    RetryConfig, VersionMetadata,
};

/// Crates.io registry client with caching.
pub struct CargoClient {
    http: Client,
    base_url: Url,
    static_url: Url,
    cache: Arc<RegistryCache>,
}

impl CargoClient {
    /// Default crates.io API URL.
    pub const DEFAULT_REGISTRY: &'static str = "https://crates.io";

    /// Static download URL for crate files.
    pub const STATIC_URL: &'static str = "https://static.crates.io";

    /// Creates a new cargo client with default settings.
    #[must_use]
    pub fn new() -> Self {
        Self::with_config(
            Self::DEFAULT_REGISTRY,
            Self::STATIC_URL,
            Arc::new(RegistryCache::new()),
        )
    }

    /// Creates a client with custom URLs and cache.
    #[must_use]
    pub fn with_config(api_url: &str, static_url: &str, cache: Arc<RegistryCache>) -> Self {
        let http = Client::builder()
            .timeout(Duration::from_secs(30))
            .gzip(true)
            // crates.io requires a user agent
            .user_agent("sctv-registry-client/0.1.0 (supply-chain-trust-verifier)")
            .build()
            .expect("Failed to create HTTP client");

        Self {
            http,
            base_url: Url::parse(api_url).expect("Invalid API URL"),
            static_url: Url::parse(static_url).expect("Invalid static URL"),
            cache,
        }
    }

    /// Fetches full crate metadata.
    async fn fetch_crate(&self, name: &str) -> RegistryResult<CrateResponse> {
        let url = self
            .base_url
            .join(&format!("/api/v1/crates/{}", name))
            .map_err(|e| RegistryError::Parse(e.to_string()))?;

        tracing::debug!("Fetching crate {} from {}", name, url);

        let response = retry_http(&RetryConfig::default(), || self.http.get(url.clone()).send()).await?;

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

    /// Fetches dependencies for a specific version.
    async fn fetch_dependencies(
        &self,
        name: &str,
        version: &str,
    ) -> RegistryResult<DependenciesResponse> {
        let url = self
            .base_url
            .join(&format!("/api/v1/crates/{}/{}/dependencies", name, version))
            .map_err(|e| RegistryError::Parse(e.to_string()))?;

        tracing::debug!("Fetching dependencies for {}@{} from {}", name, version, url);

        let response = retry_http(&RetryConfig::default(), || self.http.get(url.clone()).send()).await?;

        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Err(RegistryError::VersionNotFound(
                name.to_string(),
                version.to_string(),
            ));
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

    /// Fetches crate owners (maintainers).
    async fn fetch_owners(&self, name: &str) -> RegistryResult<Vec<String>> {
        let url = self
            .base_url
            .join(&format!("/api/v1/crates/{}/owners", name))
            .map_err(|e| RegistryError::Parse(e.to_string()))?;

        let response = retry_http(&RetryConfig::default(), || self.http.get(url.clone()).send()).await?;

        if !response.status().is_success() {
            // Non-fatal - return empty list
            return Ok(Vec::new());
        }

        let owners: OwnersResponse = response
            .json()
            .await
            .map_err(|e| RegistryError::Parse(e.to_string()))?;

        Ok(owners.users.into_iter().map(|o| o.login).collect())
    }

    /// Builds the download URL for a crate version.
    fn build_download_url(&self, name: &str, version: &str) -> RegistryResult<Url> {
        // Format: https://static.crates.io/crates/{name}/{name}-{version}.crate
        self.static_url
            .join(&format!("/crates/{}/{}-{}.crate", name, name, version))
            .map_err(|e| RegistryError::Parse(e.to_string()))
    }
}

impl Default for CargoClient {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl RegistryClient for CargoClient {
    fn ecosystem(&self) -> PackageEcosystem {
        PackageEcosystem::Cargo
    }

    fn base_url(&self) -> &Url {
        &self.base_url
    }

    async fn get_package(&self, name: &str) -> RegistryResult<PackageMetadata> {
        // Check cache first
        if let Some(cached) = self.cache.get_package(PackageEcosystem::Cargo, name) {
            tracing::debug!("Cache hit for crate {}", name);
            return Ok(cached);
        }

        let crate_data = self.fetch_crate(name).await?;
        let owners = self.fetch_owners(name).await.unwrap_or_default();

        let versions: Vec<String> = crate_data.versions.iter().map(|v| v.num.clone()).collect();

        // Find latest stable or latest overall
        let latest = crate_data
            .krate
            .max_stable_version
            .clone()
            .or_else(|| Some(crate_data.krate.max_version.clone()));

        // Parse timestamps
        let first_published = chrono::DateTime::parse_from_rfc3339(&crate_data.krate.created_at)
            .ok()
            .map(|dt| dt.with_timezone(&chrono::Utc));

        let last_updated = chrono::DateTime::parse_from_rfc3339(&crate_data.krate.updated_at)
            .ok()
            .map(|dt| dt.with_timezone(&chrono::Utc));

        // Parse URLs
        let homepage = crate_data
            .krate
            .homepage
            .as_ref()
            .and_then(|h| Url::parse(h).ok());

        let repository = crate_data
            .krate
            .repository
            .as_ref()
            .and_then(|r| Url::parse(r).ok());

        // Determine popularity (top 1000 by downloads is considered popular)
        let is_popular = crate_data.krate.downloads > 1_000_000;

        let package = Package {
            id: PackageId::new(),
            ecosystem: PackageEcosystem::Cargo,
            name: crate_data.krate.name.clone(),
            normalized_name: normalize_package_name(&crate_data.krate.name),
            description: crate_data.krate.description,
            homepage,
            repository,
            popularity_rank: None,
            is_popular,
            maintainers: owners,
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
            .set_package(PackageEcosystem::Cargo, name, metadata.clone());

        Ok(metadata)
    }

    async fn get_version(&self, name: &str, version: &str) -> RegistryResult<VersionMetadata> {
        // Check cache first
        if let Some(cached) = self.cache.get_version(PackageEcosystem::Cargo, name, version) {
            tracing::debug!("Cache hit for {}@{}", name, version);
            return Ok(cached);
        }

        // Fetch crate data to get version info
        let crate_data = self.fetch_crate(name).await?;

        let version_data = crate_data
            .versions
            .iter()
            .find(|v| v.num == version)
            .ok_or_else(|| {
                RegistryError::VersionNotFound(name.to_string(), version.to_string())
            })?;

        // Fetch dependencies
        let deps_result = self.fetch_dependencies(name, version).await;
        let dependencies = match deps_result {
            Ok(deps) => deps
                .dependencies
                .into_iter()
                .map(|d| PackageDependency {
                    name: d.crate_id,
                    version_constraint: d.req,
                    is_optional: d.optional,
                    is_dev: d.kind == "dev",
                })
                .collect(),
            Err(_) => Vec::new(),
        };

        let parsed_version = Version::parse(&version_data.num)
            .map_err(|e| RegistryError::Parse(format!("Invalid version: {e}")))?;

        let download_url = self.build_download_url(name, version)?;

        let checksums = PackageChecksums {
            sha1: None,
            sha256: Some(version_data.checksum.clone()),
            sha512: None,
            integrity: None,
        };

        let published_at = chrono::DateTime::parse_from_rfc3339(&version_data.created_at)
            .ok()
            .map(|dt| dt.with_timezone(&chrono::Utc));

        let package_version = PackageVersion {
            package_id: PackageId::new(),
            version: parsed_version,
            published_at,
            yanked: version_data.yanked,
            deprecated: version_data.yanked,
            deprecation_message: if version_data.yanked {
                Some("This version has been yanked".to_string())
            } else {
                None
            },
            checksums,
            download_url: Some(download_url.clone()),
            size_bytes: version_data.crate_size,
            attestations: Vec::new(),
            dependencies,
            cached_at: chrono::Utc::now(),
        };

        let metadata = VersionMetadata {
            version: package_version,
            download_url: Some(download_url),
        };

        self.cache
            .set_version(PackageEcosystem::Cargo, name, version, metadata.clone());

        Ok(metadata)
    }

    async fn download_package(&self, name: &str, version: &str) -> RegistryResult<Bytes> {
        let url = self.build_download_url(name, version)?;

        tracing::debug!("Downloading {}@{} from {}", name, version, url);

        let response = retry_http(&RetryConfig::default(), || self.http.get(url.clone()).send()).await?;

        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Err(RegistryError::VersionNotFound(
                name.to_string(),
                version.to_string(),
            ));
        }

        if !response.status().is_success() {
            return Err(RegistryError::Unavailable(format!(
                "Download failed with status {}",
                response.status()
            )));
        }

        Ok(response.bytes().await?)
    }

    async fn list_popular(&self, limit: usize) -> RegistryResult<Vec<String>> {
        // Well-known popular Rust crates
        let popular = vec![
            "serde",
            "tokio",
            "rand",
            "log",
            "clap",
            "regex",
            "chrono",
            "reqwest",
            "serde_json",
            "anyhow",
            "thiserror",
            "tracing",
            "futures",
            "async-trait",
            "hyper",
            "axum",
            "sqlx",
            "diesel",
            "rocket",
            "actix-web",
            "syn",
            "quote",
            "proc-macro2",
            "lazy_static",
            "once_cell",
            "bytes",
            "uuid",
            "itertools",
            "rayon",
            "crossbeam",
            "parking_lot",
            "dashmap",
            "indexmap",
            "hashbrown",
            "smallvec",
            "arrayvec",
            "bitflags",
            "num",
            "url",
            "http",
            "tower",
            "tonic",
            "prost",
            "rustls",
            "ring",
            "sha2",
            "base64",
            "hex",
            "toml",
            "yaml-rust",
        ];

        Ok(popular.into_iter().take(limit).map(String::from).collect())
    }

    async fn package_exists(&self, name: &str) -> RegistryResult<bool> {
        match self.fetch_crate(name).await {
            Ok(_) => Ok(true),
            Err(RegistryError::PackageNotFound(_)) => Ok(false),
            Err(e) => Err(e),
        }
    }

    async fn get_download_url(&self, name: &str, version: &str) -> RegistryResult<Url> {
        self.build_download_url(name, version)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_download_url() {
        let client = CargoClient::new();
        let url = client.build_download_url("serde", "1.0.0").unwrap();
        assert_eq!(
            url.as_str(),
            "https://static.crates.io/crates/serde/serde-1.0.0.crate"
        );
    }
}
