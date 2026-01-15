# Coding Standards

This document defines the coding standards and best practices for the Supply Chain Trust Verifier (SCTV) project. All contributors should follow these guidelines to maintain code quality and consistency.

## Table of Contents

- [Rust Coding Conventions](#rust-coding-conventions)
- [Error Handling Patterns](#error-handling-patterns)
- [Async/Await Best Practices](#asyncawait-best-practices)
- [Database Query Patterns](#database-query-patterns)
- [API Design Guidelines](#api-design-guidelines)
- [Naming Conventions](#naming-conventions)
- [Documentation Requirements](#documentation-requirements)
- [Clippy Lints](#clippy-lints)
- [Code Formatting (rustfmt)](#code-formatting-rustfmt)
- [Security Considerations](#security-considerations)

## Rust Coding Conventions

### General Guidelines

1. **Follow the Rust API Guidelines**: https://rust-lang.github.io/api-guidelines/

2. **Use idiomatic Rust patterns**:
   ```rust
   // Good: Use pattern matching
   match result {
       Ok(value) => process(value),
       Err(e) => handle_error(e),
   }

   // Avoid: Nested if-let
   if let Ok(value) = result {
       process(value);
   }
   ```

3. **Prefer iterators over loops**:
   ```rust
   // Good
   let names: Vec<String> = users
       .iter()
       .filter(|u| u.is_active)
       .map(|u| u.name.clone())
       .collect();

   // Avoid
   let mut names = Vec::new();
   for user in &users {
       if user.is_active {
           names.push(user.name.clone());
       }
   }
   ```

4. **Avoid unnecessary clones**: Use references when possible
   ```rust
   // Good
   fn process_user(user: &User) { }

   // Avoid (unless ownership transfer is needed)
   fn process_user(user: User) { }
   ```

### Workspace Lint Configuration

The project uses strict workspace lints defined in `Cargo.toml`:

```toml
[workspace.lints.rust]
unsafe_code = "forbid"

[workspace.lints.clippy]
all = "warn"
pedantic = "warn"
nursery = "warn"
```

All crates inherit these lints via:
```toml
[lints]
workspace = true
```

## Error Handling Patterns

### Use thiserror for Domain Errors

Define domain-specific error types using `thiserror`:

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RepositoryError {
    #[error("Entity not found")]
    NotFound,

    #[error("Entity already exists")]
    AlreadyExists,

    #[error("Database error: {0}")]
    Database(String),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Invalid data: {0}")]
    InvalidData(String),
}

pub type RepositoryResult<T> = Result<T, RepositoryError>;
```

### Use anyhow for Application Errors

Use `anyhow` for application-level error handling where detailed error types aren't needed:

```rust
use anyhow::{Context, Result};

async fn load_config() -> Result<Config> {
    let content = tokio::fs::read_to_string("config.toml")
        .await
        .context("Failed to read config file")?;

    toml::from_str(&content)
        .context("Failed to parse config")
}
```

### Error Conversion

Implement `From` traits for error conversion:

```rust
impl From<sqlx::Error> for RepositoryError {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::RowNotFound => Self::NotFound,
            _ => Self::Database(err.to_string()),
        }
    }
}
```

### API Layer Error Handling

Convert domain errors to HTTP responses:

```rust
#[derive(Debug, Error)]
pub enum ApiError {
    #[error("Authentication required")]
    Unauthorized,

    #[error("Resource not found: {0}")]
    NotFound(String),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Internal server error: {0}")]
    Internal(String),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = self.status_code();
        let body = ErrorResponse {
            error: ErrorDetails {
                code: self.error_code(),
                message: self.to_string(),
                details: None,
            },
        };
        (status, Json(body)).into_response()
    }
}
```

### Never Panic in Production Code

```rust
// Good: Return errors
pub fn parse_version(s: &str) -> Result<Version, ParseError> {
    Version::parse(s).map_err(|e| ParseError::InvalidFormat(e.to_string()))
}

// Avoid: Panic (except in tests or truly impossible situations)
pub fn parse_version(s: &str) -> Version {
    Version::parse(s).expect("Invalid version")
}
```

## Async/Await Best Practices

### Use async-trait for Trait Methods

```rust
use async_trait::async_trait;

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn find_by_id(&self, id: UserId) -> RepositoryResult<Option<User>>;
    async fn create(&self, user: &User) -> RepositoryResult<()>;
}
```

### Prefer Tokio Runtime

Use Tokio as the async runtime throughout the project:

```rust
#[tokio::main]
async fn main() -> Result<()> {
    // Application code
    Ok(())
}

// For tests
#[tokio::test]
async fn test_async_function() {
    let result = async_operation().await;
    assert!(result.is_ok());
}
```

### Avoid Blocking in Async Code

```rust
// Good: Use async I/O
let content = tokio::fs::read_to_string("file.txt").await?;

// Avoid: Blocking I/O in async context
let content = std::fs::read_to_string("file.txt")?;

// If blocking is necessary, use spawn_blocking
let result = tokio::task::spawn_blocking(|| {
    expensive_cpu_work()
}).await?;
```

### Handle Concurrent Operations Properly

```rust
// Good: Use join! for concurrent operations
let (users, projects) = tokio::join!(
    user_repo.find_all(),
    project_repo.find_all()
);

// Good: Use select! for racing operations
tokio::select! {
    result = operation_a() => handle_a(result),
    result = operation_b() => handle_b(result),
}
```

### Set Timeouts for External Calls

```rust
use tokio::time::{timeout, Duration};

let result = timeout(
    Duration::from_secs(30),
    external_api_call()
).await??;
```

## Database Query Patterns

### Use SQLx for Type-Safe Queries

```rust
// Good: Compile-time checked query
let user = sqlx::query_as!(
    User,
    r#"
    SELECT id, tenant_id, email, name, role as "role: UserRole"
    FROM users
    WHERE id = $1
    "#,
    user_id.0
)
.fetch_optional(pool)
.await?;
```

### Use Transactions for Multi-Step Operations

```rust
pub async fn create_project_with_dependencies(
    &self,
    project: &Project,
    dependencies: &[Dependency],
) -> RepositoryResult<()> {
    let mut tx = self.pool.begin().await?;

    // Insert project
    sqlx::query!(
        "INSERT INTO projects (id, tenant_id, name) VALUES ($1, $2, $3)",
        project.id.0,
        project.tenant_id.0,
        project.name
    )
    .execute(&mut *tx)
    .await?;

    // Insert dependencies
    for dep in dependencies {
        sqlx::query!(
            "INSERT INTO dependencies (id, project_id, name) VALUES ($1, $2, $3)",
            dep.id.0,
            project.id.0,
            dep.name
        )
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;
    Ok(())
}
```

### Use Proper Indexing and Filtering

```rust
// Add indexes in migrations for frequently queried columns
CREATE INDEX idx_users_tenant_email ON users(tenant_id, email);
CREATE INDEX idx_projects_tenant_id ON projects(tenant_id);
CREATE INDEX idx_alerts_project_status ON alerts(project_id, status);
```

### Batch Operations When Possible

```rust
// Good: Batch insert
pub async fn create_batch(&self, dependencies: &[Dependency]) -> RepositoryResult<()> {
    if dependencies.is_empty() {
        return Ok(());
    }

    let mut query_builder = sqlx::QueryBuilder::new(
        "INSERT INTO dependencies (id, project_id, name, version) "
    );

    query_builder.push_values(dependencies, |mut b, dep| {
        b.push_bind(dep.id.0)
         .push_bind(dep.project_id.0)
         .push_bind(&dep.name)
         .push_bind(&dep.version);
    });

    query_builder.build().execute(&self.pool).await?;
    Ok(())
}
```

### Handle NULL Values Correctly

```rust
// Use Option<T> for nullable fields
pub struct User {
    pub id: UserId,
    pub email: String,
    pub name: Option<String>,  // Can be NULL
    pub api_key_hash: Option<String>,  // Can be NULL
}
```

## API Design Guidelines

### REST API Conventions

1. **Use standard HTTP methods**:
   - GET: Retrieve resources
   - POST: Create resources
   - PUT: Update entire resources
   - PATCH: Partial updates
   - DELETE: Remove resources

2. **Use proper HTTP status codes**:
   ```rust
   // 200 OK - Successful GET
   // 201 Created - Successful POST
   // 204 No Content - Successful DELETE
   // 400 Bad Request - Client error
   // 401 Unauthorized - Authentication required
   // 403 Forbidden - Insufficient permissions
   // 404 Not Found - Resource doesn't exist
   // 409 Conflict - Resource conflict
   // 500 Internal Server Error - Server error
   ```

3. **Use consistent JSON responses**:
   ```rust
   // Success response
   {
     "data": { ... }
   }

   // Error response
   {
     "error": {
       "code": "NOT_FOUND",
       "message": "User not found",
       "details": null
     }
   }
   ```

### GraphQL Schema Conventions

1. **Use proper GraphQL types**:
   ```rust
   #[derive(async_graphql::SimpleObject)]
   pub struct User {
       pub id: Uuid,
       pub email: String,
       pub name: Option<String>,
   }

   #[derive(async_graphql::InputObject)]
   pub struct CreateUserInput {
       pub email: String,
       pub name: Option<String>,
   }
   ```

2. **Implement proper resolvers**:
   ```rust
   #[Object]
   impl QueryRoot {
       async fn user(&self, ctx: &Context<'_>, id: Uuid) -> ApiResult<User> {
           let state = ctx.data::<Arc<AppState>>()?;
           let user = state.user_repo
               .find_by_id(UserId(id))
               .await?
               .ok_or_else(|| ApiError::NotFound("User not found".to_string()))?;
           Ok(user)
       }
   }
   ```

### Authentication and Authorization

```rust
// Extract authenticated user from request
pub async fn authenticated_user(
    TypedHeader(auth): TypedHeader<Authorization<Bearer>>,
    State(state): State<Arc<AppState>>,
) -> ApiResult<User> {
    let token = auth.token();
    let claims = verify_jwt(token, &state.jwt_secret)?;

    let user = state.user_repo
        .find_by_id(UserId(claims.user_id))
        .await?
        .ok_or(ApiError::Unauthorized)?;

    Ok(user)
}

// Use as extractor in handlers
pub async fn get_profile(
    user: User,  // Automatically authenticated
) -> ApiResult<Json<User>> {
    Ok(Json(user))
}
```

## Naming Conventions

### Crate Names

- Use `sctv-` prefix: `sctv-core`, `sctv-api`, `sctv-db`
- Use kebab-case: `sctv-notifications`, not `sctv_notifications`

### Module Names

- Use snake_case: `user_repository`, `error_handling`
- Keep module names short and descriptive

### Type Names

- Use PascalCase: `User`, `ProjectId`, `RepositoryError`
- Suffix error types with `Error`: `ApiError`, `ParseError`
- Suffix result types with `Result`: `ApiResult`, `RepositoryResult`

### Function Names

- Use snake_case: `find_by_id`, `create_user`, `is_valid`
- Use verbs for actions: `create`, `update`, `delete`, `find`
- Use `is_` prefix for boolean functions: `is_valid`, `is_active`

### Variable Names

- Use snake_case: `user_id`, `project_name`, `api_key`
- Be descriptive but concise
- Avoid single-letter names except in closures

### Constants

- Use SCREAMING_SNAKE_CASE: `MAX_CONNECTIONS`, `DEFAULT_TIMEOUT`

```rust
const MAX_BATCH_SIZE: usize = 1000;
const DEFAULT_PAGE_SIZE: u32 = 50;
const JWT_EXPIRATION_HOURS: i64 = 24;
```

### Database Conventions

- Table names: plural snake_case: `users`, `projects`, `audit_logs`
- Column names: snake_case: `user_id`, `created_at`, `is_active`
- Foreign keys: `{table}_id`: `tenant_id`, `project_id`

## Documentation Requirements

### Module Documentation

Every module should have top-level documentation:

```rust
//! User repository implementation.
//!
//! This module provides the PostgreSQL implementation of the `UserRepository` trait.
//! It handles all user-related database operations including authentication,
//! authorization, and profile management.

use crate::*;
```

### Public API Documentation

Document all public items:

```rust
/// Represents a unique identifier for a package.
///
/// Package IDs are UUIDv4 values generated when a package is first cached
/// from a registry. They remain stable across package updates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PackageId(pub Uuid);

impl PackageId {
    /// Creates a new random package ID.
    ///
    /// # Examples
    ///
    /// ```
    /// use sctv_core::PackageId;
    ///
    /// let id = PackageId::new();
    /// assert_ne!(id, PackageId::new());
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}
```

### Function Documentation

Document public functions with examples:

```rust
/// Normalizes a package name for comparison.
///
/// This function handles different naming conventions across ecosystems
/// by converting to lowercase and replacing special characters with hyphens.
///
/// # Arguments
///
/// * `name` - The package name to normalize
///
/// # Returns
///
/// The normalized package name as a `String`.
///
/// # Examples
///
/// ```
/// use sctv_core::normalize_package_name;
///
/// assert_eq!(normalize_package_name("My_Package"), "my-package");
/// assert_eq!(normalize_package_name("@scope/package"), "scope-package");
/// ```
#[must_use]
pub fn normalize_package_name(name: &str) -> String {
    name.to_lowercase()
        .replace('_', "-")
        .replace('.', "-")
        .trim_start_matches('@')
        .replace('/', "-")
}
```

### Use #[must_use] Annotation

Mark functions that should not have their return value ignored:

```rust
#[must_use]
pub fn new() -> Self { ... }

#[must_use]
pub fn is_stale(&self, max_age: Duration) -> bool { ... }
```

## Clippy Lints

### Run Clippy Regularly

```bash
# Run clippy on all workspace crates
cargo clippy --workspace -- -D warnings

# Run clippy with all features
cargo clippy --workspace --all-features -- -D warnings

# Run clippy on specific crate
cargo clippy -p sctv-core -- -D warnings
```

### Common Clippy Patterns to Follow

```rust
// Good: Use if let for single pattern matching
if let Some(value) = option {
    process(value);
}

// Good: Use map/and_then for Option chaining
let result = maybe_value
    .map(|v| v * 2)
    .unwrap_or_default();

// Good: Use ? operator for error propagation
let user = user_repo.find_by_id(id).await?;

// Good: Avoid unnecessary borrows
fn process(s: &str) { }  // Not &String

// Good: Use .is_empty() instead of .len() == 0
if users.is_empty() { }
```

### Allow Specific Lints When Necessary

```rust
// Allow specific lint for one item
#[allow(clippy::too_many_arguments)]
pub fn complex_function(...) { }

// Allow for entire module (use sparingly)
#![allow(clippy::module_name_repetitions)]
```

## Code Formatting (rustfmt)

### Automatic Formatting

Format code before committing:

```bash
# Format all code
cargo fmt --all

# Check formatting without modifying
cargo fmt --all -- --check
```

### Formatting Configuration

The project uses default rustfmt settings. Create `.rustfmt.toml` if custom settings are needed:

```toml
max_width = 100
hard_tabs = false
tab_spaces = 4
newline_style = "Unix"
use_small_heuristics = "Default"
reorder_imports = true
reorder_modules = true
```

### Pre-commit Hook

Set up a git pre-commit hook:

```bash
#!/bin/sh
# .git/hooks/pre-commit

set -e

echo "Running cargo fmt..."
cargo fmt --all -- --check

echo "Running cargo clippy..."
cargo clippy --workspace -- -D warnings

echo "All checks passed!"
```

## Security Considerations

### Input Validation

Always validate and sanitize user input:

```rust
// Good: Validate email format
pub fn validate_email(email: &str) -> Result<(), ValidationError> {
    if !email.contains('@') || email.len() < 3 {
        return Err(ValidationError::InvalidEmail);
    }
    Ok(())
}

// Good: Use type system for validation
pub struct ValidatedEmail(String);

impl ValidatedEmail {
    pub fn new(email: String) -> Result<Self, ValidationError> {
        validate_email(&email)?;
        Ok(Self(email))
    }
}
```

### Avoid SQL Injection

Always use parameterized queries:

```rust
// Good: Parameterized query
sqlx::query!("SELECT * FROM users WHERE email = $1", email)
    .fetch_one(pool)
    .await?;

// NEVER: String concatenation (SQL injection risk!)
// let query = format!("SELECT * FROM users WHERE email = '{}'", email);
```

### Secrets Management

Never hardcode secrets:

```rust
// Good: Load from environment
let jwt_secret = env::var("SCTV_JWT_SECRET")
    .context("SCTV_JWT_SECRET must be set")?;

// Avoid: Hardcoded secret
// const JWT_SECRET: &str = "hardcoded-secret";
```

### Authentication and Authorization

```rust
// Always verify permissions
pub async fn delete_project(
    user: User,
    project_id: ProjectId,
) -> ApiResult<()> {
    let project = project_repo.find_by_id(project_id).await?
        .ok_or(ApiError::NotFound("Project not found".to_string()))?;

    // Verify user has access to this project's tenant
    if project.tenant_id != user.tenant_id {
        return Err(ApiError::Forbidden);
    }

    // Verify user has required role
    if !user.can_delete_project() {
        return Err(ApiError::Forbidden);
    }

    project_repo.delete(project_id).await?;
    Ok(())
}
```

### Sensitive Data Handling

```rust
// Good: Hash passwords and API keys
use sha2::{Sha256, Digest};

pub fn hash_api_key(api_key: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(api_key.as_bytes());
    format!("{:x}", hasher.finalize())
}

// Never log sensitive data
tracing::debug!("User authenticated: {}", user.email);  // OK
// tracing::debug!("API key: {}", api_key);  // NEVER!
```

### Rate Limiting

Implement rate limiting for public endpoints:

```rust
// Use tower middleware for rate limiting
use tower_governor::{GovernorLayer, governor::GovernorConfigBuilder};

let governor_conf = GovernorConfigBuilder::default()
    .per_second(10)
    .burst_size(20)
    .finish()
    .unwrap();

let app = Router::new()
    .route("/api/v1/public", get(public_handler))
    .layer(GovernorLayer { config: Arc::new(governor_conf) });
```

## Code Review Checklist

Before submitting code for review, ensure:

- [ ] Code follows all naming conventions
- [ ] All public APIs are documented
- [ ] Tests are included for new functionality
- [ ] No clippy warnings
- [ ] Code is formatted with rustfmt
- [ ] No hardcoded secrets or sensitive data
- [ ] Error handling is proper and consistent
- [ ] No unnecessary clones or allocations
- [ ] Async code doesn't block
- [ ] Database queries use parameterized statements
- [ ] Input validation is implemented
- [ ] Authorization checks are in place

## Resources

- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [Async Rust Book](https://rust-lang.github.io/async-book/)
- [SQLx Documentation](https://docs.rs/sqlx/)
- [Axum Documentation](https://docs.rs/axum/)
- [Clippy Lints](https://rust-lang.github.io/rust-clippy/)
