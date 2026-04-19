//! Common types shared between SBOM formats.
//!
//! This module provides types that are used by both `CycloneDX` and SPDX generators,
//! ensuring consistent handling of licenses, hashes, external references, etc.

use serde::{Deserialize, Serialize};

/// Configuration options for SBOM generation.
#[derive(Debug, Clone)]
pub struct GeneratorConfig {
    /// Include development dependencies.
    pub include_dev_dependencies: bool,
    /// Include transitive dependencies.
    pub include_transitive: bool,
    /// Include hash/checksum information.
    pub include_hashes: bool,
    /// Include license information.
    pub include_licenses: bool,
    /// Include external references (VCS, issue trackers, etc.).
    pub include_external_refs: bool,
    /// Include vulnerability information.
    pub include_vulnerabilities: bool,
    /// Tool name to include in metadata.
    pub tool_name: String,
    /// Tool version to include in metadata.
    pub tool_version: String,
    /// Tool vendor to include in metadata.
    pub tool_vendor: String,
    /// Organization name for the supplier.
    pub supplier_name: Option<String>,
    /// Organization URL for the supplier.
    pub supplier_url: Option<String>,
    /// Pretty-print JSON output.
    pub pretty_print: bool,
}

impl Default for GeneratorConfig {
    fn default() -> Self {
        Self {
            include_dev_dependencies: true,
            include_transitive: true,
            include_hashes: true,
            include_licenses: true,
            include_external_refs: true,
            include_vulnerabilities: true,
            tool_name: "Supply Chain Trust Verifier".to_string(),
            tool_version: env!("CARGO_PKG_VERSION").to_string(),
            tool_vendor: "SCTV".to_string(),
            supplier_name: None,
            supplier_url: None,
            pretty_print: true,
        }
    }
}

impl GeneratorConfig {
    /// Creates a new configuration with default settings.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets whether to include development dependencies.
    #[must_use]
    pub const fn with_include_dev_dependencies(mut self, include: bool) -> Self {
        self.include_dev_dependencies = include;
        self
    }

    /// Sets whether to include transitive dependencies.
    #[must_use]
    pub const fn with_include_transitive(mut self, include: bool) -> Self {
        self.include_transitive = include;
        self
    }

    /// Sets whether to include hash information.
    #[must_use]
    pub const fn with_include_hashes(mut self, include: bool) -> Self {
        self.include_hashes = include;
        self
    }

    /// Sets whether to include license information.
    #[must_use]
    pub const fn with_include_licenses(mut self, include: bool) -> Self {
        self.include_licenses = include;
        self
    }

    /// Sets whether to include external references.
    #[must_use]
    pub const fn with_include_external_refs(mut self, include: bool) -> Self {
        self.include_external_refs = include;
        self
    }

    /// Sets whether to include vulnerability information.
    #[must_use]
    pub const fn with_include_vulnerabilities(mut self, include: bool) -> Self {
        self.include_vulnerabilities = include;
        self
    }

    /// Sets the tool information.
    #[must_use]
    pub fn with_tool(mut self, name: String, version: String, vendor: String) -> Self {
        self.tool_name = name;
        self.tool_version = version;
        self.tool_vendor = vendor;
        self
    }

    /// Sets the supplier information.
    #[must_use]
    pub fn with_supplier(mut self, name: String, url: Option<String>) -> Self {
        self.supplier_name = Some(name);
        self.supplier_url = url;
        self
    }

    /// Sets whether to pretty-print output.
    #[must_use]
    pub const fn with_pretty_print(mut self, pretty: bool) -> Self {
        self.pretty_print = pretty;
        self
    }
}

/// Hash algorithm identifiers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum HashAlgorithm {
    Md5,
    Sha1,
    Sha256,
    Sha384,
    Sha512,
    Sha3_256,
    Sha3_384,
    Sha3_512,
    Blake2b256,
    Blake2b384,
    Blake2b512,
    Blake3,
}

impl HashAlgorithm {
    /// Returns the `CycloneDX` algorithm identifier.
    #[must_use]
    pub const fn cyclonedx_id(&self) -> &'static str {
        match self {
            Self::Md5 => "MD5",
            Self::Sha1 => "SHA-1",
            Self::Sha256 => "SHA-256",
            Self::Sha384 => "SHA-384",
            Self::Sha512 => "SHA-512",
            Self::Sha3_256 => "SHA3-256",
            Self::Sha3_384 => "SHA3-384",
            Self::Sha3_512 => "SHA3-512",
            Self::Blake2b256 => "BLAKE2b-256",
            Self::Blake2b384 => "BLAKE2b-384",
            Self::Blake2b512 => "BLAKE2b-512",
            Self::Blake3 => "BLAKE3",
        }
    }

    /// Returns the SPDX algorithm identifier.
    #[must_use]
    pub const fn spdx_id(&self) -> &'static str {
        match self {
            Self::Md5 => "MD5",
            Self::Sha1 => "SHA1",
            Self::Sha256 => "SHA256",
            Self::Sha384 => "SHA384",
            Self::Sha512 => "SHA512",
            Self::Sha3_256 => "SHA3-256",
            Self::Sha3_384 => "SHA3-384",
            Self::Sha3_512 => "SHA3-512",
            Self::Blake2b256 => "BLAKE2b-256",
            Self::Blake2b384 => "BLAKE2b-384",
            Self::Blake2b512 => "BLAKE2b-512",
            Self::Blake3 => "BLAKE3",
        }
    }
}

