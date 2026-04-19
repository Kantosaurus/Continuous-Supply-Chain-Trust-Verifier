//! `NuGet` API v3 response models.

use serde::Deserialize;

/// `NuGet` service index response.
#[derive(Debug, Clone, Deserialize)]
pub struct ServiceIndex {
    pub version: String,
    pub resources: Vec<ServiceResource>,
}

/// A resource in the service index.
#[derive(Debug, Clone, Deserialize)]
pub struct ServiceResource {
    #[serde(rename = "@id")]
    pub id: String,
    #[serde(rename = "@type")]
    pub resource_type: String,
    pub comment: Option<String>,
}

impl ServiceIndex {
    /// Gets the URL for a specific service type.
    #[must_use]
    pub fn get_service_url(&self, service_type: &str) -> Option<&str> {
        self.resources
            .iter()
            .find(|r| r.resource_type == service_type)
            .map(|r| r.id.as_str())
    }

    /// Gets the registration base URL.
    #[must_use]
    pub fn registration_base(&self) -> Option<&str> {
        // Try different registration endpoints in order of preference
        self.get_service_url("RegistrationsBaseUrl/3.6.0")
            .or_else(|| self.get_service_url("RegistrationsBaseUrl/3.4.0"))
            .or_else(|| self.get_service_url("RegistrationsBaseUrl/3.0.0-rc"))
            .or_else(|| self.get_service_url("RegistrationsBaseUrl"))
    }

    /// Gets the package content base URL.
    #[must_use]
    pub fn package_content_base(&self) -> Option<&str> {
        self.get_service_url("PackageBaseAddress/3.0.0")
    }

    /// Gets the search query service URL.
    #[must_use]
    pub fn search_service(&self) -> Option<&str> {
        self.get_service_url("SearchQueryService")
            .or_else(|| self.get_service_url("SearchQueryService/3.5.0"))
            .or_else(|| self.get_service_url("SearchQueryService/3.0.0-rc"))
    }
}

/// Registration index response (package metadata).
#[derive(Debug, Clone, Deserialize)]
pub struct RegistrationIndex {
    #[serde(rename = "@id")]
    pub id: String,
    pub count: u32,
    pub items: Vec<RegistrationPage>,
}

/// A page in the registration index.
#[derive(Debug, Clone, Deserialize)]
pub struct RegistrationPage {
    #[serde(rename = "@id")]
    pub id: String,
    pub count: u32,
    pub lower: String,
    pub upper: String,
    #[serde(default)]
    pub items: Vec<RegistrationLeaf>,
}

/// A leaf entry in the registration (a specific version).
#[derive(Debug, Clone, Deserialize)]
pub struct RegistrationLeaf {
    #[serde(rename = "@id")]
    pub id: String,
    #[serde(rename = "catalogEntry")]
    pub catalog_entry: CatalogEntry,
    #[serde(rename = "packageContent")]
    pub package_content: String,
}

/// Catalog entry containing version details.
#[derive(Debug, Clone, Deserialize)]
pub struct CatalogEntry {
    #[serde(rename = "@id")]
    pub id: String,
    pub authors: Option<AuthorsField>,
    pub description: Option<String>,
    #[serde(rename = "iconUrl")]
    pub icon_url: Option<String>,
    #[serde(rename = "id")]
    pub package_id: String,
    pub language: Option<String>,
    #[serde(rename = "licenseExpression")]
    pub license_expression: Option<String>,
    #[serde(rename = "licenseUrl")]
    pub license_url: Option<String>,
    pub listed: Option<bool>,
    #[serde(rename = "minClientVersion")]
    pub min_client_version: Option<String>,
    #[serde(rename = "packageContent")]
    pub package_content: Option<String>,
    #[serde(rename = "projectUrl")]
    pub project_url: Option<String>,
    pub published: Option<String>,
    #[serde(rename = "requireLicenseAcceptance")]
    pub require_license_acceptance: Option<bool>,
    pub summary: Option<String>,
    pub tags: Option<TagsField>,
    pub title: Option<String>,
    pub version: String,
    #[serde(rename = "dependencyGroups")]
    pub dependency_groups: Option<Vec<DependencyGroup>>,
    pub deprecation: Option<Deprecation>,
    pub vulnerabilities: Option<Vec<Vulnerability>>,
}

/// Authors can be a string or array.
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum AuthorsField {
    Single(String),
    Multiple(Vec<String>),
}

impl AuthorsField {
    #[must_use]
    pub fn to_vec(&self) -> Vec<String> {
        match self {
            Self::Single(s) => s.split(',').map(|s| s.trim().to_string()).collect(),
            Self::Multiple(v) => v.clone(),
        }
    }
}

/// Tags can be a string or array.
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum TagsField {
    Single(String),
    Multiple(Vec<String>),
}

/// Dependency group for a target framework.
#[derive(Debug, Clone, Deserialize)]
pub struct DependencyGroup {
    #[serde(rename = "targetFramework")]
    pub target_framework: Option<String>,
    pub dependencies: Option<Vec<Dependency>>,
}

/// A package dependency.
#[derive(Debug, Clone, Deserialize)]
pub struct Dependency {
    #[serde(rename = "@id")]
    pub id: String,
    #[serde(rename = "id")]
    pub package_id: String,
    pub range: Option<String>,
    pub registration: Option<String>,
}

/// Deprecation information.
#[derive(Debug, Clone, Deserialize)]
pub struct Deprecation {
    pub message: Option<String>,
    pub reasons: Option<Vec<String>>,
    #[serde(rename = "alternatePackage")]
    pub alternate_package: Option<AlternatePackage>,
}

/// Alternate package suggestion for deprecated packages.
#[derive(Debug, Clone, Deserialize)]
pub struct AlternatePackage {
    pub id: String,
    pub range: Option<String>,
}

/// Vulnerability information.
#[derive(Debug, Clone, Deserialize)]
pub struct Vulnerability {
    #[serde(rename = "@id")]
    pub id: String,
    #[serde(rename = "advisoryUrl")]
    pub advisory_url: String,
    pub severity: String,
}

/// Search response.
#[derive(Debug, Clone, Deserialize)]
pub struct SearchResponse {
    #[serde(rename = "totalHits")]
    pub total_hits: u64,
    pub data: Vec<SearchResult>,
}

/// A search result entry.
#[derive(Debug, Clone, Deserialize)]
pub struct SearchResult {
    pub id: String,
    pub version: String,
    pub description: Option<String>,
    pub versions: Vec<SearchVersionInfo>,
    pub authors: Option<Vec<String>>,
    #[serde(rename = "iconUrl")]
    pub icon_url: Option<String>,
    #[serde(rename = "licenseUrl")]
    pub license_url: Option<String>,
    pub owners: Option<Vec<String>>,
    #[serde(rename = "projectUrl")]
    pub project_url: Option<String>,
    pub registration: Option<String>,
    pub summary: Option<String>,
    pub tags: Option<Vec<String>>,
    pub title: Option<String>,
    #[serde(rename = "totalDownloads")]
    pub total_downloads: Option<u64>,
    pub verified: Option<bool>,
}

/// Version info in search results.
#[derive(Debug, Clone, Deserialize)]
pub struct SearchVersionInfo {
    pub version: String,
    pub downloads: u64,
    #[serde(rename = "@id")]
    pub id: String,
}
