# Configuration Guide

**Version:** 0.1.0

Complete reference for configuring Supply Chain Trust Verifier (SCTV).

---

## Table of Contents

- [Configuration Files](#configuration-files)
- [Environment Variables](#environment-variables)
- [Database Configuration](#database-configuration)
- [API Server Configuration](#api-server-configuration)
- [Worker Configuration](#worker-configuration)
- [Notification Channels](#notification-channels)
- [Security Settings](#security-settings)
- [Logging and Monitoring](#logging-and-monitoring)
- [Performance Tuning](#performance-tuning)
- [Examples](#examples)

---

## Configuration Files

SCTV uses TOML configuration files by default. Configuration can also be provided via environment variables.

### Configuration File Locations

SCTV searches for configuration in this order:

1. Path specified via `--config` flag
2. `./config.toml` (current directory)
3. `~/.config/sctv/config.toml` (user config)
4. `/etc/sctv/config.toml` (system config)

### Basic Configuration Structure

```toml
# /etc/sctv/config.toml

[database]
host = "localhost"
port = 5432
database = "sctv"
username = "sctv"
password = "secure-password"
max_connections = 20
connection_timeout_seconds = 30

[api]
bind_addr = "0.0.0.0:3000"
jwt_secret = "your-jwt-secret-change-in-production"
jwt_expiry_hours = 24
enable_cors = true
cors_allowed_origins = ["http://localhost:3001"]
max_request_size_mb = 10

[worker]
pool_size = 4
poll_interval_seconds = 5
max_retries = 3
retry_backoff_multiplier = 2.0
stale_job_timeout_minutes = 60
cleanup_completed_after_days = 30

[logging]
level = "info"
format = "json"
file = "/var/log/sctv/sctv.log"
max_size_mb = 100
max_backups = 10

[metrics]
enabled = true
bind_addr = "0.0.0.0:9090"
```

---

## Environment Variables

All configuration options can be set via environment variables using the prefix `SCTV_`:

| Environment Variable | Config Path | Default | Description |
|---------------------|-------------|---------|-------------|
| `SCTV_DATABASE_HOST` | `database.host` | `localhost` | PostgreSQL host |
| `SCTV_DATABASE_PORT` | `database.port` | `5432` | PostgreSQL port |
| `SCTV_DATABASE_NAME` | `database.database` | `sctv` | Database name |
| `SCTV_DATABASE_USER` | `database.username` | `sctv` | Database user |
| `SCTV_DATABASE_PASSWORD` | `database.password` | - | Database password (required) |
| `SCTV_API_BIND_ADDR` | `api.bind_addr` | `0.0.0.0:3000` | API server address |
| `SCTV_JWT_SECRET` | `api.jwt_secret` | - | JWT signing secret (required) |
| `SCTV_WORKER_POOL_SIZE` | `worker.pool_size` | `4` | Worker thread count |
| `SCTV_LOG_LEVEL` | `logging.level` | `info` | Log level |

### Example with Environment Variables

```bash
export SCTV_DATABASE_HOST=db.example.com
export SCTV_DATABASE_PASSWORD=super-secret
export SCTV_JWT_SECRET=jwt-signing-secret
export SCTV_LOG_LEVEL=debug

sctv-api
```

---

## Database Configuration

### Connection Settings

```toml
[database]
# PostgreSQL connection
host = "localhost"
port = 5432
database = "sctv"
username = "sctv"
password = "your-password"

# Connection pool settings
max_connections = 20
min_connections = 5
connection_timeout_seconds = 30
idle_timeout_seconds = 600

# SSL/TLS settings
require_ssl = true
ssl_mode = "require"  # Options: disable, allow, prefer, require, verify-ca, verify-full
ssl_ca_cert = "/etc/sctv/certs/ca.crt"
ssl_client_cert = "/etc/sctv/certs/client.crt"
ssl_client_key = "/etc/sctv/certs/client.key"
```

### Connection String Format

Alternatively, use a PostgreSQL connection string:

```toml
[database]
url = "postgresql://sctv:password@localhost:5432/sctv?sslmode=require"
```

### Performance Tuning

For high-throughput deployments:

```toml
[database]
max_connections = 50
statement_cache_capacity = 100
test_connection_on_checkout = true
```

---

## API Server Configuration

### Server Settings

```toml
[api]
# Network binding
bind_addr = "0.0.0.0:3000"
bind_addr_ipv6 = "[::]:3000"

# Request handling
max_request_size_mb = 10
request_timeout_seconds = 30
keep_alive_timeout_seconds = 75

# Concurrency
max_concurrent_requests = 1000
```

### Authentication

```toml
[api.auth]
# JWT configuration
jwt_secret = "your-secret-key-min-32-bytes"
jwt_algorithm = "HS256"  # HS256, HS384, HS512, RS256
jwt_expiry_hours = 24
jwt_refresh_hours = 168  # 7 days

# API key settings
api_key_header = "X-API-Key"
api_key_length = 64

# Session settings
session_duration_hours = 12
session_absolute_timeout_hours = 720  # 30 days
```

### CORS Configuration

```toml
[api.cors]
enabled = true
allowed_origins = [
    "http://localhost:3001",
    "https://dashboard.example.com"
]
allowed_methods = ["GET", "POST", "PUT", "DELETE", "PATCH"]
allowed_headers = ["Authorization", "Content-Type"]
allow_credentials = true
max_age_seconds = 3600
```

### GraphQL Settings

```toml
[api.graphql]
enabled = true
introspection_enabled = true  # Disable in production
max_query_depth = 10
max_query_complexity = 1000
upload_max_size_mb = 50
```

### Rate Limiting

```toml
[api.rate_limit]
enabled = true
requests_per_minute = 60
requests_per_hour = 1000
burst_size = 10
by_ip = true
by_api_key = true
```

---

## Worker Configuration

### Pool Settings

```toml
[worker]
# Worker pool
pool_size = 4
min_workers = 2
max_workers = 16
scale_up_threshold = 0.8  # Scale up when 80% busy
scale_down_threshold = 0.2  # Scale down when <20% busy

# Job polling
poll_interval_seconds = 5
poll_batch_size = 10
claim_timeout_seconds = 300

# Retry configuration
max_retries = 3
retry_backoff_multiplier = 2.0
max_retry_delay_seconds = 3600

# Stale job recovery
stale_job_timeout_minutes = 60
recovery_check_interval_minutes = 5

# Cleanup
cleanup_completed_after_days = 30
cleanup_failed_after_days = 90
cleanup_interval_hours = 24
```

### Job Priorities

```toml
[worker.priorities]
scan_project = 5
verify_provenance = 8
monitor_registry = 3
send_notification = 10
```

### Executor Configuration

```toml
[worker.executors]
# Scan project executor
[worker.executors.scan_project]
timeout_seconds = 600
max_dependencies = 10000
parallel_fetches = 20

# Provenance verification
[worker.executors.verify_provenance]
timeout_seconds = 300
sigstore_fulcio_url = "https://fulcio.sigstore.dev"
sigstore_rekor_url = "https://rekor.sigstore.dev"

# Registry monitoring
[worker.executors.monitor_registry]
timeout_seconds = 120
check_interval_hours = 6
```

---

## Notification Channels

### Email Notifications

```toml
[notifications.email]
enabled = true
smtp_host = "smtp.gmail.com"
smtp_port = 587
smtp_username = "alerts@example.com"
smtp_password = "app-specific-password"
smtp_use_tls = true
from_address = "alerts@example.com"
from_name = "SCTV Alerts"
default_recipients = ["security@example.com"]
min_severity = "medium"
```

### Slack Integration

```toml
[notifications.slack]
enabled = true
webhook_url = "https://hooks.slack.com/services/xxx/yyy/zzz"
channel = "#security-alerts"
username = "SCTV Bot"
icon_emoji = ":shield:"
min_severity = "high"
mention_on_critical = "@security-team"
thread_alerts = true
```

### Microsoft Teams

```toml
[notifications.teams]
enabled = true
webhook_url = "https://outlook.office.com/webhook/xxx"
min_severity = "medium"
theme_color = "FF0000"  # Red for alerts
```

### PagerDuty

```toml
[notifications.pagerduty]
enabled = true
integration_key = "your-integration-key"
routing_key = "your-routing-key"
min_severity = "critical"
client_name = "SCTV"
dedup_alerts = true
```

### Generic Webhook

```toml
[notifications.webhook]
enabled = true
url = "https://api.example.com/webhooks/sctv"
method = "POST"
headers = { Authorization = "Bearer token", Content-Type = "application/json" }
timeout_seconds = 10
retry_count = 3
min_severity = "medium"

# Webhook payload template
template = '''
{
  "alert_id": "{{alert_id}}",
  "severity": "{{severity}}",
  "title": "{{title}}",
  "description": "{{description}}",
  "project": "{{project_name}}",
  "timestamp": "{{created_at}}"
}
'''
```

---

## Security Settings

### TLS/SSL Configuration

```toml
[api.tls]
enabled = true
cert_file = "/etc/sctv/certs/server.crt"
key_file = "/etc/sctv/certs/server.key"
ca_file = "/etc/sctv/certs/ca.crt"
min_tls_version = "1.2"
cipher_suites = [
    "TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256",
    "TLS_ECDHE_RSA_WITH_AES_256_GCM_SHA384"
]
```

### Security Headers

```toml
[api.security]
# Security headers
enable_hsts = true
hsts_max_age_seconds = 31536000
hsts_include_subdomains = true

enable_csp = true
csp_directives = "default-src 'self'; script-src 'self' 'unsafe-inline'"

x_frame_options = "DENY"
x_content_type_options = "nosniff"
x_xss_protection = "1; mode=block"
```

### Password Policy

```toml
[auth.password_policy]
min_length = 12
require_uppercase = true
require_lowercase = true
require_numbers = true
require_special_chars = true
max_age_days = 90
prevent_reuse_count = 5
```

---

## Logging and Monitoring

### Logging Configuration

```toml
[logging]
# Log level: trace, debug, info, warn, error
level = "info"

# Log format: text, json
format = "json"

# Output destinations
[logging.outputs]
stdout = true
file = "/var/log/sctv/sctv.log"
syslog = false
syslog_address = "localhost:514"

# File rotation
[logging.file_rotation]
max_size_mb = 100
max_backups = 10
max_age_days = 30
compress = true

# Structured logging fields
[logging.fields]
service = "sctv"
environment = "production"
datacenter = "us-east-1"
```

### Metrics and Observability

```toml
[metrics]
# Prometheus metrics
enabled = true
bind_addr = "0.0.0.0:9090"
path = "/metrics"

# Custom metrics
[metrics.custom]
include_go_metrics = true
include_process_metrics = true
histogram_buckets = [0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0]
```

### Distributed Tracing

```toml
[tracing]
enabled = true
endpoint = "http://jaeger:14268/api/traces"
service_name = "sctv-api"
sampling_rate = 0.1  # Sample 10% of requests
```

---

## Performance Tuning

### Caching Configuration

```toml
[cache]
# Registry data cache
[cache.registry]
enabled = true
ttl_seconds = 3600
max_entries = 10000
eviction_policy = "lru"

# Package metadata cache
[cache.packages]
enabled = true
ttl_seconds = 7200
max_size_mb = 500
```

### Concurrency Limits

```toml
[performance]
# API concurrency
max_concurrent_api_requests = 1000
api_request_queue_size = 5000

# Worker concurrency
max_concurrent_jobs = 20
max_concurrent_registry_requests = 50

# Database
database_connection_pool_size = 50
```

### Resource Limits

```toml
[limits]
# Request limits
max_project_dependencies = 50000
max_sbom_size_mb = 100
max_policy_rules = 100

# Tenant limits
max_projects_per_tenant = 100
max_policies_per_tenant = 50
max_users_per_tenant = 1000
```

---

## Examples

### Development Configuration

```toml
# config.dev.toml - For local development

[database]
host = "localhost"
port = 5432
database = "sctv_dev"
username = "postgres"
password = "postgres"
max_connections = 5

[api]
bind_addr = "127.0.0.1:3000"
jwt_secret = "dev-secret-not-for-production"
enable_cors = true

[worker]
pool_size = 2

[logging]
level = "debug"
format = "text"

[metrics]
enabled = false
```

### Production Configuration

```toml
# config.prod.toml - For production deployment

[database]
url = "postgresql://sctv:${SCTV_DB_PASSWORD}@db.internal:5432/sctv?sslmode=require"
max_connections = 50
require_ssl = true

[api]
bind_addr = "0.0.0.0:3000"
jwt_secret = "${SCTV_JWT_SECRET}"
enable_cors = false
max_request_size_mb = 5

[api.tls]
enabled = true
cert_file = "/etc/sctv/certs/server.crt"
key_file = "/etc/sctv/certs/server.key"

[worker]
pool_size = 8
max_retries = 5

[logging]
level = "info"
format = "json"
file = "/var/log/sctv/sctv.log"

[metrics]
enabled = true
bind_addr = "0.0.0.0:9090"

[notifications.slack]
enabled = true
webhook_url = "${SLACK_WEBHOOK_URL}"
min_severity = "high"

[notifications.pagerduty]
enabled = true
integration_key = "${PAGERDUTY_KEY}"
min_severity = "critical"
```

### High-Availability Configuration

```toml
# config.ha.toml - For HA deployment

[database]
# Use managed database with read replicas
primary_url = "postgresql://sctv:pass@primary.db:5432/sctv"
replica_urls = [
    "postgresql://sctv:pass@replica1.db:5432/sctv",
    "postgresql://sctv:pass@replica2.db:5432/sctv"
]
max_connections = 100

[api]
bind_addr = "0.0.0.0:3000"
max_concurrent_requests = 5000

[worker]
pool_size = 16
min_workers = 8
max_workers = 32
scale_up_threshold = 0.7
scale_down_threshold = 0.3

[cache.registry]
enabled = true
backend = "redis"
redis_url = "redis://cache.internal:6379"
ttl_seconds = 1800

[logging]
outputs = ["stdout", "file"]
file = "/var/log/sctv/sctv.log"

[metrics]
enabled = true
```

---

## Configuration Validation

Validate your configuration file:

```bash
# Check configuration syntax
sctv-cli config validate --config /etc/sctv/config.toml

# Show resolved configuration (with env vars)
sctv-cli config show --config /etc/sctv/config.toml

# Test database connection
sctv-cli config test-db --config /etc/sctv/config.toml
```

---

## Next Steps

- **Deploy to Production:** See [Deployment Guide](../operations/deployment.md)
- **Monitor Your Deployment:** See [Monitoring Guide](../operations/monitoring.md)
- **Secure Your Installation:** See [Security Hardening](../operations/security.md)

---

## Reference

- [Configuration Schema](../reference/configuration.md) - Complete schema reference
- [Environment Variables](../reference/configuration.md#environment-variables) - All env vars
- [Security Best Practices](../operations/security.md) - Production security
