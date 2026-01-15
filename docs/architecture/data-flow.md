# Data Flow Documentation

This document details the data flows through the Supply Chain Trust Verifier (SCTV) system, including request processing, scanning, job execution, notifications, and authentication.

## Table of Contents

1. [Request Flow](#request-flow)
2. [Scan Flow](#scan-flow)
3. [Job Processing Flow](#job-processing-flow)
4. [Notification Flow](#notification-flow)
5. [Authentication Flow](#authentication-flow)
6. [State Diagrams](#state-diagrams)
7. [Error Propagation](#error-propagation)

---

## Request Flow

### REST API Request Flow

```
┌──────────┐
│  Client  │
└────┬─────┘
     │ HTTP Request
     │ Authorization: Bearer <JWT>
     │
     ▼
┌─────────────────────────────────────────────────────────┐
│                    API Server (Axum)                    │
├─────────────────────────────────────────────────────────┤
│                                                         │
│  ┌──────────────────────────────────────────────────┐  │
│  │ 1. Middleware Pipeline                           │  │
│  ├──────────────────────────────────────────────────┤  │
│  │  - CORS                                          │  │
│  │  - Request Tracing (correlation ID)             │  │
│  │  - Compression                                   │  │
│  └──────────────────────────────────────────────────┘  │
│                       │                                 │
│                       ▼                                 │
│  ┌──────────────────────────────────────────────────┐  │
│  │ 2. Authentication Extractor                      │  │
│  ├──────────────────────────────────────────────────┤  │
│  │  - Extract Authorization header                  │  │
│  │  - Decode JWT token                              │  │
│  │  - Validate signature & expiration               │  │
│  │  - Extract Claims → AuthUser                     │  │
│  └──────────────────────────────────────────────────┘  │
│                       │                                 │
│                       ▼                                 │
│  ┌──────────────────────────────────────────────────┐  │
│  │ 3. Route Handler                                 │  │
│  ├──────────────────────────────────────────────────┤  │
│  │  GET /api/v1/projects                            │  │
│  │  ↓                                               │  │
│  │  handlers::list_projects(auth: AuthUser)         │  │
│  └──────────────────────────────────────────────────┘  │
│                       │                                 │
│                       ▼                                 │
│  ┌──────────────────────────────────────────────────┐  │
│  │ 4. Business Logic / Service Layer                │  │
│  ├──────────────────────────────────────────────────┤  │
│  │  - Validate tenant access                        │  │
│  │  - Fetch from repository                         │  │
│  │  - Apply business rules                          │  │
│  └──────────────────────────────────────────────────┘  │
│                       │                                 │
│                       ▼                                 │
│  ┌──────────────────────────────────────────────────┐  │
│  │ 5. Repository Layer                              │  │
│  ├──────────────────────────────────────────────────┤  │
│  │  project_repo.find_by_tenant(tenant_id)          │  │
│  └──────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────┘
                       │
                       ▼
┌─────────────────────────────────────────────────────────┐
│                   Database Layer                        │
├─────────────────────────────────────────────────────────┤
│  SELECT * FROM projects                                 │
│  WHERE tenant_id = $1                                   │
│  ORDER BY created_at DESC                               │
└─────────────────────────────────────────────────────────┘
                       │
                       ▼
                 ┌──────────┐
                 │PostgreSQL│
                 └──────────┘
                       │
                       │ Result Set
                       ▼
┌─────────────────────────────────────────────────────────┐
│                    API Server                           │
│  ┌──────────────────────────────────────────────────┐  │
│  │ 6. Response Serialization                        │  │
│  ├──────────────────────────────────────────────────┤  │
│  │  Vec<Project> → JSON                             │  │
│  │  + HTTP 200 OK                                   │  │
│  └──────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────┘
                       │
                       ▼
                  ┌──────────┐
                  │  Client  │
                  └──────────┘
```

### GraphQL Request Flow

```
Client
  │
  │ POST /graphql
  │ {
  │   "query": "query { projects { id name } }"
  │ }
  │
  ▼
┌─────────────────────────────────────────────┐
│         GraphQL Handler                     │
├─────────────────────────────────────────────┤
│ 1. Parse GraphQL query                      │
│ 2. Validate schema                          │
│ 3. Extract auth from headers                │
│ 4. Create GqlContext {                      │
│      state: AppState,                       │
│      tenant_id: Some(uuid),                 │
│      user_id: Some(uuid)                    │
│    }                                        │
└─────────────────────────────────────────────┘
  │
  ▼
┌─────────────────────────────────────────────┐
│         Query Resolver                      │
├─────────────────────────────────────────────┤
│ QueryRoot::projects(ctx, page, per_page)    │
│   ├─ Extract tenant_id from context         │
│   ├─ Get repository from state              │
│   ├─ Fetch projects from DB                 │
│   ├─ Apply pagination                       │
│   └─ Enrich with counts (deps, alerts)      │
└─────────────────────────────────────────────┘
  │
  ▼
Database (same as REST)
  │
  ▼
Response
{
  "data": {
    "projects": [
      {"id": "...", "name": "my-project"}
    ]
  }
}
```

### Sequence Diagram: API Request

```
Client          API Server      Auth Layer     Repository      Database
  │                 │               │              │               │
  ├─ POST ──────────>│               │              │               │
  │  /api/v1/alerts │               │              │               │
  │                 │               │              │               │
  │                 ├─ Extract ────>│              │               │
  │                 │   JWT         │              │               │
  │                 │               │              │               │
  │                 │<─ AuthUser ───┤              │               │
  │                 │   (tenant_id) │              │               │
  │                 │               │              │               │
  │                 ├─ find_with_filter ─────────>│               │
  │                 │   (tenant_id, filter)        │               │
  │                 │               │              │               │
  │                 │               │              ├─ SELECT ─────>│
  │                 │               │              │   alerts      │
  │                 │               │              │   WHERE...    │
  │                 │               │              │               │
  │                 │               │              │<─ Rows ───────┤
  │                 │               │              │               │
  │                 │<─ Vec<Alert> ─────────────────┤              │
  │                 │               │              │               │
  │<─ JSON Response ┤               │              │               │
  │   200 OK        │               │              │               │
  │                 │               │              │               │
```

---

## Scan Flow

### Project Scan Flow

```
┌──────────┐
│   User   │ Trigger scan
└────┬─────┘
     │
     │ POST /api/v1/projects/{id}/scan
     │
     ▼
┌─────────────────────────────────────────────────────────┐
│                  API Handler                            │
├─────────────────────────────────────────────────────────┤
│  1. Validate project exists                             │
│  2. Check tenant access                                 │
│  3. Create job payload                                  │
└─────────────────────────────────────────────────────────┘
     │
     │ Enqueue Job
     │
     ▼
┌─────────────────────────────────────────────────────────┐
│                  Job Queue                              │
├─────────────────────────────────────────────────────────┤
│  INSERT INTO jobs (                                     │
│    job_type = 'ScanProject',                            │
│    payload = '{"project_id": "...", "full_scan": true}',│
│    status = 'pending',                                  │
│    priority = 'normal'                                  │
│  )                                                      │
└─────────────────────────────────────────────────────────┘
     │
     │ Job ID returned
     │
     ▼
┌──────────┐
│   User   │ Receives job_id
└──────────┘

... later ...

┌─────────────────────────────────────────────────────────┐
│                Worker Pool                              │
├─────────────────────────────────────────────────────────┤
│  Worker Thread polling for jobs                         │
└─────────────────────────────────────────────────────────┘
     │
     │ claim_next(['ScanProject'])
     │
     ▼
┌─────────────────────────────────────────────────────────┐
│                Job Queue                                │
├─────────────────────────────────────────────────────────┤
│  SELECT * FROM jobs                                     │
│  WHERE status = 'pending'                               │
│    AND job_type = 'ScanProject'                         │
│  ORDER BY priority DESC, created_at ASC                 │
│  LIMIT 1                                                │
│  FOR UPDATE SKIP LOCKED;                                │
│                                                         │
│  UPDATE jobs SET                                        │
│    status = 'running',                                  │
│    started_at = NOW()                                   │
│  WHERE id = $1;                                         │
└─────────────────────────────────────────────────────────┘
     │
     │ Job claimed
     │
     ▼
┌─────────────────────────────────────────────────────────┐
│          ScanProjectExecutor                            │
├─────────────────────────────────────────────────────────┤
│                                                         │
│  ┌───────────────────────────────────────────────────┐ │
│  │ 1. Load Project                                   │ │
│  │    - Fetch from database                          │ │
│  │    - Get repository URL                           │ │
│  └───────────────────────────────────────────────────┘ │
│                       │                                 │
│                       ▼                                 │
│  ┌───────────────────────────────────────────────────┐ │
│  │ 2. Parse Dependency Manifest                      │ │
│  │    - package.json (npm)                           │ │
│  │    - requirements.txt (PyPI)                      │ │
│  │    - pom.xml (Maven)                              │ │
│  │    - Cargo.toml (Rust)                            │ │
│  └───────────────────────────────────────────────────┘ │
│                       │                                 │
│                       ▼                                 │
│  ┌───────────────────────────────────────────────────┐ │
│  │ 3. Resolve Dependencies                           │ │
│  │    - Fetch metadata from registries               │ │
│  │    - Build dependency tree                        │ │
│  │    - Mark direct vs transitive                    │ │
│  └───────────────────────────────────────────────────┘ │
│                       │                                 │
│                       ▼                                 │
│  ┌───────────────────────────────────────────────────┐ │
│  │ 4. Store Dependencies                             │ │
│  │    - Upsert to database                           │ │
│  │    - Track first_seen_at                          │ │
│  │    - Update last_verified_at                      │ │
│  └───────────────────────────────────────────────────┘ │
│                       │                                 │
│                       ▼                                 │
│  ┌───────────────────────────────────────────────────┐ │
│  │ 5. Run Detectors (Parallel)                       │ │
│  │    ┌─────────────────────────────────────────┐    │ │
│  │    │ Typosquatting Detector                  │    │ │
│  │    │  - Compare against popular packages     │    │ │
│  │    │  - Calculate similarity scores          │    │ │
│  │    │  - Detect keyboard typos                │    │ │
│  │    └─────────────────────────────────────────┘    │ │
│  │    ┌─────────────────────────────────────────┐    │ │
│  │    │ Tampering Detector                      │    │ │
│  │    │  - Download packages                    │    │ │
│  │    │  - Compute SHA-256                      │    │ │
│  │    │  - Compare against registry             │    │ │
│  │    └─────────────────────────────────────────┘    │ │
│  │    ┌─────────────────────────────────────────┐    │ │
│  │    │ Downgrade Detector                      │    │ │
│  │    │  - Compare with previous versions       │    │ │
│  │    │  - Check vulnerability databases        │    │ │
│  │    └─────────────────────────────────────────┘    │ │
│  │    ┌─────────────────────────────────────────┐    │ │
│  │    │ Provenance Detector                     │    │ │
│  │    │  - Fetch SLSA attestations              │    │ │
│  │    │  - Verify Sigstore signatures           │    │ │
│  │    │  - Validate build metadata              │    │ │
│  │    └─────────────────────────────────────────┘    │ │
│  └───────────────────────────────────────────────────┘ │
│                       │                                 │
│                       │ DetectionResults                │
│                       ▼                                 │
│  ┌───────────────────────────────────────────────────┐ │
│  │ 6. Create Alerts                                  │ │
│  │    - Convert detections to alerts                 │ │
│  │    - Apply severity rules                         │ │
│  │    - Check against policies                       │ │
│  │    - De-duplicate existing alerts                 │ │
│  │    - Store in database                            │ │
│  └───────────────────────────────────────────────────┘ │
│                       │                                 │
│                       ▼                                 │
│  ┌───────────────────────────────────────────────────┐ │
│  │ 7. Enqueue Notification Jobs                      │ │
│  │    For each high/critical alert:                  │ │
│  │      - Create SendNotification job                │ │
│  │      - Include alert context                      │ │
│  │      - Set to configured channels                 │ │
│  └───────────────────────────────────────────────────┘ │
│                       │                                 │
│                       ▼                                 │
│  ┌───────────────────────────────────────────────────┐ │
│  │ 8. Update Job Status                              │ │
│  │    - Mark job as 'completed'                      │ │
│  │    - Store result summary                         │ │
│  │    - Update project.last_scan_at                  │ │
│  └───────────────────────────────────────────────────┘ │
│                                                         │
└─────────────────────────────────────────────────────────┘
```

### Scan Sequence Diagram

```
Worker      Executor    DepRepo    Registry   Detectors   AlertRepo   JobQueue
  │            │           │           │           │           │           │
  ├─ Claim ───────────────────────────────────────────────────────────────>│
  │   Job      │           │           │           │           │           │
  │            │           │           │           │           │           │
  │<─ Job ──────────────────────────────────────────────────────────────────┤
  │  Payload   │           │           │           │           │           │
  │            │           │           │           │           │           │
  ├─ Execute ─>│           │           │           │           │           │
  │            │           │           │           │           │           │
  │            ├─ Parse ───┤           │           │           │           │
  │            │  manifest │           │           │           │           │
  │            │           │           │           │           │           │
  │            ├─ Fetch metadata ─────>│           │           │           │
  │            │           │           │           │           │           │
  │            │<─ Metadata ───────────┤           │           │           │
  │            │   + hashes│           │           │           │           │
  │            │           │           │           │           │           │
  │            ├─ Upsert ─>│           │           │           │           │
  │            │  deps     │           │           │           │           │
  │            │           │           │           │           │           │
  │            │<─ OK ─────┤           │           │           │           │
  │            │           │           │           │           │           │
  │            ├─ Analyze ─────────────────────────>│           │           │
  │            │  (parallel)           │           │           │           │
  │            │           │           │           │           │           │
  │            │<─ Detection results ──────────────┤           │           │
  │            │           │           │           │           │           │
  │            ├─ Create alerts ─────────────────────────────>│           │
  │            │           │           │           │           │           │
  │            │<─ Alert IDs ──────────────────────────────────┤           │
  │            │           │           │           │           │           │
  │            ├─ Complete job ────────────────────────────────────────────>│
  │            │  (result)  │           │           │           │           │
  │            │           │           │           │           │           │
  │<─ Success ─┤           │           │           │           │           │
  │            │           │           │           │           │           │
```

---

## Job Processing Flow

### Job Lifecycle

```
┌─────────────┐
│   PENDING   │ ◄── Enqueued
└──────┬──────┘
       │
       │ Worker claims job
       │ (SELECT FOR UPDATE SKIP LOCKED)
       │
       ▼
┌─────────────┐
│   RUNNING   │
└──────┬──────┘
       │
       ├───────────────┬─────────────┐
       │               │             │
       ▼               ▼             ▼
┌─────────────┐  ┌──────────┐  ┌──────────┐
│  COMPLETED  │  │  FAILED  │  │ CANCELLED│
└─────────────┘  └────┬─────┘  └──────────┘
                      │
                      │ attempt < max_attempts
                      │
                      ▼
                 ┌─────────────┐
                 │   PENDING   │ (retry)
                 └─────────────┘
```

### Job Queue Processing Loop

```
┌──────────────────────────────────────────────────┐
│              Worker Thread                       │
├──────────────────────────────────────────────────┤
│                                                  │
│  loop {                                          │
│    ┌──────────────────────────────────────┐     │
│    │ 1. Poll for jobs                     │     │
│    │    queue.claim_next(job_types)       │     │
│    └──────────────────────────────────────┘     │
│                   │                              │
│                   ▼                              │
│    ┌──────────────────────────────────────┐     │
│    │ Job available?                       │     │
│    └──────────────────────────────────────┘     │
│         │                      │                 │
│         │ Yes                  │ No              │
│         ▼                      ▼                 │
│    ┌────────────┐    ┌───────────────────┐      │
│    │ Claim job  │    │ Sleep poll_interval│     │
│    └────────────┘    │ (e.g., 5s)        │      │
│         │            └───────────────────┘      │
│         │                      │                 │
│         ▼                      │                 │
│    ┌──────────────────────────────────────┐     │
│    │ 2. Look up executor                  │     │
│    │    registry.get_executor(job.type)   │     │
│    └──────────────────────────────────────┘     │
│         │                                        │
│         ▼                                        │
│    ┌──────────────────────────────────────┐     │
│    │ 3. Execute with timeout              │     │
│    │    tokio::timeout(                   │     │
│    │      timeout_duration,               │     │
│    │      executor.execute(job, ctx)      │     │
│    │    )                                 │     │
│    └──────────────────────────────────────┘     │
│         │                                        │
│         ├──────────────┬──────────────┐          │
│         │              │              │          │
│         ▼              ▼              ▼          │
│    ┌────────┐    ┌─────────┐    ┌─────────┐    │
│    │Success │    │ Error   │    │ Timeout │    │
│    └────────┘    └─────────┘    └─────────┘    │
│         │              │              │          │
│         ▼              ▼              ▼          │
│    ┌──────────────────────────────────────┐     │
│    │ 4. Update job status                 │     │
│    │    - completed                       │     │
│    │    - failed (retry if < max_attempts)│     │
│    │    - failed (timeout)                │     │
│    └──────────────────────────────────────┘     │
│         │                                        │
│         └────────────────┐                       │
│                          │                       │
│  } // End loop           │                       │
│                          │                       │
│    ┌──────────────────────────────────────┐     │
│    │ 5. Check shutdown signal             │     │
│    │    If received, finish current job   │     │
│    │    and exit gracefully               │     │
│    └──────────────────────────────────────┘     │
│                                                  │
└──────────────────────────────────────────────────┘
```

### Job Retry Logic

```
Job Failure
    │
    ▼
┌────────────────────────┐
│ Check attempt count    │
└────────┬───────────────┘
         │
         ├─────────────┬─────────────┐
         │             │             │
  attempt < max   attempt >= max   cancelled
         │             │             │
         ▼             ▼             ▼
    ┌────────┐   ┌──────────┐  ┌──────────┐
    │ Retry  │   │ Permanent│  │  Delete  │
    │        │   │  Failure │  │   Job    │
    └────┬───┘   └──────────┘  └──────────┘
         │
         │ Increment attempt
         │ Calculate backoff
         │ (exponential: 2^attempt * base_delay)
         │
         ▼
    ┌─────────────────┐
    │ Update job:     │
    │ - status=pending│
    │ - attempt += 1  │
    │ - scheduled_at  │
    │   = NOW() + delay│
    └─────────────────┘
```

### Stale Job Recovery

```
┌──────────────────────────────────────┐
│   Maintenance Task (runs hourly)     │
├──────────────────────────────────────┤
│                                      │
│  SELECT * FROM jobs                  │
│  WHERE status = 'running'            │
│    AND started_at < NOW() - INTERVAL │
│        '{stale_timeout} minutes';    │
│                                      │
│  For each stale job:                 │
│    UPDATE jobs SET                   │
│      status = 'pending',             │
│      started_at = NULL,              │
│      error_message = 'Job recovered  │
│        from stale state'             │
│    WHERE id = $1;                    │
│                                      │
└──────────────────────────────────────┘
```

---

## Notification Flow

### Alert to Notification

```
┌─────────────────┐
│ Alert Created   │ (From scan or manual)
└────────┬────────┘
         │
         ▼
┌─────────────────────────────────────────┐
│ Check Alert Severity                    │
└────────┬────────────────────────────────┘
         │
         ├─────────────┬─────────────┐
         │             │             │
   Critical/High     Medium         Low/Info
         │             │             │
         ▼             ▼             ▼
    ┌────────┐   ┌─────────┐   ┌──────────┐
    │Immediate│  │Scheduled│   │  Batched │
    │        │   │ (1 hour)│   │  (daily) │
    └────┬───┘   └────┬────┘   └────┬─────┘
         │            │             │
         └────────────┴─────────────┘
                      │
                      ▼
         ┌────────────────────────────┐
         │ Enqueue Notification Job   │
         ├────────────────────────────┤
         │ JobType: SendNotification  │
         │ Payload: {                 │
         │   alert_id: ...,           │
         │   channels: [              │
         │     'email',               │
         │     'slack',               │
         │     'pagerduty'            │
         │   ],                       │
         │   context: {               │
         │     project_name: ...,     │
         │     dashboard_url: ...     │
         │   }                        │
         │ }                          │
         └────────────────────────────┘
                      │
                      ▼
         ┌────────────────────────────┐
         │     Job Queue              │
         └────────────────────────────┘

... Worker picks up job ...

         ┌────────────────────────────┐
         │ SendNotificationExecutor   │
         ├────────────────────────────┤
         │ 1. Load alert from DB      │
         │ 2. Load tenant config      │
         │ 3. Build notification      │
         │    context                 │
         └────────────────────────────┘
                      │
                      ▼
         ┌────────────────────────────┐
         │ NotificationService        │
         ├────────────────────────────┤
         │ send(notification)         │
         │                            │
         │ Parallel delivery:         │
         │  ├─ Email Channel          │
         │  ├─ Slack Channel          │
         │  └─ PagerDuty Channel      │
         └────────────────────────────┘
                      │
         ┌────────────┼────────────┐
         │            │            │
         ▼            ▼            ▼
    ┌────────┐  ┌────────┐  ┌─────────┐
    │ Email  │  │ Slack  │  │PagerDuty│
    │ SMTP   │  │Webhook │  │Events   │
    │        │  │        │  │API v2   │
    └────┬───┘  └────┬───┘  └────┬────┘
         │           │           │
         │ Results collected     │
         └───────────┴───────────┘
                     │
                     ▼
         ┌────────────────────────────┐
         │ Record Delivery Results    │
         ├────────────────────────────┤
         │ - Update alert metadata    │
         │ - Log successful deliveries│
         │ - Store failed channels    │
         │   for retry                │
         └────────────────────────────┘
```

### Notification Channel Flow

```
NotificationService
        │
        │ For each configured channel
        ▼
┌──────────────────────────────────────────────┐
│          Channel Selection                   │
├──────────────────────────────────────────────┤
│  if alert.severity >= channel.min_severity { │
│    send via channel                          │
│  }                                           │
└──────────────────────────────────────────────┘
        │
        ├─────────────────┬──────────────────┬─────────────────┐
        │                 │                  │                 │
        ▼                 ▼                  ▼                 ▼
   ┌─────────┐      ┌─────────┐       ┌─────────┐      ┌──────────┐
   │  Email  │      │  Slack  │       │  Teams  │      │PagerDuty │
   └────┬────┘      └────┬────┘       └────┬────┘      └────┬─────┘
        │                │                  │                │
        ▼                ▼                  ▼                ▼
   ┌─────────────┐ ┌──────────────┐ ┌──────────────┐ ┌─────────────┐
   │Build SMTP   │ │Build Block   │ │Build Adaptive│ │Build Event  │
   │message with │ │Kit message   │ │Card          │ │payload      │
   │HTML + plain │ │with rich     │ │              │ │             │
   │             │ │formatting    │ │              │ │             │
   └──────┬──────┘ └──────┬───────┘ └──────┬───────┘ └──────┬──────┘
          │                │                │                │
          ▼                ▼                ▼                ▼
   ┌─────────────┐ ┌──────────────┐ ┌──────────────┐ ┌─────────────┐
   │Send via     │ │POST to       │ │POST to       │ │POST to      │
   │SMTP server  │ │webhook URL   │ │webhook URL   │ │events API   │
   └──────┬──────┘ └──────┬───────┘ └──────┬───────┘ └──────┬──────┘
          │                │                │                │
          └────────────────┴────────────────┴────────────────┘
                                   │
                                   ▼
                        ┌────────────────────┐
                        │  Collect Results   │
                        │  - Success count   │
                        │  - Failed channels │
                        │  - Error messages  │
                        └────────────────────┘
```

---

## Authentication Flow

### JWT Authentication Flow

```
┌──────────┐
│  Client  │
└────┬─────┘
     │
     │ 1. Login Request
     │ POST /api/v1/auth/login
     │ {
     │   "email": "user@example.com",
     │   "password": "********"
     │ }
     │
     ▼
┌─────────────────────────────────────────────┐
│           Auth Handler                      │
├─────────────────────────────────────────────┤
│ 1. Validate credentials                     │
│    - Hash password                          │
│    - Compare with stored hash               │
│                                             │
│ 2. Load user from database                  │
│    - Get user_id, tenant_id, roles          │
│                                             │
│ 3. Generate JWT                             │
│    Claims {                                 │
│      sub: user_id,                          │
│      tenant_id: tenant_id,                  │
│      email: "user@example.com",             │
│      roles: ["user"],                       │
│      iat: now(),                            │
│      exp: now() + 24h,                      │
│      iss: "sctv-api",                       │
│      aud: "sctv"                            │
│    }                                        │
│                                             │
│ 4. Sign with secret                         │
│    HMAC-SHA256(header.payload, secret)      │
└─────────────────────────────────────────────┘
     │
     │ 2. Response with JWT
     │ {
     │   "token": "eyJhbGc...",
     │   "expires_in": 86400
     │ }
     │
     ▼
┌──────────┐
│  Client  │ Stores token
└────┬─────┘
     │
     │ 3. Subsequent Request
     │ GET /api/v1/projects
     │ Authorization: Bearer eyJhbGc...
     │
     ▼
┌─────────────────────────────────────────────┐
│        Authentication Extractor             │
├─────────────────────────────────────────────┤
│ 1. Extract Authorization header             │
│    - Check for "Bearer " prefix             │
│                                             │
│ 2. Decode JWT                               │
│    - Parse header, payload, signature       │
│                                             │
│ 3. Verify signature                         │
│    - Compute expected signature             │
│    - Compare with token signature           │
│                                             │
│ 4. Validate claims                          │
│    - Check exp (expiration)                 │
│    - Check iss (issuer)                     │
│    - Check aud (audience)                   │
│                                             │
│ 5. Extract AuthUser                         │
│    AuthUser {                               │
│      user_id: claims.sub,                   │
│      tenant_id: claims.tenant_id,           │
│      email: claims.email,                   │
│      roles: claims.roles                    │
│    }                                        │
└─────────────────────────────────────────────┘
     │
     │ AuthUser injected into handler
     │
     ▼
┌─────────────────────────────────────────────┐
│            API Handler                      │
├─────────────────────────────────────────────┤
│ async fn list_projects(                     │
│   auth: AuthUser,  // ◄── Auto-extracted   │
│   State(state): State<AppState>             │
│ ) -> Result<Json<Vec<Project>>>             │
│                                             │
│ Use auth.tenant_id for data isolation       │
└─────────────────────────────────────────────┘
```

### API Key Authentication Flow

```
┌──────────┐
│  Client  │
│(Service) │
└────┬─────┘
     │
     │ 1. Request with API Key
     │ GET /api/v1/projects
     │ X-API-Key: sctv_abc123def456...
     │
     ▼
┌─────────────────────────────────────────────┐
│       API Key Authentication                │
├─────────────────────────────────────────────┤
│ 1. Extract X-API-Key header                 │
│                                             │
│ 2. Hash the key                             │
│    SHA-256(api_key)                         │
│                                             │
│ 3. Look up in database                      │
│    SELECT * FROM api_keys                   │
│    WHERE key_hash = $1                      │
│      AND is_active = true                   │
│      AND expires_at > NOW()                 │
│                                             │
│ 4. Verify scopes                            │
│    Check if requested operation             │
│    is allowed by key scopes                 │
│                                             │
│ 5. Extract ApiKeyAuth                       │
│    ApiKeyAuth {                             │
│      key_id: key.id,                        │
│      tenant_id: key.tenant_id,              │
│      scopes: key.scopes                     │
│    }                                        │
└─────────────────────────────────────────────┘
     │
     │ ApiKeyAuth injected
     │
     ▼
┌─────────────────────────────────────────────┐
│            API Handler                      │
└─────────────────────────────────────────────┘
```

---

## State Diagrams

### Alert State Machine

```
┌──────────┐
│   NEW    │ ◄── Alert created by detector
└────┬─────┘
     │
     │ Auto-transition on create
     │
     ▼
┌──────────┐
│   OPEN   │
└────┬─────┘
     │
     ├────────────┬────────────┬──────────────┐
     │            │            │              │
     │ User       │ User       │ User         │ System
     │ action     │ action     │ action       │ action
     │            │            │              │
     ▼            ▼            ▼              ▼
┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐
│ACKNOWLEDGED│ │ RESOLVED │ │SUPPRESSED│ │  EXPIRED │
└────┬─────┘ └────┬─────┘ └──────────┘ └──────────┘
     │            │
     │ User       │
     │ action     │
     │            │
     ▼            │
┌──────────┐      │
│ RESOLVED │ ◄────┘
└──────────┘

Transitions:
  OPEN → ACKNOWLEDGED  (User clicks "Acknowledge")
  OPEN → RESOLVED      (User provides remediation)
  OPEN → SUPPRESSED    (User suppresses false positive)
  OPEN → EXPIRED       (Alert older than retention period)
  ACKNOWLEDGED → RESOLVED (User completes remediation)
```

### Job State Machine

```
┌───────────┐
│ SCHEDULED │ ◄── Job enqueued for future execution
└─────┬─────┘
      │
      │ scheduled_at <= NOW()
      │
      ▼
┌───────────┐
│  PENDING  │ ◄── Job ready for execution
└─────┬─────┘
      │
      │ Worker claims job
      │
      ▼
┌───────────┐
│  RUNNING  │
└─────┬─────┘
      │
      ├──────────────┬──────────────┬──────────────┐
      │              │              │              │
  Success        Error          Timeout        Cancelled
      │              │              │              │
      ▼              ▼              ▼              ▼
┌───────────┐  ┌──────────┐  ┌──────────┐  ┌───────────┐
│ COMPLETED │  │  FAILED  │  │  FAILED  │  │ CANCELLED │
└───────────┘  └─────┬────┘  └─────┬────┘  └───────────┘
                     │              │
                     │ Retry logic  │
                     │              │
                     ▼              ▼
              ┌─────────────────────────┐
              │ attempt < max_attempts? │
              └───────┬──────────┬──────┘
                      │          │
                   Yes│          │No
                      │          │
                      ▼          ▼
               ┌───────────┐  ┌──────────────┐
               │  PENDING  │  │FAILED (final)│
               │  (retry)  │  └──────────────┘
               └───────────┘

State Transitions:
  SCHEDULED → PENDING    (Time-based)
  PENDING   → RUNNING    (Worker claims)
  RUNNING   → COMPLETED  (Success)
  RUNNING   → FAILED     (Error/Timeout)
  RUNNING   → CANCELLED  (User/System cancellation)
  FAILED    → PENDING    (Retry if attempts remaining)
```

### Dependency State Machine

```
┌────────────┐
│   NEW      │ ◄── First time seen in project
└──────┬─────┘
       │
       │ Initial scan complete
       │
       ▼
┌────────────┐
│  VERIFIED  │ ◄── Hashes match, no threats detected
└──────┬─────┘
       │
       ├─────────────┬────────────┬──────────────┐
       │             │            │              │
   Re-scan       Tampering    Downgrade     Version
   (periodic)    detected     detected      updated
       │             │            │              │
       ▼             ▼            ▼              ▼
┌────────────┐ ┌───────────┐ ┌──────────┐ ┌────────────┐
│  VERIFIED  │ │ SUSPICIOUS│ │OUTDATED  │ │  UPDATED   │
└────────────┘ └───────────┘ └──────────┘ └──────┬─────┘
       ▲             │            │              │
       │             │            │              │
       │      Alert created  Alert created       │
       │             │            │              │
       └─────────────┴────────────┴──────────────┘
                Re-verification
```

---

## Error Propagation

### Error Flow Layers

```
┌─────────────────────────────────────────────────────┐
│               Application Layer                     │
│  Handler returns Result<Json<T>, ApiError>          │
└──────────────────┬──────────────────────────────────┘
                   │
                   │ Error occurs
                   │
                   ▼
┌─────────────────────────────────────────────────────┐
│              Service Layer                          │
│  Service methods return Result<T, ServiceError>     │
└──────────────────┬──────────────────────────────────┘
                   │
                   │ Database error
                   │
                   ▼
┌─────────────────────────────────────────────────────┐
│            Repository Layer                         │
│  Repo methods return Result<T, RepositoryError>     │
└──────────────────┬──────────────────────────────────┘
                   │
                   │ SQL error
                   │
                   ▼
┌─────────────────────────────────────────────────────┐
│             Database Layer                          │
│  sqlx::query!() returns sqlx::Error                 │
└─────────────────────────────────────────────────────┘
```

### Error Conversion Chain

```rust
// Database error
sqlx::Error::RowNotFound
    │
    │ .map_err(RepositoryError::Database)
    │
    ▼
RepositoryError::NotFound
    │
    │ .map_err(ServiceError::NotFound)
    │
    ▼
ServiceError::NotFound
    │
    │ .map_err(ApiError::from)
    │
    ▼
ApiError::NotFound
    │
    │ IntoResponse implementation
    │
    ▼
HTTP Response:
{
  "error": "Not found",
  "details": "Project with id ... not found",
  "status": 404
}
```

### Error Response Format

```
Success Path:
Client ──[Request]──> API ──[Process]──> DB ──[Data]──> API ──[JSON]──> Client
                                                                         200 OK

Error Paths:

1. Authentication Error:
   Client ──[No Auth]──> API ──[Validate]──X
                                           │
                                           └──> 401 Unauthorized
                                                {
                                                  "error": "Unauthorized",
                                                  "message": "Missing authentication"
                                                }

2. Authorization Error:
   Client ──[Auth: TenantA]──> API ──[Check Access]──X (Project in TenantB)
                                                      │
                                                      └──> 403 Forbidden
                                                           {
                                                             "error": "Forbidden",
                                                             "message": "Access denied"
                                                           }

3. Validation Error:
   Client ──[Invalid Data]──> API ──[Validate]──X
                                                 │
                                                 └──> 400 Bad Request
                                                      {
                                                        "error": "Validation failed",
                                                        "fields": {
                                                          "name": "Required"
                                                        }
                                                      }

4. Not Found Error:
   Client ──[Request ID]──> API ──[Query DB]──> DB ──[Empty]──X
                                                               │
                                                               └──> 404 Not Found
                                                                    {
                                                                      "error": "Not found",
                                                                      "message": "Project not found"
                                                                    }

5. Database Error:
   Client ──[Request]──> API ──[Query DB]──> DB ──[Connection Error]──X
                                                                       │
                                                                       └──> 500 Internal Server Error
                                                                            {
                                                                              "error": "Internal server error",
                                                                              "message": "Database unavailable",
                                                                              "request_id": "abc-123"
                                                                            }
```

### Error Context Enrichment

```
Error occurs at:
  File: src/db/repositories/project_repo.rs:42
  Function: PgProjectRepository::find_by_id()

Error propagation with context:

sqlx::Error::RowNotFound
  ↓
.context("Failed to find project")
  ↓
RepositoryError::NotFound {
  entity: "Project",
  id: "123e4567-e89b-12d3-a456-426614174000",
  context: "Failed to find project"
}
  ↓
.context(format!("Project lookup failed for ID {}", id))
  ↓
ServiceError::NotFound {
  message: "Project lookup failed for ID 123e4567-e89b-12d3-a456-426614174000",
  source: RepositoryError::NotFound { ... }
}
  ↓
Into<ApiError>
  ↓
ApiError::NotFound {
  message: "Project not found",
  details: Some("Project lookup failed for ID ..."),
  request_id: "req_abc123"
}
  ↓
IntoResponse
  ↓
HTTP 404 with JSON body

Logged as:
ERROR [sctv_api::handlers::projects] Project not found
  entity=Project
  id=123e4567-e89b-12d3-a456-426614174000
  request_id=req_abc123
  tenant_id=tenant_xyz
  user_id=user_abc
```

---

## Data Consistency

### Transaction Boundaries

```
Scan Job Transaction:
┌──────────────────────────────────────┐
│ BEGIN TRANSACTION                    │
├──────────────────────────────────────┤
│                                      │
│ 1. Update job status to 'running'   │
│    UPDATE jobs SET status = 'running'│
│                                      │
│ COMMIT                               │
└──────────────────────────────────────┘
        │
        │ (Job processing - outside transaction)
        │
        ▼
┌──────────────────────────────────────┐
│ BEGIN TRANSACTION                    │
├──────────────────────────────────────┤
│                                      │
│ 1. Upsert dependencies               │
│    ON CONFLICT UPDATE                │
│                                      │
│ 2. Create alerts                     │
│    INSERT INTO alerts                │
│                                      │
│ 3. Update project.last_scan_at       │
│    UPDATE projects SET...            │
│                                      │
│ 4. Update job status to 'completed'  │
│    UPDATE jobs SET status = 'completed'
│                                      │
│ COMMIT (All or nothing)              │
└──────────────────────────────────────┘
```

### Idempotency

```
Alert Creation (Idempotent):

Key: (tenant_id, project_id, alert_type, dependency_id)

INSERT INTO alerts (...)
VALUES (...)
ON CONFLICT (tenant_id, project_id, alert_type, dependency_id)
DO UPDATE SET
  updated_at = EXCLUDED.updated_at,
  metadata = EXCLUDED.metadata
WHERE alerts.status = 'open';

Result: Same alert not duplicated on re-scan
```

This completes the comprehensive data flow documentation for the SCTV project.
