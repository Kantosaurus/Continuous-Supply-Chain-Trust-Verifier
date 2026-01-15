# Supply Chain Trust Verifier (SCTV) Documentation

**Version:** 0.1.0
**Last Updated:** 2026-01-15

## Welcome to SCTV

Supply Chain Trust Verifier (SCTV) is an enterprise-grade platform for detecting and preventing supply chain attacks in software dependencies. It provides continuous monitoring, threat detection, and policy enforcement across multiple package ecosystems.

---

## Quick Links

### Getting Started
- [Installation Guide](getting-started/installation.md) - Set up SCTV in your environment
- [Quick Start Tutorial](getting-started/quickstart.md) - Get up and running in 15 minutes
- [Configuration Guide](getting-started/configuration.md) - Configure SCTV for your needs

### User Documentation
- [User Guide Overview](user-guide/README.md) - Learn how to use SCTV effectively
- [Dashboard Guide](user-guide/dashboard.md) - Navigate the web interface
- [CLI Reference](reference/cli-reference.md) - Command-line tool usage
- [Best Practices](user-guide/best-practices.md) - Security recommendations

### API Documentation
- [REST API Reference](api/rest-api.md) - HTTP endpoints and examples
- [GraphQL API](api/graphql-api.md) - GraphQL schema and queries
- [Authentication](api/authentication.md) - API authentication methods
- [Webhooks](api/webhooks.md) - CI/CD integration webhooks

### Development
- [Development Setup](development/setup.md) - Set up local development environment
- [Contributing Guide](development/contributing.md) - How to contribute to SCTV
- [Code Standards](development/code-standards.md) - Coding conventions and guidelines
- [Testing Guide](development/testing.md) - Writing and running tests

### Operations
- [Deployment Guide](operations/deployment.md) - Deploy SCTV to production
- [Monitoring](operations/monitoring.md) - Observability and metrics
- [Troubleshooting](operations/troubleshooting.md) - Common issues and solutions
- [Security Hardening](operations/security.md) - Production security guidelines
- [Backup and Recovery](operations/backup.md) - Data protection strategies

### Architecture
- [System Architecture](architecture/overview.md) - High-level system design
- [Component Design](architecture/components.md) - Individual component architecture
- [Data Flow](architecture/data-flow.md) - How data moves through the system
- [Database Schema](architecture/database.md) - PostgreSQL schema documentation

### Reference
- [Configuration Reference](reference/configuration.md) - All configuration options
- [Error Codes](reference/error-codes.md) - Error codes and meanings
- [Supported Ecosystems](reference/ecosystems.md) - Package manager support matrix
- [Threat Types](reference/threat-types.md) - Detection capabilities

---

## Documentation Structure

```
docs/
├── README.md (this file)
├── getting-started/         # Installation, quickstart, configuration
│   ├── installation.md
│   ├── quickstart.md
│   └── configuration.md
├── user-guide/              # End-user documentation
│   ├── README.md
│   ├── dashboard.md
│   ├── projects.md
│   ├── alerts.md
│   ├── policies.md
│   ├── sbom.md
│   └── best-practices.md
├── api/                     # API documentation
│   ├── rest-api.md
│   ├── graphql-api.md
│   ├── authentication.md
│   └── webhooks.md
├── architecture/            # System design
│   ├── overview.md
│   ├── components.md
│   ├── data-flow.md
│   └── database.md
├── development/             # Developer documentation
│   ├── setup.md
│   ├── contributing.md
│   ├── code-standards.md
│   └── testing.md
├── operations/              # Operations and deployment
│   ├── deployment.md
│   ├── monitoring.md
│   ├── troubleshooting.md
│   ├── security.md
│   └── backup.md
└── reference/               # Technical reference
    ├── configuration.md
    ├── cli-reference.md
    ├── error-codes.md
    ├── ecosystems.md
    └── threat-types.md
```

---

## What is SCTV?

SCTV is a comprehensive supply chain security platform that helps organizations:

### 🔍 Detect Threats
- **Typosquatting attacks** - Identify malicious packages with similar names
- **Dependency tampering** - Verify package integrity with cryptographic hashes
- **Downgrade attacks** - Detect suspicious version rollbacks
- **Provenance failures** - Validate SLSA/Sigstore attestations
- **Policy violations** - Enforce organizational security policies
- **Suspicious activity** - Monitor maintainer and package behavior

