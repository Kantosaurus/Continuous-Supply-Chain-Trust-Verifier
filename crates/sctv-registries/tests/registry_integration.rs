//! Integration tests for registry clients using mock HTTP servers.
//!
//! These tests verify the registry client implementations work correctly
//! against simulated registry responses, including:
//! - Package metadata retrieval
//! - Version-specific metadata fetching
//! - Package download functionality
//! - Error handling (404, rate limiting, server errors)
//! - Caching behavior
//! - Scoped package handling

mod common;

use sctv_core::PackageEcosystem;
use sctv_registries::{npm::NpmClient, RegistryCache, RegistryClient, RegistryError};
use std::sync::Arc;
use wiremock::{
    matchers::{method, path},
    Mock, MockServer, ResponseTemplate,
};

use common::*;

// =============================================================================
// npm Registry Client Tests
// =============================================================================

mod npm_client {
    use super::*;

    /// Test that the npm client can fetch full package metadata.
    #[tokio::test]
    async fn test_get_package_success() {
        let server = setup_npm_mock_server().await;
        mount_npm_package(&server, "lodash", &["4.17.20", "4.17.21"]).await;

        let cache = Arc::new(RegistryCache::new());
        let client = NpmClient::with_config(&server.uri(), cache);

        let result = client.get_package("lodash").await;

        assert!(result.is_ok(), "Expected successful response: {result:?}");
        let metadata = result.unwrap();

        assert_eq!(metadata.package.name, "lodash");
        assert_eq!(metadata.package.ecosystem, PackageEcosystem::Npm);
        assert_eq!(metadata.available_versions.len(), 2);
        assert!(metadata.available_versions.contains(&"4.17.20".to_string()));
        assert!(metadata.available_versions.contains(&"4.17.21".to_string()));
        assert_eq!(metadata.latest_version, Some("4.17.21".to_string()));
    }

    /// Test fetching a scoped npm package.
    #[tokio::test]
    async fn test_get_scoped_package() {
        let server = setup_npm_mock_server().await;
        mount_npm_package(&server, "@babel/core", &["7.23.0", "7.24.0"]).await;

        let cache = Arc::new(RegistryCache::new());
        let client = NpmClient::with_config(&server.uri(), cache);

        let result = client.get_package("@babel/core").await;

        assert!(result.is_ok(), "Expected successful response: {result:?}");
        let metadata = result.unwrap();

        assert_eq!(metadata.package.name, "@babel/core");
        assert_eq!(metadata.available_versions.len(), 2);
    }

    /// Test handling of non-existent package.
    #[tokio::test]
    async fn test_package_not_found() {
        let server = setup_npm_mock_server().await;
        mount_npm_not_found(&server, "nonexistent-package-xyz").await;

        let cache = Arc::new(RegistryCache::new());
        let client = NpmClient::with_config(&server.uri(), cache);

        let result = client.get_package("nonexistent-package-xyz").await;

        assert!(matches!(result, Err(RegistryError::PackageNotFound(_))));
    }

    /// Test rate limiting response handling.
    #[tokio::test]
    async fn test_rate_limiting() {
        let server = setup_npm_mock_server().await;
        mount_npm_rate_limit(&server, "rate-limited-pkg", 60).await;

        let cache = Arc::new(RegistryCache::new());
        let client = NpmClient::with_config(&server.uri(), cache);

        let result = client.get_package("rate-limited-pkg").await;

        assert!(matches!(result, Err(RegistryError::RateLimited)));
    }

    /// Test server error handling.
    #[tokio::test]
    async fn test_server_error() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/server-error-pkg"))
            .respond_with(ResponseTemplate::new(500).set_body_string("Internal Server Error"))
            .mount(&server)
            .await;

        let cache = Arc::new(RegistryCache::new());
        let client = NpmClient::with_config(&server.uri(), cache);

        let result = client.get_package("server-error-pkg").await;

