//! `NuGet` registry client implementation.

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
use tokio::sync::OnceCell;
use url::Url;

use super::models::{RegistrationIndex, RegistrationLeaf, RegistrationPage, ServiceIndex};
use crate::{
    retry_http, PackageMetadata, RegistryCache, RegistryClient, RegistryError, RegistryResult,
    RetryConfig, VersionMetadata,
};

/// `NuGet` registry client with caching and service discovery.
pub struct NuGetClient {
    http: Client,
    base_url: Url,
    cache: Arc<RegistryCache>,
    service_index: OnceCell<ServiceIndex>,
}

impl NuGetClient {
    /// Default `NuGet` API URL.
    pub const DEFAULT_REGISTRY: &'static str = "https://api.nuget.org/v3/index.json";

    /// Creates a new `NuGet` client with default settings.
    #[must_use]
    pub fn new() -> Self {
        Self::with_config(Self::DEFAULT_REGISTRY, Arc::new(RegistryCache::new()))
    }

    /// Creates a client with custom URL and cache.
    ///
    /// # Panics
    ///
    /// Panics if the HTTP client cannot be built or if `service_url` is not a valid URL.
    #[must_use]
    pub fn with_config(service_url: &str, cache: Arc<RegistryCache>) -> Self {
        let http = Client::builder()
            .timeout(Duration::from_secs(30))
            .gzip(true)
            .user_agent("sctv-registry-client/0.1.0")
            .build()
            .expect("Failed to create HTTP client");

        Self {
            http,
            base_url: Url::parse(service_url).expect("Invalid service URL"),
            cache,
            service_index: OnceCell::new(),
        }
    }

    /// Gets the service index, fetching it if needed.
    async fn get_service_index(&self) -> RegistryResult<&ServiceIndex> {
        self.service_index
            .get_or_try_init(|| async {
                tracing::debug!("Fetching NuGet service index from {}", self.base_url);

                let response = self.http.get(self.base_url.clone()).send().await?;

                if !response.status().is_success() {
                    return Err(RegistryError::Unavailable(format!(
                        "Service index returned status {}",
                        response.status()
                    )));
                }

                response
                    .json::<ServiceIndex>()
                    .await
                    .map_err(|e| RegistryError::Parse(e.to_string()))
            })
            .await
    }

    /// Fetches registration data for a package.
    async fn fetch_registration(&self, name: &str) -> RegistryResult<RegistrationIndex> {
        let service_index = self.get_service_index().await?;

        let base_url = service_index
            .registration_base()
            .ok_or_else(|| RegistryError::Unavailable("No registration service found".into()))?;

        // NuGet uses lowercase package IDs in URLs
        let package_id = name.to_lowercase();
        let url = format!("{base_url}{package_id}/index.json");

        tracing::debug!("Fetching registration for {} from {}", name, url);

        let response = self.http.get(&url).send().await?;

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

    /// Fetches a specific registration page if items are not inline.
    async fn fetch_registration_page(&self, page_url: &str) -> RegistryResult<RegistrationPage> {
        tracing::debug!("Fetching registration page from {}", page_url);

        let response = self.http.get(page_url).send().await?;

        if !response.status().is_success() {
            return Err(RegistryError::Unavailable(format!(
                "Failed to fetch page: status {}",
                response.status()
            )));
        }

        response
            .json()
            .await
            .map_err(|e| RegistryError::Parse(e.to_string()))
    }

    /// Gets all versions from a registration index, fetching pages if needed.
    async fn get_all_versions(
        &self,
        index: &RegistrationIndex,
    ) -> RegistryResult<Vec<RegistrationLeaf>> {
        let mut all_items = Vec::new();

        for page in &index.items {
            if page.items.is_empty() {
                // Items are not inline, need to fetch the page
                let fetched_page = self.fetch_registration_page(&page.id).await?;
                all_items.extend(fetched_page.items);
            } else {
                all_items.extend(page.items.clone());
            }
        }

        Ok(all_items)
    }

    /// Builds the download URL for a package version.
    async fn build_download_url(&self, name: &str, version: &str) -> RegistryResult<Url> {
        let service_index = self.get_service_index().await?;

        let base_url = service_index
            .package_content_base()
            .ok_or_else(|| RegistryError::Unavailable("No package content service found".into()))?;

        // Format: {base}/{id-lower}/{version}/{id-lower}.{version}.nupkg
        let package_id = name.to_lowercase();
        let version_lower = version.to_lowercase();
        let url_str =
            format!("{base_url}{package_id}/{version_lower}/{package_id}.{version_lower}.nupkg");

        Url::parse(&url_str).map_err(|e| RegistryError::Parse(e.to_string()))
    }

    /// Parses a `NuGet` timestamp.
    fn parse_timestamp(s: &str) -> Option<chrono::DateTime<chrono::Utc>> {
        chrono::DateTime::parse_from_rfc3339(s)
            .ok()
            .map(|dt| dt.with_timezone(&chrono::Utc))
    }
}

impl Default for NuGetClient {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl RegistryClient for NuGetClient {
    fn ecosystem(&self) -> PackageEcosystem {
        PackageEcosystem::NuGet
    }

