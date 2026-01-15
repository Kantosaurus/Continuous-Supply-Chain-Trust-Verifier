//! Domain models for the Supply Chain Trust Verifier.

mod alert;
mod attestation;
mod audit_log;
mod dependency;
mod job;
mod package;
mod policy;
mod project;
mod sbom;
mod tenant;
mod user;

pub use alert::*;
pub use attestation::*;
pub use audit_log::*;
pub use dependency::*;
pub use job::*;
pub use package::*;
pub use policy::*;
pub use project::*;
pub use sbom::*;
pub use tenant::*;
pub use user::*;

use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};

#[cfg(feature = "graphql")]
use async_graphql::Enum;

/// Supported package ecosystems for supply chain monitoring.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Display, EnumString)]
#[cfg_attr(feature = "graphql", derive(Enum))]
#[strum(serialize_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum PackageEcosystem {
    Npm,
    PyPi,
    Maven,
    NuGet,
    RubyGems,
    Cargo,
    GoModules,
}

impl PackageEcosystem {
    /// Returns the Package URL type for this ecosystem.
    #[must_use]
    pub const fn purl_type(&self) -> &'static str {
        match self {
            Self::Npm => "npm",
            Self::PyPi => "pypi",
            Self::Maven => "maven",
            Self::NuGet => "nuget",
            Self::RubyGems => "gem",
            Self::Cargo => "cargo",
            Self::GoModules => "golang",
        }
    }

    /// Returns the default registry URL for this ecosystem.
    #[must_use]
    pub const fn default_registry_url(&self) -> &'static str {
        match self {
            Self::Npm => "https://registry.npmjs.org",
            Self::PyPi => "https://pypi.org",
            Self::Maven => "https://repo1.maven.org/maven2",
            Self::NuGet => "https://api.nuget.org/v3",
            Self::RubyGems => "https://rubygems.org",
            Self::Cargo => "https://crates.io",
            Self::GoModules => "https://proxy.golang.org",
        }
    }
}

/// Severity levels for alerts and policy violations.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Display,
)]
#[cfg_attr(feature = "graphql", derive(Enum))]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Info,
    Low,
    Medium,
    High,
    Critical,
}

/// Hash algorithms supported for integrity verification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Display, EnumString)]
#[serde(rename_all = "lowercase")]
pub enum HashAlgorithm {
    Sha256,
    Sha512,
    Blake3,
}
