//! SPDX 2.3 schema models.
//!
//! This module defines types that closely follow the SPDX 2.3 specification.
//! See: <https://spdx.github.io/spdx-spec/v2.3>/

use serde::{Deserialize, Serialize};

/// The root SPDX document.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Document {
    /// SPDX version (e.g., "SPDX-2.3").
    pub spdx_version: String,
    /// Data license (always CC0-1.0 for SPDX documents).
    pub data_license: String,
    /// Unique identifier for this SPDX document.
    #[serde(rename = "SPDXID")]
    pub spdx_id: String,
    /// Document name.
    pub name: String,
    /// Document namespace (unique URI).
    pub document_namespace: String,
    /// External document references.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub external_document_refs: Vec<ExternalDocumentRef>,
    /// Document creation information.
    pub creation_info: CreationInfo,
    /// Packages described in this document.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub packages: Vec<Package>,
    /// Files described in this document.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub files: Vec<File>,
    /// Snippets described in this document.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub snippets: Vec<Snippet>,
    /// Relationships between elements.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub relationships: Vec<Relationship>,
    /// Annotations.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub annotations: Vec<Annotation>,
    /// Document-level comment.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
}

impl Document {
    /// Creates a new SPDX 2.3 document.
    #[must_use]
    pub fn new(name: impl Into<String>, namespace: impl Into<String>) -> Self {
        Self {
            spdx_version: "SPDX-2.3".to_string(),
            data_license: "CC0-1.0".to_string(),
            spdx_id: "SPDXRef-DOCUMENT".to_string(),
            name: name.into(),
            document_namespace: namespace.into(),
            external_document_refs: Vec::new(),
            creation_info: CreationInfo::default(),
            packages: Vec::new(),
            files: Vec::new(),
            snippets: Vec::new(),
            relationships: Vec::new(),
            annotations: Vec::new(),
            comment: None,
        }
    }

    /// Adds a package.
    pub fn add_package(&mut self, package: Package) {
        self.packages.push(package);
    }

    /// Adds a relationship.
    pub fn add_relationship(&mut self, relationship: Relationship) {
        self.relationships.push(relationship);
    }

    /// Sets the creation info.
    #[must_use]
    pub fn with_creation_info(mut self, info: CreationInfo) -> Self {
        self.creation_info = info;
        self
    }
}

/// External document reference.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExternalDocumentRef {
    /// Reference ID (e.g., "DocumentRef-ext").
    pub external_document_id: String,
    /// Document URI.
    pub spdx_document: String,
    /// Document checksum.
    pub checksum: Checksum,
}

/// Document creation information.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreationInfo {
    /// ISO 8601 timestamp of creation.
    pub created: String,
    /// List of creators (tools, people, organizations).
    pub creators: Vec<String>,
    /// License list version used.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub license_list_version: Option<String>,
    /// Comment on creation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
}

impl CreationInfo {
    /// Creates new creation info with current timestamp.
    #[must_use]
    pub fn new() -> Self {
        Self {
            created: chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string(),
            creators: Vec::new(),
            license_list_version: Some("3.21".to_string()),
            comment: None,
        }
    }

    /// Adds a tool creator.
    pub fn add_tool(&mut self, name: &str, version: &str) {
        self.creators.push(format!("Tool: {name}-{version}"));
    }

    /// Adds an organization creator.
    pub fn add_organization(&mut self, name: &str) {
        self.creators.push(format!("Organization: {name}"));
    }

    /// Adds a person creator.
    pub fn add_person(&mut self, name: &str, email: Option<&str>) {
        let creator = if let Some(email) = email {
            format!("Person: {name} ({email})")
        } else {
            format!("Person: {name}")
        };
        self.creators.push(creator);
    }
}

impl Default for CreationInfo {
    fn default() -> Self {
        Self::new()
    }
}