/// A cryptographic hash value.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hash {
    /// The hash algorithm used.
    pub algorithm: HashAlgorithm,
    /// The hash value (hex-encoded).
    pub value: String,
}

impl Hash {
    /// Creates a new hash.
    #[must_use]
    pub const fn new(algorithm: HashAlgorithm, value: String) -> Self {
        Self { algorithm, value }
    }

    /// Creates a SHA-256 hash.
    #[must_use]
    pub const fn sha256(value: String) -> Self {
        Self::new(HashAlgorithm::Sha256, value)
    }

    /// Creates a SHA-512 hash.
    #[must_use]
    pub const fn sha512(value: String) -> Self {
        Self::new(HashAlgorithm::Sha512, value)
    }
}

/// Types of external references.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ExternalReferenceType {
    /// Version control system.
    Vcs,
    /// Issue tracker.
    IssueTracker,
    /// Project website.
    Website,
    /// Build system.
    BuildSystem,
    /// Distribution/package.
    Distribution,
    /// Documentation.
    Documentation,
    /// License information.
    License,
    /// Mailing list.
    MailingList,
    /// Social media.
    Social,
    /// Chat/messaging.
    Chat,
    /// Support resources.
    Support,
    /// Adversary model.
    AdversaryModel,
    /// Attestation.
    Attestation,
    /// Bill of materials.
    Bom,
    /// Security advisory.
    SecurityAdvisory,
    /// Other reference type.
    Other,
}

impl ExternalReferenceType {
    /// Returns the `CycloneDX` type identifier.
    #[must_use]
    pub const fn cyclonedx_type(&self) -> &'static str {
        match self {
            Self::Vcs => "vcs",
            Self::IssueTracker => "issue-tracker",
            Self::Website => "website",
            Self::BuildSystem => "build-system",
            Self::Distribution => "distribution",
            Self::Documentation => "documentation",
            Self::License => "license",
            Self::MailingList => "mailing-list",
            Self::Social => "social",
            Self::Chat => "chat",
            Self::Support => "support",
            Self::AdversaryModel => "adversary-model",
            Self::Attestation => "attestation",
            Self::Bom => "bom",
            Self::SecurityAdvisory => "security-advisory",
            Self::Other => "other",
        }
    }

    /// Returns the SPDX reference category.
    #[must_use]
    pub const fn spdx_category(&self) -> &'static str {
        match self {
            Self::Vcs | Self::BuildSystem => "PACKAGE-MANAGER",
            Self::Documentation | Self::Website => "OTHER",
            Self::SecurityAdvisory => "SECURITY",
            _ => "OTHER",
        }
    }
}

/// An external reference to a resource.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalReference {
    /// The type of reference.
    pub reference_type: ExternalReferenceType,
    /// The URL of the reference.
    pub url: String,
    /// Optional comment describing the reference.
    pub comment: Option<String>,
}

impl ExternalReference {
    /// Creates a new external reference.
    #[must_use]
    pub const fn new(reference_type: ExternalReferenceType, url: String) -> Self {
        Self {
            reference_type,
            url,
            comment: None,
        }
    }

    /// Creates a VCS reference.
    #[must_use]
    pub const fn vcs(url: String) -> Self {
        Self::new(ExternalReferenceType::Vcs, url)
    }

    /// Creates a website reference.
    #[must_use]
    pub const fn website(url: String) -> Self {
        Self::new(ExternalReferenceType::Website, url)
    }

    /// Sets the comment.
    #[must_use]
    pub fn with_comment(mut self, comment: String) -> Self {
        self.comment = Some(comment);
        self
    }
}

/// An SPDX license expression.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseExpression {
    /// The SPDX expression (e.g., "MIT OR Apache-2.0").
    pub expression: String,
}

impl LicenseExpression {
    /// Creates a new license expression.
    #[must_use]
    pub fn new(expression: impl Into<String>) -> Self {
        Self {
            expression: expression.into(),
        }
    }

    /// Creates an MIT license.
    #[must_use]
    pub fn mit() -> Self {
        Self::new("MIT")
    }

    /// Creates an Apache-2.0 license.
    #[must_use]
    pub fn apache2() -> Self {
        Self::new("Apache-2.0")
    }

