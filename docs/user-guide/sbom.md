# SBOM Guide

**Version:** 0.1.0
**Last Updated:** 2026-01-15

Software Bill of Materials (SBOM) are essential for supply chain transparency and compliance. This guide covers everything you need to know about generating and using SBOMs with SCTV.

---

## Table of Contents

- [What is an SBOM?](#what-is-an-sbom)
- [CycloneDX vs SPDX](#cyclonedx-vs-spdx)
- [Generating SBOMs](#generating-sboms)
- [Exporting SBOMs](#exporting-sboms)
- [SBOM in CI/CD](#sbom-in-cicd)
- [Compliance Requirements](#compliance-requirements)
- [SBOM Best Practices](#sbom-best-practices)

---

## What is an SBOM?

### Definition

A **Software Bill of Materials (SBOM)** is a comprehensive inventory of all software components, dependencies, and metadata that make up an application.

Think of it as an **ingredients list** for software:
- What components are used
- Where they come from
- What versions are included
- How they're licensed
- Known vulnerabilities

### Why SBOMs Matter

**Security:**
- Quickly identify vulnerable components
- Understand attack surface
- Track dependency provenance
- Detect supply chain risks

**Compliance:**
- Required by many regulations (EO 14028, NTIA)
- Demonstrate due diligence
- License compliance tracking
- Audit trail

**Operational:**
- Vulnerability management
- Incident response
- Software transparency
- Risk assessment

### SBOM Components

**Essential Information:**
```yaml
Component:
  name: express
  version: 4.18.2
  type: library
  ecosystem: npm

Supplier:
  name: Node.js Foundation
  url: https://openjsf.org

Licenses:
  - MIT

Hashes:
  sha256: abc123def456...

External References:
  repository: https://github.com/expressjs/express
  website: https://expressjs.com
  issue_tracker: https://github.com/expressjs/express/issues

Dependencies:
  - accepts@1.3.8
  - array-flatten@1.1.1
  - body-parser@1.20.1
  - ...
```

**Metadata:**
- Creation timestamp
- Creator tool (SCTV)
- Document serial number
- Relationships between components
- Vulnerability references

### SBOM Use Cases

**1. Vulnerability Management**
```
New CVE announced → Check SBOM → Identify affected products
```

**2. License Compliance**
```
SBOM → Extract licenses → Verify compliance → Generate report
```

**3. Procurement**
```
Customer: "What's in your software?"
You: "Here's our SBOM" (instant transparency)
```

**4. Incident Response**
```
Security incident → SBOM → Identify affected components → Patch
```

**5. Regulatory Compliance**
```
Auditor: "Prove you track dependencies"
You: "Here are our SBOMs" (automated compliance)
```

---

## CycloneDX vs SPDX

SCTV supports two industry-standard SBOM formats.

### CycloneDX

**Overview:**
- Created by OWASP
- Designed for security use cases
- Version 1.5 (latest)
- JSON and XML formats

**Strengths:**
- Rich vulnerability information
- Service and dependency graph support
- Security-focused metadata
- Extensive tool ecosystem
- Continuous evolution

**Format:**
```json
{
  "bomFormat": "CycloneDX",
  "specVersion": "1.5",
  "serialNumber": "urn:uuid:550e8400-e29b-41d4-a716-446655440000",
  "version": 1,
  "metadata": {
    "timestamp": "2026-01-15T10:30:00Z",
    "tools": [
      {
        "vendor": "SCTV",
        "name": "Supply Chain Trust Verifier",
        "version": "0.1.0"
      }
    ],
    "component": {
      "type": "application",
      "name": "E-Commerce API",
      "version": "2.3.1"
    }
  },
  "components": [
    {
      "type": "library",
      "name": "express",
      "version": "4.18.2",
      "purl": "pkg:npm/express@4.18.2",
      "licenses": [
        {
          "license": {
            "id": "MIT"
          }
        }
      ],
      "hashes": [
        {
          "alg": "SHA-256",
          "content": "abc123def456..."
        }
      ],
      "externalReferences": [
        {
          "type": "website",
          "url": "https://expressjs.com"
        },
        {
          "type": "vcs",
          "url": "https://github.com/expressjs/express"
        }
      ]
    }
  ],
  "dependencies": [
    {
      "ref": "pkg:npm/express@4.18.2",
      "dependsOn": [
        "pkg:npm/accepts@1.3.8",
        "pkg:npm/body-parser@1.20.1"
      ]
    }
  ]
}
```

**Best For:**
- Security-focused organizations
- DevSecOps teams
- Vulnerability management
- Continuous monitoring
- Cloud-native applications

### SPDX

**Overview:**
- Created by Linux Foundation
- Industry standard since 2010
- Version 2.3 (current)
- JSON, tag-value, YAML, RDF formats
- ISO/IEC 5962:2021 standard

**Strengths:**
- Mature and stable
- Strong license compliance features
- Wide industry adoption
- ISO standard
- Legal clarity

**Format (Tag-Value):**
```
SPDXVersion: SPDX-2.3
DataLicense: CC0-1.0
SPDXID: SPDXRef-DOCUMENT
DocumentName: E-Commerce-API
DocumentNamespace: https://sctv.example.com/sboms/550e8400
Creator: Tool: SCTV-0.1.0
Created: 2026-01-15T10:30:00Z

PackageName: express
SPDXID: SPDXRef-Package-express-4.18.2
PackageVersion: 4.18.2
PackageSupplier: Organization: Node.js Foundation
PackageDownloadLocation: https://registry.npmjs.org/express/-/express-4.18.2.tgz
FilesAnalyzed: false
PackageVerificationCode: abc123def456...
PackageLicenseConcluded: MIT
PackageLicenseDeclared: MIT
PackageCopyrightText: Copyright (c) 2009-2023 TJ Holowaychuk
ExternalRef: PACKAGE-MANAGER purl pkg:npm/express@4.18.2

Relationship: SPDXRef-DOCUMENT DESCRIBES SPDXRef-Package-express-4.18.2
Relationship: SPDXRef-Package-express-4.18.2 DEPENDS_ON SPDXRef-Package-accepts-1.3.8
```

**Format (JSON):**
```json
{
  "spdxVersion": "SPDX-2.3",
  "dataLicense": "CC0-1.0",
  "SPDXID": "SPDXRef-DOCUMENT",
  "name": "E-Commerce-API",
  "documentNamespace": "https://sctv.example.com/sboms/550e8400",
  "creationInfo": {
    "created": "2026-01-15T10:30:00Z",
    "creators": ["Tool: SCTV-0.1.0"]
  },
  "packages": [
    {
      "SPDXID": "SPDXRef-Package-express-4.18.2",
      "name": "express",
      "versionInfo": "4.18.2",
      "supplier": "Organization: Node.js Foundation",
      "downloadLocation": "https://registry.npmjs.org/express/-/express-4.18.2.tgz",
      "filesAnalyzed": false,
      "licenseConcluded": "MIT",
      "licenseDeclared": "MIT",
      "copyrightText": "Copyright (c) 2009-2023 TJ Holowaychuk",
      "externalRefs": [
        {
          "referenceCategory": "PACKAGE-MANAGER",
          "referenceType": "purl",
          "referenceLocator": "pkg:npm/express@4.18.2"
        }
      ]
    }
  ],
  "relationships": [
    {
      "spdxElementId": "SPDXRef-DOCUMENT",
      "relatedSpdxElement": "SPDXRef-Package-express-4.18.2",
      "relationshipType": "DESCRIBES"
    }
  ]
}
```

**Best For:**
- License compliance
- Legal teams
- Open source management
- Government contracts
- Long-term archival

### Comparison

| Feature | CycloneDX | SPDX |
|---------|-----------|------|
| **Primary Focus** | Security | License compliance |
| **Vulnerability Info** | Extensive | Basic |
| **License Info** | Good | Extensive |
| **Formats** | JSON, XML | JSON, Tag-Value, YAML, RDF |
| **Graph Support** | Excellent | Good |
| **Adoption** | Growing fast | Wide, established |
| **Tool Support** | Modern tools | Broad ecosystem |
| **Spec Version** | 1.5 | 2.3 (ISO standard) |
| **Best For** | DevSecOps | Compliance, Legal |

### Which Format to Choose?

**Choose CycloneDX if:**
- Security is primary concern
- Using DevSecOps tools
- Need vulnerability tracking
- Want modern JSON format
- Building cloud-native apps

**Choose SPDX if:**
- License compliance is key
- Legal team requirements
- Government contracts
- Need ISO standard
- Long-term archival

**Use Both if:**
- Maximum compatibility
- Different stakeholders need different formats
- Compliance requires specific format
- Want redundancy

---

## Generating SBOMs

### Via Dashboard

**Step 1: Navigate to Project**
```
Dashboard → Projects → [Select Project] → SBOMs Tab
```

**Step 2: Generate SBOM**
```
┌─────────────────────────────────────────────┐
│ Generate SBOM                               │
├─────────────────────────────────────────────┤
│                                             │
│ Format: *                                   │
│ ○ CycloneDX JSON (recommended)              │
│ ○ CycloneDX XML                             │
│ ○ SPDX JSON                                 │
│ ○ SPDX Tag-Value                            │
│                                             │
│ Options:                                    │
│ ☑ Include dev dependencies                  │
│ ☑ Include transitive dependencies           │
│ ☑ Include license information               │
│ ☑ Include vulnerability references          │
│ ☑ Include cryptographic hashes              │
│ ☑ Include provenance information            │
│                                             │
│ SBOM Metadata:                              │
│ Author: [Security Team         ]            │
│ Organization: [Example Corp    ]            │
│                                             │
│      [Cancel] [Generate and Download]       │
│                                             │
└─────────────────────────────────────────────┘
```

**Step 3: Download**
- SBOM generates automatically
- Download button appears
- File saved with timestamp

**Filename Format:**
```
project-name_sbom_YYYY-MM-DD_HHmmss.cdx.json
e-commerce-api_sbom_2026-01-15_103045.cdx.json
```

### Via CLI

**Basic Generation:**
```bash
# Generate CycloneDX SBOM
sctv sbom generate <project-id> \
  --format cyclonedx \
  --output sbom.cdx.json

# Generate SPDX SBOM
sctv sbom generate <project-id> \
  --format spdx \
  --output sbom.spdx.json
```

**With Options:**
```bash
sctv sbom generate <project-id> \
  --format cyclonedx \
  --include-dev \
  --include-hashes \
  --include-vulnerabilities \
  --include-provenance \
  --author "Security Team" \
  --organization "Example Corp" \
  --output sbom.cdx.json
```

**All Formats:**
```bash
# Generate all SBOM formats
sctv sbom generate <project-id> --all-formats

# Outputs:
# - sbom.cdx.json (CycloneDX JSON)
# - sbom.cdx.xml (CycloneDX XML)
# - sbom.spdx.json (SPDX JSON)
# - sbom.spdx (SPDX Tag-Value)
```

### Via API

**Endpoint:** `POST /api/v1/projects/{id}/sbom`

**Request:**
```bash
curl -X POST https://sctv.example.com/api/v1/projects/{id}/sbom \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "format": "cyclonedx",
    "include_dev_dependencies": true,
    "include_hashes": true,
    "include_vulnerabilities": true
  }' \
  -o sbom.cdx.json
```

**Response Headers:**
```
Content-Type: application/vnd.cyclonedx+json
Content-Disposition: attachment; filename="e-commerce-api_sbom.cdx.json"
X-SBOM-Serial-Number: urn:uuid:550e8400-e29b-41d4-a716-446655440000
X-SBOM-Version: 1
X-SBOM-Timestamp: 2026-01-15T10:30:00Z
```

### Automated Generation

**Schedule Regular SBOM Generation:**
```yaml
# In project settings
sbom_generation:
  enabled: true
  schedule: weekly  # daily, weekly, monthly
  formats:
    - cyclonedx
    - spdx
  storage:
    type: s3
    bucket: sboms
    retention_days: 90
```

**Post-Scan Generation:**
```yaml
# Generate SBOM after each scan
post_scan_actions:
  - generate_sbom:
      formats: [cyclonedx]
      upload_to: artifact_storage
```

---

## Exporting SBOMs

### Export Locations

**Local File:**
```bash
sctv sbom generate <project-id> --output /path/to/sbom.json
```

**Cloud Storage:**
```bash
# S3
sctv sbom generate <project-id> --upload s3://my-bucket/sboms/

# Azure Blob
sctv sbom generate <project-id> --upload azure://container/sboms/

# Google Cloud Storage
sctv sbom generate <project-id> --upload gs://bucket/sboms/
```

**Artifact Repository:**
```bash
# Upload to artifact server
sctv sbom generate <project-id> \
  --upload https://artifacts.example.com/sboms/ \
  --auth-token $ARTIFACT_TOKEN
```

### Signing SBOMs

**GPG Signature:**
```bash
# Generate and sign
sctv sbom generate <project-id> \
  --format cyclonedx \
  --output sbom.cdx.json \
  --sign \
  --gpg-key security@example.com

# Creates:
# - sbom.cdx.json
# - sbom.cdx.json.sig
```

**Verify Signature:**
```bash
gpg --verify sbom.cdx.json.sig sbom.cdx.json
```

**Sigstore/Cosign:**
```bash
# Sign with Sigstore
cosign sign-blob sbom.cdx.json \
  --bundle sbom.cdx.json.bundle

# Verify
cosign verify-blob sbom.cdx.json \
  --bundle sbom.cdx.json.bundle \
  --certificate-identity security@example.com
```

### SBOM Distribution

**Embed in Container:**
```dockerfile
# Dockerfile
FROM node:18-alpine

# Copy application
COPY . /app
WORKDIR /app

# Generate and embed SBOM
RUN sctv sbom generate . --format cyclonedx --output /sbom.cdx.json

# SBOM accessible at /sbom.cdx.json in container
```

**Package with Release:**
```bash
# Include SBOM in release artifacts
tar czf release-v1.2.3.tar.gz \
  app/ \
  sbom.cdx.json \
  sbom.spdx.json \
  README.md
```

**Host on Web Server:**
```
https://example.com/sboms/
  ├── product-a/
  │   ├── v1.0.0/
  │   │   ├── sbom.cdx.json
  │   │   └── sbom.spdx.json
  │   └── v1.1.0/
  │       ├── sbom.cdx.json
  │       └── sbom.spdx.json
  └── product-b/
      └── ...
```

---

## SBOM in CI/CD

### GitHub Actions

**Workflow:**
```yaml
name: Generate SBOM

on:
  push:
    branches: [main]
  release:
    types: [published]

jobs:
  sbom:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Install SCTV CLI
        run: |
          curl -sSL https://sctv.example.com/install.sh | sh
          sctv auth login --token ${{ secrets.SCTV_TOKEN }}

      - name: Generate SBOM
        run: |
          sctv sbom generate . \
            --format cyclonedx \
            --format spdx \
            --include-dev \
            --include-hashes \
            --output-dir sboms/

      - name: Sign SBOM
        run: |
          cosign sign-blob sboms/sbom.cdx.json \
            --bundle sboms/sbom.cdx.json.bundle

      - name: Upload to Release
        uses: actions/upload-release-asset@v1
        with:
          upload_url: ${{ github.event.release.upload_url }}
          asset_path: sboms/sbom.cdx.json
          asset_name: sbom.cdx.json
          asset_content_type: application/vnd.cyclonedx+json

      - name: Upload to Artifact Storage
        run: |
          aws s3 cp sboms/ s3://company-sboms/${{ github.repository }}/${{ github.sha }}/ \
            --recursive
```

### GitLab CI

**Pipeline:**
```yaml
# .gitlab-ci.yml
stages:
  - build
  - test
  - sbom
  - deploy

generate-sbom:
  stage: sbom
  image: sctv/cli:latest
  script:
    - sctv auth login --token $SCTV_TOKEN
    - sctv sbom generate . --format cyclonedx --output sbom.cdx.json
    - sctv sbom generate . --format spdx --output sbom.spdx.json
  artifacts:
    paths:
      - sbom.cdx.json
      - sbom.spdx.json
    expire_in: 1 year
  only:
    - main
    - tags
```

### Jenkins

**Jenkinsfile:**
```groovy
pipeline {
    agent any

    stages {
        stage('Build') {
            steps {
                sh 'npm install'
                sh 'npm run build'
            }
        }

        stage('Generate SBOM') {
            steps {
                withCredentials([string(credentialsId: 'sctv-token', variable: 'SCTV_TOKEN')]) {
                    sh '''
                        sctv auth login --token $SCTV_TOKEN
                        sctv sbom generate . \
                            --format cyclonedx \
                            --output sbom.cdx.json \
                            --include-vulnerabilities
                    '''
                }
            }
        }

        stage('Archive SBOM') {
            steps {
                archiveArtifacts artifacts: 'sbom.cdx.json', fingerprint: true
            }
        }

        stage('Upload SBOM') {
            steps {
                sh 's3cmd put sbom.cdx.json s3://sbom-storage/${JOB_NAME}/${BUILD_NUMBER}/'
            }
        }
    }
}
```

### Container Builds

**Multi-stage Dockerfile:**
```dockerfile
# Build stage
FROM node:18 AS builder
WORKDIR /app
COPY package*.json ./
RUN npm ci
COPY . .
RUN npm run build

# SBOM generation stage
FROM sctv/cli:latest AS sbom-generator
COPY --from=builder /app /app
WORKDIR /app
RUN sctv sbom generate . \
    --format cyclonedx \
    --output /sbom.cdx.json

# Final stage
FROM node:18-alpine
WORKDIR /app
COPY --from=builder /app/dist ./dist
COPY --from=builder /app/node_modules ./node_modules
COPY --from=sbom-generator /sbom.cdx.json /sbom.cdx.json

# SBOM available at /sbom.cdx.json
EXPOSE 3000
CMD ["node", "dist/index.js"]
```

---

## Compliance Requirements

### U.S. Executive Order 14028

**Requirements:**
- Software vendors must provide SBOM
- Use industry-standard formats (SPDX, CycloneDX)
- Include component names, versions, suppliers
- Maintain and update SBOMs

**SCTV Support:**
```bash
# Generate EO 14028 compliant SBOM
sctv sbom generate <project-id> \
  --format cyclonedx \
  --format spdx \
  --include-all \
  --compliance eo14028
```

### NTIA Minimum Elements

**Required Fields:**
1. Supplier Name
2. Component Name
3. Version of Component
4. Other Unique Identifiers
5. Dependency Relationship
6. Author of SBOM Data
7. Timestamp

**SCTV Compliance:**
```json
{
  "supplier": "Node.js Foundation",      // ✓ Supplier Name
  "name": "express",                     // ✓ Component Name
  "version": "4.18.2",                   // ✓ Version
  "purl": "pkg:npm/express@4.18.2",     // ✓ Unique Identifier
  "dependencies": [...],                 // ✓ Relationships
  "author": "Security Team",             // ✓ Author
  "timestamp": "2026-01-15T10:30:00Z"   // ✓ Timestamp
}
```

### Industry Standards

**OpenSSF:**
```bash
# Generate OpenSSF-compliant SBOM
sctv sbom generate <project-id> \
  --format cyclonedx \
  --include-provenance \
  --include-signatures \
  --compliance openssf
```

**CISA:**
```bash
# CISA SBOM requirements
sctv sbom generate <project-id> \
  --format spdx \
  --include-all \
  --compliance cisa
```

### Compliance Validation

**Validate SBOM:**
```bash
# Validate against NTIA minimum elements
sctv sbom validate sbom.cdx.json --standard ntia

# Output:
✓ Supplier Name: Present
✓ Component Name: Present
✓ Version: Present
✓ Unique Identifiers: Present
✓ Dependencies: Present
✓ Author: Present
✓ Timestamp: Present

Result: COMPLIANT with NTIA minimum elements
```

**Tools:**
```bash
# CycloneDX validator
cyclonedx-cli validate --input-file sbom.cdx.json

# SPDX validator
java -jar spdx-tools.jar Verify sbom.spdx.json
```

---

## SBOM Best Practices

### When to Generate SBOMs

**Recommended:**
- Every release (production)
- Major version changes
- Security patches
- Compliance audits
- Customer requests

**Optional:**
- Every build (development)
- Nightly builds
- Feature branches
- Pre-release versions

### SBOM Storage

**Version Control:**
```
repository/
  ├── src/
  ├── package.json
  └── sboms/
      ├── v1.0.0/
      │   ├── sbom.cdx.json
      │   └── sbom.spdx.json
      ├── v1.1.0/
      │   ├── sbom.cdx.json
      │   └── sbom.spdx.json
      └── latest/
          ├── sbom.cdx.json
          └── sbom.spdx.json
```

**Artifact Storage:**
```bash
# S3 bucket organization
s3://company-sboms/
  ├── products/
  │   ├── api/
  │   │   ├── 2026/
  │   │   │   ├── 01/
  │   │   │   │   └── api_v1.2.3_20260115.cdx.json
  │   │   │   └── 02/
  │   │   └── latest.cdx.json
  │   └── frontend/
  └── compliance-reports/
```

**Retention Policy:**
```yaml
retention:
  production_releases: 7 years
  development_builds: 90 days
  feature_branches: 30 days
```

### SBOM Accuracy

**Keep Updated:**
```bash
# Regenerate after dependency changes
npm install new-package
sctv sbom generate . --output sbom.cdx.json
```

**Verify Accuracy:**
```bash
# Compare SBOM to actual dependencies
sctv sbom verify sbom.cdx.json --against package-lock.json

# Output:
✓ All components in SBOM found in lock file
✓ All lock file entries in SBOM
✓ Versions match
✓ Hashes match

Result: SBOM ACCURATE
```

**Track Changes:**
```bash
# Diff two SBOMs
sctv sbom diff sbom-v1.0.0.json sbom-v1.1.0.json

# Output:
Added (3):
  + axios@1.6.2
  + dotenv@16.0.3
  + helmet@7.1.0

Removed (1):
  - request@2.88.2

Updated (5):
  express: 4.18.1 → 4.18.2
  ...
```

### Documentation

**Include README:**
```markdown
# SBOM Documentation

## About This SBOM

Generated: 2026-01-15
Format: CycloneDX 1.5 JSON
Tool: SCTV v0.1.0
Project: E-Commerce API v1.2.3

## Contents

- 847 total components
- 156 direct dependencies
- 691 transitive dependencies

## Ecosystems

- npm: 423 packages
- PyPI: 424 packages

## Verification

Hash: sha256:abc123def456...
Signature: See sbom.cdx.json.sig
Signed by: security@example.com

## Usage

```bash
# Validate
cyclonedx-cli validate --input-file sbom.cdx.json

# Search for component
jq '.components[] | select(.name=="express")' sbom.cdx.json

# List all licenses
jq '.components[].licenses[].license.id' sbom.cdx.json | sort -u
```

## Contact

Questions: security@example.com
```

### SBOM Sharing

**Public Projects:**
```bash
# Publish SBOM alongside release
https://github.com/org/project/releases/download/v1.2.3/sbom.cdx.json
```

**Customer Delivery:**
```
Email: sbom@example.com
Subject: SBOM for Product X v1.2.3

Attached:
- sbom.cdx.json (CycloneDX format)
- sbom.spdx.json (SPDX format)
- sbom.cdx.json.sig (GPG signature)
- README.md (Documentation)
```

**Self-Service Portal:**
```
https://sbom.example.com/
  └── [Login]
      └── Download SBOMs
          ├── Filter by product
          ├── Filter by version
          └── Select format
```

---

## Advanced Topics

### Enriching SBOMs

**Add Vulnerability Data:**
```bash
sctv sbom generate <project-id> \
  --format cyclonedx \
  --include-vulnerabilities \
  --vulnerability-sources osv,nvd,github
```

**Add Provenance:**
```bash
sctv sbom generate <project-id> \
  --include-provenance \
  --slsa-level 2
```

**Custom Metadata:**
```bash
sctv sbom generate <project-id> \
  --metadata "environment:production" \
  --metadata "team:backend" \
  --metadata "compliance:pci-dss"
```

### SBOM Analysis

**License Analysis:**
```bash
# Extract all licenses
sctv sbom analyze sbom.cdx.json --licenses

# Output:
MIT: 423 components (50%)
Apache-2.0: 234 components (27%)
BSD-3-Clause: 156 components (18%)
ISC: 34 components (4%)
```

**Vulnerability Scan:**
```bash
# Scan SBOM for vulnerabilities
sctv sbom scan sbom.cdx.json

# Output:
Critical: 2
High: 5
Medium: 12
Low: 23
```

**Dependency Analysis:**
```bash
# Find deep dependencies
sctv sbom analyze sbom.cdx.json --depth

# Output:
Max depth: 8 levels
Average depth: 4.2 levels
Most deep: prototype-pollution-lib (8 levels deep)
```

### SBOM Automation

**Pre-commit Hook:**
```bash
#!/bin/bash
# .git/hooks/pre-commit

# Regenerate SBOM if dependencies changed
if git diff --cached --name-only | grep -q "package-lock.json"; then
    echo "Dependencies changed, regenerating SBOM..."
    sctv sbom generate . --output sbom.cdx.json
    git add sbom.cdx.json
fi
```

**Automated Updates:**
```bash
# Cron job: Update SBOMs nightly
0 2 * * * cd /app && sctv sbom generate . --upload s3://sboms/
```

---

## Troubleshooting

### Common Issues

**Large SBOM Size**

**Problem:** SBOM file is very large (>10MB)

**Solutions:**
- Exclude dev dependencies
- Exclude transitive dependencies beyond depth N
- Use compressed format
- Split into multiple SBOMs

**Missing Components**

**Problem:** Some dependencies not in SBOM

**Solutions:**
- Ensure all lock files present
- Check ecosystem detection
- Verify scan completed successfully
- Include all dependency types

**Invalid SBOM**

**Problem:** SBOM fails validation

**Solutions:**
- Validate with official tools
- Check format version
- Verify required fields
- Review error messages

---

## Next Steps

- **[Policy Guide](policies.md)** - Enforce SBOM generation
- **[Projects Guide](projects.md)** - Configure SBOM settings
- **[CI/CD Integration](../api/webhooks.md)** - Automate SBOM generation
- **[Best Practices](best-practices.md)** - SBOM recommendations

---

**Need help with SBOMs?** Check the [compliance guide](../operations/compliance.md) or contact support.