/// An SPDX package.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Package {
    /// Unique identifier within this document.
    #[serde(rename = "SPDXID")]
    pub spdx_id: String,
    /// Package name.
    pub name: String,
    /// Package version.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version_info: Option<String>,
    /// Package file name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub package_file_name: Option<String>,
    /// Package supplier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supplier: Option<String>,
    /// Package originator.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub originator: Option<String>,
    /// Download location.
    pub download_location: String,
    /// Whether files have been analyzed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub files_analyzed: Option<bool>,
    /// Package verification code.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub package_verification_code: Option<PackageVerificationCode>,
    /// Package checksums.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub checksums: Vec<Checksum>,
    /// Package home page.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub homepage: Option<String>,
    /// Source information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_info: Option<String>,
    /// Concluded license.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub license_concluded: Option<String>,
    /// License info from files.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub license_info_from_files: Vec<String>,
    /// Declared license.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub license_declared: Option<String>,
    /// License comments.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub license_comments: Option<String>,
    /// Copyright text.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub copyright_text: Option<String>,
    /// Package summary description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    /// Package detailed description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Package comment.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    /// External references.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub external_refs: Vec<ExternalRef>,
    /// Attribution text.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub attribution_text: Vec<String>,
    /// Primary package purpose.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub primary_package_purpose: Option<String>,
    /// Release date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub release_date: Option<String>,
    /// Built date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub built_date: Option<String>,
    /// Valid until date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub valid_until_date: Option<String>,
}

impl Package {
    /// Creates a new package.
    #[must_use]
    pub fn new(spdx_id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            spdx_id: spdx_id.into(),
            name: name.into(),
            version_info: None,
            package_file_name: None,
            supplier: None,
            originator: None,
            download_location: "NOASSERTION".to_string(),
            files_analyzed: Some(false),
            package_verification_code: None,
            checksums: Vec::new(),
            homepage: None,
            source_info: None,
            license_concluded: Some("NOASSERTION".to_string()),
            license_info_from_files: Vec::new(),
            license_declared: Some("NOASSERTION".to_string()),
            license_comments: None,
            copyright_text: Some("NOASSERTION".to_string()),
            summary: None,
            description: None,
            comment: None,
            external_refs: Vec::new(),
            attribution_text: Vec::new(),
            primary_package_purpose: None,
            release_date: None,
            built_date: None,
            valid_until_date: None,
        }
    }

    /// Sets the version.
    #[must_use]
    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.version_info = Some(version.into());
        self
    }

    /// Sets the download location.
    #[must_use]
    pub fn with_download_location(mut self, url: impl Into<String>) -> Self {
        self.download_location = url.into();
        self
    }

    /// Sets the primary purpose.
    #[must_use]
    pub fn with_purpose(mut self, purpose: PackagePurpose) -> Self {
        self.primary_package_purpose = Some(purpose.as_str().to_string());
        self
    }

    /// Adds a checksum.
    pub fn add_checksum(&mut self, checksum: Checksum) {
        self.checksums.push(checksum);
    }

    /// Adds an external reference.
    pub fn add_external_ref(&mut self, ext_ref: ExternalRef) {
        self.external_refs.push(ext_ref);
    }

    /// Sets the supplier.
    #[must_use]
    pub fn with_supplier(mut self, supplier: impl Into<String>) -> Self {
        self.supplier = Some(supplier.into());
        self
    }
}

/// Package purpose types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackagePurpose {
    Application,
    Framework,
    Library,
    Container,
    OperatingSystem,
    Device,
    Firmware,
    Source,
    Archive,
    File,
    Install,
    Other,
}

impl PackagePurpose {
    /// Returns the SPDX string representation.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Application => "APPLICATION",
            Self::Framework => "FRAMEWORK",
            Self::Library => "LIBRARY",
            Self::Container => "CONTAINER",
            Self::OperatingSystem => "OPERATING-SYSTEM",
            Self::Device => "DEVICE",
            Self::Firmware => "FIRMWARE",
            Self::Source => "SOURCE",
            Self::Archive => "ARCHIVE",
            Self::File => "FILE",
            Self::Install => "INSTALL",
            Self::Other => "OTHER",
        }
    }
}

/// Package verification code.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PackageVerificationCode {
    /// The verification code value.
    pub package_verification_code_value: String,
    /// Files excluded from verification.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub package_verification_code_excluded_files: Vec<String>,
}

/// A checksum/hash.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Checksum {
    /// Hash algorithm.
    pub algorithm: String,
    /// Hash value.
    pub checksum_value: String,
}

