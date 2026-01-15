# Integration Tests

This directory contains integration tests for the Supply Chain Trust Verifier (SCTV) platform. The tests are organized by component and follow best practices for modular, maintainable test code.

## Test Structure

```
tests/
├── README.md                    # This file
crates/
├── sctv-registries/
│   └── tests/
│       ├── common/
│       │   └── mod.rs           # Shared test utilities
│       └── registry_integration.rs  # Registry client tests
├── sctv-api/
│   └── tests/
│       └── api_integration.rs   # REST API endpoint tests
├── sctv-worker/
│   └── tests/
│       └── workflow_integration.rs  # Scan workflow tests
└── sctv-notifications/
    └── tests/
        └── channel_integration.rs   # Notification channel tests
```

## Running Tests

### Run All Tests

```bash
cargo test --workspace
```

### Run Integration Tests Only

```bash
# Registry client tests
cargo test -p sctv-registries --test registry_integration

# API endpoint tests
cargo test -p sctv-api --test api_integration

# Workflow tests
cargo test -p sctv-worker --test workflow_integration

# Notification tests
cargo test -p sctv-notifications --test channel_integration
```

### Run Tests with Output

```bash
cargo test --workspace -- --nocapture
```

### Run Specific Test Module

```bash
# Run only npm client tests
cargo test -p sctv-registries --test registry_integration npm_client

# Run only authentication tests
cargo test -p sctv-api --test api_integration authentication
```

## Test Categories

### 1. Registry Client Tests (`sctv-registries`)

Tests for package registry clients using mock HTTP servers (wiremock).

**Coverage:**
- Package metadata retrieval
- Version-specific metadata fetching
- Package download functionality
- Error handling (404, rate limiting, server errors)
- Caching behavior
- Scoped package handling (e.g., `@babel/core`)
- Integrity verification (SHA256, SHA512, npm integrity)
- Concurrent request handling

**Key Test Files:**
- `tests/registry_integration.rs` - Main test file
- `tests/common/mod.rs` - Shared utilities and fixtures

### 2. API Endpoint Tests (`sctv-api`)

Tests for REST API endpoints without requiring a database connection.

**Coverage:**
- Health check endpoint
- Authentication (JWT validation, expired tokens)
- Request/response validation
- Error response formatting
- Pagination parameter parsing
- Webhook handling (GitHub, GitLab)
- CORS configuration
- Content-type handling

**Key Test Files:**
- `tests/api_integration.rs` - API endpoint tests

### 3. Scan Workflow Tests (`sctv-worker`)

Tests for the background job processing system.

**Coverage:**
- Job creation and lifecycle management
- Job status transitions (Pending → Running → Completed/Failed)
- Retry logic and failure handling
- Priority-based processing
- Job serialization/deserialization
- Payload and result type handling

**Key Test Files:**
- `tests/workflow_integration.rs` - Workflow tests

### 4. Notification Channel Tests (`sctv-notifications`)

Tests for notification delivery channels using mock HTTP servers.

**Coverage:**
- Slack webhook delivery
- Microsoft Teams adaptive cards
- PagerDuty event API
- Generic webhook (POST/PUT, auth methods)
- Rate limiting handling
- Multi-channel delivery
- Severity-based filtering

**Key Test Files:**
- `tests/channel_integration.rs` - Channel tests

## Test Utilities

### Mock Server Helpers

Located in `crates/sctv-registries/tests/common/mod.rs`:

```rust
// Setup mock npm registry
let server = setup_npm_mock_server().await;
mount_npm_package(&server, "lodash", &["4.17.20", "4.17.21"]).await;

// Create client pointing to mock server
let client = NpmClient::with_config(&server.uri(), cache);
```

### Test Fixtures

The common module provides test fixtures for:

- npm package metadata (`npm_package_metadata`)
- npm abbreviated metadata (`npm_abbreviated_metadata`)
- npm version metadata (`npm_version_metadata`)
- PyPI package metadata (`pypi_package_metadata`)
- Maven POM metadata (`maven_pom_metadata`)

### Authentication Helpers

Located in API tests:

```rust
// Create test JWT token
let token = create_test_token(tenant_id, user_id, "test-secret");

// Use in request
Request::builder()
    .header(AUTHORIZATION, format!("Bearer {}", token))
    .build()
```

## Database Integration Tests

For tests requiring a database, use testcontainers:

```rust
use testcontainers::{clients::Cli, images::postgres::Postgres};

#[tokio::test]
async fn test_with_database() {
    let docker = Cli::default();
    let postgres = docker.run(Postgres::default());

    let connection_string = format!(
        "postgres://postgres:postgres@localhost:{}/postgres",
        postgres.get_host_port_ipv4(5432)
    );

    // Run migrations and tests...
}
```

## Best Practices

### Test Isolation

Each test should be independent and not rely on state from other tests:

```rust
#[tokio::test]
async fn test_something() {
    // Setup - create fresh state
    let server = MockServer::start().await;
    let cache = Arc::new(RegistryCache::new());

    // Test - perform operations
    // ...

    // Verify - check results
    // ...
    // Cleanup happens automatically via Drop
}
```

### Mock Server Expectations

Use `expect()` to verify the number of requests:

```rust
Mock::given(method("POST"))
    .respond_with(ResponseTemplate::new(200))
    .expect(1)  // Exactly one request expected
    .mount(&server)
    .await;
```

### Async Tests

All integration tests should use `#[tokio::test]`:

```rust
#[tokio::test]
async fn test_async_operation() {
    // Test code...
}
```

### Error Handling Tests

Always test error conditions:

```rust
#[tokio::test]
async fn test_not_found_error() {
    mount_npm_not_found(&server, "nonexistent-package").await;

    let result = client.get_package("nonexistent-package").await;

    assert!(matches!(result, Err(RegistryError::PackageNotFound(_))));
}
```

## Adding New Tests

1. **Choose the appropriate crate** based on what you're testing
2. **Add test utilities** to `tests/common/mod.rs` if reusable
3. **Follow existing patterns** for consistency
4. **Document test purpose** with clear test names and comments
5. **Test both success and failure** scenarios

## Continuous Integration

Tests are run automatically in CI on every pull request. Ensure all tests pass locally before submitting:

```bash
cargo test --workspace
cargo clippy --workspace --all-targets
cargo fmt --all -- --check
```
