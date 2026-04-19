# Configuration Reference

Complete configuration reference for the SCTV (Supply Chain Trust Verifier) platform.

## Table of Contents

- [Environment Variables](#environment-variables)
- [Configuration Files](#configuration-files)
- [Configuration Precedence](#configuration-precedence)
- [Default Values](#default-values)
- [Per-Environment Configuration](#per-environment-configuration)
- [Feature Flags](#feature-flags)
- [Advanced Tuning Options](#advanced-tuning-options)
- [Configuration Validation](#configuration-validation)
- [Migration Between Versions](#migration-between-versions)

## Environment Variables

SCTV uses environment variables for configuration. All variables are prefixed with `SCTV_` or follow standard naming conventions for specific services.

### Database Configuration

| Variable | Type | Default | Description |
|----------|------|---------|-------------|
| `DATABASE_URL` | String | *Required* | PostgreSQL connection string in the format `postgres://user:password@host:port/database` |
| `POSTGRES_USER` | String | `sctv` | PostgreSQL username (used in Docker Compose) |
| `POSTGRES_PASSWORD` | String | *Required* | PostgreSQL password (used in Docker Compose) |
| `POSTGRES_DB` | String | `sctv` | PostgreSQL database name (used in Docker Compose) |
| `POSTGRES_PORT` | Integer | `5432` | PostgreSQL port (used in Docker Compose) |

**Example:**
```bash
DATABASE_URL=postgres://sctv:secure_password@localhost:5432/sctv_production
```

**Connection Pool Settings:**
- Connection pooling is managed automatically by SQLx
- Default pool size: 10 connections
- Connection timeout: 30 seconds

### API Server Configuration

| Variable | Type | Default | Description |
|----------|------|---------|-------------|
| `API_PORT` | Integer | `3000` | Port to bind the API server |
| `SCTV_JWT_SECRET` | String | *Required in production* | Secret key for JWT token signing (minimum 32 characters recommended) |
| `SCTV_ENABLE_CORS` | Boolean | `true` | Enable Cross-Origin Resource Sharing (disable in production if not needed) |
| `SCTV_LOG_FORMAT` | String | `json` | Log format: `json` (structured) or `pretty` (development) |

**Security Notes:**
- Always set a strong, random `SCTV_JWT_SECRET` in production
- Generate with: `openssl rand -base64 32`
- Disable CORS unless you need cross-origin API access (production defaults to off in `docker-compose.prod.yml`)

**Example:**
```bash
SCTV_JWT_SECRET=your-super-secret-jwt-key-change-in-production
API_PORT=3000
SCTV_ENABLE_CORS=false
SCTV_LOG_FORMAT=json
```

### Worker Configuration

| Variable | Type | Default | Description |
|----------|------|---------|-------------|
| `SCTV_WORKER_COUNT` | Integer | `4` | Number of concurrent worker threads (set based on CPU cores) |
| `SCTV_POLL_INTERVAL_MS` | Integer | `1000` | Job queue polling interval in milliseconds |
| `SCTV_SHUTDOWN_TIMEOUT_SECS` | Integer | `30` | Graceful shutdown timeout in seconds |

**Performance Tuning:**
- Set `SCTV_WORKER_COUNT` to number of CPU cores for CPU-bound workloads
- Increase for I/O-bound workloads (registry fetching, HTTP requests)
- Recommended: Start with 4-8 workers and adjust based on monitoring

**Example:**
```bash
SCTV_WORKER_COUNT=8
SCTV_POLL_INTERVAL_MS=500
SCTV_SHUTDOWN_TIMEOUT_SECS=60
```

### Logging Configuration

| Variable | Type | Default | Description |
|----------|------|---------|-------------|
| `RUST_LOG` | String | `info` | Rust logging level configuration (supports per-module filtering) |
| `SCTV_LOG_FORMAT` | String | `json` | Output format: `json` or `pretty` |

**Log Levels (in order of verbosity):**
- `error` - Error messages only
- `warn` - Warnings and errors
- `info` - Informational messages (default)
- `debug` - Debug information
- `trace` - Verbose trace information

**Example Configurations:**
```bash
# Production: JSON logging, info level
RUST_LOG=info,sctv_api=info,sctv_worker=info
SCTV_LOG_FORMAT=json

# Development: Pretty logging with debug
RUST_LOG=debug,sctv_api=debug,sctv_worker=debug,tower_http=info,sqlx=warn
SCTV_LOG_FORMAT=pretty

# Debug specific components
RUST_LOG=info,sctv_detectors=debug,sctv_registries=debug
```

### Email/SMTP Configuration

| Variable | Type | Default | Description |
|----------|------|---------|-------------|
| `SMTP_HOST` | String | - | SMTP server hostname |
| `SMTP_PORT` | Integer | `587` | SMTP server port (587 for STARTTLS, 465 for SSL/TLS) |
| `SMTP_USERNAME` | String | - | SMTP authentication username |
| `SMTP_PASSWORD` | String | - | SMTP authentication password |
| `SMTP_FROM` | String | `noreply@sctv.local` | Default sender email address |
| `SMTP_TLS` | Boolean | `true` | Use TLS for SMTP connection |

**Example:**
```bash
SMTP_HOST=smtp.gmail.com
SMTP_PORT=587
SMTP_USERNAME=alerts@example.com
SMTP_PASSWORD=app-specific-password
SMTP_FROM=sctv-alerts@example.com
SMTP_TLS=true
```

### External Integrations

| Variable | Type | Default | Description |
|----------|------|---------|-------------|
| `GITHUB_TOKEN` | String | - | GitHub Personal Access Token for enhanced API rate limits |
| `GITLAB_TOKEN` | String | - | GitLab Personal Access Token |
| `SIGSTORE_ENABLED` | Boolean | `true` | Enable Sigstore/Fulcio provenance verification |

**GitHub Token Scopes:**
- `public_repo` - Access public repositories
- No scopes needed for public package metadata

**Example:**
```bash
GITHUB_TOKEN=ghp_xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx
GITLAB_TOKEN=glpat-xxxxxxxxxxxxxxxxxxxx
SIGSTORE_ENABLED=true
```

### Dashboard Configuration

| Variable | Type | Default | Description |
|----------|------|---------|-------------|
| `DASHBOARD_PORT` | Integer | `8080` | Port for the web dashboard (if using separate dashboard service) |

## Configuration Files

SCTV primarily uses environment variables, but certain features support configuration files.

### Environment File (.env)

Create a `.env` file in the project root for local development:

```bash
# Copy from example
cp .env.example .env

# Edit with your settings
nano .env
```

**Important:** Never commit `.env` files with real credentials to version control.

### Docker Compose Files

SCTV provides multiple Docker Compose configurations:

| File | Purpose | Environment |
|------|---------|-------------|
| `docker-compose.yml` | Base configuration | All |
| `docker-compose.override.yml` | Development overrides | Development |
| `docker-compose.prod.yml` | Production configuration | Production |

**Usage:**
```bash
# Development (uses override automatically)
docker-compose up

# Production
docker-compose -f docker-compose.yml -f docker-compose.prod.yml up
```

## Configuration Precedence

Configuration values are loaded in the following order (later values override earlier):

1. Default values (hardcoded in application)
2. `.env` file (development only)
3. Environment variables
4. Command-line arguments (if supported)

**Example:**
```bash
# Default port is 3000
# .env has API_PORT=4000
# Environment variable overrides both
API_PORT=5000 cargo run --bin sctv-api
# Server starts on port 5000
```

## Default Values

### API Server Defaults

```rust
ServerConfig {
    bind_addr: "127.0.0.1:3000",
    jwt_secret: "development-secret-change-in-production",
    enable_cors: true,
}
```

### Worker Defaults

```rust
WorkerConfig {
    worker_count: 4,
    poll_interval_ms: 1000,
    shutdown_timeout_secs: 30,
}
```

### Registry Client Defaults

```rust
// npm
NpmClient::DEFAULT_REGISTRY = "https://registry.npmjs.org"

// PyPI
PyPiClient::DEFAULT_REGISTRY = "https://pypi.org"

// Maven
MavenClient::DEFAULT_REGISTRY = "https://repo1.maven.org/maven2"

// NuGet
NuGetClient::DEFAULT_REGISTRY = "https://api.nuget.org/v3/index.json"

// RubyGems
RubyGemsClient::DEFAULT_REGISTRY = "https://rubygems.org"

// Cargo
CargoClient::DEFAULT_REGISTRY = "https://crates.io"

// Go Modules
GoModulesClient::DEFAULT_REGISTRY = "https://proxy.golang.org"
```

**HTTP Client Settings:**
- Timeout: 30 seconds
- User-Agent: `sctv-registry-client/0.1.0`
- GZIP compression: Enabled

## Per-Environment Configuration

### Development Environment

```bash
# .env.development
DATABASE_URL=postgres://sctv:sctv_dev@localhost:5432/sctv_dev
SCTV_JWT_SECRET=development-secret-not-for-production
SCTV_ENABLE_CORS=true
SCTV_LOG_FORMAT=pretty
RUST_LOG=debug,sctv_api=debug,sctv_worker=debug,sqlx=warn
SCTV_WORKER_COUNT=2
```

### Testing Environment

```bash
# .env.test
DATABASE_URL=postgres://sctv_test:sctv_test@localhost:5432/sctv_test
SCTV_JWT_SECRET=test-secret-for-integration-tests
SCTV_ENABLE_CORS=true
RUST_LOG=info,sctv_api=debug,sctv_worker=debug
```

### Staging Environment

```bash
# .env.staging
DATABASE_URL=postgres://sctv:${POSTGRES_PASSWORD}@db.staging.example.com:5432/sctv_staging
SCTV_JWT_SECRET=${JWT_SECRET_FROM_VAULT}
SCTV_ENABLE_CORS=true
SCTV_LOG_FORMAT=json
RUST_LOG=info,sctv_api=debug
SCTV_WORKER_COUNT=4
```

### Production Environment

```bash
# Production environment variables (set via secrets management)
DATABASE_URL=postgres://sctv:${POSTGRES_PASSWORD}@db.prod.example.com:5432/sctv
SCTV_JWT_SECRET=${JWT_SECRET_FROM_VAULT}
SCTV_ENABLE_CORS=false
SCTV_LOG_FORMAT=json
RUST_LOG=info,sctv_api=info,sctv_worker=info,tower_http=warn
SCTV_WORKER_COUNT=8
SCTV_POLL_INTERVAL_MS=500

# Email notifications
SMTP_HOST=smtp.sendgrid.net
SMTP_PORT=587
SMTP_USERNAME=${SENDGRID_USERNAME}
SMTP_PASSWORD=${SENDGRID_API_KEY}
SMTP_FROM=alerts@example.com

# External integrations
GITHUB_TOKEN=${GITHUB_TOKEN_FROM_VAULT}
SIGSTORE_ENABLED=true
```

## Feature Flags

SCTV supports the following feature flags:

### Ecosystem Support

Control which package ecosystems are enabled:

```rust
// Tenant settings (database-stored)
TenantSettings {
    allowed_ecosystems: vec![
        PackageEcosystem::Npm,
        PackageEcosystem::PyPi,
        PackageEcosystem::Maven,
        // Optionally exclude others
    ],
}
```

### Detection Features

```rust
// Per-tenant configuration
TenantSettings {
    continuous_monitoring: true,  // Enable automatic scanning
}
```

### Notification Channels

```rust
// Per-tenant notification configuration
NotificationChannelConfig {
    channel_type: NotificationChannelType::Email,
    enabled: true,
    min_severity: Severity::Medium,
    config: serde_json::json!({
        "recipients": ["security@example.com"]
    }),
}
```

## Advanced Tuning Options

### Database Performance

```bash
# Increase connection pool for high concurrency
# Note: Requires application code changes for advanced tuning
# Default pool size is managed by SQLx: ~10 connections
```

**PostgreSQL Configuration:**
```ini
# postgresql.conf recommendations
max_connections = 100
shared_buffers = 256MB
effective_cache_size = 1GB
work_mem = 16MB
maintenance_work_mem = 64MB
```

### Worker Pool Tuning

```bash
# CPU-intensive workloads (analysis, hashing)
SCTV_WORKER_COUNT=4  # ~= CPU cores

# I/O-intensive workloads (registry fetching)
SCTV_WORKER_COUNT=16  # > CPU cores

# Reduce polling overhead
SCTV_POLL_INTERVAL_MS=2000  # Poll every 2 seconds instead of 1
```

### Registry Client Tuning

Registry clients use the following hardcoded settings:

- **Timeout:** 30 seconds
- **GZIP:** Enabled
- **Connection pooling:** Managed by reqwest HTTP client

For custom timeouts or retry logic, modify the client implementation.

### Caching Strategy

SCTV uses in-memory LRU caching for registry data:

```rust
// Default cache settings (in RegistryCache)
// - Stores package metadata to reduce API calls
// - TTL and size managed internally
```

## Configuration Validation

SCTV validates configuration on startup and reports errors clearly.

### Common Validation Errors

**Missing Required Variables:**
```
Error: Environment variable SCTV_JWT_SECRET is required in production
```

**Solution:** Set the required variable in your environment or `.env` file.

**Invalid Database URL:**
```
Error: Failed to connect to database: invalid connection string
```

**Solution:** Verify `DATABASE_URL` format and credentials.

**Port Already in Use:**
```
Error: Failed to bind to 127.0.0.1:3000: address already in use
```

**Solution:** Change `API_PORT` or stop the conflicting service.

### Validation Checklist

Before deploying to production:

- [ ] `SCTV_JWT_SECRET` is set to a strong random value (min 32 characters)
- [ ] `DATABASE_URL` points to production database
- [ ] `POSTGRES_PASSWORD` is strong and secure
- [ ] `SCTV_ENABLE_CORS` is `false` (unless a gateway in front of the API handles CORS)
- [ ] `RUST_LOG` is set to `info` or `warn` (not `debug`)
- [ ] SMTP credentials are configured for email notifications
- [ ] External integration tokens are set (if needed)

## Migration Between Versions

### Version 0.1.x to 0.2.x (Future)

Configuration changes will be documented here when new versions are released.

**Breaking Changes:** None yet (initial version)

### Environment Variable Renames

If variables are renamed in future versions, the migration path will be documented:

```bash
# Example (not yet applicable)
# OLD_VAR_NAME → NEW_VAR_NAME
```

### Database Migrations

Database schema migrations are handled automatically by SQLx at startup.

**Manual Migration:**
```bash
# Run migrations manually
sqlx migrate run --database-url $DATABASE_URL

# Revert last migration
sqlx migrate revert --database-url $DATABASE_URL
```

**Migration Files Location:**
```
migrations/
├── 20240101000000_initial_schema.sql
├── 20240102000000_add_indexes.sql
└── ...
```

### Backward Compatibility

SCTV maintains backward compatibility for:
- Environment variable names (with deprecation warnings)
- Database schema (migrations are additive)
- API endpoints (with version prefixes)

**Deprecation Policy:**
- Deprecated features receive warnings for at least 2 minor versions
- Removal only occurs in major version updates
- Migration guides provided for all breaking changes

## Troubleshooting

### Configuration Not Loading

**Problem:** Environment variables don't seem to apply.

**Solutions:**
1. Check variable names (case-sensitive)
2. Restart the service after changing variables
3. Verify no typos in `.env` file
4. Check Docker Compose passes variables correctly

### JWT Authentication Failures

**Problem:** `401 Unauthorized` on all API requests.

**Solutions:**
1. Verify `SCTV_JWT_SECRET` is set and matches between API and client
2. Check token hasn't expired
3. Ensure token is passed in `Authorization: Bearer <token>` header

### Database Connection Errors

**Problem:** Cannot connect to PostgreSQL.

**Solutions:**
1. Verify `DATABASE_URL` format and credentials
2. Check PostgreSQL is running: `docker-compose ps postgres`
3. Test connection manually: `psql $DATABASE_URL`
4. Check firewall/network connectivity

### Worker Not Processing Jobs

**Problem:** Jobs stuck in pending state.

**Solutions:**
1. Check worker service is running: `docker-compose ps worker`
2. Verify `DATABASE_URL` is accessible from worker
3. Check logs: `docker-compose logs worker`
4. Increase `SCTV_WORKER_COUNT` if overwhelmed

## See Also

- [Getting Started - Configuration](../getting-started/configuration.md)
- [Architecture - Configuration System](../architecture/configuration.md)
- [Deployment - Environment Setup](../deployment/environment-setup.md)
- [API Reference](./api.md)