        assert!(matches!(result, Err(RegistryError::Unavailable(_))));
    }

    /// Test fetching specific version metadata.
    #[tokio::test]
    async fn test_get_version_success() {
        let server = setup_npm_mock_server().await;
        mount_npm_version(&server, "express", "4.18.2").await;

        let cache = Arc::new(RegistryCache::new());
        let client = NpmClient::with_config(&server.uri(), cache);

        let result = client.get_version("express", "4.18.2").await;

        assert!(result.is_ok(), "Expected successful response: {result:?}");
        let version_meta = result.unwrap();

        assert_eq!(version_meta.version.version.to_string(), "4.18.2");
        assert!(version_meta.download_url.is_some());
        assert!(version_meta.version.checksums.integrity.is_some());
    }

    /// Test version not found handling.
    #[tokio::test]
    async fn test_version_not_found() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/some-package/99.99.99"))
            .respond_with(ResponseTemplate::new(404).set_body_json(serde_json::json!({
                "error": "version not found"
            })))
            .mount(&server)
            .await;

        let cache = Arc::new(RegistryCache::new());
        let client = NpmClient::with_config(&server.uri(), cache);

        let result = client.get_version("some-package", "99.99.99").await;

        assert!(matches!(result, Err(RegistryError::VersionNotFound(_, _))));
    }

    /// Test package existence check.
    #[tokio::test]
    async fn test_package_exists_true() {
        let server = setup_npm_mock_server().await;
        mount_npm_abbreviated(&server, "react", &["18.2.0"]).await;

        let cache = Arc::new(RegistryCache::new());
        let client = NpmClient::with_config(&server.uri(), cache);

        let result = client.package_exists("react").await;

        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    /// Test package existence check for non-existent package.
    #[tokio::test]
    async fn test_package_exists_false() {
        let server = setup_npm_mock_server().await;
        mount_npm_not_found(&server, "fake-package-that-does-not-exist").await;

        let cache = Arc::new(RegistryCache::new());
        let client = NpmClient::with_config(&server.uri(), cache);

        let result = client
            .package_exists("fake-package-that-does-not-exist")
            .await;

        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    /// Test package download functionality.
    #[tokio::test]
    async fn test_download_package() {
        let server = MockServer::start().await;

        // Mount version metadata with tarball URL pointing to mock server
        Mock::given(method("GET"))
            .and(path("/test-pkg/1.0.0"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "name": "test-pkg",
                "version": "1.0.0",
                "dist": {
                    "shasum": "abc123",
                    "tarball": format!("{}/test-pkg/-/test-pkg-1.0.0.tgz", server.uri()),
                    "integrity": "sha512-test=="
                }
            })))
            .mount(&server)
            .await;

        // Mount the tarball download
        let tarball_content = b"fake tarball content for testing";
        Mock::given(method("GET"))
            .and(path("/test-pkg/-/test-pkg-1.0.0.tgz"))
            .respond_with(
                ResponseTemplate::new(200)
                    .insert_header("Content-Type", "application/gzip")
                    .set_body_bytes(tarball_content.to_vec()),
            )
            .mount(&server)
            .await;

        let cache = Arc::new(RegistryCache::new());
        let client = NpmClient::with_config(&server.uri(), cache);

        let result = client.download_package("test-pkg", "1.0.0").await;

        assert!(result.is_ok(), "Expected successful download: {result:?}");
        let bytes = result.unwrap();
        assert_eq!(bytes.as_ref(), tarball_content);
    }

    /// Test listing popular packages.
    #[tokio::test]
    async fn test_list_popular() {
        let cache = Arc::new(RegistryCache::new());
        let client = NpmClient::with_config("https://registry.npmjs.org", cache);

        let result = client.list_popular(10).await;

        assert!(result.is_ok());
        let packages = result.unwrap();
        assert_eq!(packages.len(), 10);
        assert!(packages.contains(&"lodash".to_string()));
        assert!(packages.contains(&"react".to_string()));
    }

    /// Test caching behavior - second request should use cache.
    #[tokio::test]
    async fn test_caching_behavior() {
        let server = setup_npm_mock_server().await;

        // Mount with expect(1) to verify only one request is made
        Mock::given(method("GET"))
            .and(path("/cached-pkg"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(npm_package_metadata("cached-pkg", &["1.0.0"])),
            )
            .expect(1)
            .mount(&server)
            .await;

        let cache = Arc::new(RegistryCache::new());
        let client = NpmClient::with_config(&server.uri(), cache);

        // First request - hits the server
        let result1 = client.get_package("cached-pkg").await;
        assert!(result1.is_ok());

        // Second request - should use cache
        let result2 = client.get_package("cached-pkg").await;
        assert!(result2.is_ok());

        // Both should return same data
        assert_eq!(result1.unwrap().package.name, result2.unwrap().package.name);
    }

    /// Test deprecated package detection.
    #[tokio::test]
    async fn test_deprecated_version() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/deprecated-pkg/1.0.0"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "name": "deprecated-pkg",
                "version": "1.0.0",
                "deprecated": "This version is deprecated, please upgrade to 2.0.0",
                "dist": {
                    "shasum": "abc123",
                    "tarball": "https://registry.npmjs.org/deprecated-pkg/-/deprecated-pkg-1.0.0.tgz",
                    "integrity": "sha512-test=="
                }
            })))
            .mount(&server)
            .await;

        let cache = Arc::new(RegistryCache::new());
        let client = NpmClient::with_config(&server.uri(), cache);

        let result = client.get_version("deprecated-pkg", "1.0.0").await;

        assert!(result.is_ok());
        let version = result.unwrap();
        assert!(version.version.deprecated);
        assert!(version.version.deprecation_message.is_some());
        assert!(version
            .version
            .deprecation_message
            .unwrap()
            .contains("deprecated"));
    }

    /// Test parsing dependencies from version metadata.
    #[tokio::test]
    async fn test_version_dependencies_parsing() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/pkg-with-deps/1.0.0"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "name": "pkg-with-deps",
                "version": "1.0.0",
                "dependencies": {
                    "lodash": "^4.17.21",
                    "express": "^4.18.0"
                },
                "devDependencies": {
                    "jest": "^29.0.0",
                    "typescript": "^5.0.0"
                },
                "dist": {
                    "shasum": "abc123",
                    "tarball": "https://registry.npmjs.org/pkg-with-deps/-/pkg-with-deps-1.0.0.tgz",
                    "integrity": "sha512-test=="
                }
            })))
            .mount(&server)
            .await;

        let cache = Arc::new(RegistryCache::new());
        let client = NpmClient::with_config(&server.uri(), cache);

        let result = client.get_version("pkg-with-deps", "1.0.0").await;

        assert!(result.is_ok());
        let version = result.unwrap();

        // Should have both regular and dev dependencies
        assert_eq!(version.version.dependencies.len(), 4);

        let regular_dep_count = version
            .version
            .dependencies
            .iter()
            .filter(|d| !d.is_dev)
            .count();
        let dev_dep_count = version
            .version
            .dependencies
            .iter()
            .filter(|d| d.is_dev)
            .count();

        assert_eq!(regular_dep_count, 2);
        assert_eq!(dev_dep_count, 2);
    }

    /// Test malformed JSON response handling.
    #[tokio::test]
    async fn test_malformed_response() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/malformed-pkg"))
            .respond_with(ResponseTemplate::new(200).set_body_string("this is not valid json {{{"))
            .mount(&server)
            .await;

        let cache = Arc::new(RegistryCache::new());
        let client = NpmClient::with_config(&server.uri(), cache);

        let result = client.get_package("malformed-pkg").await;

        assert!(matches!(result, Err(RegistryError::Parse(_))));
    }

    /// Test connection timeout handling.
    #[tokio::test]
    async fn test_slow_response() {
        use std::time::Duration;

        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/slow-pkg"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(npm_package_metadata("slow-pkg", &["1.0.0"]))
                    .set_delay(Duration::from_millis(100)), // Small delay
            )
            .mount(&server)
            .await;

        let cache = Arc::new(RegistryCache::new());
        let client = NpmClient::with_config(&server.uri(), cache);

        // Should still succeed with a small delay
        let result = client.get_package("slow-pkg").await;
        assert!(result.is_ok());
    }

    /// Test ecosystem method returns correct value.
    #[tokio::test]
    async fn test_ecosystem() {
        let client = NpmClient::new();
        assert_eq!(client.ecosystem(), PackageEcosystem::Npm);
    }

    /// Test base URL accessor.
    #[tokio::test]
    async fn test_base_url() {
        let cache = Arc::new(RegistryCache::new());
        let client = NpmClient::with_config("https://custom.registry.example.com", cache);

        assert_eq!(
            client.base_url().as_str(),
            "https://custom.registry.example.com/"
        );
    }
}

