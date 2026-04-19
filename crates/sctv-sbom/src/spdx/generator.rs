//! SPDX 2.3 SBOM generator.

use sctv_core::{Dependency, PackageEcosystem, Project};
use uuid::Uuid;

use super::models::*;
use crate::common::{generate_bom_ref, GeneratorConfig};
use crate::{SbomError, SbomFormat, SbomGenerator, SbomOutput, SbomResult};

/// SPDX 2.3 SBOM generator.
pub struct SpdxGenerator {
    /// Whether to output tag-value format instead of JSON.
    tag_value_output: bool,
}

impl SpdxGenerator {
    /// Creates a new SPDX generator.
    #[must_use]
    pub fn new(tag_value_output: bool) -> Self {
        Self { tag_value_output }
    }

    /// Creates a JSON generator.
    #[must_use]
    pub fn json() -> Self {
        Self::new(false)
    }

    /// Creates a tag-value generator.
    #[must_use]
    pub fn tag_value() -> Self {
        Self::new(true)
    }

    /// Builds the SPDX document.
    fn build_document(
        &self,
        project: &Project,
        dependencies: &[Dependency],
        config: &GeneratorConfig,
    ) -> SbomResult<Document> {
        let doc_uuid = Uuid::new_v4();
        let namespace = format!(
            "https://spdx.org/spdxdocs/{}-{}",
            sanitize_spdx_id(&project.name),
            doc_uuid
        );

        let mut doc = Document::new(format!("{} SBOM", project.name), namespace.clone());

        // Build creation info
        let creation_info = self.build_creation_info(config)?;
        doc.creation_info = creation_info;

        // Add main project package
        let project_pkg = self.build_project_package(project)?;
        let project_spdx_id = project_pkg.spdx_id.clone();
        doc.add_package(project_pkg);

        // Document DESCRIBES the main project
        doc.add_relationship(Relationship::describes(
            "SPDXRef-DOCUMENT",
            &project_spdx_id,
        ));

        // Filter dependencies based on config
        let filtered_deps: Vec<&Dependency> = dependencies
            .iter()
            .filter(|dep| {
                if !config.include_dev_dependencies && dep.is_dev_dependency {
                    return false;
                }
                if !config.include_transitive && !dep.is_direct {
                    return false;
                }
                true
            })
            .collect();

        // Build packages for each dependency
        for dep in &filtered_deps {
            let package = self.build_dependency_package(dep, config)?;
            let pkg_spdx_id = package.spdx_id.clone();
            doc.add_package(package);

            // Add relationship to project for direct dependencies
            if dep.is_direct {
                if dep.is_dev_dependency {
                    doc.add_relationship(Relationship::dev_dependency_of(
                        &pkg_spdx_id,
                        &project_spdx_id,
                    ));
                } else {
                    doc.add_relationship(Relationship::dependency_of(
                        &pkg_spdx_id,
                        &project_spdx_id,
                    ));
                }
            }
        }

        // Build transitive dependency relationships
        for dep in &filtered_deps {
            if let Some(parent_id) = dep.parent_id {
                // Find parent in filtered deps
                if let Some(parent) = filtered_deps.iter().find(|d| d.id == parent_id) {
                    let dep_spdx_id = self.generate_spdx_id(dep);
                    let parent_spdx_id = self.generate_spdx_id(parent);

                    doc.add_relationship(Relationship::dependency_of(
                        &dep_spdx_id,
                        &parent_spdx_id,
                    ));
                }
            }
        }

        Ok(doc)
    }

    /// Builds the creation info section.
    fn build_creation_info(&self, config: &GeneratorConfig) -> SbomResult<CreationInfo> {
        let mut info = CreationInfo::new();

        // Add tool
        info.add_tool(&config.tool_name, &config.tool_version);

        // Add organization if configured
        if let Some(supplier) = &config.supplier_name {
            info.add_organization(supplier);
        }

        Ok(info)
    }

