//! Crates.io API response models.

use serde::Deserialize;
use std::collections::HashMap;

/// Response from /api/v1/crates/{name}
#[derive(Debug, Clone, Deserialize)]
pub struct CrateResponse {
    #[serde(rename = "crate")]
    pub krate: CrateData,
    pub versions: Vec<CrateVersion>,
    pub keywords: Option<Vec<Keyword>>,
    pub categories: Option<Vec<Category>>,
}

/// Core crate metadata.
#[derive(Debug, Clone, Deserialize)]
pub struct CrateData {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub homepage: Option<String>,
    pub repository: Option<String>,
    pub documentation: Option<String>,
    pub downloads: u64,
    pub recent_downloads: Option<u64>,
    pub max_version: String,
    pub max_stable_version: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Version metadata.
#[derive(Debug, Clone, Deserialize)]
pub struct CrateVersion {
    pub id: u64,
    #[serde(rename = "crate")]
    pub crate_name: String,
    pub num: String,
    pub dl_path: String,
    pub readme_path: Option<String>,
    pub yanked: bool,
    pub license: Option<String>,
    pub crate_size: Option<u64>,
    pub checksum: String,
    pub created_at: String,
    pub updated_at: String,
    pub downloads: u64,
    pub features: HashMap<String, Vec<String>>,
    pub rust_version: Option<String>,
}

/// Keyword metadata.
#[derive(Debug, Clone, Deserialize)]
pub struct Keyword {
    pub id: String,
    pub keyword: String,
    pub crates_cnt: u32,
}

/// Category metadata.
#[derive(Debug, Clone, Deserialize)]
pub struct Category {
    pub id: String,
    pub category: String,
    pub slug: String,
    pub description: String,
    pub crates_cnt: u32,
}

/// Response from /api/v1/crates/{name}/{version}
#[derive(Debug, Clone, Deserialize)]
pub struct VersionResponse {
    pub version: CrateVersion,
}

/// Response from /api/v1/crates/{name}/{version}/dependencies
#[derive(Debug, Clone, Deserialize)]
pub struct DependenciesResponse {
    pub dependencies: Vec<CrateDependency>,
}

/// Dependency information.
#[derive(Debug, Clone, Deserialize)]
pub struct CrateDependency {
    pub id: u64,
    pub version_id: u64,
    pub crate_id: String,
    pub req: String,
    pub optional: bool,
    pub default_features: bool,
    pub features: Vec<String>,
    pub target: Option<String>,
    pub kind: String, // "normal", "dev", "build"
}

/// Response from /api/v1/crates/{name}/owners
#[derive(Debug, Clone, Deserialize)]
pub struct OwnersResponse {
    pub users: Vec<Owner>,
}

/// Crate owner information.
#[derive(Debug, Clone, Deserialize)]
pub struct Owner {
    pub id: u64,
    pub login: String,
    pub name: Option<String>,
    pub avatar: Option<String>,
    pub url: Option<String>,
    pub kind: String, // "user" or "team"
}

/// Search response.
#[derive(Debug, Clone, Deserialize)]
pub struct SearchResponse {
    pub crates: Vec<SearchCrate>,
    pub meta: SearchMeta,
}

/// Crate in search results (abbreviated).
#[derive(Debug, Clone, Deserialize)]
pub struct SearchCrate {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub downloads: u64,
    pub recent_downloads: Option<u64>,
    pub max_version: String,
    pub max_stable_version: Option<String>,
    pub newest_version: String,
}

/// Search metadata.
#[derive(Debug, Clone, Deserialize)]
pub struct SearchMeta {
    pub total: u64,
}
