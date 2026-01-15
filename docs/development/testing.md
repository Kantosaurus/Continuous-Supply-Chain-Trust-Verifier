# Testing Guide

This guide covers testing practices and patterns for the Supply Chain Trust Verifier (SCTV) project.

## Table of Contents

- [Test Organization](#test-organization)
- [Unit Testing Patterns](#unit-testing-patterns)
- [Integration Testing with Testcontainers](#integration-testing-with-testcontainers)
- [API Endpoint Testing](#api-endpoint-testing)
- [Database Testing](#database-testing)
- [Mocking External Services](#mocking-external-services)
- [Test Coverage Requirements](#test-coverage-requirements)
- [Running Specific Tests](#running-specific-tests)
- [CI Test Pipeline](#ci-test-pipeline)
- [Performance Testing](#performance-testing)

## Test Organization

### Project Structure

Tests are organized in three ways:

1. **Unit tests** - In the same file as the code (using `#[cfg(test)]` modules)
2. **Integration tests** - In the `tests/` directory of each crate
3. **End-to-end tests** - In the workspace-level `tests/` directory

```
crates/
├── sctv-core/
│   ├── src/
│   │   ├── domain/
│   │   │   └── package.rs  # Contains #[cfg(test)] mod tests
│   │   └── lib.rs
│   └── tests/              # Integration tests for core (if any)
├── sctv-db/
│   ├── src/
│   │   └── repositories/
│   │       └── user_repo.rs
│   └── tests/              # Database integration tests
│       ├── common/
│       │   └── mod.rs      # Test utilities
│       ├── user_repo_tests.rs
│       ├── job_repo_tests.rs
│       └── sbom_repo_tests.rs
└── sctv-api/
    ├── src/
    │   └── rest/
    │       └── handlers.rs
    └── tests/              # API integration tests
        └── api_tests.rs
tests/                      # Workspace-level E2E tests
└── e2e_tests.rs
```

### Test Module Naming

- Unit test modules: `mod tests` at the end of source files
- Integration test files: `*_tests.rs` or `test_*.rs`
- Common test utilities: `tests/common/mod.rs`

## Unit Testing Patterns

### Basic Unit Tests

Unit tests go in the same file as the code they test:

```rust
/// Normalizes a package name for comparison.
#[must_use]
pub fn normalize_package_name(name: &str) -> String {
    name.to_lowercase()
        .replace('_', "-")
        .replace('.', "-")
        .trim_start_matches('@')
        .replace('/', "-")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_package_name() {
        assert_eq!(normalize_package_name("My_Package"), "my-package");
        assert_eq!(normalize_package_name("my.package"), "my-package");
        assert_eq!(normalize_package_name("@scope/package"), "scope-package");
        assert_eq!(normalize_package_name("UPPERCASE"), "uppercase");
    }

    #[test]
    fn test_normalize_empty_string() {
        assert_eq!(normalize_package_name(""), "");
    }
}
```

### Testing Domain Logic

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    #[test]
    fn test_package_creation() {
        let package = Package::new(
            PackageEcosystem::Npm,
            "express".to_string()
        );

        assert_eq!(package.ecosystem, PackageEcosystem::Npm);
        assert_eq!(package.name, "express");
        assert_eq!(package.normalized_name, "express");
        assert_eq!(package.is_popular, false);
    }

    #[test]
    fn test_package_is_stale() {
        let mut package = Package::new(
            PackageEcosystem::Npm,
            "express".to_string()
        );

        // Fresh package
        assert!(!package.is_stale(Duration::hours(24)));

        // Old package
        package.cached_at = Utc::now() - Duration::days(7);
        assert!(package.is_stale(Duration::days(1)));
    }

    #[test]
    fn test_package_version_is_new() {
        let mut version = PackageVersion::new(
            PackageId::new(),
            Version::parse("1.0.0").unwrap()
        );

        version.published_at = Some(Utc::now() - Duration::days(5));
        assert!(version.is_new(7));  // Within 7 days
        assert!(!version.is_new(3)); // Not within 3 days
    }
}
```

### Async Unit Tests

Use `#[tokio::test]` for async tests:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_async_operation() {
        let result = async_function().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_timeout() {
        use tokio::time::{timeout, Duration};

        let result = timeout(
            Duration::from_millis(100),
            slow_operation()
        ).await;

        assert!(result.is_err()); // Should timeout
    }
}
```

## Integration Testing with Testcontainers

### Test Database Setup

The project uses testcontainers for database integration tests. See `crates/sctv-db/tests/common/mod.rs`:

```rust
use sqlx::PgPool;
use std::sync::Once;
use testcontainers::{runners::AsyncRunner, ContainerAsync, ImageExt};
use testcontainers::core::IntoContainerPort;

static INIT: Once = Once::new();

/// Initialize tracing for tests (only once).
pub fn init_tracing() {
    INIT.call_once(|| {
        tracing_subscriber::fmt()
            .with_env_filter("error")
            .with_test_writer()
            .try_init()
            .ok();
    });
}

/// PostgreSQL container image for testing.
#[derive(Debug, Clone)]
pub struct PostgresImage {
    env_vars: Vec<(String, String)>,
}

impl Default for PostgresImage {
    fn default() -> Self {
        Self {
            env_vars: vec![
                ("POSTGRES_USER".to_string(), "test".to_string()),
                ("POSTGRES_PASSWORD".to_string(), "test".to_string()),
                ("POSTGRES_DB".to_string(), "sctv_test".to_string()),
            ],
        }
    }
}

/// Test database context holding the container and pool.
pub struct TestDb {
    #[allow(dead_code)]
    container: ContainerAsync<PostgresImage>,
    pub pool: PgPool,
}

impl TestDb {
    /// Creates a new test database with migrations applied.
    pub async fn new() -> Self {
        init_tracing();

        let container = PostgresImage::default()
            .with_mapped_port(5432, 5432.tcp())
            .start()
            .await
            .expect("Failed to start PostgreSQL container");

        let host_port = container
            .get_host_port_ipv4(5432)
            .await
            .expect("Failed to get host port");

        let database_url = format!(
            "postgres://test:test@127.0.0.1:{}/sctv_test",
            host_port
        );

        // Wait for PostgreSQL to be ready
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;

        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await
            .expect("Failed to connect to test database");

        // Run migrations
        sqlx::migrate!("../../migrations")
            .run(&pool)
            .await
            .expect("Failed to run migrations");

        Self { container, pool }
    }
}
```

### Writing Database Integration Tests

```rust
mod common;

use common::{create_test_tenant, create_test_user, TestDb};
use sctv_core::traits::{TenantRepository, UserRepository};
use sctv_db::{PgTenantRepository, PgUserRepository};

#[tokio::test]
async fn test_create_and_find_user_by_id() {
    let db = TestDb::new().await;
    let tenant_repo = PgTenantRepository::new(db.pool.clone());
    let user_repo = PgUserRepository::new(db.pool.clone());

    // Create a tenant first
    let tenant = create_test_tenant();
    tenant_repo.create(&tenant).await.expect("Failed to create tenant");

    // Create a user
    let user = create_test_user(tenant.id);
    user_repo.create(&user).await.expect("Failed to create user");

    // Find by ID
    let found = user_repo
        .find_by_id(user.id)
        .await
        .expect("Failed to find user")
        .expect("User not found");

    assert_eq!(found.id, user.id);
    assert_eq!(found.email, user.email);
    assert_eq!(found.tenant_id, tenant.id);
}

#[tokio::test]
async fn test_find_user_by_email_not_found() {
    let db = TestDb::new().await;
    let tenant_repo = PgTenantRepository::new(db.pool.clone());
    let user_repo = PgUserRepository::new(db.pool.clone());

    let tenant = create_test_tenant();
    tenant_repo.create(&tenant).await.expect("Failed to create tenant");

    let result = user_repo
        .find_by_email(tenant.id, "nonexistent@example.com")
        .await
        .expect("Failed to query");

    assert!(result.is_none());
}

#[tokio::test]
async fn test_duplicate_email_same_tenant_fails() {
    let db = TestDb::new().await;
    let tenant_repo = PgTenantRepository::new(db.pool.clone());
    let user_repo = PgUserRepository::new(db.pool.clone());

    let tenant = create_test_tenant();
    tenant_repo.create(&tenant).await.expect("Failed to create tenant");

    let user1 = create_test_user(tenant.id);
    user_repo.create(&user1).await.expect("Failed to create user1");

    // Try to create another user with same email
    let mut user2 = create_test_user(tenant.id);
    user2.email = user1.email.clone();

    let result = user_repo.create(&user2).await;
    assert!(result.is_err());
}
```

### Test Data Builders

Create helper functions for test data:

```rust
/// Creates a test tenant for use in tests.
pub fn create_test_tenant() -> Tenant {
    Tenant::new(
        format!("Test Tenant {}", uuid::Uuid::new_v4()),
        format!("test-{}", uuid::Uuid::new_v4().to_string().split('-').next().unwrap()),
    )
}

/// Creates a test user for use in tests.
pub fn create_test_user(tenant_id: TenantId) -> User {
    User::new(
        tenant_id,
        format!("test-{}@example.com", uuid::Uuid::new_v4()),
    )
}

/// Creates a test project for use in tests.
pub fn create_test_project(tenant_id: TenantId) -> Project {
    Project::new(
        tenant_id,
        format!("test-project-{}", uuid::Uuid::new_v4()),
    )
}
```

## API Endpoint Testing

### Testing Axum Handlers

```rust
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use tower::ServiceExt; // for `oneshot`
use serde_json::json;

#[tokio::test]
async fn test_health_endpoint() {
    let app = create_test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .unwrap()
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["status"], "healthy");
}

#[tokio::test]
async fn test_create_user() {
    let app = create_test_app().await;

    let payload = json!({
        "email": "newuser@example.com",
        "name": "New User"
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/users")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&payload).unwrap()))
                .unwrap()
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
}

#[tokio::test]
async fn test_unauthorized_access() {
    let app = create_test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/projects")
                .body(Body::empty())
                .unwrap()
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}
```

### Testing with Authentication

```rust
#[tokio::test]
async fn test_authenticated_endpoint() {
    let app = create_test_app().await;

    // Create test user and generate JWT
    let user = create_test_user_in_db().await;
    let token = generate_test_jwt(&user);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/profile")
                .header("authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap()
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}
```

### Testing GraphQL Endpoints

```rust
#[tokio::test]
async fn test_graphql_query() {
    let app = create_test_app().await;

    let query = json!({
        "query": "{ users { id email } }"
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/graphql")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&query).unwrap()))
                .unwrap()
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert!(json["data"]["users"].is_array());
}
```

## Database Testing

### Testing Transactions

```rust
#[tokio::test]
async fn test_transaction_rollback_on_error() {
    let db = TestDb::new().await;
    let repo = PgProjectRepository::new(db.pool.clone());

    let tenant = create_test_tenant();
    // ... create tenant

    let project = create_test_project(tenant.id);

    // This should fail and rollback
    let result = repo.create_with_invalid_data(&project).await;
    assert!(result.is_err());

    // Verify nothing was created
    let found = repo.find_by_id(project.id).await.unwrap();
    assert!(found.is_none());
}
```

### Testing Batch Operations

```rust
#[tokio::test]
async fn test_batch_create_dependencies() {
    let db = TestDb::new().await;
    let dep_repo = PgDependencyRepository::new(db.pool.clone());

    let dependencies = vec![
        create_test_dependency(project.id, "express", "4.18.0"),
        create_test_dependency(project.id, "lodash", "4.17.21"),
        create_test_dependency(project.id, "axios", "1.6.0"),
    ];

    dep_repo.create_batch(&dependencies).await.expect("Failed to batch create");

    let found = dep_repo.find_by_project(project.id).await.unwrap();
    assert_eq!(found.len(), 3);
}
```

### Testing Database Constraints

```rust
#[tokio::test]
async fn test_unique_constraint_violation() {
    let db = TestDb::new().await;
    let tenant_repo = PgTenantRepository::new(db.pool.clone());

    let tenant1 = Tenant::new("Test Tenant".to_string(), "test-slug".to_string());
    tenant_repo.create(&tenant1).await.expect("Failed to create first tenant");

    // Try to create another tenant with same slug
    let tenant2 = Tenant::new("Another Tenant".to_string(), "test-slug".to_string());
    let result = tenant_repo.create(&tenant2).await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_foreign_key_constraint() {
    let db = TestDb::new().await;
    let user_repo = PgUserRepository::new(db.pool.clone());

    // Try to create user with non-existent tenant
    let fake_tenant_id = TenantId(Uuid::new_v4());
    let user = User::new(fake_tenant_id, "test@example.com".to_string());

    let result = user_repo.create(&user).await;
    assert!(result.is_err());
}
```

## Mocking External Services

### Using wiremock for HTTP Mocking

```rust
use wiremock::{MockServer, Mock, ResponseTemplate};
use wiremock::matchers::{method, path};

#[tokio::test]
async fn test_external_registry_call() {
    // Start mock server
    let mock_server = MockServer::start().await;

    // Set up mock response
    Mock::given(method("GET"))
        .and(path("/api/package/express"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "name": "express",
            "version": "4.18.2",
            "description": "Fast web framework"
        })))
        .mount(&mock_server)
        .await;

    // Use mock server URL in client
    let client = create_registry_client(&mock_server.uri());
    let package = client.get_package("express").await.unwrap();

    assert_eq!(package.name, "express");
    assert_eq!(package.version, "4.18.2");
}
```

### Using mockall for Trait Mocking

Add `mockall` to dev dependencies:

```rust
use mockall::{automock, predicate::*};

#[automock]
#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn find_by_id(&self, id: UserId) -> RepositoryResult<Option<User>>;
    async fn create(&self, user: &User) -> RepositoryResult<()>;
}

#[tokio::test]
async fn test_with_mock_repository() {
    let mut mock_repo = MockUserRepository::new();

    // Set expectations
    mock_repo
        .expect_find_by_id()
        .with(eq(UserId(Uuid::nil())))
        .times(1)
        .returning(|_| Ok(None));

    // Use mock in test
    let result = mock_repo.find_by_id(UserId(Uuid::nil())).await;
    assert!(result.unwrap().is_none());
}
```

## Test Coverage Requirements

### Running Coverage Reports

```bash
# Install cargo-llvm-cov
cargo install cargo-llvm-cov

# Generate coverage report
cargo llvm-cov --workspace

# Generate HTML report
cargo llvm-cov --workspace --html
open target/llvm-cov/html/index.html

# Generate lcov format (for CI)
cargo llvm-cov --workspace --lcov --output-path lcov.info
```

### Coverage Targets

- **Core domain logic** (`sctv-core`): 90%+ coverage
- **Database layer** (`sctv-db`): 85%+ coverage
- **API handlers** (`sctv-api`): 80%+ coverage
- **Business services**: 85%+ coverage

### Viewing Coverage in CI

Coverage is automatically uploaded to Codecov on each CI run. View reports at:
- https://codecov.io/gh/example/supply-chain-trust-verifier

## Running Specific Tests

### Run All Tests

```bash
# Run all tests in workspace
cargo test --workspace

# Run with output
cargo test --workspace -- --nocapture

# Run with specific log level
RUST_LOG=debug cargo test --workspace
```

### Run Specific Test Files

```bash
# Run tests in specific crate
cargo test -p sctv-core

# Run specific test file
cargo test --test user_repo_tests

# Run specific test function
cargo test test_create_user

# Run tests matching pattern
cargo test user_
```

### Run Tests with Features

```bash
# Run tests with all features
cargo test --workspace --all-features

# Run tests with specific feature
cargo test -p sctv-core --features graphql
```

### Run Ignored Tests

```bash
# Mark slow tests as ignored
#[test]
#[ignore]
fn slow_integration_test() { }

# Run only ignored tests
cargo test -- --ignored

# Run all tests including ignored
cargo test -- --include-ignored
```

### Parallel vs Sequential Execution

```bash
# Run tests sequentially (for database tests)
cargo test -- --test-threads=1

# Run with specific number of threads
cargo test -- --test-threads=4
```

## CI Test Pipeline

### GitHub Actions Configuration

The project uses GitHub Actions for CI. See `.github/workflows/ci.yml`:

```yaml
test:
  name: Test
  runs-on: ubuntu-latest
  services:
    postgres:
      image: postgres:16-alpine
      env:
        POSTGRES_USER: sctv_test
        POSTGRES_PASSWORD: sctv_test
        POSTGRES_DB: sctv_test
      options: >-
        --health-cmd pg_isready
        --health-interval 10s
        --health-timeout 5s
        --health-retries 5
      ports:
        - 5432:5432

  steps:
    - name: Checkout repository
      uses: actions/checkout@v4

    - name: Install Rust toolchain
      uses: dtolnay/rust-action@stable

    - name: Cache cargo registry
      uses: actions/cache@v4
      with:
        path: |
          ~/.cargo/registry
          ~/.cargo/git
          target
        key: ${{ runner.os }}-cargo-test-${{ hashFiles('**/Cargo.lock') }}

    - name: Install sqlx-cli
      run: cargo install sqlx-cli --no-default-features --features postgres

    - name: Run database migrations
      env:
        DATABASE_URL: postgres://sctv_test:sctv_test@localhost:5432/sctv_test
      run: sqlx migrate run

    - name: Run tests
      env:
        DATABASE_URL: postgres://sctv_test:sctv_test@localhost:5432/sctv_test
        RUST_LOG: debug
      run: cargo test --workspace --all-features
```

### Local CI Simulation

Run tests the same way CI does:

```bash
# Start PostgreSQL
docker run -d --name test-postgres \
  -e POSTGRES_USER=sctv_test \
  -e POSTGRES_PASSWORD=sctv_test \
  -e POSTGRES_DB=sctv_test \
  -p 5432:5432 \
  postgres:16-alpine

# Wait for PostgreSQL
sleep 5

# Run migrations
export DATABASE_URL=postgres://sctv_test:sctv_test@localhost:5432/sctv_test
sqlx migrate run

# Run tests
cargo test --workspace --all-features

# Cleanup
docker rm -f test-postgres
```

## Performance Testing

### Benchmarking with criterion

Add to `dev-dependencies`:

```toml
[dev-dependencies]
criterion = { version = "0.5", features = ["async_tokio"] }
```

Create benchmarks:

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn normalize_benchmark(c: &mut Criterion) {
    c.bench_function("normalize_package_name", |b| {
        b.iter(|| normalize_package_name(black_box("@scope/My_Package.Name")))
    });
}

criterion_group!(benches, normalize_benchmark);
criterion_main!(benches);
```

Run benchmarks:

```bash
cargo bench
```

### Load Testing

Use tools like `wrk` or `k6` for load testing:

```bash
# Install wrk
# Load test the API
wrk -t12 -c400 -d30s http://localhost:3000/health

# Using k6
k6 run load-test.js
```

### Database Performance Testing

```rust
#[tokio::test]
#[ignore] // Run manually for performance testing
async fn test_batch_insert_performance() {
    let db = TestDb::new().await;
    let repo = PgDependencyRepository::new(db.pool.clone());

    let start = std::time::Instant::now();

    // Create 1000 dependencies
    let dependencies: Vec<_> = (0..1000)
        .map(|i| create_test_dependency(project.id, &format!("pkg-{}", i), "1.0.0"))
        .collect();

    repo.create_batch(&dependencies).await.unwrap();

    let duration = start.elapsed();
    println!("Batch insert of 1000 records took: {:?}", duration);

    // Assert reasonable performance
    assert!(duration.as_secs() < 5, "Batch insert took too long");
}
```

## Test Best Practices

### Do's

- Write tests for all public APIs
- Test both success and error cases
- Use descriptive test names
- Keep tests independent and idempotent
- Clean up test data (containers handle this automatically)
- Use test data builders for complex objects
- Test edge cases and boundary conditions

### Don'ts

- Don't test implementation details
- Don't share mutable state between tests
- Don't rely on test execution order
- Don't skip error handling in tests
- Don't use `unwrap()` without a reason
- Don't write flaky tests

### Example: Good vs Bad Tests

```rust
// Good: Clear name, tests specific behavior
#[test]
fn test_normalize_removes_scope_prefix() {
    assert_eq!(normalize_package_name("@scope/package"), "scope-package");
}

// Bad: Unclear what it tests
#[test]
fn test_function() {
    let result = normalize_package_name("something");
    assert!(!result.is_empty());
}

// Good: Tests error case
#[tokio::test]
async fn test_find_user_returns_error_for_invalid_id() {
    let db = TestDb::new().await;
    let repo = PgUserRepository::new(db.pool.clone());

    let result = repo.find_by_id(UserId(Uuid::nil())).await;
    assert!(result.is_ok());
    assert!(result.unwrap().is_none());
}

// Bad: No error handling
#[tokio::test]
async fn test_user() {
    let db = TestDb::new().await;
    let repo = PgUserRepository::new(db.pool.clone());
    let user = repo.find_by_id(UserId(Uuid::nil())).await.unwrap();
    // What if this panics?
}
```

## Debugging Test Failures

### Enable Test Logging

```bash
# Show all logs during tests
RUST_LOG=debug cargo test -- --nocapture

# Show logs for specific module
RUST_LOG=sctv_db=trace cargo test
```

### Run Single Test

```bash
# Run specific test
cargo test test_create_user -- --nocapture

# Run with exact match
cargo test --test user_repo_tests -- test_create_user --exact
```

### Use Test Output

```rust
#[test]
fn test_with_debug_output() {
    let value = calculate_something();
    eprintln!("Debug: value = {:?}", value);  // Shows in test output
    assert_eq!(value, expected);
}
```

## Resources

- [Rust Testing Documentation](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [Tokio Testing Guide](https://tokio.rs/tokio/topics/testing)
- [SQLx Testing Examples](https://github.com/launchbadge/sqlx/tree/main/tests)
- [Testcontainers Rust](https://docs.rs/testcontainers/)
- [Criterion Benchmarking](https://bheisler.github.io/criterion.rs/book/)