// =============================================================================
// Registry Cache Tests
// =============================================================================

mod cache_tests {
    use super::*;

    /// Test cache stores and retrieves package metadata.
    #[tokio::test]
    async fn test_cache_package_storage() {
        let cache = RegistryCache::new();

        let metadata = sctv_registries::PackageMetadata {
            package: create_test_package("test-pkg", PackageEcosystem::Npm),
            available_versions: vec!["1.0.0".to_string(), "2.0.0".to_string()],
            latest_version: Some("2.0.0".to_string()),
        };

        cache.set_package(PackageEcosystem::Npm, "test-pkg", metadata);

        let cached = cache.get_package(PackageEcosystem::Npm, "test-pkg");
        assert!(cached.is_some());
        assert_eq!(cached.unwrap().package.name, "test-pkg");
    }

    /// Test cache miss for non-existent package.
    #[tokio::test]
    async fn test_cache_miss() {
        let cache = RegistryCache::new();

        let cached = cache.get_package(PackageEcosystem::Npm, "not-cached");
        assert!(cached.is_none());
    }

    /// Test version caching.
    #[tokio::test]
    async fn test_cache_version_storage() {
        let cache = RegistryCache::new();

        let version_meta = sctv_registries::VersionMetadata {
            version: create_test_version("1.0.0"),
            download_url: Some(url::Url::parse("https://example.com/pkg.tgz").unwrap()),
        };

        cache.set_version(PackageEcosystem::Npm, "test-pkg", "1.0.0", version_meta);

        let cached = cache.get_version(PackageEcosystem::Npm, "test-pkg", "1.0.0");
        assert!(cached.is_some());
        assert_eq!(cached.unwrap().version.version.to_string(), "1.0.0");
    }

