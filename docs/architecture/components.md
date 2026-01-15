# Component Architecture

This document provides detailed documentation for all components in the Supply Chain Trust Verifier (SCTV) system.

## Table of Contents

1. [sctv-core](#sctv-core) - Domain Models and Business Logic
2. [sctv-api](#sctv-api) - REST and GraphQL API
3. [sctv-db](#sctv-db) - Database Layer
4. [sctv-detectors](#sctv-detectors) - Threat Detection Engines
5. [sctv-registries](#sctv-registries) - Package Registry Clients
6. [sctv-sbom](#sctv-sbom) - SBOM Generation
7. [sctv-worker](#sctv-worker) - Background Job Processing
8. [sctv-notifications](#sctv-notifications) - Alert Notifications
9. [sctv-ci](#sctv-ci) - CI/CD Integration
10. [sctv-cli](#sctv-cli) - Command-Line Interface
11. [sctv-dashboard](#sctv-dashboard) - Web Dashboard

---

## sctv-core

**Location**: `crates/sctv-core`

**Purpose**: Core domain models, business logic, and trait definitions used throughout the system.

### Domain Models

#### Project
```rust
pub struct Project {
    pub id: ProjectId,
    pub tenant_id: TenantId,
    pub name: String,
    pub description: Option<String>,
    pub repository_url: Option<Url>,
    pub status: ProjectStatus,
    pub scan_schedule: Option<ScanSchedule>,
    pub last_scan_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub metadata: ProjectMetadata,
}
```

**Key Features**:
- Multi-tenant isolation via `tenant_id`
- Scheduled scanning support
- Extensible metadata storage
- Repository tracking

#### Dependency
```rust
pub struct Dependency {
    pub id: DependencyId,
    pub project_id: ProjectId,
    pub package_name: String,
    pub ecosystem: PackageEcosystem,
    pub version_constraint: String,
    pub resolved_version: Version,
    pub is_direct: bool,
    pub is_dev_dependency: bool,
    pub depth: usize,
    pub integrity: DependencyIntegrity,
    pub first_seen_at: DateTime<Utc>,
    pub last_verified_at: DateTime<Utc>,
}

pub struct DependencyIntegrity {
    pub hash_sha256: Option<String>,
    pub hash_sha512: Option<String>,
    pub signature_status: SignatureStatus,
    pub provenance_status: ProvenanceStatus,
    pub provenance_details: Option<ProvenanceDetails>,
}
```

**Key Features**:
- Multi-ecosystem support (npm, PyPI, Maven, NuGet, RubyGems, Cargo, Go)
- Dependency tree tracking with depth
- Cryptographic hash verification
- SLSA provenance tracking
- Signature verification support

#### Alert
```rust
pub struct Alert {
    pub id: AlertId,
    pub tenant_id: TenantId,
    pub project_id: ProjectId,
    pub alert_type: AlertType,
    pub severity: Severity,
    pub title: String,
    pub description: String,
    pub status: AlertStatus,
    pub remediation: Option<Remediation>,
    pub metadata: AlertMetadata,
    pub created_at: DateTime<Utc>,
    pub acknowledged_at: Option<DateTime<Utc>>,
    pub resolved_at: Option<DateTime<Utc>>,
}

pub enum AlertType {
    DependencyTampering(TamperingDetails),
    DowngradeAttack(DowngradeDetails),
    Typosquatting(TyposquattingDetails),
    ProvenanceFailure(ProvenanceFailureDetails),
    PolicyViolation(PolicyViolationDetails),
    NewPackage(NewPackageDetails),
    SuspiciousMaintainer(MaintainerDetails),
}
```

**Key Features**:
- Type-safe alert variants with specific details
- Workflow tracking (Open → Acknowledged → Resolved/Suppressed)
- Severity levels: Info, Low, Medium, High, Critical
- Remediation tracking

#### Policy
```rust
pub struct Policy {
    pub id: PolicyId,
    pub tenant_id: TenantId,
    pub name: String,
    pub description: Option<String>,
    pub rules: Vec<PolicyRule>,
    pub severity_overrides: Vec<SeverityOverride>,
    pub is_default: bool,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub enum PolicyRule {
    RequireProvenance { min_level: u8 },
    RequireSignature,
    BlockPackages { patterns: Vec<PackagePattern> },
    AllowPackages { patterns: Vec<PackagePattern> },
    RequireVersionPinning { strategy: VersionPinningStrategy },
    MaxDependencyAge { days: u32 },
    ProhibitPrerelease,
    RequireLicense { allowed: Vec<String> },
}
```

**Key Features**:
- Flexible rule engine
- Package pattern matching
- License compliance enforcement
- Version pinning strategies
- Provenance requirements

#### Job
```rust
pub struct Job {
    pub id: JobId,
    pub job_type: JobType,
    pub tenant_id: Option<TenantId>,
    pub status: JobStatus,
    pub priority: JobPriority,
    pub payload: serde_json::Value,
    pub result: Option<serde_json::Value>,
    pub error_message: Option<String>,
    pub attempt: u32,
    pub max_attempts: u32,
    pub created_at: DateTime<Utc>,
    pub scheduled_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
}

pub enum JobType {
    ScanProject,
    MonitorRegistry,
    VerifyProvenance,
    SendNotification,
    GenerateSbom,
    CleanupOldData,
}
```

**Key Features**:
- PostgreSQL-backed job queue
- Priority-based execution
- Automatic retry with configurable max attempts
- Job lifecycle tracking

### Core Traits

#### Repository Traits
```rust
pub trait ProjectRepository: Send + Sync {
    async fn create(&self, project: &Project) -> Result<()>;
    async fn find_by_id(&self, id: ProjectId) -> Result<Option<Project>>;
    async fn find_by_tenant(&self, tenant_id: TenantId) -> Result<Vec<Project>>;
    async fn update(&self, project: &Project) -> Result<()>;
    async fn delete(&self, id: ProjectId) -> Result<()>;
}

pub trait AlertRepository: Send + Sync {
    async fn create(&self, alert: &Alert) -> Result<()>;
    async fn find_by_id(&self, id: AlertId) -> Result<Option<Alert>>;
    async fn find_with_filter(&self, tenant_id: TenantId, filter: AlertFilter,
                              limit: u32, offset: u32) -> Result<Vec<Alert>>;
    async fn update(&self, alert: &Alert) -> Result<()>;
    async fn count_open_by_project(&self, project_id: ProjectId) -> Result<u64>;
}
```

**Key Features**:
- Trait-based abstraction for testability
- Async/await support
- Multi-tenant filtering
- Pagination support

#### Service Traits
```rust
pub trait ScanService: Send + Sync {
    async fn scan_project(&self, project_id: ProjectId) -> Result<ScanResult>;
}

pub trait DetectorService: Send + Sync {
    async fn detect(&self, dependency: &Dependency) -> Result<Vec<DetectionResult>>;
}

pub trait RegistryService: Send + Sync {
    async fn fetch_metadata(&self, name: &str, version: &str) -> Result<PackageMetadata>;
    async fn download_package(&self, name: &str, version: &str) -> Result<Bytes>;
}
```

### Enums and Types

```rust
pub enum PackageEcosystem {
    Npm,
    PyPi,
    Maven,
    NuGet,
    RubyGems,
    Cargo,
    GoModules,
}

pub enum Severity {
    Info,
    Low,
    Medium,
    High,
    Critical,
}

pub enum SignatureStatus {
    NotSigned,
    Verified,
    Invalid,
    Unknown,
}

pub enum ProvenanceStatus {
    None,
    SlsaLevel0,
    SlsaLevel1,
    SlsaLevel2,
    SlsaLevel3,
    Invalid,
}
```

---

## sctv-api

**Location**: `crates/sctv-api`

**Purpose**: HTTP API layer providing REST endpoints and GraphQL schema for the SCTV platform.

### Architecture

```
┌──────────────────────────────────────┐
│         Axum HTTP Server             │
├──────────────────────────────────────┤
│  Middleware Layer                    │
│  - CORS                              │
│  - Compression                       │
│  - Tracing                           │
│  - Authentication (JWT/API Key)      │
├──────────────────────────────────────┤
│  REST API v1          GraphQL API    │
│  /api/v1/...          /graphql       │
└──────────────────────────────────────┘
```

### REST API Endpoints

```
Projects:
  GET    /api/v1/projects
  POST   /api/v1/projects
  GET    /api/v1/projects/{id}
  PUT    /api/v1/projects/{id}
  DELETE /api/v1/projects/{id}
  POST   /api/v1/projects/{id}/scan
  GET    /api/v1/projects/{id}/dependencies

Alerts:
  GET    /api/v1/alerts
  GET    /api/v1/alerts/{id}
  POST   /api/v1/alerts/{id}/acknowledge
  POST   /api/v1/alerts/{id}/resolve
  POST   /api/v1/alerts/{id}/suppress

Policies:
  GET    /api/v1/policies
  POST   /api/v1/policies
  GET    /api/v1/policies/{id}
  PUT    /api/v1/policies/{id}
  DELETE /api/v1/policies/{id}

Dependencies:
  GET    /api/v1/dependencies/{id}
  POST   /api/v1/dependencies/{id}/verify

Webhooks:
  POST   /api/v1/webhooks/github
  POST   /api/v1/webhooks/gitlab
```

### GraphQL Schema

```graphql
type Query {
  projects(page: Int = 1, perPage: Int = 20): [Project!]!
  project(id: ID!): Project
  alerts(projectId: ID, severity: Severity, status: AlertStatus,
         page: Int = 1, perPage: Int = 20): [Alert!]!
  alert(id: ID!): Alert
  dependencies(projectId: ID!, ecosystem: PackageEcosystem,
               isDirect: Boolean, page: Int = 1, perPage: Int = 50): [Dependency!]!
  policies: [Policy!]!
}

type Mutation {
  createProject(input: CreateProjectInput!): Project!
  updateProject(id: ID!, input: UpdateProjectInput!): Project
  deleteProject(id: ID!): Boolean!
  triggerScan(projectId: ID!, fullScan: Boolean): Scan!
  acknowledgeAlert(id: ID!, notes: String): Alert
  resolveAlert(id: ID!, actionTaken: String!, newVersion: String): Alert
  createPolicy(input: CreatePolicyInput!): Policy!
}
```

### Authentication

#### JWT Authentication
```rust
pub struct Claims {
    pub sub: Uuid,              // User ID
    pub tenant_id: Uuid,        // Tenant ID
    pub email: String,
    pub roles: Vec<String>,
    pub iat: i64,               // Issued at
    pub exp: i64,               // Expiration
    pub iss: String,            // Issuer
    pub aud: String,            // Audience
}
```

**Flow**:
1. Client authenticates with credentials
2. Server issues JWT with claims
3. Client includes JWT in `Authorization: Bearer <token>` header
4. Server validates signature and expiration
5. Extracts `AuthUser` from claims

#### API Key Authentication
- Alternative to JWT for service-to-service communication
- Passed via `X-API-Key` header
- Scoped permissions per key

### State Management

```rust
pub struct AppState {
    pub jwt_secret: String,
    pub jwt_issuer: String,
    pub jwt_audience: String,
    pub db_pool: Option<sqlx::PgPool>,
    pub repositories: Option<Repositories>,
}

pub struct Repositories {
    pub projects: Arc<dyn ProjectRepository>,
    pub alerts: Arc<dyn AlertRepository>,
    pub dependencies: Arc<dyn DependencyRepository>,
    pub policies: Arc<dyn PolicyRepository>,
}
```

**Features**:
- Shared state via Arc for thread-safety
- Trait object repositories for testability
- Graceful degradation when DB unavailable

---

## sctv-db

**Location**: `crates/sctv-db`

**Purpose**: Database abstraction layer with PostgreSQL implementations of repository traits.

### Architecture Pattern

```
Repository Pattern:
┌─────────────────────┐
│   Repository Trait  │  (in sctv-core)
└─────────────────────┘
          ↑
          │ implements
          │
┌─────────────────────┐
│ PgXxxRepository     │  (in sctv-db)
│ - pool: PgPool      │
└─────────────────────┘
          ↓
┌─────────────────────┐
│   PostgreSQL DB     │
└─────────────────────┘
```

### Repository Implementations

#### PgProjectRepository
```rust
pub struct PgProjectRepository {
    pool: PgPool,
}

impl ProjectRepository for PgProjectRepository {
    async fn create(&self, project: &Project) -> Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO projects (id, tenant_id, name, description, repository_url,
                                  status, scan_schedule, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            "#,
            project.id.0,
            project.tenant_id.0,
            project.name,
            project.description,
            project.repository_url.as_ref().map(|u| u.as_str()),
            project.status as _,
            project.scan_schedule as _,
            project.created_at,
            project.updated_at
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    // ... other methods
}
```

### Multi-Tenant Isolation

All queries enforce tenant isolation:

```sql
SELECT * FROM projects
WHERE tenant_id = $1 AND id = $2;

SELECT * FROM alerts
WHERE tenant_id = $1
  AND project_id IN (SELECT id FROM projects WHERE tenant_id = $1);
```

### Database Schema

```
┌─────────────┐     ┌──────────────┐     ┌──────────────┐
│   tenants   │────<│   projects   │────<│ dependencies │
└─────────────┘     └──────────────┘     └──────────────┘
                            │
                            v
                    ┌──────────────┐
                    │    alerts    │
                    └──────────────┘

┌─────────────┐     ┌──────────────┐
│    users    │     │   policies   │
└─────────────┘     └──────────────┘
       │
       v
┌─────────────┐     ┌──────────────┐
│ audit_logs  │     │     jobs     │
└─────────────┘     └──────────────┘
```

### Migration System

Uses SQLx migrations:

```
migrations/
├── 20240101000000_create_tenants.sql
├── 20240101000001_create_users.sql
├── 20240101000002_create_projects.sql
├── 20240101000003_create_dependencies.sql
├── 20240101000004_create_alerts.sql
├── 20240101000005_create_policies.sql
├── 20240101000006_create_jobs.sql
└── 20240101000007_create_audit_logs.sql
```

### Connection Pooling

```rust
pub async fn create_pool(database_url: &str) -> Result<PgPool> {
    PgPoolOptions::new()
        .max_connections(20)
        .min_connections(5)
        .acquire_timeout(Duration::from_secs(5))
        .connect(database_url)
        .await
}
```

---

## sctv-detectors

**Location**: `crates/sctv-detectors`

**Purpose**: Threat detection engines for identifying supply chain attacks.

### Detector Types

#### 1. Typosquatting Detector
```rust
pub struct TyposquattingDetector {
    popular_packages: Arc<DashMap<PackageEcosystem, HashSet<String>>>,
    similarity_threshold: f64,
}
```

**Detection Methods**:
- **Levenshtein Distance**: Character-level edit distance
- **Damerau-Levenshtein**: Includes transpositions
- **Keyboard Distance**: Adjacent key typos
- **Homograph Attack**: Lookalike characters (0/O, l/I)
- **Combosquatting**: Package name prefix/suffix additions

**Example**:
```
Suspicious: "lodash-utils" (Popular: "lodash")
Method: Combosquatting
Confidence: 0.92
```

#### 2. Tampering Detector
```rust
pub struct TamperingDetector {
    hash_algorithm: HashAlgorithm,
}
```

**Detection Logic**:
1. Fetch package from registry
2. Download package tarball
3. Compute SHA-256/SHA-512 hash
4. Compare against registry-published hash
5. Flag mismatches as tampering

**Example**:
```
Package: express@4.18.2
Expected: abc123...
Actual:   def456...
Result: TAMPERING DETECTED
```

#### 3. Downgrade Detector
```rust
pub struct DowngradeDetector {
    version_history: Arc<DashMap<String, Vec<Version>>>,
}
```

**Detection Logic**:
1. Track resolved versions over time
2. Compare new resolution against previous
3. Flag semantic version decreases
4. Consider vulnerability database

**Example**:
```
Previous: lodash@4.17.21
Current:  lodash@4.17.19
Severity: HIGH (known CVE in 4.17.19)
```

#### 4. Provenance Detector
```rust
pub struct ProvenanceDetector {
    sigstore_client: SigstoreClient,
    slsa_verifier: SlsaVerifier,
}
```

**Verification Steps**:
1. Fetch SLSA provenance attestation
2. Verify signature against Sigstore/Rekor
3. Validate builder identity
4. Check build reproducibility
5. Assign SLSA level (0-3)

**Example**:
```
Package: react@18.2.0
Attestation: Found
Signature: Valid (Sigstore)
Builder: GitHub Actions
SLSA Level: 3
```

### Detector Interface

```rust
#[async_trait]
pub trait Detector: Send + Sync {
    fn detector_type(&self) -> &'static str;

    async fn analyze(&self, dependency: &Dependency)
        -> DetectorResult<Vec<DetectionResult>>;

    fn create_alerts(&self, dependency: &Dependency,
                     results: &[DetectionResult]) -> Vec<Alert>;
}
```

---

## sctv-registries

**Location**: `crates/sctv-registries`

**Purpose**: Package registry clients for fetching metadata and packages.

### Supported Registries

#### 1. npm Registry
```rust
pub struct NpmClient {
    base_url: String,
    http_client: reqwest::Client,
    cache: Arc<RegistryCache>,
}
```

**API Endpoints**:
- Metadata: `GET https://registry.npmjs.org/{package}`
- Tarball: `GET https://registry.npmjs.org/{package}/-/{package}-{version}.tgz`
- Package Info: Includes dist.tarball, dist.shasum, maintainers

#### 2. PyPI Registry
```rust
pub struct PyPiClient {
    base_url: String,
    http_client: reqwest::Client,
}
```

**API Endpoints**:
- JSON API: `GET https://pypi.org/pypi/{package}/json`
- Release Info: `GET https://pypi.org/pypi/{package}/{version}/json`
- Wheel/Tarball downloads with SHA-256 hashes

#### 3. Maven Central
```rust
pub struct MavenClient {
    base_url: String,
    http_client: reqwest::Client,
}
```

**API Endpoints**:
- Metadata: `GET https://repo1.maven.org/maven2/{group}/{artifact}/maven-metadata.xml`
- Artifact: `GET https://repo1.maven.org/maven2/{group}/{artifact}/{version}/{artifact}-{version}.jar`
- Checksums: `.sha1`, `.sha256`, `.md5` files

#### 4. NuGet
```rust
pub struct NuGetClient {
    service_index_url: String,
    http_client: reqwest::Client,
}
```

**API v3**:
- Service Index: `GET https://api.nuget.org/v3/index.json`
- Package Metadata: Service resource lookup
- Package Download: `.nupkg` files with SHA-512

#### 5. RubyGems
```rust
pub struct RubyGemsClient {
    base_url: String,
    http_client: reqwest::Client,
}
```

**API Endpoints**:
- Gem Info: `GET https://rubygems.org/api/v1/gems/{gem}.json`
- Versions: `GET https://rubygems.org/api/v1/versions/{gem}.json`
- Download: `GET https://rubygems.org/gems/{gem}-{version}.gem`

#### 6. Cargo (crates.io)
```rust
pub struct CargoClient {
    base_url: String,
    http_client: reqwest::Client,
}
```

**API Endpoints**:
- Crate Info: `GET https://crates.io/api/v1/crates/{crate}`
- Download: `GET https://crates.io/api/v1/crates/{crate}/{version}/download`
- Uses sparse registry protocol

#### 7. Go Modules
```rust
pub struct GoModulesClient {
    proxy_url: String,
    http_client: reqwest::Client,
}
```

**API Endpoints**:
- Module Proxy: `GET https://proxy.golang.org/{module}/@v/{version}.mod`
- Module Info: `GET https://proxy.golang.org/{module}/@v/{version}.info`
- Checksum DB: `https://sum.golang.org/lookup/{module}@{version}`

### Registry Cache

```rust
pub struct RegistryCache {
    metadata_cache: Arc<DashMap<String, CachedMetadata>>,
    ttl: Duration,
}

struct CachedMetadata {
    data: PackageMetadata,
    cached_at: Instant,
}
```

**Features**:
- In-memory caching with TTL
- LRU eviction (planned)
- Thread-safe via DashMap
- Reduces registry API calls

---

## sctv-sbom

**Location**: `crates/sctv-sbom`

**Purpose**: Software Bill of Materials (SBOM) generation in industry-standard formats.

### Supported Formats

#### 1. CycloneDX 1.5
```rust
pub struct CycloneDxGenerator {
    xml_mode: bool,
}
```

**Features**:
- JSON and XML output
- Component metadata
- Dependency relationships
- License information
- External references
- Vulnerability correlation

**Example Output**:
```json
{
  "bomFormat": "CycloneDX",
  "specVersion": "1.5",
  "serialNumber": "urn:uuid:...",
  "version": 1,
  "metadata": {
    "timestamp": "2024-01-15T10:30:00Z",
    "component": {
      "type": "application",
      "name": "my-project",
      "version": "1.0.0"
    }
  },
  "components": [
    {
      "type": "library",
      "name": "lodash",
      "version": "4.17.21",
      "purl": "pkg:npm/lodash@4.17.21",
      "hashes": [
        {
          "alg": "SHA-256",
          "content": "abc123..."
        }
      ]
    }
  ]
}
```

#### 2. SPDX 2.3
```rust
pub struct SpdxGenerator {
    tag_value_mode: bool,
}
```

**Features**:
- JSON and tag-value output
- License compliance focus
- Package relationships
- File-level information
- Copyright notices

**Example Output**:
```json
{
  "spdxVersion": "SPDX-2.3",
  "dataLicense": "CC0-1.0",
  "SPDXID": "SPDXRef-DOCUMENT",
  "name": "my-project",
  "documentNamespace": "https://...",
  "creationInfo": {
    "created": "2024-01-15T10:30:00Z",
    "creators": ["Tool: SCTV-0.1.0"]
  },
  "packages": [
    {
      "SPDXID": "SPDXRef-Package-lodash-4.17.21",
      "name": "lodash",
      "versionInfo": "4.17.21",
      "downloadLocation": "https://registry.npmjs.org/lodash/-/lodash-4.17.21.tgz",
      "licenseConcluded": "MIT",
      "checksums": [
        {
          "algorithm": "SHA256",
          "checksumValue": "abc123..."
        }
      ]
    }
  ]
}
```

### Generator Configuration

```rust
pub struct GeneratorConfig {
    pub include_dev_dependencies: bool,
    pub include_hashes: bool,
    pub include_licenses: bool,
    pub include_external_refs: bool,
    pub author: Option<OrganizationalEntity>,
    pub supplier: Option<OrganizationalEntity>,
}
```

---

## sctv-worker

**Location**: `crates/sctv-worker`

**Purpose**: Background job processing system with PostgreSQL-backed queue.

### Architecture

```
┌────────────────────────────────────────────┐
│           WorkerService                    │
├────────────────────────────────────────────┤
│  ┌──────────────┐    ┌─────────────────┐  │
│  │ WorkerPool   │───>│ ExecutorRegistry│  │
│  │ (4 workers)  │    │  - ScanProject  │  │
│  │              │    │  - VerifyProv.. │  │
│  │              │    │  - SendNotif..  │  │
│  └──────────────┘    └─────────────────┘  │
│         │                                   │
│         v                                   │
│  ┌──────────────┐                          │
│  │  JobQueue    │                          │
│  │ (PostgreSQL) │                          │
│  └──────────────┘                          │
└────────────────────────────────────────────┘
```

### Job Queue

```rust
#[async_trait]
pub trait JobQueue: Send + Sync {
    async fn enqueue(&self, payload: JobPayload, options: EnqueueOptions)
        -> WorkerResult<JobId>;

    async fn claim_next(&self, job_types: &[JobType])
        -> WorkerResult<Option<Job>>;

    async fn complete(&self, job_id: JobId, result: JobResult)
        -> WorkerResult<()>;

    async fn fail(&self, job_id: JobId, error: &str)
        -> WorkerResult<()>;

    async fn release_stale_jobs(&self, timeout_minutes: u32)
        -> WorkerResult<u32>;
}
```

**Key SQL**: Atomic job claiming
```sql
SELECT * FROM jobs
WHERE status = 'pending'
  AND scheduled_at <= NOW()
  AND job_type = ANY($1)
ORDER BY priority DESC, created_at ASC
LIMIT 1
FOR UPDATE SKIP LOCKED;
```

### Job Types

#### ScanProject
```rust
pub struct ScanProjectPayload {
    pub project_id: ProjectId,
    pub full_scan: bool,
    pub detectors: Vec<DetectorType>,
}

pub struct ScanProjectResult {
    pub dependencies_found: u32,
    pub alerts_created: u32,
    pub scan_duration_ms: u64,
}
```

#### VerifyProvenance
```rust
pub struct VerifyProvenancePayload {
    pub dependency_id: DependencyId,
    pub package_name: String,
    pub version: String,
    pub ecosystem: PackageEcosystem,
}

pub struct VerifyProvenanceResult {
    pub status: ProvenanceVerificationStatus,
    pub slsa_level: Option<u8>,
    pub sigstore_bundle: Option<SigstoreDetails>,
}
```

#### SendNotification
```rust
pub struct SendNotificationPayload {
    pub alert_id: AlertId,
    pub channels: Vec<NotificationChannel>,
    pub context: NotificationContext,
}

pub struct SendNotificationResult {
    pub sent: Vec<NotificationChannel>,
    pub failed: Vec<(NotificationChannel, String)>,
}
```

### Worker Pool

```rust
pub struct WorkerPool {
    workers: Vec<JoinHandle<()>>,
    shutdown_tx: broadcast::Sender<()>,
    config: WorkerPoolConfig,
}

pub struct WorkerPoolConfig {
    pub num_workers: usize,
    pub poll_interval: Duration,
    pub max_jobs_per_worker: usize,
    pub job_timeout: Duration,
}
```

**Worker Loop**:
```
loop {
    1. Claim next job from queue
    2. Look up executor for job type
    3. Execute job with timeout
    4. Mark job as completed/failed
    5. Retry on failure (up to max_attempts)
    6. Sleep if no jobs available
}
```

### Executors

```rust
#[async_trait]
pub trait JobExecutor: Send + Sync {
    fn handles(&self) -> Vec<JobType>;

    async fn execute(&self, job: &Job, ctx: &ExecutionContext)
        -> WorkerResult<JobResult>;
}

pub struct ExecutorRegistry {
    executors: HashMap<JobType, BoxedExecutor>,
}
```

**Registered Executors**:
- `ScanProjectExecutor`: Orchestrates dependency scanning
- `VerifyProvenanceExecutor`: Verifies SLSA attestations
- `SendNotificationExecutor`: Sends alerts via configured channels
- `MonitorRegistryExecutor`: Monitors registries for package changes

---

## sctv-notifications

**Location**: `crates/sctv-notifications`

**Purpose**: Multi-channel notification delivery system for alerts.

### Supported Channels

#### 1. Email (SMTP)
```rust
pub struct EmailChannel {
    config: EmailConfig,
    transport: SmtpTransport,
}

pub struct EmailConfig {
    pub smtp_host: String,
    pub smtp_port: u16,
    pub smtp_username: String,
    pub smtp_password: String,
    pub from_address: String,
    pub from_name: String,
}
```

#### 2. Slack
```rust
pub struct SlackChannel {
    config: SlackConfig,
    http_client: reqwest::Client,
}

pub struct SlackConfig {
    pub webhook_url: String,
    pub channel: Option<String>,
    pub username: Option<String>,
    pub icon_emoji: Option<String>,
}
```

**Payload Format**:
```json
{
  "channel": "#security-alerts",
  "username": "SCTV Bot",
  "icon_emoji": ":warning:",
  "blocks": [
    {
      "type": "header",
      "text": {
        "type": "plain_text",
        "text": "🚨 Critical Alert: Typosquatting Detected"
      }
    },
    {
      "type": "section",
      "fields": [
        {"type": "mrkdwn", "text": "*Package:* lodash-utils"},
        {"type": "mrkdwn", "text": "*Severity:* Critical"}
      ]
    }
  ]
}
```

#### 3. Microsoft Teams
```rust
pub struct TeamsChannel {
    config: TeamsConfig,
    http_client: reqwest::Client,
}

pub struct TeamsConfig {
    pub webhook_url: String,
}
```

**Payload Format**: Office 365 Connector Card

#### 4. PagerDuty
```rust
pub struct PagerDutyChannel {
    config: PagerDutyConfig,
    http_client: reqwest::Client,
}

pub struct PagerDutyConfig {
    pub integration_key: String,
    pub severity_mapping: HashMap<Severity, String>,
}
```

**Events API v2**:
- Trigger incidents for Critical/High alerts
- Auto-resolve when alerts resolved
- Custom routing keys per severity

#### 5. Generic Webhook
```rust
pub struct WebhookChannel {
    config: WebhookConfig,
    http_client: reqwest::Client,
}

pub struct WebhookConfig {
    pub url: String,
    pub method: HttpMethod,
    pub headers: HashMap<String, String>,
    pub auth: Option<WebhookAuth>,
}
```

### Notification Service

```rust
pub struct NotificationService {
    channels: Vec<ChannelConfig>,
    config: NotificationServiceConfig,
}

pub struct NotificationServiceConfig {
    pub parallel_delivery: bool,
    pub continue_on_failure: bool,
    pub retry_attempts: u32,
    pub retry_delay: Duration,
}
```

**Delivery Logic**:
```rust
pub async fn send(&self, notification: &Notification) -> MultiChannelResult {
    let mut results = Vec::new();

    for channel in &self.channels {
        if notification.severity >= channel.min_severity {
            let result = channel.send(notification).await;
            results.push((channel.name(), result));
        }
    }

    MultiChannelResult::from(results)
}
```

---

## sctv-ci

**Location**: `crates/sctv-ci`

**Purpose**: CI/CD integration with SARIF output for code scanning platforms.

### SARIF Report Generation

```rust
pub struct SarifReport {
    pub schema: String,  // "https://json.schemastore.org/sarif-2.1.0.json"
    pub version: String, // "2.1.0"
    pub runs: Vec<SarifRun>,
}

pub struct SarifRun {
    pub tool: SarifTool,
    pub results: Vec<SarifResult>,
    pub invocations: Vec<SarifInvocation>,
}
```

**Alert Mapping**:
```
Alert Type          → SARIF Rule
─────────────────────────────────
Typosquatting       → typosquatting
Tampering           → dependency_tampering
Downgrade Attack    → downgrade_attack
Provenance Failure  → provenance_failure
Policy Violation    → policy_violation
New Package         → new_package
```

**Severity Mapping**:
```
SCTV Severity    → SARIF Level
─────────────────────────────────
Critical, High   → error
Medium           → warning
Low, Info        → note
```

### CI Integration

**GitHub Actions**:
```yaml
- name: Run SCTV Scan
  run: sctv scan --format sarif --output sctv-results.sarif

- name: Upload Results
  uses: github/codeql-action/upload-sarif@v2
  with:
    sarif_file: sctv-results.sarif
```

**GitLab CI**:
```yaml
sctv-scan:
  script:
    - sctv scan --format sarif --output gl-sast-report.json
  artifacts:
    reports:
      sast: gl-sast-report.json
```

### Exit Code Determination

```rust
pub struct CiConfig {
    pub fail_on_critical: bool,
    pub fail_on_high: bool,
    pub output_sarif: bool,
    pub sarif_output_path: Option<String>,
}

pub fn determine_exit_code(alerts: &[Alert], config: &CiConfig) -> i32 {
    for alert in alerts {
        match alert.severity {
            Severity::Critical if config.fail_on_critical => return 1,
            Severity::High if config.fail_on_high => return 1,
            _ => {}
        }
    }
    0
}
```

---

## sctv-cli

**Location**: `crates/sctv-cli`

**Purpose**: Command-line interface for local dependency scanning and verification.

### Command Structure

```
sctv [OPTIONS] <COMMAND>

Options:
  -v, --verbose         Enable verbose output
  -f, --format <FORMAT> Output format: text, json, sarif [default: text]

Commands:
  scan    Scan dependencies for supply chain threats
  check   Check a specific package for typosquatting
  verify  Verify package integrity
  policy  Evaluate a policy against dependencies
```

### Scan Command

```bash
sctv scan [OPTIONS]

Options:
  -p, --path <PATH>           Path to project directory [default: .]
  -e, --ecosystem <ECOSYSTEM> Package ecosystem to scan

Examples:
  sctv scan
  sctv scan --path ./my-project
  sctv scan --ecosystem npm --format json
```

**Output**:
```
Scanning project: my-project
Ecosystem: npm
Dependencies: 247 total, 12 direct

Threats Detected:
┌─────────────────────────────────────────────────────────────┐
│ [CRITICAL] Typosquatting Detected                           │
├─────────────────────────────────────────────────────────────┤
│ Package:    lodash-utils                                    │
│ Similar to: lodash                                          │
│ Method:     Combosquatting                                  │
│ Confidence: 0.92                                            │
└─────────────────────────────────────────────────────────────┘

Summary:
  Critical: 1
  High:     2
  Medium:   5
  Low:      3
```

### Check Command

```bash
sctv check <PACKAGE> [OPTIONS]

Options:
  -e, --ecosystem <ECOSYSTEM> Package ecosystem [default: npm]

Examples:
  sctv check lodash-utils
  sctv check requests --ecosystem pypi
```

### Verify Command

```bash
sctv verify <PACKAGE> <VERSION> [OPTIONS]

Options:
  -e, --ecosystem <ECOSYSTEM> Package ecosystem [default: npm]

Examples:
  sctv verify lodash 4.17.21
  sctv verify django 4.2.0 --ecosystem pypi
```

**Verification Steps**:
1. Fetch package metadata from registry
2. Download package tarball
3. Compute SHA-256 hash
4. Compare against registry hash
5. Verify signature (if available)
6. Check provenance attestation

### Policy Command

```bash
sctv policy [OPTIONS]

Options:
  -p, --policy <FILE> Path to policy file
  --path <PATH>       Path to project directory

Examples:
  sctv policy --policy policy.yaml
```

**Policy File Format** (YAML):
```yaml
name: Production Security Policy
rules:
  - type: RequireProvenance
    min_level: 2
  - type: RequireSignature
  - type: BlockPackages
    patterns:
      - "*-utils"
      - "test-*"
  - type: MaxDependencyAge
    days: 180
```

---

## sctv-dashboard

**Location**: `crates/sctv-dashboard`

**Purpose**: Web-based dashboard built with Leptos for server-side rendering and reactive UI.

### Architecture

```
┌───────────────────────────────────────┐
│         Leptos Application            │
├───────────────────────────────────────┤
│  SSR (Server-Side Rendering)          │
│  - Initial HTML generation            │
│  - SEO optimization                   │
├───────────────────────────────────────┤
│  Hydration (Client-Side)              │
│  - WebAssembly                        │
│  - Reactive signals                   │
│  - Router                             │
└───────────────────────────────────────┘
```

### Pages

#### Projects Page (`/projects`)
- List all projects with pagination
- Quick stats: Dependencies, Alerts, Last Scan
- Create new project modal
- Trigger scan button

#### Alerts Page (`/alerts`)
- Filterable alert list
- Severity badges
- Status workflow (Open → Acknowledged → Resolved)
- Alert details modal with remediation steps

#### Policies Page (`/policies`)
- Policy list with enable/disable toggle
- Create policy wizard
- Rule configuration UI
- Test policy against projects

#### Settings Page (`/settings`)
- Notification channel configuration
- API key management
- Team member invitations
- Audit log viewer

### Components

```rust
// Alert Badge Component
#[component]
pub fn AlertBadge(severity: Severity) -> impl IntoView {
    let (color, text) = match severity {
        Severity::Critical => ("bg-red-600", "Critical"),
        Severity::High => ("bg-orange-500", "High"),
        Severity::Medium => ("bg-yellow-500", "Medium"),
        Severity::Low => ("bg-blue-500", "Low"),
        Severity::Info => ("bg-gray-500", "Info"),
    };

    view! {
        <span class={format!("px-2 py-1 rounded text-white {color}")}>
            {text}
        </span>
    }
}

// Project Card Component
#[component]
pub fn ProjectCard(project: Project) -> impl IntoView {
    view! {
        <div class="border rounded-lg p-4 hover:shadow-lg">
            <h3 class="text-lg font-semibold">{&project.name}</h3>
            <p class="text-gray-600">{&project.description}</p>
            <div class="flex gap-4 mt-4">
                <Stat label="Dependencies" value={project.dependency_count} />
                <Stat label="Alerts" value={project.alert_count} />
            </div>
        </div>
    }
}
```

### State Management

```rust
// Global app context
#[derive(Clone)]
pub struct AppContext {
    pub api_base_url: Signal<String>,
    pub auth_token: RwSignal<Option<String>>,
    pub current_user: RwSignal<Option<User>>,
}

// Resource for fetching projects
pub fn use_projects() -> Resource<Vec<Project>> {
    let ctx = use_context::<AppContext>().expect("AppContext not found");

    create_resource(
        move || ctx.auth_token.get(),
        |token| async move {
            if let Some(token) = token {
                fetch_projects(&token).await
            } else {
                vec![]
            }
        },
    )
}
```

### Styling

Uses Tailwind CSS with custom SCTV design system:

```css
/* Custom color palette */
:root {
  --sctv-primary: #2563eb;
  --sctv-danger: #dc2626;
  --sctv-warning: #f59e0b;
  --sctv-success: #10b981;
}
```

**Typography**:
- Headings: Instrument Sans (Google Fonts)
- Code: Space Mono

---

## Component Dependencies

```
sctv-core (no dependencies on other SCTV crates)
   ↑
   ├── sctv-db
   ├── sctv-detectors
   ├── sctv-registries
   ├── sctv-sbom
   └── sctv-notifications
       ↑
       ├── sctv-worker ──┐
       ├── sctv-api ─────┤
       ├── sctv-ci       │
       ├── sctv-cli ─────┤
       └── sctv-dashboard┘
```

## Cross-Cutting Concerns

### Observability

All components use:
- `tracing` for structured logging
- `metrics` for Prometheus-compatible metrics
- Correlation IDs for request tracing

### Error Handling

```rust
// Each crate defines its own error type
pub enum XxxError {
    // Variants specific to the crate
}

// Converted to thiserror::Error
#[derive(Debug, Error)]
pub enum ApiError {
    #[error("Not found")]
    NotFound,

    #[error("Unauthorized")]
    Unauthorized,

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
}
```

### Testing

- Unit tests: Each crate has `#[cfg(test)]` modules
- Integration tests: `tests/` directories
- Mock support: `mockall` for repository traits
- Test containers: PostgreSQL via `testcontainers`

---

## Deployment Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                      Load Balancer                          │
└───────────────────────┬─────────────────────────────────────┘
                        │
        ┌───────────────┼───────────────┐
        │               │               │
   ┌────▼────┐    ┌────▼────┐    ┌────▼────┐
   │ API     │    │ API     │    │ API     │
   │ Server  │    │ Server  │    │ Server  │
   └─────────┘    └─────────┘    └─────────┘
        │               │               │
        └───────────────┼───────────────┘
                        │
        ┌───────────────┼───────────────┐
        │               │               │
   ┌────▼────┐    ┌────▼────┐    ┌────▼────┐
   │ Worker  │    │ Worker  │    │ Worker  │
   │ Pool    │    │ Pool    │    │ Pool    │
   └─────────┘    └─────────┘    └─────────┘
        │               │               │
        └───────────────┼───────────────┘
                        │
                 ┌──────▼──────┐
                 │ PostgreSQL  │
                 │  (Primary)  │
                 └─────────────┘
```

This completes the comprehensive component documentation for the SCTV project.
