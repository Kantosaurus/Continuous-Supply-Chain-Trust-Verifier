# Error Codes Reference

Complete reference for SCTV error codes, troubleshooting, and error handling.

## Table of Contents

- [Error Response Format](#error-response-format)
- [API Error Codes (4xx, 5xx)](#api-error-codes-4xx-5xx)
- [Domain Error Codes](#domain-error-codes)
- [Database Error Codes](#database-error-codes)
- [Detection Error Codes](#detection-error-codes)
- [Registry Client Errors](#registry-client-errors)
- [Notification Errors](#notification-errors)
- [Worker/Job Errors](#worker-job-errors)
- [Troubleshooting Guide](#troubleshooting-guide)

## Error Response Format

All API errors follow a consistent JSON format:

```json
{
  "error": {
    "code": "ERROR_CODE",
    "message": "Human-readable error message",
    "details": {
      "additional": "context",
      "field": "value"
    }
  }
}
```

### Response Structure

| Field | Type | Description |
|-------|------|-------------|
| `error.code` | String | Machine-readable error code (uppercase, underscore-separated) |
| `error.message` | String | Human-readable error description |
| `error.details` | Object | Optional additional context (omitted if null) |

### Example Error Response

```json
{
  "error": {
    "code": "NOT_FOUND",
    "message": "Resource not found: Project with ID abc123",
    "details": {
      "resource_type": "Project",
      "resource_id": "abc123"
    }
  }
}
```

## API Error Codes (4xx, 5xx)

HTTP status codes and corresponding error codes returned by the REST and GraphQL APIs.

### 400 Bad Request

**Status Code:** `400`

| Error Code | Description | Troubleshooting |
|------------|-------------|-----------------|
| `BAD_REQUEST` | Invalid request format or parameters | Check request body and query parameters match API specification |

**Example:**
```json
{
  "error": {
    "code": "BAD_REQUEST",
    "message": "Bad request: Missing required field 'name'",
    "details": {
      "field": "name",
      "expected": "non-empty string"
    }
  }
}
```

**Common Causes:**
- Missing required fields in request body
- Invalid JSON syntax
- Wrong data types for fields
- Invalid query parameters

**Resolution:**
1. Validate request against API documentation
2. Check JSON is well-formed
3. Ensure all required fields are present
4. Verify data types match expected schema

### 401 Unauthorized

**Status Code:** `401`

| Error Code | Description | Troubleshooting |
|------------|-------------|-----------------|
| `UNAUTHORIZED` | Authentication required or failed | Provide valid JWT token in Authorization header |

**Example:**
```json
{
  "error": {
    "code": "UNAUTHORIZED",
    "message": "Authentication required"
  }
}
```

**Common Causes:**
- No `Authorization` header provided
- Invalid or expired JWT token
- Malformed token format
- JWT secret mismatch

**Resolution:**
1. Include `Authorization: Bearer <token>` header
2. Obtain a fresh token via `/api/v1/auth/login`
3. Verify token hasn't expired (check `exp` claim)
4. Ensure `SCTV_JWT_SECRET` matches between services

### 403 Forbidden

**Status Code:** `403`

| Error Code | Description | Troubleshooting |
|------------|-------------|-----------------|
| `FORBIDDEN` | Authenticated but not authorized for this resource | Check user permissions and resource ownership |

**Example:**
```json
{
  "error": {
    "code": "FORBIDDEN",
    "message": "Access denied"
  }
}
```

**Common Causes:**
- User doesn't have permission for the action
- Accessing another tenant's resources
- Insufficient role/privileges

**Resolution:**
1. Verify user has appropriate role (Admin, User, etc.)
2. Check resource belongs to user's tenant
3. Request access from tenant administrator

### 404 Not Found

**Status Code:** `404`

| Error Code | Description | Troubleshooting |
|------------|-------------|-----------------|
| `NOT_FOUND` | Requested resource doesn't exist | Verify resource ID and that resource wasn't deleted |

**Example:**
```json
{
  "error": {
    "code": "NOT_FOUND",
    "message": "Resource not found: Project with ID 550e8400-e29b-41d4-a716-446655440000"
  }
}
```

**Common Causes:**
- Resource ID doesn't exist
- Resource was deleted
- Typo in resource ID
- Accessing resource from wrong tenant

**Resolution:**
1. Verify resource ID is correct
2. Check resource exists via list endpoint
3. Confirm resource wasn't deleted
4. Ensure querying correct tenant

### 409 Conflict

**Status Code:** `409`

| Error Code | Description | Troubleshooting |
|------------|-------------|-----------------|
| `CONFLICT` | Resource already exists or state conflict | Check for existing resources or resolve state issues |

**Example:**
```json
{
  "error": {
    "code": "CONFLICT",
    "message": "Conflict: Project with name 'my-app' already exists",
    "details": {
      "field": "name",
      "value": "my-app"
    }
  }
}
```

**Common Causes:**
- Duplicate resource (unique constraint violation)
- Invalid state transition
- Concurrent modification

**Resolution:**
1. Use unique names for resources
2. Check if resource already exists
3. Update existing resource instead of creating
4. Retry operation if due to race condition

### 422 Unprocessable Entity

**Status Code:** `422`

| Error Code | Description | Troubleshooting |
|------------|-------------|-----------------|
| `VALIDATION_ERROR` | Request validation failed | Fix validation errors in request data |

**Example:**
```json
{
  "error": {
    "code": "VALIDATION_ERROR",
    "message": "Validation error: Email address is invalid",
    "details": {
      "field": "email",
      "value": "not-an-email",
      "constraint": "must be valid email format"
    }
  }
}
```

**Common Causes:**
- Invalid email format
- String too long/short
- Number out of range
- Invalid enum value

**Resolution:**
1. Check field constraints in API documentation
2. Validate input on client side
3. Fix invalid values
4. Ensure enums match allowed values

### 429 Too Many Requests

**Status Code:** `429`

| Error Code | Description | Troubleshooting |
|------------|-------------|-----------------|
| `RATE_LIMITED` | Rate limit exceeded | Wait before retrying, implement backoff |

**Example:**
```json
{
  "error": {
    "code": "RATE_LIMITED",
    "message": "Rate limit exceeded",
    "details": {
      "retry_after_seconds": 60,
      "limit": "100 requests per minute"
    }
  }
}
```

**Common Causes:**
- Too many requests in short time period
- Upstream registry rate limiting
- API key rate limits

**Resolution:**
1. Implement exponential backoff
2. Respect `Retry-After` header
3. Cache results to reduce requests
4. Contact support for rate limit increase

### 500 Internal Server Error

**Status Code:** `500`

| Error Code | Description | Troubleshooting |
|------------|-------------|-----------------|
| `INTERNAL_ERROR` | Unexpected server error | Report bug with request details |

**Example:**
```json
{
  "error": {
    "code": "INTERNAL_ERROR",
    "message": "Internal server error: Database connection failed"
  }
}
```

**Common Causes:**
- Database connection failure
- Unhandled exception
- Service misconfiguration
- Bug in server code

**Resolution:**
1. Check server logs for stack trace
2. Verify database is accessible
3. Retry request (may be transient)
4. Report issue with reproduction steps

### 503 Service Unavailable

**Status Code:** `503`

| Error Code | Description | Troubleshooting |
|------------|-------------|-----------------|
| `SERVICE_UNAVAILABLE` | Service temporarily unavailable | Retry after delay, check service health |

**Example:**
```json
{
  "error": {
    "code": "SERVICE_UNAVAILABLE",
    "message": "Service unavailable: Worker queue is full"
  }
}
```

**Common Causes:**
- Service overloaded
- Dependency service down
- Maintenance mode
- Resource exhaustion

**Resolution:**
1. Check `/health` endpoint
2. Retry with exponential backoff
3. Check service status page
4. Wait for maintenance to complete

## Domain Error Codes

Errors related to business logic and domain validation.

### Repository Errors

| Error Code | HTTP Status | Description |
|------------|-------------|-------------|
| `ENTITY_NOT_FOUND` | 404 | Database entity doesn't exist |
| `ENTITY_ALREADY_EXISTS` | 409 | Unique constraint violation |
| `DATABASE_ERROR` | 500 | Database operation failed |
| `SERIALIZATION_ERROR` | 500 | Data serialization failed |
| `INVALID_DATA` | 422 | Data doesn't meet constraints |

**Example:**
```rust
// From sctv_core::traits::RepositoryError
RepositoryError::NotFound
RepositoryError::AlreadyExists
RepositoryError::Database("Connection refused")
RepositoryError::Serialization("Invalid JSON")
RepositoryError::InvalidData("Email must be valid")
```

## Database Error Codes

Errors from PostgreSQL database operations.

### Common Database Errors

| SQLState | Error | Description | Resolution |
|----------|-------|-------------|------------|
| `08006` | Connection Failure | Cannot connect to database | Check `DATABASE_URL` and network |
| `23505` | Unique Violation | Duplicate key value | Use unique identifiers |
| `23503` | Foreign Key Violation | Referenced entity doesn't exist | Create referenced entity first |
| `42P01` | Undefined Table | Table doesn't exist | Run database migrations |
| `42703` | Undefined Column | Column doesn't exist | Check schema version |

**Example Error:**
```
Database error: duplicate key value violates unique constraint "projects_name_tenant_id_key"
```

**Resolution:**
1. Check for existing resources
2. Use unique names within tenant
3. Update instead of insert if exists

### Migration Errors

| Error | Description | Resolution |
|-------|-------------|------------|
| `Migration not found` | Migration file missing | Ensure migrations directory is complete |
| `Checksum mismatch` | Migration was modified | Revert changes or create new migration |
| `Version conflict` | Migration already applied | Check migration status with `sqlx migrate info` |

## Detection Error Codes

Errors from threat detection engines.

### Detector Errors

| Error Code | Description | Troubleshooting |
|------------|-------------|-----------------|
| `ANALYSIS_FAILED` | Threat analysis failed | Check detector configuration and input data |
| `DETECTOR_CONFIGURATION` | Detector misconfigured | Verify detector settings |
| `DATA_UNAVAILABLE` | Required data not available | Ensure package metadata is fetched |

**Example:**
```rust
// From sctv_detectors::traits::DetectorError
DetectorError::AnalysisFailed("Sigstore verification failed")
DetectorError::Configuration("Missing API key")
DetectorError::DataUnavailable("Package not found in registry")
```

**Common Scenarios:**

**Typosquatting Detection:**
- Requires package name and ecosystem
- May fail if comparison package not in database

**Provenance Verification:**
- Requires attestation data
- Sigstore validation may fail if attestation malformed

**Downgrade Detection:**
- Requires version history
- Needs semantic version parsing

## Registry Client Errors

Errors from package registry interactions.

### Registry Error Codes

| Error Code | Description | Troubleshooting |
|------------|-------------|-----------------|
| `PACKAGE_NOT_FOUND` | Package doesn't exist in registry | Verify package name and ecosystem |
| `VERSION_NOT_FOUND` | Package version doesn't exist | Check version string is valid |
| `HTTP_ERROR` | HTTP request failed | Check network and registry availability |
| `PARSE_ERROR` | Failed to parse registry response | Registry may have changed API format |
| `RATE_LIMITED` | Registry rate limit exceeded | Implement backoff or provide auth token |
| `REGISTRY_UNAVAILABLE` | Registry service is down | Wait and retry with exponential backoff |
| `CACHE_ERROR` | Cache operation failed | Clear cache or check memory |

**Example:**
```rust
// From sctv_registries::RegistryError
RegistryError::PackageNotFound("nonexistent-package")
RegistryError::VersionNotFound("lodash", "99.99.99")
RegistryError::Http(reqwest::Error)
RegistryError::Parse("Invalid JSON response")
RegistryError::RateLimited
RegistryError::Unavailable("Registry returned 503")
RegistryError::Cache("Eviction failed")
```

### Per-Ecosystem Errors

**npm:**
- 404: Package or scoped package not found
- Check scoped packages use format `@scope/package`

**PyPI:**
- Normalizes names (hyphens, underscores, dots)
- Search with normalized name

**Maven:**
- Requires full coordinate: `groupId:artifactId`
- 404 may indicate wrong repository

**NuGet:**
- Case-insensitive but preserves original casing
- Service index may be cached

**RubyGems:**
- Version format differences (uses gem version, not strict semver)

**Cargo (crates.io):**
- Requires exact package name (case-sensitive)
- User agent required

**Go Modules:**
- Module path must be exact (case-sensitive)
- Uppercase letters encoded as `!lowercase`

## Notification Errors

Errors from notification delivery system.

### Notification Error Codes

| Error Code | Description | Troubleshooting |
|------------|-------------|-----------------|
| `EMAIL_DELIVERY_FAILED` | Email couldn't be sent | Check SMTP configuration |
| `SMTP_TRANSPORT_ERROR` | SMTP connection failed | Verify SMTP host and credentials |
| `WEBHOOK_DELIVERY_FAILED` | Webhook POST failed | Check webhook URL and endpoint |
| `INVALID_WEBHOOK_URL` | Webhook URL is invalid | Verify URL format |
| `HTTP_REQUEST_FAILED` | HTTP request error | Check network and endpoint |
| `SERIALIZATION_ERROR` | Payload serialization failed | Check notification data is valid |
| `INVALID_CONFIG` | Channel config is invalid | Verify channel configuration |
| `CHANNEL_DISABLED` | Notification channel disabled | Enable channel in settings |
| `RATE_LIMITED` | Notification rate limit hit | Reduce notification frequency |
| `AUTHENTICATION_FAILED` | Auth to service failed | Check API keys and credentials |
| `TIMEOUT` | Request timed out | Increase timeout or check endpoint |

**Example:**
```rust
// From sctv_notifications::NotificationError
NotificationError::EmailDelivery("SMTP connection refused")
NotificationError::WebhookDelivery("POST to https://hooks.slack.com failed")
NotificationError::RateLimited { retry_after_secs: 60 }
NotificationError::Timeout { timeout_secs: 30 }
```

**Email Troubleshooting:**
1. Verify SMTP_HOST and SMTP_PORT
2. Check SMTP_USERNAME and SMTP_PASSWORD
3. Ensure SMTP_TLS setting matches server
4. Test with: `telnet $SMTP_HOST $SMTP_PORT`

**Webhook Troubleshooting:**
1. Verify webhook URL is accessible
2. Check webhook endpoint accepts POST
3. Verify content-type is application/json
4. Check for firewall blocking outbound requests

## Worker/Job Errors

Errors from background job processing.

### Worker Error Codes

| Error Code | Description | Troubleshooting |
|------------|-------------|-----------------|
| `DATABASE_ERROR` | Database operation failed | Check database connectivity |
| `JOB_NOT_FOUND` | Job ID doesn't exist | Verify job was created |
| `INVALID_JOB_STATUS` | Invalid status transition | Check job lifecycle |
| `INVALID_JOB_TYPE` | Unknown job type | Verify job type is registered |
| `INVALID_JOB_PRIORITY` | Priority out of range | Use priority 0-10 |
| `SERIALIZATION_ERROR` | Job payload serialization failed | Check job data is serializable |
| `EXECUTION_ERROR` | Job execution failed | Check job logs for details |
| `POOL_ERROR` | Worker pool error | Check worker configuration |
| `SHUTDOWN` | Queue is shutting down | Wait for shutdown to complete |
| `TIMEOUT` | Job execution timeout | Increase timeout or optimize job |
| `CONFIGURATION_ERROR` | Worker misconfigured | Check worker environment variables |
| `REGISTRY_ERROR` | Registry client error | See Registry Client Errors |
| `DETECTOR_ERROR` | Detection engine error | See Detection Error Codes |
| `NOTIFICATION_ERROR` | Notification delivery failed | See Notification Errors |

**Example:**
```rust
// From sctv_worker::WorkerError
WorkerError::JobNotFound("job-123")
WorkerError::InvalidJobStatus("Cannot transition from Complete to Pending")
WorkerError::Execution("Scan failed: timeout")
WorkerError::Shutdown
```

### Job Lifecycle

Valid job status transitions:

```
Pending → Running → Completed
         ↓
         Failed
         ↓
         Retrying → Running
```

**Invalid Transitions:**
- Completed → Running
- Failed → Completed (must retry)

## Troubleshooting Guide

### Quick Diagnosis

**Step 1: Check HTTP Status Code**
- 4xx: Client error (check request)
- 5xx: Server error (check logs)

**Step 2: Read Error Code**
- Identifies specific error type
- Points to relevant documentation

**Step 3: Check Error Message**
- Human-readable description
- Often contains specific entity IDs

**Step 4: Review Details Object**
- Additional context
- Field-specific errors

### Common Issues and Solutions

**Issue: All API requests return 401**

**Diagnosis:** Authentication problem

**Solutions:**
1. Check `Authorization` header is present
2. Verify JWT token is valid: `jwt.io`
3. Ensure token hasn't expired
4. Verify `SCTV_JWT_SECRET` matches

**Issue: Package not found errors**

**Diagnosis:** Registry client issue

**Solutions:**
1. Verify package name spelling
2. Check ecosystem is correct
3. Test package exists: Visit registry website
4. For scoped packages (npm), use `@scope/name` format

**Issue: Jobs stuck in Pending**

**Diagnosis:** Worker not processing jobs

**Solutions:**
1. Check worker is running: `docker-compose ps worker`
2. Verify worker can connect to database
3. Check worker logs: `docker-compose logs worker`
4. Increase `SCTV_WORKER_COUNT` if overloaded

**Issue: Database connection errors**

**Diagnosis:** Database connectivity

**Solutions:**
1. Verify PostgreSQL is running
2. Check `DATABASE_URL` is correct
3. Test connection: `psql $DATABASE_URL`
4. Check network/firewall rules

**Issue: Rate limit errors**

**Diagnosis:** Too many requests

**Solutions:**
1. Implement exponential backoff
2. Cache results
3. Provide GitHub token for higher limits
4. Reduce request frequency

### Debugging Workflow

1. **Enable Debug Logging**
   ```bash
   RUST_LOG=debug docker-compose restart api worker
   ```

2. **Check Service Logs**
   ```bash
   docker-compose logs -f api
   docker-compose logs -f worker
   ```

3. **Verify Configuration**
   ```bash
   docker-compose config
   ```

4. **Test Database Connection**
   ```bash
   docker-compose exec postgres psql -U sctv -d sctv
   ```

5. **Check Service Health**
   ```bash
   curl http://localhost:3000/health
   ```

### Getting Help

When reporting errors, include:

1. **Error Details**
   - Full error response JSON
   - HTTP status code
   - Request URL and method

2. **Environment**
   - SCTV version
   - Deployment method (Docker, native)
   - Operating system

3. **Reproduction Steps**
   - Minimal steps to reproduce
   - Sample request data
   - Expected vs actual behavior

4. **Logs**
   - Relevant log excerpts
   - Stack traces if available
   - Timestamps

## See Also

- [API Reference](./api.md)
- [Configuration Reference](./configuration.md)
- [Troubleshooting Guide](../troubleshooting.md)
- [Contributing - Reporting Issues](../development/contributing.md#reporting-issues)