    fn base_url(&self) -> &Url {
        &self.base_url
    }

    async fn get_package(&self, name: &str) -> RegistryResult<PackageMetadata> {
        // Check cache first
        if let Some(cached) = self.cache.get_package(PackageEcosystem::NuGet, name) {
            tracing::debug!("Cache hit for package {}", name);
            return Ok(cached);
        }

        let registration = self.fetch_registration(name).await?;
        let all_versions = self.get_all_versions(&registration).await?;

        if all_versions.is_empty() {
            return Err(RegistryError::PackageNotFound(name.to_string()));
        }

        // Get version strings
        let versions: Vec<String> = all_versions
            .iter()
            .map(|v| v.catalog_entry.version.clone())
            .collect();

        // Find the latest (highest semver) non-prerelease version
        let latest = all_versions
            .iter()
            .filter(|v| {
                Version::parse(&v.catalog_entry.version)
                    .map(|ver| ver.pre.is_empty())
                    .unwrap_or(false)
            })
            .max_by(|a, b| {
                let va = Version::parse(&a.catalog_entry.version).ok();
                let vb = Version::parse(&b.catalog_entry.version).ok();
                va.cmp(&vb)
            })
            .or_else(|| all_versions.last())
            .map(|v| v.catalog_entry.version.clone());

        // Use the latest version's catalog entry for package metadata
        let latest_entry = all_versions
            .last()
            .map(|v| &v.catalog_entry)
            .ok_or_else(|| RegistryError::Parse("No versions found".into()))?;

        // Extract maintainers/authors
        let maintainers = latest_entry
            .authors
            .as_ref()
            .map(super::models::AuthorsField::to_vec)
            .unwrap_or_default();

        // Parse timestamps
        let first_published = all_versions
            .first()
            .and_then(|v| v.catalog_entry.published.as_ref())
            .and_then(|s| Self::parse_timestamp(s));

        let last_updated = all_versions
            .last()
            .and_then(|v| v.catalog_entry.published.as_ref())
            .and_then(|s| Self::parse_timestamp(s));

        // Parse URLs
        let homepage = latest_entry
            .project_url
            .as_ref()
            .and_then(|u| Url::parse(u).ok());

        // NuGet doesn't have separate repository URL, use project URL
        let repository = homepage.clone();

        let package = Package {
            id: PackageId::new(),
            ecosystem: PackageEcosystem::NuGet,
            name: latest_entry.package_id.clone(),
            normalized_name: normalize_package_name(&latest_entry.package_id),
            description: latest_entry
                .description
                .clone()
                .or_else(|| latest_entry.summary.clone()),
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
            .set_package(PackageEcosystem::NuGet, name, metadata.clone());

        Ok(metadata)
    }

    async fn get_version(&self, name: &str, version: &str) -> RegistryResult<VersionMetadata> {
        // Check cache first
        if let Some(cached) = self
            .cache
            .get_version(PackageEcosystem::NuGet, name, version)
        {
            tracing::debug!("Cache hit for {}@{}", name, version);
            return Ok(cached);
        }

        let registration = self.fetch_registration(name).await?;
        let all_versions = self.get_all_versions(&registration).await?;

        // Find the specific version
        let version_data = all_versions
            .iter()
            .find(|v| v.catalog_entry.version.to_lowercase() == version.to_lowercase())
            .ok_or_else(|| RegistryError::VersionNotFound(name.to_string(), version.to_string()))?;

        let entry = &version_data.catalog_entry;

        let parsed_version = Version::parse(&entry.version)
            .map_err(|e| RegistryError::Parse(format!("Invalid version: {e}")))?;

        let download_url = self.build_download_url(name, &entry.version).await?;

        // Parse dependencies
        let dependencies = entry
            .dependency_groups
            .as_ref()
            .map(|groups| {
                groups
                    .iter()
                    .flat_map(|g| {
                        g.dependencies.as_ref().map_or(vec![], |deps| {
                            deps.iter()
                                .map(|d| PackageDependency {
                                    name: d.package_id.clone(),
                                    version_constraint: d.range.clone().unwrap_or_default(),
                                    is_optional: false,
                                    is_dev: false,
                                })
                                .collect()
                        })
                    })
                    .collect()
            })
            .unwrap_or_default();

        // Check deprecation
        let (deprecated, deprecation_message) = entry
            .deprecation
            .as_ref()
            .map_or((false, None), |dep| (true, dep.message.clone()));

        let published_at = entry
            .published
            .as_ref()
            .and_then(|s| Self::parse_timestamp(s));

        // NuGet doesn't provide checksums in the registration API
        let checksums = PackageChecksums::default();

        let package_version = PackageVersion {
            package_id: PackageId::new(),
            version: parsed_version,
            published_at,
            yanked: entry.listed == Some(false),
            deprecated,
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
            .set_version(PackageEcosystem::NuGet, name, version, metadata.clone());

        Ok(metadata)
    }

    async fn download_package(&self, name: &str, version: &str) -> RegistryResult<Bytes> {
        let url = self.build_download_url(name, version).await?;

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
        // Well-known popular NuGet packages
        let popular = vec![
            "Newtonsoft.Json",
            "Microsoft.Extensions.Logging",
            "Microsoft.Extensions.DependencyInjection",
            "Microsoft.Extensions.Configuration",
            "System.Text.Json",
            "AutoMapper",
            "Serilog",
            "FluentValidation",
            "Dapper",
            "Entity Framework Core",
            "xunit",
            "NUnit",
            "Moq",
            "Polly",
            "MediatR",
            "Swashbuckle.AspNetCore",
            "StackExchange.Redis",
            "Npgsql",
            "Microsoft.EntityFrameworkCore.SqlServer",
            "Microsoft.AspNetCore.Authentication.JwtBearer",
            "Azure.Storage.Blobs",
            "AWSSDK.Core",
            "RestSharp",
            "Humanizer",
            "MailKit",
            "CsvHelper",
            "Hangfire",
            "Quartz",
            "RabbitMQ.Client",
            "Confluent.Kafka",
            "Grpc.Net.Client",
            "GraphQL",
            "HtmlAgilityPack",
            "AngleSharp",
            "Bogus",
            "FluentAssertions",
            "BenchmarkDotNet",
            "Autofac",
            "NLog",
            "log4net",
        ];

        Ok(popular.into_iter().take(limit).map(String::from).collect())
    }

    async fn package_exists(&self, name: &str) -> RegistryResult<bool> {
        match self.fetch_registration(name).await {
            Ok(_) => Ok(true),
            Err(RegistryError::PackageNotFound(_)) => Ok(false),
            Err(e) => Err(e),
        }
    }

    async fn get_download_url(&self, name: &str, version: &str) -> RegistryResult<Url> {
        self.build_download_url(name, version).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service_index_parsing() {
        let json = r#"{
            "version": "3.0.0",
            "resources": [
                {
                    "@id": "https://api.nuget.org/v3/registration5-semver1/",
                    "@type": "RegistrationsBaseUrl/3.6.0"
                },
                {
                    "@id": "https://api.nuget.org/v3-flatcontainer/",
                    "@type": "PackageBaseAddress/3.0.0"
                }
            ]
        }"#;

        let index: ServiceIndex = serde_json::from_str(json).unwrap();
        assert_eq!(
            index.registration_base(),
            Some("https://api.nuget.org/v3/registration5-semver1/")
        );
        assert_eq!(
            index.package_content_base(),
            Some("https://api.nuget.org/v3-flatcontainer/")
        );
    }
}