    /// Checks if this is a valid SPDX identifier.
    #[must_use]
    pub fn is_valid_spdx(&self) -> bool {
        // Common SPDX identifiers
        let valid_ids = [
            "MIT",
            "Apache-2.0",
            "GPL-2.0",
            "GPL-2.0-only",
            "GPL-2.0-or-later",
            "GPL-3.0",
            "GPL-3.0-only",
            "GPL-3.0-or-later",
            "LGPL-2.1",
            "LGPL-3.0",
            "BSD-2-Clause",
            "BSD-3-Clause",
            "ISC",
            "MPL-2.0",
            "AGPL-3.0",
            "Unlicense",
            "CC0-1.0",
            "WTFPL",
            "Zlib",
        ];

        // Simple check - a more complete implementation would parse the expression
        let expr = self.expression.trim();
        valid_ids.contains(&expr)
            || expr.contains(" OR ")
            || expr.contains(" AND ")
            || expr.contains(" WITH ")
    }
}

/// A choice between a license expression and named licenses.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum LicenseChoice {
    /// An SPDX license expression.
    Expression(LicenseExpression),
    /// A named license (possibly non-SPDX).
    Named { name: String, url: Option<String> },
}

impl LicenseChoice {
    /// Creates from an SPDX expression.
    #[must_use]
    pub fn expression(expr: impl Into<String>) -> Self {
        Self::Expression(LicenseExpression::new(expr))
    }

    /// Creates from a named license.
    #[must_use]
    pub fn named(name: impl Into<String>) -> Self {
        Self::Named {
            name: name.into(),
            url: None,
        }
    }

    /// Creates from a named license with URL.
    #[must_use]
    pub fn named_with_url(name: impl Into<String>, url: impl Into<String>) -> Self {
        Self::Named {
            name: name.into(),
            url: Some(url.into()),
        }
    }
}

/// An organizational entity (company, team, etc.).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganizationalEntity {
    /// The name of the organization.
    pub name: String,
    /// URLs associated with the organization.
    pub urls: Vec<String>,
    /// Contact information.
    pub contacts: Vec<OrganizationalContact>,
}

impl OrganizationalEntity {
    /// Creates a new organizational entity.
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            urls: Vec::new(),
            contacts: Vec::new(),
        }
    }

    /// Adds a URL.
    #[must_use]
    pub fn with_url(mut self, url: impl Into<String>) -> Self {
        self.urls.push(url.into());
        self
    }

    /// Adds a contact.
    #[must_use]
    pub fn with_contact(mut self, contact: OrganizationalContact) -> Self {
        self.contacts.push(contact);
        self
    }
}

/// A contact person within an organization.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganizationalContact {
    /// The contact's name.
    pub name: Option<String>,
    /// The contact's email address.
    pub email: Option<String>,
    /// The contact's phone number.
    pub phone: Option<String>,
}

impl OrganizationalContact {
    /// Creates a new contact with a name.
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: Some(name.into()),
            email: None,
            phone: None,
        }
    }

    /// Creates a contact with email only.
    #[must_use]
    pub fn with_email(email: impl Into<String>) -> Self {
        Self {
            name: None,
            email: Some(email.into()),
            phone: None,
        }
    }

    /// Adds an email address.
    #[must_use]
    pub fn email(mut self, email: impl Into<String>) -> Self {
        self.email = Some(email.into());
        self
    }
}

/// Generates a unique BOM reference ID from a dependency.
#[must_use]
pub fn generate_bom_ref(ecosystem: &str, name: &str, version: &str) -> String {
    use sha2::{Digest, Sha256};

    let input = format!("{ecosystem}:{name}@{version}");
    let hash = Sha256::digest(input.as_bytes());
    format!(
        "{}-{}",
        name.replace(['/', '@', '.'], "-"),
        &hex::encode(&hash[..8])
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generator_config_builder() {
        let config = GeneratorConfig::new()
            .with_include_dev_dependencies(false)
            .with_include_hashes(true)
            .with_pretty_print(false);

        assert!(!config.include_dev_dependencies);
        assert!(config.include_hashes);
        assert!(!config.pretty_print);
    }

    #[test]
    fn test_hash_algorithms() {
        assert_eq!(HashAlgorithm::Sha256.cyclonedx_id(), "SHA-256");
        assert_eq!(HashAlgorithm::Sha256.spdx_id(), "SHA256");
    }

    #[test]
    fn test_license_expression_validation() {
        assert!(LicenseExpression::mit().is_valid_spdx());
        assert!(LicenseExpression::apache2().is_valid_spdx());
        assert!(LicenseExpression::new("MIT OR Apache-2.0").is_valid_spdx());
    }

    #[test]
    fn test_generate_bom_ref() {
        let ref1 = generate_bom_ref("npm", "lodash", "4.17.21");
        let ref2 = generate_bom_ref("npm", "lodash", "4.17.21");
        assert_eq!(ref1, ref2); // Deterministic

        let ref3 = generate_bom_ref("npm", "lodash", "4.17.20");
        assert_ne!(ref1, ref3); // Different version = different ref
    }
}
