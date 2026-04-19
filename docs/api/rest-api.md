# REST API Reference

**Version:** 0.1.0
**Base URL:** `http://localhost:3000/api/v1`

Complete reference for the SCTV REST API.

---

## Table of Contents

- [Authentication](#authentication)
- [Response Format](#response-format)
- [Error Handling](#error-handling)
- [Pagination](#pagination)
- [Rate Limiting](#rate-limiting) (planned, not yet implemented)
- [Projects API](#projects-api)
- [Dependencies API](#dependencies-api)
- [Alerts API](#alerts-api)
- [Policies API](#policies-api)
- [Scans API](#scans-api)
- [Webhooks API](#webhooks-api)

---

## Authentication

SCTV supports two authentication methods:

### 1. JWT Bearer Token

Obtain a JWT token via login, then include it in requests:

```http
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
```

**Get a token:**

```http
POST /api/v1/auth/login
Content-Type: application/json

{
  "email": "user@example.com",
  "password": "your-password"
}
```

**Response:**

```json
{
  "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "expires_at": "2026-01-16T10:30:00Z",
  "user": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "email": "user@example.com",
    "name": "John Doe",
    "role": "admin"
  }
}
```

### 2. API Key

Generate an API key for programmatic access:

```http
X-API-Key: sctv_live_abc123xyz456...
```

**Create an API key:**

```http
POST /api/v1/auth/api-keys
Authorization: Bearer <your-jwt-token>
Content-Type: application/json

{
  "name": "CI/CD Pipeline",
  "expires_in_days": 365
}
```

**Response:**

```json
{
  "id": "660e8400-e29b-41d4-a716-446655440001",
  "key": "sctv_live_abc123xyz456...",
  "name": "CI/CD Pipeline",
  "created_at": "2026-01-15T10:30:00Z",
  "expires_at": "2027-01-15T10:30:00Z"
}
```

**Note:** Save the key immediately - it's only shown once!

---

## Response Format

All responses are JSON with this structure:

### Success Response

```json
{
  "data": {
    // Response payload
  },
  "meta": {
    "timestamp": "2026-01-15T10:30:00Z",
    "request_id": "req_abc123"
  }
}
```

### Error Response

```json
{
  "error": {
    "code": "VALIDATION_ERROR",
    "message": "Invalid request parameters",
    "details": {
      "field": "name",
      "issue": "Field is required"
    }
  },
  "meta": {
    "timestamp": "2026-01-15T10:30:00Z",
    "request_id": "req_abc123"
  }
}
```

---

## Error Handling

### HTTP Status Codes

| Status | Meaning | When Used |
|--------|---------|-----------|
| 200 | OK | Successful request |
| 201 | Created | Resource created successfully |
| 204 | No Content | Successful deletion |
| 400 | Bad Request | Invalid input |
| 401 | Unauthorized | Missing/invalid authentication |
| 403 | Forbidden | Insufficient permissions |
| 404 | Not Found | Resource doesn't exist |
| 409 | Conflict | Resource already exists |
| 422 | Unprocessable Entity | Validation failed |
| 429 | Too Many Requests | Rate limit exceeded (planned; not yet emitted by the server) |
| 500 | Internal Server Error | Server error |
| 503 | Service Unavailable | Service temporarily down |

### Error Codes

| Code | Description |
|------|-------------|
| `AUTHENTICATION_FAILED` | Invalid credentials |
| `UNAUTHORIZED` | Missing authentication |
| `FORBIDDEN` | Insufficient permissions |
| `NOT_FOUND` | Resource not found |
| `VALIDATION_ERROR` | Input validation failed |
| `CONFLICT` | Resource conflict |
| `RATE_LIMIT_EXCEEDED` | Too many requests (planned; not yet emitted by the server) |
| `INTERNAL_ERROR` | Server error |

---

## Pagination

List endpoints support pagination using cursor-based pagination:

### Request Parameters

```http
GET /api/v1/projects?limit=20&cursor=eyJpZCI6IjU1MGU4NDAwIn0
```

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `limit` | integer | 20 | Number of items (max: 100) |
| `cursor` | string | - | Pagination cursor |

### Response Format

```json
{
  "data": [...],
  "pagination": {
    "has_next": true,
    "next_cursor": "eyJpZCI6IjY2MGU4NDAwIn0",
    "total": 147
  }
}
```

---

## Rate Limiting

**Status: Not yet implemented — future work.**

The SCTV API server does not currently enforce rate limits. The `429 Too
Many Requests` status code and `RATE_LIMIT_EXCEEDED` error code are
reserved for a future release. Clients should still implement sensible
client-side throttling and exponential backoff to avoid overwhelming the
server and any upstream package registries.

Rate-limit response headers (`X-RateLimit-Limit`, `X-RateLimit-Remaining`,
`X-RateLimit-Reset`) and tiered quotas are planned but not currently
returned.

---

## Projects API

### List Projects

```http
GET /api/v1/projects
```

**Query Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `limit` | integer | Number of results (default: 20) |
| `cursor` | string | Pagination cursor |
| `status` | string | Filter by status: `healthy`, `at_risk`, `vulnerable` |
| `ecosystem` | string | Filter by ecosystem |

**Response:**

```json
{
  "data": [
    {
      "id": "550e8400-e29b-41d4-a716-446655440000",
      "name": "My Application",
      "description": "Production web app",
      "repository_url": "https://github.com/org/repo",
      "ecosystems": ["npm", "pypi"],
      "status": "healthy",
      "last_scan_at": "2026-01-15T08:00:00Z",
      "alert_counts": {
        "critical": 0,
        "high": 1,
        "medium": 5,
        "low": 12
      },
      "created_at": "2026-01-01T00:00:00Z",
      "updated_at": "2026-01-15T10:30:00Z"
    }
  ],
  "pagination": {
    "has_next": true,
    "next_cursor": "eyJpZCI6IjY2MGU4NDAwIn0",
    "total": 42
  }
}
```

### Create Project

```http
POST /api/v1/projects
Content-Type: application/json
```

**Request Body:**

```json
{
  "name": "My Application",
  "description": "Production web application",
  "repository_url": "https://github.com/org/repo",
  "default_branch": "main",
  "ecosystems": ["npm", "pypi"],
  "scan_schedule": {
    "type": "daily",
    "hour": 2
  },
  "policy_id": "660e8400-e29b-41d4-a716-446655440001"
}
```

**Response:** `201 Created`

```json
{
  "data": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "name": "My Application",
    // ... full project object
  }
}
```

### Get Project

```http
GET /api/v1/projects/{id}
```

**Response:** `200 OK`

```json
{
  "data": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "name": "My Application",
    "description": "Production web application",
    "repository_url": "https://github.com/org/repo",
    "default_branch": "main",
    "ecosystems": ["npm", "pypi"],
    "scan_schedule": {
      "type": "daily",
      "hour": 2
    },
    "policy_id": "660e8400-e29b-41d4-a716-446655440001",
    "last_scan_at": "2026-01-15T08:00:00Z",
    "status": "healthy",
    "dependency_count": 847,
    "alert_counts": {
      "critical": 0,
      "high": 1,
      "medium": 5,
      "low": 12,
      "info": 8
    },
    "created_at": "2026-01-01T00:00:00Z",
    "updated_at": "2026-01-15T10:30:00Z"
  }
}
```

### Update Project

```http
PUT /api/v1/projects/{id}
Content-Type: application/json
```

**Request Body:**

```json
{
  "name": "My Application (Updated)",
  "description": "Updated description",
  "scan_schedule": {
    "type": "hourly"
  }
}
```

**Response:** `200 OK`

### Delete Project

```http
DELETE /api/v1/projects/{id}
```

**Response:** `204 No Content`

### Trigger Project Scan

```http
POST /api/v1/projects/{id}/scan
```

**Response:** `202 Accepted`

```json
{
  "data": {
    "job_id": "770e8400-e29b-41d4-a716-446655440002",
    "status": "pending",
    "created_at": "2026-01-15T10:30:00Z"
  }
}
```

### List Project Dependencies

```http
GET /api/v1/projects/{id}/dependencies
```

**Query Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `ecosystem` | string | Filter by ecosystem |
| `direct_only` | boolean | Show only direct dependencies |
| `include_dev` | boolean | Include dev dependencies |

**Response:**

```json
{
  "data": [
    {
      "id": "880e8400-e29b-41d4-a716-446655440003",
      "package_name": "lodash",
      "ecosystem": "npm",
      "version_constraint": "^4.17.0",
      "resolved_version": "4.17.21",
      "is_direct": true,
      "is_dev_dependency": false,
      "depth": 0,
      "hash_sha256": "abc123...",
      "signature_status": "verified",
      "provenance_status": "verified",
      "alert_count": 0,
      "last_verified_at": "2026-01-15T08:00:00Z"
    }
  ],
  "pagination": {
    "total": 847
  }
}
```

---

## Dependencies API

### Get Dependency

```http
GET /api/v1/dependencies/{id}
```

**Response:**

```json
{
  "data": {
    "id": "880e8400-e29b-41d4-a716-446655440003",
    "project_id": "550e8400-e29b-41d4-a716-446655440000",
    "package_name": "lodash",
    "ecosystem": "npm",
    "version_constraint": "^4.17.0",
    "resolved_version": "4.17.21",
    "is_direct": true,
    "is_dev_dependency": false,
    "depth": 0,
    "parent_id": null,
    "hash_sha256": "abc123...",
    "hash_sha512": "def456...",
    "signature_status": "verified",
    "provenance_status": "verified",
    "provenance_details": {
      "slsa_level": 2,
      "builder": {
        "id": "https://github.com/actions"
      }
    },
    "alerts": [],
    "first_seen_at": "2026-01-01T00:00:00Z",
    "last_verified_at": "2026-01-15T08:00:00Z"
  }
}
```

### Verify Dependency

Manually trigger integrity verification for a dependency:

```http
POST /api/v1/dependencies/{id}/verify
```

**Response:** `202 Accepted`

```json
{
  "data": {
    "job_id": "990e8400-e29b-41d4-a716-446655440004",
    "status": "pending"
  }
}
```

---

## Alerts API

### List Alerts

```http
GET /api/v1/alerts
```

**Query Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `project_id` | UUID | Filter by project |
| `severity` | string | Filter by severity (comma-separated) |
| `status` | string | Filter by status |
| `type` | string | Filter by alert type |
| `since` | ISO8601 | Alerts created after this time |

**Example:**

```http
GET /api/v1/alerts?severity=critical,high&status=open&since=2026-01-01T00:00:00Z
```

**Response:**

```json
{
  "data": [
    {
      "id": "aa0e8400-e29b-41d4-a716-446655440005",
      "project_id": "550e8400-e29b-41d4-a716-446655440000",
      "project_name": "My Application",
      "dependency_id": "880e8400-e29b-41d4-a716-446655440003",
      "alert_type": "typosquatting",
      "severity": "critical",
      "status": "open",
      "title": "Typosquatting Detected: lodash-utils",
      "description": "Package 'lodash-utils' is suspiciously similar to 'lodash'",
      "details": {
        "suspicious_package": "lodash-utils",
        "target_package": "lodash",
        "similarity_score": 0.92,
        "method": "levenshtein"
      },
      "remediation": {
        "steps": [
          "Remove 'lodash-utils' from dependencies",
          "Use official 'lodash' package instead",
          "Review code for potential compromise"
        ],
        "automated": false
      },
      "created_at": "2026-01-15T10:00:00Z",
      "acknowledged_at": null,
      "resolved_at": null
    }
  ],
  "pagination": {
    "total": 23
  }
}
```

### Get Alert

```http
GET /api/v1/alerts/{id}
```

**Response:** Full alert details including complete remediation steps.

### Acknowledge Alert

Mark an alert as acknowledged:

```http
POST /api/v1/alerts/{id}/acknowledge
Content-Type: application/json
```

**Request Body:**

```json
{
  "comment": "Security team investigating"
}
```

**Response:** `200 OK`

```json
{
  "data": {
    "id": "aa0e8400-e29b-41d4-a716-446655440005",
    "status": "acknowledged",
    "acknowledged_at": "2026-01-15T10:30:00Z",
    "acknowledged_by": {
      "id": "bb0e8400-e29b-41d4-a716-446655440006",
      "name": "John Doe",
      "email": "john@example.com"
    }
  }
}
```

### Resolve Alert

Mark an alert as resolved:

```http
POST /api/v1/alerts/{id}/resolve
Content-Type: application/json
```

**Request Body:**

```json
{
  "resolution": "Removed malicious dependency and updated to official package",
  "comment": "Issue fixed in commit abc123"
}
```

**Response:** `200 OK`

### Suppress Alert

Suppress a false positive:

```http
POST /api/v1/alerts/{id}/suppress
Content-Type: application/json
```

**Request Body:**

```json
{
  "reason": "Internally maintained package, not a typosquat",
  "suppress_similar": true
}
```

**Response:** `200 OK`

---

## Policies API

### List Policies

```http
GET /api/v1/policies
```

**Response:**

```json
{
  "data": [
    {
      "id": "cc0e8400-e29b-41d4-a716-446655440007",
      "name": "Production Security Policy",
      "description": "Security requirements for production deployments",
      "is_default": true,
      "enabled": true,
      "rule_count": 7,
      "project_count": 12,
      "created_at": "2026-01-01T00:00:00Z",
      "updated_at": "2026-01-10T15:00:00Z"
    }
  ]
}
```

### Create Policy

```http
POST /api/v1/policies
Content-Type: application/json
```

**Request Body:**

```json
{
  "name": "Strict Security Policy",
  "description": "Maximum security requirements",
  "rules": [
    {
      "type": "BlockDeprecated",
      "severity": "high"
    },
    {
      "type": "RequireProvenance",
      "min_slsa_level": 3,
      "apply_to": "direct"
    },
    {
      "type": "BlockPackageAge",
      "min_age_days": 90,
      "severity": "medium",
      "exemptions": ["@myorg/*"]
    },
    {
      "type": "RequireSignatures",
      "severity": "high"
    }
  ],
  "enabled": true
}
```

**Response:** `201 Created`

### Get Policy

```http
GET /api/v1/policies/{id}
```

**Response:**

```json
{
  "data": {
    "id": "cc0e8400-e29b-41d4-a716-446655440007",
    "name": "Production Security Policy",
    "description": "Security requirements for production deployments",
    "rules": [
      {
        "type": "BlockDeprecated",
        "severity": "high"
      },
      {
        "type": "RequireProvenance",
        "min_slsa_level": 2,
        "apply_to": "direct"
      }
    ],
    "severity_overrides": [
      {
        "package_pattern": "@myorg/*",
        "override_severity": "low"
      }
    ],
    "is_default": true,
    "enabled": true,
    "created_at": "2026-01-01T00:00:00Z",
    "updated_at": "2026-01-10T15:00:00Z"
  }
}
```

### Update Policy

```http
PUT /api/v1/policies/{id}
Content-Type: application/json
```

**Request Body:** Same as create.

**Response:** `200 OK`

### Delete Policy

```http
DELETE /api/v1/policies/{id}
```

**Response:** `204 No Content`

---

## Scans API

### List Scans

```http
GET /api/v1/scans?project_id={project_id}
```

**Response:**

```json
{
  "data": [
    {
      "id": "dd0e8400-e29b-41d4-a716-446655440008",
      "project_id": "550e8400-e29b-41d4-a716-446655440000",
      "status": "completed",
      "dependencies_found": 847,
      "alerts_created": 23,
      "duration_seconds": 18.4,
      "started_at": "2026-01-15T08:00:00Z",
      "completed_at": "2026-01-15T08:00:18Z"
    }
  ]
}
```

### Get Scan

```http
GET /api/v1/scans/{id}
```

**Response:**

```json
{
  "data": {
    "id": "dd0e8400-e29b-41d4-a716-446655440008",
    "project_id": "550e8400-e29b-41d4-a716-446655440000",
    "status": "completed",
    "dependencies_found": 847,
    "direct_dependencies": 143,
    "transitive_dependencies": 704,
    "alerts_created": 23,
    "alerts_by_severity": {
      "critical": 1,
      "high": 3,
      "medium": 7,
      "low": 12
    },
    "duration_seconds": 18.4,
    "started_at": "2026-01-15T08:00:00Z",
    "completed_at": "2026-01-15T08:00:18Z"
  }
}
```

---

## Webhooks API

### GitHub Webhook

Receive push events from GitHub:

```http
POST /api/v1/webhooks/github
X-GitHub-Event: push
X-Hub-Signature-256: sha256=...
Content-Type: application/json
```

**Request Body:** GitHub webhook payload

**Response:** `202 Accepted`

### GitLab Webhook

Receive push events from GitLab:

```http
POST /api/v1/webhooks/gitlab
X-Gitlab-Event: Push Hook
X-Gitlab-Token: your-secret-token
Content-Type: application/json
```

**Request Body:** GitLab webhook payload

**Response:** `202 Accepted`

---

## Next Steps

- [GraphQL API](graphql-api.md) - Alternative GraphQL interface
- [Authentication Guide](authentication.md) - Detailed auth documentation
- [Webhooks Guide](webhooks.md) - CI/CD integration setup
