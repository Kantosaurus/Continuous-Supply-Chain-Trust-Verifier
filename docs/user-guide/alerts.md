# Alert Management Guide

**Version:** 0.1.0
**Last Updated:** 2026-01-15

Alerts are security findings that require your attention. This comprehensive guide covers understanding, managing, and responding to alerts in SCTV.

---

## Table of Contents

- [Understanding Alerts](#understanding-alerts)
- [Alert Types](#alert-types)
- [Severity Levels](#severity-levels)
- [Viewing Alert Details](#viewing-alert-details)
- [Acknowledging Alerts](#acknowledging-alerts)
- [Resolving Alerts](#resolving-alerts)
- [Alert Filtering and Search](#alert-filtering-and-search)
- [Bulk Operations](#bulk-operations)
- [Alert Notifications](#alert-notifications)
- [Alert Lifecycle](#alert-lifecycle)

---

## Understanding Alerts

### What is an Alert?

An **alert** is a security finding generated when SCTV detects a supply chain threat or policy violation in your dependencies.

**Each Alert Contains:**
- **Type** - Category of threat (typosquatting, tampering, etc.)
- **Severity** - Impact level (Critical, High, Medium, Low, Info)
- **Title** - Brief description
- **Description** - Detailed explanation
- **Evidence** - Detection details and proof
- **Remediation** - Recommended actions
- **Status** - Current state (Open, Acknowledged, Resolved, etc.)
- **Metadata** - Related packages, timestamps, users

### Alert Structure

**Example Alert:**
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "project_id": "123e4567-e89b-12d3-a456-426614174000",
  "alert_type": "typosquatting",
  "severity": "critical",
  "title": "Typosquatting Detected: event-streem",
  "description": "Package 'event-streem' is 92% similar to popular package 'event-stream' (50M+ downloads/week)",
  "status": "open",
  "created_at": "2026-01-15T10:30:00Z",
  "dependency": {
    "name": "event-streem",
    "version": "1.0.0",
    "ecosystem": "npm"
  },
  "details": {
    "suspicious_package": "event-streem",
    "similar_popular_package": "event-stream",
    "similarity_score": 0.92,
    "detection_method": "levenshtein",
    "popular_package_downloads": 50000000
  },
  "remediation": {
    "action": "Remove 'event-streem' and install official 'event-stream'",
    "commands": [
      "npm uninstall event-streem",
      "npm install event-stream"
    ]
  }
}
```

---

## Alert Types

SCTV detects seven categories of supply chain threats.

### 1. Typosquatting

**Description:** Malicious packages with names similar to popular packages.

**Detection Methods:**
- Levenshtein distance (edit distance)
- Damerau-Levenshtein (transpositions)
- Jaro-Winkler similarity
- Phonetic matching
- Keyboard distance (adjacent keys)
- Combosquatting (added prefixes/suffixes)

**Example Alert:**
```
CRITICAL: Typosquatting Detected

Package: event-streem (npm)
Similar to: event-stream (50M+ downloads/week)
Similarity: 92% (one character different)
Method: Levenshtein distance

Evidence:
  event-stream → event-streem (replaced 'a' with 'ee')
  event-stream: 50,000,000 downloads/week
  event-streem: 147 downloads (published 5 days ago)

Risk:
  High - This is a popular target for supply chain attacks
  The official package 'event-stream' was previously compromised in 2018

Remediation:
  1. Immediately remove 'event-streem' from dependencies
  2. Install official 'event-stream' package
  3. Audit code for any malicious behavior
  4. Review dependency lock file

Commands:
  npm uninstall event-streem
  npm install event-stream
```

**Common Patterns:**
- `lodash` vs `lodash-utils`, `lodash-node`
- `request` vs `requests`, `reqwest`
- `axios` vs `axois`, `axiom`
- Visual confusion: `i` vs `l`, `0` vs `O`, `rn` vs `m`

### 2. Dependency Tampering

**Description:** Package contents don't match expected cryptographic hash.

**Causes:**
- Package modified after publication
- Man-in-the-middle attack
- Registry compromise
- Mirror/proxy tampering
- Local cache corruption

**Example Alert:**
```
CRITICAL: Dependency Tampering Detected

Package: axios@0.21.1 (npm)
Expected Hash: sha256:abc123def456...
Actual Hash:   sha256:xyz789abc123...
Algorithm: SHA-256
Source: registry.npmjs.org

Evidence:
  Downloaded package hash does not match registry metadata
  Package may have been modified or corrupted

Possible Causes:
  1. Package tampering (malicious modification)
  2. Man-in-the-middle attack
  3. Corrupted download
  4. Registry compromise
  5. Mirror/proxy issue

Remediation:
  IMMEDIATE ACTION REQUIRED:
  1. DO NOT USE this package version
  2. Delete the package from cache
  3. Clear npm cache: npm cache clean --force
  4. Re-download from official registry
  5. If hash still mismatches, report to npm security
  6. Consider using a different version

Commands:
  rm -rf node_modules/axios
  npm cache clean --force
  npm install axios@0.21.1 --registry=https://registry.npmjs.org
```

**Verification Process:**
```
1. Download package from registry
2. Calculate SHA-256 hash
3. Compare with registry metadata
4. If mismatch → Generate alert
5. Block installation (if policy enforced)
```

### 3. Downgrade Attack

**Description:** Package version was unexpectedly downgraded.

**Indicators:**
- Major version decrease (5.0.0 → 4.9.0)
- Minor version decrease (4.5.0 → 4.3.0)
- Patch version decrease (4.5.3 → 4.5.1)
- Lock file version differs from manifest

**Example Alert:**
```
HIGH: Downgrade Attack Detected

Package: webpack (npm)
Previous Version: 5.89.0
Current Version: 5.75.0
Lock File: 5.89.0
Downgrade Type: Patch downgrade

Evidence:
  Version decreased from 5.89.0 to 5.75.0
  Lock file still references 5.89.0
  This may reintroduce known vulnerabilities

Known Issues in 5.75.0:
  CVE-2023-1234 (High) - Path traversal vulnerability
  CVE-2023-5678 (Medium) - Denial of service
  CVE-2023-9012 (Low) - Information disclosure

These were fixed in versions 5.76.0 - 5.89.0

Remediation:
  1. Investigate why version was downgraded
  2. Update to latest version (5.89.0)
  3. Verify package-lock.json matches package.json
  4. Check for dependency conflicts

Commands:
  npm install webpack@5.89.0
  npm audit fix
```

**Detection Logic:**
```javascript
if (current_version < previous_version) {
  severity = calculateSeverity(
    versionDiff,
    knownVulnerabilities,
    lockFileMismatch
  );
  generateAlert("downgrade_attack", severity);
}
```

### 4. Provenance Failure

**Description:** Package lacks valid build provenance or SLSA attestation.

**SLSA Levels:**
- **Level 0** - No provenance
- **Level 1** - Provenance exists
- **Level 2** - Provenance verified, build service
- **Level 3** - Hardened build platform
- **Level 4** - Audited build platform (highest)

**Example Alert:**
```
MEDIUM: Provenance Failure

Package: critical-lib@2.0.0 (npm)
Expected SLSA Level: 2+
Actual SLSA Level: None
Attestation Status: Not available

Evidence:
  Package has no SLSA provenance attestation
  Cannot verify build integrity or source
  No cryptographic signature found

Project Policy:
  "Production Security" requires SLSA Level 2+ for critical dependencies
  This package is marked as critical dependency

Risks:
  Cannot verify:
    - Build process integrity
    - Source code repository
    - Build environment security
    - Reproducible builds

Remediation:
  1. Contact package maintainer to request provenance
  2. Consider alternative packages with provenance
  3. Add package to policy exemptions if trusted
  4. Perform manual source code audit

Alternative Packages with SLSA Level 2+:
  - alternative-lib@3.1.0 (SLSA 2)
  - secure-lib@1.5.0 (SLSA 3)
```

**Verification:**
```yaml
Check for:
  - SLSA attestation bundle
  - Sigstore signature
  - In-toto provenance
  - NPM provenance (npm v9+)
  - Package signature (GPG)

If missing → Generate alert based on policy
```

### 5. Policy Violation

**Description:** Dependency violates custom security policy rules.

**Rule Types:**
- Version constraints
- License restrictions
- Age requirements
- Download thresholds
- Maintainer requirements
- Ecosystem blocks
- Package deny lists

**Example Alert:**
```
HIGH: Policy Violation

Package: colors@1.4.0 (npm)
Policy: Production Security
Rule: Block deprecated packages
Severity: High

Evidence:
  Package is marked as DEPRECATED on npm registry
  Deprecation message: "Please use chalk instead"
  Last publish: 2 years ago
  No security updates available

Policy Details:
  Rule Type: Block deprecated packages
  Configured Severity: High
  Action: Block installation in production

Deprecation Info:
  Reason: Maintainer recommends migration to 'chalk'
  Deprecated since: 2022-03-15
  Replacement: chalk@5.2.0

Remediation:
  1. Remove 'colors' from dependencies
  2. Install replacement package 'chalk'
  3. Update code to use chalk API
  4. Test thoroughly before deployment

Migration Example:
  // Old (colors)
  console.log('Hello'.red);

  // New (chalk)
  import chalk from 'chalk';
  console.log(chalk.red('Hello'));

Commands:
  npm uninstall colors
  npm install chalk
```

**Policy Evaluation:**
```
For each dependency:
  1. Load applicable policies
  2. Evaluate each rule
  3. If violation found:
     - Generate alert
     - Apply rule severity
     - Block if configured
```

### 6. New Package Warning

**Description:** Recently published package (< 30 days old).

**Risk Factors:**
- Low download count
- New maintainer account
- Minimal version history
- No community vetting
- Potential typosquatting

**Example Alert:**
```
MEDIUM: New Package Detected

Package: super-new-lib@1.0.0 (npm)
Published: 5 days ago
Age Threshold: 30 days (policy requirement)

Evidence:
  Package creation: 2026-01-10
  Current date: 2026-01-15
  Age: 5 days (under 30 day threshold)

Package Statistics:
  Total downloads: 147
  Maintainers: 1
  Versions published: 1
  Repository: github.com/newuser/super-new-lib
  License: MIT

Maintainer Info:
  Username: newuser2026
  Account created: 7 days ago (2026-01-08)
  Other packages: 0
  GitHub profile: Created Jan 2026

Risk Assessment:
  Age: Very new (5 days)
  Popularity: Very low (147 downloads)
  Maintainer: New account (7 days)
  History: No version history

Recommendations:
  1. Review package source code carefully
  2. Check for typosquatting similarities
  3. Verify maintainer legitimacy
  4. Monitor for community feedback
  5. Consider waiting 30 days before use

If package is legitimate:
  - Suppress this alert with justification
  - Add to policy exemptions
```

**Detection Criteria:**
```yaml
Alert if:
  package_age_days < threshold AND (
    downloads < minimum OR
    maintainer_age_days < 90 OR
    version_count < 3
  )
```

### 7. Suspicious Maintainer Activity

**Description:** Unusual maintainer behavior indicating potential compromise.

**Indicators:**
- Recent maintainer change
- Mass updates across packages
- Geographic anomalies
- Unusual publishing patterns
- Maintainer account compromise

**Example Alert:**
```
HIGH: Suspicious Maintainer Activity

Package: popular-lib@4.5.0 (npm)
Activity Type: Maintainer change
Risk Level: High

Evidence:
  Maintainer changed 2 days ago (2026-01-13)
  Previous maintainer: original-dev (5+ years)
  New maintainer: new-contributor (account age: 3 days)

Previous Maintainer:
  Username: original-dev
  Account age: 5 years 3 months
  Packages maintained: 23
  Total downloads: 500M+
  Last activity: 2026-01-13 (last update before change)

New Maintainer:
  Username: new-contributor
  Account age: 3 days (created 2026-01-12)
  Packages maintained: 1 (just this one)
  Total downloads: 0 (no prior packages)
  GitHub: No public profile

Suspicious Indicators:
  ⚠ Maintainer change very recent
  ⚠ New maintainer has very new account
  ⚠ Original maintainer had long history
  ⚠ No public announcement of change
  ⚠ New maintainer has no track record

Recommendations:
  IMMEDIATE ACTION:
  1. DO NOT update to versions after maintainer change
  2. Pin to last version by original maintainer
  3. Monitor package for suspicious changes
  4. Contact original maintainer if possible
  5. Consider forking or using alternative

Investigation:
  1. Check npm security advisories
  2. Review recent commit history
  3. Verify maintainer change was legitimate
  4. Search for community discussions

Safe Version: popular-lib@4.4.9 (last by original-dev)
```

---

## Severity Levels

SCTV uses five severity levels to prioritize alerts.

### Critical

**Definition:** Immediate security risk requiring urgent action.

**Examples:**
- Typosquatting of popular packages
- Dependency tampering (hash mismatch)
- Active exploitation in the wild
- Deny list violations

**Response Time:** Immediate (within 1 hour)

**Actions:**
- Block deployment
- Trigger PagerDuty
- Immediate notification
- Security team involvement

**Icon:** `[!]` Red exclamation

### High

**Definition:** Significant security risk needing prompt attention.

**Examples:**
- Downgrade attacks
- Provenance failures on critical dependencies
- Suspicious maintainer changes
- Major policy violations

**Response Time:** Within 24 hours

**Actions:**
- Review before next deployment
- Email notification
- Slack alert
- Include in daily standup

**Icon:** `[⚠]` Orange warning triangle

### Medium

**Definition:** Moderate security concern requiring investigation.

**Examples:**
- New package warnings
- Minor policy violations
- Provenance failures on non-critical dependencies
- Outdated dependencies

**Response Time:** Within 1 week

**Actions:**
- Include in sprint planning
- Weekly digest notification
- Schedule remediation

**Icon:** `[●]` Yellow circle

### Low

**Definition:** Minor security issue or best practice recommendation.

**Examples:**
- Optional dependency issues
- Dev dependency concerns
- License compliance (non-blocking)
- Performance recommendations

**Response Time:** As time permits

**Actions:**
- Include in backlog
- Monthly review
- Optional notification

**Icon:** `[i]` Blue info icon

### Info

**Definition:** Informational notice, no action required.

**Examples:**
- Package updates available
- Best practice tips
- Scan completion notices
- Policy evaluation results

**Response Time:** No action needed

**Actions:**
- Log only
- No notifications
- Dashboard display

**Icon:** `[·]` Gray dot

### Severity Override

**Admin users can override severity:**

```yaml
# Example: Downgrade for known false positive
Alert: Typosquatting on 'lodash-es'
Original Severity: Critical
Override Severity: Info
Reason: "lodash-es is official ESM version of lodash"
```

---

## Viewing Alert Details

### Alert List View

**Access:** Dashboard → Alerts

```
┌─────────────────────────────────────────────────────────────┐
│ Alerts (18 open)                  [Search] [Filter] [Actions]│
├─────────────────────────────────────────────────────────────┤
│                                                             │
│ ┌─────────────────────────────────────────────────────────┐ │
│ │ [!] CRITICAL                               2 minutes ago │ │
│ │ Typosquatting Detected: event-streem                    │ │
│ │ Package 'event-streem' (npm) is 92% similar to popular  │ │
│ │ package 'event-stream' (50M+ downloads/week)            │ │
│ │                                                         │ │
│ │ Project: E-Commerce API                                 │ │
│ │ Dependency: event-streem@1.0.0                          │ │
│ │ Detection: Levenshtein distance                         │ │
│ │                                                         │ │
│ │ [Acknowledge] [Resolve] [Suppress] [View Details]       │ │
│ └─────────────────────────────────────────────────────────┘ │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

### Alert Details Modal

**Click "View Details" for full information:**

```
┌─────────────────────────────────────────────────────────────┐
│ Alert Details                                           [✕] │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│ Typosquatting Detected: event-streem                        │
│ [!] CRITICAL                     Created: 2 minutes ago     │
│                                                             │
│ Overview                                                    │
│ ─────────────────────────────────────────────────────────── │
│ Type: Typosquatting                                         │
│ Status: Open                                                │
│ Project: E-Commerce API                                     │
│ Dependency: event-streem@1.0.0 (npm)                        │
│                                                             │
│ Detection Details                                           │
│ ─────────────────────────────────────────────────────────── │
│ Suspicious Package: event-streem                            │
│ Similar Package: event-stream                               │
│ Similarity Score: 92%                                       │
│ Detection Method: Levenshtein distance                      │
│                                                             │
│ Package Comparison                                          │
│ ─────────────────────────────────────────────────────────── │
│ event-stream (legitimate):                                  │
│   Downloads: 50,000,000/week                                │
│   Age: 8 years                                              │
│   Maintainers: 3 (verified)                                 │
│   GitHub Stars: 15,234                                      │
│                                                             │
│ event-streem (suspicious):                                  │
│   Downloads: 147 total                                      │
│   Age: 5 days                                               │
│   Maintainers: 1 (new account)                              │
│   GitHub: No repository                                     │
│                                                             │
│ Evidence                                                    │
│ ─────────────────────────────────────────────────────────── │
│ • One character difference (a → ee)                         │
│ • Published shortly after event-stream incident             │
│ • New maintainer account (3 days old)                       │
│ • No legitimate use case for this name                      │
│ • Appears in no major projects                              │
│                                                             │
│ Remediation                                                 │
│ ─────────────────────────────────────────────────────────── │
│ IMMEDIATE ACTION REQUIRED:                                  │
│                                                             │
│ 1. Remove malicious package:                                │
│    npm uninstall event-streem                               │
│                                                             │
│ 2. Install legitimate package:                              │
│    npm install event-stream                                 │
│                                                             │
│ 3. Audit your codebase:                                     │
│    - Search for references to 'event-streem'                │
│    - Check for suspicious code                              │
│    - Review recent commits                                  │
│                                                             │
│ 4. Update lock file:                                        │
│    rm package-lock.json                                     │
│    npm install                                              │
│                                                             │
│ Related Alerts                                              │
│ ─────────────────────────────────────────────────────────── │
│ (none)                                                      │
│                                                             │
│ History                                                     │
│ ─────────────────────────────────────────────────────────── │
│ • Created by system (2 min ago)                             │
│ • Severity: Critical                                        │
│ • First detected: 2026-01-15 10:30:00 UTC                   │
│                                                             │
│ [Acknowledge] [Resolve] [Suppress] [Export] [Share]         │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

### Alert Metadata

**Additional Information:**
- Alert ID (UUID)
- Scan ID that generated alert
- First detection timestamp
- Occurrence count (if recurring)
- Related alerts
- External references (CVEs, advisories)

---

## Acknowledging Alerts

**Purpose:** Indicate you're aware and investigating the alert.

### How to Acknowledge

**Via Dashboard:**

1. Navigate to alert
2. Click "Acknowledge" button
3. Optionally add notes
4. Confirm

**Via CLI:**
```bash
sctv alert acknowledge <alert-id> \
  --notes "Investigating with security team"
```

**Via API:**
```bash
curl -X POST https://sctv.example.com/api/v1/alerts/{id}/acknowledge \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"notes": "Investigating with security team"}'
```

### Acknowledgment Dialog

```
┌─────────────────────────────────────────────┐
│ Acknowledge Alert                           │
├─────────────────────────────────────────────┤
│                                             │
│ Alert: Typosquatting Detected (CRITICAL)    │
│                                             │
│ Notes (optional):                           │
│ ┌─────────────────────────────────────────┐ │
│ │ Investigating with security team.       │ │
│ │ Will update lock file and re-scan       │ │
│ │ in test environment first.              │ │
│ └─────────────────────────────────────────┘ │
│                                             │
│ This will:                                  │
│ • Change status to "Acknowledged"           │
│ • Record your username and timestamp        │
│ • Add note to alert history                 │
│ • Notify assigned users                     │
│                                             │
│       [Cancel]  [Acknowledge Alert]         │
│                                             │
└─────────────────────────────────────────────┘
```

### What Happens

**Status Changes:**
- `Open` → `Acknowledged`

**Records:**
- Username of acknowledger
- Timestamp
- Notes (if provided)

**Notifications:**
- Team members notified
- Added to audit log

---

## Resolving Alerts

**Purpose:** Mark alert as fixed with details of resolution.

### Resolution Types

1. **Fixed** - Issue resolved by updating/removing dependency
2. **Mitigated** - Risk reduced through other means
3. **Accepted Risk** - Decided to accept the risk
4. **False Positive** - Alert was incorrect

### How to Resolve

**Via Dashboard:**

1. Navigate to alert
2. Click "Resolve" button
3. Fill in resolution details
4. Confirm

**Resolution Dialog:**
```
┌─────────────────────────────────────────────┐
│ Resolve Alert                               │
├─────────────────────────────────────────────┤
│                                             │
│ Alert: Dependency Tampering (CRITICAL)      │
│ Package: axios@0.21.1                       │
│                                             │
│ Resolution Type: *                          │
│ ○ Fixed - Issue resolved                    │
│ ○ Mitigated - Risk reduced                  │
│ ○ Accepted Risk - Acknowledged but accepted │
│ ○ False Positive - Alert was incorrect      │
│                                             │
│ Action Taken: *                             │
│ ┌─────────────────────────────────────────┐ │
│ │ Updated axios to 1.6.2                  │ │
│ │ Verified hash matches registry          │ │
│ │ Cleared npm cache and reinstalled       │ │
│ └─────────────────────────────────────────┘ │
│                                             │
│ New Version (if updated):                   │
│ [1.6.2                     ]                │
│                                             │
│ Notes:                                      │
│ ┌─────────────────────────────────────────┐ │
│ │ Tested in staging environment.          │ │
│ │ All integration tests passing.          │ │
│ │ Ready for production deployment.        │ │
│ └─────────────────────────────────────────┘ │
│                                             │
│ ☑ Run verification scan after resolution    │
│                                             │
│       [Cancel]  [Resolve Alert]             │
│                                             │
└─────────────────────────────────────────────┘
```

**Via CLI:**
```bash
sctv alert resolve <alert-id> \
  --action "Updated axios to 1.6.2" \
  --new-version "1.6.2" \
  --notes "Tested in staging, ready for production" \
  --verify
```

### Verification Scan

**Optional:** Trigger a scan after resolution to verify the fix.

```bash
# Resolve and verify
sctv alert resolve <alert-id> \
  --action "Updated package" \
  --verify

# SCTV will:
1. Mark alert as resolved
2. Trigger project scan
3. Verify alert no longer appears
4. Update alert status
5. Notify team
```

---

## Alert Filtering and Search

### Quick Filters

**Pre-defined Filters:**
- My Alerts
- Critical Only
- Open Alerts
- Unassigned
- Last 24 Hours
- Last 7 Days

### Advanced Filtering

**Filter Panel:**
```
┌─────────────────────────────────────────┐
│ Filter Alerts                           │
├─────────────────────────────────────────┤
│                                         │
│ Status:                                 │
│ ☑ Open                                  │
│ ☐ Acknowledged                          │
│ ☐ Investigating                         │
│ ☐ Resolved                              │
│ ☐ Suppressed                            │
│ ☐ False Positive                        │
│                                         │
│ Severity:                               │
│ ☑ Critical                              │
│ ☑ High                                  │
│ ☐ Medium                                │
│ ☐ Low                                   │
│ ☐ Info                                  │
│                                         │
│ Type:                                   │
│ ☐ Typosquatting                         │
│ ☐ Dependency Tampering                  │
│ ☐ Downgrade Attack                      │
│ ☐ Provenance Failure                    │
│ ☐ Policy Violation                      │
│ ☐ New Package                           │
│ ☐ Suspicious Maintainer                 │
│                                         │
│ Project:                                │
│ [Select projects...         ▼]          │
│                                         │
│ Date Range:                             │
│ From: [2026-01-01] To: [2026-01-15]     │
│                                         │
│ Assigned To:                            │
│ [Anyone                     ▼]          │
│                                         │
│ [Clear] [Apply Filters]                 │
│                                         │
└─────────────────────────────────────────┘
```

### Search Syntax

**Simple Search:**
```
axios           - Find "axios" anywhere
"exact match"   - Exact phrase
```

**Field Search:**
```
title:typosquatting          - In alert title
description:hash             - In description
package:axios                - Package name
project:"E-Commerce API"     - Project name
severity:critical            - Severity level
status:open                  - Alert status
type:tampering               - Alert type
```

**Boolean Operators:**
```
axios AND critical           - Both terms
critical OR high             - Either term
NOT resolved                 - Exclude term
```

**Wildcards:**
```
typo*                        - Starts with "typo"
*squatting                   - Ends with "squatting"
*tampering*                  - Contains "tampering"
```

**Date Ranges:**
```
created:2026-01-15           - Specific date
created:>2026-01-01          - After date
created:<2026-01-15          - Before date
created:2026-01-01..2026-01-15 - Date range
```

**Examples:**
```
# Critical alerts in production projects
severity:critical project:*production*

# Open typosquatting alerts
type:typosquatting status:open

# Alerts from last week not yet resolved
created:>2026-01-08 status:open

# High or critical alerts on axios
package:axios (severity:critical OR severity:high)
```

---

## Bulk Operations

**Purpose:** Perform actions on multiple alerts simultaneously.

### Selection Methods

**Manual Selection:**
- Click checkboxes on individual alerts
- Shift+click to select range
- Ctrl/Cmd+A to select all visible

**Select by Filter:**
1. Apply filters
2. Click "Select All Matching" (selects all, not just visible)

### Available Bulk Actions

```
┌─────────────────────────────────────────┐
│ Bulk Actions (23 alerts selected)       │
├─────────────────────────────────────────┤
│                                         │
│ ☐ Acknowledge Selected                  │
│ ☐ Resolve Selected                      │
│ ☐ Suppress Selected                     │
│ ☐ Assign To User                        │
│ ☐ Change Severity                       │
│ ☐ Export Selected                       │
│ ☐ Delete Selected (admin only)          │
│                                         │
│ [Cancel] [Apply Action]                 │
│                                         │
└─────────────────────────────────────────┘
```

### Bulk Acknowledge

```bash
# Via CLI
sctv alert acknowledge --filter "status:open severity:critical" \
  --notes "Batch acknowledgment for incident response"

# Affects all matching alerts
```

### Bulk Resolve

**Use Case:** Multiple alerts for same issue

**Example:**
```
Scenario: Updated axios from 0.21.1 to 1.6.2
This fixes 5 related alerts:
  - Dependency Tampering
  - Provenance Failure
  - 3× Policy Violations

Action: Bulk resolve all 5 alerts
```

**Process:**
1. Select all 5 alerts
2. Click "Bulk Resolve"
3. Enter shared resolution details
4. Apply to all

### Bulk Suppress

**Use Case:** Known false positives

**Example:**
```
Scenario: lodash-es flagged as typosquatting
Reality: Official ESM version of lodash

Action: Bulk suppress all lodash-es alerts
```

### Bulk Export

**Export Selected Alerts:**

**Formats:**
- CSV - Spreadsheet import
- JSON - API consumption
- SARIF - CI/CD integration
- PDF - Reports

**CLI:**
```bash
sctv alert export \
  --filter "project:production status:open" \
  --format csv \
  --output alerts-report.csv
```

---

## Alert Notifications

### Notification Channels

**Available Channels:**
- Email
- Slack
- Microsoft Teams
- PagerDuty
- Custom Webhooks

### Email Notifications

**Configuration:**
```yaml
Email Notifications:
  Critical Alerts:
    Delivery: Immediate
    Recipients:
      - security@example.com
      - oncall@example.com
    Template: critical-alert

  High Alerts:
    Delivery: Daily digest (09:00 UTC)
    Recipients:
      - dev-team@example.com
    Template: high-alert-digest

  Medium/Low Alerts:
    Delivery: Weekly digest (Monday 09:00 UTC)
    Recipients:
      - dev-team@example.com
    Template: weekly-digest
```

**Email Template:**
```
Subject: [SCTV] CRITICAL Alert: Typosquatting Detected

Project: E-Commerce API
Alert: Typosquatting Detected
Severity: CRITICAL
Package: event-streem@1.0.0

Summary:
Package 'event-streem' is 92% similar to popular package
'event-stream' (50M+ downloads/week). This appears to be
a typosquatting attack.

IMMEDIATE ACTION REQUIRED:
1. Remove event-streem from dependencies
2. Install official event-stream package
3. Audit code for malicious behavior

View Alert: https://sctv.example.com/alerts/550e8400...
View Project: https://sctv.example.com/projects/123e4567...

---
SCTV - Supply Chain Trust Verifier
```

### Slack Notifications

**Configuration:**
```yaml
Slack Notifications:
  Webhook: https://hooks.slack.com/services/T00/B00/xxx
  Channel: #security-alerts

  Message Format:
    Critical: @channel with full details
    High: Standard notification
    Medium/Low: Silent notification
```

**Slack Message:**
```
🚨 CRITICAL ALERT

*Typosquatting Detected*
Project: E-Commerce API
Package: event-streem@1.0.0

Package 'event-streem' is 92% similar to 'event-stream'
(50M+ downloads/week)

*Immediate Actions:*
1. Remove event-streem
2. Install event-stream
3. Audit codebase

<https://sctv.example.com/alerts/550e8400|View Alert>
<https://sctv.example.com/projects/123e4567|View Project>

@channel
```

### PagerDuty Integration

**Incident Creation:**
```yaml
PagerDuty:
  Integration Key: xxx

  Critical Alerts:
    Create Incident: Yes
    Severity: high
    Urgency: high
    Escalation: Security Team

  High Alerts:
    Create Incident: Yes
    Severity: medium
    Urgency: low
```

### Custom Webhooks

**HTTP POST on alert creation:**

```json
POST https://your-system.com/webhook/sctv-alerts

{
  "event": "alert.created",
  "timestamp": "2026-01-15T10:30:00Z",
  "alert": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "type": "typosquatting",
    "severity": "critical",
    "title": "Typosquatting Detected: event-streem",
    "project_id": "123e4567-e89b-12d3-a456-426614174000",
    "project_name": "E-Commerce API",
    "package": {
      "name": "event-streem",
      "version": "1.0.0",
      "ecosystem": "npm"
    }
  }
}
```

---

## Alert Lifecycle

### State Diagram

```
        ┌─────────┐
        │ Created │
        └────┬────┘
             │
             ▼
        ┌────────┐
        │  Open  │◄──────────────┐
        └───┬─┬──┘               │
            │ │                  │
     ┌──────┘ └────────┐         │
     │                 │         │
     ▼                 ▼         │
┌──────────────┐  ┌─────────────┴────┐
│ Acknowledged │  │  Investigating   │
└──────┬───────┘  └────────┬─────────┘
       │                   │
       │    ┌──────────────┘
       │    │
       ▼    ▼
    ┌──────────┐
    │ Resolved │
    └──────────┘

    OR

    ┌───────────────┐
    │ False Positive│
    └───────────────┘

    OR

    ┌────────────┐
    │ Suppressed │
    └────────────┘
```

### State Transitions

**Open → Acknowledged**
- User acknowledges awareness
- Investigation begins

**Acknowledged → Investigating**
- Active work in progress
- Assigned to team member

**Investigating → Resolved**
- Issue fixed
- Verification completed

**Open → Suppressed**
- False positive
- Accepted risk
- Policy exemption

**Any → Resolved**
- Can resolve from any state
- Requires resolution details

### Auto-Transitions

**Resolved → Reopened:**
```
Condition: If issue reappears in subsequent scan
Action: Reopen alert with "recurred" flag
```

**Suppressed → Open:**
```
Condition: Suppression expiration time reached
Action: Reopen alert for re-evaluation
```

---

## Best Practices

### Alert Triage Workflow

**Daily Routine:**
```
1. Morning (30 min):
   - Review new critical/high alerts
   - Acknowledge time-sensitive alerts
   - Assign to team members

2. Weekly (1 hour):
   - Review all open alerts
   - Resolve fixed issues
   - Update investigation status
   - Suppress false positives

3. Monthly (2 hours):
   - Review suppressed alerts
   - Update policies based on patterns
   - Clean up old alerts
   - Generate metrics reports
```

### Response Time Targets

```
Critical:  < 1 hour   (acknowledged)
           < 4 hours  (resolved or plan in place)

High:      < 24 hours (acknowledged)
           < 1 week   (resolved)

Medium:    < 1 week   (acknowledged)
           < 1 month  (resolved)

Low:       Best effort
```

### Alert Assignment

**Auto-Assignment Rules:**
```yaml
Rules:
  - If alert_type == "typosquatting":
      assign_to: security_team

  - If project.team == "backend":
      assign_to: backend_team_lead

  - If severity == "critical":
      assign_to: oncall_engineer
      notify: pagerduty
```

### Documentation

**For Each Alert Resolution:**
- Document root cause
- List steps taken
- Note any code changes
- Record verification results
- Update runbooks

**Example:**
```markdown
Alert #550e8400: Dependency Tampering on axios@0.21.1

Root Cause:
  npm cache corruption on build server

Resolution:
  1. Cleared npm cache
  2. Reinstalled all dependencies
  3. Verified hashes match registry
  4. Updated CI cache strategy

Prevention:
  - Modified CI to clear cache weekly
  - Added hash verification to CI pipeline
  - Documented cache clearing procedure

Verified: 2026-01-15 by @alice
Status: Resolved
```

---

## Troubleshooting

### Common Issues

**Too Many Alerts**

**Problem:** Overwhelmed by alert volume

**Solutions:**
1. Adjust policy sensitivity
2. Suppress known false positives
3. Focus on critical/high first
4. Create exemptions for trusted packages

**False Positives**

**Problem:** Legitimate packages flagged

**Solutions:**
1. Review detection criteria
2. Add to policy whitelist
3. Adjust similarity thresholds
4. Document and suppress

**Missing Alerts**

**Problem:** Expected alerts not appearing

**Solutions:**
1. Verify scan completed successfully
2. Check policy configuration
3. Review detection thresholds
4. Examine scan logs

**Notification Overload**

**Problem:** Too many notifications

**Solutions:**
1. Use digest instead of immediate
2. Filter by severity
3. Route to appropriate channels
4. Adjust escalation rules

---

## Next Steps

- **[Policy Management](policies.md)** - Create custom security policies
- **[Projects Guide](projects.md)** - Manage your projects
- **[SBOM Guide](sbom.md)** - Generate compliance reports
- **[Best Practices](best-practices.md)** - Security recommendations

---

**Need help with alerts?** Check the [troubleshooting guide](../operations/troubleshooting.md) or contact support.
