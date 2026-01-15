//! CycloneDX SBOM generator.

use sctv_core::{Dependency, PackageEcosystem, Project};
use uuid::Uuid;

use super::models::*;
use crate::common::{generate_bom_ref, GeneratorConfig};
use crate::{SbomError, SbomFormat, SbomGenerator, SbomOutput, SbomResult};

/// CycloneDX 1.5 SBOM generator.
pub struct CycloneDxGenerator {
    /// Whether to output XML instead of JSON.
    xml_output: bool,
}

impl CycloneDxGenerator {
    /// Creates a new CycloneDX generator.
    #[must_use]
    pub fn new(xml_output: bool) -> Self {
        Self { xml_output }
    }

    /// Creates a JSON generator.
    #[must_use]
    pub fn json() -> Self {
        Self::new(false)
    }

    /// Creates an XML generator.
    #[must_use]
    pub fn xml() -> Self {
        Self::new(true)
    }

    /// Builds the BOM document.
    fn build_bom(
        &self,
        project: &Project,
        dependencies: &[Dependency],
        config: &GeneratorConfig,
    ) -> SbomResult<Bom> {
        let serial = format!("urn:uuid:{}", Uuid::new_v4());

        let mut bom = Bom::new().with_serial_number(serial);

        // Build metadata
        let metadata = self.build_metadata(project, config)?;
        bom.metadata = Some(metadata);

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

        // Build components
        for dep in &filtered_deps {
            let component = self.build_component(dep, config)?;
            bom.add_component(component);
        }

        // Build dependency relationships
        let dep_graph = self.build_dependency_graph(&filtered_deps, project)?;
        bom.dependencies = dep_graph;

        // Add composition assertion
        bom.compositions.push(Composition::incomplete());

        Ok(bom)
    }

    /// Builds the metadata section.
    fn build_metadata(
        &self,
        project: &Project,
        config: &GeneratorConfig,
    ) -> SbomResult<Metadata> {
        let mut metadata = Metadata::new();

        // Add tool information
        let tool = Tool::new(&config.tool_name, &config.tool_version)
            .with_vendor(&config.tool_vendor);
        metadata.add_tool(tool);

        // Add main component (the project itself)
        let mut main_component = Component::application(&project.name);
        main_component.bom_ref = Some(format!("project-{}", project.id));

        if let Some(desc) = &project.description {
            main_component.description = Some(desc.clone());
        }

        // Add repository URL as external reference
        if let Some(repo_url) = &project.repository_url {
            main_component.add_external_reference(ExternalReference::vcs(repo_url.to_string()));
        }

        // Add ecosystems as properties
        for ecosystem in &project.ecosystems {
            main_component.add_property(Property::new(
                "sctv:ecosystem",
                ecosystem.purl_type(),
            ));
        }

        metadata.component = Some(main_component);

        // Add supplier if configured
        if let Some(supplier_name) = &config.supplier_name {
            let mut supplier = OrganizationalEntity::new(supplier_name);
            if let Some(supplier_url) = &config.supplier_url {
                supplier = supplier.with_url(supplier_url);
            }
            metadata.supplier = Some(supplier);
        }

        Ok(metadata)
    }

    /// Builds a component from a dependency.
    fn build_component(
        &self,
        dep: &Dependency,
        config: &GeneratorConfig,
    ) -> SbomResult<Component> {
        let bom_ref = generate_bom_ref(
            dep.ecosystem.purl_type(),
            &dep.package_name,
            &dep.resolved_version.to_string(),
        );

        let mut component = Component::library(&dep.package_name)
            .with_version(dep.resolved_version.to_string())
            .with_bom_ref(&bom_ref)
            .with_purl(dep.purl());

        // Set scope
        if dep.is_dev_dependency {
            component.scope = Some(ComponentScope::Optional);
        } else {
            component.scope = Some(ComponentScope::Required);
        }

        // Set group for Maven packages
        if dep.ecosystem == PackageEcosystem::Maven {
            if let Some((group, _name)) = dep.package_name.rsplit_once(':') {
                component.group = Some(group.to_string());
            }
        }

        // Add hashes if configured
        if config.include_hashes {
            if let Some(sha256) = &dep.integrity.hash_sha256 {
                component.add_hash(Hash::sha256(sha256));
            }
            if let Some(sha512) = &dep.integrity.hash_sha512 {
                component.add_hash(Hash::sha512(sha512));
            }
        }

        // Add properties
        component.add_property(Property::new(
            "sctv:ecosystem",
            dep.ecosystem.purl_type(),
        ));

        component.add_property(Property::new(
            "sctv:direct",
            if dep.is_direct { "true" } else { "false" },
        ));

        component.add_property(Property::new(
            "sctv:depth",
            dep.depth.to_string(),
        ));

        // Add provenance information if available
        if let Some(level) = dep.integrity.provenance_status.level() {
            component.add_property(Property::new("sctv:slsa-level", level.to_string()));
        }

        if let Some(provenance) = &dep.integrity.provenance_details {
            if let Some(builder_id) = &provenance.builder_id {
                component.add_property(Property::new("sctv:builder-id", builder_id));
            }
            if let Some(source_uri) = &provenance.source_uri {
                component.add_external_reference(
                    ExternalReference::vcs(source_uri).with_comment("Source repository"),
                );
            }
        }

        Ok(component)
    }

