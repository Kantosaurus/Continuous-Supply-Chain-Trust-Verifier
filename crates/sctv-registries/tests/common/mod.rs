//! Common test utilities for registry integration tests.
//!
//! This module provides shared helpers, mock server setup, and test fixtures
//! used across all registry integration tests.

use sctv_core::{Package, PackageChecksums, PackageEcosystem, PackageId, PackageVersion};
use semver::Version;
use serde_json::json;
use std::sync::Arc;
use wiremock::{
    matchers::{header, method, path, path_regex},
    Mock, MockServer, ResponseTemplate,
};

/// Test fixture for npm package metadata.
pub fn npm_package_metadata(name: &str, versions: &[&str]) -> serde_json::Value {
    let mut version_map = serde_json::Map::new();
    let mut time_map = serde_json::Map::new();

    time_map.insert("created".to_string(), json!("2020-01-01T00:00:00.000Z"));
    time_map.insert("modified".to_string(), json!("2024-01-01T00:00:00.000Z"));

    for (i, version) in versions.iter().enumerate() {
        version_map.insert(
            version.to_string(),
            json!({
                "name": name,
                "version": *version,
                "dist": {
                    "shasum": format!("sha1hash{}", i),
                    "tarball": format!("https://registry.npmjs.org/{}/-/{}-{}.tgz", name, name, version),
                    "integrity": format!("sha512-test{}==", i),
                    "unpackedSize": 1000 + i * 100
                },
                "dependencies": {
                    "some-dep": "^1.0.0"
                }
            }),
        );
        time_map.insert(version.to_string(), json!(format!("2024-0{}-01T00:00:00.000Z", i + 1)));
    }

    json!({
        "name": name,
        "description": format!("Test package: {}", name),
        "dist-tags": {
            "latest": versions.last().unwrap_or(&"1.0.0")
        },
        "versions": version_map,
        "time": time_map,
        "maintainers": [
            { "name": "test-maintainer", "email": "test@example.com" }
        ],
        "repository": {
            "type": "git",
            "url": format!("git+https://github.com/test/{}.git", name)
        },
        "homepage": format!("https://github.com/test/{}", name)
    })
}

/// Test fixture for npm abbreviated package metadata.
pub fn npm_abbreviated_metadata(name: &str, versions: &[&str]) -> serde_json::Value {
    let mut version_map = serde_json::Map::new();

    for version in versions {
        version_map.insert(
            version.to_string(),
            json!({
                "name": name,
                "version": *version,
                "dist": {
                    "tarball": format!("https://registry.npmjs.org/{}/-/{}-{}.tgz", name, name, version),
                    "integrity": "sha512-test=="
                }
            }),
        );
    }

    json!({
        "name": name,
        "dist-tags": {
            "latest": versions.last().unwrap_or(&"1.0.0")
        },
        "versions": version_map,
        "modified": "2024-01-01T00:00:00.000Z"
    })
}

/// Test fixture for npm version metadata.
pub fn npm_version_metadata(name: &str, version: &str) -> serde_json::Value {
    json!({
        "name": name,
        "version": version,
        "description": format!("Version {} of {}", version, name),
        "main": "index.js",
        "scripts": {
            "test": "jest"
        },
        "dependencies": {
            "lodash": "^4.17.21"
        },
        "devDependencies": {
            "jest": "^29.0.0"
        },
        "dist": {
            "shasum": "abc123def456",
            "tarball": format!("https://registry.npmjs.org/{}/-/{}-{}.tgz", name, name, version),
            "integrity": "sha512-abcdef123456789==",
            "unpackedSize": 12345
        },
        "maintainers": [
            { "name": "maintainer1", "email": "maintainer1@example.com" }
        ]
    })
}

