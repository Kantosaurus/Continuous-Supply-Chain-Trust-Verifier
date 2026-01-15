//! Go module proxy client implementation.

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
    PackageMetadata, RegistryCache, RegistryClient, RegistryError, RegistryResult, VersionMetadata,
};

/// Go module proxy client with caching.
pub struct GoModulesClient {
    http: Client,
    base_url: Url,
    cache: Arc<RegistryCache>,
}

impl GoModulesClient {
    /// Default Go module proxy URL.
    pub const DEFAULT_REGISTRY: &'static str = "https://proxy.golang.org";

    /// Creates a new Go modules client with default settings.
    #[must_use]
    pub fn new() -> Self {
        Self::with_config(Self::DEFAULT_REGISTRY, Arc::new(RegistryCache::new()))
    }

    /// Creates a client with custom URL and cache.
    #[must_use]
    pub fn with_config(proxy_url: &str, cache: Arc<RegistryCache>) -> Self {
        let http = Client::builder()
            .timeout(Duration::from_secs(30))
            .gzip(true)
            .user_agent("sctv-registry-client/0.1.0")
            .build()
            .expect("Failed to create HTTP client");

        Self {
            http,
            base_url: Url::parse(proxy_url).expect("Invalid proxy URL"),
            cache,
        }
    }

    /// Encodes a module path for use in URLs.
    /// Per the module proxy protocol, uppercase letters are encoded as !lowercase.
    fn encode_module_path(path: &str) -> String {
        let mut result = String::with_capacity(path.len() * 2);
        for c in path.chars() {
            if c.is_uppercase() {
                result.push('!');
                result.push(c.to_lowercase().next().unwrap_or(c));
            } else {
                result.push(c);
            }
        }
        result
    }

    /// Fetches the list of available versions.
    async fn fetch_version_list(&self, module: &str) -> RegistryResult<Vec<String>> {
        let encoded = Self::encode_module_path(module);
        let url = self
            .base_url
            .join(&format!("/{}/@v/list", encoded))
            .map_err(|e| RegistryError::Parse(e.to_string()))?;

        tracing::debug!("Fetching version list for {} from {}", module, url);

        let response = self.http.get(url).send().await?;

        if response.status() == reqwest::StatusCode::NOT_FOUND
            || response.status() == reqwest::StatusCode::GONE
        {
            return Err(RegistryError::PackageNotFound(module.to_string()));
        }

        if response.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
            return Err(RegistryError::RateLimited);
        }

        if !response.status().is_success() {
            return Err(RegistryError::Unavailable(format!(
                "Proxy returned status {}",
                response.status()
            )));
        }

        let text = response.text().await?;

        // Each line is a version
        let versions: Vec<String> = text
            .lines()
            .map(|line| line.trim().to_string())
            .filter(|line| !line.is_empty())
            .collect();

