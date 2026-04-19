//! Maven Central API and XML response models.

use serde::Deserialize;

/// Maven coordinate (groupId:artifactId).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MavenCoordinate {
    pub group_id: String,
    pub artifact_id: String,
}

impl MavenCoordinate {
    /// Creates a new Maven coordinate.
    pub fn new(group_id: impl Into<String>, artifact_id: impl Into<String>) -> Self {
        Self {
            group_id: group_id.into(),
            artifact_id: artifact_id.into(),
        }
    }

    /// Parses a coordinate from "groupId:artifactId" format.
    pub fn parse(name: &str) -> Option<Self> {
        let parts: Vec<&str> = name.split(':').collect();
        if parts.len() >= 2 {
            Some(Self {
                group_id: parts[0].to_string(),
                artifact_id: parts[1].to_string(),
            })
        } else {
            None
        }
    }

    /// Returns the repository path for this coordinate.
    pub fn repo_path(&self) -> String {
        format!("{}/{}", self.group_id.replace('.', "/"), self.artifact_id)
    }

    /// Returns the artifact path for a specific version.
    pub fn artifact_path(&self, version: &str, extension: &str) -> String {
        format!(
            "{}/{}/{}-{}.{}",
            self.repo_path(),
            version,
            self.artifact_id,
            version,
            extension
        )
    }

    /// Returns the full coordinate string.
    pub fn to_string(&self) -> String {
        format!("{}:{}", self.group_id, self.artifact_id)
    }
}

/// Maven metadata XML structure.
#[derive(Debug, Deserialize)]
#[serde(rename = "metadata")]
pub struct MavenMetadata {
    #[serde(rename = "groupId")]
    pub group_id: Option<String>,
    #[serde(rename = "artifactId")]
    pub artifact_id: Option<String>,
    pub versioning: Option<MavenVersioning>,
}

/// Versioning information from maven-metadata.xml.
#[derive(Debug, Deserialize)]
pub struct MavenVersioning {
    pub latest: Option<String>,
    pub release: Option<String>,
    pub versions: Option<MavenVersions>,
    #[serde(rename = "lastUpdated")]
    pub last_updated: Option<String>,
}

/// List of versions from maven-metadata.xml.
#[derive(Debug, Deserialize)]
pub struct MavenVersions {
    #[serde(rename = "version", default)]
    pub version: Vec<String>,
}

/// Maven POM file structure (simplified).
#[derive(Debug, Deserialize)]
#[serde(rename = "project")]
pub struct MavenPom {
    #[serde(rename = "modelVersion")]
    pub model_version: Option<String>,
    #[serde(rename = "groupId")]
    pub group_id: Option<String>,
    #[serde(rename = "artifactId")]
    pub artifact_id: Option<String>,
    pub version: Option<String>,
    pub packaging: Option<String>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub url: Option<String>,
    pub licenses: Option<MavenLicenses>,
    pub developers: Option<MavenDevelopers>,
    pub scm: Option<MavenScm>,
    pub dependencies: Option<MavenDependencies>,
    pub parent: Option<MavenParent>,
    pub properties: Option<serde_json::Value>,
}

/// Parent POM reference.
#[derive(Debug, Deserialize)]
pub struct MavenParent {
    #[serde(rename = "groupId")]
    pub group_id: Option<String>,
    #[serde(rename = "artifactId")]
    pub artifact_id: Option<String>,
    pub version: Option<String>,
    #[serde(rename = "relativePath")]
    pub relative_path: Option<String>,
}

/// Licenses section.
#[derive(Debug, Deserialize)]
pub struct MavenLicenses {
    #[serde(rename = "license", default)]
    pub license: Vec<MavenLicense>,
}

/// Individual license.
#[derive(Debug, Deserialize)]
pub struct MavenLicense {
    pub name: Option<String>,
    pub url: Option<String>,
    pub distribution: Option<String>,
    pub comments: Option<String>,
}

/// Developers section.
#[derive(Debug, Deserialize)]
pub struct MavenDevelopers {
    #[serde(rename = "developer", default)]
    pub developer: Vec<MavenDeveloper>,
}

/// Individual developer.
#[derive(Debug, Deserialize)]
pub struct MavenDeveloper {
    pub id: Option<String>,
    pub name: Option<String>,
    pub email: Option<String>,
    pub url: Option<String>,
    pub organization: Option<String>,
    #[serde(rename = "organizationUrl")]
    pub organization_url: Option<String>,
    pub roles: Option<MavenRoles>,
    pub timezone: Option<String>,
}

