//! # SCTV SBOM
//!
//! Software Bill of Materials generation for Supply Chain Trust Verifier.
//!
//! This crate provides SBOM generation in two industry-standard formats:
//!
//! - **`CycloneDX` 1.5** - OWASP standard with rich component metadata
//! - **SPDX 2.3** - Linux Foundation standard for license compliance
//!
//! ## Features
//!
//! - Complete schema compliance for both formats
//! - Package URL (purl) generation for all supported ecosystems
//! - License expression handling (SPDX identifiers)
//! - External reference linking (VCS, issue trackers, etc.)
//! - Dependency relationship tracking
//! - Hash/checksum embedding
//! - Vulnerability correlation
//!
//! ## Usage
//!
//! ```ignore
//! use sctv_sbom::{generate, SbomFormat, GeneratorConfig};
//!
//! let config = GeneratorConfig::default()
//!     .with_include_dev_dependencies(false)
//!     .with_include_hashes(true);
//!
//! let sbom = generate(&project, &dependencies, SbomFormat::CycloneDx, &config)?;
//! println!("{}", sbom.content);
//! ```

pub mod common;
pub mod cyclonedx;
pub mod spdx;

use sctv_core::{Dependency, Project};
use thiserror::Error;

pub use common::{
    ExternalReference, ExternalReferenceType, GeneratorConfig, Hash, HashAlgorithm, LicenseChoice,
    LicenseExpression, OrganizationalContact, OrganizationalEntity,
};
pub use cyclonedx::CycloneDxGenerator;
pub use spdx::SpdxGenerator;

/// Errors that can occur during SBOM generation.
#[derive(Debug, Error)]
pub enum SbomError {
    /// Failed to generate the SBOM.
    #[error("Generation failed: {0}")]
    GenerationFailed(String),

    /// Failed to serialize the SBOM.
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Invalid input data.
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    /// Unsupported format or feature.
    #[error("Unsupported: {0}")]
    Unsupported(String),
}

/// Result type for SBOM operations.
pub type SbomResult<T> = Result<T, SbomError>;

/// SBOM output format.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SbomFormat {
    /// `CycloneDX` 1.5 JSON format.
    CycloneDx,
    /// `CycloneDX` 1.5 XML format.
    CycloneDxXml,
    /// SPDX 2.3 JSON format.
    Spdx,
    /// SPDX 2.3 tag-value format.
    SpdxTagValue,
}

impl SbomFormat {
    /// Returns the file extension for this format.
    #[must_use]
    pub const fn extension(&self) -> &'static str {
        match self {
            Self::CycloneDx => "cdx.json",
            Self::CycloneDxXml => "cdx.xml",
            Self::Spdx => "spdx.json",
            Self::SpdxTagValue => "spdx",
        }
    }

    /// Returns the MIME type for this format.
    #[must_use]
    pub const fn mime_type(&self) -> &'static str {
        match self {
            Self::CycloneDx => "application/vnd.cyclonedx+json",
            Self::CycloneDxXml => "application/vnd.cyclonedx+xml",
            Self::Spdx => "application/spdx+json",
            Self::SpdxTagValue => "text/spdx",
        }
    }
}

/// Generated SBOM output.
#[derive(Debug, Clone)]
pub struct SbomOutput {
    /// The format of this SBOM.
    pub format: SbomFormat,
    /// The serialized SBOM content.
    pub content: String,
    /// When this SBOM was generated.
    pub generated_at: chrono::DateTime<chrono::Utc>,
    /// Serial number or document ID.
    pub serial_number: String,
    /// Number of components included.
    pub component_count: usize,
}

/// Trait for SBOM generators.
pub trait SbomGenerator {
    /// Returns the format this generator produces.
    fn format(&self) -> SbomFormat;

    /// Generates an SBOM from project and dependencies.
    fn generate(
        &self,
        project: &Project,
        dependencies: &[Dependency],
        config: &GeneratorConfig,
    ) -> SbomResult<SbomOutput>;
}

/// Generate an SBOM from project dependencies.
///
/// This is the main entry point for SBOM generation.
///
/// # Arguments
///
/// * `project` - The project to generate an SBOM for
/// * `dependencies` - The project's dependencies
/// * `format` - The output format
/// * `config` - Generator configuration options
///
/// # Returns
///
/// The generated SBOM output or an error.
pub fn generate(
    project: &Project,
    dependencies: &[Dependency],
    format: SbomFormat,
    config: &GeneratorConfig,
) -> SbomResult<SbomOutput> {
    match format {
        SbomFormat::CycloneDx | SbomFormat::CycloneDxXml => {
            let generator = CycloneDxGenerator::new(format == SbomFormat::CycloneDxXml);
            generator.generate(project, dependencies, config)
        }
        SbomFormat::Spdx | SbomFormat::SpdxTagValue => {
            let generator = SpdxGenerator::new(format == SbomFormat::SpdxTagValue);
            generator.generate(project, dependencies, config)
        }
    }
}

/// Generate an SBOM with default configuration.
///
/// Convenience function using default settings.
pub fn generate_default(
    project: &Project,
    dependencies: &[Dependency],
    format: SbomFormat,
) -> SbomResult<SbomOutput> {
    generate(project, dependencies, format, &GeneratorConfig::default())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sbom_format_extension() {
        assert_eq!(SbomFormat::CycloneDx.extension(), "cdx.json");
        assert_eq!(SbomFormat::Spdx.extension(), "spdx.json");
    }

    #[test]
    fn test_sbom_format_mime_type() {
        assert_eq!(
            SbomFormat::CycloneDx.mime_type(),
            "application/vnd.cyclonedx+json"
        );
        assert_eq!(SbomFormat::Spdx.mime_type(), "application/spdx+json");
    }
}
