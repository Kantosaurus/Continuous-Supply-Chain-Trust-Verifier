# Architecture Overview

**Version:** 0.1.0

This document provides a comprehensive overview of the Supply Chain Trust Verifier (SCTV) system architecture.

---

## Table of Contents

- [Design Principles](#design-principles)
- [High-Level Architecture](#high-level-architecture)
- [System Components](#system-components)
- [Technology Stack](#technology-stack)
- [Data Flow](#data-flow)
- [Deployment Architecture](#deployment-architecture)
- [Security Architecture](#security-architecture)
- [Scalability and Performance](#scalability-and-performance)

---

## Design Principles

SCTV is built on the following architectural principles:

### 1. Security First
- **Memory Safety:** Built in Rust to prevent memory-related vulnerabilities
- **Type Safety:** Strong type system eliminates entire classes of bugs
- **Secure by Default:** Security features enabled out-of-the-box
- **Defense in Depth:** Multiple layers of security controls

### 2. Multi-Tenancy
- **Tenant Isolation:** Strict data separation between organizations
- **Row-Level Security:** Database-level tenant isolation
- **Resource Quotas:** Per-tenant limits and rate limiting
- **Audit Logging:** Complete audit trail for compliance

### 3. Scalability
- **Horizontal Scaling:** Scale API servers and workers independently
- **Asynchronous Processing:** Background jobs for long-running tasks
- **Efficient Caching:** Minimize redundant registry queries
- **Connection Pooling:** Optimized database access

### 4. Reliability
- **Fault Tolerance:** Graceful degradation under load
- **Retry Mechanisms:** Automatic retry with exponential backoff
- **Job Recovery:** Resume interrupted scans
- **Health Checks:** Comprehensive health monitoring

### 5. Observability
- **Structured Logging:** JSON-formatted logs for analysis
- **Metrics Export:** Prometheus-compatible metrics
- **Distributed Tracing:** Request tracing across services
- **Audit Trails:** Complete activity logging

---

## High-Level Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                         Client Layer                                │
│  ┌──────────────┐  ┌──────────────┐  ┌────────────┐  ┌──────────┐ │
│  │  Web Browser │  │     CLI      │  │  CI/CD     │  │   API    │ │
│  │  (Dashboard) │  │    Tool      │  │  Pipeline  │  │  Clients │ │
│  └──────┬───────┘  └──────┬───────┘  └─────┬──────┘  └────┬─────┘ │
└─────────┼──────────────────┼─────────────────┼──────────────┼───────┘
          │                  │                 │              │
          └──────────────────┴─────────────────┴──────────────┘
                                     │
┌────────────────────────────────────┼─────────────────────────────────┐
│                          API Gateway Layer                           │
│  ┌──────────────────────────────────────────────────────────────┐   │
│  │              API Server (sctv-api)                           │   │
│  │  ┌────────────┐  ┌────────────┐  ┌──────────────────────┐   │   │
│  │  │  REST API  │  │  GraphQL   │  │   Authentication     │   │   │
│  │  │  Handlers  │  │   Schema   │  │   & Authorization    │   │   │
│  │  └────────────┘  └────────────┘  └──────────────────────┘   │   │
│  │  ┌────────────┐  ┌────────────┐  ┌──────────────────────┐   │   │
│  │  │  Webhook   │  │ Middleware │  │    Rate Limiting     │   │   │
│  │  │  Handlers  │  │   Stack    │  │   & Validation       │   │   │
│  │  └────────────┘  └────────────┘  └──────────────────────┘   │   │
│  └──────────────────────────────────────────────────────────────┘   │
└──────────────────────────────┬───────────────────────────────────────┘
                               │
          ┌────────────────────┴─────────────────────┐
          │                                          │
┌─────────┴─────────────┐                ┌───────────┴──────────────┐
│   Application Layer   │                │    Worker Layer          │
│  ┌─────────────────┐  │                │  ┌────────────────────┐  │
│  │  sctv-core      │  │                │  │  sctv-worker       │  │
│  │  Domain Models  │  │                │  │  Job Queue System  │  │
│  │  Business Logic │  │                │  └────────┬───────────┘  │
│  └─────────────────┘  │                │           │              │
│  ┌─────────────────┐  │                │  ┌────────┴───────────┐  │
│  │  sctv-detectors │  │                │  │  Job Executors     │  │
│  │  Threat Engines │◄─┼────────────────┼─►│  - ScanProject     │  │
│  └─────────────────┘  │                │  │  - VerifyProvenance│  │
│  ┌─────────────────┐  │                │  │  - MonitorRegistry │  │
│  │ sctv-registries │  │                │  │  - Notifications   │  │
│  │ Package Clients │  │                │  └────────────────────┘  │
│  └─────────────────┘  │                └──────────────────────────┘
│  ┌─────────────────┐  │
│  │   sctv-sbom     │  │
│  │ SBOM Generators │  │
│  └─────────────────┘  │
│  ┌─────────────────┐  │
│  │sctv-notifications│ │
│  │ Alert Delivery  │  │
│  └─────────────────┘  │
└───────────┬───────────┘
            │
┌───────────┴─────────────────────────────────────────────────────────┐
│                       Persistence Layer                             │
│  ┌──────────────────────────────────────────────────────────────┐   │
│  │                    sctv-db (Database Layer)                  │   │
│  │  ┌────────────┐  ┌────────────┐  ┌──────────────────────┐   │   │
│  │  │ Repository │  │   Models   │  │   Connection Pool    │   │   │
│  │  │   Pattern  │  │  Mapping   │  │    Management        │   │   │
│  │  └────────────┘  └────────────┘  └──────────────────────┘   │   │
│  └──────────────────────┬───────────────────────────────────────┘   │
│                         │                                           │
│  ┌──────────────────────┴───────────────────────────────────────┐   │
│  │                   PostgreSQL Database                        │   │
│  │  ┌──────┐  ┌─────────┐  ┌──────┐  ┌──────────┐  ┌────────┐  │   │
│  │  │Tenants│  │Projects │  │Alerts│  │Policies  │  │  Jobs  │  │   │
│  │  └──────┘  └─────────┘  └──────┘  └──────────┘  └────────┘  │   │
│  │  ┌──────┐  ┌─────────┐  ┌──────┐  ┌──────────┐  ┌────────┐  │   │
│  │  │ Users│  │  Deps   │  │Pkgs  │  │  SBOMs   │  │Audit   │  │   │
│  │  └──────┘  └─────────┘  └──────┘  └──────────┘  └────────┘  │   │
│  └──────────────────────────────────────────────────────────────┘   │
└──────────────────────────────────────────────────────────────────────┘

┌──────────────────────────────────────────────────────────────────────┐
│                      External Integrations                           │
│  ┌──────────┐  ┌─────────┐  ┌──────────┐  ┌────────┐  ┌─────────┐  │
│  │   npm    │  │  PyPI   │  │  Maven   │  │ Cargo  │  │  NuGet  │  │
│  │ Registry │  │Registry │  │ Central  │  │Registry│  │ Gallery │  │
│  └──────────┘  └─────────┘  └──────────┘  └────────┘  └─────────┘  │
│  ┌──────────┐  ┌─────────┐  ┌──────────┐  ┌────────┐  ┌─────────┐  │
│  │ Sigstore │  │  Rekor  │  │  Fulcio  │  │ Slack  │  │PagerDuty│  │
│  │ Cosign   │  │   Log   │  │   CA     │  │ Hooks  │  │ Events  │  │
│  └──────────┘  └─────────┘  └──────────┘  └────────┘  └─────────┘  │
└──────────────────────────────────────────────────────────────────────┘
```

---

## System Components

### 1. sctv-core
**Purpose:** Domain models and business logic

**Responsibilities:**
- Define core domain entities (Project, Dependency, Alert, Policy, etc.)
- Business rules and validation
- Service trait definitions
- Domain events

**Key Modules:**
- `domain/` - Entity definitions
- `traits/` - Service and repository interfaces
- `events/` - Domain event types

### 2. sctv-api
**Purpose:** HTTP API server (REST + GraphQL)

**Responsibilities:**
- RESTful API endpoints
- GraphQL query and mutation resolvers
- Authentication and authorization
- Request validation and error handling
- Webhook receivers (GitHub, GitLab)

**Key Modules:**
- `rest/` - REST API handlers
- `graphql/` - GraphQL schema and resolvers
- `auth/` - JWT authentication
- `middleware/` - Request middleware stack

**Technology:**
- Axum web framework
- async-graphql for GraphQL
- JWT for authentication

### 3. sctv-db
**Purpose:** Database access layer

**Responsibilities:**
- PostgreSQL connection management
- Repository pattern implementation
- SQL query execution
- Database migrations
- Multi-tenant data isolation

**Key Modules:**
- `repositories/` - Data access implementations
- `models/` - Database models
- `pool.rs` - Connection pool management

**Technology:**
- SQLx for async database access
- PostgreSQL 14+

### 4. sctv-detectors
**Purpose:** Threat detection engines

**Responsibilities:**
- Typosquatting detection (edit distance, visual similarity)
- Tampering detection (hash verification)
- Downgrade attack detection
- Provenance verification (SLSA, Sigstore)
- Maintainer activity monitoring

**Key Modules:**
- `typosquatting.rs` - Name similarity detection
- `tampering.rs` - Integrity verification
- `downgrade.rs` - Version rollback detection
- `provenance.rs` - SLSA provenance validation

**Algorithms:**
- Levenshtein distance
- Jaro-Winkler similarity
- Visual similarity (character substitution)
- Cryptographic hash verification

### 5. sctv-registries
**Purpose:** Package registry clients

**Responsibilities:**
- Fetch package metadata from registries
- Download package artifacts
- Query package versions
- Cache registry responses
- Rate limiting per registry

**Supported Ecosystems:**
- npm (JavaScript/Node.js)
- PyPI (Python)
- Maven Central (Java)
- NuGet Gallery (.NET)
- crates.io (Rust)
- RubyGems (Ruby)
- Go modules (Go)

**Key Modules:**
- `npm.rs` - npm registry client
- `pypi.rs` - PyPI client
- `maven.rs` - Maven Central client
- `cache.rs` - Response caching

### 6. sctv-sbom
**Purpose:** Software Bill of Materials generation

**Responsibilities:**
- Generate CycloneDX SBOMs
- Generate SPDX SBOMs
- Package URL (purl) generation
- License expression handling
- Vulnerability correlation

**Supported Formats:**
- CycloneDX 1.5 (JSON, XML)
- SPDX 2.3 (JSON, tag-value)

**Key Modules:**
- `cyclonedx.rs` - CycloneDX generator
- `spdx.rs` - SPDX generator
- `common.rs` - Shared SBOM utilities

### 7. sctv-worker
**Purpose:** Background job processing

**Responsibilities:**
- Job queue management
- Concurrent job execution
- Retry logic with exponential backoff
- Job prioritization
- Stale job recovery

**Job Types:**
- `ScanProject` - Scan project dependencies
- `VerifyProvenance` - Verify SLSA attestations
- `MonitorRegistry` - Monitor package updates
- `SendNotification` - Deliver alerts

**Key Modules:**
- `queue.rs` - PostgreSQL-backed job queue
- `pool.rs` - Worker thread pool
- `executor.rs` - Job execution logic
- `service.rs` - High-level worker service

**Technology:**
- PostgreSQL `SELECT FOR UPDATE SKIP LOCKED` for atomic job claiming
- Tokio async runtime
- Configurable worker pool

### 8. sctv-notifications
**Purpose:** Multi-channel alert delivery

**Responsibilities:**
- Send notifications to configured channels
- Severity-based filtering
- Retry failed deliveries
- Template rendering

**Supported Channels:**
- Email (SMTP)
- Slack (webhooks)
- Microsoft Teams (webhooks)
- PagerDuty (Events API v2)
- Generic webhooks

**Key Modules:**
- `channels/` - Channel implementations
- `service.rs` - Notification coordinator
- `types.rs` - Notification types

### 9. sctv-ci
**Purpose:** CI/CD integrations

**Responsibilities:**
- Generate SARIF reports
- Determine CI exit codes
- Format results for CI tools

**Supported Formats:**
- SARIF 2.1.0 (GitHub Code Scanning, GitLab SAST)

**Key Modules:**
- `lib.rs` - SARIF generation

### 10. sctv-cli
**Purpose:** Command-line interface

**Responsibilities:**
- Local dependency scanning
- Project management
- Alert triage
- SBOM generation
- Configuration management

**Commands:**
- `scan` - Scan dependencies
- `check` - Check specific package
- `verify` - Verify package integrity
- `policy` - Evaluate policies

**Technology:**
- Clap for CLI parsing

### 11. sctv-dashboard
**Purpose:** Web-based user interface

**Responsibilities:**
- Project management UI
- Alert visualization
- Policy configuration
- User management
- Analytics and reporting

**Technology:**
- Leptos (Rust WASM framework)
- Server-side rendering (SSR)
- Client-side hydration

---

## Technology Stack

### Backend
- **Language:** Rust 1.75+
- **Async Runtime:** Tokio
- **Web Framework:** Axum 0.8
- **GraphQL:** async-graphql 7.0
- **Database:** PostgreSQL 14+ with SQLx
- **Serialization:** serde, serde_json

### Frontend
- **Framework:** Leptos 0.7
- **Rendering:** SSR with hydration
- **Compilation:** WASM

### Infrastructure
- **Containerization:** Docker
- **Orchestration:** Kubernetes (optional)
- **Reverse Proxy:** nginx (recommended)
- **Monitoring:** Prometheus + Grafana
- **Logging:** structured JSON logs

### Security
- **Memory Safety:** Rust's ownership model
- **Cryptography:** ring, sha2
- **JWT:** jsonwebtoken
- **SLSA:** Sigstore/cosign integration

---

## Data Flow

### Dependency Scan Flow

```
1. User triggers scan (CLI, API, scheduled job)
   │
   ▼
2. API creates ScanProject job
   │
   ▼
3. Worker claims job from queue
   │
   ▼
4. Worker analyzes project files
   │
   ├─► package.json (npm)
   ├─► requirements.txt (PyPI)
   ├─► pom.xml (Maven)
   └─► Cargo.toml (Rust)
   │
   ▼
5. Extract dependency list
   │
   ▼
6. For each dependency:
   │
   ├─► Fetch metadata from registry
   ├─► Verify checksums
   ├─► Check provenance attestations
   ├─► Run threat detectors
   └─► Evaluate policies
   │
   ▼
7. Generate alerts for violations
   │
   ▼
8. Store results in database
   │
   ▼
9. Queue notification jobs
   │
   ▼
10. Send alerts to configured channels
   │
   ▼
11. Update scan status to "completed"
```

### Alert Notification Flow

```
1. Alert created during scan
   │
   ▼
2. Evaluate notification rules
   │
   ├─► Check severity threshold
   ├─► Apply tenant settings
   └─► Check rate limits
   │
   ▼
3. Queue SendNotification job
   │
   ▼
4. Worker claims notification job
   │
   ▼
5. Render notification template
   │
   ▼
6. Send to configured channels:
   │
   ├─► Email (SMTP)
   ├─► Slack (webhook)
   ├─► Teams (webhook)
   ├─► PagerDuty (Events API)
   └─► Custom webhook
   │
   ▼
7. Record delivery status
   │
   ▼
8. Retry on failure (exponential backoff)
```

---

## Deployment Architecture

### Single-Server Deployment

```
┌─────────────────────────────────────┐
│         Single Host                 │
│  ┌───────────────────────────────┐  │
│  │  Docker Compose               │  │
│  │  ┌─────────┐  ┌──────────┐   │  │
│  │  │ API     │  │ Worker   │   │  │
│  │  │ Server  │  │ Pool     │   │  │
│  │  └────┬────┘  └─────┬────┘   │  │
│  │       │             │         │  │
│  │       └──────┬──────┘         │  │
│  │              │                │  │
│  │       ┌──────┴──────┐         │  │
│  │       │ PostgreSQL  │         │  │
│  │       └─────────────┘         │  │
│  └───────────────────────────────┘  │
└─────────────────────────────────────┘
```

### High-Availability Deployment

```
┌────────────────────────────────────────────────────────────────┐
│                      Load Balancer (nginx)                     │
└──────────────────────┬─────────────────────────────────────────┘
                       │
        ┌──────────────┴──────────────┐
        │                             │
┌───────┴────────┐            ┌───────┴────────┐
│  API Server 1  │            │  API Server 2  │
│  (Kubernetes)  │            │  (Kubernetes)  │
└───────┬────────┘            └───────┬────────┘
        │                             │
        └──────────────┬──────────────┘
                       │
        ┌──────────────┴──────────────┐
        │                             │
┌───────┴────────┐            ┌───────┴────────┐
│   Worker 1     │            │   Worker 2     │
│  (4 threads)   │            │  (4 threads)   │
└───────┬────────┘            └───────┬────────┘
        │                             │
        └──────────────┬──────────────┘
                       │
        ┌──────────────┴──────────────┐
        │                             │
┌───────┴────────┐            ┌───────┴────────┐
│  PostgreSQL    │◄──────────►│  PostgreSQL    │
│   Primary      │  Replication│   Standby      │
└────────────────┘            └────────────────┘
```

---

## Security Architecture

### Multi-Tenant Isolation

```
Application Layer:
┌─────────────────────────────────────────┐
│  Tenant Context (from JWT/API Key)     │
│  ┌───────────────────────────────────┐ │
│  │  All queries filtered by tenant_id│ │
│  └───────────────────────────────────┘ │
└─────────────────────────────────────────┘
                    │
Database Layer:     ▼
┌─────────────────────────────────────────┐
│  Row-Level Security Policies            │
│  ┌───────────────────────────────────┐ │
│  │ WHERE tenant_id = current_tenant()│ │
│  └───────────────────────────────────┘ │
└─────────────────────────────────────────┘
```

### Authentication Flow

```
1. User → Login request → API
2. API → Validate credentials → Database
3. Database → Return user → API
4. API → Generate JWT token → User
5. User → Request with JWT → API
6. API → Validate & extract tenant → Continue
```

---

## Scalability and Performance

### Horizontal Scaling

**API Servers:**
- Stateless design allows unlimited horizontal scaling
- Load balancer distributes requests
- No session affinity required

**Workers:**
- Add workers to increase job throughput
- Each worker claims jobs independently
- No coordination overhead

### Performance Optimizations

1. **Connection Pooling:** Reuse database connections
2. **Registry Caching:** Cache package metadata (TTL: 1 hour)
3. **Batch Processing:** Process multiple dependencies in parallel
4. **Lazy Loading:** Load data only when needed
5. **Indexing:** Strategic database indexes for query performance

---

## Next Steps

- [Component Design](components.md) - Detailed component architecture
- [Data Flow](data-flow.md) - Detailed data flow diagrams
- [Database Schema](database.md) - Complete schema documentation
