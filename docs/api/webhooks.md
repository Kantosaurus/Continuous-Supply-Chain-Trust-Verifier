# Webhooks Documentation

**Version:** 0.1.0

Complete guide to webhook integration for the SCTV platform.

---

## Table of Contents

- [Overview](#overview)
- [Webhook Types](#webhook-types)
- [GitHub Webhooks](#github-webhooks)
- [GitLab Webhooks](#gitlab-webhooks)
- [Custom Webhooks](#custom-webhooks)
- [Webhook Payload Formats](#webhook-payload-formats)
- [Signature Verification](#signature-verification)
- [Retry Policies](#retry-policies)
- [Event Types](#event-types)
- [CI/CD Integration](#cicd-integration)
- [Testing Webhooks](#testing-webhooks)
- [Code Examples](#code-examples)
- [Troubleshooting](#troubleshooting)

---

## Overview

SCTV supports webhook integrations for:

1. **Incoming Webhooks**: Receive events from external systems (GitHub, GitLab)
2. **Outgoing Webhooks**: Send notifications to external systems (Slack, custom endpoints)

### Use Cases

- **Automated Scanning**: Trigger dependency scans on code push
- **CI/CD Integration**: Block deployments with critical vulnerabilities
- **Alerting**: Send real-time notifications to communication platforms
- **Custom Workflows**: Integrate with existing security tools

### Webhook Flow

```
┌──────────┐                ┌─────────┐                ┌──────────┐
│ GitHub/  │                │  SCTV   │                │ External │
│ GitLab   │                │   API   │                │ Service  │
└────┬─────┘                └────┬────┘                └────┬─────┘
     │                           │                          │
     │  1. Push Event            │                          │
     ├──────────────────────────>│                          │
     │                           │                          │
     │  2. Verify Signature      │                          │
     │                           │                          │
     │  3. Parse Payload         │                          │
     │                           │                          │
     │  4. Trigger Scan          │                          │
     │                           │                          │
     │  5. Scan Completes        │                          │
     │                           │                          │
     │                           │  6. Send Alert           │
     │                           ├─────────────────────────>│
     │                           │                          │
     │                           │  7. Acknowledgment       │
     │                           │<─────────────────────────┤
     │                           │                          │
     │  8. Webhook Response      │                          │
     │<──────────────────────────┤                          │
```

---

## Webhook Types

### Incoming Webhooks

Receive events from external platforms:

| Platform | Endpoint | Purpose |
|----------|----------|---------|
| GitHub | `/api/v1/webhooks/github` | Push, PR events |
| GitLab | `/api/v1/webhooks/gitlab` | Push, MR events |
| Custom | `/api/v1/webhooks/custom/{id}` | Custom integrations |

### Outgoing Webhooks

Send notifications to external systems:

| Type | Description | Use Case |
|------|-------------|----------|
| Slack | Slack webhook integration | Alert notifications |
| Teams | Microsoft Teams integration | Team notifications |
| Discord | Discord webhook | Developer notifications |
| Custom | Generic HTTP POST | Custom integrations |

---

## GitHub Webhooks

### Setup

1. **Navigate to Repository Settings**
   - Go to your GitHub repository
   - Click "Settings" > "Webhooks" > "Add webhook"

2. **Configure Webhook**
   ```
   Payload URL: https://api.sctv.example.com/api/v1/webhooks/github
   Content type: application/json
   Secret: <your-webhook-secret>
   ```

3. **Select Events**
   - Push events
   - Pull request events
   - Release events

### Webhook URL

```
POST https://api.sctv.example.com/api/v1/webhooks/github
```

### Supported Events

| Event | Action | SCTV Response |
|-------|--------|---------------|
| `push` | Code pushed to repository | Trigger dependency scan |
| `pull_request` | PR opened/synchronized | Scan PR changes |
| `release` | New release published | Verify release artifacts |
| `create` | Branch/tag created | Optional scan |

### Payload Structure

```json
{
  "action": "push",
  "ref": "refs/heads/main",
  "repository": {
    "id": 123456789,
    "name": "my-project",
    "full_name": "org/my-project",
    "clone_url": "https://github.com/org/my-project.git",
    "default_branch": "main"
  },
  "pusher": {
    "name": "johndoe",
    "email": "john@example.com"
  },
  "commits": [
    {
      "id": "abc123def456",
      "message": "Update dependencies",
      "timestamp": "2026-01-15T10:30:00Z",
      "author": {
        "name": "John Doe",
        "email": "john@example.com"
      }
    }
  ]
}
```

### Example Request

```bash
curl -X POST https://api.sctv.example.com/api/v1/webhooks/github \
  -H "Content-Type: application/json" \
  -H "X-GitHub-Event: push" \
  -H "X-Hub-Signature-256: sha256=abc123..." \
  -H "X-GitHub-Delivery: 12345678-1234-1234-1234-123456789abc" \
  -d @github_push_payload.json
```

### Response

```json
{
  "received": true,
  "message": "Webhook received successfully",
  "scan_triggered": true,
  "scan_id": "550e8400-e29b-41d4-a716-446655440000"
}
```

### GitHub Signature Verification

```rust
use hmac::{Hmac, Mac};
use sha2::Sha256;

fn verify_github_signature(
    payload: &[u8],
    signature: &str,
    secret: &str,
) -> bool {
    type HmacSha256 = Hmac<Sha256>;

    // Remove "sha256=" prefix
    let signature_bytes = match hex::decode(signature.strip_prefix("sha256=").unwrap_or(signature)) {
        Ok(bytes) => bytes,
        Err(_) => return false,
    };

    let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).unwrap();
    mac.update(payload);

    mac.verify_slice(&signature_bytes).is_ok()
}
```

### Headers

| Header | Description |
|--------|-------------|
| `X-GitHub-Event` | Event type (push, pull_request, etc.) |
| `X-Hub-Signature-256` | HMAC SHA256 signature |
| `X-GitHub-Delivery` | Unique delivery UUID |
| `User-Agent` | GitHub-Hookshot/{version} |

---

## GitLab Webhooks

### Setup

1. **Navigate to Project Settings**
   - Go to your GitLab project
   - Click "Settings" > "Webhooks"

2. **Configure Webhook**
   ```
   URL: https://api.sctv.example.com/api/v1/webhooks/gitlab
   Secret Token: <your-webhook-secret>
   ```

3. **Select Triggers**
   - Push events
   - Merge request events
   - Tag push events

### Webhook URL

```
POST https://api.sctv.example.com/api/v1/webhooks/gitlab
```

### Supported Events

| Event | Action | SCTV Response |
|-------|--------|---------------|
| `push` | Code pushed | Trigger scan |
| `merge_request` | MR opened/updated | Scan MR |
| `tag_push` | Tag created | Verify tag |
| `release` | Release created | Scan release |

### Payload Structure

```json
{
  "object_kind": "push",
  "ref": "refs/heads/main",
  "before": "def456abc789",
  "after": "abc123def456",
  "project": {
    "id": 123,
    "name": "my-project",
    "path_with_namespace": "org/my-project",
    "git_http_url": "https://gitlab.com/org/my-project.git",
    "default_branch": "main"
  },
  "user_name": "John Doe",
  "user_email": "john@example.com",
  "commits": [
    {
      "id": "abc123def456",
      "message": "Update dependencies",
      "timestamp": "2026-01-15T10:30:00+00:00",
      "author": {
        "name": "John Doe",
        "email": "john@example.com"
      }
    }
  ]
}
```

### Example Request

```bash
curl -X POST https://api.sctv.example.com/api/v1/webhooks/gitlab \
  -H "Content-Type: application/json" \
  -H "X-Gitlab-Event: Push Hook" \
  -H "X-Gitlab-Token: your-secret-token" \
  -d @gitlab_push_payload.json
```

### Response

```json
{
  "received": true,
  "message": "Webhook received successfully",
  "scan_triggered": true,
  "scan_id": "660e8400-e29b-41d4-a716-446655440001"
}
```

### GitLab Token Verification

```rust
fn verify_gitlab_token(provided_token: &str, expected_token: &str) -> bool {
    use constant_time_eq::constant_time_eq;

    if provided_token.len() != expected_token.len() {
        return false;
    }

    constant_time_eq(provided_token.as_bytes(), expected_token.as_bytes())
}
```

### Headers

| Header | Description |
|--------|-------------|
| `X-Gitlab-Event` | Event type |
| `X-Gitlab-Token` | Secret token for verification |
| `X-Gitlab-Instance` | GitLab instance URL |
| `User-Agent` | GitLab/{version} |

---

## Custom Webhooks

Create custom webhook endpoints for specific integrations.

### Creating a Custom Webhook

```http
POST /api/v1/webhooks/custom
Authorization: Bearer <jwt-token>
Content-Type: application/json

{
  "name": "CI Pipeline Integration",
  "description": "Trigger scans from Jenkins",
  "events": ["scan.completed", "alert.created"],
  "target_url": "https://jenkins.example.com/sctv-webhook",
  "secret": "webhook-secret-token",
  "enabled": true
}
```

**Response:**

```json
{
  "id": "770e8400-e29b-41d4-a716-446655440002",
  "name": "CI Pipeline Integration",
  "webhook_url": "https://api.sctv.example.com/api/v1/webhooks/custom/770e8400-e29b-41d4-a716-446655440002",
  "secret": "webhook-secret-token",
  "events": ["scan.completed", "alert.created"],
  "created_at": "2026-01-15T10:30:00Z"
}
```

### Invoking Custom Webhook

```bash
curl -X POST https://api.sctv.example.com/api/v1/webhooks/custom/770e8400-e29b-41d4-a716-446655440002 \
  -H "Content-Type: application/json" \
  -H "X-Webhook-Secret: webhook-secret-token" \
  -d '{
    "event": "custom.trigger",
    "project_id": "550e8400-e29b-41d4-a716-446655440000",
    "action": "scan"
  }'
```

### Managing Custom Webhooks

#### List Webhooks

```http
GET /api/v1/webhooks/custom
Authorization: Bearer <jwt-token>
```

#### Update Webhook

```http
PATCH /api/v1/webhooks/custom/{id}
Authorization: Bearer <jwt-token>
Content-Type: application/json

{
  "enabled": false,
  "events": ["alert.created"]
}
```

#### Delete Webhook

```http
DELETE /api/v1/webhooks/custom/{id}
Authorization: Bearer <jwt-token>
```

---

## Webhook Payload Formats

### Scan Completed Event

```json
{
  "event": "scan.completed",
  "timestamp": "2026-01-15T10:35:00Z",
  "tenant_id": "660e8400-e29b-41d4-a716-446655440001",
  "data": {
    "scan_id": "880e8400-e29b-41d4-a716-446655440003",
    "project_id": "550e8400-e29b-41d4-a716-446655440000",
    "project_name": "my-api-server",
    "status": "completed",
    "duration_seconds": 45,
    "dependencies_found": 127,
    "alerts_created": 3,
    "critical_count": 1,
    "high_count": 2,
    "medium_count": 0,
    "low_count": 0
  }
}
```

### Alert Created Event

```json
{
  "event": "alert.created",
  "timestamp": "2026-01-15T10:35:30Z",
  "tenant_id": "660e8400-e29b-41d4-a716-446655440001",
  "data": {
    "alert_id": "990e8400-e29b-41d4-a716-446655440004",
    "project_id": "550e8400-e29b-41d4-a716-446655440000",
    "project_name": "my-api-server",
    "alert_type": "DependencyTampering",
    "severity": "CRITICAL",
    "title": "Package hash mismatch detected",
    "description": "The package 'lodash@4.17.21' has a hash that doesn't match the registry",
    "dependency_name": "lodash",
    "dependency_version": "4.17.21",
    "ecosystem": "NPM",
    "created_at": "2026-01-15T10:35:30Z"
  }
}
```

### Alert Resolved Event

```json
{
  "event": "alert.resolved",
  "timestamp": "2026-01-15T11:00:00Z",
  "tenant_id": "660e8400-e29b-41d4-a716-446655440001",
  "data": {
    "alert_id": "990e8400-e29b-41d4-a716-446655440004",
    "project_id": "550e8400-e29b-41d4-a716-446655440000",
    "project_name": "my-api-server",
    "resolved_by": {
      "user_id": "550e8400-e29b-41d4-a716-446655440000",
      "email": "user@example.com"
    },
    "action_taken": "Upgraded to patched version",
    "new_version": "4.17.22",
    "resolved_at": "2026-01-15T11:00:00Z"
  }
}
```

### Policy Violation Event

```json
{
  "event": "policy.violated",
  "timestamp": "2026-01-15T10:40:00Z",
  "tenant_id": "660e8400-e29b-41d4-a716-446655440001",
  "data": {
    "policy_id": "aa0e8400-e29b-41d4-a716-446655440005",
    "policy_name": "Block Unverified Packages",
    "project_id": "550e8400-e29b-41d4-a716-446655440000",
    "project_name": "my-api-server",
    "violation_type": "provenance_missing",
    "dependency_name": "example-package",
    "dependency_version": "1.2.3",
    "action": "blocked",
    "created_at": "2026-01-15T10:40:00Z"
  }
}
```

---

## Signature Verification

### Why Verify Signatures?

Signature verification ensures:
1. Webhook came from trusted source
2. Payload wasn't tampered with
3. Protection against replay attacks

### HMAC SHA256 Verification

```rust
use hmac::{Hmac, Mac};
use sha2::Sha256;
use hex;

pub fn verify_webhook_signature(
    payload: &[u8],
    signature: &str,
    secret: &str,
) -> Result<bool, Box<dyn std::error::Error>> {
    type HmacSha256 = Hmac<Sha256>;

    // Create HMAC instance
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes())?;

    // Update with payload
    mac.update(payload);

    // Get expected signature
    let expected = mac.finalize();
    let expected_bytes = expected.into_bytes();

    // Decode provided signature
    let provided_bytes = hex::decode(signature)?;

    // Constant-time comparison
    Ok(expected_bytes.as_slice() == provided_bytes.as_slice())
}
```

### Implementation Examples

#### Node.js

```javascript
const crypto = require('crypto');

function verifyWebhookSignature(payload, signature, secret) {
  const hmac = crypto.createHmac('sha256', secret);
  hmac.update(payload);
  const expectedSignature = hmac.digest('hex');

  // Constant-time comparison
  return crypto.timingSafeEqual(
    Buffer.from(signature),
    Buffer.from(expectedSignature)
  );
}

// Express middleware
app.post('/webhooks/github', express.raw({ type: 'application/json' }), (req, res) => {
  const signature = req.headers['x-hub-signature-256'];
  const secret = process.env.WEBHOOK_SECRET;

  if (!signature) {
    return res.status(401).json({ error: 'Missing signature' });
  }

  // Remove "sha256=" prefix
  const providedSignature = signature.replace('sha256=', '');

  if (!verifyWebhookSignature(req.body, providedSignature, secret)) {
    return res.status(401).json({ error: 'Invalid signature' });
  }

  // Process webhook
  const payload = JSON.parse(req.body);
  processWebhook(payload);

  res.json({ received: true });
});
```

#### Python

```python
import hmac
import hashlib

def verify_webhook_signature(payload: bytes, signature: str, secret: str) -> bool:
    expected_signature = hmac.new(
        secret.encode('utf-8'),
        payload,
        hashlib.sha256
    ).hexdigest()

    # Constant-time comparison
    return hmac.compare_digest(signature, expected_signature)

# Flask example
from flask import Flask, request, jsonify

app = Flask(__name__)

@app.route('/webhooks/github', methods=['POST'])
def github_webhook():
    signature = request.headers.get('X-Hub-Signature-256', '')
    secret = os.environ['WEBHOOK_SECRET']

    if not signature:
        return jsonify({'error': 'Missing signature'}), 401

    # Remove "sha256=" prefix
    provided_signature = signature.replace('sha256=', '')

    if not verify_webhook_signature(request.data, provided_signature, secret):
        return jsonify({'error': 'Invalid signature'}), 401

    # Process webhook
    payload = request.get_json()
    process_webhook(payload)

    return jsonify({'received': True})
```

---

## Retry Policies

### SCTV Outgoing Webhook Retries

When SCTV sends webhooks to your endpoints:

```rust
pub struct RetryPolicy {
    max_attempts: u32,      // Default: 3
    initial_delay: Duration, // Default: 1 second
    max_delay: Duration,     // Default: 60 seconds
    backoff_multiplier: f64, // Default: 2.0
}
```

**Retry Schedule:**

| Attempt | Delay |
|---------|-------|
| 1 | Immediate |
| 2 | 1 second |
| 3 | 2 seconds |
| 4 | 4 seconds (capped at max_delay) |

### Expected Response Codes

| Status Code | Action |
|-------------|--------|
| 200-299 | Success, no retry |
| 429 | Rate limited, retry with backoff |
| 500-599 | Server error, retry |
| 408 | Timeout, retry |
| Other | Fail, no retry |

### Webhook Delivery Status

Check webhook delivery status:

```http
GET /api/v1/webhooks/{webhook_id}/deliveries
Authorization: Bearer <jwt-token>
```

**Response:**

```json
{
  "deliveries": [
    {
      "id": "bb0e8400-e29b-41d4-a716-446655440006",
      "event": "scan.completed",
      "status": "success",
      "status_code": 200,
      "attempts": 1,
      "delivered_at": "2026-01-15T10:35:05Z",
      "response_body": "{\"received\":true}"
    },
    {
      "id": "cc0e8400-e29b-41d4-a716-446655440007",
      "event": "alert.created",
      "status": "failed",
      "status_code": 500,
      "attempts": 3,
      "last_attempt_at": "2026-01-15T10:36:10Z",
      "error": "Internal Server Error"
    }
  ]
}
```

### Manual Retry

Retry a failed webhook delivery:

```http
POST /api/v1/webhooks/{webhook_id}/deliveries/{delivery_id}/retry
Authorization: Bearer <jwt-token>
```

---

## Event Types

### Available Events

| Event | Description | When Triggered |
|-------|-------------|----------------|
| `scan.queued` | Scan queued | Scan enqueued |
| `scan.started` | Scan started | Scan begins processing |
| `scan.completed` | Scan completed | Scan finishes successfully |
| `scan.failed` | Scan failed | Scan encounters error |
| `alert.created` | Alert created | New alert detected |
| `alert.acknowledged` | Alert acknowledged | User acknowledges alert |
| `alert.resolved` | Alert resolved | Alert fixed |
| `alert.suppressed` | Alert suppressed | Alert temporarily ignored |
| `policy.violated` | Policy violated | Policy rule broken |
| `policy.created` | Policy created | New policy added |
| `policy.updated` | Policy updated | Policy modified |
| `dependency.added` | Dependency added | New dependency detected |
| `dependency.updated` | Dependency updated | Dependency version changed |
| `dependency.removed` | Dependency removed | Dependency no longer used |

### Subscribing to Events

```http
POST /api/v1/webhooks/custom
Authorization: Bearer <jwt-token>
Content-Type: application/json

{
  "name": "Alert Notifications",
  "target_url": "https://hooks.slack.com/services/YOUR/WEBHOOK/URL",
  "events": [
    "alert.created",
    "alert.resolved"
  ],
  "filters": {
    "severity": ["CRITICAL", "HIGH"]
  }
}
```

---

## CI/CD Integration

### GitHub Actions

```yaml
# .github/workflows/sctv-scan.yml
name: SCTV Security Scan

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
  sctv-scan:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Trigger SCTV Scan
        env:
          SCTV_API_KEY: ${{ secrets.SCTV_API_KEY }}
          SCTV_PROJECT_ID: ${{ secrets.SCTV_PROJECT_ID }}
        run: |
          curl -X POST https://api.sctv.example.com/api/v1/projects/${SCTV_PROJECT_ID}/scan \
            -H "X-API-Key: ${SCTV_API_KEY}" \
            -H "Content-Type: application/json" \
            -d '{"full_scan": true}'

      - name: Check Scan Results
        env:
          SCTV_API_KEY: ${{ secrets.SCTV_API_KEY }}
          SCTV_PROJECT_ID: ${{ secrets.SCTV_PROJECT_ID }}
        run: |
          # Wait for scan to complete
          sleep 30

          # Get critical alerts
          ALERTS=$(curl -s https://api.sctv.example.com/api/v1/alerts \
            -H "X-API-Key: ${SCTV_API_KEY}" \
            -G --data-urlencode "project_id=${SCTV_PROJECT_ID}" \
            --data-urlencode "severity=CRITICAL" \
            --data-urlencode "status=OPEN")

          COUNT=$(echo $ALERTS | jq '.data | length')

          if [ "$COUNT" -gt 0 ]; then
            echo "::error::Found ${COUNT} critical vulnerabilities"
            exit 1
          fi
```

### GitLab CI

```yaml
# .gitlab-ci.yml
sctv-scan:
  stage: security
  script:
    - |
      curl -X POST https://api.sctv.example.com/api/v1/projects/${SCTV_PROJECT_ID}/scan \
        -H "X-API-Key: ${SCTV_API_KEY}" \
        -H "Content-Type: application/json" \
        -d '{"full_scan": true}'

    - sleep 30

    - |
      ALERTS=$(curl -s https://api.sctv.example.com/api/v1/alerts \
        -H "X-API-Key: ${SCTV_API_KEY}" \
        -G --data-urlencode "project_id=${SCTV_PROJECT_ID}" \
        --data-urlencode "severity=CRITICAL" \
        --data-urlencode "status=OPEN")

      COUNT=$(echo $ALERTS | jq '.data | length')

      if [ "$COUNT" -gt 0 ]; then
        echo "Found ${COUNT} critical vulnerabilities"
        exit 1
      fi
  only:
    - main
    - merge_requests
```

### Jenkins Pipeline

```groovy
pipeline {
    agent any

    environment {
        SCTV_API_KEY = credentials('sctv-api-key')
        SCTV_PROJECT_ID = 'your-project-id'
    }

    stages {
        stage('SCTV Scan') {
            steps {
                script {
                    def response = sh(
                        script: """
                        curl -X POST https://api.sctv.example.com/api/v1/projects/${SCTV_PROJECT_ID}/scan \
                          -H "X-API-Key: ${SCTV_API_KEY}" \
                          -H "Content-Type: application/json" \
                          -d '{"full_scan": true}'
                        """,
                        returnStdout: true
                    ).trim()

                    def scan = readJSON text: response
                    env.SCAN_ID = scan.scan_id
                }
            }
        }

        stage('Wait for Scan') {
            steps {
                script {
                    def completed = false
                    def attempts = 0

                    while (!completed && attempts < 10) {
                        sleep 30

                        def status = sh(
                            script: """
                            curl -s https://api.sctv.example.com/api/v1/scans/${env.SCAN_ID} \
                              -H "X-API-Key: ${SCTV_API_KEY}"
                            """,
                            returnStdout: true
                        ).trim()

                        def scan = readJSON text: status
                        completed = scan.status == 'completed'
                        attempts++
                    }
                }
            }
        }

        stage('Check Results') {
            steps {
                script {
                    def alerts = sh(
                        script: """
                        curl -s 'https://api.sctv.example.com/api/v1/alerts?project_id=${SCTV_PROJECT_ID}&severity=CRITICAL&status=OPEN' \
                          -H "X-API-Key: ${SCTV_API_KEY}"
                        """,
                        returnStdout: true
                    ).trim()

                    def result = readJSON text: alerts
                    def count = result.data.size()

                    if (count > 0) {
                        error("Found ${count} critical vulnerabilities")
                    }
                }
            }
        }
    }
}
```

### CircleCI

```yaml
# .circleci/config.yml
version: 2.1

jobs:
  sctv-scan:
    docker:
      - image: cimg/base:stable
    steps:
      - checkout

      - run:
          name: Trigger SCTV Scan
          command: |
            curl -X POST https://api.sctv.example.com/api/v1/projects/${SCTV_PROJECT_ID}/scan \
              -H "X-API-Key: ${SCTV_API_KEY}" \
              -H "Content-Type: application/json" \
              -d '{"full_scan": true}'

      - run:
          name: Wait for Scan
          command: sleep 30

      - run:
          name: Check Results
          command: |
            ALERTS=$(curl -s https://api.sctv.example.com/api/v1/alerts \
              -H "X-API-Key: ${SCTV_API_KEY}" \
              -G --data-urlencode "project_id=${SCTV_PROJECT_ID}" \
              --data-urlencode "severity=CRITICAL" \
              --data-urlencode "status=OPEN")

            COUNT=$(echo $ALERTS | jq '.data | length')

            if [ "$COUNT" -gt 0 ]; then
              echo "Found ${COUNT} critical vulnerabilities"
              exit 1
            fi

workflows:
  main:
    jobs:
      - sctv-scan:
          filters:
            branches:
              only: main
```

---

## Testing Webhooks

### Local Testing with ngrok

```bash
# Install ngrok
brew install ngrok  # macOS
# or download from https://ngrok.com

# Start ngrok tunnel
ngrok http 3000

# Use the generated URL in webhook configuration
# https://abc123.ngrok.io/api/v1/webhooks/github
```

### Mock Webhook Payloads

#### GitHub Push Event

```bash
curl -X POST http://localhost:3000/api/v1/webhooks/github \
  -H "Content-Type: application/json" \
  -H "X-GitHub-Event: push" \
  -H "X-Hub-Signature-256: sha256=$(echo -n '{"action":"push"}' | openssl dgst -sha256 -hmac 'your-secret' | awk '{print $2}')" \
  -d '{
    "action": "push",
    "ref": "refs/heads/main",
    "repository": {
      "id": 123456789,
      "name": "test-project",
      "full_name": "org/test-project",
      "clone_url": "https://github.com/org/test-project.git"
    }
  }'
```

#### GitLab Push Event

```bash
curl -X POST http://localhost:3000/api/v1/webhooks/gitlab \
  -H "Content-Type: application/json" \
  -H "X-Gitlab-Event: Push Hook" \
  -H "X-Gitlab-Token: your-secret-token" \
  -d '{
    "object_kind": "push",
    "ref": "refs/heads/main",
    "project": {
      "id": 123,
      "name": "test-project",
      "path_with_namespace": "org/test-project",
      "git_http_url": "https://gitlab.com/org/test-project.git"
    }
  }'
```

### Webhook Testing Tools

- **RequestBin**: https://requestbin.com
- **Webhook.site**: https://webhook.site
- **Postman**: https://www.postman.com
- **Insomnia**: https://insomnia.rest

---

## Code Examples

### Receiving Webhooks (Node.js)

```javascript
const express = require('express');
const crypto = require('crypto');

const app = express();
app.use(express.json());

// Middleware to verify webhook signature
function verifyWebhook(req, res, next) {
  const signature = req.headers['x-hub-signature-256'];
  const secret = process.env.WEBHOOK_SECRET;

  if (!signature) {
    return res.status(401).json({ error: 'Missing signature' });
  }

  const hmac = crypto.createHmac('sha256', secret);
  hmac.update(JSON.stringify(req.body));
  const expectedSignature = 'sha256=' + hmac.digest('hex');

  if (!crypto.timingSafeEqual(
    Buffer.from(signature),
    Buffer.from(expectedSignature)
  )) {
    return res.status(401).json({ error: 'Invalid signature' });
  }

  next();
}

// Handle GitHub webhook
app.post('/webhooks/github', verifyWebhook, (req, res) => {
  const event = req.headers['x-github-event'];
  const payload = req.body;

  console.log(`Received ${event} event`);

  if (event === 'push') {
    const branch = payload.ref.replace('refs/heads/', '');
    console.log(`Push to ${branch} branch`);

    // Trigger scan
    triggerScan(payload.repository.name);
  }

  res.json({ received: true });
});

async function triggerScan(projectName) {
  // Implementation
}

app.listen(3000, () => {
  console.log('Webhook server listening on port 3000');
});
```

### Sending Webhooks (Python)

```python
import requests
import hmac
import hashlib
import json

class WebhookSender:
    def __init__(self, target_url: str, secret: str):
        self.target_url = target_url
        self.secret = secret

    def send(self, event: str, data: dict) -> bool:
        payload = {
            'event': event,
            'timestamp': datetime.utcnow().isoformat() + 'Z',
            'data': data
        }

        payload_bytes = json.dumps(payload).encode('utf-8')

        # Generate signature
        signature = hmac.new(
            self.secret.encode('utf-8'),
            payload_bytes,
            hashlib.sha256
        ).hexdigest()

        headers = {
            'Content-Type': 'application/json',
            'X-Webhook-Signature': signature,
            'X-Webhook-Event': event
        }

        try:
            response = requests.post(
                self.target_url,
                data=payload_bytes,
                headers=headers,
                timeout=10
            )
            response.raise_for_status()
            return True
        except requests.exceptions.RequestException as e:
            print(f'Webhook delivery failed: {e}')
            return False

# Usage
sender = WebhookSender(
    target_url='https://hooks.example.com/sctv',
    secret='your-webhook-secret'
)

sender.send('scan.completed', {
    'scan_id': '550e8400-e29b-41d4-a716-446655440000',
    'project_id': '660e8400-e29b-41d4-a716-446655440001',
    'status': 'completed',
    'alerts_created': 3
})
```

---

## Troubleshooting

### Common Issues

#### Webhook Not Triggering

**Check:**

1. Webhook URL is accessible
   ```bash
   curl https://api.sctv.example.com/api/v1/webhooks/github
   ```

2. Firewall rules allow inbound traffic
3. SSL certificate is valid (for HTTPS)
4. Webhook is enabled in platform settings

#### Signature Verification Failed

**Debug:**

```bash
# Calculate expected signature
echo -n '{"action":"push"}' | openssl dgst -sha256 -hmac 'your-secret'

# Compare with received signature
```

**Common causes:**
- Wrong secret key
- Payload modification (whitespace, encoding)
- Timing issues (use raw body)

#### Timeouts

**Solutions:**

1. Increase timeout on sender side
2. Respond quickly (process async)
   ```javascript
   app.post('/webhook', (req, res) => {
     // Respond immediately
     res.json({ received: true });

     // Process asynchronously
     processWebhookAsync(req.body);
   });
   ```

3. Implement retry logic

#### Missing Events

**Check webhook logs:**

```http
GET /api/v1/webhooks/{id}/logs
Authorization: Bearer <jwt-token>
```

**Verify event subscription:**

```http
GET /api/v1/webhooks/{id}
Authorization: Bearer <jwt-token>
```

### Debug Mode

Enable verbose webhook logging:

```bash
SCTV_WEBHOOK_DEBUG=true
SCTV_LOG_LEVEL=debug
```

### Testing Checklist

- [ ] Webhook URL is correct and accessible
- [ ] Secret token matches on both sides
- [ ] Signature verification logic is correct
- [ ] Event types are subscribed
- [ ] Firewall/network allows traffic
- [ ] SSL certificate is valid
- [ ] Timeout is sufficient
- [ ] Retry logic is implemented
- [ ] Error handling is in place
- [ ] Logs are reviewed

---

## Additional Resources

- [GitHub Webhooks Documentation](https://docs.github.com/en/developers/webhooks-and-events/webhooks)
- [GitLab Webhooks Documentation](https://docs.gitlab.com/ee/user/project/integrations/webhooks.html)
- [HMAC Authentication Guide](https://www.ietf.org/rfc/rfc2104.txt)
- [SCTV REST API Documentation](rest-api.md)
- [SCTV CI/CD Integration Guide](../development/ci-cd-integration.md)

---

**Last Updated:** 2026-01-15
**API Version:** 0.1.0
**Maintainer:** SCTV Team
