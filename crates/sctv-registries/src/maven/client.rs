//! Maven Central registry client implementation with full functionality.

use async_trait::async_trait;
use bytes::Bytes;
use reqwest::Client;
use sctv_core::{
    Package, PackageChecksums, PackageDependency, PackageEcosystem, PackageId, PackageVersion,
};
use semver::Version;
use sha1::Sha1;
use sha2::{Digest, Sha256};
use std::sync::Arc;
use std::time::Duration;
use url::Url;

use super::models::{
    MavenCoordinate, MavenMetadata, MavenPom, MavenSearchDoc, MavenSearchResponse,
};
use crate::{
    retry_http, PackageMetadata, RegistryCache, RegistryClient, RegistryError, RegistryResult,
    RetryConfig, VersionMetadata,
};

/// Maven Central registry client with caching and hash verification.
pub struct MavenClient {
    http: Client,
    base_url: Url,
    search_url: Url,
    cache: Arc<RegistryCache>,
}

impl MavenClient {
    /// Default Maven Central repository URL.
    pub const DEFAULT_REGISTRY: &'static str = "https://repo1.maven.org/maven2";

    /// Maven Central Search API URL.
    pub const SEARCH_API: &'static str = "https://search.maven.org/solrsearch/select";

    /// Creates a new Maven client with default settings.
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
            search_url: Url::parse(Self::SEARCH_API).expect("Invalid search URL"),
            cache,
        }
    }

    /// Fetches maven-metadata.xml for a coordinate.
    async fn fetch_metadata(&self, coord: &MavenCoordinate) -> RegistryResult<MavenMetadata> {
        let url = self
            .base_url
            .join(&format!("{}/maven-metadata.xml", coord.repo_path()))
            .map_err(|e| RegistryError::Parse(e.to_string()))?;

        tracing::debug!("Fetching Maven metadata from {}", url);

        let response = retry_http(&RetryConfig::default(), || {
            self.http.get(url.clone()).send()
        })
        .await?;

        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Err(RegistryError::PackageNotFound(coord.to_string()));
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

        let xml_text = response.text().await?;
        quick_xml::de::from_str(&xml_text)
            .map_err(|e| RegistryError::Parse(format!("Failed to parse metadata: {e}")))
    }

    /// Fetches POM file for a specific version.
    async fn fetch_pom(&self, coord: &MavenCoordinate, version: &str) -> RegistryResult<MavenPom> {
        let pom_path = coord.artifact_path(version, "pom");
        let url = self
            .base_url
            .join(&pom_path)
            .map_err(|e| RegistryError::Parse(e.to_string()))?;

        tracing::debug!("Fetching POM from {}", url);

        let response = retry_http(&RetryConfig::default(), || {
            self.http.get(url.clone()).send()
        })
        .await?;

        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Err(RegistryError::VersionNotFound(
                coord.to_string(),
                version.to_string(),
            ));
        }

        if !response.status().is_success() {
            return Err(RegistryError::Unavailable(format!(
                "Registry returned status {}",
                response.status()
            )));
        }

        let xml_text = response.text().await?;
        quick_xml::de::from_str(&xml_text)
            .map_err(|e| RegistryError::Parse(format!("Failed to parse POM: {e}")))
    }

    /// Fetches the SHA-1 checksum for an artifact.
    async fn fetch_sha1(&self, artifact_path: &str) -> RegistryResult<Option<String>> {
        let url = self
            .base_url
            .join(&format!("{artifact_path}.sha1"))
            .map_err(|e| RegistryError::Parse(e.to_string()))?;

        let response = retry_http(&RetryConfig::default(), || {
            self.http.get(url.clone()).send()
        })
        .await?;

        if !response.status().is_success() {
            return Ok(None);
        }

        let text = response.text().await?;
        // SHA1 files sometimes have extra info after the hash
        let hash = text.split_whitespace().next().map(String::from);
        Ok(hash)
    }

    /// Fetches the SHA-256 checksum for an artifact.
    async fn fetch_sha256(&self, artifact_path: &str) -> RegistryResult<Option<String>> {
        let url = self
            .base_url
            .join(&format!("{artifact_path}.sha256"))
            .map_err(|e| RegistryError::Parse(e.to_string()))?;

        let response = retry_http(&RetryConfig::default(), || {
            self.http.get(url.clone()).send()
        })
        .await?;

        if !response.status().is_success() {
            return Ok(None);
        }

        let text = response.text().await?;
        let hash = text.split_whitespace().next().map(String::from);
        Ok(hash)
    }

    /// Searches Maven Central for artifacts.
    pub async fn search(&self, query: &str, limit: usize) -> RegistryResult<Vec<MavenSearchDoc>> {
        let url = format!("{}?q={}&rows={}&wt=json", self.search_url, query, limit);

        let response = self.http.get(&url).send().await?;

        if !response.status().is_success() {
            return Err(RegistryError::Unavailable(format!(
                "Search API returned status {}",
                response.status()
            )));
        }

        let search_response: MavenSearchResponse = response.json().await?;
        Ok(search_response.response.docs)
    }

    /// Verifies the integrity of downloaded artifact bytes.
    pub fn verify_integrity(
        &self,
        bytes: &Bytes,
        expected: &PackageChecksums,
    ) -> MavenIntegrityResult {
        let mut result = MavenIntegrityResult {
            sha1_match: None,
            sha256_match: None,
            md5_match: None,
            computed_sha1: None,
            computed_sha256: None,
        };

        // Compute SHA-1
        let sha1_hash = {
            let mut hasher = Sha1::new();
            hasher.update(bytes);
            hex::encode(hasher.finalize())
        };
        result.computed_sha1 = Some(sha1_hash.clone());

        // Compute SHA-256
        let sha256_hash = {
            let mut hasher = Sha256::new();
            hasher.update(bytes);
            hex::encode(hasher.finalize())
        };
        result.computed_sha256 = Some(sha256_hash.clone());

        // Check against expected SHA-1
        if let Some(expected_sha1) = &expected.sha1 {
            result.sha1_match = Some(expected_sha1.to_lowercase() == sha1_hash);
        }

        // Check against expected SHA-256
        if let Some(expected_sha256) = &expected.sha256 {
            result.sha256_match = Some(expected_sha256.to_lowercase() == sha256_hash);
        }

        result
    }

    /// Converts Maven dependencies to package dependencies.
    fn parse_dependencies(pom: &MavenPom) -> Vec<PackageDependency> {
        pom.dependencies
            .as_ref()
            .map(|deps| {
                deps.dependency
                    .iter()
                    .filter_map(|d| {
                        let name = d.coordinate()?;
                        Some(PackageDependency {
                            name,
                            version_constraint: d.version.clone().unwrap_or_default(),
                            is_optional: d.is_optional() || d.is_provided(),
                            is_dev: d.is_test(),
                        })
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Determines the best artifact extension (jar, war, pom-only, etc.).
    fn get_artifact_extension(&self, pom: &MavenPom) -> &'static str {
        match pom.packaging.as_deref() {
            Some("war") => "war",
            Some("ear") => "ear",
            Some("pom") => "pom",
            Some("aar") => "aar",
            _ => "jar",
        }
    }
}

impl Default for MavenClient {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl RegistryClient for MavenClient {
    fn ecosystem(&self) -> PackageEcosystem {
        PackageEcosystem::Maven
    }

    fn base_url(&self) -> &Url {
        &self.base_url
    }

    async fn get_package(&self, name: &str) -> RegistryResult<PackageMetadata> {
        // Check cache first
        if let Some(cached) = self.cache.get_package(PackageEcosystem::Maven, name) {
            tracing::debug!("Cache hit for Maven package {}", name);
            return Ok(cached);
        }

        let coord = MavenCoordinate::parse(name)
            .ok_or_else(|| RegistryError::Parse(format!("Invalid Maven coordinate: {name}")))?;

        let metadata = self.fetch_metadata(&coord).await?;

        let versions: Vec<String> = metadata
            .versioning
            .as_ref()
            .and_then(|v| v.versions.as_ref())
            .map(|v| v.version.clone())
            .unwrap_or_default();

        let latest = metadata
            .versioning
            .as_ref()
            .and_then(|v| v.release.clone().or_else(|| v.latest.clone()));

        // Parse last updated time
        let last_updated = metadata
            .versioning
            .as_ref()
            .and_then(|v| v.last_updated.as_ref())
            .and_then(|s| parse_maven_timestamp(s));

        // Try to get more info from the latest POM
        let (description, homepage, repository, maintainers) = if let Some(ref version) = latest {
            match self.fetch_pom(&coord, version).await {
                Ok(pom) => {
                    let desc = pom.description.clone().or_else(|| pom.name.clone());
                    let home = pom.url.as_ref().and_then(|u| Url::parse(u).ok());
                    let repo = pom
                        .scm
                        .as_ref()
                        .and_then(|s| s.url.as_ref())
                        .and_then(|u| Url::parse(u).ok());
                    let maintainers: Vec<String> = pom
                        .developers
                        .as_ref()
                        .map(|d| {
                            d.developer
                                .iter()
                                .filter_map(|dev| dev.name.clone().or_else(|| dev.id.clone()))
                                .collect()
                        })
                        .unwrap_or_default();
                    (desc, home, repo, maintainers)
                }
                Err(_) => (None, None, None, Vec::new()),
            }
        } else {
            (None, None, None, Vec::new())
        };

        let package = Package {
            id: PackageId::new(),
            ecosystem: PackageEcosystem::Maven,
            name: name.to_string(),
            normalized_name: name.to_lowercase(),
            description,
            homepage,
            repository,
            popularity_rank: None,
            is_popular: false,
            maintainers,
            first_published: None,
            last_updated,
            cached_at: chrono::Utc::now(),
        };

        let pkg_metadata = PackageMetadata {
            package,
            available_versions: versions,
            latest_version: latest,
        };

        self.cache
            .set_package(PackageEcosystem::Maven, name, pkg_metadata.clone());

        Ok(pkg_metadata)
    }

    async fn get_version(&self, name: &str, version: &str) -> RegistryResult<VersionMetadata> {
        // Check cache first
        if let Some(cached) = self
            .cache
            .get_version(PackageEcosystem::Maven, name, version)
        {
            tracing::debug!("Cache hit for {}:{}", name, version);
            return Ok(cached);
        }

        let coord = MavenCoordinate::parse(name)
            .ok_or_else(|| RegistryError::Parse(format!("Invalid Maven coordinate: {name}")))?;

        let pom = self.fetch_pom(&coord, version).await?;

        // Parse version
        let parsed_version = parse_maven_version(version)
            .map_err(|e| RegistryError::Parse(format!("Invalid version: {e}")))?;

        // Determine artifact extension
        let extension = self.get_artifact_extension(&pom);
        let artifact_path = coord.artifact_path(version, extension);

        // Fetch checksums in parallel
        let (sha1, sha256) = tokio::join!(
            self.fetch_sha1(&artifact_path),
            self.fetch_sha256(&artifact_path)
        );

        let checksums = PackageChecksums {
            sha1: sha1.ok().flatten(),
            sha256: sha256.ok().flatten(),
            sha512: None,
            integrity: None,
        };

        let download_url = self.base_url.join(&artifact_path).ok();

        // Parse dependencies
        let dependencies = Self::parse_dependencies(&pom);

        let package_version = PackageVersion {
            package_id: PackageId::new(),
            version: parsed_version,
            published_at: None,
            yanked: false,
            deprecated: false,
            deprecation_message: None,
            checksums,
            download_url: download_url.clone(),
            size_bytes: None,
            attestations: Vec::new(),
            dependencies,
            cached_at: chrono::Utc::now(),
        };

        let metadata = VersionMetadata {
            version: package_version,
            download_url,
        };

        self.cache
            .set_version(PackageEcosystem::Maven, name, version, metadata.clone());

        Ok(metadata)
    }

    async fn download_package(&self, name: &str, version: &str) -> RegistryResult<Bytes> {
        let url = self.get_download_url(name, version).await?;

        tracing::debug!("Downloading {}:{} from {}", name, version, url);

        let response = retry_http(&RetryConfig::default(), || {
            self.http.get(url.clone()).send()
        })
        .await?;

        if !response.status().is_success() {
            return Err(RegistryError::Unavailable(format!(
                "Download failed with status {}",
                response.status()
            )));
        }

        Ok(response.bytes().await?)
    }

    async fn list_popular(&self, limit: usize) -> RegistryResult<Vec<String>> {
        // Return a curated list of popular Maven artifacts
        let popular = vec![
            "org.slf4j:slf4j-api",
            "com.google.guava:guava",
            "org.apache.commons:commons-lang3",
            "com.fasterxml.jackson.core:jackson-databind",
            "org.junit.jupiter:junit-jupiter-api",
            "org.apache.logging.log4j:log4j-core",
            "com.google.code.gson:gson",
            "org.projectlombok:lombok",
            "org.mockito:mockito-core",
            "org.springframework:spring-core",
            "org.springframework.boot:spring-boot-starter",
            "com.squareup.okhttp3:okhttp",
            "org.apache.httpcomponents:httpclient",
            "io.netty:netty-all",
            "junit:junit",
            "org.assertj:assertj-core",
            "com.h2database:h2",
            "org.postgresql:postgresql",
            "mysql:mysql-connector-java",
            "org.apache.maven:maven-core",
            "org.apache.commons:commons-io",
            "org.apache.commons:commons-collections4",
            "com.amazonaws:aws-java-sdk-core",
            "io.grpc:grpc-core",
            "org.jetbrains.kotlin:kotlin-stdlib",
            "org.scala-lang:scala-library",
            "io.quarkus:quarkus-core",
            "io.micronaut:micronaut-core",
            "org.jboss:jboss-common-core",
            "org.eclipse.jetty:jetty-server",
        ];

        Ok(popular.into_iter().take(limit).map(String::from).collect())
    }

    async fn package_exists(&self, name: &str) -> RegistryResult<bool> {
        let coord = match MavenCoordinate::parse(name) {
            Some(c) => c,
            None => return Ok(false),
        };

        match self.fetch_metadata(&coord).await {
            Ok(_) => Ok(true),
            Err(RegistryError::PackageNotFound(_)) => Ok(false),
            Err(e) => Err(e),
        }
    }

    async fn get_download_url(&self, name: &str, version: &str) -> RegistryResult<Url> {
        let version_meta = self.get_version(name, version).await?;

        version_meta.download_url.ok_or_else(|| {
            RegistryError::Unavailable(format!("No download URL for {name}:{version}"))
        })
    }
}

/// Result of integrity verification for Maven packages.
#[derive(Debug, Clone)]
pub struct MavenIntegrityResult {
    pub sha1_match: Option<bool>,
    pub sha256_match: Option<bool>,
    pub md5_match: Option<bool>,
    pub computed_sha1: Option<String>,
    pub computed_sha256: Option<String>,
}

impl MavenIntegrityResult {
    /// Returns true if all available checks passed.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        let checks = [self.sha1_match, self.sha256_match, self.md5_match];
        checks.iter().filter_map(|c| *c).all(|matched| matched)
    }

    /// Returns true if any check failed.
    #[must_use]
    pub fn has_failure(&self) -> bool {
        let checks = [self.sha1_match, self.sha256_match, self.md5_match];
        checks.iter().filter_map(|c| *c).any(|matched| !matched)
    }
}

/// Parses a Maven timestamp (`YYYYMMDDHHmmss` format).
fn parse_maven_timestamp(s: &str) -> Option<chrono::DateTime<chrono::Utc>> {
    if s.len() < 14 {
        return None;
    }

    let year: i32 = s[0..4].parse().ok()?;
    let month: u32 = s[4..6].parse().ok()?;
    let day: u32 = s[6..8].parse().ok()?;
    let hour: u32 = s[8..10].parse().ok()?;
    let min: u32 = s[10..12].parse().ok()?;
    let sec: u32 = s[12..14].parse().ok()?;

    chrono::NaiveDate::from_ymd_opt(year, month, day)
        .and_then(|date| date.and_hms_opt(hour, min, sec))
        .map(|naive| chrono::DateTime::from_naive_utc_and_offset(naive, chrono::Utc))
}

/// Parses a Maven version string into a semver Version.
/// Maven versions are more flexible than strict semver.
fn parse_maven_version(version: &str) -> Result<Version, String> {
    // Try direct semver parse first
    if let Ok(v) = Version::parse(version) {
        return Ok(v);
    }

    // Handle common Maven patterns
    let cleaned = version
        .trim()
        .trim_start_matches('v')
        .trim_start_matches('V');

    // Handle versions like "3.12" -> "3.12.0"
    let parts: Vec<&str> = cleaned.split('.').collect();
    let normalized = match parts.len() {
        1 => format!("{}.0.0", extract_numeric_prefix(parts[0])),
        2 => format!(
            "{}.{}.0",
            extract_numeric_prefix(parts[0]),
            extract_numeric_prefix(parts[1])
        ),
        _ => {
            let major = extract_numeric_prefix(parts[0]);
            let minor = extract_numeric_prefix(parts[1]);
            let patch = extract_numeric_prefix(parts[2]);
            format!("{major}.{minor}.{patch}")
        }
    };

    Version::parse(&normalized).map_err(|e| e.to_string())
}

/// Extracts the leading numeric portion of a version segment.
fn extract_numeric_prefix(s: &str) -> u64 {
    let numeric: String = s.chars().take_while(char::is_ascii_digit).collect();
    numeric.parse().unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Datelike;

    #[test]
    fn test_parse_maven_version() {
        assert!(parse_maven_version("3.12.0").is_ok());
        assert!(parse_maven_version("1.0").is_ok());

        let v = parse_maven_version("3.12").unwrap();
        assert_eq!(v.to_string(), "3.12.0");
    }

    #[test]
    fn test_parse_maven_timestamp() {
        let ts = parse_maven_timestamp("20231215143022").unwrap();
        assert_eq!(ts.year(), 2023);
        assert_eq!(ts.month(), 12);
    }
}