        Ok(versions)
    }

    /// Fetches version info.
    async fn fetch_version_info(&self, module: &str, version: &str) -> RegistryResult<VersionInfo> {
        let encoded = Self::encode_module_path(module);
        let url = self
            .base_url
            .join(&format!("/{}/@v/{}.info", encoded, version))
            .map_err(|e| RegistryError::Parse(e.to_string()))?;

        tracing::debug!("Fetching version info for {}@{} from {}", module, version, url);

        let response = self.http.get(url).send().await?;

        if response.status() == reqwest::StatusCode::NOT_FOUND
            || response.status() == reqwest::StatusCode::GONE
        {
            return Err(RegistryError::VersionNotFound(
                module.to_string(),
                version.to_string(),
            ));
        }

        if !response.status().is_success() {
            return Err(RegistryError::Unavailable(format!(
                "Proxy returned status {}",
                response.status()
            )));
        }

        response
            .json()
            .await
            .map_err(|e| RegistryError::Parse(e.to_string()))
    }

    /// Fetches the go.mod file for a version.
    async fn fetch_go_mod(&self, module: &str, version: &str) -> RegistryResult<GoMod> {
        let encoded = Self::encode_module_path(module);
        let url = self
            .base_url
            .join(&format!("/{}/@v/{}.mod", encoded, version))
            .map_err(|e| RegistryError::Parse(e.to_string()))?;

        tracing::debug!("Fetching go.mod for {}@{} from {}", module, version, url);

        let response = self.http.get(url).send().await?;

        if response.status() == reqwest::StatusCode::NOT_FOUND
            || response.status() == reqwest::StatusCode::GONE
        {
            return Err(RegistryError::VersionNotFound(
                module.to_string(),
                version.to_string(),
            ));
        }

        if !response.status().is_success() {
            return Err(RegistryError::Unavailable(format!(
                "Proxy returned status {}",
                response.status()
            )));
        }

        let content = response.text().await?;
        Ok(GoMod::parse(&content))
    }

    /// Builds the download URL for a module version.
    fn build_download_url(&self, module: &str, version: &str) -> RegistryResult<Url> {
        let encoded = Self::encode_module_path(module);
        self.base_url
            .join(&format!("/{}/@v/{}.zip", encoded, version))
            .map_err(|e| RegistryError::Parse(e.to_string()))
    }

    /// Parses a Go module version to semver.
    /// Go versions are like v1.2.3, v1.2.3-rc1, v1.2.3+incompatible
    fn parse_go_version(version: &str) -> Result<Version, RegistryError> {
        // Remove the 'v' prefix if present
        let version_str = version.strip_prefix('v').unwrap_or(version);

        // Remove +incompatible suffix
        let version_str = version_str.split('+').next().unwrap_or(version_str);

        Version::parse(version_str)
            .map_err(|e| RegistryError::Parse(format!("Invalid version '{}': {}", version, e)))
    }

    /// Parses a timestamp from Go proxy format (RFC3339).
    fn parse_timestamp(s: &str) -> Option<chrono::DateTime<chrono::Utc>> {
        chrono::DateTime::parse_from_rfc3339(s)
            .ok()
            .map(|dt| dt.with_timezone(&chrono::Utc))
    }

    /// Extracts repository URL from module path.
    fn infer_repository_url(module: &str) -> Option<Url> {
        // Common hosting patterns
        if module.starts_with("github.com/") {
            return Url::parse(&format!("https://{}", module)).ok();
        }
        if module.starts_with("gitlab.com/") {
            return Url::parse(&format!("https://{}", module)).ok();
        }
        if module.starts_with("bitbucket.org/") {
            return Url::parse(&format!("https://{}", module)).ok();
        }

        // For other modules, try to construct a URL
        // Take the first two path segments for the repository
        let parts: Vec<&str> = module.splitn(3, '/').collect();
        if parts.len() >= 2 {
            return Url::parse(&format!("https://{}/{}", parts[0], parts[1])).ok();
        }

        None
    }
}

impl Default for GoModulesClient {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl RegistryClient for GoModulesClient {
    fn ecosystem(&self) -> PackageEcosystem {
        PackageEcosystem::GoModules
    }

    fn base_url(&self) -> &Url {
        &self.base_url
    }