/// Developer roles.
#[derive(Debug, Deserialize)]
pub struct MavenRoles {
    #[serde(rename = "role", default)]
    pub role: Vec<String>,
}

/// SCM (Source Control Management) information.
#[derive(Debug, Deserialize)]
pub struct MavenScm {
    pub connection: Option<String>,
    #[serde(rename = "developerConnection")]
    pub developer_connection: Option<String>,
    pub url: Option<String>,
    pub tag: Option<String>,
}

/// Dependencies section.
#[derive(Debug, Deserialize)]
pub struct MavenDependencies {
    #[serde(rename = "dependency", default)]
    pub dependency: Vec<MavenDependency>,
}

/// Individual dependency.
#[derive(Debug, Deserialize)]
pub struct MavenDependency {
    #[serde(rename = "groupId")]
    pub group_id: Option<String>,
    #[serde(rename = "artifactId")]
    pub artifact_id: Option<String>,
    pub version: Option<String>,
    #[serde(rename = "type")]
    pub dependency_type: Option<String>,
    pub scope: Option<String>,
    pub classifier: Option<String>,
    pub optional: Option<String>,
    pub exclusions: Option<MavenExclusions>,
}

impl MavenDependency {
    /// Returns the full coordinate for this dependency.
    pub fn coordinate(&self) -> Option<String> {
        match (&self.group_id, &self.artifact_id) {
            (Some(g), Some(a)) => Some(format!("{}:{}", g, a)),
            _ => None,
        }
    }

    /// Returns true if this is an optional dependency.
    pub fn is_optional(&self) -> bool {
        self.optional.as_deref() == Some("true")
    }

    /// Returns true if this is a test-scoped dependency.
    pub fn is_test(&self) -> bool {
        self.scope.as_deref() == Some("test")
    }

    /// Returns true if this is a provided-scoped dependency.
    pub fn is_provided(&self) -> bool {
        self.scope.as_deref() == Some("provided")
    }
}

/// Exclusions for a dependency.
#[derive(Debug, Deserialize)]
pub struct MavenExclusions {
    #[serde(rename = "exclusion", default)]
    pub exclusion: Vec<MavenExclusion>,
}

/// Individual exclusion.
#[derive(Debug, Deserialize)]
pub struct MavenExclusion {
    #[serde(rename = "groupId")]
    pub group_id: Option<String>,
    #[serde(rename = "artifactId")]
    pub artifact_id: Option<String>,
}

/// Maven Central Search API response.
#[derive(Debug, Deserialize)]
pub struct MavenSearchResponse {
    pub response: MavenSearchResponseBody,
}

/// Search response body.
#[derive(Debug, Deserialize)]
pub struct MavenSearchResponseBody {
    #[serde(rename = "numFound")]
    pub num_found: u64,
    pub start: u64,
    pub docs: Vec<MavenSearchDoc>,
}

/// Individual search result document.
#[derive(Debug, Deserialize)]
pub struct MavenSearchDoc {
    pub id: String,
    pub g: String,
    pub a: String,
    pub v: Option<String>,
    #[serde(rename = "latestVersion")]
    pub latest_version: Option<String>,
    #[serde(rename = "repositoryId")]
    pub repository_id: Option<String>,
    pub p: Option<String>,
    pub timestamp: Option<u64>,
    #[serde(rename = "versionCount")]
    pub version_count: Option<u64>,
    pub text: Option<Vec<String>>,
    pub ec: Option<Vec<String>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_coordinate_parse() {
        let coord = MavenCoordinate::parse("org.apache.commons:commons-lang3").unwrap();
        assert_eq!(coord.group_id, "org.apache.commons");
        assert_eq!(coord.artifact_id, "commons-lang3");
    }

    #[test]
    fn test_coordinate_repo_path() {
        let coord = MavenCoordinate::new("org.apache.commons", "commons-lang3");
        assert_eq!(coord.repo_path(), "org/apache/commons/commons-lang3");
    }

    #[test]
    fn test_coordinate_artifact_path() {
        let coord = MavenCoordinate::new("org.apache.commons", "commons-lang3");
        assert_eq!(
            coord.artifact_path("3.12.0", "jar"),
            "org/apache/commons/commons-lang3/3.12.0/commons-lang3-3.12.0.jar"
        );
    }

    #[test]
    fn test_invalid_coordinate() {
        assert!(MavenCoordinate::parse("invalid").is_none());
    }
}
