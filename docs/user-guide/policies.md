# Policy Management Guide

**Version:** 0.1.0
**Last Updated:** 2026-01-15

Security policies define the rules that govern your software supply chain. This comprehensive guide covers creating, managing, and enforcing policies in SCTV.

---

## Table of Contents

- [Policy Concepts](#policy-concepts)
- [Policy Types](#policy-types)
- [Creating Custom Policies](#creating-custom-policies)
- [Policy Rules Syntax](#policy-rules-syntax)
- [Version Constraints](#version-constraints)
- [License Policies](#license-policies)
- [Blocking Packages](#blocking-packages)
- [Policy Evaluation](#policy-evaluation)
- [Policy Templates](#policy-templates)

---

## Policy Concepts

### What is a Security Policy?

A **policy** is a set of security rules that SCTV enforces across your dependencies. Policies help you:

- Block risky packages automatically
- Enforce organizational security standards
- Ensure compliance requirements
- Detect policy violations
- Prevent supply chain attacks

### Policy Components

```yaml
Policy:
  name: Production Security
  description: Enhanced security for production environments
  enabled: true
  is_default: false

  rules:
    - RequireHashVerification
    - BlockTyposquatting
    - RequireProvenance
    - EnforceVersionPinning
    - AllowList / DenyList
    - RequireMinimumAge
    - RequireMinimumMaintainers
    - BlockEcosystems
    - RequireMinimumDownloads
    - RequireSignature

  severity_overrides:
    - Override severity for specific packages/rules
```

### Policy Hierarchy

```
┌──────────────────────────────────────┐
│ Tenant Default Policy                │
│ (Applies to all projects by default) │
└────────────┬─────────────────────────┘
             │
             ▼
┌──────────────────────────────────────┐
│ Project-Specific Policy (Optional)   │
│ (Overrides tenant default)           │
└────────────┬─────────────────────────┘
             │
             ▼
┌──────────────────────────────────────┐
│ Package-Level Exemptions             │
│ (Exceptions to policy rules)         │
└──────────────────────────────────────┘
```

### Policy Scope

**Tenant-Level:**
- Default policy for all projects
- Organization-wide standards
- Managed by administrators

**Project-Level:**
- Specific to one project
- Can be stricter or more permissive
- Managed by project owners

**Package-Level:**
- Exemptions for specific packages
- Temporary or permanent
- Requires justification

---

## Policy Types

SCTV provides pre-built policy templates for common scenarios.

### Strict Security (Recommended for Production)

**Purpose:** Maximum security for critical applications

**Rules:**
```yaml
name: Strict Security
description: Maximum security for production environments

rules:
  - RequireHashVerification:
      algorithms: [sha256, sha512]

  - BlockTyposquatting:
      threshold: 0.85
      check_popular_packages: true

  - RequireProvenance:
      minimum_slsa_level: 2
      require_signature: true

  - EnforceVersionPinning:
      strategy: locked
      allow_patch_updates: false

  - RequireMinimumAge:
      days: 30
      exceptions: []

  - RequireMinimumMaintainers:
      count: 2

  - RequireMinimumDownloads:
      count: 10000
      period: week
```

**Best For:**
- Production applications
- Financial systems
- Healthcare applications
- Critical infrastructure
- Compliance-required environments

### Balanced Security (Default)

**Purpose:** Good security without being too restrictive

**Rules:**
```yaml
name: Balanced Security
description: Balanced approach for most applications

rules:
  - RequireHashVerification:
      algorithms: [sha256]

  - BlockTyposquatting:
      threshold: 0.90

  - RequireProvenance:
      minimum_slsa_level: 1
      require_signature: false

  - EnforceVersionPinning:
      strategy: semver_minor
      allow_patch_updates: true

  - RequireMinimumAge:
      days: 14
      exceptions: []

  - RequireMinimumDownloads:
      count: 1000
      period: week
```

**Best For:**
- General applications
- Internal tools
- Standard web applications
- Most use cases

### Permissive (Development)

**Purpose:** Minimal restrictions for development environments

**Rules:**
```yaml
name: Permissive
description: Minimal restrictions for development

rules:
  - BlockTyposquatting:
      threshold: 0.95

  - RequireHashVerification:
      algorithms: [sha256]
      warn_only: true
```

**Best For:**
- Development environments
- Experimental projects
- Proof of concepts
- Learning projects

### Compliance-Focused

**Purpose:** Meet regulatory compliance requirements

**Rules:**
```yaml
name: Compliance Security
description: Focused on compliance and audit requirements

rules:
  - RequireHashVerification:
      algorithms: [sha256, sha512]

  - RequireProvenance:
      minimum_slsa_level: 2
      require_signature: true

  - RequireSignature:
      trusted_keys: [...]
      require_verified_maintainer: true

  - BlockLicenses:
      prohibited: [GPL, AGPL, SSPL]

  - RequireLicenses:
      allowed: [MIT, Apache-2.0, BSD-3-Clause]

  - RequireMinimumAge:
      days: 90
```

**Best For:**
- Regulated industries
- Government contracts
- SOC 2 compliance
- PCI-DSS requirements
- HIPAA compliance

---

## Creating Custom Policies

### Via Dashboard

**Step 1: Navigate to Policies**
```
Dashboard → Policies → Create Policy
```

**Step 2: Basic Information**
```
┌─────────────────────────────────────────────┐
│ Create Security Policy                      │
├─────────────────────────────────────────────┤
│                                             │
│ Policy Name: *                              │
│ [Production Security            ]           │
│                                             │
│ Description:                                │
│ ┌─────────────────────────────────────────┐ │
│ │ Enhanced security policy for production │ │
│ │ environments. Enforces strict supply    │ │
│ │ chain verification and blocks risky     │ │
│ │ dependencies.                           │ │
│ └─────────────────────────────────────────┘ │
│                                             │
│ Based on template:                          │
│ [Strict Security            ▼]              │
│ (Optional - start from template)            │
│                                             │
│ ☑ Enable this policy                        │
│ ☐ Set as tenant default                     │
│                                             │
│            [Cancel] [Next: Add Rules]       │
│                                             │
└─────────────────────────────────────────────┘
```

**Step 3: Add Rules**
```
┌─────────────────────────────────────────────┐
│ Add Policy Rules                            │
├─────────────────────────────────────────────┤
│                                             │
│ Available Rules:                            │
│                                             │
│ ☑ Require Hash Verification                 │
│   Configure → [Algorithms: SHA-256, SHA-512]│
│                                             │
│ ☑ Block Typosquatting                       │
│   Configure → [Threshold: 0.85]             │
│                                             │
│ ☑ Require Provenance                        │
│   Configure → [SLSA Level: 2+]              │
│                                             │
│ ☑ Enforce Version Pinning                   │
│   Configure → [Strategy: Locked]            │
│                                             │
│ ☑ Block Packages by Pattern                 │
│   Configure → [Add deny list]               │
│                                             │
│ ☐ Require Minimum Age                       │
│   Configure → [Days: 30]                    │
│                                             │
│ ☐ Require Minimum Maintainers               │
│   Configure → [Count: 2]                    │
│                                             │
│ ☐ Block Ecosystems                          │
│   Configure → [Select ecosystems]           │
│                                             │
│ ☐ Require Minimum Downloads                 │
│   Configure → [Count: 10,000/week]          │
│                                             │
│        [Back] [Cancel] [Create Policy]      │
│                                             │
└─────────────────────────────────────────────┘
```

### Via CLI

```bash
# Create from template
sctv policy create \
  --name "Production Security" \
  --description "Enhanced security for production" \
  --template strict \
  --enabled

# Create custom policy
sctv policy create \
  --name "Custom Policy" \
  --rule "require_hash_verification:sha256" \
  --rule "block_typosquatting:0.85" \
  --rule "require_provenance:slsa2" \
  --enabled

# Create from file
sctv policy create --from-file policy.yaml
```

### Via API

**Endpoint:** `POST /api/v1/policies`

**Request:**
```json
{
  "name": "Production Security",
  "description": "Enhanced security for production environments",
  "rules": [
    {
      "rule_type": "require_hash_verification",
      "config": {
        "algorithms": ["sha256", "sha512"]
      }
    },
    {
      "rule_type": "block_typosquatting",
      "config": {
        "threshold": 0.85
      }
    },
    {
      "rule_type": "require_provenance",
      "config": {
        "minimum_slsa_level": 2
      }
    }
  ],
  "severity": "high",
  "is_enabled": true
}
```

### Policy YAML Format

**Complete Example:**
```yaml
name: Production Security
description: Enhanced security for production environments
enabled: true
is_default: false

rules:
  # Integrity Verification
  - type: require_hash_verification
    algorithms:
      - sha256
      - sha512
    severity: high

  # Typosquatting Detection
  - type: block_typosquatting
    threshold: 0.85
    check_popular_packages: true
    minimum_downloads: 100000
    severity: critical

  # Build Provenance
  - type: require_provenance
    minimum_slsa_level: 2
    require_signature: true
    trusted_builders:
      - "https://github.com/*"
    severity: high

  # Version Management
  - type: enforce_version_pinning
    strategy: locked
    allow_patch_updates: false
    severity: medium

  # Package Allow/Deny Lists
  - type: allow_list
    packages:
      - ecosystem: npm
        name_pattern: "@company/*"
      - ecosystem: npm
        name_pattern: "react*"
      - ecosystem: pypi
        name_pattern: "django*"
    severity: high

  - type: deny_list
    packages:
      - ecosystem: npm
        name_pattern: "colors"
        version_pattern: "*"
        reason: "Deprecated, use chalk instead"
      - ecosystem: npm
        name_pattern: "*evil*"
        version_pattern: "*"
        reason: "Suspicious naming pattern"
    severity: critical

  # Age Requirements
  - type: require_minimum_age
    days: 30
    exceptions:
      - "@company/*"  # Company packages exempt
      - "typescript"  # Trusted package
    severity: medium

  # Maintainer Requirements
  - type: require_minimum_maintainers
    count: 2
    allow_verified_single_maintainer: true
    severity: low

  # Ecosystem Restrictions
  - type: block_ecosystems
    ecosystems:
      - rubygems  # Not used in this project
    severity: high

  # Popularity Requirements
  - type: require_minimum_downloads
    count: 10000
    period: week
    exceptions:
      - "@company/*"
    severity: low

  # License Restrictions
  - type: block_licenses
    licenses:
      - GPL
      - AGPL
      - SSPL
    severity: high

  - type: require_licenses
    licenses:
      - MIT
      - Apache-2.0
      - BSD-3-Clause
      - ISC
    allow_unlicensed: false
    severity: medium

# Severity Overrides
severity_overrides:
  # Less strict for development dependencies
  - rule_type: require_minimum_age
    package_pattern:
      name_pattern: "*"
      is_dev_dependency: true
    severity: low

  # More strict for critical packages
  - package_pattern:
      name_pattern: "express"
    severity: critical
```

---

## Policy Rules Syntax

### Require Hash Verification

**Purpose:** Ensure packages haven't been tampered with

**Configuration:**
```yaml
type: require_hash_verification
algorithms:
  - sha256    # Required
  - sha512    # Optional, additional verification
fail_on_mismatch: true
warn_on_missing: true
severity: high
```

**Behavior:**
- Downloads package
- Calculates hash
- Compares with registry metadata
- Generates alert on mismatch

**Example Violation:**
```
Package: axios@0.21.1
Expected: sha256:abc123...
Actual:   sha256:def456...
Result: CRITICAL alert generated
```

### Block Typosquatting

**Purpose:** Detect packages with names similar to popular packages

**Configuration:**
```yaml
type: block_typosquatting
threshold: 0.85  # 0.0 to 1.0 (higher = more similar)
methods:
  - levenshtein
  - damerau_levenshtein
  - jaro_winkler
  - keyboard_distance
check_popular_packages: true
minimum_downloads: 100000  # What counts as "popular"
severity: critical
```

**Similarity Calculation:**
```
Levenshtein("event-stream", "event-streem") = 0.92
  One character difference: 'a' → 'ee'

If 0.92 > threshold (0.85):
  → Generate CRITICAL alert
```

**Example Violation:**
```
Package: lodash-utils
Similar to: lodash (50M+ downloads/week)
Similarity: 0.89
Detection: Combosquatting (added suffix)
Result: CRITICAL alert
```

### Require Provenance

**Purpose:** Ensure packages have verified build provenance

**Configuration:**
```yaml
type: require_provenance
minimum_slsa_level: 2  # 0, 1, 2, 3, or 4
require_signature: true
trusted_builders:
  - "https://github.com/*"
  - "https://gitlab.com/*"
verify_source_repo: true
severity: high
```

**SLSA Levels:**
```
Level 0: No provenance
Level 1: Provenance exists
Level 2: Provenance verified, hosted build service
Level 3: Hardened build platform
Level 4: Highest assurance (audited platform)
```

**Example Violation:**
```
Package: critical-lib@2.0.0
Required: SLSA Level 2+
Actual: No provenance
Result: HIGH alert
```

### Enforce Version Pinning

**Purpose:** Control how version ranges are specified

**Configuration:**
```yaml
type: enforce_version_pinning
strategy: locked  # exact, locked, semver_patch, semver_minor
allow_patch_updates: false
require_lock_file: true
check_lock_file_sync: true
severity: medium
```

**Strategies:**
```yaml
exact:
  - Allowed: "1.2.3"
  - Blocked: "^1.2.3", "~1.2.3", ">=1.2.3"

locked:
  - Must match lock file exactly
  - No version ranges allowed

semver_patch:
  - Allowed: "~1.2.3" (allows 1.2.x)
  - Blocked: "^1.2.3" (would allow 1.x.x)

semver_minor:
  - Allowed: "^1.2.3" (allows 1.x.x)
  - Blocked: "*", ">=1.0.0"
```

**Example Violation:**
```
Package: express
package.json: "^4.18.0" (allows 4.x.x)
Policy: Exact version required
Result: MEDIUM alert
```

### Allow List / Deny List

**Purpose:** Explicitly allow or block specific packages

**Allow List Configuration:**
```yaml
type: allow_list
packages:
  # Allow company packages
  - ecosystem: npm
    name_pattern: "@mycompany/*"
    version_pattern: "*"

  # Allow specific packages
  - ecosystem: npm
    name_pattern: "react"
    version_pattern: "^18.0.0"

  # Allow by prefix
  - ecosystem: pypi
    name_pattern: "django*"

block_unlisted: true  # Block anything not in allow list
severity: high
```

**Deny List Configuration:**
```yaml
type: deny_list
packages:
  # Block deprecated package
  - ecosystem: npm
    name_pattern: "colors"
    version_pattern: "*"
    reason: "Deprecated, use chalk instead"

  # Block vulnerable versions
  - ecosystem: npm
    name_pattern: "lodash"
    version_pattern: "<4.17.21"
    reason: "Vulnerable to prototype pollution"

  # Block suspicious patterns
  - ecosystem: npm
    name_pattern: "*evil*"
    version_pattern: "*"
    reason: "Suspicious naming"

  # Block specific version
  - ecosystem: pypi
    name_pattern: "requests"
    version_pattern: "2.0.0"
    reason: "Known security issue"

severity: critical
```

**Pattern Matching:**
```
Exact:     "lodash"         → matches "lodash" only
Prefix:    "lodash*"        → matches "lodash", "lodash-es", etc.
Suffix:    "*lodash"        → matches "lodash", "my-lodash", etc.
Contains:  "*lodash*"       → matches anything with "lodash"
Scoped:    "@company/*"     → matches all @company packages
Wildcard:  "*"              → matches everything
```

### Require Minimum Age

**Purpose:** Avoid very new packages that lack community vetting

**Configuration:**
```yaml
type: require_minimum_age
days: 30  # Package must be at least 30 days old
check_version_age: true  # Check version age, not package age
exceptions:
  - "@mycompany/*"  # Internal packages exempt
  - "typescript"    # Trusted packages exempt
severity: medium
```

**Age Calculation:**
```
Package first published: 2026-01-01
Current date: 2026-01-15
Age: 15 days

If 15 < minimum_age (30):
  → Generate MEDIUM alert
```

**Example Violation:**
```
Package: new-package@1.0.0
Published: 5 days ago
Required: 30 days minimum
Result: MEDIUM alert
```

### Require Minimum Maintainers

**Purpose:** Prefer packages with multiple maintainers (bus factor)

**Configuration:**
```yaml
type: require_minimum_maintainers
count: 2
allow_verified_single_maintainer: true
check_maintainer_age: true
minimum_maintainer_age_days: 90
severity: low
```

**Example Violation:**
```
Package: solo-maintained-lib
Maintainers: 1
Required: 2 minimum
Result: LOW alert (unless verified)
```

### Block Ecosystems

**Purpose:** Restrict which package ecosystems can be used

**Configuration:**
```yaml
type: block_ecosystems
ecosystems:
  - rubygems  # Not used in this project
  - cargo     # Rust not approved
reason: "Not approved for use in this project"
severity: high
```

**Example Violation:**
```
Package: some-gem@1.0.0 (rubygems)
Policy: rubygems blocked
Result: HIGH alert
```

### Require Minimum Downloads

**Purpose:** Ensure packages have community adoption

**Configuration:**
```yaml
type: require_minimum_downloads
count: 10000
period: week  # week, month, total
check_ecosystem: npm  # Different thresholds per ecosystem
exceptions:
  - "@mycompany/*"
severity: low
```

**Example Violation:**
```
Package: unpopular-lib
Downloads: 500/week
Required: 10,000/week minimum
Result: LOW alert
```

### Require Signature

**Purpose:** Ensure packages are cryptographically signed

**Configuration:**
```yaml
type: require_signature
trusted_keys:
  - "fingerprint:ABCD1234..."
  - "keyid:12345678"
require_keyserver_verification: true
check_sigstore: true  # Check Sigstore signatures
severity: high
```

**Example Violation:**
```
Package: unsigned-package@1.0.0
Signature: None
Required: GPG or Sigstore signature
Result: HIGH alert
```

---

## Version Constraints

### Semantic Versioning Primer

**Format:** `MAJOR.MINOR.PATCH`

```
1.2.3
│ │ │
│ │ └─ Patch: Bug fixes (backwards compatible)
│ └─── Minor: New features (backwards compatible)
└───── Major: Breaking changes
```

### Version Range Syntax

**Exact Version:**
```
"1.2.3"          → Only 1.2.3
```

**Caret (Minor Compatible):**
```
"^1.2.3"         → >=1.2.3 <2.0.0
"^0.2.3"         → >=0.2.3 <0.3.0 (0.x treated as major)
"^0.0.3"         → >=0.0.3 <0.0.4 (0.0.x exact)
```

**Tilde (Patch Compatible):**
```
"~1.2.3"         → >=1.2.3 <1.3.0
"~1.2"           → >=1.2.0 <1.3.0
"~1"             → >=1.0.0 <2.0.0
```

**Comparison:**
```
">1.2.3"         → Greater than 1.2.3
">=1.2.3"        → Greater than or equal
"<2.0.0"         → Less than 2.0.0
"<=2.0.0"        → Less than or equal
```

**Range:**
```
"1.2.3 - 1.8.9"  → >=1.2.3 <=1.8.9
```

**Multiple:**
```
">=1.2.3 <2.0.0" → Between versions
"1.2.x"          → Any patch version
```

### Policy Constraints

**Strict Policy:**
```yaml
# Only allow exact versions
enforce_version_pinning:
  strategy: exact

# Example: Must specify "1.2.3", not "^1.2.3"
```

**Locked Policy:**
```yaml
# Must match lock file
enforce_version_pinning:
  strategy: locked
  require_lock_file: true

# package.json can use ranges, but installed version must be locked
```

**Patch Updates Allowed:**
```yaml
# Allow patch updates only
enforce_version_pinning:
  strategy: semver_patch

# Allows: ~1.2.3 (gets 1.2.x)
# Blocks: ^1.2.3 (would get 1.x.x)
```

**Minor Updates Allowed:**
```yaml
# Allow minor and patch updates
enforce_version_pinning:
  strategy: semver_minor

# Allows: ^1.2.3 (gets 1.x.x)
# Blocks: * or >=1.0.0
```

---

## License Policies

### License Detection

**SCTV detects licenses from:**
- package.json `license` field
- LICENSE file in repository
- README file mentions
- SPDX identifiers
- Multiple licenses (dual licensed)

### Common Licenses

**Permissive (Usually Allowed):**
```
- MIT
- Apache-2.0
- BSD-2-Clause
- BSD-3-Clause
- ISC
```

**Copyleft (Often Restricted):**
```
- GPL-2.0
- GPL-3.0
- LGPL-2.1
- LGPL-3.0
- AGPL-3.0
```

**Proprietary:**
```
- UNLICENSED
- Custom licenses
```

### License Policy Configuration

**Block Copyleft Licenses:**
```yaml
type: block_licenses
licenses:
  - GPL-2.0
  - GPL-3.0
  - LGPL-2.1
  - LGPL-3.0
  - AGPL-3.0
  - SSPL-1.0
reason: "Copyleft licenses not allowed in proprietary software"
severity: high
```

**Require Permissive Licenses:**
```yaml
type: require_licenses
licenses:
  - MIT
  - Apache-2.0
  - BSD-2-Clause
  - BSD-3-Clause
  - ISC
  - Unlicense
allow_unlicensed: false
allow_custom: false
severity: high
```

**Dual License Handling:**
```yaml
# Package with (MIT OR Apache-2.0)
dual_license_strategy: any_allowed  # Allow if ANY license is permitted
# OR
dual_license_strategy: all_allowed  # Require ALL licenses be permitted
```

**Example Violation:**
```
Package: gpl-package@1.0.0
License: GPL-3.0
Policy: Copyleft licenses blocked
Result: HIGH alert
```

---

## Blocking Packages

### Deny List Strategies

**Block Specific Packages:**
```yaml
deny_list:
  - ecosystem: npm
    name_pattern: "colors"
    reason: "Deprecated, use chalk"
```

**Block by Pattern:**
```yaml
deny_list:
  # Block all packages with "evil" in name
  - ecosystem: npm
    name_pattern: "*evil*"
    reason: "Suspicious naming"

  # Block packages from specific scope
  - ecosystem: npm
    name_pattern: "@malicious/*"
    reason: "Known malicious publisher"
```

**Block Specific Versions:**
```yaml
deny_list:
  # Block vulnerable version
  - ecosystem: npm
    name_pattern: "lodash"
    version_pattern: "<4.17.21"
    reason: "Vulnerable to prototype pollution (CVE-2020-8203)"

  # Block specific bad version
  - ecosystem: npm
    name_pattern: "event-stream"
    version_pattern: "3.3.6"
    reason: "Contained malicious code (2018 incident)"
```

**Block Entire Ecosystems:**
```yaml
block_ecosystems:
  ecosystems:
    - rubygems
    - cargo
  reason: "Not approved for use"
```

### Allow List Strategies

**Allow Only Company Packages:**
```yaml
allow_list:
  packages:
    - ecosystem: npm
      name_pattern: "@mycompany/*"
  block_unlisted: true
```

**Allow Specific Packages:**
```yaml
allow_list:
  packages:
    # Core framework
    - name_pattern: "react"
    - name_pattern: "react-dom"

    # Utilities
    - name_pattern: "lodash"
    - name_pattern: "axios"

    # UI Library
    - name_pattern: "@mui/*"
```

**Hybrid Approach:**
```yaml
# Allow list for production
allow_list:
  packages:
    - name_pattern: "approved-*"
  block_unlisted: true
  apply_to:
    - production

# No restrictions for development
allow_list:
  block_unlisted: false
  apply_to:
    - development
    - testing
```

---

## Policy Evaluation

### Evaluation Process

**When Policies are Evaluated:**
1. During dependency scans
2. When new dependencies are added
3. When policies are updated
4. On-demand evaluation via API/CLI

**Evaluation Steps:**
```
For each dependency:
  1. Load applicable policies (tenant + project)
  2. For each policy rule:
     a. Check if rule applies to dependency
     b. Evaluate rule condition
     c. If violation: generate alert
  3. Apply severity overrides
  4. Record evaluation results
```

### Evaluation Results

**Example Output:**
```
Policy Evaluation: Production Security
Project: E-Commerce API
Evaluated: 2026-01-15 10:30:00 UTC
Dependencies: 847

Results:
  Total Rules: 8
  Passed: 6
  Violations: 2

Rule Results:
  ✓ Require Hash Verification (0 violations)
  ✗ Block Typosquatting (2 violations)
  ✓ Require Provenance (0 violations)
  ✓ Enforce Version Pinning (0 violations)
  ✓ Allow List (0 violations)
  ✓ Require Minimum Age (0 violations)
  ✓ Require Minimum Maintainers (0 violations)
  ✓ Require Minimum Downloads (0 violations)

Violations:
  1. event-streem@1.0.0
     Rule: Block Typosquatting
     Severity: CRITICAL
     Details: 92% similar to 'event-stream'

  2. new-package@1.0.0
     Rule: Require Minimum Age
     Severity: MEDIUM
     Details: Only 5 days old (require 30 days)

Alerts Generated: 2
```

### Evaluation API

**Evaluate Policy Against Project:**
```bash
# Via CLI
sctv policy evaluate <policy-id> --project <project-id>

# Via API
POST /api/v1/policies/{policy_id}/evaluate
{
  "project_id": "123e4567-...",
  "dry_run": false
}
```

**Response:**
```json
{
  "policy_id": "550e8400-...",
  "policy_name": "Production Security",
  "project_id": "123e4567-...",
  "evaluated_at": "2026-01-15T10:30:00Z",
  "dependency_count": 847,
  "rule_count": 8,
  "violations": [
    {
      "rule": "block_typosquatting",
      "severity": "critical",
      "package": "event-streem@1.0.0",
      "details": "92% similar to popular package 'event-stream'"
    }
  ],
  "alerts_created": 2
}
```

### Dry Run Mode

**Test Policy Without Creating Alerts:**
```bash
sctv policy evaluate <policy-id> \
  --project <project-id> \
  --dry-run

# Returns violations but doesn't create alerts
```

**Use Cases:**
- Test new policy before applying
- Estimate impact of policy changes
- Preview violations
- Compliance reporting

---

## Policy Templates

### Using Templates

**Available Templates:**
1. Strict Security
2. Balanced Security
3. Permissive
4. Compliance-Focused
5. OSS Project
6. Enterprise

**Create from Template:**
```bash
# Via CLI
sctv policy create \
  --name "My Production Policy" \
  --template strict \
  --customize

# Via Dashboard
Policies → Create → Select Template → Customize
```

### Custom Templates

**Save as Template:**
```bash
sctv policy template create \
  --from-policy <policy-id> \
  --name "Company Standard" \
  --description "Our organization's standard policy"
```

**Template YAML:**
```yaml
template:
  name: Company Standard
  description: Organization-wide security policy
  author: Security Team
  version: 1.0.0
  created: 2026-01-15

  base_rules:
    - require_hash_verification
    - block_typosquatting
    - require_provenance

  customizable_rules:
    - enforce_version_pinning:
        default: locked
        options: [exact, locked, semver_patch, semver_minor]

    - require_minimum_age:
        default: 30
        range: [0, 365]

  required_rules:
    # These cannot be disabled
    - block_typosquatting
    - require_hash_verification
```

---

## Best Practices

### Policy Design

**Start Permissive, Tighten Gradually:**
```
Week 1: Permissive policy, observe
Week 2: Add typosquatting detection
Week 3: Add hash verification
Week 4: Add provenance requirements
Week 5: Full production policy
```

**Use Environment-Specific Policies:**
```yaml
Development:
  - Permissive rules
  - Warnings only
  - Fast feedback

Staging:
  - Moderate rules
  - Block critical issues
  - Similar to production

Production:
  - Strict rules
  - Block all violations
  - Maximum security
```

**Document Exemptions:**
```yaml
exemptions:
  - package: "@company/legacy-lib"
    reason: "Internal package, maintained by security team"
    expires: 2026-12-31
    approved_by: security-team

  - package: "experimental-*"
    reason: "R&D packages, isolated environment"
    applies_to: development
```

### Policy Maintenance

**Regular Review:**
- Monthly: Review exemptions
- Quarterly: Update templates
- Annually: Comprehensive policy audit

**Version Control:**
```bash
# Store policies in Git
policies/
  ├── production.yaml
  ├── staging.yaml
  ├── development.yaml
  └── templates/
      ├── strict.yaml
      └── balanced.yaml
```

**Change Management:**
```
1. Propose policy change
2. Dry run evaluation
3. Review impact
4. Get approval
5. Apply to staging
6. Test thoroughly
7. Apply to production
8. Monitor alerts
```

### Common Pitfalls

**Too Strict Initially:**
```
Problem: 1000+ alerts on first scan
Solution: Start permissive, increase gradually
```

**No Exemptions Process:**
```
Problem: Legitimate packages blocked, no recourse
Solution: Document exemption process
```

**One Policy for All Projects:**
```
Problem: Production and development have different needs
Solution: Use project-specific policies
```

**Ignoring False Positives:**
```
Problem: Teams ignore all alerts due to noise
Solution: Tune thresholds, add exemptions
```

---

## Troubleshooting

### Common Issues

**Policy Not Applied**

**Symptoms:** Rules not enforcing

**Solutions:**
1. Verify policy is enabled
2. Check policy assignment to project
3. Trigger manual evaluation
4. Review logs for errors

**Too Many Violations**

**Symptoms:** Hundreds of alerts

**Solutions:**
1. Review thresholds (e.g., typosquatting similarity)
2. Add exemptions for known good packages
3. Use more permissive template
4. Phase in rules gradually

**Unexpected Violations**

**Symptoms:** Legitimate packages flagged

**Solutions:**
1. Review rule configuration
2. Check for false positives
3. Add to allow list
4. Adjust severity overrides

**Policy Conflicts**

**Symptoms:** Conflicting rules

**Solutions:**
1. Review rule priority
2. Use severity overrides
3. Simplify policy
4. Use project-specific policies

---

## Next Steps

- **[Alert Management](alerts.md)** - Respond to policy violations
- **[Projects Guide](projects.md)** - Assign policies to projects
- **[Best Practices](best-practices.md)** - Policy recommendations
- **[CLI Reference](../reference/cli-reference.md)** - Manage policies via CLI

---

**Need help with policies?** Check the [policy examples](policy-examples.md) or contact support.
