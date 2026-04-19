//! `CycloneDX` 1.5 schema models.
//!
//! This module defines types that closely follow the `CycloneDX` 1.5 specification.
//! See: <https://cyclonedx.org/docs/1.5/json>/

use serde::{Deserialize, Serialize};

/// The root `CycloneDX` BOM document.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Bom {
    /// Must be "`CycloneDX`".
    pub bom_format: String,
    /// The spec version (1.5).
    pub spec_version: String,
    /// The BOM serial number (URN UUID).
    pub serial_number: Option<String>,
    /// The BOM version (increments on updates).
    pub version: u32,
    /// BOM metadata.
    pub metadata: Option<Metadata>,
    /// The list of components.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub components: Vec<Component>,
    /// Dependency relationships.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub dependencies: Vec<Dependency>,
    /// External references for the BOM itself.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub external_references: Vec<ExternalReference>,
    /// Vulnerability information.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub vulnerabilities: Vec<Vulnerability>,
    /// Compositions (completeness assertions).
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub compositions: Vec<Composition>,
}

impl Bom {
    /// Creates a new `CycloneDX` 1.5 BOM.
    #[must_use]
    pub fn new() -> Self {
        Self {
            bom_format: "CycloneDX".to_string(),
            spec_version: "1.5".to_string(),
            serial_number: None,
            version: 1,
            metadata: None,
            components: Vec::new(),
            dependencies: Vec::new(),
            external_references: Vec::new(),
            vulnerabilities: Vec::new(),
            compositions: Vec::new(),
        }
    }

    /// Sets the serial number.
    #[must_use]
    pub fn with_serial_number(mut self, serial: String) -> Self {
        self.serial_number = Some(serial);
        self
    }

    /// Sets the metadata.
    #[must_use]
    pub fn with_metadata(mut self, metadata: Metadata) -> Self {
        self.metadata = Some(metadata);
        self
    }

    /// Adds a component.
    pub fn add_component(&mut self, component: Component) {
        self.components.push(component);
    }

    /// Adds a dependency relationship.
    pub fn add_dependency(&mut self, dependency: Dependency) {
        self.dependencies.push(dependency);
    }
}

impl Default for Bom {
    fn default() -> Self {
        Self::new()
    }
}

/// BOM metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Metadata {
    /// Timestamp of BOM creation.
    pub timestamp: Option<String>,
    /// Tools used to create the BOM.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub tools: Vec<Tool>,
    /// Authors of the BOM.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub authors: Vec<OrganizationalContact>,
    /// The main component (application) this BOM describes.
    pub component: Option<Component>,
    /// The organization that manufactured the component.
    pub manufacture: Option<OrganizationalEntity>,
    /// The organization that supplies the component.
    pub supplier: Option<OrganizationalEntity>,
    /// Licenses for the BOM itself.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub licenses: Vec<LicenseChoice>,
    /// Properties.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub properties: Vec<Property>,
}

impl Metadata {
    /// Creates new metadata with the current timestamp.
    #[must_use]
    pub fn new() -> Self {
        Self {
            timestamp: Some(chrono::Utc::now().to_rfc3339()),
            tools: Vec::new(),
            authors: Vec::new(),
            component: None,
            manufacture: None,
            supplier: None,
            licenses: Vec::new(),
            properties: Vec::new(),
        }
    }

    /// Adds a tool.
    pub fn add_tool(&mut self, tool: Tool) {
        self.tools.push(tool);
    }

    /// Sets the main component.
    #[must_use]
    pub fn with_component(mut self, component: Component) -> Self {
        self.component = Some(component);
        self
    }

    /// Sets the supplier.
    #[must_use]
    pub fn with_supplier(mut self, supplier: OrganizationalEntity) -> Self {
        self.supplier = Some(supplier);
        self
    }
}

impl Default for Metadata {
    fn default() -> Self {
        Self::new()
    }
}

/// A tool used to create the BOM.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tool {
    /// Tool vendor.
    pub vendor: Option<String>,
    /// Tool name.
    pub name: Option<String>,
    /// Tool version.
    pub version: Option<String>,
    /// Tool hashes.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub hashes: Vec<Hash>,
    /// External references for the tool.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub external_references: Vec<ExternalReference>,
}

