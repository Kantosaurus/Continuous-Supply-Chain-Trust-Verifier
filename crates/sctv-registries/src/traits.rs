//! Registry client trait definitions.

use async_trait::async_trait;
use bytes::Bytes;
use sctv_core::{Package, PackageEcosystem, PackageVersion};
use thiserror::Error;
use url::Url;

/// Errors that can occur during registry operations.
#[derive(Debug, Error)]
pub enum RegistryError {
    #[error("Package not found: {0}")]
    PackageNotFound(String),

    #[error("Version not found: {0}@{1}")]
    VersionNotFound(String, String),

    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("Rate limited")]
    RateLimited,

    #[error("Registry unavailable: {0}")]
    Unavailable(String),

    #[error("Cache error: {0}")]
    Cache(String),
}

/// Result type for registry operations.
pub type RegistryResult<T> = Result<T, RegistryError>;

/// Metadata about a package from a registry.
#[derive(Debug, Clone)]
pub struct PackageMetadata {
    pub package: Package,
    pub available_versions: Vec<String>,
    pub latest_version: Option<String>,
}

/// Metadata about a specific package version.
#[derive(Debug, Clone)]
pub struct VersionMetadata {
    pub version: PackageVersion,
    pub download_url: Option<Url>,
}

/// Trait for interacting with package registries.
#[async_trait]
pub trait RegistryClient: Send + Sync {
    /// Returns the ecosystem this client handles.
    fn ecosystem(&self) -> PackageEcosystem;

    /// Returns the base URL for this registry.
    fn base_url(&self) -> &Url;

    /// Fetches metadata for a package.
    async fn get_package(&self, name: &str) -> RegistryResult<PackageMetadata>;

    /// Fetches metadata for a specific version.
    async fn get_version(&self, name: &str, version: &str) -> RegistryResult<VersionMetadata>;

    /// Downloads the package archive for a specific version.
    async fn download_package(&self, name: &str, version: &str) -> RegistryResult<Bytes>;

    /// Lists popular packages in this ecosystem.
    async fn list_popular(&self, limit: usize) -> RegistryResult<Vec<String>>;

    /// Checks if a package exists.
    async fn package_exists(&self, name: &str) -> RegistryResult<bool>;

    /// Returns the download URL for a package version.
    async fn get_download_url(&self, name: &str, version: &str) -> RegistryResult<Url>;
}

/// Factory for creating registry clients.
pub struct RegistryClientFactory;

impl RegistryClientFactory {
    /// Creates a registry client for the given ecosystem.
    #[must_use]
    pub fn create(ecosystem: PackageEcosystem) -> Box<dyn RegistryClient> {
        match ecosystem {
            PackageEcosystem::Npm => Box::new(crate::npm::NpmClient::new()),
            PackageEcosystem::PyPi => Box::new(crate::pypi::PyPiClient::new()),
            PackageEcosystem::Maven => Box::new(crate::maven::MavenClient::new()),
            PackageEcosystem::NuGet => Box::new(crate::nuget::NuGetClient::new()),
            PackageEcosystem::RubyGems => Box::new(crate::rubygems::RubyGemsClient::new()),
            PackageEcosystem::Cargo => Box::new(crate::cargo::CargoClient::new()),
            PackageEcosystem::GoModules => Box::new(crate::go_modules::GoModulesClient::new()),
        }
    }
}
