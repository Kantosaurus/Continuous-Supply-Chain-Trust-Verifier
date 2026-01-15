# Quick Start Guide

**Version:** 0.1.0
**Time to Complete:** 15 minutes

This guide gets you up and running with SCTV quickly. You'll scan your first project and detect supply chain threats.

---

## Table of Contents

- [Before You Begin](#before-you-begin)
- [Step 1: Start SCTV](#step-1-start-sctv)
- [Step 2: Access the Dashboard](#step-2-access-the-dashboard)
- [Step 3: Create Your First Project](#step-3-create-your-first-project)
- [Step 4: Run a Security Scan](#step-4-run-a-security-scan)
- [Step 5: Review Alerts](#step-5-review-alerts)
- [Step 6: Create a Security Policy](#step-6-create-a-security-policy)
- [Next Steps](#next-steps)

---

## Before You Begin

Ensure you have:

- Docker and Docker Compose installed
- At least 4 GB of free RAM
- A project with dependencies (npm, pip, Maven, etc.)

If you haven't installed SCTV yet, see the [Installation Guide](installation.md).

---

## Step 1: Start SCTV

### Using Docker Compose (Recommended)

```bash
# Clone the repository
git clone https://github.com/example/supply-chain-trust-verifier.git
cd supply-chain-trust-verifier

# Start all services
docker-compose up -d

# Wait for services to be ready (about 30 seconds)
docker-compose logs -f api | grep "Starting SCTV API server"
```

### Using the CLI (Native Installation)

```bash
# Start the API server
sctv-api --config /etc/sctv/config.toml &

# Start the worker service
sctv-worker --config /etc/sctv/config.toml &
```

### Verify Services are Running

```bash
# Check API health
curl http://localhost:3000/health

# Expected output:
# {"status":"healthy","version":"0.1.0","timestamp":"..."}
```

---

## Step 2: Access the Dashboard

Open your web browser and navigate to:

```
http://localhost:3000
```

### First-Time Setup

1. **Create an admin account:**
   - Email: Your email address
   - Password: Choose a strong password
   - Organization name: Your company/team name

2. **Complete the onboarding wizard:**
   - Select your primary package ecosystems (npm, PyPI, Maven, etc.)
   - Configure notification channels (optional, can be done later)
   - Set up your first project

### Using API Keys (Alternative)

If you prefer programmatic access:

```bash
# Create an API key using the CLI
export SCTV_API_KEY=$(sctv-cli auth create-key \
  --name "My API Key" \
  --email admin@example.com)

# Use the API key for subsequent requests
curl -H "Authorization: Bearer $SCTV_API_KEY" \
  http://localhost:3000/api/v1/projects
```

---

## Step 3: Create Your First Project

### Option A: Using the Dashboard

1. Click **"New Project"** in the dashboard
2. Fill in project details:
   - **Name:** "My Application"
   - **Description:** "Production web application"
   - **Repository URL:** https://github.com/Kantosaurus/your-repo (optional)
   - **Ecosystems:** Select your package managers (e.g., npm, PyPI)
3. Click **"Create Project"**

### Option B: Using the CLI

```bash
# Create a project for an npm-based application
sctv-cli project create \
  --name "My Application" \
  --ecosystem npm \
  --path ./my-app

# Output:
# ✓ Project created successfully
# ID: 550e8400-e29b-41d4-a716-446655440000
```

### Option C: Using the REST API

```bash
curl -X POST http://localhost:3000/api/v1/projects \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $SCTV_API_KEY" \
  -d '{
    "name": "My Application",
    "description": "Production web application",
    "repository_url": "https://github.com/Kantosaurus/your-repo",
    "ecosystems": ["npm"],
    "scan_schedule": {
      "type": "daily",
      "hour": 2
    }
  }'
```

---

## Step 4: Run a Security Scan

### Scan Using the CLI

Navigate to your project directory and run:

```bash
# Scan a Node.js project
cd /path/to/your/project
sctv-cli scan --ecosystem npm

# Scan with verbose output
sctv-cli scan --ecosystem npm --verbose

# Scan and output results as JSON
sctv-cli scan --ecosystem npm --format json > scan-results.json

# Scan a Python project
sctv-cli scan --ecosystem pypi --path /path/to/python/project

# Scan multiple ecosystems
sctv-cli scan --ecosystem npm --ecosystem pypi
```

### Scan Using the Dashboard

1. Navigate to your project in the dashboard
2. Click **"Scan Now"** button
3. Watch real-time progress as SCTV:
   - Discovers dependencies
   - Queries package registries
   - Runs threat detectors
   - Generates alerts

### Understanding Scan Results

SCTV will detect:

- **Typosquatting** - Packages with names similar to popular libraries
- **Tampering** - Packages with mismatched checksums
- **Downgrade Attacks** - Suspicious version rollbacks
- **Provenance Failures** - Missing or invalid build attestations
- **Policy Violations** - Dependencies that violate your security rules
- **New Packages** - Recently published packages (< 30 days old)
- **Suspicious Maintainers** - Unusual maintainer activity

### Example Scan Output

```
Scanning project: My Application
Ecosystem: npm
───────────────────────────────────────────────────────

✓ Discovered 847 dependencies (143 direct, 704 transitive)
✓ Verified checksums for 847 packages
✓ Checked 143 direct dependencies for typosquatting
✓ Validated provenance for 12 critical packages

Findings:
─────────────────────────────────────────────────────
🔴 CRITICAL (1):
  - Typosquatting: lodash-utils → lodash (similarity: 92%)

🟠 HIGH (3):
  - Provenance failure: webpack@5.89.0 (missing SLSA attestation)
  - New package: super-new-lib@1.0.0 (published 5 days ago)
  - Policy violation: colors@1.4.0 (deprecated package)

🟡 MEDIUM (7):
  - Downgrade: axios 1.6.0 → 1.5.1 (suspicious rollback)
  - ... (6 more)

🔵 LOW (12):
  - ... (12 items)

Total alerts: 23
Scan completed in 18.4s
```

---

## Step 5: Review Alerts

### View Alerts in the Dashboard

1. Navigate to **Alerts** in the sidebar
2. Filter by:
   - **Severity:** Critical, High, Medium, Low, Info
   - **Type:** Typosquatting, Tampering, Downgrade, etc.
   - **Status:** Open, Acknowledged, Resolved
   - **Project:** Specific project or all projects

3. Click on an alert to see:
   - **Description:** What was detected
   - **Severity:** Impact level
   - **Affected Package:** Name, version, ecosystem
   - **Evidence:** Detection details
   - **Remediation:** Recommended actions
   - **Timeline:** When detected, acknowledged, resolved

### Example Alert Details

```
Alert: Typosquatting Detected
Severity: CRITICAL
Status: Open
Created: 2026-01-15 10:30:00 UTC

Package: lodash-utils@1.0.0
Ecosystem: npm
Suspected Target: lodash (99% similarity)

Evidence:
- Package name is suspiciously similar to 'lodash' (popular package)
- Published by a new maintainer (account created 7 days ago)
- Contains obfuscated code
- Low download count (147 downloads)

Remediation:
1. Remove 'lodash-utils' from dependencies
2. Use official 'lodash' package instead
3. Review code for potential compromise
4. Rotate any secrets that may have been exposed

Related CVEs: None
CVSS Score: N/A (Supply chain attack)
```

### Managing Alerts

**Acknowledge an alert:**
```bash
sctv-cli alert acknowledge <alert-id> \
  --comment "Reviewing with security team"
```

**Resolve an alert:**
```bash
sctv-cli alert resolve <alert-id> \
  --resolution "Removed malicious dependency"
```

**Suppress an alert (false positive):**
```bash
sctv-cli alert suppress <alert-id> \
  --reason "Internally maintained package"
```

---

## Step 6: Create a Security Policy

Policies enforce security rules across your projects.

### Create a Basic Policy

```bash
# Create a policy file: policy.json
cat > policy.json << 'EOF'
{
  "name": "Production Security Policy",
  "description": "Security requirements for production deployments",
  "rules": [
    {
      "type": "BlockDeprecated",
      "severity": "high"
    },
    {
      "type": "RequireProvenanceForCritical",
      "min_slsa_level": 2
    },
    {
      "type": "BlockPackageAge",
      "min_age_days": 30,
      "severity": "medium"
    },
    {
      "type": "BlockMaintainerChange",
      "lookback_days": 90,
      "severity": "high"
    },
    {
      "type": "RequireMinimumDownloads",
      "min_downloads": 10000,
      "apply_to_direct": true,
      "severity": "medium"
    }
  ]
}
EOF

# Apply the policy
sctv-cli policy create --file policy.json
```

### Policy Rule Types

| Rule Type | Description |
|-----------|-------------|
| `BlockDeprecated` | Block packages marked as deprecated |
| `BlockPackageAge` | Block packages published recently |
| `RequireProvenance` | Require SLSA provenance attestations |
| `RequireSignatures` | Require package signatures |
| `BlockMaintainerChange` | Alert on recent maintainer changes |
| `RequireMinimumDownloads` | Require packages to be popular |
| `BlockByLicense` | Block packages with specific licenses |
| `AllowlistPackages` | Only allow specific packages |
| `BlocklistPackages` | Block specific packages |

### Assign Policy to Project

```bash
# Via CLI
sctv-cli project update <project-id> \
  --policy-id <policy-id>

# Via Dashboard
# 1. Go to Project Settings
# 2. Select "Security Policy"
# 3. Choose policy from dropdown
# 4. Click "Save"
```

---

## Next Steps

Congratulations! You've completed the quick start. Here's what to do next:

### 1. Set Up Continuous Monitoring

Enable automated scans:

```bash
# Set scan schedule for a project
sctv-cli project update <project-id> \
  --scan-schedule daily \
  --scan-hour 2
```

### 2. Configure Notifications

Set up alerts for your team:

```bash
# Configure Slack notifications
sctv-cli notifications configure slack \
  --webhook-url "https://hooks.slack.com/services/xxx/yyy/zzz" \
  --channel "#security-alerts" \
  --min-severity high

# Configure email notifications
sctv-cli notifications configure email \
  --smtp-host "smtp.gmail.com" \
  --smtp-port 587 \
  --from "alerts@example.com" \
  --to "security-team@example.com"
```

See [Configuration Guide](configuration.md#notifications) for all notification options.

### 3. Integrate with CI/CD

Add SCTV to your build pipeline:

**GitHub Actions:**
```yaml
- name: SCTV Security Scan
  run: |
    sctv-cli scan --ecosystem npm --format sarif > sctv.sarif

- name: Upload SARIF
  uses: github/codeql-action/upload-sarif@v2
  with:
    sarif_file: sctv.sarif
```

**GitLab CI:**
```yaml
sctv_scan:
  script:
    - sctv-cli scan --ecosystem npm --format json > sctv-results.json
  artifacts:
    reports:
      dependency_scanning: sctv-results.json
```

See [Webhooks Documentation](../api/webhooks.md) for detailed CI/CD integration.

### 4. Generate SBOMs

Create Software Bill of Materials for compliance:

```bash
# Generate CycloneDX SBOM
sctv-cli sbom generate \
  --project-id <project-id> \
  --format cyclonedx \
  --output sbom.cdx.json

# Generate SPDX SBOM
sctv-cli sbom generate \
  --project-id <project-id> \
  --format spdx \
  --output sbom.spdx.json
```

### 5. Explore Advanced Features

- **Dashboard Analytics** - View trends and metrics
- **GraphQL API** - Build custom integrations
- **Custom Detectors** - Write your own threat detectors
- **Multi-Tenant Management** - Manage multiple organizations
- **Audit Logs** - Track all security-related actions

---

## Common Tasks

### Check a Specific Package

```bash
# Check if a package is a typosquat
sctv-cli check lodash-utils --ecosystem npm

# Verify package integrity
sctv-cli verify react --version 18.2.0 --ecosystem npm
```

### View Scan History

```bash
# List all scans
sctv-cli scans list --project-id <project-id>

# View specific scan results
sctv-cli scans get <scan-id>
```

### Export Alert Report

```bash
# Export all critical/high alerts as JSON
sctv-cli alerts export \
  --severity critical,high \
  --format json \
  --output security-report.json

# Export as CSV
sctv-cli alerts export \
  --format csv \
  --output alerts.csv
```

---

## Troubleshooting

### Scan Takes Too Long

**Solution:** Enable caching and parallel processing:

```bash
sctv-cli scan \
  --ecosystem npm \
  --cache-registry-data \
  --parallel-workers 8
```

### False Positives

**Solution:** Suppress specific alerts or create exemptions in your policy:

```json
{
  "rules": [
    {
      "type": "BlockPackageAge",
      "min_age_days": 30,
      "exemptions": [
        "@myorg/internal-package"
      ]
    }
  ]
}
```

### Missing Detections

**Solution:** Ensure all ecosystems are configured:

```bash
# List supported ecosystems
sctv-cli ecosystems list

# Add ecosystem to project
sctv-cli project update <project-id> \
  --add-ecosystem pypi
```

---

## Getting Help

- **Full Documentation:** [Documentation Index](../README.md)
- **Configuration Reference:** [Configuration Guide](configuration.md)
- **API Documentation:** [REST API](../api/rest-api.md) | [GraphQL](../api/graphql-api.md)
- **Community:** [GitHub Discussions](https://github.com/example/supply-chain-trust-verifier/discussions)

---

**You're all set!** 🎉 SCTV is now protecting your supply chain.