### 📦 Multi-Ecosystem Support
- npm (JavaScript/Node.js)
- PyPI (Python)
- Maven (Java)
- NuGet (.NET)
- RubyGems (Ruby)
- Cargo (Rust)
- Go modules (Go)

### 🛡️ Continuous Monitoring
- Automated dependency scanning
- Real-time threat detection
- Registry monitoring
- Scheduled security audits
- Webhook integrations for CI/CD

### 📊 Enterprise Features
- Multi-tenant architecture
- Role-based access control (RBAC)
- Custom policy engine
- SBOM generation (CycloneDX, SPDX)
- SARIF output for CI integration
- Audit logging and compliance

### 🔔 Smart Notifications
- Email, Slack, Teams, PagerDuty
- Severity-based filtering
- Custom notification rules
- Webhook delivery

---

## Key Concepts

### Projects
A **project** represents a software application or service with dependencies. Each project is scanned regularly to detect supply chain threats.

### Dependencies
**Dependencies** are external packages your project relies on. SCTV tracks both direct and transitive dependencies across all supported ecosystems.

### Alerts
**Alerts** are security findings generated when threats are detected. Each alert includes severity, description, remediation steps, and can be acknowledged or resolved.

### Policies
**Policies** define security rules for your projects. Rules can enforce version constraints, block packages, require signatures, and more.

### SBOM (Software Bill of Materials)
An **SBOM** is a complete inventory of your software components. SCTV generates SBOMs in industry-standard formats for compliance and security audits.

---

## Architecture Overview

SCTV is built with a microservices architecture using Rust for performance and safety:

```
┌─────────────────────────────────────────────────────────────┐
│                         Frontend                            │
│                    (Leptos Dashboard)                       │
└────────────────────────┬────────────────────────────────────┘
                         │
┌────────────────────────┴────────────────────────────────────┐
│                       API Layer                             │
│              (REST API + GraphQL Server)                    │
└────────────────────────┬────────────────────────────────────┘
                         │
          ┌──────────────┴──────────────┐
          │                             │
┌─────────┴─────────┐         ┌────────┴────────┐
│   Core Services   │         │  Worker System  │
│  - Detectors      │         │  - Job Queue    │
│  - Registry       │         │  - Executors    │
│    Clients        │         │  - Background   │
│  - SBOM Gen       │         │    Processing   │
└─────────┬─────────┘         └────────┬────────┘
          │                            │
          └──────────┬─────────────────┘
                     │
          ┌──────────┴──────────┐
          │   PostgreSQL DB     │
          │  - Multi-tenant     │
          │  - Job Queue        │
          │  - Audit Logs       │
          └─────────────────────┘
```

### Core Components

1. **sctv-core** - Domain models and business logic
2. **sctv-api** - REST and GraphQL API server
3. **sctv-db** - Database layer with repositories
4. **sctv-detectors** - Threat detection engines
5. **sctv-registries** - Package registry clients
6. **sctv-sbom** - SBOM generation (CycloneDX/SPDX)
7. **sctv-worker** - Background job processing
8. **sctv-notifications** - Multi-channel alerts
9. **sctv-ci** - CI/CD integrations (SARIF)
10. **sctv-cli** - Command-line interface
11. **sctv-dashboard** - Web UI

---

## Support and Community

### Getting Help
- **Documentation**: You're reading it!
- **Issues**: [GitHub Issues](https://github.com/example/supply-chain-trust-verifier/issues)
- **Discussions**: [GitHub Discussions](https://github.com/example/supply-chain-trust-verifier/discussions)

### Contributing
We welcome contributions! See our [Contributing Guide](development/contributing.md) for details.

### Security
Found a security issue? Please email security@example.com instead of filing a public issue.

---

## License

SCTV is dual-licensed under MIT OR Apache-2.0.

See [LICENSE-MIT](../LICENSE-MIT) and [LICENSE-APACHE](../LICENSE-APACHE) for details.

---

## Next Steps

- **New to SCTV?** Start with the [Quick Start Guide](getting-started/quickstart.md)
- **Setting up production?** Check the [Deployment Guide](operations/deployment.md)
- **Integrating with CI/CD?** See [Webhooks Documentation](api/webhooks.md)
- **Building custom policies?** Read the [User Guide](user-guide/policies.md)

---

**Happy securing!** 🔒
