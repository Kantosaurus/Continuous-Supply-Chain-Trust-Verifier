//! RubyGems API response models.

use serde::Deserialize;

/// Response from /api/v1/gems/{name}.json
#[derive(Debug, Clone, Deserialize)]
pub struct GemInfo {
    pub name: String,
    pub downloads: u64,
    pub version: String,
    pub version_created_at: Option<String>,
    pub version_downloads: u64,
    pub platform: String,
    pub authors: Option<String>,
    pub info: Option<String>,
    pub licenses: Option<Vec<String>>,
    pub metadata: GemMetadata,
    #[serde(rename = "yanked")]
    pub is_yanked: bool,
    pub sha: String,
    pub project_uri: Option<String>,
    pub gem_uri: Option<String>,
    pub homepage_uri: Option<String>,
    pub wiki_uri: Option<String>,
    pub documentation_uri: Option<String>,
    pub mailing_list_uri: Option<String>,
    pub source_code_uri: Option<String>,
    pub bug_tracker_uri: Option<String>,
    pub changelog_uri: Option<String>,
    pub funding_uri: Option<String>,
    pub dependencies: GemDependencies,
}

/// Gem metadata from rubygems_metadata.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct GemMetadata {
    pub bug_tracker_uri: Option<String>,
    pub changelog_uri: Option<String>,
    pub documentation_uri: Option<String>,
    pub funding_uri: Option<String>,
    pub homepage_uri: Option<String>,
    pub mailing_list_uri: Option<String>,
    pub source_code_uri: Option<String>,
    pub wiki_uri: Option<String>,
}

/// Dependencies for a gem.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct GemDependencies {
    #[serde(default)]
    pub development: Vec<GemDependency>,
    #[serde(default)]
    pub runtime: Vec<GemDependency>,
}

/// A single dependency.
#[derive(Debug, Clone, Deserialize)]
pub struct GemDependency {
    pub name: String,
    pub requirements: String,
}

/// Response from /api/v1/versions/{name}.json
#[derive(Debug, Clone, Deserialize)]
pub struct VersionInfo {
    pub authors: Option<String>,
    pub built_at: String,
    pub created_at: String,
    pub description: Option<String>,
    pub downloads_count: u64,
    pub metadata: GemMetadata,
    pub number: String,
    pub summary: Option<String>,
    pub platform: String,
    pub rubygems_version: Option<String>,
    pub ruby_version: Option<String>,
    pub prerelease: bool,
    pub licenses: Option<Vec<String>>,
    pub requirements: Option<Vec<String>>,
    pub sha: String,
    #[serde(rename = "yanked")]
    pub is_yanked: bool,
}

/// Response from search endpoint.
#[derive(Debug, Clone, Deserialize)]
pub struct SearchResult {
    pub name: String,
    pub downloads: u64,
    pub version: String,
    pub version_downloads: u64,
    pub platform: String,
    pub authors: Option<String>,
    pub info: Option<String>,
    pub licenses: Option<Vec<String>>,
    pub project_uri: Option<String>,
    pub gem_uri: Option<String>,
    pub homepage_uri: Option<String>,
    pub source_code_uri: Option<String>,
    pub documentation_uri: Option<String>,
}

/// Response from /api/v1/owners/{name}.json
#[derive(Debug, Clone, Deserialize)]
pub struct Owner {
    pub id: u64,
    pub handle: String,
    pub email: Option<String>,
}

/// Response from /api/v1/gems/{name}/reverse_dependencies.json
#[derive(Debug, Clone, Deserialize)]
pub struct ReverseDependency {
    pub name: String,
    pub downloads: u64,
    pub version: String,
}
