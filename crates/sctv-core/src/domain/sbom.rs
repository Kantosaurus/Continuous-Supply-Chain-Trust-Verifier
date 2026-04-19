//! Software Bill of Materials (SBOM) domain model.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{ProjectId, TenantId};

/// Unique identifier for an SBOM.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SbomId(pub Uuid);

impl SbomId {
    /// Creates a new random SBOM ID.
    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for SbomId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for SbomId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// SBOM format specification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum SbomFormat {
    /// `CycloneDX` format (recommended).
    #[default]
    CycloneDx,
    /// SPDX format.
    Spdx,
}

impl std::fmt::Display for SbomFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CycloneDx => write!(f, "cyclonedx"),
            Self::Spdx => write!(f, "spdx"),
        }
    }
}

impl std::str::FromStr for SbomFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "cyclonedx" => Ok(Self::CycloneDx),
            "spdx" => Ok(Self::Spdx),
            _ => Err(format!("Unknown SBOM format: {s}")),
        }
    }
}

/// A Software Bill of Materials document.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sbom {
    pub id: SbomId,
    pub project_id: ProjectId,
    pub tenant_id: TenantId,
    pub format: SbomFormat,
    pub format_version: String,
    pub content: serde_json::Value,
    pub generated_at: DateTime<Utc>,
    pub scan_id: Option<Uuid>,
}

impl Sbom {
    /// Creates a new SBOM for a project.
    #[must_use]
    pub fn new(
        project_id: ProjectId,
        tenant_id: TenantId,
        format: SbomFormat,
        format_version: String,
        content: serde_json::Value,
    ) -> Self {
        Self {
            id: SbomId::new(),
            project_id,
            tenant_id,
            format,
            format_version,
            content,
            generated_at: Utc::now(),
            scan_id: None,
        }
    }

    /// Creates an SBOM linked to a specific scan.
    #[must_use]
    pub fn from_scan(
        project_id: ProjectId,
        tenant_id: TenantId,
        format: SbomFormat,
        format_version: String,
        content: serde_json::Value,
        scan_id: Uuid,
    ) -> Self {
        let mut sbom = Self::new(project_id, tenant_id, format, format_version, content);
        sbom.scan_id = Some(scan_id);
        sbom
    }

    /// Returns the default format version for a given format.
    #[must_use]
    pub const fn default_version(format: SbomFormat) -> &'static str {
        match format {
            SbomFormat::CycloneDx => "1.5",
            SbomFormat::Spdx => "2.3",
        }
    }

    /// Creates a new `CycloneDX` SBOM.
    #[must_use]
    pub fn cyclonedx(
        project_id: ProjectId,
        tenant_id: TenantId,
        content: serde_json::Value,
    ) -> Self {
        Self::new(
            project_id,
            tenant_id,
            SbomFormat::CycloneDx,
            Self::default_version(SbomFormat::CycloneDx).to_string(),
            content,
        )
    }

    /// Creates a new SPDX SBOM.
    #[must_use]
    pub fn spdx(project_id: ProjectId, tenant_id: TenantId, content: serde_json::Value) -> Self {
        Self::new(
            project_id,
            tenant_id,
            SbomFormat::Spdx,
            Self::default_version(SbomFormat::Spdx).to_string(),
            content,
        )
    }
}