impl Tool {
    /// Creates a new tool.
    #[must_use]
    pub fn new(name: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            vendor: None,
            name: Some(name.into()),
            version: Some(version.into()),
            hashes: Vec::new(),
            external_references: Vec::new(),
        }
    }

    /// Sets the vendor.
    #[must_use]
    pub fn with_vendor(mut self, vendor: impl Into<String>) -> Self {
        self.vendor = Some(vendor.into());
        self
    }
}

/// A software component.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Component {
    /// Component type.
    #[serde(rename = "type")]
    pub component_type: ComponentType,
    /// MIME type (for files).
    pub mime_type: Option<String>,
    /// Unique reference for this component within the BOM.
    #[serde(rename = "bom-ref")]
    pub bom_ref: Option<String>,
    /// The organization that supplies the component.
    pub supplier: Option<OrganizationalEntity>,
    /// The organization that authored the component.
    pub author: Option<String>,
    /// The component publisher.
    pub publisher: Option<String>,
    /// The component group (e.g., Maven groupId).
    pub group: Option<String>,
    /// The component name.
    pub name: String,
    /// The component version.
    pub version: Option<String>,
    /// Component description.
    pub description: Option<String>,
    /// The scope of the component.
    pub scope: Option<ComponentScope>,
    /// Hashes of the component.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub hashes: Vec<Hash>,
    /// Licenses.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub licenses: Vec<LicenseChoice>,
    /// Copyright text.
    pub copyright: Option<String>,
    /// Package URL (purl).
    pub purl: Option<String>,
    /// External references.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub external_references: Vec<ExternalReference>,
    /// Properties.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub properties: Vec<Property>,
    /// Nested components.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub components: Vec<Self>,
    /// Evidence supporting the component's identity.
    pub evidence: Option<ComponentEvidence>,
    /// Release notes.
    pub release_notes: Option<ReleaseNotes>,
}

impl Component {
    /// Creates a new library component.
    #[must_use]
    pub fn library(name: impl Into<String>) -> Self {
        Self {
            component_type: ComponentType::Library,
            mime_type: None,
            bom_ref: None,
            supplier: None,
            author: None,
            publisher: None,
            group: None,
            name: name.into(),
            version: None,
            description: None,
            scope: None,
            hashes: Vec::new(),
            licenses: Vec::new(),
            copyright: None,
            purl: None,
            external_references: Vec::new(),
            properties: Vec::new(),
            components: Vec::new(),
            evidence: None,
            release_notes: None,
        }
    }

    /// Creates a new application component.
    #[must_use]
    pub fn application(name: impl Into<String>) -> Self {
        Self {
            component_type: ComponentType::Application,
            ..Self::library(name)
        }
    }

    /// Sets the version.
    #[must_use]
    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.version = Some(version.into());
        self
    }

    /// Sets the bom-ref.
    #[must_use]
    pub fn with_bom_ref(mut self, bom_ref: impl Into<String>) -> Self {
        self.bom_ref = Some(bom_ref.into());
        self
    }

    /// Sets the purl.
    #[must_use]
    pub fn with_purl(mut self, purl: impl Into<String>) -> Self {
        self.purl = Some(purl.into());
        self
    }

    /// Sets the description.
    #[must_use]
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Sets the scope.
    #[must_use]
    pub const fn with_scope(mut self, scope: ComponentScope) -> Self {
        self.scope = Some(scope);
        self
    }

    /// Sets the group.
    #[must_use]
    pub fn with_group(mut self, group: impl Into<String>) -> Self {
        self.group = Some(group.into());
        self
    }

    /// Adds a hash.
    pub fn add_hash(&mut self, hash: Hash) {
        self.hashes.push(hash);
    }

    /// Adds a license.
    pub fn add_license(&mut self, license: LicenseChoice) {
        self.licenses.push(license);
    }

    /// Adds an external reference.
    pub fn add_external_reference(&mut self, ext_ref: ExternalReference) {
        self.external_references.push(ext_ref);
    }

    /// Adds a property.
    pub fn add_property(&mut self, property: Property) {
        self.properties.push(property);
    }
}

