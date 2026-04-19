# GraphQL API Reference

**Version:** 0.1.0
**Endpoint:** `http://localhost:3000/graphql`

Complete reference for the SCTV GraphQL API, built with async-graphql.

---

## Table of Contents

- [Getting Started](#getting-started)
- [Authentication](#authentication)
- [Schema Overview](#schema-overview)
- [Queries](#queries)
- [Mutations](#mutations)
- [Subscriptions](#subscriptions)
- [Types](#types)
- [Input Types](#input-types)
- [Enums](#enums)
- [Pagination](#pagination)
- [Error Handling](#error-handling)
- [Code Examples](#code-examples)
- [Best Practices](#best-practices)

---

## Getting Started

### Making Your First Request

```bash
curl -X POST http://localhost:3000/graphql \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer YOUR_JWT_TOKEN" \
  -d '{
    "query": "{ projects(page: 1, perPage: 10) { id name alertCount } }"
  }'
```

### Interactive Query Tools

The API server does not ship an embedded GraphQL playground. The GraphQL endpoint accepts only `POST /graphql`. To explore the schema interactively, use an external client such as:

- [Apollo Sandbox](https://studio.apollographql.com/sandbox/explorer) (browser-based)
- [Altair GraphQL](https://altairgraphql.dev/)
- [Insomnia](https://insomnia.rest/) or [Postman](https://www.postman.com/) with their built-in GraphQL support

Point the client at `http://localhost:3000/graphql` and supply a JWT `Authorization: Bearer <token>` header.

---

## Authentication

GraphQL requests require authentication via JWT tokens. Include the token in the Authorization header:

```http
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
```

### How Authentication Works

1. The token is extracted from the `Authorization` header
2. JWT is validated against the server's secret
3. Claims are parsed to extract `tenant_id` and `user_id`
4. Context is injected into GraphQL resolvers
5. Multi-tenant isolation is enforced at the repository level

### Obtaining a Token

Use the REST authentication endpoint:

```bash
curl -X POST http://localhost:3000/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "email": "user@example.com",
    "password": "your-password"
  }'
```

See [Authentication Documentation](authentication.md) for complete details.

---

## Schema Overview

The SCTV GraphQL schema is organized into three main components:

### Root Types

- **Query**: Read operations for fetching data
- **Mutation**: Write operations for creating, updating, and deleting data
- **Subscription**: Real-time event notifications (currently EmptySubscription)

### Core Domain Types

- **Project**: Software projects being monitored
- **Dependency**: Package dependencies with integrity data
- **Alert**: Security alerts and policy violations
- **Policy**: Configurable security policies
- **Scan**: Dependency scan operations and results

### Type System

The GraphQL schema uses:
- **ID**: Unique identifiers (UUIDs)
- **DateTime**: ISO 8601 timestamps
- **Enums**: Constrained value sets (Status, Severity, etc.)
- **Custom Scalars**: Domain-specific types

---

## Queries

### projects

Fetch all projects for the authenticated tenant with pagination.

**Signature:**

```graphql
projects(page: Int = 1, perPage: Int = 20): [Project!]!
```

**Arguments:**

| Name | Type | Default | Description |
|------|------|---------|-------------|
| `page` | Int | 1 | Page number (1-indexed) |
| `perPage` | Int | 20 | Items per page (max: 100) |

**Returns:** List of Project objects

**Example:**

```graphql
query GetProjects {
  projects(page: 1, perPage: 10) {
    id
    name
    description
    repositoryUrl
    status
    dependencyCount
    alertCount
    lastScanAt
    createdAt
  }
}
```

**Response:**

```json
{
  "data": {
    "projects": [
      {
        "id": "550e8400-e29b-41d4-a716-446655440000",
        "name": "my-api-server",
        "description": "REST API backend service",
        "repositoryUrl": "https://github.com/org/my-api-server",
        "status": "HEALTHY",
        "dependencyCount": 127,
        "alertCount": 3,
        "lastScanAt": "2026-01-15T10:30:00Z",
        "createdAt": "2026-01-10T08:00:00Z"
      }
    ]
  }
}
```

---

### project

Fetch a specific project by ID.

**Signature:**

```graphql
project(id: ID!): Project
```

**Arguments:**

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `id` | ID | Yes | Project UUID |

**Returns:** Project object or null if not found

**Example:**

```graphql
query GetProject($projectId: ID!) {
  project(id: $projectId) {
    id
    name
    description
    repositoryUrl
    status
    isActive
    dependencyCount
    alertCount
    lastScanAt
    createdAt
  }
}
```

**Variables:**

```json
{
  "projectId": "550e8400-e29b-41d4-a716-446655440000"
}
```

---

### alerts

Fetch alerts with optional filters and pagination.

**Signature:**

```graphql
alerts(
  projectId: ID
  severity: Severity
  status: AlertStatus
  page: Int = 1
  perPage: Int = 20
): [Alert!]!
```

**Arguments:**

| Name | Type | Default | Description |
|------|------|---------|-------------|
| `projectId` | ID | null | Filter by project |
| `severity` | Severity | null | Filter by severity level |
| `status` | AlertStatus | null | Filter by alert status |
| `page` | Int | 1 | Page number |
| `perPage` | Int | 20 | Items per page |

**Example:**

```graphql
query GetCriticalAlerts {
  alerts(severity: CRITICAL, status: OPEN, page: 1, perPage: 25) {
    id
    projectId
    alertType
    severity
    title
    description
    status
    dependencyName
    dependencyVersion
    createdAt
    acknowledgedAt
    resolvedAt
  }
}
```

**Filtering Examples:**

```graphql
# All alerts for a specific project
query ProjectAlerts($projectId: ID!) {
  alerts(projectId: $projectId) {
    id
    title
    severity
    status
  }
}

# High and critical severity alerts
query HighSeverityAlerts {
  alerts(severity: HIGH, status: OPEN) {
    id
    title
    description
  }
}
```

---

### alert

Fetch a specific alert by ID.

**Signature:**

```graphql
alert(id: ID!): Alert
```

**Example:**

```graphql
query GetAlert($alertId: ID!) {
  alert(id: $alertId) {
    id
    projectId
    alertType
    severity
    title
    description
    status
    dependencyName
    dependencyVersion
    createdAt
    acknowledgedAt
    resolvedAt
  }
}
```

---

### dependencies

Fetch dependencies for a project with optional filters.

**Signature:**

```graphql
dependencies(
  projectId: ID!
  ecosystem: PackageEcosystem
  isDirect: Boolean
  page: Int = 1
  perPage: Int = 50
): [Dependency!]!
```

**Arguments:**

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `projectId` | ID | Yes | Project UUID |
| `ecosystem` | PackageEcosystem | No | Filter by ecosystem |
| `isDirect` | Boolean | No | Filter direct/transitive deps |
| `page` | Int | No | Page number |
| `perPage` | Int | No | Items per page |

**Example:**

```graphql
query GetProjectDependencies($projectId: ID!) {
  dependencies(projectId: $projectId, isDirect: true) {
    id
    packageName
    ecosystem
    versionConstraint
    resolvedVersion
    isDirect
    isDevDependency
    depth
    hashSha256
    signatureVerified
    provenanceLevel
    firstSeenAt
    lastVerifiedAt
  }
}
```

**Ecosystem Filtering:**

```graphql
query NpmDependencies($projectId: ID!) {
  dependencies(projectId: $projectId, ecosystem: NPM) {
    packageName
    resolvedVersion
    signatureVerified
  }
}
```

---

### policies

Fetch all policies for the authenticated tenant.

**Signature:**

```graphql
policies: [Policy!]!
```

**Example:**

```graphql
query GetPolicies {
  policies {
    id
    name
    description
    severity
    isEnabled
    createdAt
    updatedAt
  }
}
```

---

## Mutations

### createProject

Create a new project.

**Signature:**

```graphql
createProject(input: CreateProjectInput!): Project!
```

**Input:**

```graphql
input CreateProjectInput {
  name: String!
  description: String
  repositoryUrl: String
}
```

**Example:**

```graphql
mutation CreateProject($input: CreateProjectInput!) {
  createProject(input: $input) {
    id
    name
    description
    repositoryUrl
    status
    createdAt
  }
}
```

**Variables:**

```json
{
  "input": {
    "name": "my-new-project",
    "description": "A new microservice project",
    "repositoryUrl": "https://github.com/org/my-new-project"
  }
}
```

**Response:**

```json
{
  "data": {
    "createProject": {
      "id": "660e8400-e29b-41d4-a716-446655440001",
      "name": "my-new-project",
      "description": "A new microservice project",
      "repositoryUrl": "https://github.com/org/my-new-project",
      "status": "UNKNOWN",
      "createdAt": "2026-01-15T10:30:00Z"
    }
  }
}
```

---

### updateProject

Update an existing project.

**Signature:**

```graphql
updateProject(id: ID!, input: UpdateProjectInput!): Project
```

**Input:**

```graphql
input UpdateProjectInput {
  name: String
  description: String
  repositoryUrl: String
  isActive: Boolean
}
```

**Example:**

```graphql
mutation UpdateProject($id: ID!, $input: UpdateProjectInput!) {
  updateProject(id: $id, input: $input) {
    id
    name
    description
    updatedAt
  }
}
```

**Variables:**

```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "input": {
    "description": "Updated project description"
  }
}
```

---

### deleteProject

Delete a project and all associated data.

**Signature:**

```graphql
deleteProject(id: ID!): Boolean!
```

**Example:**

```graphql
mutation DeleteProject($id: ID!) {
  deleteProject(id: $id)
}
```

**Returns:** `true` if deleted, `false` if not found

---

### triggerScan

Trigger a dependency scan for a project.

**Signature:**

```graphql
triggerScan(projectId: ID!, fullScan: Boolean): Scan!
```

**Arguments:**

| Name | Type | Default | Description |
|------|------|---------|-------------|
| `projectId` | ID | Required | Project to scan |
| `fullScan` | Boolean | false | Force full rescan |

**Example:**

```graphql
mutation TriggerScan($projectId: ID!) {
  triggerScan(projectId: $projectId, fullScan: false) {
    id
    projectId
    status
    startedAt
    completedAt
    dependenciesFound
    alertsCreated
    errorMessage
  }
}
```

**Response:**

```json
{
  "data": {
    "triggerScan": {
      "id": "770e8400-e29b-41d4-a716-446655440002",
      "projectId": "550e8400-e29b-41d4-a716-446655440000",
      "status": "queued",
      "startedAt": "2026-01-15T10:35:00Z",
      "completedAt": null,
      "dependenciesFound": 0,
      "alertsCreated": 0,
      "errorMessage": null
    }
  }
}
```

---

### acknowledgeAlert

Acknowledge an alert to indicate it has been reviewed.

**Signature:**

```graphql
acknowledgeAlert(id: ID!, notes: String): Alert
```

**Example:**

```graphql
mutation AcknowledgeAlert($id: ID!, $notes: String) {
  acknowledgeAlert(id: $id, notes: $notes) {
    id
    status
    acknowledgedAt
  }
}
```

**Variables:**

```json
{
  "id": "880e8400-e29b-41d4-a716-446655440003",
  "notes": "Investigating with security team"
}
```

---

### resolveAlert

Resolve an alert with remediation details.

**Signature:**

```graphql
resolveAlert(
  id: ID!
  actionTaken: String!
  newVersion: String
): Alert
```

**Arguments:**

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `id` | ID | Yes | Alert UUID |
| `actionTaken` | String | Yes | Description of action |
| `newVersion` | String | No | New version if upgraded |

**Example:**

```graphql
mutation ResolveAlert($id: ID!) {
  resolveAlert(
    id: $id
    actionTaken: "Upgraded to patched version"
    newVersion: "2.4.1"
  ) {
    id
    status
    resolvedAt
  }
}
```

---

### createPolicy

Create a new security policy.

**Signature:**

```graphql
createPolicy(input: CreatePolicyInput!): Policy!
```

**Input:**

```graphql
input CreatePolicyInput {
  name: String!
  description: String
  severity: Severity!
  isEnabled: Boolean
}
```

**Example:**

```graphql
mutation CreatePolicy($input: CreatePolicyInput!) {
  createPolicy(input: $input) {
    id
    name
    description
    severity
    isEnabled
    createdAt
  }
}
```

**Variables:**

```json
{
  "input": {
    "name": "Block High Severity Vulnerabilities",
    "description": "Prevent dependencies with high severity issues",
    "severity": "HIGH",
    "isEnabled": true
  }
}
```

---

## Subscriptions

Subscriptions enable real-time updates via WebSocket connections. Currently, the SCTV API uses `EmptySubscription`, but the following subscriptions are planned for future releases:

### alertCreated (Planned)

Subscribe to new alert notifications.

```graphql
subscription OnAlertCreated($projectId: ID) {
  alertCreated(projectId: $projectId) {
    id
    projectId
    title
    severity
    createdAt
  }
}
```

### scanProgress (Planned)

Monitor scan progress in real-time.

```graphql
subscription OnScanProgress($scanId: ID!) {
  scanProgress(scanId: $scanId) {
    scanId
    status
    progress
    currentStep
    dependenciesProcessed
  }
}
```

**Implementation Status:** Subscriptions are in the roadmap. Use polling or webhooks for real-time updates in the current version.

---

## Types

### Project

Represents a software project being monitored.

```graphql
type Project {
  id: ID!
  name: String!
  description: String
  repositoryUrl: String
  status: ProjectStatus!
  isActive: Boolean!
  dependencyCount: Int!
  alertCount: Int!
  lastScanAt: DateTime
  createdAt: DateTime!
}
```

**Fields:**

| Field | Type | Description |
|-------|------|-------------|
| `id` | ID! | Unique identifier (UUID) |
| `name` | String! | Project name |
| `description` | String | Optional description |
| `repositoryUrl` | String | Git repository URL |
| `status` | ProjectStatus! | Health status |
| `isActive` | Boolean! | Whether project is active |
| `dependencyCount` | Int! | Total dependencies |
| `alertCount` | Int! | Open alerts count |
| `lastScanAt` | DateTime | Last scan timestamp |
| `createdAt` | DateTime! | Creation timestamp |

---

### Alert

Represents a security alert or policy violation.

```graphql
type Alert {
  id: ID!
  projectId: ID!
  alertType: String!
  severity: Severity!
  title: String!
  description: String!
  status: AlertStatus!
  dependencyName: String
  dependencyVersion: String
  createdAt: DateTime!
  acknowledgedAt: DateTime
  resolvedAt: DateTime
}
```

**Alert Types:**

- `DependencyTampering`: Package hash mismatch
- `DowngradeAttack`: Version downgrade detected
- `Typosquatting`: Suspicious package name
- `ProvenanceFailure`: Missing or invalid provenance
- `PolicyViolation`: Policy rule violation
- `NewPackage`: New dependency added
- `SuspiciousMaintainer`: Untrusted maintainer

---

### Dependency

Represents a package dependency with integrity data.

```graphql
type Dependency {
  id: ID!
  projectId: ID!
  packageName: String!
  ecosystem: PackageEcosystem!
  versionConstraint: String!
  resolvedVersion: String!
  isDirect: Boolean!
  isDevDependency: Boolean!
  depth: Int!
  hashSha256: String
  signatureVerified: Boolean!
  provenanceLevel: Int
  firstSeenAt: DateTime!
  lastVerifiedAt: DateTime!
}
```

**Provenance Levels:**

- `0`: SLSA Level 0 (no provenance)
- `1`: SLSA Level 1 (basic provenance)
- `2`: SLSA Level 2 (verified build)
- `3`: SLSA Level 3 (hardened build)

---

### Policy

Represents a security policy configuration.

```graphql
type Policy {
  id: ID!
  name: String!
  description: String
  severity: Severity!
  isEnabled: Boolean!
  createdAt: DateTime!
  updatedAt: DateTime!
}
```

---

### Scan

Represents a dependency scan operation.

```graphql
type Scan {
  id: ID!
  projectId: ID!
  status: String!
  startedAt: DateTime!
  completedAt: DateTime
  dependenciesFound: Int!
  alertsCreated: Int!
  errorMessage: String
}
```

**Scan Statuses:**

- `queued`: Waiting to start
- `running`: Currently scanning
- `completed`: Finished successfully
- `failed`: Encountered an error

---

## Input Types

### CreateProjectInput

```graphql
input CreateProjectInput {
  name: String!
  description: String
  repositoryUrl: String
}
```

### UpdateProjectInput

```graphql
input UpdateProjectInput {
  name: String
  description: String
  repositoryUrl: String
  isActive: Boolean
}
```

### CreatePolicyInput

```graphql
input CreatePolicyInput {
  name: String!
  description: String
  severity: Severity!
  isEnabled: Boolean
}
```

---

## Enums

### ProjectStatus

```graphql
enum ProjectStatus {
  UNKNOWN
  HEALTHY
  AT_RISK
  VULNERABLE
  CRITICAL
}
```

### AlertStatus

```graphql
enum AlertStatus {
  OPEN
  ACKNOWLEDGED
  RESOLVED
  SUPPRESSED
  FALSE_POSITIVE
}
```

### Severity

```graphql
enum Severity {
  LOW
  MEDIUM
  HIGH
  CRITICAL
}
```

### PackageEcosystem

```graphql
enum PackageEcosystem {
  NPM
  PYPI
  CARGO
  MAVEN
  NUGET
  GO_MODULES
  RUBYGEMS
}
```

---

## Pagination

SCTV GraphQL API uses offset-based pagination with `page` and `perPage` parameters.

### Pattern

```graphql
query PaginatedProjects($page: Int!, $perPage: Int!) {
  projects(page: $page, perPage: $perPage) {
    id
    name
  }
}
```

### Best Practices

1. **Default Values**: Use `page: 1` and `perPage: 20` as defaults
2. **Maximum Page Size**: Respect maximum `perPage: 100` limit
3. **Empty Results**: Empty arrays indicate no more results
4. **Performance**: Request only needed fields to reduce payload size

### Example Pagination Logic

```javascript
let page = 1;
let allProjects = [];
let hasMore = true;

while (hasMore) {
  const result = await fetchProjects(page, 50);
  allProjects = allProjects.concat(result.projects);
  hasMore = result.projects.length === 50;
  page++;
}
```

---

## Error Handling

### Error Format

GraphQL errors are returned in the standard GraphQL error format:

```json
{
  "errors": [
    {
      "message": "Authentication required",
      "locations": [{ "line": 2, "column": 3 }],
      "path": ["projects"],
      "extensions": {
        "code": "UNAUTHENTICATED"
      }
    }
  ],
  "data": null
}
```

### Common Error Codes

| Code | Description | Solution |
|------|-------------|----------|
| `UNAUTHENTICATED` | No/invalid token | Provide valid JWT token |
| `FORBIDDEN` | Insufficient permissions | Check user role/tenant access |
| `NOT_FOUND` | Resource not found | Verify ID is correct |
| `BAD_USER_INPUT` | Invalid input | Check input format |
| `INTERNAL_SERVER_ERROR` | Server error | Check server logs |

### Error Handling Best Practices

```javascript
async function fetchProject(id) {
  try {
    const result = await client.query({
      query: GET_PROJECT,
      variables: { id }
    });

    if (result.errors) {
      // Handle GraphQL errors
      console.error('GraphQL errors:', result.errors);
      return null;
    }

    return result.data.project;
  } catch (error) {
    // Handle network/client errors
    console.error('Request failed:', error);
    throw error;
  }
}
```

---

## Code Examples

### JavaScript (Apollo Client)

```javascript
import { ApolloClient, InMemoryCache, gql, createHttpLink } from '@apollo/client';
import { setContext } from '@apollo/client/link/context';

// Configure Apollo Client with authentication
const httpLink = createHttpLink({
  uri: 'http://localhost:3000/graphql',
});

const authLink = setContext((_, { headers }) => {
  const token = localStorage.getItem('jwt_token');
  return {
    headers: {
      ...headers,
      authorization: token ? `Bearer ${token}` : '',
    }
  };
});

const client = new ApolloClient({
  link: authLink.concat(httpLink),
  cache: new InMemoryCache(),
});

// Query projects
const GET_PROJECTS = gql`
  query GetProjects {
    projects(page: 1, perPage: 10) {
      id
      name
      alertCount
      dependencyCount
    }
  }
`;

client.query({ query: GET_PROJECTS })
  .then(result => console.log(result.data.projects))
  .catch(error => console.error(error));

// Create project mutation
const CREATE_PROJECT = gql`
  mutation CreateProject($input: CreateProjectInput!) {
    createProject(input: $input) {
      id
      name
      createdAt
    }
  }
`;

client.mutate({
  mutation: CREATE_PROJECT,
  variables: {
    input: {
      name: 'my-service',
      description: 'Microservice project',
      repositoryUrl: 'https://github.com/org/my-service'
    }
  }
}).then(result => console.log(result.data.createProject));
```

### Python (gql)

```python
from gql import gql, Client
from gql.transport.requests import RequestsHTTPTransport

# Configure client with authentication
transport = RequestsHTTPTransport(
    url='http://localhost:3000/graphql',
    headers={'Authorization': f'Bearer {jwt_token}'},
    verify=True,
    retries=3,
)

client = Client(transport=transport, fetch_schema_from_transport=True)

# Query alerts
query = gql('''
    query GetAlerts($severity: Severity!, $status: AlertStatus!) {
      alerts(severity: $severity, status: $status) {
        id
        title
        severity
        status
        createdAt
      }
    }
''')

result = client.execute(query, variable_values={
    'severity': 'CRITICAL',
    'status': 'OPEN'
})

for alert in result['alerts']:
    print(f"Alert: {alert['title']} - {alert['severity']}")
```

### Rust (graphql-client)

```rust
use graphql_client::{GraphQLQuery, Response};
use reqwest::header::{HeaderMap, AUTHORIZATION, CONTENT_TYPE};

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schema.graphql",
    query_path = "queries/get_projects.graphql",
    response_derives = "Debug"
)]
pub struct GetProjects;

async fn fetch_projects(token: &str) -> Result<Vec<Project>, Box<dyn std::error::Error>> {
    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, "application/json".parse()?);
    headers.insert(AUTHORIZATION, format!("Bearer {}", token).parse()?);

    let client = reqwest::Client::builder()
        .default_headers(headers)
        .build()?;

    let variables = get_projects::Variables {
        page: Some(1),
        per_page: Some(20),
    };

    let request_body = GetProjects::build_query(variables);

    let response = client
        .post("http://localhost:3000/graphql")
        .json(&request_body)
        .send()
        .await?;

    let response_body: Response<get_projects::ResponseData> = response.json().await?;

    if let Some(data) = response_body.data {
        Ok(data.projects)
    } else {
        Err("No data returned".into())
    }
}
```

### cURL

```bash
# Query example
curl -X POST http://localhost:3000/graphql \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer YOUR_JWT_TOKEN" \
  -d '{
    "query": "query GetProjects { projects(page: 1, perPage: 5) { id name alertCount } }"
  }'

# Mutation example
curl -X POST http://localhost:3000/graphql \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer YOUR_JWT_TOKEN" \
  -d '{
    "query": "mutation CreateProject($input: CreateProjectInput!) { createProject(input: $input) { id name createdAt } }",
    "variables": {
      "input": {
        "name": "my-project",
        "description": "Test project"
      }
    }
  }'
```

---

## Best Practices

### 1. Request Only What You Need

GraphQL allows you to specify exactly which fields you want. Use this to minimize payload size:

```graphql
# Good - Request only needed fields
query GetProjectNames {
  projects {
    id
    name
  }
}

# Avoid - Requesting everything
query GetProjects {
  projects {
    id
    name
    description
    repositoryUrl
    status
    isActive
    dependencyCount
    alertCount
    lastScanAt
    createdAt
  }
}
```

### 2. Use Variables for Dynamic Queries

```graphql
# Good - Use variables
query GetProject($id: ID!) {
  project(id: $id) {
    name
  }
}

# Avoid - Inline values
query GetProject {
  project(id: "550e8400-e29b-41d4-a716-446655440000") {
    name
  }
}
```

### 3. Handle Errors Gracefully

```javascript
const result = await client.query({ query: GET_PROJECTS });

if (result.errors) {
  result.errors.forEach(error => {
    console.error(`GraphQL Error: ${error.message}`);
    // Log to error tracking service
    Sentry.captureException(error);
  });
}

if (result.data?.projects) {
  // Process data
}
```

### 4. Use Fragments for Reusable Selections

```graphql
fragment ProjectSummary on Project {
  id
  name
  status
  alertCount
}

query GetProjects {
  projects {
    ...ProjectSummary
  }
}

query GetProject($id: ID!) {
  project(id: $id) {
    ...ProjectSummary
    description
    dependencyCount
  }
}
```

### 5. Implement Caching

```javascript
const client = new ApolloClient({
  cache: new InMemoryCache({
    typePolicies: {
      Query: {
        fields: {
          projects: {
            merge(existing, incoming) {
              return incoming;
            }
          }
        }
      }
    }
  })
});
```

### 6. Batch Related Queries

```graphql
query GetDashboardData($projectId: ID!) {
  project(id: $projectId) {
    id
    name
    alertCount
    dependencyCount
  }
  alerts(projectId: $projectId, status: OPEN) {
    id
    title
    severity
  }
  policies {
    id
    name
    isEnabled
  }
}
```

### 7. Use Aliases for Multiple Similar Queries

```graphql
query GetMultipleSeverityAlerts($projectId: ID!) {
  criticalAlerts: alerts(projectId: $projectId, severity: CRITICAL) {
    id
    title
  }
  highAlerts: alerts(projectId: $projectId, severity: HIGH) {
    id
    title
  }
}
```

### 8. Implement Retry Logic

```javascript
async function queryWithRetry(query, variables, maxRetries = 3) {
  for (let i = 0; i < maxRetries; i++) {
    try {
      return await client.query({ query, variables });
    } catch (error) {
      if (i === maxRetries - 1) throw error;
      await new Promise(resolve => setTimeout(resolve, 1000 * Math.pow(2, i)));
    }
  }
}
```

### 9. Monitor Query Performance

```javascript
const link = new ApolloLink((operation, forward) => {
  const startTime = Date.now();

  return forward(operation).map(response => {
    const duration = Date.now() - startTime;
    console.log(`Query ${operation.operationName} took ${duration}ms`);
    return response;
  });
});
```

### 10. Validate Inputs Client-Side

```javascript
function createProject(input) {
  // Validate before sending
  if (!input.name || input.name.length < 3) {
    throw new Error('Project name must be at least 3 characters');
  }

  if (input.repositoryUrl && !isValidUrl(input.repositoryUrl)) {
    throw new Error('Invalid repository URL');
  }

  return client.mutate({
    mutation: CREATE_PROJECT,
    variables: { input }
  });
}
```

---

## Additional Resources

- [GraphQL Official Documentation](https://graphql.org/learn/)
- [async-graphql Documentation](https://async-graphql.github.io/async-graphql/)
- [SCTV Authentication Guide](authentication.md)
- [SCTV REST API Documentation](rest-api.md)
- [GraphQL Best Practices](https://graphql.org/learn/best-practices/)

---

**Last Updated:** 2026-01-15
**API Version:** 0.1.0
**Maintainer:** SCTV Team