    /// Builds a package for the main project.
    fn build_project_package(&self, project: &Project) -> SbomResult<Package> {
        let spdx_id = format!("SPDXRef-Package-{}", sanitize_spdx_id(&project.name));

        let mut pkg =
            Package::new(&spdx_id, &project.name).with_purpose(PackagePurpose::Application);

        if let Some(desc) = &project.description {
            pkg.description = Some(desc.clone());
        }

        // Add repository URL as download location
        if let Some(repo_url) = &project.repository_url {
            pkg.download_location = repo_url.to_string();
            pkg.homepage = Some(repo_url.to_string());
        }

        // Add supplier
        pkg.supplier = Some(format!("Organization: {}", project.name));

        Ok(pkg)
    }

    /// Builds a package from a dependency.
    fn build_dependency_package(
        &self,
        dep: &Dependency,
        config: &GeneratorConfig,
    ) -> SbomResult<Package> {
        let spdx_id = self.generate_spdx_id(dep);

        let mut pkg = Package::new(&spdx_id, &dep.package_name)
            .with_version(dep.resolved_version.to_string())
            .with_purpose(PackagePurpose::Library);

        // Add PURL as external reference
        pkg.add_external_ref(ExternalRef::purl(dep.purl()));

        // Set download location based on ecosystem
        pkg.download_location = self.get_download_location(dep);

        // Add checksums if configured
        if config.include_hashes {
            if let Some(sha256) = &dep.integrity.hash_sha256 {
                pkg.add_checksum(Checksum::sha256(sha256));
            }
            if let Some(sha512) = &dep.integrity.hash_sha512 {
                pkg.add_checksum(Checksum::sha512(sha512));
            }
        }

        // Add provenance information as external reference
        if let Some(provenance) = &dep.integrity.provenance_details {
            if let Some(source_uri) = &provenance.source_uri {
                pkg.add_external_ref(
                    ExternalRef::new("OTHER", "vcs", source_uri).with_comment("Source repository"),
                );
            }
        }

        // Set supplier based on ecosystem
        pkg.supplier = Some(format!(
            "Organization: {} Registry",
            dep.ecosystem.purl_type()
        ));

        Ok(pkg)
    }

    /// Generates an SPDX ID for a dependency.
    fn generate_spdx_id(&self, dep: &Dependency) -> String {
        let bom_ref = generate_bom_ref(
            dep.ecosystem.purl_type(),
            &dep.package_name,
            &dep.resolved_version.to_string(),
        );
        format!("SPDXRef-Package-{}", sanitize_spdx_id(&bom_ref))
    }

    /// Gets the download location for a dependency.
    fn get_download_location(&self, dep: &Dependency) -> String {
        match dep.ecosystem {
            PackageEcosystem::Npm => {
                format!(
                    "https://registry.npmjs.org/{}/-/{}-{}.tgz",
                    dep.package_name,
                    dep.package_name
                        .rsplit('/')
                        .next()
                        .unwrap_or(&dep.package_name),
                    dep.resolved_version
                )
            }
            PackageEcosystem::PyPi => {
                format!(
                    "https://pypi.org/project/{}/{}/#files",
                    dep.package_name, dep.resolved_version
                )
            }
            PackageEcosystem::Maven => {
                if let Some((group, artifact)) = dep.package_name.rsplit_once(':') {
                    let group_path = group.replace('.', "/");
                    format!(
                        "https://repo1.maven.org/maven2/{}/{}/{}",
                        group_path, artifact, dep.resolved_version
                    )
                } else {
                    "NOASSERTION".to_string()
                }
            }
            PackageEcosystem::Cargo => {
                format!(
                    "https://crates.io/api/v1/crates/{}/{}/download",
                    dep.package_name, dep.resolved_version
                )
            }
            PackageEcosystem::NuGet => {
                format!(
                    "https://www.nuget.org/api/v2/package/{}/{}",
                    dep.package_name, dep.resolved_version
                )
            }
            PackageEcosystem::RubyGems => {
                format!(
                    "https://rubygems.org/downloads/{}-{}.gem",
                    dep.package_name, dep.resolved_version
                )
            }
            PackageEcosystem::GoModules => {
                format!(
                    "https://proxy.golang.org/{}/@v/v{}.zip",
                    dep.package_name, dep.resolved_version
                )
            }
        }
    }

