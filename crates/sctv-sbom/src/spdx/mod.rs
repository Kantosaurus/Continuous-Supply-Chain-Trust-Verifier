//! SPDX SBOM format support.
//!
//! This module provides SPDX 2.3 SBOM generation conforming to the
//! Linux Foundation SPDX specification.
//!
//! # Specification
//!
//! SPDX (Software Package Data Exchange) is an open standard for communicating
//! software bill of material information, including components, licenses,
//! copyrights, and security references.
//!
//! - Specification: <https://spdx.github.io/spdx-spec/v2.3/>
//! - JSON Schema: <https://github.com/spdx/spdx-spec/blob/development/v2.3/schemas/spdx-schema.json>
//!
//! # Features
//!
//! - Complete document structure with packages, files, and snippets
//! - Package URL (purl) support as external references
//! - Checksum embedding (SHA-256, SHA-512, SHA-1, MD5)
//! - License expression support (SPDX identifiers)
//! - Relationship tracking (`DEPENDS_ON`, `DEPENDENCY_OF`, etc.)
//! - Both JSON and tag-value output formats
//!
//! # Usage
//!
//! ```ignore
//! use sctv_sbom::spdx::SpdxGenerator;
//! use sctv_sbom::{GeneratorConfig, SbomGenerator};
//!
//! let generator = SpdxGenerator::json();
//! let config = GeneratorConfig::default();
//! let output = generator.generate(&project, &dependencies, &config)?;
//! ```

mod generator;
pub mod models;

pub use generator::SpdxGenerator;
