# Threat Types Reference

**Version:** 0.1.0

Complete reference of supply chain threats detected by SCTV.

---

## Table of Contents

- [Overview](#overview)
- [Typosquatting](#typosquatting)
- [Dependency Tampering](#dependency-tampering)
- [Downgrade Attacks](#downgrade-attacks)
- [Provenance Failures](#provenance-failures)
- [Policy Violations](#policy-violations)
- [New Package Warnings](#new-package-warnings)
- [Suspicious Maintainer Activity](#suspicious-maintainer-activity)
- [Severity Levels](#severity-levels)
- [Detection Algorithms](#detection-algorithms)

---

## Overview

SCTV detects seven categories of supply chain threats:

| Threat Type | Default Severity | Description |
|-------------|-----------------|-------------|
| Typosquatting | Critical | Malicious packages with similar names |
| Dependency Tampering | High | Package integrity violations |
| Downgrade Attacks | Medium | Suspicious version rollbacks |
| Provenance Failures | High | Missing/invalid build attestations |
| Policy Violations | Variable | Security policy violations |
| New Packages | Low | Recently published packages |
| Suspicious Maintainer | High | Unusual maintainer activity |

---

## Typosquatting

### Description

Typosquatting attacks use package names similar to popular libraries to trick developers into installing malicious code.

### Detection Methods

1. **Levenshtein Distance** - Edit distance between names
2. **Jaro-Winkler Similarity** - Weighted string similarity
3. **Visual Similarity** - Character substitution detection
4. **Homoglyph Detection** - Unicode lookalike characters

### Examples

```
Target: lodash
Typosquats:
  - lodash-utils (similar name)
  - lodahs (character transposition)
  - lodash_ (punctuation addition)
  - l0dash (number substitution)
  - lodαsh (homoglyph: α instead of a)

Target: requests (Python)
Typosquats:
  - request (singular)
  - requsets (typo)
  - requests2 (version suffix)
  - python-requests (prefix)
```

### Alert Details

```json
{
  "alert_type": "typosquatting",
  "severity": "critical",
  "details": {
    "suspicious_package": "lodash-utils",
    "target_package": "lodash",
    "ecosystem": "npm",
    "similarity_score": 0.92,
    "method": "levenshtein",
    "evidence": {
      "target_downloads": 99850234,
      "suspicious_downloads": 147,
      "target_age_days": 3650,
      "suspicious_age_days": 5,
      "new_maintainer": true,
      "obfuscated_code": true
    }
  }
}
```

### Remediation

1. **Remove the suspicious package** immediately
2. **Install the official package** instead
3. **Review code changes** for potential backdoors
4. **Rotate secrets** that may have been exposed
5. **Scan systems** for signs of compromise
6. **Report to registry** to protect others

### Prevention

- Use exact package names from official documentation
- Enable package lock files (package-lock.json, requirements.txt)
- Configure private registries with allowlists
- Use SCTV's typosquatting detector in CI/CD
- Review dependencies during code review

---

## Dependency Tampering

### Description

Package tampering occurs when a downloaded package's checksum doesn't match the registry's published checksum, indicating the package may have been modified.

### Detection Methods

1. **SHA-256 Hash Verification** - Compare computed vs. expected hash
2. **SHA-512 Hash Verification** - Additional hash verification
3. **Registry Metadata Check** - Verify against multiple registries
4. **Signature Verification** - Check GPG/PGP signatures

### Examples

```
Package: axios@0.21.1
Registry SHA-256: abc123def456...
Downloaded SHA-256: 123abc456def...
Status: MISMATCH - Potential tampering

Package: react@18.2.0
Registry SHA-256: xyz789uvw012...
Downloaded SHA-256: xyz789uvw012...
Signature: INVALID
Status: Unsigned or signature mismatch
```

### Alert Details

```json
{
  "alert_type": "dependency_tampering",
  "severity": "high",
  "details": {
    "package_name": "axios",
    "ecosystem": "npm",
    "version": "0.21.1",
    "expected_hash": "abc123def456...",
    "actual_hash": "123abc456def...",
    "algorithm": "sha256",
    "registry_source": "https://registry.npmjs.org",
    "download_source": "https://registry.npmjs.org/axios/-/axios-0.21.1.tgz"
  }
}
```

### Causes

1. **Man-in-the-middle attack** - Network interception
2. **Compromised registry mirror** - Unofficial mirror serving malicious packages
3. **CDN cache poisoning** - Poisoned edge cache
4. **Registry compromise** - Official registry temporarily compromised
5. **Build system tampering** - CI/CD pipeline compromised

### Remediation

1. **Do not use the package** until verified
2. **Clear package cache** to remove tampered version
3. **Download from official registry** directly
4. **Verify with maintainer** if hash discrepancy persists
5. **Check network security** for MITM attacks
6. **Use HTTPS** and certificate pinning
7. **Report to registry security team**

### Prevention

- Always use HTTPS for package downloads
- Enable subresource integrity (SRI) where supported
- Use package lock files with integrity hashes
- Configure trusted registries only
- Enable SCTV continuous monitoring
- Use private package mirrors with integrity checks

---

## Downgrade Attacks

### Description

Downgrade attacks occur when a dependency is unexpectedly rolled back to an older version, potentially reintroducing known vulnerabilities.

### Detection Methods

1. **Version History Analysis** - Track version changes over time
2. **Semantic Version Comparison** - Detect major/minor downgrades
3. **CVE Correlation** - Check if downgrade reintroduces CVEs
4. **Lock File Comparison** - Compare against previous state

### Examples

```
Previous: webpack@5.89.0 (latest, secure)
Current:  webpack@5.75.0 (14 versions behind)
Severity: HIGH
Reason: 5.75.0 has 2 known CVEs

Previous: axios@1.6.0
Current:  axios@1.5.1
Severity: MEDIUM
Reason: Minor version downgrade, security patch removed

Previous: lodash@4.17.21
Current:  lodash@3.10.1
Severity: HIGH
Reason: Major version downgrade, multiple CVEs
```

### Alert Details

```json
{
  "alert_type": "downgrade_attack",
  "severity": "high",
  "details": {
    "package_name": "webpack",
    "ecosystem": "npm",
    "previous_version": "5.89.0",
    "current_version": "5.75.0",
    "versions_behind": 14,
    "known_cves": [
      "CVE-2023-1234",
      "CVE-2023-5678"
    ],
    "downgrade_date": "2026-01-15T10:00:00Z"
  }
}
```

### Causes

1. **Dependency confusion** - Wrong package source
2. **Malicious PR** - Intentional downgrade in pull request
3. **Compromised lock file** - Attacker modified lock file
4. **Build system misconfiguration** - Incorrect version resolution
5. **Registry manipulation** - Attacker republished old version as new

### Remediation

1. **Identify the cause** of the downgrade
2. **Review recent changes** to dependencies
3. **Update to latest secure version** immediately
4. **Check for CVEs** in downgraded version
5. **Audit access logs** for unauthorized changes
6. **Review lock files** in version control
7. **Scan for vulnerabilities** with updated version

### Prevention

- Use exact versions in lock files
- Pin critical dependencies to specific versions
- Review dependency updates in code review
- Use dependabot or renovate with auto-merge disabled
- Enable SCTV monitoring for version changes
- Require signed commits for dependency changes

---

## Provenance Failures

### Description

Provenance verification ensures packages were built in a trusted environment using Supply-chain Levels for Software Artifacts (SLSA) attestations.

### SLSA Levels

| Level | Requirements | Assurance |
|-------|-------------|-----------|
| SLSA 0 | No guarantees | None |
| SLSA 1 | Build process documented | Minimal |
| SLSA 2 | Build service, signed provenance | Basic |
| SLSA 3 | Hardened build, non-falsifiable | Strong |
| SLSA 4 | Two-party review | Maximum |

### Detection Methods

1. **Sigstore Verification** - Verify using Rekor transparency log
2. **In-toto Attestation** - Validate in-toto link metadata
3. **SLSA Provenance** - Check SLSA provenance bundle
4. **Builder Verification** - Verify trusted builder identity

### Examples

```
Package: critical-lib@2.0.0
Provenance: MISSING
SLSA Level: 0
Recommendation: Request provenance or find alternative

Package: secure-lib@1.5.0
Provenance: PRESENT
SLSA Level: 2
Builder: GitHub Actions
Status: VERIFIED

Package: suspicious-lib@3.0.0
Provenance: PRESENT
SLSA Level: 1
Signature: INVALID
Status: FAILED VERIFICATION
```

### Alert Details

```json
{
  "alert_type": "provenance_failure",
  "severity": "high",
  "details": {
    "package_name": "critical-lib",
    "ecosystem": "npm",
    "version": "2.0.0",
    "provenance_status": "missing",
    "slsa_level": 0,
    "expected_level": 2,
    "verification_error": "No provenance attestation found",
    "sigstore_bundle": null
  }
}
```

### Remediation

1. **Contact package maintainer** to request provenance
2. **Verify package authenticity** through other means
3. **Consider alternatives** with proper provenance
4. **Accept risk** with documented justification
5. **Add to policy exemptions** if internally verified
6. **Monitor for future updates** with provenance

### Prevention

- Prefer packages with SLSA Level 2+ provenance
- Configure policies to require provenance
- Use registries that enforce provenance (e.g., npm with provenance)
- Build internal packages with provenance enabled
- Participate in Sigstore adoption

---

## Policy Violations

### Description

Policy violations occur when dependencies don't meet organizational security requirements defined in SCTV policies.

### Common Policy Rules

| Rule Type | Purpose | Example |
|-----------|---------|---------|
| `BlockDeprecated` | Prevent deprecated packages | Block colors@1.4.0 |
| `RequireProvenance` | Ensure build integrity | Require SLSA 2+ |
| `BlockPackageAge` | Avoid immature packages | Block < 30 days old |
| `RequireSignatures` | Ensure authenticity | GPG signature required |
| `BlockMaintainerChange` | Detect compromised accounts | Alert on maintainer change |
| `RequireMinimumDownloads` | Ensure community trust | Require 10k+ downloads |
| `BlockByLicense` | License compliance | Block GPL, AGPL |
| `AllowlistPackages` | Explicit approval only | Only allow approved |
| `BlocklistPackages` | Explicitly forbidden | Block known-bad packages |

### Alert Details

```json
{
  "alert_type": "policy_violation",
  "severity": "medium",
  "details": {
    "policy_name": "Production Security Policy",
    "policy_id": "uuid",
    "rule_type": "BlockDeprecated",
    "violated_rule": {
      "type": "BlockDeprecated",
      "severity": "high"
    },
    "package_name": "colors",
    "version": "1.4.0",
    "violation_reason": "Package is deprecated",
    "deprecation_message": "Package no longer maintained"
  }
}
```

### Remediation

1. **Review policy requirements** to understand violation
2. **Update dependency** to compliant version
3. **Request exemption** if violation is justified
4. **Document the risk** if accepting violation
5. **Update policy** if rule is too strict
6. **Find alternative** package if needed

### Prevention

- Define clear security policies upfront
- Review policies with security team
- Test policies before enforcement
- Provide exemption process
- Document policy rationale
- Review and update policies regularly

---

## New Package Warnings

### Description

Newly published packages (< 30 days) may not have undergone sufficient community review and could be malicious or unstable.

### Risk Factors

1. **Age** - Days since first publication
2. **Maintainer Age** - Account creation date
3. **Download Count** - Adoption level
4. **Version Count** - Number of published versions
5. **Documentation** - Presence of README, docs
6. **Repository** - Linked source repository
7. **Dependencies** - Number and quality of dependencies

### Examples

```
Package: super-new-lib@1.0.0
Published: 5 days ago
Maintainer Account: 7 days old
Downloads: 147
Repository: None
README: Minimal
Risk: HIGH - Suspicious new package

Package: useful-tool@0.1.0
Published: 15 days ago
Maintainer Account: 2 years old
Downloads: 1,234
Repository: github.com/author/useful-tool
README: Comprehensive
Risk: LOW - Legitimate new package
```

### Alert Details

```json
{
  "alert_type": "new_package",
  "severity": "low",
  "details": {
    "package_name": "super-new-lib",
    "ecosystem": "npm",
    "version": "1.0.0",
    "published_at": "2026-01-10T10:00:00Z",
    "age_days": 5,
    "maintainer_account_age_days": 7,
    "download_count": 147,
    "has_repository": false,
    "has_readme": false,
    "version_count": 1
  }
}
```

### Remediation

1. **Review package source code** if repository exists
2. **Check maintainer reputation** and history
3. **Verify package purpose** matches description
4. **Look for code obfuscation** or suspicious patterns
5. **Search for alternatives** with longer history
6. **Wait for community adoption** if not urgent
7. **Accept risk** with monitoring enabled

### Prevention

- Set minimum package age in policies
- Require minimum download counts
- Use established packages when possible
- Review new dependencies in code review
- Enable SCTV new package monitoring
- Maintain internal package allowlist

---

## Suspicious Maintainer Activity

### Description

Unusual maintainer behavior that may indicate account compromise or malicious intent.

### Indicators

1. **Sudden Maintainer Change** - New maintainer added/removed
2. **Mass Package Updates** - Multiple packages updated simultaneously
3. **Geographic Anomalies** - Login from unusual locations
4. **Publishing Pattern Changes** - Unusual publishing frequency
5. **Code Style Changes** - Sudden changes in coding patterns
6. **New Maintainer Accounts** - Very recent account creation
7. **Orphaned Packages** - Maintainer takeover of abandoned packages

### Examples

```
Package: popular-lib
Previous Maintainer: john-doe (5+ years)
New Maintainer: new-user (account created 3 days ago)
Change Date: 2 days ago
Status: SUSPICIOUS

Package: utility-package
Maintainer: established-dev
Recent Activity:
  - 15 package updates in 1 hour (unusual)
  - Login from new country
  - Code obfuscation added
Status: HIGHLY SUSPICIOUS
```

### Alert Details

```json
{
  "alert_type": "suspicious_maintainer",
  "severity": "high",
  "details": {
    "package_name": "popular-lib",
    "ecosystem": "npm",
    "previous_maintainer": {
      "username": "john-doe",
      "tenure_days": 1825
    },
    "new_maintainer": {
      "username": "new-user",
      "account_age_days": 3
    },
    "change_date": "2026-01-13T10:00:00Z",
    "indicators": [
      "new_maintainer_recent_account",
      "rapid_maintainer_change",
      "mass_package_updates"
    ]
  }
}
```

### Remediation

1. **Do not update** until investigation complete
2. **Check package changelog** for legitimacy
3. **Verify maintainer identity** through official channels
4. **Review recent code changes** for malicious code
5. **Contact previous maintainer** if possible
6. **Report to registry** if compromise suspected
7. **Pin to safe version** until resolved
8. **Monitor for updates** and community response

### Prevention

- Enable 2FA for all maintainers
- Use hardware security keys
- Monitor maintainer changes
- Require multiple maintainers for critical packages
- Set up alerts for maintainer changes
- Review all updates from new maintainers

---

## Severity Levels

### Critical

**Characteristics:**
- Immediate threat to security
- High likelihood of exploitation
- Significant potential impact
- Requires immediate action

**Examples:**
- Active typosquatting with malicious code
- Known malware in dependency
- Compromised package with backdoor

**Recommended Action:** Remove immediately, investigate potential breach

### High

**Characteristics:**
- Serious security concern
- Moderate likelihood of exploitation
- Substantial potential impact
- Requires urgent attention

**Examples:**
- Tampering detection
- Provenance verification failure
- Suspicious maintainer activity
- Major version downgrade with CVEs

**Recommended Action:** Investigate and resolve within 24 hours

### Medium

**Characteristics:**
- Notable security issue
- Lower likelihood of exploitation
- Moderate potential impact
- Should be addressed soon

**Examples:**
- Minor version downgrade
- Policy violations
- Package age warnings (< 30 days)

**Recommended Action:** Address within 1 week

### Low

**Characteristics:**
- Minor concern
- Low likelihood of exploitation
- Limited potential impact
- Can be addressed in normal cycle

**Examples:**
- New package warnings
- Low-priority policy violations
- Informational alerts

**Recommended Action:** Review during regular maintenance

### Info

**Characteristics:**
- No immediate security concern
- Informational only
- No direct impact
- Optional action

**Examples:**
- Dependency tree changes
- Version updates available
- Package metadata changes

**Recommended Action:** Review when convenient

---

## Detection Algorithms

### Typosquatting Detection

**Levenshtein Distance:**
```rust
fn levenshtein_distance(a: &str, b: &str) -> usize {
    // Minimum number of single-character edits
    // lodash -> lodahs = 2 (swap s and h)
}

fn similarity_score(a: &str, b: &str) -> f64 {
    let distance = levenshtein_distance(a, b);
    let max_len = a.len().max(b.len());
    1.0 - (distance as f64 / max_len as f64)
}
```

**Jaro-Winkler Similarity:**
```rust
fn jaro_winkler(a: &str, b: &str) -> f64 {
    // Weighted similarity favoring common prefixes
    // More weight to characters at start of string
    // Range: 0.0 (no match) to 1.0 (exact match)
}
```

**Visual Similarity:**
```rust
fn visual_similarity(a: &str, b: &str) -> bool {
    // Check for confusable characters:
    // i/l, 0/O, rn/m, vv/w, etc.
    // Unicode homoglyphs: a/α, o/о, etc.
}
```

### Hash Verification

```rust
async fn verify_integrity(package: &Package) -> Result<bool> {
    // Download package
    let content = download_package(package).await?;

    // Compute hashes
    let sha256 = compute_sha256(&content);
    let sha512 = compute_sha512(&content);

    // Compare with registry
    let matches_sha256 = sha256 == package.checksum_sha256;
    let matches_sha512 = sha512 == package.checksum_sha512;

    Ok(matches_sha256 && matches_sha512)
}
```

### Provenance Verification

```rust
async fn verify_provenance(package: &Package) -> Result<ProvenanceResult> {
    // Fetch attestation bundle
    let bundle = fetch_sigstore_bundle(package).await?;

    // Verify signature against Rekor transparency log
    let signature_valid = verify_rekor_signature(&bundle).await?;

    // Verify builder identity
    let builder_trusted = verify_builder(&bundle.predicate).await?;

    // Check SLSA level
    let slsa_level = determine_slsa_level(&bundle)?;

    Ok(ProvenanceResult {
        verified: signature_valid && builder_trusted,
        slsa_level,
        builder: bundle.predicate.builder,
    })
}
```

---

## Next Steps

- [Ecosystems Reference](ecosystems.md) - Supported package ecosystems
- [Error Codes](error-codes.md) - Complete error code reference
- [Configuration Reference](configuration.md) - Configure detection thresholds
- [Best Practices](../user-guide/best-practices.md) - Security recommendations
