//! # SCTV Detectors
//!
//! Threat detection engines for the Supply Chain Trust Verifier.
//! Includes detection for typosquatting, tampering, downgrade attacks, and provenance.
//!
//! # Available Detectors
//!
//! - [`typosquatting::TyposquattingDetector`] - Detects typosquatting attacks
//! - [`tampering::TamperingDetector`] - Detects package tampering
//! - [`downgrade::DowngradeDetector`] - Detects version downgrade attacks
//! - [`provenance::ProvenanceDetector`] - Verifies SLSA provenance attestations
//!
//! # Usage
//!
//! ```rust,ignore
//! use sctv_detectors::{Detector, DetectionResult};
//! use sctv_detectors::provenance::ProvenanceDetector;
//! use sctv_detectors::downgrade::DowngradeDetector;
//!
//! // Create detectors
//! let provenance_detector = ProvenanceDetector::new();
//! let downgrade_detector = DowngradeDetector::new();
//!
//! // Analyze dependencies
//! let results = provenance_detector.analyze(&dependency).await?;
//! ```

mod traits;

pub mod downgrade;
pub mod provenance;
pub mod tampering;
pub mod typosquatting;

pub use traits::*;

// Re-export main detector types for convenience
pub use downgrade::{DowngradeConfig, DowngradeDetector, DowngradeSeverity};
pub use provenance::{ProvenanceConfig, ProvenanceDetector, ProvenanceVerificationResult};