    /// Test different ecosystems are cached separately.
    #[tokio::test]
    async fn test_cache_ecosystem_isolation() {
        let cache = RegistryCache::new();

        let npm_metadata = sctv_registries::PackageMetadata {
            package: create_test_package("test-pkg", PackageEcosystem::Npm),
            available_versions: vec!["1.0.0".to_string()],
            latest_version: Some("1.0.0".to_string()),
        };

        let pypi_metadata = sctv_registries::PackageMetadata {
            package: create_test_package("test-pkg", PackageEcosystem::PyPi),
            available_versions: vec!["2.0.0".to_string()],
            latest_version: Some("2.0.0".to_string()),
        };

        cache.set_package(PackageEcosystem::Npm, "test-pkg", npm_metadata);
        cache.set_package(PackageEcosystem::PyPi, "test-pkg", pypi_metadata);

        let npm_cached = cache.get_package(PackageEcosystem::Npm, "test-pkg");
        let pypi_cached = cache.get_package(PackageEcosystem::PyPi, "test-pkg");

        assert!(npm_cached.is_some());
        assert!(pypi_cached.is_some());
        assert_eq!(npm_cached.unwrap().available_versions[0], "1.0.0");
        assert_eq!(pypi_cached.unwrap().available_versions[0], "2.0.0");
    }
}

// =============================================================================
// Integrity Verification Tests
// =============================================================================

mod integrity_tests {
    use super::*;
    use bytes::Bytes;
    use sctv_core::PackageChecksums;
    use sha2::{Digest, Sha256, Sha512};

    /// Test integrity verification with matching SHA256.
    #[tokio::test]
    async fn test_verify_integrity_sha256_match() {
        let content = b"test package content";
        let bytes = Bytes::from_static(content);

        // Compute expected hash
        let mut hasher = Sha256::new();
        hasher.update(content);
        let expected_hash = hex::encode(hasher.finalize());

        let checksums = PackageChecksums {
            sha1: None,
            sha256: Some(expected_hash),
            sha512: None,
            integrity: None,
        };

        let client = NpmClient::new();
        let result = client.verify_integrity(&bytes, &checksums);

        assert!(result.sha256_match.unwrap_or(false));
        assert!(result.is_valid());
        assert!(!result.has_failure());
    }

    /// Test integrity verification with mismatched hash.
    #[tokio::test]
    async fn test_verify_integrity_mismatch() {
        let content = b"actual content";
        let bytes = Bytes::from_static(content);

        let checksums = PackageChecksums {
            sha1: None,
            sha256: Some("wrong_hash_value".to_string()),
            sha512: None,
            integrity: None,
        };

        let client = NpmClient::new();
        let result = client.verify_integrity(&bytes, &checksums);

        assert!(!result.sha256_match.unwrap_or(true));
        assert!(result.has_failure());
        assert!(!result.is_valid());
    }