    /// Serializes the document to JSON.
    fn serialize_json(&self, doc: &Document, pretty: bool) -> SbomResult<String> {
        if pretty {
            serde_json::to_string_pretty(doc)
        } else {
            serde_json::to_string(doc)
        }
        .map_err(|e| SbomError::Serialization(e.to_string()))
    }

    /// Serializes the document to tag-value format.
    fn serialize_tag_value(&self, doc: &Document) -> SbomResult<String> {
        let mut output = String::new();

        // Document information
        output.push_str(&format!("SPDXVersion: {}\n", doc.spdx_version));
        output.push_str(&format!("DataLicense: {}\n", doc.data_license));
        output.push_str(&format!("SPDXID: {}\n", doc.spdx_id));
        output.push_str(&format!("DocumentName: {}\n", doc.name));
        output.push_str(&format!("DocumentNamespace: {}\n", doc.document_namespace));

        // Creation info
        output.push_str(&format!(
            "Creator: Tool: {}\n",
            doc.creation_info
                .creators
                .iter()
                .find(|c| c.starts_with("Tool:"))
                .map(|s| s.trim_start_matches("Tool: "))
                .unwrap_or("unknown")
        ));
        for creator in &doc.creation_info.creators {
            if !creator.starts_with("Tool:") {
                output.push_str(&format!("Creator: {}\n", creator));
            }
        }
        output.push_str(&format!("Created: {}\n", doc.creation_info.created));
        if let Some(license_version) = &doc.creation_info.license_list_version {
            output.push_str(&format!("LicenseListVersion: {}\n", license_version));
        }

        output.push('\n');

        // Packages
        for pkg in &doc.packages {
            output.push_str("##### Package\n\n");
            output.push_str(&format!("PackageName: {}\n", pkg.name));
            output.push_str(&format!("SPDXID: {}\n", pkg.spdx_id));

            if let Some(version) = &pkg.version_info {
                output.push_str(&format!("PackageVersion: {}\n", version));
            }

            if let Some(supplier) = &pkg.supplier {
                output.push_str(&format!("PackageSupplier: {}\n", supplier));
            }

            output.push_str(&format!(
                "PackageDownloadLocation: {}\n",
                pkg.download_location
            ));

            if let Some(files_analyzed) = pkg.files_analyzed {
                output.push_str(&format!("FilesAnalyzed: {}\n", files_analyzed));
            }

            for checksum in &pkg.checksums {
                output.push_str(&format!(
                    "PackageChecksum: {}: {}\n",
                    checksum.algorithm, checksum.checksum_value
                ));
            }

            if let Some(homepage) = &pkg.homepage {
                output.push_str(&format!("PackageHomePage: {}\n", homepage));
            }

            if let Some(license) = &pkg.license_concluded {
                output.push_str(&format!("PackageLicenseConcluded: {}\n", license));
            }

            if let Some(license) = &pkg.license_declared {
                output.push_str(&format!("PackageLicenseDeclared: {}\n", license));
            }

            if let Some(copyright) = &pkg.copyright_text {
                output.push_str(&format!("PackageCopyrightText: {}\n", copyright));
            }

            if let Some(purpose) = &pkg.primary_package_purpose {
                output.push_str(&format!("PrimaryPackagePurpose: {}\n", purpose));
            }

            for ext_ref in &pkg.external_refs {
                output.push_str(&format!(
                    "ExternalRef: {} {} {}\n",
                    ext_ref.reference_category, ext_ref.reference_type, ext_ref.reference_locator
                ));
            }

            if let Some(desc) = &pkg.description {
                output.push_str(&format!("PackageDescription: <text>{}</text>\n", desc));
            }

            output.push('\n');
        }

        // Relationships
        for rel in &doc.relationships {
            output.push_str(&format!(
                "Relationship: {} {} {}\n",
                rel.spdx_element_id, rel.relationship_type, rel.related_spdx_element
            ));
        }

        Ok(output)
    }
}

