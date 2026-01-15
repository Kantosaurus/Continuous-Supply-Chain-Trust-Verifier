# Contributing Guide

**Version:** 0.1.0

Thank you for your interest in contributing to Supply Chain Trust Verifier (SCTV)! This guide will help you get started.

---

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Setup](#development-setup)
- [Project Structure](#project-structure)
- [Development Workflow](#development-workflow)
- [Coding Standards](#coding-standards)
- [Testing](#testing)
- [Submitting Changes](#submitting-changes)
- [Areas for Contribution](#areas-for-contribution)

---

## Code of Conduct

We are committed to providing a welcoming and inspiring community for all. Please read and follow our [Code of Conduct](CODE_OF_CONDUCT.md).

**Key principles:**
- Be respectful and inclusive
- Welcome newcomers
- Focus on constructive feedback
- Assume good faith

---

## Getting Started

### Prerequisites

**Required:**
- Rust 1.75+ with cargo
- PostgreSQL 14+
- Git

**Optional:**
- Docker and Docker Compose
- Node.js 18+ (for dashboard development)
- Make

### Fork and Clone

```bash
# Fork the repository on GitHub
# Then clone your fork
git clone https://github.com/YOUR-USERNAME/supply-chain-trust-verifier.git
cd supply-chain-trust-verifier

# Add upstream remote
git remote add upstream https://github.com/example/supply-chain-trust-verifier.git
```

---

## Development Setup

### Option 1: Docker Compose (Recommended)

```bash
# Start development environment
docker-compose -f docker-compose.dev.yml up -d

# Run migrations
docker-compose exec api sctv-api migrate

# View logs
docker-compose logs -f
```

### Option 2: Native Development

**1. Install Dependencies:**

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install PostgreSQL
# Ubuntu/Debian:
sudo apt install postgresql-14

# macOS:
brew install postgresql@14
```

**2. Setup Database:**

```bash
# Create database
createdb sctv_dev

# Set environment variables
export DATABASE_URL="postgresql://postgres:postgres@localhost/sctv_dev"

# Run migrations
cargo run --bin sctv-api -- migrate
```

**3. Build and Run:**

```bash
# Build all crates
cargo build

# Run API server
cargo run --bin sctv-api

# Run worker (in separate terminal)
cargo run --bin sctv-worker

# Run CLI
cargo run --bin sctv-cli -- --help
```

**4. Run Tests:**

```bash
# Run all tests
cargo test

# Run specific crate tests
cargo test -p sctv-core

# Run with output
cargo test -- --nocapture
```

---

## Project Structure

```
supply-chain-trust-verifier/
├── crates/                      # Rust workspace crates
│   ├── sctv-core/              # Core domain models
│   ├── sctv-api/               # REST/GraphQL API
│   ├── sctv-db/                # Database layer
│   ├── sctv-detectors/         # Threat detectors
│   ├── sctv-registries/        # Package registry clients
│   ├── sctv-sbom/              # SBOM generators
│   ├── sctv-worker/            # Background workers
│   ├── sctv-notifications/     # Alert notifications
│   ├── sctv-ci/                # CI/CD integrations
│   ├── sctv-cli/               # Command-line tool
│   └── sctv-dashboard/         # Web UI (Leptos)
├── migrations/                  # Database migrations
├── docs/                        # Documentation
├── tests/                       # Integration tests
├── examples/                    # Example code
├── scripts/                     # Build/deployment scripts
├── Cargo.toml                   # Workspace manifest
├── Cargo.lock                   # Dependency lock file
└── README.md
```

### Crate Dependencies

```
sctv-core (domain models, no dependencies on other crates)
    ↑
    ├── sctv-db (implements repositories)
    ├── sctv-detectors (uses domain models)
    ├── sctv-registries (uses domain models)
    ├── sctv-sbom (uses domain models)
    └── sctv-notifications (uses domain models)
        ↑
        ├── sctv-worker (orchestrates services)
        └── sctv-api (exposes services via HTTP)
            ↑
            ├── sctv-cli (calls API)
            └── sctv-dashboard (UI for API)
```

---

## Development Workflow

### 1. Create a Branch

```bash
# Update your fork
git fetch upstream
git checkout main
git merge upstream/main

# Create feature branch
git checkout -b feature/your-feature-name

# Or bug fix branch
git checkout -b fix/bug-description
```

### 2. Make Changes

Follow our [Code Standards](code-standards.md).

**Key points:**
- Write clear, self-documenting code
- Add tests for new functionality
- Update documentation
- Follow Rust idioms and best practices

### 3. Test Your Changes

```bash
# Run tests
cargo test

# Check formatting
cargo fmt --check

# Run clippy lints
cargo clippy -- -D warnings

# Check for security vulnerabilities
cargo audit
```

### 4. Commit Your Changes

We follow [Conventional Commits](https://www.conventionalcommits.org/):

```bash
# Format: <type>(<scope>): <subject>

# Examples:
git commit -m "feat(detectors): add npm typosquatting detection"
git commit -m "fix(api): correct pagination cursor encoding"
git commit -m "docs(readme): update installation instructions"
git commit -m "test(core): add policy validation tests"
git commit -m "refactor(db): optimize dependency query"
```

**Commit Types:**
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `test`: Adding or updating tests
- `refactor`: Code refactoring
- `perf`: Performance improvement
- `chore`: Build/tooling changes
- `style`: Code style changes (formatting, etc.)

### 5. Push and Create Pull Request

```bash
# Push to your fork
git push origin feature/your-feature-name

# Create pull request on GitHub
# - Provide clear description
# - Link related issues
# - Add screenshots for UI changes
```

---

## Coding Standards

### Rust Style

**Follow Rust conventions:**

```rust
// Use explicit error types
pub type Result<T> = std::result::Result<T, Error>;

// Document public APIs
/// Scans a project for supply chain threats.
///
/// # Arguments
///
/// * `project_id` - The project to scan
/// * `options` - Scan configuration options
///
/// # Returns
///
/// A `ScanResult` containing discovered alerts.
///
/// # Errors
///
/// Returns `Error::NotFound` if project doesn't exist.
pub async fn scan_project(
    project_id: &ProjectId,
    options: &ScanOptions,
) -> Result<ScanResult> {
    // Implementation
}

// Use builder pattern for complex constructors
let config = ScanConfig::builder()
    .parallel_workers(8)
    .timeout_seconds(300)
    .build();

// Prefer composition over inheritance
trait Detector {
    async fn analyze(&self, dependency: &Dependency) -> Result<Vec<Alert>>;
}

struct TyposquattingDetector {
    threshold: f64,
}

impl Detector for TyposquattingDetector {
    async fn analyze(&self, dependency: &Dependency) -> Result<Vec<Alert>> {
        // Implementation
    }
}
```

**Error Handling:**

```rust
// Use thiserror for error types
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Project not found: {0}")]
    NotFound(ProjectId),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
}

// Propagate errors with ?
async fn get_project(id: &ProjectId) -> Result<Project> {
    let project = repository.find_by_id(id).await?;
    Ok(project)
}
```

**Async/Await:**

```rust
// Use async-trait for trait methods
#[async_trait]
pub trait Repository {
    async fn find_by_id(&self, id: &Id) -> Result<Entity>;
}

// Prefer tokio::spawn for CPU-bound work
let handle = tokio::task::spawn_blocking(|| {
    expensive_computation()
});
let result = handle.await?;
```

### Database Code

```rust
// Use compile-time checked queries
let project = sqlx::query_as!(
    ProjectModel,
    r#"
    SELECT id, tenant_id, name, description
    FROM projects
    WHERE id = $1 AND tenant_id = $2
    "#,
    project_id.as_uuid(),
    tenant_id.as_uuid(),
)
.fetch_one(&pool)
.await?;

// Always filter by tenant_id for multi-tenancy
// Use transactions for multi-step operations
let mut tx = pool.begin().await?;

// Step 1
sqlx::query!(...)
    .execute(&mut *tx)
    .await?;

// Step 2
sqlx::query!(...)
    .execute(&mut *tx)
    .await?;

tx.commit().await?;
```

### Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_typosquatting_detection() {
        // Arrange
        let detector = TyposquattingDetector::new(0.85);
        let dependency = create_test_dependency("lodash-util");

        // Act
        let alerts = detector.analyze(&dependency).await.unwrap();

        // Assert
        assert_eq!(alerts.len(), 1);
        assert_eq!(alerts[0].alert_type, AlertType::Typosquatting);
    }

    #[test]
    fn test_similarity_score() {
        let score = calculate_similarity("lodash", "lodash-utils");
        assert!(score > 0.9);
    }
}
```

---

## Testing

### Unit Tests

```bash
# Run all unit tests
cargo test

# Run tests for specific crate
cargo test -p sctv-detectors

# Run specific test
cargo test test_typosquatting_detection

# Show test output
cargo test -- --nocapture
```

### Integration Tests

```bash
# Run integration tests (requires test database)
cargo test --test integration

# Set up test database
export DATABASE_URL="postgresql://postgres:postgres@localhost/sctv_test"
sqlx database create
sqlx migrate run
```

### Code Coverage

```bash
# Install tarpaulin
cargo install cargo-tarpaulin

# Generate coverage report
cargo tarpaulin --out Html --output-dir coverage
```

---

## Submitting Changes

### Pull Request Checklist

Before submitting, ensure:

- [ ] Code builds without warnings (`cargo build`)
- [ ] All tests pass (`cargo test`)
- [ ] Code is formatted (`cargo fmt`)
- [ ] Clippy passes (`cargo clippy -- -D warnings`)
- [ ] Documentation is updated
- [ ] CHANGELOG.md is updated (for user-facing changes)
- [ ] Commit messages follow conventional commits
- [ ] PR description explains the change

### Pull Request Template

```markdown
## Description
Brief description of what this PR does.

## Related Issues
Fixes #123
Relates to #456

## Changes
- Added typosquatting detection for npm packages
- Updated API to expose new detector
- Added comprehensive tests

## Testing
- [ ] Unit tests added/updated
- [ ] Integration tests added/updated
- [ ] Manual testing performed

## Screenshots (if applicable)
[Add screenshots for UI changes]

## Checklist
- [x] Code builds without warnings
- [x] Tests pass
- [x] Documentation updated
- [x] CHANGELOG updated
```

### Review Process

1. **Automated checks** - CI runs tests, lints, format checks
2. **Code review** - Maintainer reviews code quality and design
3. **Discussion** - Address feedback and questions
4. **Approval** - Maintainer approves changes
5. **Merge** - PR is merged to main branch

---

## Areas for Contribution

### High Priority

**Security Detectors:**
- Additional typosquatting algorithms
- License compliance checking
- Maintainer reputation scoring
- Dependency confusion detection

**Registry Support:**
- Go modules improvements
- Composer (PHP) support
- CocoaPods (iOS) support
- Pub (Dart/Flutter) support

**Integrations:**
- Additional notification channels (Discord, Telegram)
- More CI/CD platforms (Jenkins, CircleCI)
- SIEM integrations (Splunk, Datadog)

### Documentation

- Tutorial videos
- Blog posts and examples
- Language translations
- API client libraries

### Testing

- Increase test coverage
- Add integration tests
- Performance benchmarks
- Security testing

### Infrastructure

- Helm chart improvements
- Terraform modules
- Ansible playbooks
- Monitoring dashboards

---

## Getting Help

**Questions?**
- GitHub Discussions: Ask questions and discuss ideas
- Discord: Join our community chat
- Email: dev@example.com

**Found a bug?**
- Search existing issues first
- Create detailed bug report with reproduction steps
- Include version, OS, and configuration

**Want to propose a feature?**
- Open an issue with [Feature Request] tag
- Describe the use case and expected behavior
- Wait for feedback before implementing

---

## Recognition

Contributors are recognized in:
- CONTRIBUTORS.md file
- Release notes
- Project documentation

Thank you for contributing to SCTV! 🎉

---

## Next Steps

- [Development Setup](setup.md) - Detailed setup instructions
- [Code Standards](code-standards.md) - Complete coding guidelines
- [Testing Guide](testing.md) - Testing best practices
- [Architecture](../architecture/overview.md) - System architecture
