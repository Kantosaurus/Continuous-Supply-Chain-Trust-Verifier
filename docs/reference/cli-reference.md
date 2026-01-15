# CLI Reference

**Version:** 0.1.0

Complete reference for the SCTV command-line interface.

---

## Table of Contents

- [Installation](#installation)
- [Global Options](#global-options)
- [Commands](#commands)
  - [scan](#scan)
  - [check](#check)
  - [verify](#verify)
  - [policy](#policy)
  - [project](#project)
  - [alert](#alert)
  - [sbom](#sbom)
  - [auth](#auth)
  - [config](#config)
- [Output Formats](#output-formats)
- [Exit Codes](#exit-codes)
- [Environment Variables](#environment-variables)
- [Examples](#examples)

---

## Installation

### Binary Installation

```bash
# Download latest release
curl -L https://github.com/example/sctv/releases/latest/download/sctv-linux-amd64 -o sctv
chmod +x sctv
sudo mv sctv /usr/local/bin/

# Verify installation
sctv --version
```

### Build from Source

```bash
git clone https://github.com/example/supply-chain-trust-verifier.git
cd supply-chain-trust-verifier
cargo build --release --bin sctv-cli
sudo cp target/release/sctv-cli /usr/local/bin/sctv
```

### Package Managers

```bash
# Homebrew (macOS/Linux)
brew install sctv

# Cargo
cargo install sctv-cli

# npm
npm install -g @sctv/cli
```

---

## Global Options

These options work with all commands:

```
--verbose, -v          Enable verbose output
--quiet, -q            Suppress output except errors
--format <FORMAT>      Output format: text, json, sarif (default: text)
--color <WHEN>         Colorize output: auto, always, never (default: auto)
--config <FILE>        Path to configuration file
--api-url <URL>        SCTV API URL (default: http://localhost:3000)
--api-key <KEY>        API key for authentication
--help, -h             Show help information
--version, -V          Show version information
```

---

## Commands

### scan

Scan a project for supply chain threats.

**Usage:**

```bash
sctv scan [OPTIONS] [PATH]
```

**Options:**

```
--path <PATH>              Project path (default: current directory)
--ecosystem <ECO>          Package ecosystem: npm, pypi, maven, nuget, cargo, gem, go
--output <FILE>            Write results to file
--format <FMT>             Output format: text, json, sarif
--fail-on <SEVERITY>       Exit with error code on severity: critical, high, medium, low
--cache                    Enable registry caching
--parallel <N>             Number of parallel workers (default: 4)
--policy <FILE>            Policy file to evaluate
--project-id <ID>          Associate scan with existing project
```

**Examples:**

```bash
# Scan current directory (auto-detect ecosystem)
sctv scan

# Scan specific path
sctv scan --path /path/to/project

# Scan npm project with JSON output
sctv scan --ecosystem npm --format json

# Scan and fail CI on critical alerts
sctv scan --fail-on critical

# Scan with custom policy
sctv scan --policy security-policy.json

# Generate SARIF for GitHub Code Scanning
sctv scan --format sarif --output sctv.sarif
```

**Output:**

```
Scanning project: /home/user/my-app
Ecosystem: npm
───────────────────────────────────────────────────────

✓ Discovered 847 dependencies (143 direct, 704 transitive)
✓ Verified checksums for 847 packages
✓ Checked provenance for 143 direct dependencies
✓ Evaluated security policy

Findings:
─────────────────────────────────────────────────────
🔴 CRITICAL (1):
  lodash-utils@1.0.0 - Typosquatting (similarity: 92% to 'lodash')
  → Remove 'lodash-utils' and use official 'lodash'

🟠 HIGH (3):
  webpack@5.75.0 - Provenance verification failed
  → No SLSA attestation found

  colors@1.4.0 - Deprecated package
  → Migrate to maintained alternative

  axios - Downgrade from 1.6.0 to 1.5.1
  → Review version constraint and update

🟡 MEDIUM (7):
  ... (7 items)

🔵 LOW (12):
  ... (12 items)

Total: 23 alerts (1 critical, 3 high, 7 medium, 12 low)
Scan completed in 18.4s
```

### check

Check if a package is a typosquatting attempt.

**Usage:**

```bash
sctv check <PACKAGE> [OPTIONS]
```

**Options:**

```
--ecosystem <ECO>          Package ecosystem (default: npm)
--verbose, -v              Show detailed similarity analysis
--threshold <SCORE>        Similarity threshold 0.0-1.0 (default: 0.85)
```

**Examples:**

```bash
# Check npm package
sctv check lodash-utils --ecosystem npm

# Check with verbose output
sctv check lodash-utils --verbose

# Check Python package
sctv check requestes --ecosystem pypi
```

**Output:**

```
Checking: lodash-utils (npm)
─────────────────────────────────────────────────────

⚠️  POTENTIAL TYPOSQUAT DETECTED

Target: lodash (99,850,234 weekly downloads)
Suspicious: lodash-utils (147 downloads)

Similarity Analysis:
  Levenshtein distance: 6
  Jaro-Winkler score: 0.92
  Visual similarity: HIGH

Evidence:
  ✗ Low download count (147 vs 99.8M)
  ✗ New maintainer (account created 7 days ago)
  ✗ Recently published (5 days ago)
  ✗ No README or documentation
  ⚠️  Contains obfuscated code

Recommendation: DO NOT USE
Use official 'lodash' package instead
```

### verify

Verify package integrity and provenance.

**Usage:**

```bash
sctv verify <PACKAGE> --version <VERSION> [OPTIONS]
```

**Options:**

```
--version <VER>            Package version (required)
--ecosystem <ECO>          Package ecosystem (default: npm)
--checksum <HASH>          Expected SHA-256 checksum
--verify-provenance        Verify SLSA provenance
--verify-signature         Verify package signature
```

**Examples:**

```bash
# Verify package integrity
sctv verify react --version 18.2.0 --ecosystem npm

# Verify with expected checksum
sctv verify react --version 18.2.0 --checksum abc123...

# Verify SLSA provenance
sctv verify react --version 18.2.0 --verify-provenance

# Complete verification
sctv verify react --version 18.2.0 \
  --verify-provenance \
  --verify-signature \
  --checksum abc123...
```

**Output:**

```
Verifying: react@18.2.0 (npm)
─────────────────────────────────────────────────────

✓ Package found in registry
✓ SHA-256 checksum verified
✓ SHA-512 checksum verified
✓ Package signature valid
✓ SLSA provenance verified (Level 2)

Package Details:
  Published: 2023-06-14T10:30:00Z
  Size: 485 KB
  License: MIT
  Maintainers: 12

Provenance:
  Builder: GitHub Actions
  SLSA Level: 2
  Build Type: github-actions@v1
  Source: github.com/facebook/react@v18.2.0

Status: ✅ VERIFIED
```

### policy

Evaluate or manage security policies.

**Usage:**

```bash
sctv policy <SUBCOMMAND>
```

**Subcommands:**

```
eval        Evaluate policy against project
create      Create new policy
list        List policies
get         Get policy details
update      Update policy
delete      Delete policy
validate    Validate policy file
```

**Examples:**

```bash
# Evaluate policy against current project
sctv policy eval --policy production.json

# Create policy from file
sctv policy create --file security-policy.json --name "Production"

# List all policies
sctv policy list

# Get policy details
sctv policy get <policy-id>

# Validate policy file
sctv policy validate --file policy.json
```

**Policy File Example:**

```json
{
  "name": "Production Security Policy",
  "description": "Security requirements for production",
  "rules": [
    {
      "type": "BlockDeprecated",
      "severity": "high"
    },
    {
      "type": "RequireProvenance",
      "min_slsa_level": 2,
      "apply_to": "direct"
    },
    {
      "type": "BlockPackageAge",
      "min_age_days": 30,
      "severity": "medium",
      "exemptions": ["@myorg/*"]
    },
    {
      "type": "RequireMinimumDownloads",
      "min_downloads": 10000,
      "apply_to_direct": true
    }
  ]
}
```

### project

Manage projects.

**Usage:**

```bash
sctv project <SUBCOMMAND>
```

**Subcommands:**

```
create      Create new project
list        List projects
get         Get project details
update      Update project
delete      Delete project
scan        Trigger project scan
```

**Examples:**

```bash
# Create project
sctv project create \
  --name "My App" \
  --ecosystem npm \
  --policy production-policy

# List all projects
sctv project list

# Get project details
sctv project get <project-id>

# Update project
sctv project update <project-id> \
  --scan-schedule daily \
  --scan-hour 2

# Trigger scan
sctv project scan <project-id>

# Delete project
sctv project delete <project-id>
```

### alert

Manage security alerts.

**Usage:**

```bash
sctv alert <SUBCOMMAND>
```

**Subcommands:**

```
list           List alerts
get            Get alert details
acknowledge    Acknowledge alert
resolve        Resolve alert
suppress       Suppress false positive
export         Export alerts to file
```

**Examples:**

```bash
# List all open critical/high alerts
sctv alert list --severity critical,high --status open

# List alerts for specific project
sctv alert list --project-id <id>

# Get alert details
sctv alert get <alert-id>

# Acknowledge alert
sctv alert acknowledge <alert-id> \
  --comment "Investigating with security team"

# Resolve alert
sctv alert resolve <alert-id> \
  --resolution "Updated to safe version" \
  --comment "Fixed in PR #123"

# Suppress false positive
sctv alert suppress <alert-id> \
  --reason "Internal package, not a typosquat"

# Export alerts as JSON
sctv alert export --format json --output alerts.json

# Export alerts as CSV
sctv alert export --format csv --output alerts.csv
```

### sbom

Generate Software Bill of Materials.

**Usage:**

```bash
sctv sbom generate [OPTIONS]
```

**Options:**

```
--project-id <ID>          Project ID (required)
--format <FMT>             Format: cyclonedx, cyclonedx-xml, spdx, spdx-tag
--output <FILE>            Output file path
--include-dev              Include dev dependencies
--include-hashes           Include checksums
--include-licenses         Include license information
--include-vulns            Include known vulnerabilities
```

**Examples:**

```bash
# Generate CycloneDX SBOM
sctv sbom generate \
  --project-id <id> \
  --format cyclonedx \
  --output sbom.cdx.json

# Generate SPDX SBOM with all metadata
sctv sbom generate \
  --project-id <id> \
  --format spdx \
  --include-dev \
  --include-hashes \
  --include-licenses \
  --include-vulns \
  --output sbom.spdx.json

# Generate XML format
sctv sbom generate \
  --project-id <id> \
  --format cyclonedx-xml \
  --output sbom.xml
```

### auth

Authentication management.

**Usage:**

```bash
sctv auth <SUBCOMMAND>
```

**Subcommands:**

```
login          Login and get JWT token
logout         Clear stored credentials
status         Show authentication status
create-key     Create API key
list-keys      List API keys
revoke-key     Revoke API key
```

**Examples:**

```bash
# Login
sctv auth login --email user@example.com

# Check auth status
sctv auth status

# Create API key
sctv auth create-key --name "CI Pipeline" --expires-in-days 365

# List API keys
sctv auth list-keys

# Revoke API key
sctv auth revoke-key <key-id>

# Logout
sctv auth logout
```

### config

Configuration management.

**Usage:**

```bash
sctv config <SUBCOMMAND>
```

**Subcommands:**

```
show           Show current configuration
set            Set configuration value
get            Get configuration value
validate       Validate configuration file
test-db        Test database connection
```

**Examples:**

```bash
# Show configuration
sctv config show

# Set API URL
sctv config set api.url https://sctv.example.com

# Get configuration value
sctv config get api.url

# Validate configuration file
sctv config validate --config /etc/sctv/config.toml

# Test database connection
sctv config test-db
```

---

## Output Formats

### text (default)

Human-readable terminal output with colors and formatting.

### json

Structured JSON output for programmatic parsing:

```json
{
  "scan_id": "uuid",
  "project": {
    "name": "My App",
    "path": "/path/to/project"
  },
  "summary": {
    "dependencies": 847,
    "alerts": 23,
    "by_severity": {
      "critical": 1,
      "high": 3,
      "medium": 7,
      "low": 12
    }
  },
  "alerts": [
    {
      "id": "uuid",
      "type": "typosquatting",
      "severity": "critical",
      "title": "Typosquatting Detected",
      "package": "lodash-utils",
      "details": {...}
    }
  ],
  "completed_at": "2026-01-15T10:30:00Z",
  "duration_seconds": 18.4
}
```

### sarif

SARIF 2.1.0 format for CI/CD integration:

```json
{
  "$schema": "https://json.schemastore.org/sarif-2.1.0.json",
  "version": "2.1.0",
  "runs": [
    {
      "tool": {
        "driver": {
          "name": "Supply Chain Trust Verifier",
          "version": "0.1.0"
        }
      },
      "results": [...]
    }
  ]
}
```

---

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success, no critical issues |
| 1 | Critical alerts found (when using `--fail-on critical`) |
| 2 | High severity alerts found (when using `--fail-on high`) |
| 3 | Command error (invalid arguments, etc.) |
| 4 | Authentication error |
| 5 | Network/API error |
| 6 | File system error |
| 10 | Unspecified error |

---

## Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `SCTV_API_URL` | API server URL | `http://localhost:3000` |
| `SCTV_API_KEY` | API key for authentication | - |
| `SCTV_CONFIG` | Path to config file | `~/.config/sctv/config.toml` |
| `SCTV_LOG_LEVEL` | Log level: error, warn, info, debug | `info` |
| `SCTV_NO_COLOR` | Disable colored output | `false` |
| `SCTV_CACHE_DIR` | Cache directory | `~/.cache/sctv` |

---

## Examples

### CI/CD Integration

**GitHub Actions:**

```yaml
name: SCTV Security Scan

on: [push, pull_request]

jobs:
  security-scan:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Install SCTV
        run: |
          curl -L https://github.com/example/sctv/releases/latest/download/sctv-linux-amd64 -o sctv
          chmod +x sctv
          sudo mv sctv /usr/local/bin/

      - name: Run security scan
        env:
          SCTV_API_KEY: ${{ secrets.SCTV_API_KEY }}
        run: |
          sctv scan \
            --format sarif \
            --output sctv.sarif \
            --fail-on critical

      - name: Upload SARIF
        uses: github/codeql-action/upload-sarif@v2
        with:
          sarif_file: sctv.sarif
```

**GitLab CI:**

```yaml
sctv_scan:
  stage: test
  image: alpine:latest
  before_script:
    - apk add --no-cache curl
    - curl -L https://github.com/example/sctv/releases/latest/download/sctv-linux-amd64 -o /usr/local/bin/sctv
    - chmod +x /usr/local/bin/sctv
  script:
    - sctv scan --format json --output sctv-results.json --fail-on high
  artifacts:
    reports:
      dependency_scanning: sctv-results.json
```

### Local Development

```bash
# Scan before committing
sctv scan --fail-on high

# Check a new dependency
sctv check new-package-name

# Verify package integrity
sctv verify package-name --version 1.2.3 --verify-provenance

# Generate SBOM for documentation
sctv sbom generate \
  --project-id <id> \
  --format cyclonedx \
  --output docs/sbom.json
```

### Automation Scripts

```bash
#!/bin/bash
# daily-scan.sh - Daily security scan

# Set credentials
export SCTV_API_KEY="your-api-key"

# Scan all projects
for project_id in $(sctv project list --format json | jq -r '.data[].id'); do
  echo "Scanning project: $project_id"
  sctv project scan "$project_id"
done

# Export critical alerts
sctv alert export \
  --severity critical,high \
  --status open \
  --format csv \
  --output "/reports/alerts-$(date +%Y%m%d).csv"
```

---

## Next Steps

- [Configuration Reference](configuration.md) - All configuration options
- [API Reference](../api/rest-api.md) - REST API documentation
- [Error Codes](error-codes.md) - Detailed error code reference