/// Test fixture for PyPI package metadata.
pub fn pypi_package_metadata(name: &str, version: &str) -> serde_json::Value {
    json!({
        "info": {
            "name": name,
            "version": version,
            "summary": format!("Test Python package: {}", name),
            "author": "Test Author",
            "author_email": "test@example.com",
            "home_page": format!("https://github.com/test/{}", name),
            "project_url": format!("https://pypi.org/project/{}/", name),
            "requires_dist": ["requests>=2.0.0", "numpy>=1.0.0"]
        },
        "releases": {
            version: [{
                "filename": format!("{}-{}.tar.gz", name, version),
                "url": format!("https://files.pythonhosted.org/packages/{}/{}-{}.tar.gz", name, name, version),
                "digests": {
                    "sha256": "abc123def456789",
                    "md5": "md5hash123"
                },
                "size": 50000,
                "upload_time": "2024-01-01T00:00:00"
            }]
        },
        "urls": [{
            "filename": format!("{}-{}.tar.gz", name, version),
            "url": format!("https://files.pythonhosted.org/packages/{}/{}-{}.tar.gz", name, name, version),
            "digests": {
                "sha256": "abc123def456789",
                "md5": "md5hash123"
            },
            "size": 50000
        }]
    })
}

/// Test fixture for Maven POM metadata.
pub fn maven_pom_metadata(group_id: &str, artifact_id: &str, version: &str) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<project xmlns="http://maven.apache.org/POM/4.0.0">
    <modelVersion>4.0.0</modelVersion>
    <groupId>{}</groupId>
    <artifactId>{}</artifactId>
    <version>{}</version>
    <packaging>jar</packaging>
    <name>{}</name>
    <description>Test Maven artifact</description>
    <url>https://github.com/test/{}</url>
    <dependencies>
        <dependency>
            <groupId>org.example</groupId>
            <artifactId>test-dep</artifactId>
            <version>1.0.0</version>
        </dependency>
    </dependencies>
</project>"#,
        group_id, artifact_id, version, artifact_id, artifact_id
    )
}

/// Test fixture for Maven metadata.xml.
pub fn maven_metadata(group_id: &str, artifact_id: &str, versions: &[&str]) -> String {
    let versions_xml: String = versions
        .iter()
        .map(|v| format!("        <version>{}</version>", v))
        .collect::<Vec<_>>()
        .join("\n");

    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<metadata>
    <groupId>{}</groupId>
    <artifactId>{}</artifactId>
    <versioning>
        <latest>{}</latest>
        <release>{}</release>
        <versions>
{}
        </versions>
        <lastUpdated>20240101000000</lastUpdated>
    </versioning>
</metadata>"#,
        group_id,
        artifact_id,
        versions.last().unwrap_or(&"1.0.0"),
        versions.last().unwrap_or(&"1.0.0"),
        versions_xml
    )
}

/// Sets up a mock npm registry server with common responses.
pub async fn setup_npm_mock_server() -> MockServer {
    let server = MockServer::start().await;
    server
}

/// Mounts a package metadata response on the mock server.
pub async fn mount_npm_package(server: &MockServer, name: &str, versions: &[&str]) {
    let encoded_name = if name.starts_with('@') {
        name.replace('/', "%2F")
    } else {
        name.to_string()
    };

    // Full package metadata
    Mock::given(method("GET"))
        .and(path(format!("/{}", encoded_name)))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(npm_package_metadata(name, versions)),
        )
        .mount(server)
        .await;
}

/// Mounts an abbreviated package metadata response.
pub async fn mount_npm_abbreviated(server: &MockServer, name: &str, versions: &[&str]) {
    let encoded_name = if name.starts_with('@') {
        name.replace('/', "%2F")
    } else {
        name.to_string()
    };

    Mock::given(method("GET"))
        .and(path(format!("/{}", encoded_name)))
        .and(header("Accept", "application/vnd.npm.install-v1+json"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(npm_abbreviated_metadata(name, versions)),
        )
        .mount(server)
        .await;
}

/// Mounts a specific version metadata response.
pub async fn mount_npm_version(server: &MockServer, name: &str, version: &str) {
    let encoded_name = if name.starts_with('@') {
        name.replace('/', "%2F")
    } else {
        name.to_string()
    };

    Mock::given(method("GET"))
        .and(path(format!("/{}/{}", encoded_name, version)))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(npm_version_metadata(name, version)),
        )
        .mount(server)
        .await;
}

/// Mounts a 404 response for a non-existent package.
pub async fn mount_npm_not_found(server: &MockServer, name: &str) {
    let encoded_name = if name.starts_with('@') {
        name.replace('/', "%2F")
    } else {
        name.to_string()
    };

    Mock::given(method("GET"))
        .and(path(format!("/{}", encoded_name)))
        .respond_with(
            ResponseTemplate::new(404)
                .set_body_json(json!({ "error": "Not found" })),
        )
        .mount(server)
        .await;
}

