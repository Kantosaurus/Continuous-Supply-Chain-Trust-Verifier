# User Guide

**Version:** 0.1.0

Welcome to the SCTV User Guide. This comprehensive guide helps you use SCTV effectively to protect your software supply chain.

---

## Table of Contents

### Getting Started
- [Dashboard Overview](dashboard.md) - Navigate the web interface
- [Projects](projects.md) - Create and manage projects
- [Running Scans](scanning.md) - Scan your dependencies

### Security Features
- [Alerts](alerts.md) - Understand and manage security alerts
- [Policies](policies.md) - Define and enforce security policies
- [SBOMs](sbom.md) - Generate Software Bill of Materials

### Best Practices
- [Security Best Practices](best-practices.md) - Recommended security practices
- [Policy Examples](policy-examples.md) - Common policy configurations
- [Workflow Integration](workflows.md) - Integrate SCTV into your workflow

---

## What Can SCTV Do?

### 1. Detect Supply Chain Threats

SCTV continuously monitors your dependencies for:

#### Typosquatting Attacks
Malicious packages with names similar to popular libraries:
- `lodash` vs `lodash-utils`
- `requests` vs `request`
- Visual similarity (e.g., `i` vs `l`, `0` vs `O`)

**Example Alert:**
```
CRITICAL: Typosquatting Detected
Package 'event-streem' (npm) is 92% similar to 'event-stream'
Recommendation: Remove 'event-streem' and use official 'event-stream'
```

#### Dependency Tampering
Packages with mismatched checksums indicating potential tampering:
- Downloaded hash doesn't match registry
- Package contents modified after publication
- Man-in-the-middle attack detection

**Example Alert:**
```
HIGH: Package Tampering Detected
Package 'axios@0.21.1' SHA-256 hash mismatch
Expected: abc123...
Actual:   def456...
Recommendation: Verify source and re-download from official registry
```

#### Downgrade Attacks
Suspicious version rollbacks that could reintroduce vulnerabilities:
- Major version downgrade (e.g., 5.0.0 → 4.9.0)
- Skipping security patches
- Unexpected version resolution

**Example Alert:**
```
MEDIUM: Downgrade Attack Detected
Package 'webpack' downgraded from 5.89.0 to 5.75.0
This version has 3 known vulnerabilities (CVE-2023-xxxx)
Recommendation: Update to latest version 5.89.0
```

#### Provenance Failures
Missing or invalid SLSA provenance attestations:
- No build provenance available
- Invalid signature
- Unverified build environment
- Failed SLSA verification

**Example Alert:**
```
HIGH: Provenance Verification Failed
Package 'critical-lib@2.0.0' has no SLSA provenance
Cannot verify build integrity
Recommendation: Request provenance from maintainer or use alternative
```

#### Policy Violations
Dependencies that violate your security policies:
- Deprecated packages
- Unmaintained packages
- License violations
- Insufficient popularity
- Recent maintainer changes

**Example Alert:**
```
MEDIUM: Policy Violation
Package 'colors@1.4.0' is deprecated
Policy 'Production Security' requires non-deprecated packages
Recommendation: Migrate to maintained alternative
```

#### New Package Warnings
Recently published packages (< 30 days) that may lack community vetting:
- New maintainer accounts
- Low download counts
- Minimal version history

**Example Alert:**
```
LOW: New Package Detected
Package 'super-new-lib@1.0.0' published 5 days ago
Account created: 7 days ago
Downloads: 147
Recommendation: Review code before use
```

#### Suspicious Maintainer Activity
Unusual maintainer behavior indicating potential compromise:
- Sudden maintainer changes
- Mass package updates
- Geographic anomalies
- Unusual publishing patterns

**Example Alert:**
```
HIGH: Suspicious Maintainer Activity
Package 'popular-lib' maintainer changed 2 days ago
Previous maintainer: 5+ years, new maintainer: 3 days old
Recommendation: Investigate maintainer change before updating
```

### 2. Enforce Security Policies

Create custom policies to enforce organizational security standards:

```yaml
Policy: Production Security
Rules:
  - Block deprecated packages (severity: high)
  - Require SLSA Level 2+ provenance for critical dependencies
  - Block packages < 30 days old (severity: medium)
  - Block recent maintainer changes (< 90 days)
  - Require minimum 10,000 downloads for new dependencies
  - Block specific licenses (GPL, AGPL)
```

### 3. Generate SBOMs

Create comprehensive Software Bill of Materials for compliance:

**Supported Formats:**
- CycloneDX 1.5 (JSON, XML)
- SPDX 2.3 (JSON, tag-value)

**Includes:**
- Complete dependency tree
- Package URLs (purls)
- License information
- Checksums and hashes
- Vulnerability mappings
- Build provenance

### 4. Continuous Monitoring

Automated monitoring keeps your projects secure:

**Scheduled Scans:**
- Hourly, daily, weekly, or custom schedules
- Automatic alerts on new threats
- Trend analysis and reporting

**Real-Time Notifications:**
- Email alerts
- Slack messages
- Microsoft Teams
- PagerDuty incidents
- Custom webhooks

### 5. CI/CD Integration

Integrate security checks into your build pipeline:

**SARIF Output:**
- GitHub Code Scanning
- GitLab SAST
- Azure DevOps