    async fn get_package(&self, name: &str) -> RegistryResult<PackageMetadata> {
        // Check cache first
        if let Some(cached) = self.cache.get_package(PackageEcosystem::GoModules, name) {
            tracing::debug!("Cache hit for module {}", name);
            return Ok(cached);
        }

        let versions = self.fetch_version_list(name).await?;

        if versions.is_empty() {
            return Err(RegistryError::PackageNotFound(name.to_string()));
        }

        // Find the latest stable version
        let latest = versions
            .iter()
            .filter(|v| {
                // Filter out prereleases
                !v.contains("-rc")
                    && !v.contains("-alpha")
                    && !v.contains("-beta")
                    && !v.contains("-pre")
            })
            .max_by(|a, b| {
                let va = Self::parse_go_version(a).ok();
                let vb = Self::parse_go_version(b).ok();
                va.cmp(&vb)
            })
            .or_else(|| versions.last())
            .cloned();

        // Get version info for latest to extract timestamps
        let (first_published, last_updated) = if let Some(ref latest_ver) = latest {
            let info = self.fetch_version_info(name, latest_ver).await.ok();
            let last = info.as_ref().and_then(|i| Self::parse_timestamp(&i.time));

            // For first published, try to get the first version's info
            let first = if let Some(first_ver) = versions.first() {
                self.fetch_version_info(name, first_ver)
                    .await
                    .ok()
                    .and_then(|i| Self::parse_timestamp(&i.time))
            } else {
                None
            };

            (first, last)
        } else {
            (None, None)
        };

        // Infer repository URL from module path
        let repository = Self::infer_repository_url(name);

        let package = Package {
            id: PackageId::new(),
            ecosystem: PackageEcosystem::GoModules,
            name: name.to_string(),
            normalized_name: normalize_package_name(name),
            description: None, // Go proxy doesn't provide descriptions
            homepage: repository.clone(),
            repository,
            popularity_rank: None,
            is_popular: false,
            maintainers: Vec::new(), // Not available from proxy
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
            .set_package(PackageEcosystem::GoModules, name, metadata.clone());

        Ok(metadata)
    }

    async fn get_version(&self, name: &str, version: &str) -> RegistryResult<VersionMetadata> {
        // Check cache first
        if let Some(cached) = self.cache.get_version(PackageEcosystem::GoModules, name, version) {
            tracing::debug!("Cache hit for {}@{}", name, version);
            return Ok(cached);
        }

        // Fetch version info
        let info = self.fetch_version_info(name, version).await?;

        // Fetch go.mod for dependencies
        let go_mod = self.fetch_go_mod(name, version).await.ok();

        let parsed_version = Self::parse_go_version(&info.version)?;

        let download_url = self.build_download_url(name, &info.version)?;

        // Parse dependencies from go.mod
        let dependencies = go_mod
            .as_ref()
            .map(|gm| {
                gm.require
                    .iter()
                    .map(|req| PackageDependency {
                        name: req.path.clone(),
                        version_constraint: req.version.clone(),
                        is_optional: false,
                        is_dev: req.indirect,
                    })
                    .collect()
            })
            .unwrap_or_default();

        // Check if version is retracted
        let (yanked, deprecation_message) = go_mod
            .as_ref()
            .map(|gm| {
                let retracted = gm.retract.iter().any(|r| {
                    if let Some(high) = &r.high {
                        // Range retraction
                        version >= r.low.as_str() && version <= high.as_str()
                    } else {
                        // Single version retraction
                        version == r.low
                    }
                });

                if retracted {
                    let msg = gm
                        .retract
                        .iter()
                        .find_map(|r| r.rationale.clone())
                        .unwrap_or_else(|| "This version has been retracted".to_string());
                    (true, Some(msg))
                } else {
                    (false, None)
                }
            })
            .unwrap_or((false, None));

        let published_at = Self::parse_timestamp(&info.time);

        // Go proxy doesn't provide checksums directly
        let checksums = PackageChecksums::default();

        let package_version = PackageVersion {
            package_id: PackageId::new(),
            version: parsed_version,
            published_at,
            yanked,
            deprecated: yanked,
            deprecation_message,
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
            .set_version(PackageEcosystem::GoModules, name, version, metadata.clone());

        Ok(metadata)
    }

    async fn download_package(&self, name: &str, version: &str) -> RegistryResult<Bytes> {
        let url = self.build_download_url(name, version)?;

        tracing::debug!("Downloading {}@{} from {}", name, version, url);

        let response = self.http.get(url).send().await?;

        if response.status() == reqwest::StatusCode::NOT_FOUND
            || response.status() == reqwest::StatusCode::GONE
        {
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
        // Well-known popular Go modules
        let popular = vec![
            "github.com/gin-gonic/gin",
            "github.com/gorilla/mux",
            "github.com/go-chi/chi",
            "github.com/labstack/echo",
            "github.com/gofiber/fiber",
            "github.com/stretchr/testify",
            "github.com/spf13/cobra",
            "github.com/spf13/viper",
            "github.com/sirupsen/logrus",
            "go.uber.org/zap",
            "github.com/rs/zerolog",
            "github.com/pkg/errors",
            "github.com/go-sql-driver/mysql",
            "github.com/lib/pq",
            "github.com/jackc/pgx",
            "github.com/go-redis/redis",
            "github.com/go-gorm/gorm",
            "github.com/jmoiron/sqlx",
            "github.com/dgrijalva/jwt-go",
            "github.com/golang-jwt/jwt",
            "google.golang.org/grpc",
            "google.golang.org/protobuf",
            "github.com/grpc-ecosystem/grpc-gateway",
            "github.com/prometheus/client_golang",
            "github.com/opentracing/opentracing-go",
            "go.opentelemetry.io/otel",
            "github.com/aws/aws-sdk-go",
            "github.com/aws/aws-sdk-go-v2",
            "cloud.google.com/go",
            "github.com/Azure/azure-sdk-for-go",
            "k8s.io/client-go",
            "k8s.io/api",
            "k8s.io/apimachinery",
            "github.com/hashicorp/consul",
            "github.com/hashicorp/vault",
            "github.com/nats-io/nats.go",
            "github.com/segmentio/kafka-go",
            "github.com/streadway/amqp",
            "github.com/go-playground/validator",
            "github.com/mitchellh/mapstructure",
            "github.com/fatih/color",
            "github.com/schollz/progressbar",
            "github.com/google/uuid",
            "github.com/oklog/ulid",
            "github.com/shopspring/decimal",
            "github.com/robfig/cron",
            "github.com/go-co-op/gocron",
            "github.com/fsnotify/fsnotify",
            "github.com/boltdb/bolt",
            "go.etcd.io/bbolt",
        ];

        Ok(popular.into_iter().take(limit).map(String::from).collect())
    }

    async fn package_exists(&self, name: &str) -> RegistryResult<bool> {
        match self.fetch_version_list(name).await {
            Ok(versions) => Ok(!versions.is_empty()),
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
    fn test_encode_module_path() {
        assert_eq!(
            GoModulesClient::encode_module_path("github.com/Azure/azure-sdk"),
            "github.com/!azure/azure-sdk"
        );
        assert_eq!(
            GoModulesClient::encode_module_path("github.com/gin-gonic/gin"),
            "github.com/gin-gonic/gin"
        );
    }

    #[test]
    fn test_parse_go_version() {
        let v = GoModulesClient::parse_go_version("v1.2.3").unwrap();
        assert_eq!(v.to_string(), "1.2.3");

        let v = GoModulesClient::parse_go_version("v1.2.3-rc1").unwrap();
        assert_eq!(v.major, 1);
        assert!(!v.pre.is_empty());

        let v = GoModulesClient::parse_go_version("v1.2.3+incompatible").unwrap();
        assert_eq!(v.to_string(), "1.2.3");
    }

    #[test]
    fn test_infer_repository_url() {
        let url = GoModulesClient::infer_repository_url("github.com/gin-gonic/gin").unwrap();
        assert_eq!(url.as_str(), "https://github.com/gin-gonic/gin");

        // For versioned paths like /v2, we still get the full module path
        let url =
            GoModulesClient::infer_repository_url("github.com/gin-gonic/gin/v2").unwrap();
        assert_eq!(url.as_str(), "https://github.com/gin-gonic/gin/v2");
    }

    #[test]
    fn test_build_download_url() {
        let client = GoModulesClient::new();
        let url = client
            .build_download_url("github.com/gin-gonic/gin", "v1.9.0")
            .unwrap();
        assert_eq!(
            url.as_str(),
            "https://proxy.golang.org/github.com/gin-gonic/gin/@v/v1.9.0.zip"
        );
    }
}
