//! `CycloneDX` SBOM format support.
//!
//! This module provides `CycloneDX` 1.5 SBOM generation conforming to the
//! OWASP `CycloneDX` specification.
//!
//! # Specification
//!
//! `CycloneDX` is a lightweight SBOM standard designed for use in application
//! security contexts and supply chain component analysis.
//!
//! - Specification: <https://cyclonedx.org/docs/1.5/>
//! - JSON Schema: <https://cyclonedx.org/docs/1.5/json/>
//! - XML Schema: <https://cyclonedx.org/docs/1.5/xml/>
//!
//! # Features
//!
//! - Complete BOM structure with metadata, components, and dependencies
//! - Package URL (purl) support for all ecosystems
//! - Hash/checksum embedding (SHA-256, SHA-512)
//! - External references (VCS, issue trackers, etc.)
//! - Dependency relationship graph
//! - Composition assertions
//! - Both JSON and XML output formats
//!
//! # Usage
//!
//! ```ignore
//! use sctv_sbom::cyclonedx::CycloneDxGenerator;
//! use sctv_sbom::{GeneratorConfig, SbomGenerator};
//!
//! let generator = CycloneDxGenerator::json();
//! let config = GeneratorConfig::default();
//! let output = generator.generate(&project, &dependencies, &config)?;
//! ```

mod generator;
pub mod models;

pub use generator::CycloneDxGenerator;