**Webhooks:**
- Automatic scans on push
- Pull request checks
- Build-time verification

---

## Key Concepts

### Projects

A **project** represents a software application with dependencies. Each project:
- Has a name and description
- Supports multiple package ecosystems (npm, PyPI, Maven, etc.)
- Has an optional security policy
- Can be scanned on a schedule or manually
- Tracks dependency history

**Example Project:**
```json
{
  "name": "E-Commerce API",
  "ecosystems": ["npm", "pypi"],
  "scan_schedule": "daily at 2 AM",
  "policy": "Production Security",
  "dependencies": 847,
  "status": "healthy"
}
```

### Dependencies

**Dependencies** are external packages your project uses. SCTV tracks:
- Direct dependencies (specified in manifest files)
- Transitive dependencies (dependencies of dependencies)
- Dependency tree structure
- Version constraints and resolutions
- Integrity hashes
- Provenance attestations

**Dependency Attributes:**
- Package name and version
- Ecosystem (npm, PyPI, Maven, etc.)
- Hash checksums (SHA-256, SHA-512)
- Signature status
- Provenance status (SLSA level)
- Alert count

### Alerts

**Alerts** are security findings that require attention. Each alert includes:
- Severity level (Critical, High, Medium, Low, Info)
- Alert type (typosquatting, tampering, etc.)
- Affected package
- Detection evidence
- Remediation steps
- Status (Open, Acknowledged, Resolved, Suppressed)

**Alert Lifecycle:**
1. **Created** - Detected during scan
2. **Open** - Awaiting triage
3. **Acknowledged** - Team is investigating
4. **Resolved** - Issue fixed
5. **Suppressed** - False positive or accepted risk

### Policies

**Policies** define security rules for your projects. Rules can:
- Block specific packages or patterns
- Require provenance attestations
- Enforce version constraints
- Check package age and popularity
- Monitor maintainer changes
- Validate licenses

**Policy Application:**
- Assign to one or more projects
- Set as default for new projects
- Enable/disable dynamically
- Override severity for specific packages

### SBOMs

**SBOMs** (Software Bill of Materials) are complete inventories of your software components:
- List all dependencies
- Include metadata and licenses
- Map vulnerabilities
- Provide chain-of-custody
- Enable compliance reporting

**Use Cases:**
- Security audits
- Compliance requirements (NTIA, EO 14028)
- Vulnerability management
- License compliance
- Procurement verification

---

## Common Workflows

### 1. Scan a New Project

```
1. Create project in dashboard
2. Configure ecosystems (npm, PyPI, etc.)
3. Optionally assign a policy
4. Trigger initial scan
5. Review alerts
6. Acknowledge or resolve findings
7. Set up scan schedule
```

### 2. Respond to an Alert

```
1. Receive notification (email, Slack, etc.)
2. Review alert details in dashboard
3. Investigate affected dependency
4. Determine if alert is valid
5. If valid:
   - Update/remove dependency
   - Mark as resolved with explanation
6. If false positive:
   - Suppress alert with reason
   - Add exemption to policy if recurring
```

### 3. Create a Security Policy

```
1. Navigate to Policies
2. Click "Create Policy"
3. Add security rules:
   - Block deprecated packages
   - Require provenance
   - Age restrictions
   - License requirements
4. Configure severity levels
5. Add package exemptions if needed
6. Test policy against a project
7. Assign to projects
```

### 4. Generate SBOM for Compliance

```
1. Select project
2. Click "Generate SBOM"
3. Choose format (CycloneDX or SPDX)
4. Select options:
   - Include dev dependencies
   - Include hashes
   - Include vulnerabilities
5. Download SBOM
6. Provide to compliance team/customers
```

### 5. Integrate with CI/CD

```
1. Generate API key
2. Add SCTV step to pipeline:
   - Install CLI
   - Run scan
   - Output SARIF
3. Configure failure conditions:
   - Fail on critical alerts
   - Fail on high severity
4. Upload results to CI platform
5. Review in PR checks
```

---

## Dashboard Overview

### Home Page

The dashboard home provides:
- **Project summary** - Total projects, scan status
- **Alert overview** - Breakdown by severity
- **Recent activity** - Latest scans and alerts
- **Trending** - Alert trends over time

### Navigation

- **Projects** - Manage your projects
- **Alerts** - View and triage security alerts
- **Policies** - Create and manage policies
- **SBOMs** - Generate and download SBOMs
- **Settings** - Configure notifications and integrations
- **Audit Log** - View all user actions

---

## Getting Help

### In-Product Help

- **Tooltips** - Hover over fields for explanations
- **Documentation Links** - Context-specific help links
- **Examples** - Sample configurations provided

### Support Resources

- **Documentation** - Complete guides and references
- **Community Forum** - Ask questions, share knowledge
- **GitHub Issues** - Report bugs and request features
- **Email Support** - support@example.com (Enterprise)

---

## Next Steps

- **[Dashboard Guide](dashboard.md)** - Learn to navigate the UI
- **[Projects Guide](projects.md)** - Create and manage projects
- **[Alerts Guide](alerts.md)** - Understand and respond to alerts
- **[Policies Guide](policies.md)** - Create security policies
- **[Best Practices](best-practices.md)** - Security recommendations

---

**Ready to secure your supply chain?** Start with the [Quick Start Guide](../getting-started/quickstart.md).
