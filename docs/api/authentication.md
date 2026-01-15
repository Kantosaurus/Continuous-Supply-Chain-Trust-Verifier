# Authentication Documentation

**Version:** 0.1.0

Complete authentication and authorization guide for the SCTV API.

---

## Table of Contents

- [Overview](#overview)
- [Authentication Methods](#authentication-methods)
- [JWT Authentication](#jwt-authentication)
- [API Key Authentication](#api-key-authentication)
- [Token Lifecycle](#token-lifecycle)
- [OAuth2 Integration](#oauth2-integration)
- [Role-Based Access Control](#role-based-access-control)
- [Multi-Tenant Authentication](#multi-tenant-authentication)
- [Security Best Practices](#security-best-practices)
- [Code Examples](#code-examples)
- [Troubleshooting](#troubleshooting)

---

## Overview

SCTV uses a multi-layered authentication system to secure API access:

1. **JWT Bearer Tokens**: Primary method for user authentication
2. **API Keys**: Service-to-service and CI/CD authentication
3. **OAuth2**: Third-party authentication (planned)
4. **Multi-tenant Isolation**: Automatic tenant scoping for all operations

### Authentication Flow

```
┌─────────┐                ┌─────────┐                ┌──────────┐
│ Client  │                │ API     │                │ Database │
└────┬────┘                └────┬────┘                └────┬─────┘
     │                          │                          │
     │  1. Login Request        │                          │
     ├─────────────────────────>│                          │
     │                          │                          │
     │                          │  2. Verify Credentials   │
     │                          ├─────────────────────────>│
     │                          │                          │
     │                          │  3. Return User Data     │
     │                          │<─────────────────────────┤
     │                          │                          │
     │  4. JWT Token            │                          │
     │<─────────────────────────┤                          │
     │                          │                          │
     │  5. Authenticated Request│                          │
     │  (Bearer Token)          │                          │
     ├─────────────────────────>│                          │
     │                          │                          │
     │                          │  6. Validate Token       │
     │                          │  7. Extract Claims       │
     │                          │  8. Check Tenant Access  │
     │                          │                          │
     │  9. Response             │                          │
     │<─────────────────────────┤                          │
```

---

## Authentication Methods

### Method Comparison

| Feature | JWT Token | API Key |
|---------|-----------|---------|
| **Use Case** | User sessions | Service accounts, CI/CD |
| **Lifetime** | Short (24 hours) | Long (365+ days) |
| **Refresh** | Yes (refresh tokens) | No (regenerate) |
| **Revocation** | Blacklist | Database deletion |
| **Permissions** | User roles | Scoped permissions |
| **Multi-tenant** | Yes | Yes |
| **Header** | `Authorization: Bearer <token>` | `X-API-Key: <key>` |

---

## JWT Authentication

### Token Structure

SCTV uses JSON Web Tokens (JWT) with the following structure:

```
eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiI1NTBlODQwMC1lMjliLTQxZDQtYTcxNi00NDY2NTU0NDAwMDAiLCJ0ZW5hbnRfaWQiOiI2NjBlODQwMC1lMjliLTQxZDQtYTcxNi00NDY2NTU0NDAwMDEiLCJlbWFpbCI6InVzZXJAZXhhbXBsZS5jb20iLCJyb2xlcyI6WyJ1c2VyIl0sImlhdCI6MTcwNTMxNTIwMCwiZXhwIjoxNzA1NDAxNjAwLCJpc3MiOiJzY3R2LWFwaSIsImF1ZCI6InNjdHYifQ.signature
```

**Structure:**
- **Header**: Algorithm and token type
- **Payload**: Claims (user data, tenant, roles)
- **Signature**: HMAC-SHA256 signature

### JWT Claims

```rust
pub struct Claims {
    pub sub: Uuid,              // Subject (user ID)
    pub tenant_id: Uuid,        // Tenant ID
    pub email: String,          // User email
    pub roles: Vec<String>,     // User roles
    pub iat: i64,               // Issued at (timestamp)
    pub exp: i64,               // Expiration (timestamp)
    pub iss: String,            // Issuer (sctv-api)
    pub aud: String,            // Audience (sctv)
}
```

### Obtaining a Token

#### Login Endpoint

```http
POST /api/v1/auth/login
Content-Type: application/json

{
  "email": "user@example.com",
  "password": "your-secure-password"
}
```

**Response:**

```json
{
  "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "refresh_token": "refresh_abc123xyz456...",
  "expires_at": "2026-01-16T10:30:00Z",
  "user": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "tenant_id": "660e8400-e29b-41d4-a716-446655440001",
    "email": "user@example.com",
    "name": "John Doe",
    "roles": ["user", "admin"]
  }
}
```

### Using JWT Tokens

Include the token in the `Authorization` header:

```http
GET /api/v1/projects
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
```

### Token Validation Process

1. **Extract Token**: Parse from `Authorization: Bearer <token>` header
2. **Verify Signature**: Validate HMAC signature using server secret
3. **Check Expiration**: Ensure token is not expired (`exp` claim)
4. **Verify Issuer**: Check `iss` matches "sctv-api"
5. **Verify Audience**: Check `aud` matches "sctv"
6. **Extract Claims**: Parse user ID, tenant ID, roles

### Token Expiration

Default JWT expiration: **24 hours**

Configure via environment:

```bash
SCTV_JWT_EXPIRATION_HOURS=24
```

---

## API Key Authentication

API keys provide long-lived authentication for services, CI/CD pipelines, and integrations.

### Creating an API Key

```http
POST /api/v1/auth/api-keys
Authorization: Bearer <your-jwt-token>
Content-Type: application/json

{
  "name": "CI/CD Pipeline",
  "scopes": ["read:projects", "write:scans"],
  "expires_in_days": 365
}
```

**Response:**

```json
{
  "id": "770e8400-e29b-41d4-a716-446655440002",
  "key": "sctv_live_1a2b3c4d5e6f7g8h9i0j",
  "name": "CI/CD Pipeline",
  "scopes": ["read:projects", "write:scans"],
  "tenant_id": "660e8400-e29b-41d4-a716-446655440001",
  "created_at": "2026-01-15T10:30:00Z",
  "expires_at": "2027-01-15T10:30:00Z",
  "last_used_at": null
}
```

**Important:** The API key is only shown once. Store it securely!

### Using API Keys

Include the key in the `X-API-Key` header:

```http
GET /api/v1/projects
X-API-Key: sctv_live_1a2b3c4d5e6f7g8h9i0j
```

### API Key Scopes

Define granular permissions for API keys:

| Scope | Description |
|-------|-------------|
| `read:projects` | Read project data |
| `write:projects` | Create/update projects |
| `read:dependencies` | Read dependency data |
| `write:scans` | Trigger scans |
| `read:alerts` | Read alerts |
| `write:alerts` | Acknowledge/resolve alerts |
| `read:policies` | Read policies |
| `write:policies` | Create/update policies |
| `admin` | Full access |

### Managing API Keys

#### List API Keys

```http
GET /api/v1/auth/api-keys
Authorization: Bearer <your-jwt-token>
```

#### Revoke API Key

```http
DELETE /api/v1/auth/api-keys/{key_id}
Authorization: Bearer <your-jwt-token>
```

#### Rotate API Key

```http
POST /api/v1/auth/api-keys/{key_id}/rotate
Authorization: Bearer <your-jwt-token>
```

**Response:** Returns a new API key, invalidating the old one.

---

## Token Lifecycle

### JWT Token Lifecycle

```
┌─────────────┐
│   Login     │
└──────┬──────┘
       │
       ▼
┌─────────────┐
│ JWT Issued  │ ────────► Token valid for 24 hours
└──────┬──────┘
       │
       ▼
┌─────────────┐
│ Token Used  │ ────────► Multiple API calls
└──────┬──────┘
       │
       ▼
┌─────────────┐
│Token Expires│
└──────┬──────┘
       │
       ▼
┌─────────────┐
│   Refresh   │ ────────► Use refresh token
└──────┬──────┘
       │
       ▼
┌─────────────┐
│ New JWT     │
└─────────────┘
```

### Token Refresh

When a JWT expires, use the refresh token to obtain a new one:

```http
POST /api/v1/auth/refresh
Content-Type: application/json

{
  "refresh_token": "refresh_abc123xyz456..."
}
```

**Response:**

```json
{
  "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "expires_at": "2026-01-17T10:30:00Z"
}
```

### Token Revocation

#### Logout (Revoke Current Token)

```http
POST /api/v1/auth/logout
Authorization: Bearer <your-jwt-token>
```

#### Revoke All Sessions

```http
POST /api/v1/auth/revoke-all
Authorization: Bearer <your-jwt-token>
```

**Implementation:** Tokens are added to a Redis-based blacklist until expiration.

---

## OAuth2 Integration

OAuth2 support is planned for integration with external identity providers.

### Planned Providers

- GitHub
- GitLab
- Google
- Azure AD / Microsoft Entra ID
- Okta
- Custom OIDC providers

### OAuth2 Flow (Planned)

```http
# 1. Initiate OAuth flow
GET /api/v1/auth/oauth/github/authorize

# 2. User authorizes on GitHub
# Redirected back to callback URL

# 3. Exchange code for token
GET /api/v1/auth/oauth/github/callback?code=abc123&state=xyz456

# 4. Receive JWT token
{
  "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "user": { ... }
}
```

**Status:** OAuth2 integration is on the roadmap for v0.2.0.

---

## Role-Based Access Control

SCTV implements RBAC to control user permissions within tenants.

### Default Roles

| Role | Permissions | Description |
|------|-------------|-------------|
| `viewer` | Read-only access | View projects, dependencies, alerts |
| `user` | Read + limited write | Create projects, acknowledge alerts |
| `admin` | Full access | Manage users, policies, settings |
| `owner` | Full + billing | Tenant owner with billing access |

### Custom Roles (Planned)

Future versions will support custom role definitions with granular permissions.

### Permission Structure

```rust
pub enum Permission {
    ReadProjects,
    WriteProjects,
    DeleteProjects,
    ReadAlerts,
    WriteAlerts,
    ReadPolicies,
    WritePolicies,
    ManageUsers,
    ManageSettings,
    ManageBilling,
}
```

### Checking Permissions

In API handlers, permissions are checked via the `AuthUser` extractor:

```rust
pub async fn delete_project(
    user: AuthUser,  // Automatically validates JWT
    Path(id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
) -> ApiResult<StatusCode> {
    // Check admin role
    if !user.roles.contains(&"admin".to_string()) {
        return Err(ApiError::Forbidden);
    }

    // ... deletion logic
}
```

### Role Assignment

Roles are assigned during user creation and can be modified by admins:

```http
PATCH /api/v1/users/{user_id}/roles
Authorization: Bearer <admin-jwt-token>
Content-Type: application/json

{
  "roles": ["user", "admin"]
}
```

---

## Multi-Tenant Authentication

SCTV is designed for multi-tenancy with strict tenant isolation.

### Tenant Isolation

Every authenticated request includes a `tenant_id` from the JWT claims:

```rust
pub struct Claims {
    pub sub: Uuid,          // User ID
    pub tenant_id: Uuid,    // Tenant ID - automatically scoped
    // ...
}
```

### How It Works

1. **Login**: User logs in, receives JWT with `tenant_id`
2. **API Request**: JWT is validated, `tenant_id` extracted
3. **Repository Query**: All database queries automatically filter by `tenant_id`
4. **Response**: Only tenant-specific data is returned

### Tenant Switching

Users with access to multiple tenants can switch contexts:

```http
POST /api/v1/auth/switch-tenant
Authorization: Bearer <your-jwt-token>
Content-Type: application/json

{
  "tenant_id": "880e8400-e29b-41d4-a716-446655440003"
}
```

**Response:** New JWT with updated `tenant_id`

### Cross-Tenant Access Prevention

```rust
// Example: Verifying tenant access in handlers
let project = project_repo
    .find_by_id(ProjectId(id))
    .await?
    .ok_or_else(|| ApiError::NotFound(format!("Project {} not found", id)))?;

// Verify tenant access - prevents cross-tenant data access
if project.tenant_id != user.tenant_id {
    return Err(ApiError::Forbidden);
}
```

### Tenant Configuration

```rust
pub struct Tenant {
    pub id: TenantId,
    pub name: String,
    pub plan: SubscriptionPlan,
    pub settings: TenantSettings,
    pub webhook_secret: Option<String>,
    pub max_projects: u32,
    pub max_users: u32,
    pub created_at: DateTime<Utc>,
}
```

---

## Security Best Practices

### 1. Secure Token Storage

**Browser/Frontend:**

```javascript
// Good - Use httpOnly cookies
// Set during login on server side
res.cookie('jwt_token', token, {
  httpOnly: true,
  secure: true,
  sameSite: 'strict',
  maxAge: 24 * 60 * 60 * 1000
});

// Avoid - localStorage (vulnerable to XSS)
localStorage.setItem('jwt_token', token); // DON'T DO THIS
```

**Mobile Apps:**

- Use secure storage (Keychain on iOS, KeyStore on Android)
- Never store tokens in plain text

**CLI/Service:**

- Store in OS credential manager
- Use environment variables for CI/CD
- Never commit tokens to version control

### 2. Token Transmission

Always use HTTPS in production:

```bash
# Production configuration
SCTV_API_URL=https://api.sctv.example.com
SCTV_ENABLE_TLS=true
```

### 3. JWT Secret Management

**Development:**

```bash
SCTV_JWT_SECRET=development-secret-change-in-production
```

**Production:**

```bash
# Generate a strong secret
openssl rand -base64 64 > jwt_secret.txt

# Use environment variable
SCTV_JWT_SECRET=$(cat jwt_secret.txt)

# Or use secrets management
SCTV_JWT_SECRET=$(aws secretsmanager get-secret-value --secret-id sctv/jwt-secret --query SecretString --output text)
```

**Secret Rotation:**

1. Generate new secret
2. Update `SCTV_JWT_SECRET`
3. Restart API servers
4. Force user re-authentication (optional)

### 4. API Key Security

**Generation:**

```bash
# Generate cryptographically secure API key
openssl rand -hex 32
```

**Storage:**

- Store hashed versions in database (bcrypt/argon2)
- Never log API keys
- Implement key rotation policies

**Usage:**

```bash
# Environment variable
export SCTV_API_KEY=sctv_live_1a2b3c4d5e6f7g8h9i0j

# CI/CD secrets
# GitHub Actions
# secrets.SCTV_API_KEY
```

### 5. Rate Limiting

Protect authentication endpoints:

```rust
// Configuration
SCTV_AUTH_RATE_LIMIT=10/minute
SCTV_API_RATE_LIMIT=1000/hour
```

### 6. Audit Logging

Log all authentication events:

```json
{
  "event": "login_success",
  "user_id": "550e8400-e29b-41d4-a716-446655440000",
  "tenant_id": "660e8400-e29b-41d4-a716-446655440001",
  "ip_address": "192.168.1.100",
  "user_agent": "Mozilla/5.0...",
  "timestamp": "2026-01-15T10:30:00Z"
}
```

### 7. Failed Login Protection

Implement account lockout:

- **5 failed attempts**: 5-minute lockout
- **10 failed attempts**: 1-hour lockout
- **15 failed attempts**: Account disabled (admin unlock required)

### 8. Session Management

```rust
// Configuration
SCTV_MAX_CONCURRENT_SESSIONS=5
SCTV_SESSION_TIMEOUT_MINUTES=30
SCTV_IDLE_TIMEOUT_MINUTES=15
```

### 9. Password Requirements

```rust
pub struct PasswordPolicy {
    min_length: usize,          // 12
    require_uppercase: bool,    // true
    require_lowercase: bool,    // true
    require_numbers: bool,      // true
    require_special: bool,      // true
    max_age_days: u32,         // 90
    history_count: usize,      // 5 (prevent reuse)
}
```

### 10. Security Headers

Ensure proper security headers:

```http
Strict-Transport-Security: max-age=31536000; includeSubDomains
X-Content-Type-Options: nosniff
X-Frame-Options: DENY
X-XSS-Protection: 1; mode=block
Content-Security-Policy: default-src 'self'
```

---

## Code Examples

### JavaScript/TypeScript

```typescript
import axios from 'axios';

class SctvAuth {
  private baseUrl: string;
  private token: string | null = null;

  constructor(baseUrl: string) {
    this.baseUrl = baseUrl;
  }

  async login(email: string, password: string): Promise<void> {
    const response = await axios.post(`${this.baseUrl}/api/v1/auth/login`, {
      email,
      password
    });

    this.token = response.data.token;

    // Store securely (use httpOnly cookies in production)
    localStorage.setItem('jwt_token', this.token);
  }

  async makeAuthenticatedRequest(endpoint: string): Promise<any> {
    if (!this.token) {
      throw new Error('Not authenticated');
    }

    try {
      const response = await axios.get(`${this.baseUrl}${endpoint}`, {
        headers: {
          'Authorization': `Bearer ${this.token}`
        }
      });
      return response.data;
    } catch (error) {
      if (error.response?.status === 401) {
        // Token expired, try refresh
        await this.refreshToken();
        // Retry request
        return this.makeAuthenticatedRequest(endpoint);
      }
      throw error;
    }
  }

  async refreshToken(): Promise<void> {
    const refreshToken = localStorage.getItem('refresh_token');
    const response = await axios.post(`${this.baseUrl}/api/v1/auth/refresh`, {
      refresh_token: refreshToken
    });

    this.token = response.data.token;
    localStorage.setItem('jwt_token', this.token);
  }

  logout(): void {
    this.token = null;
    localStorage.removeItem('jwt_token');
    localStorage.removeItem('refresh_token');
  }
}

// Usage
const auth = new SctvAuth('http://localhost:3000');
await auth.login('user@example.com', 'password');
const projects = await auth.makeAuthenticatedRequest('/api/v1/projects');
```

### Python

```python
import requests
from typing import Optional
from datetime import datetime, timedelta

class SctvAuthClient:
    def __init__(self, base_url: str):
        self.base_url = base_url
        self.token: Optional[str] = None
        self.token_expiry: Optional[datetime] = None

    def login(self, email: str, password: str) -> dict:
        response = requests.post(
            f"{self.base_url}/api/v1/auth/login",
            json={"email": email, "password": password}
        )
        response.raise_for_status()

        data = response.json()
        self.token = data['token']
        self.token_expiry = datetime.fromisoformat(
            data['expires_at'].replace('Z', '+00:00')
        )

        return data

    def _is_token_expired(self) -> bool:
        if not self.token_expiry:
            return True
        return datetime.now() >= self.token_expiry - timedelta(minutes=5)

    def _get_headers(self) -> dict:
        if not self.token or self._is_token_expired():
            raise ValueError("Not authenticated or token expired")

        return {
            "Authorization": f"Bearer {self.token}",
            "Content-Type": "application/json"
        }

    def get(self, endpoint: str) -> dict:
        response = requests.get(
            f"{self.base_url}{endpoint}",
            headers=self._get_headers()
        )
        response.raise_for_status()
        return response.json()

    def post(self, endpoint: str, data: dict) -> dict:
        response = requests.post(
            f"{self.base_url}{endpoint}",
            headers=self._get_headers(),
            json=data
        )
        response.raise_for_status()
        return response.json()

# Usage
client = SctvAuthClient("http://localhost:3000")
client.login("user@example.com", "password")
projects = client.get("/api/v1/projects")
```

### Rust

```rust
use reqwest::{Client, header::{HeaderMap, AUTHORIZATION, CONTENT_TYPE}};
use serde::{Deserialize, Serialize};
use anyhow::Result;

#[derive(Debug, Deserialize)]
struct LoginResponse {
    token: String,
    expires_at: String,
}

pub struct SctvClient {
    client: Client,
    base_url: String,
    token: Option<String>,
}

impl SctvClient {
    pub fn new(base_url: String) -> Self {
        Self {
            client: Client::new(),
            base_url,
            token: None,
        }
    }

    pub async fn login(&mut self, email: &str, password: &str) -> Result<()> {
        #[derive(Serialize)]
        struct LoginRequest<'a> {
            email: &'a str,
            password: &'a str,
        }

        let response = self.client
            .post(&format!("{}/api/v1/auth/login", self.base_url))
            .json(&LoginRequest { email, password })
            .send()
            .await?;

        let login_response: LoginResponse = response.json().await?;
        self.token = Some(login_response.token);

        Ok(())
    }

    fn build_headers(&self) -> Result<HeaderMap> {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, "application/json".parse()?);

        if let Some(token) = &self.token {
            headers.insert(
                AUTHORIZATION,
                format!("Bearer {}", token).parse()?
            );
        }

        Ok(headers)
    }

    pub async fn get(&self, endpoint: &str) -> Result<String> {
        let response = self.client
            .get(&format!("{}{}", self.base_url, endpoint))
            .headers(self.build_headers()?)
            .send()
            .await?;

        Ok(response.text().await?)
    }
}

// Usage
#[tokio::main]
async fn main() -> Result<()> {
    let mut client = SctvClient::new("http://localhost:3000".to_string());
    client.login("user@example.com", "password").await?;
    let projects = client.get("/api/v1/projects").await?;
    println!("{}", projects);
    Ok(())
}
```

### cURL

```bash
#!/bin/bash

# Login and store token
TOKEN=$(curl -s -X POST http://localhost:3000/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "email": "user@example.com",
    "password": "password"
  }' | jq -r '.token')

# Make authenticated request
curl -X GET http://localhost:3000/api/v1/projects \
  -H "Authorization: Bearer $TOKEN"

# Using API key
curl -X GET http://localhost:3000/api/v1/projects \
  -H "X-API-Key: sctv_live_1a2b3c4d5e6f7g8h9i0j"
```

---

## Troubleshooting

### Common Issues

#### 401 Unauthorized

**Problem:** Request returns 401 status

**Causes:**
- Missing Authorization header
- Invalid token format
- Expired token
- Invalid signature

**Solutions:**

```bash
# Check token format
echo $TOKEN | cut -d'.' -f2 | base64 -d | jq

# Verify expiration
jwt_exp=$(echo $TOKEN | cut -d'.' -f2 | base64 -d | jq -r '.exp')
current_time=$(date +%s)
if [ $current_time -gt $jwt_exp ]; then
  echo "Token expired, refresh needed"
fi

# Check token signature
curl -X POST http://localhost:3000/api/v1/auth/verify \
  -H "Authorization: Bearer $TOKEN"
```

#### 403 Forbidden

**Problem:** Authenticated but access denied

**Causes:**
- Insufficient role permissions
- Cross-tenant access attempt
- Resource doesn't belong to user's tenant

**Solution:**

```bash
# Check user roles
echo $TOKEN | cut -d'.' -f2 | base64 -d | jq '.roles'

# Verify tenant_id matches
echo $TOKEN | cut -d'.' -f2 | base64 -d | jq '.tenant_id'
```

#### Token Refresh Failures

**Problem:** Refresh token not working

**Solutions:**

```bash
# Check refresh token validity
curl -X POST http://localhost:3000/api/v1/auth/refresh \
  -H "Content-Type: application/json" \
  -d '{"refresh_token": "refresh_..."}'

# If expired, re-authenticate
curl -X POST http://localhost:3000/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email": "user@example.com", "password": "password"}'
```

#### API Key Not Working

**Problem:** X-API-Key header rejected

**Checks:**

```bash
# Verify API key format
if [[ $API_KEY =~ ^sctv_live_[a-zA-Z0-9]{40}$ ]]; then
  echo "Format valid"
else
  echo "Invalid format"
fi

# Test API key
curl -X GET http://localhost:3000/api/v1/auth/api-keys/verify \
  -H "X-API-Key: $API_KEY"
```

### Debugging Tools

#### JWT Decoder

```bash
# Decode JWT (header and payload)
jwt_decode() {
  jq -R 'split(".") | .[1] | @base64d | fromjson' <<< "$1"
}

jwt_decode "$TOKEN"
```

#### Token Introspection

```bash
curl -X POST http://localhost:3000/api/v1/auth/introspect \
  -H "Authorization: Bearer $TOKEN"
```

**Response:**

```json
{
  "active": true,
  "sub": "550e8400-e29b-41d4-a716-446655440000",
  "tenant_id": "660e8400-e29b-41d4-a716-446655440001",
  "roles": ["user"],
  "exp": 1705401600,
  "iat": 1705315200
}
```

---

## Additional Resources

- [JWT RFC 7519](https://tools.ietf.org/html/rfc7519)
- [OAuth 2.0 RFC 6749](https://tools.ietf.org/html/rfc6749)
- [OWASP Authentication Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Authentication_Cheat_Sheet.html)
- [jsonwebtoken Rust Crate](https://docs.rs/jsonwebtoken/)
- [SCTV Configuration Guide](../getting-started/configuration.md)

---

**Last Updated:** 2026-01-15
**API Version:** 0.1.0
**Maintainer:** SCTV Team