/// Component types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ComponentType {
    Application,
    Framework,
    Library,
    Container,
    Platform,
    OperatingSystem,
    Device,
    DeviceDriver,
    Firmware,
    File,
    MachineLearningModel,
    Data,
}

/// Component scope.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ComponentScope {
    /// Required for runtime.
    Required,
    /// Optional component.
    Optional,
    /// Excluded component.
    Excluded,
}

/// A cryptographic hash.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hash {
    /// Hash algorithm.
    #[serde(rename = "alg")]
    pub algorithm: String,
    /// Hash content (hex-encoded).
    pub content: String,
}

impl Hash {
    /// Creates a new hash.
    #[must_use]
    pub fn new(algorithm: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            algorithm: algorithm.into(),
            content: content.into(),
        }
    }

    /// Creates a SHA-256 hash.
    #[must_use]
    pub fn sha256(content: impl Into<String>) -> Self {
        Self::new("SHA-256", content)
    }

    /// Creates a SHA-512 hash.
    #[must_use]
    pub fn sha512(content: impl Into<String>) -> Self {
        Self::new("SHA-512", content)
    }
}

/// License information.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum LicenseChoice {
    /// SPDX expression.
    Expression { expression: String },
    /// Named license.
    License { license: License },
}

impl LicenseChoice {
    /// Creates from an SPDX expression.
    #[must_use]
    pub fn expression(expr: impl Into<String>) -> Self {
        Self::Expression {
            expression: expr.into(),
        }
    }

    /// Creates from a license ID.
    #[must_use]
    pub fn license_id(id: impl Into<String>) -> Self {
        Self::License {
            license: License {
                id: Some(id.into()),
                name: None,
                url: None,
                text: None,
            },
        }
    }

    /// Creates from a license name.
    #[must_use]
    pub fn license_name(name: impl Into<String>) -> Self {
        Self::License {
            license: License {
                id: None,
                name: Some(name.into()),
                url: None,
                text: None,
            },
        }
    }
}

/// A license.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct License {
    /// SPDX license ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// License name (if not SPDX).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// License URL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    /// License text.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<AttachedText>,
}

/// Attached text content.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttachedText {
    /// Content type.
    pub content_type: Option<String>,
    /// Encoding (e.g., base64).
    pub encoding: Option<String>,
    /// The text content.
    pub content: String,
}

/// An external reference.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalReference {
    /// Reference type.
    #[serde(rename = "type")]
    pub reference_type: String,
    /// Reference URL.
    pub url: String,
    /// Comment.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    /// Hashes of the referenced resource.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub hashes: Vec<Hash>,
}

impl ExternalReference {
    /// Creates a new external reference.
    #[must_use]
    pub fn new(reference_type: impl Into<String>, url: impl Into<String>) -> Self {
        Self {
            reference_type: reference_type.into(),
            url: url.into(),
            comment: None,
            hashes: Vec::new(),
        }
    }

    /// Creates a VCS reference.
    #[must_use]
    pub fn vcs(url: impl Into<String>) -> Self {
        Self::new("vcs", url)
    }

    /// Creates a website reference.
    #[must_use]
    pub fn website(url: impl Into<String>) -> Self {
        Self::new("website", url)
    }

    /// Creates an issue tracker reference.
    #[must_use]
    pub fn issue_tracker(url: impl Into<String>) -> Self {
        Self::new("issue-tracker", url)
    }

    /// Sets the comment.
    #[must_use]
    pub fn with_comment(mut self, comment: impl Into<String>) -> Self {
        self.comment = Some(comment.into());
        self
    }
}

/// A custom property.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Property {
    /// Property name.
    pub name: String,
    /// Property value.
    pub value: String,
}

impl Property {
    /// Creates a new property.
    #[must_use]
    pub fn new(name: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            value: value.into(),
        }
    }
}

/// An organizational entity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganizationalEntity {
    /// Organization name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Organization URLs.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub url: Vec<String>,
    /// Contacts.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub contact: Vec<OrganizationalContact>,
}

impl OrganizationalEntity {
    /// Creates a new entity with a name.
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: Some(name.into()),
            url: Vec::new(),
            contact: Vec::new(),
        }
    }

    /// Adds a URL.
    #[must_use]
    pub fn with_url(mut self, url: impl Into<String>) -> Self {
        self.url.push(url.into());
        self
    }
}

