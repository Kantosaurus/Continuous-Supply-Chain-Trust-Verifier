# Dashboard User Guide

**Version:** 0.1.0
**Last Updated:** 2026-01-15

The SCTV dashboard provides a comprehensive web interface for managing your software supply chain security. This guide covers all dashboard features and navigation.

---

## Table of Contents

- [Dashboard Overview](#dashboard-overview)
- [Home Page](#home-page)
- [Navigation](#navigation)
- [Project List View](#project-list-view)
- [Alert Summary](#alert-summary)
- [Security Score](#security-score)
- [Filtering and Search](#filtering-and-search)
- [User Preferences](#user-preferences)
- [Keyboard Shortcuts](#keyboard-shortcuts)

---

## Dashboard Overview

The SCTV dashboard is your central hub for supply chain security monitoring. It provides real-time visibility into:

- **Project health** - Status of all monitored projects
- **Active alerts** - Security findings requiring attention
- **Scan activity** - Recent and scheduled scans
- **Trending data** - Alert patterns over time
- **Quick actions** - Common tasks and workflows

### Accessing the Dashboard

1. Navigate to `https://your-sctv-instance.com`
2. Log in with your credentials
3. The home page loads automatically

**Default Port:** 8080 (configurable)

---

## Home Page

The dashboard home page provides an at-a-glance view of your security posture.

```
┌─────────────────────────────────────────────────────────────────┐
│                      SCTV Dashboard                             │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌────────────┐  ┌────────────┐  ┌────────────┐  ┌──────────┐ │
│  │  Projects  │  │   Alerts   │  │   Scans    │  │  Health  │ │
│  │     42     │  │     18     │  │     156    │  │   85%    │ │
│  │  Active    │  │   Open     │  │   Today    │  │  Score   │ │
│  └────────────┘  └────────────┘  └────────────┘  └──────────┘ │
│                                                                 │
│  ┌───────────────────────────────────────────────────────────┐ │
│  │ Alert Summary                                             │ │
│  ├───────────────────────────────────────────────────────────┤ │
│  │ CRITICAL: 3    ████░░░░░░░░░░░░░░░░░░░░░░░░              │ │
│  │ HIGH:     7    ██████████░░░░░░░░░░░░░░░░░                │ │
│  │ MEDIUM:   5    ████████░░░░░░░░░░░░░░░░░░░                │ │
│  │ LOW:      3    ████░░░░░░░░░░░░░░░░░░░░░░░                │ │
│  └───────────────────────────────────────────────────────────┘ │
│                                                                 │
│  ┌───────────────────────────────────────────────────────────┐ │
│  │ Recent Activity                                           │ │
│  ├───────────────────────────────────────────────────────────┤ │
│  │ • E-Commerce API - Critical alert: Typosquatting (2m ago) │ │
│  │ • Mobile App - Scan completed (15m ago)                   │ │
│  │ • Backend Services - Policy updated (1h ago)              │ │
│  │ • Frontend - New dependency detected (2h ago)             │ │
│  └───────────────────────────────────────────────────────────┘ │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### Home Page Widgets

#### 1. Project Summary Widget

**Location:** Top left
**Purpose:** Overview of all projects

**Displays:**
- Total number of projects
- Active vs inactive count
- Projects with critical alerts
- Projects requiring scans

**Actions:**
- Click to navigate to Projects page
- Filter by project status

#### 2. Alert Summary Widget

**Location:** Top center
**Purpose:** Current alert status

**Displays:**
- Total open alerts
- Count by severity level
- Alerts requiring triage
- Resolved alerts (last 7 days)

**Color Coding:**
- Red: Critical severity
- Orange: High severity
- Yellow: Medium severity
- Blue: Low severity
- Gray: Info severity

**Actions:**
- Click severity level to filter alerts
- Navigate to Alerts page

#### 3. Scan Activity Widget

**Location:** Top right
**Purpose:** Scan statistics

**Displays:**
- Scans completed today
- Active scans in progress
- Scheduled scans (next 24h)
- Average scan duration

**Actions:**
- View scan details
- Trigger manual scan

#### 4. Security Score Widget

**Location:** Top right secondary
**Purpose:** Overall security health

**Displays:**
- Aggregate security score (0-100)
- Trend indicator (↑ improving, ↓ declining)
- Score breakdown
- Compliance status

**Score Calculation:**
```
Score = 100 - (
  (critical_alerts × 10) +
  (high_alerts × 5) +
  (medium_alerts × 2) +
  (low_alerts × 1)
)
```

**Score Interpretation:**
- 90-100: Excellent - Strong security posture
- 75-89: Good - Minor issues to address
- 60-74: Fair - Several security concerns
- 40-59: Poor - Significant security risks
- 0-39: Critical - Immediate action required

#### 5. Recent Activity Feed

**Location:** Center
**Purpose:** Timeline of recent events

**Shows:**
- New alerts created
- Scans completed
- Policy changes
- User actions
- System notifications

**Each Entry Displays:**
- Timestamp (relative time)
- Project name
- Event type and description
- Severity indicator (for alerts)
- Quick action buttons

**Actions:**
- Click to view full details
- Filter by event type
- Export activity log

#### 6. Alert Trends Chart

**Location:** Lower left
**Purpose:** Visualize alert patterns

**Chart Types:**
- Line chart: Alerts over time
- Bar chart: Alerts by type
- Pie chart: Severity distribution
- Stacked area: Cumulative alerts

**Time Ranges:**
- Last 24 hours
- Last 7 days (default)
- Last 30 days
- Last 90 days
- Custom range

**Interaction:**
- Hover for detailed counts
- Click data points to filter
- Zoom in/out on timeline

#### 7. Top Projects by Alerts

**Location:** Lower center
**Purpose:** Identify problematic projects

**Displays:**
- Top 10 projects by alert count
- Severity breakdown for each
- Trend indicators
- Last scan time

**Example:**
```
┌────────────────────────────────────────────────┐
│ Project Name       Critical  High  Med   Low   │
├────────────────────────────────────────────────┤
│ E-Commerce API          3      5     2     1   │
│ Mobile App              2      3     4     0   │
│ Backend Services        1      2     1     3   │
│ Frontend                0      1     5     2   │
└────────────────────────────────────────────────┘
```

#### 8. Quick Actions Panel

**Location:** Right sidebar
**Purpose:** Common tasks

**Available Actions:**
- Create new project
- Run manual scan
- Generate SBOM
- Create policy
- View audit log
- Export reports

---

## Navigation

### Main Navigation Bar

**Location:** Top of screen (persistent)

```
┌─────────────────────────────────────────────────────────────┐
│ [SCTV Logo]  Projects  Alerts  Policies  SBOMs  Settings  │ │
│                                               [User] [Help] │
└─────────────────────────────────────────────────────────────┘
```

#### Menu Items

**Projects**
- All Projects
- Create New Project
- Archived Projects
- Project Templates

**Alerts**
- All Alerts
- Open Alerts
- Critical/High Only
- My Alerts (assigned to you)
- Alert Rules

**Policies**
- All Policies
- Create Policy
- Policy Templates
- Evaluation History

**SBOMs**
- Generate SBOM
- SBOM History
- Export Formats
- Compliance Reports

**Settings**
- User Profile
- Notifications
- Integrations
- API Keys
- Audit Log
- System Settings (admin only)

### Breadcrumb Navigation

**Location:** Below main nav
**Purpose:** Show current location

**Example:**
```
Home > Projects > E-Commerce API > Alerts
```

**Features:**
- Click any level to navigate back
- Shows full path hierarchy
- Updates dynamically

### User Menu

**Location:** Top right
**Icon:** User avatar or initials

**Options:**
- View Profile
- Change Password
- Notification Preferences
- API Keys
- Sign Out

### Help Menu

**Location:** Top right
**Icon:** Question mark

**Options:**
- Documentation
- Keyboard Shortcuts
- What's New
- Report Issue
- About SCTV

---

## Project List View

**Navigation:** Click "Projects" in main nav

The project list provides a comprehensive view of all projects.

```
┌─────────────────────────────────────────────────────────────┐
│ Projects                          [Search] [Filter] [+ New] │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│ ☑ Active  ☐ Inactive  ☐ Archived        Sort: [Name ▼]    │
│                                                             │
│ ┌─────────────────────────────────────────────────────────┐ │
│ │ E-Commerce API                                 [Actions] │ │
│ │ Status: CRITICAL    Alerts: 11    Dependencies: 847     │ │
│ │ Last scan: 2 hours ago    Next scan: Daily at 02:00     │ │
│ │ ●●●●●○○○○○ Security Score: 62/100                       │ │
│ └─────────────────────────────────────────────────────────┘ │
│                                                             │
│ ┌─────────────────────────────────────────────────────────┐ │
│ │ Mobile App                                     [Actions] │ │
│ │ Status: HEALTHY     Alerts: 2     Dependencies: 423     │ │
│ │ Last scan: 15 minutes ago    Next scan: Hourly          │ │
│ │ ●●●●●●●●●○ Security Score: 94/100                       │ │
│ └─────────────────────────────────────────────────────────┘ │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

### Project Card Layout

Each project displays:

**Header:**
- Project name (clickable to details)
- Actions menu (⋮)

**Status Badges:**
- HEALTHY (green) - No critical/high alerts
- WARNING (yellow) - Has medium/high alerts
- CRITICAL (red) - Has critical alerts
- UNKNOWN (gray) - Never scanned

**Metrics:**
- Alert count (by severity)
- Total dependencies
- Direct vs transitive

**Scan Information:**
- Last scan timestamp
- Next scheduled scan
- Scan duration

**Security Score:**
- Visual progress bar
- Numeric score (0-100)
- Trend indicator

### Project Actions Menu

**Available Actions:**
- View Details
- Run Scan Now
- View Dependencies
- View Alerts
- Generate SBOM
- Edit Settings
- Archive Project
- Delete Project (admin only)

### List View Options

**View Modes:**
- Card view (default)
- Table view
- Compact list

**Sorting:**
- Name (A-Z or Z-A)
- Status (Critical first)
- Alert count (High to low)
- Last scan (Recent first)
- Created date

**Filtering:**
- By status (Healthy, Warning, Critical)
- By active state (Active, Inactive, Archived)
- By ecosystem (npm, PyPI, Maven, etc.)
- By policy assignment
- By alert count (>0, >5, >10)

---

## Alert Summary

**Navigation:** Click "Alerts" in main nav

The alert summary provides detailed alert management.

```
┌─────────────────────────────────────────────────────────────┐
│ Alerts                               [Search] [Filter] [⋮]  │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│ Status: ☑ Open  ☐ Acknowledged  ☐ Resolved  ☐ Suppressed  │
│ Severity: [All ▼]    Type: [All ▼]    Project: [All ▼]     │
│                                                             │
│ ┌─────────────────────────────────────────────────────────┐ │
│ │ [!] CRITICAL                               2 minutes ago │ │
│ │ Typosquatting Detected                                  │ │
│ │ Package 'event-streem' similar to 'event-stream'        │ │
│ │ Project: E-Commerce API  |  Ecosystem: npm              │ │
│ │ [Acknowledge] [Resolve] [Suppress] [Details]            │ │
│ └─────────────────────────────────────────────────────────┘ │
│                                                             │
│ ┌─────────────────────────────────────────────────────────┐ │
│ │ [⚠] HIGH                                   15 minutes ago │ │
│ │ Dependency Tampering                                    │ │
│ │ Hash mismatch for 'axios@0.21.1'                        │ │
│ │ Project: Mobile App  |  Ecosystem: npm                  │ │
│ │ [Acknowledge] [Resolve] [Suppress] [Details]            │ │
│ └─────────────────────────────────────────────────────────┘ │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

### Alert Card Components

**Header:**
- Severity icon and label
- Alert type
- Timestamp (relative)

**Description:**
- Alert title
- Brief summary
- Affected package/version

**Metadata:**
- Project name
- Ecosystem
- Dependency details

**Actions:**
- Acknowledge - Mark as being investigated
- Resolve - Mark as fixed with details
- Suppress - Hide as false positive
- View Details - Full alert information

### Severity Icons

```
CRITICAL  [!]  Red exclamation
HIGH      [⚠]  Orange warning triangle
MEDIUM    [●]  Yellow circle
LOW       [i]  Blue info icon
INFO      [·]  Gray dot
```

### Alert Filtering

**By Status:**
- Open (default) - Not yet addressed
- Acknowledged - Under investigation
- Investigating - Active work in progress
- Resolved - Fixed and verified
- Suppressed - False positive or accepted risk
- False Positive - Marked as incorrect detection

**By Severity:**
- Critical only
- Critical + High
- Medium and above
- Low and above
- All severities

**By Type:**
- Typosquatting
- Dependency Tampering
- Downgrade Attack
- Provenance Failure
- Policy Violation
- New Package
- Suspicious Maintainer

**By Project:**
- Select from dropdown
- Multi-select (Ctrl/Cmd + click)
- Recent projects

**By Date Range:**
- Last 24 hours
- Last 7 days
- Last 30 days
- Custom range

### Bulk Operations

**Selection:**
- Click checkboxes on alerts
- Select all on page
- Select by filter criteria

**Available Actions:**
- Acknowledge selected
- Resolve selected
- Suppress selected
- Export to CSV/JSON
- Assign to user
- Change severity (override)

---

## Security Score

The security score provides a quantitative measure of your project's supply chain security.

### Score Calculation

**Formula:**
```
Base Score = 100

Deductions:
- Critical Alert: -10 points each
- High Alert:     -5 points each
- Medium Alert:   -2 points each
- Low Alert:      -1 point each

Bonuses:
+ All dependencies verified:     +5 points
+ SLSA Level 2+ provenance:      +3 points
+ Regular scans (< 24h old):     +2 points
+ Active policy enforcement:     +2 points
+ SBOM generated:                +1 point

Final Score = max(0, Base Score - Deductions + Bonuses)
```

### Score Breakdown View

**Access:** Click on security score widget

```
┌─────────────────────────────────────────────────┐
│ Security Score Breakdown                        │
├─────────────────────────────────────────────────┤
│                                                 │
│ Overall Score: 78/100                           │
│ ●●●●●●●●○○                                      │
│                                                 │
│ Components:                                     │
│                                                 │
│ Alert Impact:              -18 points           │
│   3 Critical (-30)                              │
│   2 High (-10)                                  │
│   1 Medium (-2)                                 │
│                                                 │
│ Security Measures:         +12 points           │
│   ✓ Dependencies verified  (+5)                 │
│   ✓ Regular scans          (+2)                 │
│   ✓ Policy active          (+2)                 │
│   ✓ SBOM generated         (+1)                 │
│   ✗ SLSA provenance        (0)                  │
│                                                 │
│ Recommendations:                                │
│ • Resolve 3 critical alerts for +30 points      │
│ • Enable SLSA verification for +3 points        │
│                                                 │
└─────────────────────────────────────────────────┘
```

### Score History

**View:** Line chart showing score over time

**Features:**
- Hover for daily scores
- Identify score drops
- Correlate with events
- Export data

### Score Comparison

**Compare:**
- Project to project
- Project to tenant average
- Current to previous period
- Against industry benchmarks (if available)

---

## Filtering and Search

### Global Search

**Location:** Top navigation bar
**Shortcut:** Ctrl/Cmd + K

**Search Across:**
- Project names and descriptions
- Alert titles and descriptions
- Package names
- Policy names
- Dependency names

**Search Syntax:**
```
term              - Simple search
"exact phrase"    - Exact match
project:name      - Filter by project
severity:critical - Filter by severity
type:typo*        - Wildcard search
status:open       - Filter by status
```

**Examples:**
```
"event-stream"                    - Find exact package
project:"E-Commerce" critical     - Critical alerts in project
severity:high OR severity:critical - High or critical alerts
type:typosquatting status:open    - Open typosquatting alerts
```

### Advanced Filters

**Access:** Click filter icon on any list page

**Filter Panel:**
```
┌─────────────────────────────────────┐
│ Filters                             │
├─────────────────────────────────────┤
│ Status:                             │
│ ☑ Open                              │
│ ☐ Acknowledged                      │
│ ☐ Resolved                          │
│                                     │
│ Severity:                           │
│ ☑ Critical                          │
│ ☑ High                              │
│ ☐ Medium                            │
│ ☐ Low                               │
│ ☐ Info                              │
│                                     │
│ Date Range:                         │
│ [Last 7 days        ▼]              │
│                                     │
│ Project:                            │
│ [Select projects... ▼]              │
│                                     │
│ [Apply Filters] [Reset]             │
└─────────────────────────────────────┘
```

### Saved Filters

**Purpose:** Save frequently used filter combinations

**Create Saved Filter:**
1. Configure filters
2. Click "Save Filter"
3. Name the filter
4. Choose visibility (personal or shared)

**Access Saved Filters:**
- Dropdown in filter panel
- Quick access buttons
- Share via URL

**Examples:**
- "My Critical Alerts" - Critical alerts in my projects
- "Last Week Unresolved" - Open alerts from last 7 days
- "Production Only" - Alerts from production projects

---

## User Preferences

**Access:** User menu → Preferences

### Display Preferences

**Theme:**
- Light mode (default)
- Dark mode
- Auto (follow system)

**Density:**
- Comfortable (default) - More spacing
- Compact - Denser layout
- Spacious - Maximum spacing

**Default Views:**
- Default project view (card/table/list)
- Default alert view
- Items per page (10/20/50/100)

### Notification Preferences

**Email Notifications:**
- Critical alerts (immediate)
- High alerts (daily digest)
- Medium/Low alerts (weekly digest)
- Scan completions
- Policy violations

**In-App Notifications:**
- Desktop notifications
- Sound alerts
- Badge counts
- Notification duration

**Notification Channels:**
- Email address
- Slack webhook
- Microsoft Teams webhook
- Mobile push (if configured)

### Dashboard Customization

**Widget Layout:**
- Drag and drop widgets
- Show/hide widgets
- Resize widgets
- Reset to default

**Default Filters:**
- Default project filter
- Default alert filter
- Default time range

**Data Refresh:**
- Auto-refresh interval (30s, 1m, 5m, 10m, off)
- Refresh on focus
- Show data age

---

## Keyboard Shortcuts

### Global Shortcuts

```
Ctrl/Cmd + K       Open global search
Ctrl/Cmd + /       Show keyboard shortcuts
Ctrl/Cmd + ,       Open preferences
Ctrl/Cmd + D       Go to dashboard
Ctrl/Cmd + P       Go to projects
Ctrl/Cmd + A       Go to alerts
Ctrl/Cmd + L       Go to policies
Escape             Close modal/drawer
```

### Navigation Shortcuts

```
G then D           Go to Dashboard
G then P           Go to Projects
G then A           Go to Alerts
G then L           Go to Policies
G then S           Go to Settings
```

### List View Shortcuts

```
J / ↓              Next item
K / ↑              Previous item
Enter              Open selected item
X                  Select/deselect item
Shift + X          Select all
Ctrl/Cmd + A       Select all
/                  Focus search
F                  Open filters
R                  Refresh list
```

### Alert Shortcuts

```
A                  Acknowledge selected
R                  Resolve selected
S                  Suppress selected
D                  View details
C                  Create comment
E                  Edit alert
```

### Form Shortcuts

```
Ctrl/Cmd + Enter   Submit form
Escape             Cancel/close
Tab                Next field
Shift + Tab        Previous field
```

---

## Best Practices

### Dashboard Usage

1. **Check Dashboard Daily**
   - Review new critical alerts
   - Monitor score trends
   - Check scan status

2. **Use Filters Effectively**
   - Create saved filters for common queries
   - Use status filters to focus on actionable items
   - Filter by project for team-specific views

3. **Organize Projects**
   - Use consistent naming conventions
   - Tag projects by team/environment
   - Archive inactive projects

4. **Set Up Notifications**
   - Critical alerts: immediate email
   - High alerts: daily digest
   - Scan failures: immediate notification

5. **Customize Your View**
   - Arrange widgets for your workflow
   - Set appropriate refresh intervals
   - Use dark mode for extended viewing

### Performance Tips

1. **Large Datasets**
   - Use filters to limit results
   - Increase pagination size carefully
   - Archive old projects

2. **Auto-Refresh**
   - Disable for static views
   - Use longer intervals (5-10m) for large datasets
   - Enable only for monitoring dashboards

3. **Browser Performance**
   - Clear browser cache periodically
   - Close unused tabs
   - Use modern browsers (Chrome, Firefox, Edge)

---

## Troubleshooting

### Common Issues

**Dashboard Not Loading**
- Check network connection
- Verify API server is running
- Clear browser cache
- Check browser console for errors

**Data Not Refreshing**
- Check auto-refresh setting
- Manually refresh (Ctrl/Cmd + R)
- Verify websocket connection
- Check server logs

**Filters Not Working**
- Clear all filters and reapply
- Refresh the page
- Check filter syntax
- Reset to default filters

**Slow Performance**
- Reduce pagination size
- Disable auto-refresh
- Close other browser tabs
- Check network latency

**Missing Data**
- Verify user permissions
- Check project assignments
- Refresh data manually
- Contact administrator

### Getting Help

**In-App Help:**
- Click (?) icon for context help
- Press Ctrl/Cmd + / for shortcuts
- Access documentation from help menu

**Support:**
- Check documentation
- Search community forum
- Contact support team
- Report bugs via GitHub

---

## Next Steps

- **[Projects Guide](projects.md)** - Learn to manage projects
- **[Alerts Guide](alerts.md)** - Master alert management
- **[Policies Guide](policies.md)** - Create security policies
- **[CLI Reference](../reference/cli-reference.md)** - Use command-line tools

---

**Need help?** Contact support or visit our [documentation](../README.md).