    /// Builds the dependency graph.
    fn build_dependency_graph(
        &self,
        deps: &[&Dependency],
        project: &Project,
    ) -> SbomResult<Vec<super::models::Dependency>> {
        let mut graph = Vec::new();

        // Project depends on all direct dependencies
        let project_ref = format!("project-{}", project.id);
        let mut project_dep = super::models::Dependency::new(&project_ref);

        for dep in deps {
            if dep.is_direct {
                let bom_ref = generate_bom_ref(
                    dep.ecosystem.purl_type(),
                    &dep.package_name,
                    &dep.resolved_version.to_string(),
                );
                project_dep.add_dependency(bom_ref);
            }
        }

        graph.push(project_dep);

        // Build dependency relationships between components
        for dep in deps {
            let bom_ref = generate_bom_ref(
                dep.ecosystem.purl_type(),
                &dep.package_name,
                &dep.resolved_version.to_string(),
            );

            let mut dep_entry = super::models::Dependency::new(&bom_ref);

            // Find children (components that have this dep as parent)
            for child in deps {
                if let Some(parent_id) = child.parent_id {
                    if parent_id == dep.id {
                        let child_ref = generate_bom_ref(
                            child.ecosystem.purl_type(),
                            &child.package_name,
                            &child.resolved_version.to_string(),
                        );
                        dep_entry.add_dependency(child_ref);
                    }
                }
            }

            graph.push(dep_entry);
        }

        Ok(graph)
    }

    /// Serializes the BOM to JSON.
    fn serialize_json(&self, bom: &Bom, pretty: bool) -> SbomResult<String> {
        if pretty {
            serde_json::to_string_pretty(bom)
        } else {
            serde_json::to_string(bom)
        }
        .map_err(|e| SbomError::Serialization(e.to_string()))
    }