impl Checksum {
    /// Creates a new checksum.
    #[must_use]
    pub fn new(algorithm: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            algorithm: algorithm.into(),
            checksum_value: value.into(),
        }
    }

    /// Creates a SHA-256 checksum.
    #[must_use]
    pub fn sha256(value: impl Into<String>) -> Self {
        Self::new("SHA256", value)
    }

    /// Creates a SHA-512 checksum.
    #[must_use]
    pub fn sha512(value: impl Into<String>) -> Self {
        Self::new("SHA512", value)
    }

    /// Creates a SHA-1 checksum.
    #[must_use]
    pub fn sha1(value: impl Into<String>) -> Self {
        Self::new("SHA1", value)
    }

    /// Creates an MD5 checksum.
    #[must_use]
    pub fn md5(value: impl Into<String>) -> Self {
        Self::new("MD5", value)
    }
}

/// External reference for a package.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExternalRef {
    /// Reference category.
    pub reference_category: String,
    /// Reference type.
    pub reference_type: String,
    /// Reference locator (URL, PURL, etc.).
    pub reference_locator: String,
    /// Comment.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
}

impl ExternalRef {
    /// Creates a new external reference.
    #[must_use]
    pub fn new(
        category: impl Into<String>,
        ref_type: impl Into<String>,
        locator: impl Into<String>,
    ) -> Self {
        Self {
            reference_category: category.into(),
            reference_type: ref_type.into(),
            reference_locator: locator.into(),
            comment: None,
        }
    }

    /// Creates a PURL reference.
    #[must_use]
    pub fn purl(purl: impl Into<String>) -> Self {
        Self::new("PACKAGE-MANAGER", "purl", purl)
    }

    /// Creates a CPE reference.
    #[must_use]
    pub fn cpe(cpe: impl Into<String>) -> Self {
        Self::new("SECURITY", "cpe23Type", cpe)
    }

    /// Creates a security advisory reference.
    #[must_use]
    pub fn advisory(url: impl Into<String>) -> Self {
        Self::new("SECURITY", "advisory", url)
    }

    /// Sets the comment.
    #[must_use]
    pub fn with_comment(mut self, comment: impl Into<String>) -> Self {
        self.comment = Some(comment.into());
        self
    }
}

/// An SPDX file.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct File {
    /// Unique identifier.
    #[serde(rename = "SPDXID")]
    pub spdx_id: String,
    /// File name.
    pub file_name: String,
    /// File checksums.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub checksums: Vec<Checksum>,
    /// License concluded.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub license_concluded: Option<String>,
    /// License info in file.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub license_info_in_files: Vec<String>,
    /// Copyright text.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub copyright_text: Option<String>,
    /// File comment.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    /// File types.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub file_types: Vec<String>,
}

/// An SPDX snippet.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Snippet {
    /// Unique identifier.
    #[serde(rename = "SPDXID")]
    pub spdx_id: String,
    /// File containing this snippet.
    pub snippet_from_file: String,
    /// Byte range.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub ranges: Vec<SnippetRange>,
    /// License concluded.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub license_concluded: Option<String>,
    /// Copyright text.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub copyright_text: Option<String>,
    /// Snippet comment.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    /// Snippet name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

/// Snippet range.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SnippetRange {
    /// Start pointer.
    pub start_pointer: RangePointer,
    /// End pointer.
    pub end_pointer: RangePointer,
}

/// Range pointer.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RangePointer {
    /// Offset.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<u64>,
    /// Line number.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_number: Option<u64>,
    /// Reference to the file.
    pub reference: String,
}

/// A relationship between SPDX elements.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Relationship {
    /// Source element.
    pub spdx_element_id: String,
    /// Relationship type.
    pub relationship_type: String,
    /// Target element.
    pub related_spdx_element: String,
    /// Comment.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
}

impl Relationship {
    /// Creates a new relationship.
    #[must_use]
    pub fn new(
        source: impl Into<String>,
        rel_type: RelationshipType,
        target: impl Into<String>,
    ) -> Self {
        Self {
            spdx_element_id: source.into(),
            relationship_type: rel_type.as_str().to_string(),
            related_spdx_element: target.into(),
            comment: None,
        }
    }