/// Mounts a rate limit response.
pub async fn mount_npm_rate_limit(server: &MockServer, name: &str, retry_after: u32) {
    let encoded_name = if name.starts_with('@') {
        name.replace('/', "%2F")
    } else {
        name.to_string()
    };

    Mock::given(method("GET"))
        .and(path(format!("/{}", encoded_name)))
        .respond_with(
            ResponseTemplate::new(429)
                .insert_header("Retry-After", retry_after.to_string())
                .set_body_string("rate limited"),
        )
        .mount(server)
        .await;
}

/// Mounts a tarball download response.
pub async fn mount_npm_tarball(server: &MockServer, name: &str, version: &str, content: &[u8]) {
    Mock::given(method("GET"))
        .and(path(format!("/{}/-/{}-{}.tgz", name, name, version)))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("Content-Type", "application/gzip")
                .set_body_bytes(content.to_vec()),
        )
        .mount(server)
        .await;
}

/// Sets up a mock PyPI server.
pub async fn setup_pypi_mock_server() -> MockServer {
    MockServer::start().await
}

/// Mounts PyPI package metadata.
pub async fn mount_pypi_package(server: &MockServer, name: &str, version: &str) {
    Mock::given(method("GET"))
        .and(path(format!("/pypi/{}/json", name)))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(pypi_package_metadata(name, version)),
        )
        .mount(server)
        .await;
}

/// Sets up a mock Maven server.
pub async fn setup_maven_mock_server() -> MockServer {
    MockServer::start().await
}

/// Mounts Maven metadata response.
pub async fn mount_maven_metadata(
    server: &MockServer,
    group_id: &str,
    artifact_id: &str,
    versions: &[&str],
) {
    let group_path = group_id.replace('.', "/");

    Mock::given(method("GET"))
        .and(path(format!(
            "/{}/{}/maven-metadata.xml",
            group_path, artifact_id
        )))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("Content-Type", "application/xml")
                .set_body_string(maven_metadata(group_id, artifact_id, versions)),
        )
        .mount(server)
        .await;
}

/// Mounts Maven POM response.
pub async fn mount_maven_pom(
    server: &MockServer,
    group_id: &str,
    artifact_id: &str,
    version: &str,
) {
    let group_path = group_id.replace('.', "/");

    Mock::given(method("GET"))
        .and(path(format!(
            "/{}/{}/{}/{}-{}.pom",
            group_path, artifact_id, version, artifact_id, version
        )))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("Content-Type", "application/xml")
                .set_body_string(maven_pom_metadata(group_id, artifact_id, version)),
        )
        .mount(server)
        .await;
}

/// Creates a test package with default values.
pub fn create_test_package(name: &str, ecosystem: PackageEcosystem) -> Package {
    Package {
        id: PackageId::new(),
        ecosystem,
        name: name.to_string(),
        normalized_name: name.to_lowercase().replace('-', "_"),
        description: Some(format!("Test package: {}", name)),
        homepage: None,
        repository: None,
        popularity_rank: None,
        is_popular: false,
        maintainers: vec!["test-maintainer".to_string()],
        first_published: Some(chrono::Utc::now()),
        last_updated: Some(chrono::Utc::now()),
        cached_at: chrono::Utc::now(),
    }
}

/// Creates a test package version with default values.
pub fn create_test_version(version_str: &str) -> PackageVersion {
    PackageVersion {
        package_id: PackageId::new(),
        version: Version::parse(version_str).unwrap_or_else(|_| Version::new(1, 0, 0)),
        published_at: Some(chrono::Utc::now()),
        yanked: false,
        deprecated: false,
        deprecation_message: None,
        checksums: PackageChecksums {
            sha1: None,
            sha256: Some("abc123def456".to_string()),
            sha512: Some("sha512hash".to_string()),
            integrity: Some("sha512-testintegrity==".to_string()),
        },
        download_url: None,
        size_bytes: Some(10000),
        attestations: Vec::new(),
        dependencies: Vec::new(),
        cached_at: chrono::Utc::now(),
    }
}

/// Test helper to verify integrity results.
pub fn assert_integrity_valid(checksums: &PackageChecksums) {
    assert!(
        checksums.sha256.is_some() || checksums.sha512.is_some() || checksums.integrity.is_some(),
        "At least one checksum should be present"
    );
}
