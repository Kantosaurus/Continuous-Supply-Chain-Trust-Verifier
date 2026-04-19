//! RubyGems registry client implementation.

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

/// RubyGems registry client with caching.
pub struct RubyGemsClient {
    http: Client,
    base_url: Url,
    cache: Arc<RegistryCache>,
}

impl RubyGemsClient {
    /// Default RubyGems registry URL.
    pub const DEFAULT_REGISTRY: &'static str = "https://rubygems.org";

    /// Creates a new RubyGems client with default settings.
    #[must_use]
    pub fn new() -> Self {
        Self::with_config(Self::DEFAULT_REGISTRY, Arc::new(RegistryCache::new()))
    }

    /// Creates a client with custom URL and cache.
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

    /// Fetches gem information.
    async fn fetch_gem(&self, name: &str) -> RegistryResult<GemInfo> {
        let url = self
            .base_url
            .join(&format!("/api/v1/gems/{}.json", name))
            .map_err(|e| RegistryError::Parse(e.to_string()))?;

        tracing::debug!("Fetching gem {} from {}", name, url);

        let response = retry_http(&RetryConfig::default(), || {
            self.http.get(url.clone()).send()
        })
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

    /// Fetches all versions of a gem.
    async fn fetch_versions(&self, name: &str) -> RegistryResult<Vec<VersionInfo>> {
        let url = self
            .base_url
            .join(&format!("/api/v1/versions/{}.json", name))
            .map_err(|e| RegistryError::Parse(e.to_string()))?;

        tracing::debug!("Fetching versions for {} from {}", name, url);

        let response = retry_http(&RetryConfig::default(), || {
            self.http.get(url.clone()).send()
        })
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

    /// Fetches gem owners (maintainers).
    async fn fetch_owners(&self, name: &str) -> RegistryResult<Vec<String>> {
        let url = self
            .base_url
            .join(&format!("/api/v1/gems/{}/owners.json", name))
            .map_err(|e| RegistryError::Parse(e.to_string()))?;

        let response = retry_http(&RetryConfig::default(), || {
            self.http.get(url.clone()).send()
        })
        .await?;

        if !response.status().is_success() {
            // Non-fatal - return empty list
            return Ok(Vec::new());
        }

        let owners: Vec<Owner> = response
            .json()
            .await
            .map_err(|e| RegistryError::Parse(e.to_string()))?;

        Ok(owners.into_iter().map(|o| o.handle).collect())
    }

    /// Builds the download URL for a gem version.
    fn build_download_url(&self, name: &str, version: &str) -> RegistryResult<Url> {
        // Format: https://rubygems.org/gems/{name}-{version}.gem
        self.base_url
            .join(&format!("/gems/{}-{}.gem", name, version))
            .map_err(|e| RegistryError::Parse(e.to_string()))
    }

    /// Parses authors string (comma-separated) into a vector.
    fn parse_authors(authors: Option<&str>) -> Vec<String> {
        authors
            .map(|a| a.split(',').map(|s| s.trim().to_string()).collect())
            .unwrap_or_default()
    }

    /// Parses a Ruby gem version to semver.
    fn parse_ruby_version(version: &str) -> Result<Version, RegistryError> {
        // Ruby gem versions are mostly semver compatible, but may have extra segments
        // Try direct parse first
        if let Ok(v) = Version::parse(version) {
            return Ok(v);
        }

        // Handle versions like "1.2.3.4" by taking first 3 segments
        let parts: Vec<&str> = version.split('.').collect();
        if parts.len() >= 3 {
            let normalized = if parts.len() > 3 {
                // Add remaining segments as prerelease
                let prerelease = parts[3..].join(".");
                format!("{}.{}.{}-{}", parts[0], parts[1], parts[2], prerelease)
            } else {
                format!("{}.{}.{}", parts[0], parts[1], parts[2])
            };

            Version::parse(&normalized)
                .map_err(|e| RegistryError::Parse(format!("Invalid version '{}': {}", version, e)))
        } else if parts.len() == 2 {
            // Handle "1.2" -> "1.2.0"
            Version::parse(&format!("{}.{}.0", parts[0], parts[1]))
                .map_err(|e| RegistryError::Parse(format!("Invalid version '{}': {}", version, e)))
        } else if parts.len() == 1 {
            // Handle "1" -> "1.0.0"
            Version::parse(&format!("{}.0.0", parts[0]))
                .map_err(|e| RegistryError::Parse(format!("Invalid version '{}': {}", version, e)))
        } else {
            Err(RegistryError::Parse(format!(
                "Invalid version format: {}",
                version
            )))
        }
    }

    /// Parses a timestamp from RubyGems format.
    fn parse_timestamp(s: &str) -> Option<chrono::DateTime<chrono::Utc>> {
        // RubyGems uses ISO 8601 format
        chrono::DateTime::parse_from_rfc3339(s)
            .ok()
            .map(|dt| dt.with_timezone(&chrono::Utc))
    }
}

impl Default for RubyGemsClient {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl RegistryClient for RubyGemsClient {
    fn ecosystem(&self) -> PackageEcosystem {
        PackageEcosystem::RubyGems
    }

    fn base_url(&self) -> &Url {
        &self.base_url
    }

    async fn get_package(&self, name: &str) -> RegistryResult<PackageMetadata> {
        // Check cache first
        if let Some(cached) = self.cache.get_package(PackageEcosystem::RubyGems, name) {
            tracing::debug!("Cache hit for gem {}", name);
            return Ok(cached);
        }

        let gem = self.fetch_gem(name).await?;
        let versions = self.fetch_versions(name).await?;
        let owners = self.fetch_owners(name).await.unwrap_or_default();

        let version_strings: Vec<String> = versions.iter().map(|v| v.number.clone()).collect();

        // Find latest stable version (non-prerelease, not yanked)
        let latest = versions
            .iter()
            .filter(|v| !v.prerelease && !v.is_yanked)
            .max_by(|a, b| {
                let va = Self::parse_ruby_version(&a.number).ok();
                let vb = Self::parse_ruby_version(&b.number).ok();
                va.cmp(&vb)
            })
            .or_else(|| versions.first())
            .map(|v| v.number.clone());

        // Parse timestamps
        let first_published = versions
            .last()
            .and_then(|v| Self::parse_timestamp(&v.created_at));

        let last_updated = versions
            .first()
            .and_then(|v| Self::parse_timestamp(&v.created_at));

        // Parse URLs
        let homepage = gem
            .homepage_uri
            .as_ref()
            .or(gem.metadata.homepage_uri.as_ref())
            .and_then(|u| Url::parse(u).ok());

        let repository = gem
            .source_code_uri
            .as_ref()
            .or(gem.metadata.source_code_uri.as_ref())
            .and_then(|u| Url::parse(u).ok());

        // Parse maintainers from authors or owners
        let maintainers = if owners.is_empty() {
            Self::parse_authors(gem.authors.as_deref())
        } else {
            owners
        };

        // Determine popularity (gems with > 10M downloads are popular)
        let is_popular = gem.downloads > 10_000_000;

        let package = Package {
            id: PackageId::new(),
            ecosystem: PackageEcosystem::RubyGems,
            name: gem.name.clone(),
            normalized_name: normalize_package_name(&gem.name),
            description: gem.info,
            homepage,
            repository,
            popularity_rank: None,
            is_popular,
            maintainers,
            first_published,
            last_updated,
            cached_at: chrono::Utc::now(),
        };

        let metadata = PackageMetadata {
            package,
            available_versions: version_strings,
            latest_version: latest,
        };

        self.cache
            .set_package(PackageEcosystem::RubyGems, name, metadata.clone());

        Ok(metadata)
    }

    async fn get_version(&self, name: &str, version: &str) -> RegistryResult<VersionMetadata> {
        // Check cache first
        if let Some(cached) = self
            .cache
            .get_version(PackageEcosystem::RubyGems, name, version)
        {
            tracing::debug!("Cache hit for {}@{}", name, version);
            return Ok(cached);
        }

        // Fetch all versions to find the specific one
        let versions = self.fetch_versions(name).await?;

        let version_data = versions
            .iter()
            .find(|v| v.number == version)
            .ok_or_else(|| RegistryError::VersionNotFound(name.to_string(), version.to_string()))?;

        // Also fetch gem info for dependencies
        let gem = self.fetch_gem(name).await?;

        let parsed_version = Self::parse_ruby_version(&version_data.number)?;

        let download_url = self.build_download_url(name, version)?;

        // Parse dependencies (using current gem's dependencies as approximation)
        let dependencies: Vec<PackageDependency> = gem
            .dependencies
            .runtime
            .iter()
            .map(|d| PackageDependency {
                name: d.name.clone(),
                version_constraint: d.requirements.clone(),
                is_optional: false,
                is_dev: false,
            })
            .chain(
                gem.dependencies
                    .development
                    .iter()
                    .map(|d| PackageDependency {
                        name: d.name.clone(),
                        version_constraint: d.requirements.clone(),
                        is_optional: false,
                        is_dev: true,
                    }),
            )
            .collect();

        let checksums = PackageChecksums {
            sha1: None,
            sha256: Some(version_data.sha.clone()),
            sha512: None,
            integrity: None,
        };

        let published_at = Self::parse_timestamp(&version_data.created_at);

        let package_version = PackageVersion {
            package_id: PackageId::new(),
            version: parsed_version,
            published_at,
            yanked: version_data.is_yanked,
            deprecated: version_data.is_yanked,
            deprecation_message: if version_data.is_yanked {
                Some("This version has been yanked".to_string())
            } else {
                None
            },
            checksums,
            download_url: Some(download_url.clone()),
            size_bytes: None,
            attestations: Vec::new(),
            dependencies,
            cached_at: chrono::Utc::now(),
        };

        let metadata = VersionMetadata {
            version: package_version,
            download_url: Some(download_url),
        };

        self.cache
            .set_version(PackageEcosystem::RubyGems, name, version, metadata.clone());

        Ok(metadata)
    }

    async fn download_package(&self, name: &str, version: &str) -> RegistryResult<Bytes> {
        let url = self.build_download_url(name, version)?;

        tracing::debug!("Downloading {}@{} from {}", name, version, url);

        let response = retry_http(&RetryConfig::default(), || {
            self.http.get(url.clone()).send()
        })
        .await?;

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
        // Well-known popular Ruby gems
        let popular = vec![
            "rails",
            "rake",
            "bundler",
            "rspec",
            "activesupport",
            "activerecord",
            "actionpack",
            "actionview",
            "actionmailer",
            "actioncable",
            "activejob",
            "activestorage",
            "activemodel",
            "railties",
            "devise",
            "puma",
            "sidekiq",
            "rack",
            "nokogiri",
            "json",
            "multi_json",
            "aws-sdk-core",
            "aws-sdk-s3",
            "faraday",
            "rest-client",
            "httparty",
            "minitest",
            "rspec-core",
            "rspec-expectations",
            "rspec-mocks",
            "factory_bot",
            "capybara",
            "selenium-webdriver",
            "webdrivers",
            "rubocop",
            "rexml",
            "pg",
            "mysql2",
            "redis",
            "dalli",
            "sassc",
            "sass-rails",
            "sprockets",
            "turbolinks",
            "webpacker",
            "jbuilder",
            "bcrypt",
            "pundit",
            "cancancan",
            "kaminari",
        ];

        Ok(popular.into_iter().take(limit).map(String::from).collect())
    }

    async fn package_exists(&self, name: &str) -> RegistryResult<bool> {
        match self.fetch_gem(name).await {
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
        let client = RubyGemsClient::new();
        let url = client.build_download_url("rails", "7.0.0").unwrap();
        assert_eq!(url.as_str(), "https://rubygems.org/gems/rails-7.0.0.gem");
    }

    #[test]
    fn test_parse_ruby_version() {
        // Standard semver
        let v = RubyGemsClient::parse_ruby_version("1.2.3").unwrap();
        assert_eq!(v.to_string(), "1.2.3");

        // Two-part version
        let v = RubyGemsClient::parse_ruby_version("1.2").unwrap();
        assert_eq!(v.to_string(), "1.2.0");

        // Single-part version
        let v = RubyGemsClient::parse_ruby_version("1").unwrap();
        assert_eq!(v.to_string(), "1.0.0");
    }

    #[test]
    fn test_parse_authors() {
        let authors = RubyGemsClient::parse_authors(Some("Alice, Bob, Charlie"));
        assert_eq!(authors, vec!["Alice", "Bob", "Charlie"]);

        let empty = RubyGemsClient::parse_authors(None);
        assert!(empty.is_empty());
    }
}