    /// Creates a DESCRIBES relationship.
    #[must_use]
    pub fn describes(source: impl Into<String>, target: impl Into<String>) -> Self {
        Self::new(source, RelationshipType::Describes, target)
    }

    /// Creates a `DEPENDS_ON` relationship.
    #[must_use]
    pub fn depends_on(source: impl Into<String>, target: impl Into<String>) -> Self {
        Self::new(source, RelationshipType::DependsOn, target)
    }

    /// Creates a `DEPENDENCY_OF` relationship.
    #[must_use]
    pub fn dependency_of(source: impl Into<String>, target: impl Into<String>) -> Self {
        Self::new(source, RelationshipType::DependencyOf, target)
    }

    /// Creates a `DEV_DEPENDENCY_OF` relationship.
    #[must_use]
    pub fn dev_dependency_of(source: impl Into<String>, target: impl Into<String>) -> Self {
        Self::new(source, RelationshipType::DevDependencyOf, target)
    }
}

/// Relationship types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelationshipType {
    Describes,
    DescribedBy,
    Contains,
    ContainedBy,
    DependsOn,
    DependencyOf,
    DevDependencyOf,
    OptionalDependencyOf,
    BuildToolOf,
    DevToolOf,
    TestToolOf,
    DocumentationOf,
    OptionalComponentOf,
    PackageOf,
    Generates,
    GeneratedFrom,
    AncestorOf,
    DescendantOf,
    VariantOf,
    DistributionArtifact,
    PatchFor,
    CopyOf,
    FileAdded,
    FileDeleted,
    FileModified,
    ExpandedFromArchive,
    DynamicLink,
    StaticLink,
    DataFileOf,
    TestCaseOf,
    BuildRequirementOf,
    RuntimeRequirementOf,
    ExampleOf,
    Other,
}

impl RelationshipType {
    /// Returns the SPDX string representation.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Describes => "DESCRIBES",
            Self::DescribedBy => "DESCRIBED_BY",
            Self::Contains => "CONTAINS",
            Self::ContainedBy => "CONTAINED_BY",
            Self::DependsOn => "DEPENDS_ON",
            Self::DependencyOf => "DEPENDENCY_OF",
            Self::DevDependencyOf => "DEV_DEPENDENCY_OF",
            Self::OptionalDependencyOf => "OPTIONAL_DEPENDENCY_OF",
            Self::BuildToolOf => "BUILD_TOOL_OF",
            Self::DevToolOf => "DEV_TOOL_OF",
            Self::TestToolOf => "TEST_TOOL_OF",
            Self::DocumentationOf => "DOCUMENTATION_OF",
            Self::OptionalComponentOf => "OPTIONAL_COMPONENT_OF",
            Self::PackageOf => "PACKAGE_OF",
            Self::Generates => "GENERATES",
            Self::GeneratedFrom => "GENERATED_FROM",
            Self::AncestorOf => "ANCESTOR_OF",
            Self::DescendantOf => "DESCENDANT_OF",
            Self::VariantOf => "VARIANT_OF",
            Self::DistributionArtifact => "DISTRIBUTION_ARTIFACT",
            Self::PatchFor => "PATCH_FOR",
            Self::CopyOf => "COPY_OF",
            Self::FileAdded => "FILE_ADDED",
            Self::FileDeleted => "FILE_DELETED",
            Self::FileModified => "FILE_MODIFIED",
            Self::ExpandedFromArchive => "EXPANDED_FROM_ARCHIVE",
            Self::DynamicLink => "DYNAMIC_LINK",
            Self::StaticLink => "STATIC_LINK",
            Self::DataFileOf => "DATA_FILE_OF",
            Self::TestCaseOf => "TEST_CASE_OF",
            Self::BuildRequirementOf => "BUILD_REQUIREMENT_OF",
            Self::RuntimeRequirementOf => "RUNTIME_REQUIREMENT_OF",
            Self::ExampleOf => "EXAMPLE_OF",
            Self::Other => "OTHER",
        }
    }
}

/// An annotation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Annotation {
    /// Annotation ID.
    pub annotation_date: String,
    /// Annotation type.
    pub annotation_type: String,
    /// Annotator.
    pub annotator: String,
    /// Comment.
    pub comment: String,
}
