# Security Best Practices

**Version:** 0.1.0
**Last Updated:** 2026-01-15

This guide provides comprehensive security best practices for managing your software supply chain with SCTV.

---

## Table of Contents

- [Dependency Management](#dependency-management)
- [Lock File Usage](#lock-file-usage)
- [Version Pinning Strategies](#version-pinning-strategies)
- [Regular Scanning Cadence](#regular-scanning-cadence)
- [Alert Triage Workflow](#alert-triage-workflow)
- [Policy Recommendations](#policy-recommendations)
- [CI/CD Integration Patterns](#cicd-integration-patterns)
- [Team Workflows](#team-workflows)

---

## Dependency Management

### Minimize Dependencies

**Principle:** Every dependency is a potential security risk.

**Best Practices:**

**1. Audit Dependencies Regularly**
```bash
# Review all dependencies quarterly
npm ls --all > dependencies-review.txt

# Identify unused dependencies
npx depcheck

# Remove unused packages
npm uninstall unused-package
```

**2. Evaluate Before Adding**
```
Before adding a dependency, ask:
✓ Is this functionality core to my app?
✓ Can I implement it myself in reasonable time?
✓ Is the package well-maintained?
✓ Does it have many dependencies itself?
✓ Are there lighter alternatives?
✓ What's the security track record?
```

**3. Prefer Standard Library**
```javascript
// Bad: Adding dependency for simple task
const isNumber = require('is-number');

// Good: Use built-in functionality
const isNumber = (val) => typeof val === 'number' && !isNaN(val);
```

**4. Bundle Size Awareness**
```bash
# Analyze bundle size impact
npm install --save-dev webpack-bundle-analyzer

# Check package size before installing
npm view express dist.tarballSize
npm view lodash dist.tarballSize
```

### Vet Dependencies

**Before Installation:**

**1. Check Package Metadata**
```bash
# View package info
npm view express

# Key metrics:
- Maintainers: Who publishes?
- Weekly downloads: Is it popular?
- Last publish: Is it maintained?
- License: Compatible?
- Repository: Legitimate source?
```

**2. Review Source Code**
```bash
# Clone repository
git clone https://github.com/expressjs/express
cd express

# Review recent commits
git log -20 --oneline

# Check for red flags:
- Obfuscated code
- Suspicious commits
- Unusual dependencies
- Missing documentation
```

**3. Check Security History**
```bash
# Check for known vulnerabilities
npm audit

# Search CVE databases
https://nvd.nist.gov/
https://www.cvedetails.com/

# Check SCTV threat database
sctv package check express --history
```

**4. Verify Maintainers**
```
✓ Long account history (>1 year)
✓ Multiple verified maintainers
✓ Affiliated with known organization
✓ Active on GitHub/npm
✓ Responsive to issues
```

### Dependency Hygiene

**Regular Cleanup:**
```bash
# Weekly: Remove unused dependencies
npx depcheck
npm uninstall $(npx depcheck --json | jq -r '.dependencies | join(" ")')

# Monthly: Update dependencies
npm outdated
npm update

# Quarterly: Major version updates
npm outdated --long
# Review and update major versions carefully
```

**Dependency Hierarchy:**
```
Keep dependency tree shallow:
✓ Direct dependencies: < 30 preferred
✓ Total dependencies: < 500 preferred
✓ Max depth: < 5 levels preferred

Monitor:
⚠ Duplicate dependencies (different versions)
⚠ Circular dependencies
⚠ Abandoned packages (no updates in 1+ year)
```

---

## Lock File Usage

### Always Commit Lock Files

**Critical Practice:** Lock files ensure reproducible builds.

**Lock Files by Ecosystem:**
```
npm:       package-lock.json
Yarn:      yarn.lock
pnpm:      pnpm-lock.yaml
Python:    Pipfile.lock, poetry.lock, requirements.txt (with versions)
Maven:     Effective POM (pom.xml with resolved versions)
Cargo:     Cargo.lock
Go:        go.sum
```

**Git Configuration:**
```bash
# .gitignore - NEVER ignore lock files
# ❌ Bad:
package-lock.json

# ✓ Good:
# (don't ignore lock files)
```

### Lock File Best Practices

**1. Keep Lock Files Updated**
```bash
# Update lock file after installing packages
npm install new-package
git add package-lock.json
git commit -m "Add new-package dependency"

# Regenerate if corrupted
rm package-lock.json
npm install
git add package-lock.json
git commit -m "Regenerate package-lock.json"
```

**2. Review Lock File Changes**
```bash
# Before committing, review changes
git diff package-lock.json

# Look for:
- New packages added
- Version changes (expected vs unexpected)
- Removed packages
- Integrity hash changes

# Use SCTV to detect tampering
sctv verify --lock-file package-lock.json
```

**3. CI/CD Lock File Enforcement**
```yaml
# GitHub Actions
- name: Verify Lock File
  run: |
    npm ci  # Fails if package.json and package-lock.json are out of sync
    sctv verify --lock-file package-lock.json

# Fail build if lock file is out of sync
```

**4. Lock File Synchronization**
```bash
# Ensure lock file matches package.json
npm install --package-lock-only

# Audit lock file
npm audit

# Fix vulnerabilities in lock file
npm audit fix
```

### Detecting Lock File Issues

**Symptoms:**
- Different dependencies on different machines
- CI builds fail randomly
- "Works on my machine" problems
- Security scans show different results

**Diagnosis:**
```bash
# Check for missing lock file
ls package-lock.json

# Verify integrity
npm ci  # Strict mode, fails if mismatch

# Compare lock files
diff package-lock.json package-lock.json.backup

# Use SCTV
sctv project verify <project-id> --lock-file
```

**Resolution:**
```bash
# Regenerate lock file
rm package-lock.json
npm install

# Verify no unexpected changes
git diff package-lock.json

# Run tests
npm test

# Commit
git add package-lock.json
git commit -m "Regenerate lock file"
```

---

## Version Pinning Strategies

### Pinning Strategies

**1. Exact Versions (Most Secure)**
```json
{
  "dependencies": {
    "express": "4.18.2",
    "axios": "1.6.2",
    "lodash": "4.17.21"
  }
}
```

**Pros:**
- Maximum control
- Predictable builds
- No surprise updates
- Clear dependency tracking

**Cons:**
- Manual updates required
- Miss security patches
- More maintenance

**Best For:**
- Production environments
- Critical applications
- Compliance requirements

**2. Patch Updates (Balanced)**
```json
{
  "dependencies": {
    "express": "~4.18.2",   // Allows 4.18.x
    "axios": "~1.6.2",      // Allows 1.6.x
    "lodash": "~4.17.21"    // Allows 4.17.x
  }
}
```

**Pros:**
- Automatic security patches
- Bug fixes included
- Less maintenance
- Backwards compatible

**Cons:**
- Some risk of breakage
- Requires testing
- May introduce bugs

**Best For:**
- Most applications
- Good security/convenience balance
- Teams with CI/CD

**3. Minor Updates (Flexible)**
```json
{
  "dependencies": {
    "express": "^4.18.2",   // Allows 4.x.x
    "axios": "^1.6.2",      // Allows 1.x.x
    "lodash": "^4.17.21"    // Allows 4.x.x
  }
}
```

**Pros:**
- New features automatically
- Security updates
- Less maintenance

**Cons:**
- Higher risk of breakage
- May introduce bugs
- Requires thorough testing

**Best For:**
- Development environments
- Internal tools
- Rapid iteration

**4. Range/Latest (Not Recommended)**
```json
{
  "dependencies": {
    "express": "*",         // ❌ Any version
    "axios": ">=1.0.0",     // ❌ Any 1.x or higher
    "lodash": "latest"      // ❌ Whatever is latest
  }
}
```

**Never use in production!**

### Recommended Strategy by Environment

**Production:**
```json
{
  "dependencies": {
    "express": "4.18.2"  // Exact versions
  },
  "devDependencies": {
    "jest": "~29.7.0"    // Patch updates OK for dev tools
  }
}
```

**Staging:**
```json
{
  "dependencies": {
    "express": "~4.18.2"  // Patch updates to test before prod
  }
}
```

**Development:**
```json
{
  "dependencies": {
    "express": "^4.18.2"  // Minor updates for features
  }
}
```

### Version Update Workflow

**Safe Update Process:**
```bash
# 1. Check for updates
npm outdated

# 2. Review changelogs
npm view express versions
npm view express@4.19.0  # Check specific version

# 3. Update in development
npm install express@4.19.0

# 4. Run tests
npm test

# 5. Update staging
# Deploy to staging
# Run integration tests
# Monitor for issues

# 6. Update production (if staging OK)
# Deploy to production
# Monitor metrics
# Have rollback ready
```

---

## Regular Scanning Cadence

### Scanning Frequency

**Critical/Production Applications:**
```yaml
Frequency: Daily
Time: 2 AM (low traffic)
Scope: All dependencies
Actions:
  - Critical alerts: Immediate notification
  - High alerts: Email
  - Generate SBOM
  - Update metrics
```

**Standard Applications:**
```yaml
Frequency: Weekly
Day: Sunday
Time: 2 AM
Scope: All dependencies
Actions:
  - Email digest
  - Generate SBOM monthly
```

**Development/Internal:**
```yaml
Frequency: Monthly
Scope: Production dependencies only
Actions:
  - Email summary
  - No immediate action required
```

**On-Demand:**
```
Triggers:
- New dependency added
- Security advisory published
- Pre-deployment
- Compliance audit
- Customer request
```

### Scan Configuration

**Comprehensive Scan:**
```yaml
scan_config:
  include_dev_dependencies: true
  include_transitive: true
  depth: unlimited
  verify_hashes: true
  check_provenance: true
  check_signatures: true
  check_licenses: true
  check_policy: true
```

**Quick Scan (Fast Feedback):**
```yaml
scan_config:
  include_dev_dependencies: false
  include_transitive: true
  depth: 3  # Shallow scan
  verify_hashes: true
  check_policy: true
  # Skip slower checks
```

### Automated Scanning

**CI/CD Integration:**
```yaml
# .github/workflows/sctv-scan.yml
name: SCTV Security Scan

on:
  push:
    branches: [main, develop]
  pull_request:
  schedule:
    - cron: '0 2 * * *'  # Daily at 2 AM

jobs:
  scan:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: SCTV Scan
        run: |
          sctv scan . \
            --fail-on critical \
            --output sarif \
            --upload

      - name: Upload SARIF
        uses: github/codeql-action/upload-sarif@v2
        with:
          sarif_file: sctv-results.sarif
```

**Scheduled Scans:**
```bash
# Cron: Daily at 2 AM
0 2 * * * cd /app && sctv scan . --project prod-api

# Cron: Weekly on Sunday
0 2 * * 0 cd /app && sctv scan . --comprehensive
```

---

## Alert Triage Workflow

### Daily Triage Process

**Morning Review (15-30 minutes):**
```
1. Check dashboard for new alerts
2. Filter by Critical and High severity
3. Quick assessment:
   - Is it actionable?
   - Is it urgent?
   - Who should handle it?
4. Assign alerts to team members
5. Acknowledge time-sensitive alerts
6. Create tickets for investigation
```

**Triage Decision Tree:**
```
New Alert Received
    │
    ├─ Critical?
    │   ├─ Yes → Immediate action
    │   │   ├─ Security team notified
    │   │   ├─ Block deployment
    │   │   └─ Investigate within 1 hour
    │   │
    │   └─ No → Continue
    │
    ├─ High?
    │   ├─ Yes → Priority action
    │   │   ├─ Assign to developer
    │   │   ├─ Investigate within 24 hours
    │   │   └─ Fix in next sprint
    │   │
    │   └─ No → Continue
    │
    ├─ Medium?
    │   ├─ Add to backlog
    │   └─ Review in weekly meeting
    │
    └─ Low/Info?
        └─ Log for reference
```

### Alert Response Templates

**Critical Alert Response:**
```
1. Acknowledge immediately
2. Notify team: @security-team @oncall
3. Block deployment if not already deployed
4. Investigate:
   - Verify alert is valid
   - Check if already exploited
   - Assess impact
5. Remediate:
   - Remove/update package
   - Deploy fix
   - Verify resolution
6. Document:
   - Root cause
   - Actions taken
   - Prevention measures
7. Post-mortem (if needed)
```

**High Alert Response:**
```
1. Acknowledge within 4 hours
2. Assign to developer
3. Investigate:
   - Review alert details
   - Check for false positive
   - Assess risk
4. Plan remediation:
   - Update dependency
   - Test changes
   - Schedule deployment
5. Deploy fix within 1 week
6. Verify resolution
7. Document in ticket
```

### False Positive Handling

**Identification:**
```
Signs of false positive:
- Package is official variant (e.g., lodash-es)
- Well-known and trusted
- Alert doesn't apply to your use case
- Detection threshold too sensitive
```

**Process:**
```bash
# 1. Verify it's a false positive
sctv alert details <alert-id>

# 2. Document reason
sctv alert suppress <alert-id> \
  --reason "lodash-es is official ESM version of lodash" \
  --permanent

# 3. Update policy to prevent recurrence
sctv policy update <policy-id> \
  --add-exemption "lodash-es"

# 4. Share with team
# Update team wiki/docs with known false positives
```

---

## Policy Recommendations

### Production Policy

**Recommended Rules:**
```yaml
name: Production Security
description: Strict security for production environments

rules:
  # Essential
  - require_hash_verification:
      algorithms: [sha256]
      severity: high

  - block_typosquatting:
      threshold: 0.85
      severity: critical

  # Strongly Recommended
  - require_provenance:
      minimum_slsa_level: 1
      severity: medium

  - enforce_version_pinning:
      strategy: locked
      severity: medium

  # Recommended
  - require_minimum_age:
      days: 14
      exceptions: ["@yourcompany/*"]
      severity: low

  - deny_list:
      packages:
        - name_pattern: "colors"
          reason: "Deprecated"
      severity: high
```

### Development Policy

**Recommended Rules:**
```yaml
name: Development Security
description: Balanced security for development

rules:
  - block_typosquatting:
      threshold: 0.90
      severity: high

  - require_hash_verification:
      algorithms: [sha256]
      warn_only: true  # Don't block in dev
      severity: medium

  # More permissive
  - enforce_version_pinning:
      strategy: semver_minor
      severity: low
```

### Policy Evolution

**Gradual Rollout:**
```
Week 1: Observation mode
  - Enable policy
  - Log violations
  - Don't block
  - Understand baseline

Week 2-3: Soft enforcement
  - Warn on violations
  - Don't block builds
  - Address critical issues
  - Tune thresholds

Week 4+: Full enforcement
  - Block on critical/high
  - Fail builds
  - Full team trained
  - Exemptions documented
```

---

## CI/CD Integration Patterns

### Pre-Commit Hooks

**Prevent Bad Dependencies:**
```bash
#!/bin/bash
# .git/hooks/pre-commit

# Check if package files changed
if git diff --cached --name-only | grep -q "package.json\|package-lock.json"; then
    echo "Dependencies changed, running SCTV check..."

    # Quick scan
    sctv scan . --quick --fail-on critical

    if [ $? -ne 0 ]; then
        echo "❌ SCTV scan failed. Commit blocked."
        echo "Fix critical issues or use --no-verify to bypass."
        exit 1
    fi

    echo "✓ SCTV scan passed"
fi
```

### Pull Request Checks

**GitHub Actions:**
```yaml
name: Security Check

on: [pull_request]

jobs:
  sctv-scan:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: SCTV Scan
        uses: sctv/github-action@v1
        with:
          fail_on: critical,high
          comment_pr: true

      - name: SCTV Comment
        if: always()
        uses: sctv/pr-comment@v1
        with:
          summary: true
          show_passed: false
```

**PR Comment Example:**
```markdown
## SCTV Security Scan

### ❌ 2 issues found

#### Critical (1)
- **Typosquatting Detected**: `event-streem` similar to `event-stream`

#### High (1)
- **Hash Mismatch**: `axios@0.21.1` integrity check failed

### Recommendations
1. Remove `event-streem` and install official `event-stream`
2. Clear cache and reinstall `axios`

[View full report](https://sctv.example.com/scans/123)
```

### Deployment Gates

**Block Unsafe Deployments:**
```yaml
# deployment-pipeline.yml
stages:
  - build
  - test
  - security
  - deploy

security-gate:
  stage: security
  script:
    - sctv scan . --comprehensive
    - sctv policy evaluate production-policy --fail-on-violation
  allow_failure: false  # Block deployment on failure
  only:
    - main
```

### Branch Protection

**Require SCTV Checks:**
```yaml
# GitHub branch protection settings
Required checks:
  ✓ SCTV Security Scan
  ✓ SCTV Policy Check
  ✓ SCTV SBOM Generation

Settings:
  ✓ Require status checks to pass
  ✓ Require branches to be up to date
  ✓ Include administrators
```

---

## Team Workflows

### Roles and Responsibilities

**Security Team:**
```
Responsibilities:
- Define security policies
- Review critical alerts
- Approve exemptions
- Conduct audits
- Provide guidance

Daily Tasks:
- Monitor critical alerts
- Triage high severity
- Update policies
- Review exemption requests
```

**Development Team:**
```
Responsibilities:
- Follow security policies
- Respond to alerts
- Update dependencies
- Request exemptions
- Run scans before deployment

Daily Tasks:
- Check assigned alerts
- Review PR security comments
- Update vulnerable dependencies
- Run pre-deployment scans
```

**DevOps/SRE:**
```
Responsibilities:
- Maintain SCTV infrastructure
- Configure CI/CD integration
- Monitor scan performance
- Manage automation
- Generate compliance reports

Daily Tasks:
- Verify scans running
- Monitor system health
- Update integrations
- Support teams
```

### Communication Workflows

**Alert Escalation:**
```
Critical Alert Detected
    ↓
Immediate: PagerDuty + Slack
    ↓
Security Team Reviews (< 1 hour)
    ↓
├─ True Positive → Incident Response
│   ↓
│   DevOps blocks deployment
│   Dev team investigates
│   Security team coordinates
│   Fix deployed
│   Post-mortem
│
└─ False Positive → Suppress
    ↓
    Document reason
    Update policy
    Notify team
```

**Weekly Security Sync:**
```
Agenda:
1. Review week's alerts (10 min)
   - What was found?
   - What was fixed?
   - Any patterns?

2. Policy updates (5 min)
   - New rules needed?
   - Exemptions to add?
   - Threshold adjustments?

3. Upcoming work (5 min)
   - Dependency updates planned
   - New projects to onboard
   - Compliance deadlines

4. Improvements (5 min)
   - Process improvements
   - Tool enhancements
   - Training needs
```

### Knowledge Sharing

**Documentation:**
```
team-wiki/
├── sctv-quickstart.md
├── alert-response-playbook.md
├── common-false-positives.md
├── exemption-request-process.md
├── dependency-update-guide.md
└── compliance-requirements.md
```

**Training:**
```
New Developer Onboarding:
- Week 1: SCTV overview
- Week 2: Running scans
- Week 3: Alert triage
- Week 4: Policy compliance

Ongoing:
- Monthly lunch & learn
- Quarterly security updates
- Annual compliance training
```

### Metrics and Reporting

**Track KPIs:**
```yaml
Security Metrics:
  - Alert response time
  - Time to resolution
  - False positive rate
  - Policy compliance rate
  - Scan coverage

Quality Metrics:
  - Dependency freshness
  - Vulnerability age
  - SLSA adoption rate
  - SBOM generation rate

Operational Metrics:
  - Scan success rate
  - Scan duration
  - API availability
  - Alert volume trends
```

**Monthly Report:**
```markdown
# SCTV Monthly Report - January 2026

## Summary
- Projects monitored: 42
- Scans performed: 1,247
- Alerts generated: 156
- Alerts resolved: 142 (91%)
- Average response time: 4.2 hours

## Critical Findings
- 3 typosquatting attempts blocked
- 5 dependency tampering detected
- 2 major version downgrades prevented

## Team Performance
- Security team: 100% critical alerts < 1hr
- Dev team: 89% high alerts < 24hr
- Overall compliance: 94%

## Improvements
- Added 12 policy exemptions
- Updated 3 policies
- Onboarded 2 new projects
- Reduced false positive rate by 15%

## Next Month
- Roll out SLSA verification
- Train new team members
- Update compliance policies
```

---

## Security Checklist

### Pre-Deployment Checklist

```
Before deploying to production:

Dependencies:
☐ All dependencies scanned
☐ No critical/high alerts unresolved
☐ Lock files committed and synced
☐ SBOM generated

Security:
☐ Policy compliance verified
☐ All hashes verified
☐ Provenance checked (if applicable)
☐ No known vulnerabilities

Process:
☐ Code review completed
☐ Tests passing
☐ Security team approval (if needed)
☐ Deployment runbook ready
☐ Rollback plan documented

Documentation:
☐ CHANGELOG updated
☐ Dependencies documented
☐ Security notes added
☐ SBOM archived
```

### Quarterly Security Review

```
Every quarter:

Policy Review:
☐ Review all active policies
☐ Update rule thresholds
☐ Review exemptions (still needed?)
☐ Add new rules as needed

Dependency Audit:
☐ Review all direct dependencies
☐ Remove unused packages
☐ Update outdated packages
☐ Check for deprecated packages

Process Improvement:
☐ Review alert response times
☐ Analyze false positive rate
☐ Update team workflows
☐ Improve automation

Compliance:
☐ Generate compliance reports
☐ Verify SBOM coverage
☐ Audit log review
☐ Update documentation
```

---

## Additional Resources

### Learning Materials

**Documentation:**
- [Alert Management Guide](alerts.md)
- [Policy Guide](policies.md)
- [SBOM Guide](sbom.md)
- [CLI Reference](../reference/cli-reference.md)

**External Resources:**
- [NIST Secure Software Development Framework (SSDF)](https://csrc.nist.gov/Projects/ssdf)
- [SLSA Framework](https://slsa.dev/)
- [OpenSSF Best Practices](https://bestpractices.coreinfrastructure.org/)
- [OWASP Dependency-Check](https://owasp.org/www-project-dependency-check/)

### Community

**Get Help:**
- Documentation: https://docs.sctv.example.com
- Community Forum: https://community.sctv.example.com
- GitHub Issues: https://github.com/sctv/sctv/issues
- Slack: https://sctv-community.slack.com

**Stay Updated:**
- Security advisories
- Product updates
- Best practice guides
- Case studies

---

## Conclusion

Supply chain security is a continuous process, not a one-time effort. By following these best practices, you'll:

- ✓ Reduce security risks
- ✓ Improve compliance posture
- ✓ Enable faster incident response
- ✓ Build security into your workflow
- ✓ Create a security-conscious culture

**Remember:**
- Start small, improve gradually
- Automate where possible
- Keep team informed
- Document everything
- Learn from incidents

---

**Questions?** Contact the security team or check our [documentation](../README.md).
