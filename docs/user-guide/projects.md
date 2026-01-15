# Projects Management Guide

**Version:** 0.1.0
**Last Updated:** 2026-01-15

Projects are the foundation of SCTV's supply chain monitoring. This guide covers everything you need to know about creating, configuring, and managing projects.

---

## Table of Contents

- [What is a Project?](#what-is-a-project)
- [Creating a New Project](#creating-a-new-project)
- [Project Settings](#project-settings)
- [Managing Ecosystems](#managing-ecosystems)
- [Viewing Dependencies](#viewing-dependencies)
- [Dependency Tree](#dependency-tree)
- [Project Scan History](#project-scan-history)
- [Project-Level Policies](#project-level-policies)
- [Project Status and Health](#project-status-and-health)
- [Archiving and Deleting](#archiving-and-deleting)

---

## What is a Project?

A **project** in SCTV represents a software application or service with dependencies that you want to monitor for supply chain security threats.

### Project Components

Each project includes:

- **Name and description** - Identify the project
- **Repository URL** - Link to source code (optional)
- **Ecosystems** - Package managers to scan (npm, PyPI, Maven, etc.)
- **Scan schedule** - Automated scanning frequency
- **Policy** - Security rules to enforce
- **Dependencies** - Direct and transitive packages
- **Alerts** - Security findings from scans
- **SBOMs** - Software Bill of Materials

### Project Types

**By Environment:**
- Production applications
- Development/staging projects
- Test environments
- Third-party integrations

**By Technology:**
- Frontend (JavaScript/npm)
- Backend (Python/PyPI, Java/Maven)
- Mobile apps (various ecosystems)
- Microservices
- Monolithic applications

---

## Creating a New Project

### Via Dashboard

**Step-by-step:**

1. **Navigate to Projects**
   - Click "Projects" in main navigation
   - Or press `Ctrl/Cmd + P`

2. **Click "Create Project" Button**
   - Located in top-right corner
   - Or press `N` key on projects page

3. **Fill in Project Details**

```
┌─────────────────────────────────────────────────┐
│ Create New Project                              │
├─────────────────────────────────────────────────┤
│                                                 │
│ Project Name: *                                 │
│ [E-Commerce API                 ]               │
│                                                 │
│ Description:                                    │
│ [Main REST API for e-commerce platform         │
│  Handles orders, inventory, and payments]       │
│                                                 │
│ Repository URL:                                 │
│ [https://github.com/example/ecommerce-api]      │
│                                                 │
│ Default Branch:                                 │
│ [main                           ]               │
│                                                 │
│ Ecosystems: *                                   │
│ ☑ npm                  ☐ Maven                  │
│ ☑ PyPI                 ☐ NuGet                  │
│ ☐ RubyGems             ☐ Cargo                  │
│ ☐ Go Modules                                    │
│                                                 │
│ Scan Schedule:                                  │
│ ○ Manual                                        │
│ ○ Hourly                                        │
│ ● Daily at [02]:00 UTC                          │
│ ○ Weekly on [Monday ▼] at [02]:00 UTC          │
│ ○ On Push (webhook)                             │
│                                                 │
│ Security Policy:                                │
│ [Production Security    ▼]                      │
│                                                 │
│ Tags:                                           │
│ [production] [backend] [critical]               │
│                                                 │
│         [Cancel]  [Create Project]              │
│                                                 │
└─────────────────────────────────────────────────┘
```

4. **Required Fields** (marked with *)
   - Project name
   - At least one ecosystem

5. **Click "Create Project"**

### Via CLI

```bash
# Basic project creation
sctv project create \
  --name "E-Commerce API" \
  --description "Main REST API" \
  --ecosystems npm,pypi

# With all options
sctv project create \
  --name "E-Commerce API" \
  --description "Main REST API for e-commerce platform" \
  --repository "https://github.com/example/ecommerce-api" \
  --branch "main" \
  --ecosystems npm,pypi \
  --schedule "daily:02:00" \
  --policy "Production Security" \
  --tags "production,backend,critical"
```

### Via API

**Endpoint:** `POST /api/v1/projects`

**Request:**
```json
{
  "name": "E-Commerce API",
  "description": "Main REST API for e-commerce platform",
  "repository_url": "https://github.com/example/ecommerce-api",
  "default_branch": "main",
  "ecosystems": ["npm", "pypi"]
}
```

**Response:**
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "name": "E-Commerce API",
  "description": "Main REST API for e-commerce platform",
  "repository_url": "https://github.com/example/ecommerce-api",
  "default_branch": "main",
  "status": "unknown",
  "is_active": true,
  "dependency_count": 0,
  "alert_count": 0,
  "last_scan_at": null,
  "created_at": "2026-01-15T10:30:00Z",
  "updated_at": "2026-01-15T10:30:00Z"
}
```

---

## Project Settings

**Access:** Project page → Settings tab

### General Settings

**Project Information:**
- Name (required)
- Description
- Repository URL
- Default branch
- Status (active/inactive)

**Example:**
```yaml
Name: E-Commerce API
Description: Main REST API for e-commerce platform
Repository: https://github.com/example/ecommerce-api
Branch: main
Status: Active
```

### Scan Configuration

**Scan Schedule Options:**

1. **Manual**
   - No automatic scans
   - Trigger scans manually or via API/CLI
   - Best for: Development projects, testing

2. **Hourly**
   - Scans every hour
   - High resource usage
   - Best for: Critical production systems

3. **Daily**
   - Scans once per day at specified hour (UTC)
   - Recommended for most projects
   - Best for: Production applications
   - Example: Daily at 02:00 UTC

4. **Weekly**
   - Scans once per week on specified day/hour
   - Lower resource usage
   - Best for: Stable, low-change projects
   - Example: Weekly on Sunday at 02:00 UTC

5. **On Push**
   - Triggered by webhook events
   - Requires webhook configuration
   - Best for: CI/CD integration
   - See [Webhooks Guide](../api/webhooks.md)

**Scan Options:**
```yaml
Schedule: Daily at 02:00 UTC

Options:
  Include dev dependencies: Yes
  Deep dependency scanning: Yes
  Verify all checksums: Yes
  Check SLSA provenance: Yes
  Timeout: 30 minutes
```

### Notification Settings

**Project-Specific Notifications:**

**Alert Notifications:**
```yaml
Critical Alerts:
  - Email: immediate
  - Slack: #security-alerts
  - PagerDuty: High priority

High Alerts:
  - Email: daily digest
  - Slack: #security-alerts

Medium/Low Alerts:
  - Email: weekly digest
```

**Scan Notifications:**
```yaml
Scan Completed:
  - Email: Never
  - Slack: Never

Scan Failed:
  - Email: Immediate
  - Slack: #ops-alerts
  - PagerDuty: Low priority
```

### Metadata and Tags

**Tags:**
- Categorize projects
- Filter and search
- Group related projects
- Automate policies

**Common Tags:**
```
Environment: production, staging, development
Team: backend, frontend, mobile, devops
Priority: critical, high, medium, low
Language: javascript, python, java, rust
Compliance: sox, pci-dss, hipaa
```

**Usage:**
```yaml
Tags:
  - production
  - backend
  - critical
  - nodejs
  - pci-dss
```

---

## Managing Ecosystems

Projects can monitor dependencies from multiple package ecosystems simultaneously.

### Supported Ecosystems

**Available:**
- **npm** - JavaScript/Node.js packages
- **PyPI** - Python packages
- **Maven** - Java packages
- **NuGet** - .NET packages
- **RubyGems** - Ruby packages
- **Cargo** - Rust packages
- **Go Modules** - Go packages

### Adding Ecosystems

**Via Dashboard:**

1. Go to project settings
2. Click "Ecosystems" tab
3. Check ecosystems to add
4. Save changes

**Via CLI:**
```bash
# Add single ecosystem
sctv project update <project-id> --add-ecosystem npm

# Add multiple ecosystems
sctv project update <project-id> --add-ecosystems npm,pypi,maven
```

**Via API:**
```bash
curl -X PATCH https://sctv.example.com/api/v1/projects/{id} \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "ecosystems": ["npm", "pypi", "maven"]
  }'
```

### Removing Ecosystems

**Important:** Removing an ecosystem:
- Deletes associated dependencies
- Removes related alerts
- Cannot be undone

**Process:**
1. Navigate to project settings
2. Uncheck ecosystem
3. Confirm deletion warning
4. Save changes

### Ecosystem-Specific Configuration

**npm:**
```yaml
Registry: https://registry.npmjs.org
Manifest files:
  - package.json
  - package-lock.json
Include dev dependencies: Yes
Private registry: None
```

**PyPI:**
```yaml
Registry: https://pypi.org
Manifest files:
  - requirements.txt
  - Pipfile.lock
  - poetry.lock
  - pyproject.toml
Include dev dependencies: Yes
```

**Maven:**
```yaml
Repository: https://repo1.maven.org/maven2
Manifest files:
  - pom.xml
Include test scope: Yes
Private repository: None
```

---

## Viewing Dependencies

**Access:** Project page → Dependencies tab

### Dependencies List View

```
┌─────────────────────────────────────────────────────────────┐
│ Dependencies (847)                    [Search] [Filter] [↓] │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│ Direct (156)  Transitive (691)  All                         │
│ Ecosystem: [All ▼]  Has Alerts: [All ▼]  Sort: [Name ▼]    │
│                                                             │
│ ┌─────────────────────────────────────────────────────────┐ │
│ │ express  [npm]                            Direct         │ │
│ │ Version: 4.18.2                                         │ │
│ │ Latest: 4.18.2  ✓ Up to date                           │ │
│ │ Alerts: 0  Security Score: 98/100                       │ │
│ │ Hash: sha256:abc123...  Verified ✓                     │ │
│ │ SLSA: Level 2  ✓ Provenance verified                   │ │
│ │ [View Details] [Check for Updates] [View Tree]         │ │
│ └─────────────────────────────────────────────────────────┘ │
│                                                             │
│ ┌─────────────────────────────────────────────────────────┐ │
│ │ axios  [npm]                              Direct    [!] │ │
│ │ Version: 0.21.1                                         │ │
│ │ Latest: 1.6.2  ⚠ Update available                      │ │
│ │ Alerts: 2 (1 Critical, 1 High)                          │ │
│ │ Hash: sha256:def456...  Mismatch ✗                     │ │
│ │ SLSA: None  ⚠ No provenance                            │ │
│ │ [View Details] [View Alerts] [Update]                  │ │
│ └─────────────────────────────────────────────────────────┘ │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

### Dependency Details

**Click on dependency for full details:**

```
┌─────────────────────────────────────────────────┐
│ axios @ 0.21.1                              [✕] │
├─────────────────────────────────────────────────┤
│                                                 │
│ Overview                                        │
│ ────────────────────────────────────────────── │
│ Ecosystem: npm                                  │
│ Type: Direct dependency                         │
│ Current Version: 0.21.1                         │
│ Latest Version: 1.6.2                           │
│ License: MIT                                    │
│                                                 │
│ Package Information                             │
│ ────────────────────────────────────────────── │
│ Homepage: https://axios-http.com                │
│ Repository: https://github.com/axios/axios      │
│ Registry: https://www.npmjs.com/package/axios   │
│ Downloads: 50M+ per week                        │
│                                                 │
│ Security Status                                 │
│ ────────────────────────────────────────────── │
│ Alerts: 2 (1 Critical, 1 High)                  │
│   ⚠ Dependency Tampering (CRITICAL)            │
│   ⚠ Provenance Failure (HIGH)                  │
│                                                 │
│ Integrity                                       │
│ ────────────────────────────────────────────── │
│ SHA-256: abc123...def456  ✗ Mismatch           │
│ Expected: xyz789...abc123                       │
│ Source: registry.npmjs.org                      │
│                                                 │
│ Provenance                                      │
│ ────────────────────────────────────────────── │
│ SLSA Level: None                                │
│ Signature: Not signed                           │
│ Attestation: Not available                      │
│                                                 │
│ Dependencies (14)                               │
│ ────────────────────────────────────────────── │
│ follow-redirects @ 1.15.2                       │
│ form-data @ 4.0.0                               │
│ proxy-from-env @ 1.1.0                          │
│ ...view all                                     │
│                                                 │
│ Used By                                         │
│ ────────────────────────────────────────────── │
│ Imported by 12 other packages                   │
│ [View dependency graph]                         │
│                                                 │
│ [View Alerts] [Update Package] [Remove]         │
│                                                 │
└─────────────────────────────────────────────────┘
```

### Dependency Filtering

**Filter by Type:**
- Direct dependencies only
- Transitive dependencies only
- All dependencies

**Filter by Ecosystem:**
- npm
- PyPI
- Maven
- All ecosystems

**Filter by Status:**
- Has alerts
- Outdated versions
- No provenance
- Hash mismatches
- All dependencies

**Search:**
- Package name
- Version
- License
- Maintainer

---

## Dependency Tree

**Access:** Project → Dependencies → Tree View

The dependency tree visualizes your project's dependency graph.

### Tree Visualization

```
E-Commerce API
│
├─ express @ 4.18.2  [npm] ✓
│  ├─ accepts @ 1.3.8
│  ├─ array-flatten @ 1.1.1
│  ├─ body-parser @ 1.20.1
│  │  ├─ bytes @ 3.1.2
│  │  ├─ content-type @ 1.0.5
│  │  └─ raw-body @ 2.5.1
│  ├─ content-disposition @ 0.5.4
│  └─ ...more dependencies
│
├─ axios @ 0.21.1  [npm] ⚠ 2 alerts
│  ├─ follow-redirects @ 1.15.2
│  ├─ form-data @ 4.0.0
│  │  ├─ asynckit @ 0.4.0
│  │  ├─ combined-stream @ 1.0.8
│  │  └─ mime-types @ 2.1.35
│  └─ proxy-from-env @ 1.1.0
│
├─ lodash @ 4.17.21  [npm] ✓
│
└─ requests @ 2.28.2  [pypi] ✓
   ├─ charset-normalizer @ 3.0.1
   ├─ idna @ 3.4
   ├─ urllib3 @ 1.26.14
   └─ certifi @ 2022.12.7
```

### Tree Features

**Expand/Collapse:**
- Click package to expand/collapse
- Expand all / Collapse all buttons
- Default: 2 levels expanded

**Legend:**
```
✓ No issues
⚠ Has alerts
✗ Critical issues
[!] Outdated
[?] Unknown status
```

**Interactions:**
- Click package for details
- Hover for quick info
- Right-click for context menu

**Context Menu:**
- View package details
- View alerts
- Check for updates
- View on registry
- Copy package name
- Exclude from scan

### Tree Filters

**Show/Hide:**
- Dev dependencies
- Peer dependencies
- Optional dependencies
- Bundled dependencies

**Highlight:**
- Packages with alerts
- Outdated packages
- Packages without provenance
- Duplicates

**Depth:**
- Direct only (depth 1)
- 2 levels (default)
- 3 levels
- All levels

### Export Tree

**Formats:**
- ASCII text
- JSON
- GraphML (for graph tools)
- DOT (Graphviz)
- SBOM (CycloneDX/SPDX)

**Example Export:**
```bash
# Via CLI
sctv project dependencies <project-id> \
  --format tree \
  --depth 3 \
  --output tree.txt

# Via API
curl https://sctv.example.com/api/v1/projects/{id}/dependencies/tree \
  -H "Authorization: Bearer $TOKEN" \
  > tree.json
```

---

## Project Scan History

**Access:** Project → Scans tab

### Scan List

```
┌─────────────────────────────────────────────────────────────┐
│ Scan History                              [Filter] [Export] │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│ ┌─────────────────────────────────────────────────────────┐ │
│ │ Scan #156                          2 hours ago     ✓    │ │
│ │ Status: Completed                                       │ │
│ │ Duration: 2m 34s                                        │ │
│ │ Dependencies: 847 (156 direct, 691 transitive)          │ │
│ │ New Alerts: 2 (1 Critical, 1 High)                      │ │
│ │ Ecosystems: npm (423), pypi (424)                       │ │
│ │ [View Details] [View Alerts] [Download SBOM]           │ │
│ └─────────────────────────────────────────────────────────┘ │
│                                                             │
│ ┌─────────────────────────────────────────────────────────┐ │
│ │ Scan #155                          1 day ago       ✓    │ │
│ │ Status: Completed                                       │ │
│ │ Duration: 2m 18s                                        │ │
│ │ Dependencies: 845 (156 direct, 689 transitive)          │ │
│ │ New Alerts: 0                                           │ │
│ │ Ecosystems: npm (421), pypi (424)                       │ │
│ │ [View Details] [Compare]                               │ │
│ └─────────────────────────────────────────────────────────┘ │
│                                                             │
│ ┌─────────────────────────────────────────────────────────┐ │
│ │ Scan #154                          2 days ago      ✗    │ │
│ │ Status: Failed                                          │ │
│ │ Error: Timeout connecting to registry                  │ │
│ │ Duration: 30m 00s (timeout)                             │ │
│ │ [View Logs] [Retry]                                    │ │
│ └─────────────────────────────────────────────────────────┘ │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

### Scan Details

**Click scan for detailed results:**

```
Scan #156 Details
─────────────────────────────────────────────

Scan Information:
  ID: 550e8400-e29b-41d4-a716-446655440156
  Started: 2026-01-15 08:15:23 UTC
  Completed: 2026-01-15 08:17:57 UTC
  Duration: 2 minutes 34 seconds
  Triggered by: Scheduled scan

Scan Results:
  Total Dependencies: 847
    Direct: 156
    Transitive: 691

  Ecosystems Scanned:
    npm: 423 packages
    pypi: 424 packages

  Changes from Previous Scan:
    Added: 2 packages
    Removed: 0 packages
    Updated: 5 packages

Alerts Generated:
  Total: 2 new alerts
    Critical: 1 (Dependency Tampering)
    High: 1 (Provenance Failure)
    Medium: 0
    Low: 0

Verification:
  Hash Verification: 845/847 verified (99.8%)
  SLSA Provenance: 234/847 available (27.6%)
  Signatures: 89/847 signed (10.5%)

Performance:
  Registry Queries: 1,247
  Cache Hits: 623 (50%)
  Network Time: 1m 45s
  Processing Time: 49s
```

### Comparing Scans

**Compare two scans to see changes:**

1. Select two scans
2. Click "Compare"
3. View differences

**Comparison View:**
```
Comparing Scan #156 vs Scan #155
─────────────────────────────────────────────

Dependencies Changed:
  Added (2):
    + axios-retry @ 3.8.0 [npm]
    + python-dateutil @ 2.8.2 [pypi]

  Removed (0):
    (none)

  Updated (5):
    • lodash: 4.17.20 → 4.17.21
    • express: 4.18.1 → 4.18.2
    • requests: 2.28.1 → 2.28.2
    • urllib3: 1.26.13 → 1.26.14
    • certifi: 2022.09.24 → 2022.12.7

Alerts Changed:
  New Alerts (2):
    + Dependency Tampering on axios@0.21.1
    + Provenance Failure on axios@0.21.1

  Resolved Alerts (1):
    - Downgrade Attack on lodash (fixed by update)

Security Score:
  Previous: 78/100
  Current: 62/100
  Change: -16 (due to 2 new critical/high alerts)
```

### Scan Retention

**Default Retention:**
- All scans: 90 days
- Failed scans: 30 days
- Can be configured per project

**Export Before Deletion:**
```bash
# Export scan results
sctv scan export <scan-id> --format json > scan-156.json

# Export all scans for project
sctv project scans <project-id> --export --format csv
```

---

## Project-Level Policies

Projects can have custom security policies that override tenant defaults.

### Assigning a Policy

**Via Dashboard:**

1. Go to project settings
2. Click "Policy" tab
3. Select policy from dropdown
4. Save changes

**Via CLI:**
```bash
# Assign policy
sctv project update <project-id> --policy "Production Security"

# Remove policy (use tenant default)
sctv project update <project-id> --policy none
```

### Policy Inheritance

```
Tenant Default Policy
  ↓
Project Policy (if assigned)
  ↓
Dependency-Specific Overrides
```

**Example:**
```yaml
Tenant: "Strict Security" (default for all projects)

Project "E-Commerce API": "Production Security" (overrides tenant)
  - Stricter than tenant default
  - Additional rules for production

Project "Test Suite": (no policy assigned)
  - Uses tenant default "Strict Security"
```

### Creating Project-Specific Policies

**Scenario:** Production project needs stricter rules

**Steps:**

1. **Create Policy**
   ```yaml
   Name: Production Security
   Description: Enhanced security for production projects

   Rules:
     - Block deprecated packages (critical)
     - Require SLSA Level 2+ provenance
     - Block packages < 30 days old
     - Require minimum 10,000 downloads
     - Block maintainer changes < 90 days
     - Require hash verification (SHA-256)
   ```

2. **Assign to Project**
   - Go to project settings
   - Select "Production Security" policy
   - Save

3. **Verify**
   - Run test scan
   - Check alerts generated
   - Adjust rules as needed

### Policy Evaluation Results

**View policy evaluation:**

**Access:** Project → Policy tab

```
Policy Evaluation: Production Security
─────────────────────────────────────────────

Last Evaluated: 2 hours ago (Scan #156)

Rule Results:
  ✓ Block deprecated packages (0 violations)
  ✗ Require SLSA Level 2+ (123 violations)
  ✓ Block packages < 30 days (0 violations)
  ✗ Require minimum downloads (5 violations)
  ✓ Block maintainer changes (0 violations)
  ✓ Require hash verification (2 violations)

Violations by Package:
  axios@0.21.1
    ✗ No SLSA provenance
    ✗ Hash verification failed

  new-package@1.0.0
    ✗ Only 2,500 downloads

  ...view all violations
```

---

## Project Status and Health

### Status Types

**HEALTHY** (Green)
- No critical or high severity alerts
- All scans passing
- Security score > 80

**WARNING** (Yellow)
- Has medium or high severity alerts
- Some dependencies outdated
- Security score 60-80

**CRITICAL** (Red)
- Has critical severity alerts
- Failed policy evaluation
- Security score < 60

**UNKNOWN** (Gray)
- Never scanned
- Last scan > 30 days ago
- Scan in progress

### Status Updates

**Automatic Updates:**
- After each scan completion
- When alerts are resolved
- When policies change

**Manual Override:**
```bash
# Mark project as inactive (no auto-scans)
sctv project update <project-id> --inactive

# Reactivate project
sctv project update <project-id> --active
```

### Health Metrics

**Dashboard Display:**
```
Project Health: E-Commerce API
─────────────────────────────────────────────

Status: WARNING
Security Score: 62/100 (↓ -16 from last scan)

Metrics:
  Dependencies: 847 total
    Direct: 156
    Transitive: 691

  Alerts: 11 open
    Critical: 3
    High: 7
    Medium: 1
    Low: 0

  Updates Available: 23 packages
    Major: 5
    Minor: 12
    Patch: 6

  Verification Status:
    Hash Verified: 99.8%
    SLSA Provenance: 27.6%
    Signed: 10.5%

Recommendations:
  1. Resolve 3 critical alerts (-30 points)
  2. Update 23 outdated packages
  3. Enable SLSA verification (+3 points)
```

---

## Archiving and Deleting

### Archiving Projects

**When to Archive:**
- Project is no longer active
- End of life application
- Replaced by newer version
- Keep historical data

**How to Archive:**

**Via Dashboard:**
1. Go to project settings
2. Click "Archive Project"
3. Confirm action

**Via CLI:**
```bash
sctv project archive <project-id>
```

**Effects:**
- Project hidden from main list
- Scans disabled
- Data retained
- Can be restored

**View Archived:**
- Projects page → Show archived
- Filter: Status = Archived

**Restore:**
```bash
sctv project restore <project-id>
```

### Deleting Projects

**Warning:** Deletion is permanent and cannot be undone.

**What Gets Deleted:**
- Project metadata
- All dependencies
- All alerts
- Scan history
- Generated SBOMs
- Policy assignments

**What's Preserved:**
- Audit log entries
- Referenced in tenant statistics

**How to Delete:**

**Via Dashboard:**
1. Go to project settings
2. Scroll to "Danger Zone"
3. Click "Delete Project"
4. Type project name to confirm
5. Click "Permanently Delete"

**Via CLI:**
```bash
# Delete with confirmation prompt
sctv project delete <project-id>

# Force delete without prompt (dangerous!)
sctv project delete <project-id> --force
```

**Via API:**
```bash
curl -X DELETE https://sctv.example.com/api/v1/projects/{id} \
  -H "Authorization: Bearer $TOKEN"
```

---

## Best Practices

### Project Organization

1. **Naming Conventions**
   ```
   Good:
     - E-Commerce API
     - Mobile App - iOS
     - Payment Service

   Bad:
     - project1
     - test
     - new-thing
   ```

2. **Use Descriptive Descriptions**
   - What the project does
   - Who owns it
   - Environment (prod/staging/dev)

3. **Consistent Tagging**
   ```yaml
   Tags:
     Environment: [production|staging|development]
     Team: [backend|frontend|mobile|devops]
     Priority: [critical|high|medium|low]
     Compliance: [sox|pci-dss|hipaa|gdpr]
   ```

4. **Logical Grouping**
   - Group by team
   - Group by environment
   - Group by technology stack

### Scan Schedules

**Production Projects:**
- Daily scans at low-traffic hours
- Enable immediate notifications
- Use strict policies

**Development Projects:**
- Weekly scans
- Relaxed policies
- Email digests

**Critical Infrastructure:**
- Hourly scans
- Strictest policies
- PagerDuty integration

### Policy Assignment

**Start Permissive:**
1. Create project
2. Assign permissive policy
3. Run initial scan
4. Review alerts
5. Gradually tighten policy

**Example Progression:**
```
Week 1: Permissive policy
  → Understand baseline

Week 2: Add typosquatting detection
  → Catch obvious threats

Week 3: Add hash verification
  → Detect tampering

Week 4: Full production policy
  → Complete protection
```

### Dependency Management

1. **Regular Updates**
   - Schedule dependency updates
   - Test in staging first
   - Update production weekly

2. **Lock Files**
   - Always commit lock files
   - Verify after updates
   - Use SCTV to detect changes

3. **Minimize Dependencies**
   - Audit dependencies regularly
   - Remove unused packages
   - Consider alternatives

---

## Troubleshooting

### Common Issues

**Scan Failures**

**Symptom:** Scan times out or fails

**Solutions:**
- Increase timeout in project settings
- Check registry connectivity
- Verify manifest files exist
- Check logs for detailed errors

**Missing Dependencies**

**Symptom:** Dependencies not detected

**Solutions:**
- Verify correct ecosystem selected
- Check manifest file format
- Ensure repository access
- Run scan manually to test

**Incorrect Alerts**

**Symptom:** False positive alerts

**Solutions:**
- Review alert details
- Suppress false positives
- Adjust policy rules
- Report bugs to support

### Getting Help

**Documentation:**
- [Scanning Guide](scanning.md)
- [Alert Management](alerts.md)
- [Policy Guide](policies.md)

**Support:**
- Check audit logs
- Review error messages
- Contact support team
- Community forum

---

## Next Steps

- **[Viewing Dependencies](#viewing-dependencies)** - Explore your dependency tree
- **[Alert Management](alerts.md)** - Respond to security alerts
- **[Policy Guide](policies.md)** - Create custom policies
- **[SBOM Guide](sbom.md)** - Generate compliance reports

---

**Ready to create your first project?** Follow the [Quick Start Guide](../getting-started/quickstart.md).