impl SbomGenerator for SpdxGenerator {
    fn format(&self) -> SbomFormat {
        if self.tag_value_output {
            SbomFormat::SpdxTagValue
        } else {
            SbomFormat::Spdx
        }
    }

    fn generate(
        &self,
        project: &Project,
        dependencies: &[Dependency],
        config: &GeneratorConfig,
    ) -> SbomResult<SbomOutput> {
        let doc = self.build_document(project, dependencies, config)?;

        let content = if self.tag_value_output {
            self.serialize_tag_value(&doc)?
        } else {
            self.serialize_json(&doc, config.pretty_print)?
        };

        Ok(SbomOutput {
            format: self.format(),
            content,
            generated_at: chrono::Utc::now(),
            serial_number: doc.document_namespace.clone(),
            component_count: doc.packages.len(),
        })
    }
}

/// Sanitizes a string for use in an SPDX ID.
/// SPDX IDs can only contain letters, numbers, and hyphens.
fn sanitize_spdx_id(s: &str) -> String {
    s.chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' {
                c
            } else {
                '-'
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use sctv_core::TenantId;

    fn create_test_project() -> Project {
        Project::new(TenantId::new(), "test-project".to_string())
    }

    fn create_test_dependency() -> Dependency {
        use sctv_core::ProjectId;
        use semver::Version;

        Dependency::new(
            ProjectId::new(),
            TenantId::new(),
            "lodash".to_string(),
            PackageEcosystem::Npm,
            "^4.17.0".to_string(),
            Version::new(4, 17, 21),
        )
    }

    #[test]
    fn test_generate_spdx_json() {
        let generator = SpdxGenerator::json();
        let project = create_test_project();
        let deps = vec![create_test_dependency()];
        let config = GeneratorConfig::default();

        let result = generator.generate(&project, &deps, &config);
        assert!(result.is_ok());

        let output = result.unwrap();
        assert_eq!(output.format, SbomFormat::Spdx);
        assert!(output.content.contains("SPDX-2.3"));
        assert!(output.content.contains("lodash"));
        assert!(output.content.contains("SPDXRef-DOCUMENT"));
    }

    #[test]
    fn test_generate_spdx_tag_value() {
        let generator = SpdxGenerator::tag_value();
        let project = create_test_project();
        let deps = vec![create_test_dependency()];
        let config = GeneratorConfig::default();

        let result = generator.generate(&project, &deps, &config);
        assert!(result.is_ok());

        let output = result.unwrap();
        assert_eq!(output.format, SbomFormat::SpdxTagValue);
        assert!(output.content.contains("SPDXVersion: SPDX-2.3"));
        assert!(output.content.contains("DataLicense: CC0-1.0"));
        assert!(output.content.contains("PackageName: lodash"));
    }

    #[test]
    fn test_sanitize_spdx_id() {
        assert_eq!(sanitize_spdx_id("simple"), "simple");
        assert_eq!(sanitize_spdx_id("has spaces"), "has-spaces");
        assert_eq!(sanitize_spdx_id("has@special#chars"), "has-special-chars");
        assert_eq!(sanitize_spdx_id("kebab-case"), "kebab-case");
    }

    #[test]
    fn test_download_locations() {
        let generator = SpdxGenerator::json();

        let mut dep = create_test_dependency();
        assert!(generator.get_download_location(&dep).contains("npmjs.org"));

        dep.ecosystem = PackageEcosystem::PyPi;
        assert!(generator.get_download_location(&dep).contains("pypi.org"));

        dep.ecosystem = PackageEcosystem::Cargo;
        assert!(generator.get_download_location(&dep).contains("crates.io"));

        dep.ecosystem = PackageEcosystem::RubyGems;
        assert!(generator
            .get_download_location(&dep)
            .contains("rubygems.org"));

        dep.ecosystem = PackageEcosystem::GoModules;
        assert!(generator
            .get_download_location(&dep)
            .contains("proxy.golang.org"));
    }
}
