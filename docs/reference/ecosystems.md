# Package Ecosystems Reference

Complete reference for supported package ecosystems in SCTV.

## Table of Contents

- [Supported Ecosystems Overview](#supported-ecosystems-overview)
- [npm (JavaScript/Node.js)](#npm-javascriptnodejs)
- [PyPI (Python)](#pypi-python)
- [Maven Central (Java)](#maven-central-java)
- [NuGet (.NET)](#nuget-net)
- [RubyGems (Ruby)](#rubygems-ruby)
- [Cargo/crates.io (Rust)](#cargocrates-io-rust)
- [Go Modules (Go)](#go-modules-go)
- [Lock File Support Matrix](#lock-file-support-matrix)
- [Provenance Support Matrix](#provenance-support-matrix)
- [Future Ecosystem Roadmap](#future-ecosystem-roadmap)

## Supported Ecosystems Overview

SCTV supports seven major package ecosystems, each with specific features and limitations.

| Ecosystem | Registry | Identifier Format | Version Format |
|-----------|----------|-------------------|----------------|
| npm | registry.npmjs.org | Package name (scoped: `@scope/name`) | Semantic versioning |
| PyPI | pypi.org | Package name (normalized) | PEP 440 |
| Maven | repo1.maven.org | `groupId:artifactId` | Maven versioning |
| NuGet | nuget.org | Package ID | Semantic versioning |
| RubyGems | rubygems.org | Gem name | Gem versioning |
| Cargo | crates.io | Crate name | Semantic versioning |
| Go | proxy.golang.org | Module path | Semantic versioning with `v` prefix |

### Ecosystem Enum

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
```

## npm (JavaScript/Node.js)

### Overview

- **Registry:** https://registry.npmjs.org
- **Package Format:** Tarball (.tgz)
- **Version Format:** Semantic versioning (semver)
- **Package Identifier:** Package name or scoped name (`@scope/package`)

### Features

**Supported:**
- Package metadata fetching
- Version-specific metadata
- Dependency resolution
- Scoped packages (`@babel/core`, `@types/node`)
- Deprecation detection
- Download URL resolution
- Checksum verification (SHA-512, integrity field)
- Popular package listings

**Advanced Features:**
- Abbreviated package metadata (faster queries)
- Full package document with all versions
- npm integrity hash verification
- Dev dependencies tracking
- Optional dependencies detection

### Limitations

- SHA-256 checksums not provided by default (npm uses SHA-512)
- No official provenance support yet (GitHub Attestations in beta)
- Maintainer information may be incomplete
- No built-in typosquatting protection from registry

### Configuration

**Default Registry:**
```rust
NpmClient::DEFAULT_REGISTRY = "https://registry.npmjs.org"
```

**Custom Registry:**
```rust
let client = NpmClient::with_config(
    "https://custom.registry.com",
    Arc::new(RegistryCache::new())
);
```

**HTTP Settings:**
- Timeout: 30 seconds
- User-Agent: `sctv-registry-client/0.1.0`
- GZIP: Enabled
- Accept Header: `application/vnd.npm.install-v1+json` (for abbreviated)

### Package Name Format

**Standard Packages:**
```
lodash
express
react
```

**Scoped Packages:**
```
@babel/core
@types/node
@angular/core
```

**URL Encoding:**
- Scoped packages: `@babel/core` → `@babel%2Fcore`

### Version Examples

```
1.0.0          # Standard semver
2.3.4-beta.1   # Pre-release
3.0.0+build.1  # Build metadata
^1.0.0         # Caret range (dependency)
~2.3.0         # Tilde range (dependency)
```

### Checksum Verification

npm provides multiple hash formats:

```rust
PackageChecksums {
    sha256: None,  // Not provided by npm
    sha512: Some("abc123..."),  // SHA-512 in 'shasum' field
    integrity: Some("sha512-xyz789..."),  // Subresource Integrity
}
```

**Integrity Format:**
```
sha512-<base64-encoded-hash>
```

### Example API Calls

**Fetch Package Metadata:**
```bash
GET https://registry.npmjs.org/lodash
Accept: application/vnd.npm.install-v1+json
```

**Fetch Specific Version:**
```bash
GET https://registry.npmjs.org/lodash/4.17.21
```

**Download Package:**
```bash
GET https://registry.npmjs.org/lodash/-/lodash-4.17.21.tgz
```

### Known Issues

1. **Rate Limiting:** Aggressive rate limits for unauthenticated requests
2. **Large Packages:** Some packages have 100+ MB metadata (full document)
3. **Scoped Packages:** Different URL encoding required

## PyPI (Python)

### Overview

- **Registry:** https://pypi.org
- **Package Format:** Wheel (.whl) or source distribution (.tar.gz)
- **Version Format:** PEP 440 (flexible, converted to semver)
- **Package Identifier:** Package name (normalized)

### Features

**Supported:**
- Package metadata (JSON API)
- Version-specific metadata
- Dependency resolution (from `requires_dist`)
- Yanked version detection
- Multiple distribution formats
- SHA-256 and Blake2b-256 checksums
- PEP 740 attestation support (new)
- Popular package listings

**Advanced Features:**
- Name normalization (PEP 503)
- Wheel selection (prefers universal wheels)
- Project URL extraction
- Author/maintainer detection
- Upload timestamp tracking

### Limitations

- Dependencies only available for latest or specific versions
- No historical dependency data
- Version parsing may differ from pip (PEP 440 vs semver)
- Attestations (PEP 740) not yet widely adopted

### Configuration

**Default Registry:**
```rust
PyPiClient::DEFAULT_REGISTRY = "https://pypi.org"
```

**API Endpoints:**
```
/pypi/{package}/json           # Latest version
/pypi/{package}/{version}/json # Specific version
/integrity/{package}/{version}/{filename}/provenance  # PEP 740
```

### Package Name Normalization

PyPI normalizes package names per PEP 503:

```rust
// Normalization rules
"My_Package" → "my-package"
"My.Package" → "my-package"
"MY-PACKAGE" → "my-package"
```

**Example:**
```
Django        → django
Pillow        → pillow
requests      → requests
scikit-learn  → scikit-learn (already normalized)
```

### Version Format

PyPI uses PEP 440, which is more flexible than strict semver:

```
1.0.0          # Standard
2.3.4.post1    # Post-release
3.0.0rc1       # Release candidate
0.1.dev0       # Development release
1.0+local      # Local version identifier
```

**Conversion to Semver:**
```rust
"1.0" → "1.0.0"
"2.3.4.5" → "2.3.4-5" (extra segment as prerelease)
```

### Distribution Types

PyPI provides multiple file types per release:

| Type | Extension | Description | Priority |
|------|-----------|-------------|----------|
| Wheel (universal) | `.whl` (py3-none-any) | Pure Python, all platforms | Highest |
| Wheel (platform) | `.whl` (specific) | Platform-specific | High |
| Source distribution | `.tar.gz`, `.zip` | Source code | Medium |

**Selection Logic:**
1. Universal wheel (py3-none-any)
2. Compatible wheel (py3-none-*)
3. Platform wheel (py3-*)
4. Source distribution

### Checksum Verification

```rust
PackageChecksums {
    sha256: Some("abc123..."),      // Primary checksum
    sha512: None,                    // Not provided
    integrity: Some("blake2b_256:xyz..."),  // Blake2b (if available)
}
```

### PEP 740 Attestations

Fetch provenance attestations (new feature):

```rust
let attestations = client.fetch_attestations(
    "requests",
    "2.31.0",
    "requests-2.31.0-py3-none-any.whl"
).await?;
```

**Attestation Format:**
```json
{
  "version": 1,
  "attestation_bundles": [
    {
      "publisher": "...",
      "attestations": [...]
    }
  ]
}
```

### Example API Calls

**Fetch Package:**
```bash
GET https://pypi.org/pypi/requests/json
```

**Fetch Specific Version:**
```bash
GET https://pypi.org/pypi/requests/2.31.0/json
```

**Fetch Attestations:**
```bash
GET https://pypi.org/integrity/requests/2.31.0/requests-2.31.0-py3-none-any.whl/provenance
```

## Maven Central (Java)

### Overview

- **Registry:** https://repo1.maven.org/maven2
- **Search API:** https://search.maven.org
- **Package Format:** JAR, WAR, AAR, POM
- **Version Format:** Maven versioning (flexible)
- **Package Identifier:** `groupId:artifactId`

### Features

**Supported:**
- Maven metadata (XML)
- POM file parsing
- Dependency resolution
- SHA-1 and SHA-256 checksums
- Multiple artifact types
- Release and snapshot versions
- Artifact coordinate parsing

**Advanced Features:**
- Maven Central search API
- POM property resolution
- Dependency scope detection
- Transitive dependency analysis
- Developer information extraction

### Limitations

- No official deprecation/yanked support
- Large POM files can be complex to parse
- Property interpolation not fully implemented
- Parent POM resolution not automatic
- No built-in provenance system

### Configuration

**Default Registry:**
```rust
MavenClient::DEFAULT_REGISTRY = "https://repo1.maven.org/maven2"
```

**Search API:**
```rust
MavenClient::SEARCH_API = "https://search.maven.org/solrsearch/select"
```

### Package Identifier Format

Maven uses coordinates: `groupId:artifactId:version`

```
org.springframework:spring-core:5.3.23
com.google.guava:guava:31.1-jre
junit:junit:4.13.2
```

**Repository Path:**
```
groupId (dots → slashes)  : org/springframework
artifactId                : spring-core
version                   : 5.3.23
artifact                  : spring-core-5.3.23.jar
```

**Full Path:**
```
https://repo1.maven.org/maven2/org/springframework/spring-core/5.3.23/spring-core-5.3.23.jar
```

### Version Format

Maven versioning is flexible:

```
1.0             # Two-part version
1.0.0           # Three-part version
1.0-SNAPSHOT    # Snapshot (development)
1.0-beta-1      # Qualifier
1.0.Final       # Named release
```

**Conversion to Semver:**
```rust
"5.3" → "5.3.0"
"1.0-beta" → "1.0.0-beta"
```

### Artifact Types

Maven supports multiple packaging types:

| Type | Extension | Description |
|------|-----------|-------------|
| `jar` | `.jar` | Java archive (default) |
| `war` | `.war` | Web application archive |
| `ear` | `.ear` | Enterprise archive |
| `pom` | `.pom` | POM-only (no artifact) |
| `aar` | `.aar` | Android archive |

**POM Packaging Field:**
```xml
<packaging>jar</packaging>
```

### Checksum Files

Maven provides separate checksum files:

```
spring-core-5.3.23.jar        # Main artifact
spring-core-5.3.23.jar.sha1   # SHA-1 checksum
spring-core-5.3.23.jar.sha256 # SHA-256 checksum (newer)
spring-core-5.3.23.jar.md5    # MD5 (deprecated)
```

**Checksum Format:**
```
<hash> <filename>
# or just
<hash>
```

### Dependency Scopes

Maven dependencies have scopes:

```xml
<dependency>
  <groupId>junit</groupId>
  <artifactId>junit</artifactId>
  <version>4.13.2</version>
  <scope>test</scope>
  <optional>true</optional>
</dependency>
```

**Scopes:**
- `compile` - Default, runtime and compile
- `test` - Test-only (maps to `is_dev: true`)
- `provided` - Provided by environment (maps to `is_optional: true`)
- `runtime` - Runtime only
- `system` - System path

### Example API Calls

**Fetch Metadata:**
```bash
GET https://repo1.maven.org/maven2/org/springframework/spring-core/maven-metadata.xml
```

**Fetch POM:**
```bash
GET https://repo1.maven.org/maven2/org/springframework/spring-core/5.3.23/spring-core-5.3.23.pom
```

**Search:**
```bash
GET https://search.maven.org/solrsearch/select?q=spring-core&rows=10&wt=json
```

## NuGet (.NET)

### Overview

- **Registry:** https://api.nuget.org/v3/index.json (service index)
- **Package Format:** .nupkg (ZIP-based)
- **Version Format:** Semantic versioning (NuGet flavor)
- **Package Identifier:** Package ID (case-preserving)

### Features

**Supported:**
- Service index discovery
- Registration API (package metadata)
- Package content API (downloads)
- Dependency group resolution
- Framework-specific dependencies
- Deprecation support
- Unlisted package detection

**Advanced Features:**
- Multi-page registration indexes
- Package vulnerability data
- Icon and README URLs
- License metadata
- Framework compatibility

### Limitations

- No checksum in registration API (must download)
- Complex dependency groups (target frameworks)
- Service index discovery required
- Case-insensitive but case-preserving names

### Configuration

**Default Service Index:**
```rust
NuGetClient::DEFAULT_REGISTRY = "https://api.nuget.org/v3/index.json"
```

**Service Discovery:**
```rust
// Fetches service index to find:
// - RegistrationsBaseUrl
// - PackageBaseAddress
// - SearchQueryService
```

### Package Identifier Format

NuGet uses package IDs (case-insensitive):

```
Newtonsoft.Json
Microsoft.Extensions.Logging
EntityFramework
```

**URL Format:**
```
Package ID (lowercase): newtonsoft.json
Registration URL: /newtonsoft.json/index.json
Download URL: /newtonsoft.json/13.0.3/newtonsoft.json.13.0.3.nupkg
```

### Version Format

NuGet follows semantic versioning with additions:

```
1.0.0             # Standard
2.3.4-beta        # Pre-release
3.0.0-beta.1      # Pre-release with numeric suffix
1.0.0+metadata    # Build metadata
```

**Latest Version Logic:**
1. Prefer non-prerelease
2. Highest version number

### Dependency Groups

NuGet dependencies are grouped by target framework:

```json
{
  "dependencyGroups": [
    {
      "targetFramework": "net6.0",
      "dependencies": [
        {
          "id": "System.Text.Json",
          "range": "[6.0.0, )"
        }
      ]
    },
    {
      "targetFramework": "netstandard2.0",
      "dependencies": [...]
    }
  ]
}
```

**Handled in SCTV:**
- All dependency groups merged
- Target framework ignored (assumes compatible)

### Deprecation

NuGet supports official deprecation:

```json
{
  "deprecation": {
    "reasons": ["Legacy", "CriticalBugs"],
    "message": "Package is deprecated, use X instead",
    "alternatePackage": {
      "id": "NewPackage",
      "range": "[2.0.0, )"
    }
  }
}
```

### Example API Calls

**Service Index:**
```bash
GET https://api.nuget.org/v3/index.json
```

**Registration:**
```bash
GET https://api.nuget.org/v3/registration5-semver1/newtonsoft.json/index.json
```

**Download:**
```bash
GET https://api.nuget.org/v3-flatcontainer/newtonsoft.json/13.0.3/newtonsoft.json.13.0.3.nupkg
```

## RubyGems (Ruby)

### Overview

- **Registry:** https://rubygems.org
- **Package Format:** .gem
- **Version Format:** Gem versioning (mostly semver-compatible)
- **Package Identifier:** Gem name

### Features

**Supported:**
- Gem metadata API
- Version listing
- Dependency resolution (runtime and development)
- Yanked version detection
- Owner (maintainer) information
- Download counts
- SHA-256 checksums

**Advanced Features:**
- Prerelease version filtering
- Platform-specific gems
- Required Ruby version
- License information

### Limitations

- Dependencies are for current version only (not per-version)
- No official provenance system
- Gem version format not strict semver (may have 4+ segments)
- Owner API may require authentication

### Configuration

**Default Registry:**
```rust
RubyGemsClient::DEFAULT_REGISTRY = "https://rubygems.org"
```

**API Endpoints:**
```
/api/v1/gems/{name}.json          # Gem info
/api/v1/versions/{name}.json      # All versions
/api/v1/gems/{name}/owners.json   # Owners
/gems/{name}-{version}.gem        # Download
```

### Package Name Format

Gem names are simple strings:

```
rails
bundler
devise
activerecord
```

**Naming Conventions:**
- Lowercase
- Hyphens allowed
- Underscores allowed
- No scopes/namespaces

### Version Format

Ruby uses gem versioning (flexible):

```
1.0.0             # Standard three-part
2.3.4.5           # Four or more parts
3.0.0.pre.1       # Prerelease
1.0.0.rc1         # Release candidate
```

**Conversion to Semver:**
```rust
"1.0.0" → "1.0.0"         // Direct
"1.0" → "1.0.0"           // Pad with zero
"1.0.0.5" → "1.0.0-5"     // Extra as prerelease
```

### Dependency Types

RubyGems has runtime and development dependencies:

```ruby
spec.add_dependency 'rails', '~> 7.0'
spec.add_development_dependency 'rspec', '~> 3.0'
```

**Mapped to:**
```rust
PackageDependency {
    name: "rails",
    version_constraint: "~> 7.0",
    is_optional: false,
    is_dev: false,  // runtime
}
```

### Yanked Versions

RubyGems supports yanking (soft delete):

```json
{
  "number": "1.0.0",
  "yanked": true,
  "sha": "abc123..."
}
```

**Effect:**
- Still downloadable
- Not recommended for installation
- `gem install` won't select by default

### Example API Calls

**Gem Info:**
```bash
GET https://rubygems.org/api/v1/gems/rails.json
```

**All Versions:**
```bash
GET https://rubygems.org/api/v1/versions/rails.json
```

**Download:**
```bash
GET https://rubygems.org/gems/rails-7.0.4.gem
```

## Cargo/crates.io (Rust)

### Overview

- **Registry:** https://crates.io (API)
- **Static Files:** https://static.crates.io
- **Package Format:** .crate (tarball)
- **Version Format:** Semantic versioning
- **Package Identifier:** Crate name (case-sensitive)

### Features

**Supported:**
- Crate metadata API
- Version-specific information
- Dependency resolution (with features)
- Yanked version detection
- Owner information
- SHA-256 checksums
- Download counts
- Documentation links

**Advanced Features:**
- Feature flags
- Dependency kinds (normal, dev, build)
- Optional dependencies
- License detection
- Repository links

### Limitations

- Requires specific User-Agent header
- No official provenance (cargo-vet ecosystem separate)
- Features not fully modeled in dependency graph
- Build dependencies tracked but not deeply analyzed

### Configuration

**Default Registry:**
```rust
CargoClient::DEFAULT_REGISTRY = "https://crates.io"
CargoClient::STATIC_URL = "https://static.crates.io"
```

**API Endpoints:**
```
/api/v1/crates/{name}                     # Crate info
/api/v1/crates/{name}/{version}/dependencies  # Dependencies
/api/v1/crates/{name}/owners              # Owners
```

**Download:**
```
/crates/{name}/{name}-{version}.crate
```

### Package Name Format

Crate names are case-sensitive:

```
serde
tokio
clap
serde_json
```

**Naming Rules:**
- Lowercase preferred
- Hyphens and underscores allowed
- No scopes/namespaces

### Version Format

Cargo uses strict semantic versioning:

```
1.0.0             # Release
2.3.4-beta.1      # Pre-release
3.0.0+build       # Build metadata (ignored for version comparison)
```

### Dependency Kinds

Cargo has three dependency types:

```toml
[dependencies]
serde = "1.0"          # Normal (runtime)

[dev-dependencies]
criterion = "0.5"      # Development

[build-dependencies]
cc = "1.0"             # Build-time
```

**Mapped to:**
```rust
PackageDependency {
    name: "serde",
    version_constraint: "1.0",
    is_optional: false,  // can be true
    is_dev: false,       // true for dev-dependencies
}
```

### Optional Dependencies

Cargo supports optional dependencies tied to features:

```toml
[dependencies]
serde = { version = "1.0", optional = true }

[features]
serialization = ["serde"]
```

### Yanked Versions

Yanked versions are marked but still downloadable:

```json
{
  "num": "1.0.0",
  "yanked": true,
  "created_at": "2023-01-01T00:00:00Z"
}
```

### Example API Calls

**Crate Info:**
```bash
GET https://crates.io/api/v1/crates/serde
User-Agent: sctv-registry-client/0.1.0
```

**Dependencies:**
```bash
GET https://crates.io/api/v1/crates/serde/1.0.193/dependencies
```

**Download:**
```bash
GET https://static.crates.io/crates/serde/serde-1.0.193.crate
```

## Go Modules (Go)

### Overview

- **Proxy:** https://proxy.golang.org
- **Package Format:** .zip
- **Version Format:** Semantic versioning with `v` prefix
- **Package Identifier:** Module path (case-sensitive)

### Features

**Supported:**
- Module version listing
- Version info (timestamp)
- go.mod file parsing
- Dependency resolution
- Retraction detection
- Module path encoding
- Semantic import versioning

**Advanced Features:**
- Replace directives (parsed but not resolved)
- Retract directive support
- Indirect dependency tracking
- go.sum checksum verification (future)

### Limitations

- No checksum in API (need go.sum for verification)
- No package metadata (description, authors)
- Dependency versions may be pseudo-versions
- Replace directives not fully supported
- No official provenance system

### Configuration

**Default Proxy:**
```rust
GoModulesClient::DEFAULT_REGISTRY = "https://proxy.golang.org"
```

**Proxy Protocol Endpoints:**
```
/{module}/@v/list                # Version list
/{module}/@v/{version}.info      # Version info (JSON)
/{module}/@v/{version}.mod       # go.mod file
/{module}/@v/{version}.zip       # Source code
```

### Module Path Format

Go uses import paths as identifiers:

```
github.com/gin-gonic/gin
go.uber.org/zap
golang.org/x/tools
```

**Path Encoding:**
- Uppercase letters: `A` → `!a`
- Example: `github.com/Azure/azure-sdk` → `github.com/!azure/azure-sdk`

### Version Format

Go uses semantic versioning with `v` prefix:

```
v1.0.0                          # Release
v2.3.4-beta.1                   # Pre-release
v0.0.0-20230101000000-abc123    # Pseudo-version
v1.2.3+incompatible             # Pre-module version
```

**Pseudo-Versions:**
```
v0.0.0-<timestamp>-<commit>
```

**Semantic Import Versioning:**
```
github.com/gin-gonic/gin/v2    # Major version in path
```

### go.mod Directives

**Module Declaration:**
```go
module github.com/example/project

go 1.21
```

**Requirements:**
```go
require (
    github.com/gin-gonic/gin v1.9.0
    go.uber.org/zap v1.24.0
)
```

**Indirect Dependencies:**
```go
require (
    github.com/indirect/dep v1.0.0 // indirect
)
```

**Retractions:**
```go
retract (
    v1.0.0 // Broken release
    [v1.5.0, v1.7.0] // Security issue
)
```

**Replace Directives:**
```go
replace github.com/old/module => github.com/new/module v1.2.3
```

### Retraction Detection

SCTV detects retracted versions:

```rust
// Parsed from go.mod
Retract {
    low: "v1.0.0",
    high: None,  // Single version
    rationale: Some("Broken release"),
}
```

**Effect on Analysis:**
- Marked as `yanked: true`
- Deprecation message set
- Alert raised if in use

### Example API Calls

**Version List:**
```bash
GET https://proxy.golang.org/github.com/gin-gonic/gin/@v/list
```

**Version Info:**
```bash
GET https://proxy.golang.org/github.com/gin-gonic/gin/@v/v1.9.0.info
```

**go.mod File:**
```bash
GET https://proxy.golang.org/github.com/gin-gonic/gin/@v/v1.9.0.mod
```

**Download:**
```bash
GET https://proxy.golang.org/github.com/gin-gonic/gin/@v/v1.9.0.zip
```

## Lock File Support Matrix

Lock files ensure reproducible builds by pinning exact versions.

| Ecosystem | Lock File | Supported | Parsed Dependencies | Integrity Checks |
|-----------|-----------|-----------|---------------------|------------------|
| npm | `package-lock.json` | Planned | Yes (future) | SHA-512 |
| npm | `yarn.lock` | Planned | Yes (future) | SHA-1 |
| npm | `pnpm-lock.yaml` | Planned | Yes (future) | SHA-512 |
| PyPI | `requirements.txt` | Partial | No (manual format) | No |
| PyPI | `poetry.lock` | Planned | Yes (future) | SHA-256 |
| PyPI | `Pipfile.lock` | Planned | Yes (future) | SHA-256 |
| Maven | N/A | - | - | - |
| NuGet | `packages.lock.json` | Planned | Yes (future) | SHA-512 |
| RubyGems | `Gemfile.lock` | Planned | Yes (future) | No |
| Cargo | `Cargo.lock` | Planned | Yes (future) | SHA-256 |
| Go | `go.sum` | Partial | No (checksums only) | SHA-256 |

**Current Status:**
- Lock file parsing not yet implemented
- Planned for version 0.2.x
- Will enable better dependency pinning detection

## Provenance Support Matrix

Provenance attestations verify build authenticity and integrity.

| Ecosystem | Standard | Registry Support | SCTV Support | Signature Format |
|-----------|----------|------------------|--------------|------------------|
| npm | GitHub Attestations | Beta | Planned | Sigstore |
| PyPI | PEP 740 | Rolling out | Implemented | Sigstore |
| Maven | N/A | No | No | - |
| NuGet | Package Signing | Yes | Planned | X.509 certificates |
| RubyGems | N/A | No | No | - |
| Cargo | N/A | No | No | - |
| Go | N/A | No | No | - |

**Sigstore Integration:**
- SCTV includes Sigstore verification for PyPI (PEP 740)
- Can be enabled/disabled: `SIGSTORE_ENABLED=true`
- Planned for npm GitHub Attestations

**Future Work:**
- SLSA provenance verification
- In-toto attestations
- Custom attestation formats

## Future Ecosystem Roadmap

### Planned Ecosystems

**Q2 2024:**
- **Composer (PHP)** - packagist.org
- **Hex (Elixir)** - hex.pm

**Q3 2024:**
- **CPAN (Perl)** - metacpan.org
- **Pub (Dart)** - pub.dev

**Q4 2024:**
- **Swift Package Manager** - github.com-based
- **Conan (C/C++)** - conan.io

### Enhanced Features

**All Ecosystems:**
- Lock file parsing
- Dependency graph visualization
- Vulnerability database integration
- License compliance checking

**Per-Ecosystem:**
- npm: GitHub Attestations support
- PyPI: Full PEP 740 implementation
- Maven: GPG signature verification
- NuGet: Package signing validation
- Go: go.sum verification

### Community Requests

Submit ecosystem requests via GitHub Issues:
- Label: `ecosystem-request`
- Include: Registry URL, API documentation, sample packages
- Priority based on community votes

## See Also

- [Getting Started - Supported Ecosystems](../getting-started/supported-ecosystems.md)
- [API Reference - Package Endpoints](./api.md#packages)
- [Architecture - Registry Clients](../architecture/registry-clients.md)
- [Contributing - Adding Ecosystems](../development/contributing.md#adding-ecosystems)
