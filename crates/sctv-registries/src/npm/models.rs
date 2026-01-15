//! npm API response models.

use chrono::{DateTime, Utc};
use serde::Deserialize;
use std::collections::HashMap;

/// Abbreviated package response from npm registry.
#[derive(Debug, Deserialize)]
pub struct NpmAbbreviatedPackage {
    pub name: String,
    pub modified: Option<DateTime<Utc>>,
    #[serde(rename = "dist-tags")]
    pub dist_tags: HashMap<String, String>,
    pub versions: HashMap<String, NpmAbbreviatedVersion>,
}

/// Abbreviated version info.
#[derive(Debug, Deserialize)]
pub struct NpmAbbreviatedVersion {
    pub name: String,
    pub version: String,
    pub dist: NpmDist,
    pub dependencies: Option<HashMap<String, String>>,
    #[serde(rename = "devDependencies")]
    pub dev_dependencies: Option<HashMap<String, String>>,
    #[serde(rename = "peerDependencies")]
    pub peer_dependencies: Option<HashMap<String, String>>,
    #[serde(rename = "optionalDependencies")]
    pub optional_dependencies: Option<HashMap<String, String>>,
}

/// Full version response.
#[derive(Debug, Deserialize)]
pub struct NpmVersionResponse {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub main: Option<String>,
    pub repository: Option<NpmRepository>,
    pub homepage: Option<String>,
    pub keywords: Option<Vec<String>>,
    pub license: Option<serde_json::Value>,
    pub dependencies: Option<HashMap<String, String>>,
    #[serde(rename = "devDependencies")]
    pub dev_dependencies: Option<HashMap<String, String>>,
    pub dist: NpmDist,
    pub maintainers: Option<Vec<NpmMaintainer>>,
    pub deprecated: Option<String>,
    #[serde(rename = "_npmUser")]
    pub npm_user: Option<NpmUser>,
}

/// Distribution information.
#[derive(Debug, Deserialize)]
pub struct NpmDist {
    pub shasum: Option<String>,
    pub integrity: Option<String>,
    pub tarball: Option<String>,
    #[serde(rename = "fileCount")]
    pub file_count: Option<u32>,
    #[serde(rename = "unpackedSize")]
    pub unpacked_size: Option<u64>,
    pub signatures: Option<Vec<NpmSignature>>,
}

/// npm signature information.
#[derive(Debug, Deserialize)]
pub struct NpmSignature {
    pub keyid: String,
    pub sig: String,
}

/// Repository information.
#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum NpmRepository {
    Simple(String),
    Object {
        #[serde(rename = "type")]
        repo_type: Option<String>,
        url: String,
        directory: Option<String>,
    },
}

impl NpmRepository {
    /// Extracts the repository URL.
    pub fn url(&self) -> &str {
        match self {
            Self::Simple(s) => s,
            Self::Object { url, .. } => url,
        }
    }
}

/// Maintainer information.
#[derive(Debug, Deserialize)]
pub struct NpmMaintainer {
    pub name: Option<String>,
    pub email: Option<String>,
}

/// npm user information.
#[derive(Debug, Deserialize)]
pub struct NpmUser {
    pub name: String,
    pub email: Option<String>,
}

/// Full package document (used for detailed queries).
#[derive(Debug, Deserialize)]
pub struct NpmPackageDocument {
    pub name: String,
    pub description: Option<String>,
    #[serde(rename = "dist-tags")]
    pub dist_tags: HashMap<String, String>,
    pub versions: HashMap<String, NpmVersionResponse>,
    pub time: Option<HashMap<String, String>>,
    pub maintainers: Option<Vec<NpmMaintainer>>,
    pub repository: Option<NpmRepository>,
    pub homepage: Option<String>,
    pub keywords: Option<Vec<String>>,
    pub readme: Option<String>,
    pub license: Option<serde_json::Value>,
}
