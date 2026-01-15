# GitHub Actions Workflows

This directory contains CI/CD workflows for the SCTV project.

## Workflows Overview

| Workflow | Trigger | Purpose |
|----------|---------|---------|
| `ci.yml` | Push, PR | Run tests, linting, and build verification |
| `docker.yml` | Push to main, tags | Build and push Docker images |
| `release.yml` | Version tags | Create GitHub releases with binaries |
| `security.yml` | Push, PR, scheduled | Security scanning and analysis |

## CI Workflow (`ci.yml`)

Runs on every push and pull request to `main` and `develop` branches.

### Jobs

- **lint**: Checks code formatting (`cargo fmt`) and runs Clippy linter
- **test**: Runs the test suite with PostgreSQL service container
- **build**: Verifies release builds compile correctly
- **security**: Runs `cargo audit` for vulnerability scanning
- **docs**: Verifies documentation builds without warnings
- **coverage**: Generates code coverage reports for Codecov

### Required Secrets

None - uses built-in `GITHUB_TOKEN`.

## Docker Workflow (`docker.yml`)

Builds and pushes Docker images to GitHub Container Registry (ghcr.io).

### Jobs

- **build**: Builds API and Worker images in parallel
- **scan**: Runs Trivy vulnerability scanner on images
- **deploy-staging**: Optional deployment to staging environment

### Image Tags

- `latest` - Latest main branch build
- `sha-<commit>` - Specific commit
- `v1.2.3` - Semantic version from tags
- `v1.2` - Major.minor version
- `v1` - Major version only

### Required Secrets

- `GITHUB_TOKEN` (built-in) - For pushing to ghcr.io

## Release Workflow (`release.yml`)

Creates GitHub releases when version tags are pushed.

### Triggers

```bash
# Create a release
git tag v1.0.0
git push origin v1.0.0
```

### Jobs

- **create-release**: Creates the GitHub release
- **build-binaries**: Builds binaries for multiple platforms
- **docker-release**: Builds and pushes versioned Docker images
- **post-release**: Cleanup and notifications

### Supported Platforms

| Platform | Target |
|----------|--------|
| Linux x86_64 | `x86_64-unknown-linux-gnu` |
| Linux ARM64 | `aarch64-unknown-linux-gnu` |
| macOS x86_64 | `x86_64-apple-darwin` |
| macOS ARM64 | `aarch64-apple-darwin` |
| Windows x86_64 | `x86_64-pc-windows-msvc` |

## Security Workflow (`security.yml`)

Runs security analysis on the codebase.

### Jobs

- **codeql**: Static analysis with GitHub CodeQL
- **dependency-review**: Reviews dependency changes in PRs
- **secrets-scan**: Scans for leaked secrets using TruffleHog
- **scorecard**: OSSF Scorecard analysis

### Schedule

Runs weekly on Sundays in addition to push/PR triggers.

## Branch Protection

Recommended branch protection settings for `main`:

- Require status checks to pass:
  - `lint`
  - `test`
  - `build`
  - `Dependency Review`
- Require pull request reviews
- Require signed commits (optional)
- Do not allow force pushes

## Caching

All workflows use GitHub Actions cache for:
- Cargo registry (`~/.cargo/registry`)
- Cargo git dependencies (`~/.cargo/git`)
- Build target directory (`target/`)
- Docker layer cache (GitHub Actions cache)

## Secrets Configuration

For full functionality, configure these secrets in repository settings:

| Secret | Required | Purpose |
|--------|----------|---------|
| `GITHUB_TOKEN` | Auto | Package publishing, releases |
| `CODECOV_TOKEN` | Optional | Code coverage uploads |

## Local Testing

Test workflows locally using [act](https://github.com/nektos/act):

```bash
# Install act
brew install act  # macOS
# or
curl https://raw.githubusercontent.com/nektos/act/master/install.sh | sudo bash

# Run CI workflow
act push

# Run specific job
act -j test

# Run with secrets
act -s GITHUB_TOKEN=xxx
```

## Troubleshooting

### Build Failures

1. Check Rust version compatibility in `Cargo.toml`
2. Verify all dependencies are available
3. Check for platform-specific build issues

### Docker Push Failures

1. Verify `GITHUB_TOKEN` has `packages: write` permission
2. Check image name format: `ghcr.io/owner/repo-name`
3. Verify the package visibility settings

### Test Failures

1. Check PostgreSQL service health
2. Verify database migrations are up to date
3. Review test logs for specific failures