    /// Test npm integrity field verification.
    #[tokio::test]
    async fn test_verify_npm_integrity_field() {
        let content = b"test content for integrity check";
        let bytes = Bytes::from_static(content);

        // Compute expected integrity
        let mut hasher = Sha512::new();
        hasher.update(content);
        let hash_base64 = base64::Engine::encode(
            &base64::engine::general_purpose::STANDARD,
            hasher.finalize(),
        );
        let integrity = format!("sha512-{hash_base64}");

        let checksums = PackageChecksums {
            sha1: None,
            sha256: None,
            sha512: None,
            integrity: Some(integrity),
        };

        let client = NpmClient::new();
        let result = client.verify_integrity(&bytes, &checksums);

        assert!(result.integrity_match.unwrap_or(false));
        assert!(result.is_valid());
    }

    /// Test verification with multiple checksums.
    #[tokio::test]
    async fn test_verify_multiple_checksums() {
        let content = b"content with multiple checksums";
        let bytes = Bytes::from_static(content);

        // Compute expected hashes
        let sha256_hash = {
            let mut hasher = Sha256::new();
            hasher.update(content);
            hex::encode(hasher.finalize())
        };

        let sha512_hash = {
            let mut hasher = Sha512::new();
            hasher.update(content);
            hex::encode(hasher.finalize())
        };

        let checksums = PackageChecksums {
            sha1: None,
            sha256: Some(sha256_hash),
            sha512: Some(sha512_hash),
            integrity: None,
        };

        let client = NpmClient::new();
        let result = client.verify_integrity(&bytes, &checksums);

        assert!(result.sha256_match.unwrap_or(false));
        assert!(result.sha512_match.unwrap_or(false));
        assert!(result.is_valid());
    }
}

// =============================================================================
// Concurrent Access Tests
// =============================================================================

mod concurrent_tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    /// Test concurrent requests to the same package.
    #[tokio::test]
    async fn test_concurrent_package_requests() {
        let server = setup_npm_mock_server().await;

        let request_count = Arc::new(AtomicUsize::new(0));
        let counter = request_count.clone();

        Mock::given(method("GET"))
            .and(path("/concurrent-pkg"))
            .respond_with(move |_req: &wiremock::Request| {
                counter.fetch_add(1, Ordering::SeqCst);
                ResponseTemplate::new(200)
                    .set_body_json(npm_package_metadata("concurrent-pkg", &["1.0.0"]))
            })
            .mount(&server)
            .await;

        let cache = Arc::new(RegistryCache::new());
        let client = Arc::new(NpmClient::with_config(&server.uri(), cache));

        // Spawn multiple concurrent requests
        let mut handles = vec![];
        for _ in 0..10 {
            let client_clone = client.clone();
            handles.push(tokio::spawn(async move {
                client_clone.get_package("concurrent-pkg").await
            }));
        }

        // Wait for all requests to complete
        let results: Vec<_> = futures::future::join_all(handles).await;

        // All requests should succeed
        for result in results {
            assert!(result.is_ok());
            assert!(result.unwrap().is_ok());
        }

        // Due to caching, we might have fewer actual HTTP requests
        // (first request caches, subsequent ones use cache)
        let actual_requests = request_count.load(Ordering::SeqCst);
        assert!(
            actual_requests >= 1,
            "At least one HTTP request should be made"
        );
    }

    /// Test concurrent requests to different packages.
    #[tokio::test]
    async fn test_concurrent_different_packages() {
        let server = setup_npm_mock_server().await;

        // Mount responses for multiple packages
        for i in 0..5 {
            let name = format!("pkg-{i}");
            mount_npm_package(&server, &name, &["1.0.0"]).await;
        }

        let cache = Arc::new(RegistryCache::new());
        let client = Arc::new(NpmClient::with_config(&server.uri(), cache));

        // Request different packages concurrently
        let mut handles = vec![];
        for i in 0..5 {
            let client_clone = client.clone();
            let name = format!("pkg-{i}");
            handles.push(tokio::spawn(async move {
                client_clone.get_package(&name).await
            }));
        }

        let results: Vec<_> = futures::future::join_all(handles).await;

        // All should succeed
        for result in results {
            assert!(result.is_ok());
            assert!(result.unwrap().is_ok());
        }
    }
}