    /// Serializes the BOM to XML.
    fn serialize_xml(&self, bom: &Bom, _pretty: bool) -> SbomResult<String> {
        // Note: For full XML support, we'd need a proper XML serializer.
        // This provides a basic XML structure.
        let mut xml = String::new();
        xml.push_str(r#"<?xml version="1.0" encoding="UTF-8"?>"#);
        xml.push('\n');
        xml.push_str(r#"<bom xmlns="http://cyclonedx.org/schema/bom/1.5" "#);
        xml.push_str(r#"version="1" "#);
        if let Some(serial) = &bom.serial_number {
            xml.push_str(&format!(r#"serialNumber="{}" "#, serial));
        }
        xml.push_str(">\n");

        // Metadata
        if let Some(metadata) = &bom.metadata {
            xml.push_str("  <metadata>\n");
            if let Some(timestamp) = &metadata.timestamp {
                xml.push_str(&format!("    <timestamp>{}</timestamp>\n", timestamp));
            }
            for tool in &metadata.tools {
                xml.push_str("    <tools>\n");
                xml.push_str("      <tool>\n");
                if let Some(vendor) = &tool.vendor {
                    xml.push_str(&format!("        <vendor>{}</vendor>\n", escape_xml(vendor)));
                }
                if let Some(name) = &tool.name {
                    xml.push_str(&format!("        <name>{}</name>\n", escape_xml(name)));
                }
                if let Some(version) = &tool.version {
                    xml.push_str(&format!("        <version>{}</version>\n", escape_xml(version)));
                }
                xml.push_str("      </tool>\n");
                xml.push_str("    </tools>\n");
            }
            if let Some(component) = &metadata.component {
                xml.push_str(&format!(
                    "    <component type=\"{}\">\n",
                    component_type_xml(&component.component_type)
                ));
                xml.push_str(&format!("      <name>{}</name>\n", escape_xml(&component.name)));
                if let Some(version) = &component.version {
                    xml.push_str(&format!("      <version>{}</version>\n", escape_xml(version)));
                }
                xml.push_str("    </component>\n");
            }
            xml.push_str("  </metadata>\n");
        }

        // Components
        xml.push_str("  <components>\n");
        for component in &bom.components {
            xml.push_str(&format!(
                "    <component type=\"{}\"",
                component_type_xml(&component.component_type)
            ));
            if let Some(bom_ref) = &component.bom_ref {
                xml.push_str(&format!(" bom-ref=\"{}\"", escape_xml(bom_ref)));
            }
            xml.push_str(">\n");
            xml.push_str(&format!("      <name>{}</name>\n", escape_xml(&component.name)));
            if let Some(version) = &component.version {
                xml.push_str(&format!("      <version>{}</version>\n", escape_xml(version)));
            }
            if let Some(purl) = &component.purl {
                xml.push_str(&format!("      <purl>{}</purl>\n", escape_xml(purl)));
            }
            if !component.hashes.is_empty() {
                xml.push_str("      <hashes>\n");
                for hash in &component.hashes {
                    xml.push_str(&format!(
                        "        <hash alg=\"{}\">{}</hash>\n",
                        escape_xml(&hash.algorithm),
                        escape_xml(&hash.content)
                    ));
                }
                xml.push_str("      </hashes>\n");
            }
            xml.push_str("    </component>\n");
        }
        xml.push_str("  </components>\n");

        // Dependencies
        if !bom.dependencies.is_empty() {
            xml.push_str("  <dependencies>\n");
            for dep in &bom.dependencies {
                xml.push_str(&format!("    <dependency ref=\"{}\"", escape_xml(&dep.reference)));
                if dep.depends_on.is_empty() {
                    xml.push_str(" />\n");
                } else {
                    xml.push_str(">\n");
                    for child_ref in &dep.depends_on {
                        xml.push_str(&format!(
                            "      <dependency ref=\"{}\" />\n",
                            escape_xml(child_ref)
                        ));
                    }
                    xml.push_str("    </dependency>\n");
                }
            }
            xml.push_str("  </dependencies>\n");
        }

        xml.push_str("</bom>\n");

        Ok(xml)
    }
}

impl SbomGenerator for CycloneDxGenerator {
    fn format(&self) -> SbomFormat {
        if self.xml_output {
            SbomFormat::CycloneDxXml
        } else {
            SbomFormat::CycloneDx
        }
    }

    fn generate(
        &self,
        project: &Project,
        dependencies: &[Dependency],
        config: &GeneratorConfig,
    ) -> SbomResult<SbomOutput> {
        let bom = self.build_bom(project, dependencies, config)?;

        let content = if self.xml_output {
            self.serialize_xml(&bom, config.pretty_print)?
        } else {
            self.serialize_json(&bom, config.pretty_print)?
        };

        Ok(SbomOutput {
            format: self.format(),
            content,
            generated_at: chrono::Utc::now(),
            serial_number: bom.serial_number.unwrap_or_default(),
            component_count: bom.components.len(),
        })
    }
}

/// Escapes XML special characters.
fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

/// Returns the XML representation of a component type.
fn component_type_xml(ct: &ComponentType) -> &'static str {
    match ct {
        ComponentType::Application => "application",
        ComponentType::Framework => "framework",
        ComponentType::Library => "library",
        ComponentType::Container => "container",
        ComponentType::Platform => "platform",
        ComponentType::OperatingSystem => "operating-system",
        ComponentType::Device => "device",
        ComponentType::DeviceDriver => "device-driver",
        ComponentType::Firmware => "firmware",
        ComponentType::File => "file",
        ComponentType::MachineLearningModel => "machine-learning-model",
        ComponentType::Data => "data",
    }
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
    fn test_generate_cyclonedx_json() {
        let generator = CycloneDxGenerator::json();
        let project = create_test_project();
        let deps = vec![create_test_dependency()];
        let config = GeneratorConfig::default();

        let result = generator.generate(&project, &deps, &config);
        assert!(result.is_ok());

        let output = result.unwrap();
        assert_eq!(output.format, SbomFormat::CycloneDx);
        assert_eq!(output.component_count, 1);
        assert!(output.content.contains("CycloneDX"));
        assert!(output.content.contains("lodash"));
    }

    #[test]
    fn test_generate_cyclonedx_xml() {
        let generator = CycloneDxGenerator::xml();
        let project = create_test_project();
        let deps = vec![create_test_dependency()];
        let config = GeneratorConfig::default();

        let result = generator.generate(&project, &deps, &config);
        assert!(result.is_ok());

        let output = result.unwrap();
        assert_eq!(output.format, SbomFormat::CycloneDxXml);
        assert!(output.content.contains("<?xml"));
        assert!(output.content.contains("<bom"));
    }

    #[test]
    fn test_escape_xml() {
        assert_eq!(escape_xml("<test>"), "&lt;test&gt;");
        assert_eq!(escape_xml("a & b"), "a &amp; b");
        assert_eq!(escape_xml("\"quoted\""), "&quot;quoted&quot;");
    }
}
