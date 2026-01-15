//! Detector trait definitions.

use async_trait::async_trait;
use sctv_core::{Alert, Dependency};
use thiserror::Error;

/// Errors that can occur during detection.
#[derive(Debug, Error)]
pub enum DetectorError {
    #[error("Analysis failed: {0}")]
    AnalysisFailed(String),

    #[error("Configuration error: {0}")]
    Configuration(String),

    #[error("Data unavailable: {0}")]
    DataUnavailable(String),
}

/// Result type for detector operations.
pub type DetectorResult<T> = Result<T, DetectorError>;

/// Result of a detection analysis.
#[derive(Debug, Clone)]
pub struct DetectionResult {
    /// Whether a threat was detected.
    pub detected: bool,
    /// Confidence score (0.0 to 1.0).
    pub confidence: f64,
    /// Detection method used.
    pub method: String,
    /// Additional details.
    pub details: serde_json::Value,
}

impl DetectionResult {
    /// Creates a positive detection result.
    #[must_use]
    pub fn detected(confidence: f64, method: &str, details: serde_json::Value) -> Self {
        Self {
            detected: true,
            confidence,
            method: method.to_string(),
            details,
        }
    }

    /// Creates a negative detection result.
    #[must_use]
    pub fn not_detected() -> Self {
        Self {
            detected: false,
            confidence: 0.0,
            method: String::new(),
            details: serde_json::Value::Null,
        }
    }
}

/// Trait for threat detection engines.
#[async_trait]
pub trait Detector: Send + Sync {
    /// Returns the detector type name.
    fn detector_type(&self) -> &'static str;

    /// Analyzes a dependency for threats.
    async fn analyze(&self, dependency: &Dependency) -> DetectorResult<Vec<DetectionResult>>;

    /// Creates alerts from detection results.
    fn create_alerts(
        &self,
        dependency: &Dependency,
        results: &[DetectionResult],
    ) -> Vec<Alert>;
}