/// A contact person.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganizationalContact {
    /// Contact name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Contact email.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    /// Contact phone.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phone: Option<String>,
}

/// A dependency relationship.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Dependency {
    /// The bom-ref of the component.
    #[serde(rename = "ref")]
    pub reference: String,
    /// Dependencies of this component.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub depends_on: Vec<String>,
}

impl Dependency {
    /// Creates a new dependency entry.
    #[must_use]
    pub fn new(reference: impl Into<String>) -> Self {
        Self {
            reference: reference.into(),
            depends_on: Vec::new(),
        }
    }

    /// Adds a dependency.
    pub fn add_dependency(&mut self, dep_ref: impl Into<String>) {
        self.depends_on.push(dep_ref.into());
    }
}

/// Vulnerability information.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Vulnerability {
    /// Unique reference within the BOM.
    #[serde(rename = "bom-ref")]
    pub bom_ref: Option<String>,
    /// Vulnerability ID.
    pub id: Option<String>,
    /// Source of the vulnerability info.
    pub source: Option<VulnerabilitySource>,
    /// References.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub references: Vec<VulnerabilityReference>,
    /// Ratings.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub ratings: Vec<VulnerabilityRating>,
    /// Description.
    pub description: Option<String>,
    /// Affected components.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub affects: Vec<VulnerabilityAffects>,
}

/// Vulnerability source.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VulnerabilitySource {
    /// Source name.
    pub name: Option<String>,
    /// Source URL.
    pub url: Option<String>,
}

/// Vulnerability reference.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VulnerabilityReference {
    /// Reference ID.
    pub id: String,
    /// Source.
    pub source: VulnerabilitySource,
}

/// Vulnerability rating.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VulnerabilityRating {
    /// Source of the rating.
    pub source: Option<VulnerabilitySource>,
    /// Score.
    pub score: Option<f64>,
    /// Severity.
    pub severity: Option<String>,
    /// Scoring method.
    pub method: Option<String>,
    /// CVSS vector.
    pub vector: Option<String>,
}

/// Affected component.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VulnerabilityAffects {
    /// Reference to affected component.
    #[serde(rename = "ref")]
    pub reference: String,
    /// Affected versions.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub versions: Vec<AffectedVersion>,
}

/// Affected version.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AffectedVersion {
    /// Version or range.
    pub version: Option<String>,
    /// Version range.
    pub range: Option<String>,
    /// Status.
    pub status: Option<String>,
}

/// Component evidence.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentEvidence {
    /// Licenses.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub licenses: Vec<LicenseChoice>,
    /// Copyright statements.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub copyright: Vec<Copyright>,
}

/// Copyright statement.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Copyright {
    /// Copyright text.
    pub text: String,
}

/// Release notes.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReleaseNotes {
    /// Release type.
    #[serde(rename = "type")]
    pub release_type: Option<String>,
    /// Title.
    pub title: Option<String>,
    /// Description.
    pub description: Option<String>,
    /// Timestamp.
    pub timestamp: Option<String>,
}

/// Composition assertion.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Composition {
    /// The aggregate completeness.
    pub aggregate: String,
    /// Assemblies this applies to.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub assemblies: Vec<String>,
    /// Dependencies this applies to.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub dependencies: Vec<String>,
}

impl Composition {
    /// Creates a "complete" composition.
    #[must_use]
    pub fn complete() -> Self {
        Self {
            aggregate: "complete".to_string(),
            assemblies: Vec::new(),
            dependencies: Vec::new(),
        }
    }

    /// Creates an "incomplete" composition.
    #[must_use]
    pub fn incomplete() -> Self {
        Self {
            aggregate: "incomplete".to_string(),
            assemblies: Vec::new(),
            dependencies: Vec::new(),
        }
    }

    /// Creates an "unknown" composition.
    #[must_use]
    pub fn unknown() -> Self {
        Self {
            aggregate: "unknown".to_string(),
            assemblies: Vec::new(),
            dependencies: Vec::new(),
        }
    }
}
