//! `PyPI` API response models.

use serde::Deserialize;
use std::collections::HashMap;

/// Full package response from `PyPI` JSON API.
#[derive(Debug, Deserialize)]
pub struct PyPiPackageResponse {
    pub info: PyPiPackageInfo,
    /// Map of version string to list of release files.
    pub releases: HashMap<String, Vec<PyPiReleaseFile>>,
    /// Release files for the latest version.
    pub urls: Vec<PyPiReleaseFile>,
}

/// Package metadata from `PyPI`.
#[derive(Debug, Deserialize)]
pub struct PyPiPackageInfo {
    pub name: String,
    pub version: String,
    pub summary: Option<String>,
    pub description: Option<String>,
    pub description_content_type: Option<String>,
    pub author: Option<String>,
    pub author_email: Option<String>,
    pub maintainer: Option<String>,
    pub maintainer_email: Option<String>,
    pub license: Option<String>,
    pub keywords: Option<String>,
    pub classifiers: Option<Vec<String>>,
    pub platform: Option<String>,
    pub home_page: Option<String>,
    pub download_url: Option<String>,
    pub project_url: Option<String>,
    pub project_urls: Option<HashMap<String, String>>,
    pub requires_dist: Option<Vec<String>>,
    pub requires_python: Option<String>,
    pub bugtrack_url: Option<String>,
    pub docs_url: Option<String>,
    pub package_url: Option<String>,
    pub release_url: Option<String>,
    pub yanked: Option<bool>,
    pub yanked_reason: Option<String>,
}

/// A release file (wheel, sdist, etc.) from `PyPI`.
#[derive(Debug, Clone, Deserialize)]
pub struct PyPiReleaseFile {
    /// Comment text (usually empty).
    pub comment_text: Option<String>,
    /// File digests.
    pub digests: PyPiDigests,
    /// Number of downloads (deprecated, usually -1).
    pub downloads: Option<i64>,
    /// Filename.
    pub filename: String,
    /// Whether the file has a GPG signature.
    pub has_sig: Option<bool>,
    /// MD5 digest (deprecated).
    pub md5_digest: Option<String>,
    /// Package type (e.g., "`bdist_wheel`", "sdist").
    pub packagetype: Option<String>,
    /// Python version (e.g., "py3", "source").
    pub python_version: Option<String>,
    /// Required Python version.
    pub requires_python: Option<String>,
    /// File size in bytes.
    pub size: Option<u64>,
    /// Upload time (ISO format string).
    pub upload_time: Option<String>,
    /// Upload time in ISO format.
    pub upload_time_iso_8601: Option<String>,
    /// Download URL for the file.
    pub url: String,
    /// Whether this release has been yanked.
    pub yanked: Option<bool>,
    /// Reason for yanking.
    pub yanked_reason: Option<String>,
}

/// File digests from `PyPI`.
#[derive(Debug, Clone, Deserialize)]
pub struct PyPiDigests {
    pub md5: Option<String>,
    pub sha256: Option<String>,
    /// Blake2b-256 hash.
    pub blake2b_256: Option<String>,
}

/// Version-specific response from `PyPI`.
#[derive(Debug, Deserialize)]
pub struct PyPiVersionResponse {
    pub info: PyPiPackageInfo,
    /// Release files for this specific version.
    pub urls: Vec<PyPiReleaseFile>,
}

/// Provenance attestation from `PyPI` (PEP 740).
#[derive(Debug, Clone, Deserialize)]
pub struct PyPiAttestation {
    pub version: u32,
    pub verification_material: Option<PyPiVerificationMaterial>,
    pub envelope: Option<PyPiEnvelope>,
}

/// Verification material for attestations.
#[derive(Debug, Clone, Deserialize)]
pub struct PyPiVerificationMaterial {
    pub certificate: Option<String>,
    pub transparency_entries: Option<Vec<serde_json::Value>>,
}

/// DSSE envelope for attestations.
#[derive(Debug, Clone, Deserialize)]
pub struct PyPiEnvelope {
    pub statement: Option<String>,
    pub signature: Option<String>,
}

/// Simple API index entry (for package listing).
#[derive(Debug, Deserialize)]
pub struct PyPiSimpleIndex {
    pub projects: Vec<PyPiSimpleProject>,
}

/// Project entry in simple API.
#[derive(Debug, Deserialize)]
pub struct PyPiSimpleProject {
    pub name: String,
}

/// Parsed dependency from `requires_dist`.
#[derive(Debug, Clone)]
pub struct PyPiDependency {
    pub name: String,
    pub version_constraint: Option<String>,
    pub extras: Vec<String>,
    pub markers: Option<String>,
    pub is_optional: bool,
}

impl PyPiDependency {
    /// Parses a `requires_dist` entry into a dependency.
    #[must_use]
    pub fn parse(spec: &str) -> Option<Self> {
        let spec = spec.trim();
        if spec.is_empty() {
            return None;
        }

        // Parse format: "name[extra1,extra2] (>=1.0,<2.0); marker"
        let (rest, markers) = if let Some(idx) = spec.find(';') {
            (&spec[..idx], Some(spec[idx + 1..].trim().to_string()))
        } else {
            (spec, None)
        };

        let (rest, version_constraint) = if let Some(idx) = rest.find('(') {
            let end_idx = rest.rfind(')').unwrap_or(rest.len());
            (
                &rest[..idx],
                Some(rest[idx + 1..end_idx].trim().to_string()),
            )
        } else if let Some(idx) = rest.find(['<', '>', '=', '~', '!']) {
            (&rest[..idx], Some(rest[idx..].trim().to_string()))
        } else {
            (rest, None)
        };

        let (name, extras) = if let Some(idx) = rest.find('[') {
            let end_idx = rest.find(']').unwrap_or(rest.len());
            let extras: Vec<String> = rest[idx + 1..end_idx]
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
            (&rest[..idx], extras)
        } else {
            (rest, Vec::new())
        };

        let name = name.trim().to_string();
        if name.is_empty() {
            return None;
        }

        // Check if this is an optional dependency (has environment marker)
        let is_optional = markers
            .as_ref()
            .is_some_and(|m| m.contains("extra ==") || m.contains("extra=="));

        Some(Self {
            name,
            version_constraint,
            extras,
            markers,
            is_optional,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_dependency() {
        let dep = PyPiDependency::parse("requests").unwrap();
        assert_eq!(dep.name, "requests");
        assert!(dep.version_constraint.is_none());
        assert!(dep.extras.is_empty());
    }

    #[test]
    fn test_parse_versioned_dependency() {
        let dep = PyPiDependency::parse("requests>=2.20.0").unwrap();
        assert_eq!(dep.name, "requests");
        assert_eq!(dep.version_constraint.as_deref(), Some(">=2.20.0"));
    }

    #[test]
    fn test_parse_dependency_with_extras() {
        let dep = PyPiDependency::parse("requests[security]>=2.20.0").unwrap();
        assert_eq!(dep.name, "requests");
        assert_eq!(dep.extras, vec!["security"]);
    }

    #[test]
    fn test_parse_dependency_with_markers() {
        let dep = PyPiDependency::parse("pywin32; sys_platform == 'win32'").unwrap();
        assert_eq!(dep.name, "pywin32");
        assert_eq!(dep.markers.as_deref(), Some("sys_platform == 'win32'"));
    }
}
