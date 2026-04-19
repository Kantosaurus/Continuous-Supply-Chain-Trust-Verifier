# Development Environment Setup

This guide will help you set up your development environment for the Supply Chain Trust Verifier (SCTV) project.

## Table of Contents

- [Prerequisites](#prerequisites)
- [Cloning the Repository](#cloning-the-repository)
- [Environment Configuration](#environment-configuration)
- [Docker Development Environment](#docker-development-environment)
- [Native Development Setup](#native-development-setup)
- [IDE Configuration](#ide-configuration)
- [Running Services Locally](#running-services-locally)
- [Database Setup and Migrations](#database-setup-and-migrations)
- [Hot Reload Setup](#hot-reload-setup)
- [Debugging Tips](#debugging-tips)

## Prerequisites

Before starting, ensure you have the following installed on your development machine:

### Required Tools

1. **Rust** (latest stable version)
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   rustup default stable
   ```

2. **Docker** and **Docker Compose**
   - Docker Desktop 20.10+ (includes Docker Compose v2)
   - Verify installation:
     ```bash
     docker --version
     docker compose version
     ```

3. **PostgreSQL Client Tools** (for native development)
   - PostgreSQL 16 or later
   - Includes `psql` CLI tool

4. **Git**
   ```bash
   git --version
   ```

### Optional but Recommended

- **cargo-watch** - For hot reloading during development
  ```bash
  cargo install cargo-watch
  ```

- **sqlx-cli** - For database migrations
  ```bash
  cargo install sqlx-cli --no-default-features --features postgres
  ```

- **cargo-llvm-cov** - For code coverage
  ```bash
  cargo install cargo-llvm-cov
  ```

- **cargo-audit** - For security vulnerability scanning
  ```bash
  cargo install cargo-audit
  ```

## Cloning the Repository

1. Clone the repository:
   ```bash
   git clone https://github.com/example/supply-chain-trust-verifier.git
   cd supply-chain-trust-verifier
   ```

2. Verify the workspace structure:
   ```bash
   ls -la crates/
   ```

   You should see the following crates:
   - `sctv-core` - Core domain models and business logic
   - `sctv-db` - Database layer and repositories
   - `sctv-api` - REST and GraphQL API server
   - `sctv-worker` - Background job processor
   - `sctv-cli` - Command-line interface
   - `sctv-registries` - Package registry integrations
   - `sctv-detectors` - Threat detection logic
   - `sctv-sbom` - SBOM parsing and generation
   - `sctv-notifications` - Notification system
   - `sctv-ci` - CI/CD integration helpers
   - `sctv-dashboard` - Web UI (Leptos)

## Environment Configuration

1. Copy the example environment file:
   ```bash
   cp .env.example .env
   ```

2. Edit `.env` with your settings:
   ```bash
   # Database Configuration
   POSTGRES_USER=sctv
   POSTGRES_PASSWORD=your_secure_password
   POSTGRES_DB=sctv
   POSTGRES_PORT=5432
   DATABASE_URL=postgres://sctv:your_secure_password@localhost:5432/sctv

   # API Server Configuration
   API_PORT=3000
   SCTV_JWT_SECRET=change-this-to-a-secure-secret
   SCTV_ENABLE_CORS=true
   SCTV_LOG_FORMAT=pretty

   # Logging
   RUST_LOG=info,sctv_api=debug,sctv_worker=debug,tower_http=info

   # Worker Configuration
   SCTV_WORKER_COUNT=4
   SCTV_POLL_INTERVAL_MS=1000
   SCTV_SHUTDOWN_TIMEOUT_SECS=30
   ```

3. **Important**: Never commit `.env` files with real credentials to version control!

## Docker Development Environment

The fastest way to get started is using Docker Compose.

### Start All Services

```bash
# Start all services (database, API, worker)
make dev

# Or manually:
docker compose up -d
```

This will start:
- PostgreSQL database on port 5432
- SCTV API server on port 3000
- SCTV worker in the background

### Verify Services

```bash
# Check running containers
docker compose ps

# View logs
docker compose logs -f

# Check API health
curl http://localhost:3000/health
```

### Access Services

- API: http://localhost:3000
- GraphQL endpoint (POST only): http://localhost:3000/graphql
- PostgreSQL: localhost:5432

### Common Docker Commands

```bash
# Stop all services
make dev-down
# Or: docker compose down

# Restart services
make dev-restart

# View specific service logs
make logs-api    # API logs
make logs-worker # Worker logs
make logs-db     # Database logs

# Access database shell
make db-shell
# Or: docker compose exec postgres psql -U sctv -d sctv

# Rebuild images
make build
```

## Native Development Setup

For local development without Docker (faster compilation, easier debugging):

### 1. Start PostgreSQL

Start a PostgreSQL instance (via Docker or native):

```bash
# Option A: Using Docker for database only
docker run -d \
  --name sctv-postgres \
  -e POSTGRES_USER=sctv \
  -e POSTGRES_PASSWORD=sctv \
  -e POSTGRES_DB=sctv \
  -p 5432:5432 \
  postgres:16-alpine

# Option B: Use existing PostgreSQL installation
# Create database and user manually
createdb sctv
```

### 2. Run Database Migrations

```bash
# Install sqlx-cli if not already installed
cargo install sqlx-cli --no-default-features --features postgres

# Set DATABASE_URL
export DATABASE_URL="postgres://sctv:sctv@localhost:5432/sctv"

# Run migrations
sqlx migrate run
```

### 3. Build the Project

```bash
# Build all crates
cargo build

# Build in release mode (faster runtime)
cargo build --release

# Build specific binary
cargo build --bin sctv-api
```

### 4. Run Services

Open separate terminal windows for each service:

```bash
# Terminal 1: API Server
cargo run --bin sctv-api

# Terminal 2: Worker
cargo run --bin sctv-worker

# Terminal 3: CLI (for testing)
cargo run --bin sctv -- --help
```

## IDE Configuration

### Visual Studio Code

1. **Install Extensions**:
   - rust-analyzer (official Rust language server)
   - Even Better TOML
   - CodeLLDB (for debugging)
   - Error Lens (inline errors)

2. **Create `.vscode/settings.json`**:
   ```json
   {
     "rust-analyzer.cargo.features": "all",
     "rust-analyzer.check.command": "clippy",
     "rust-analyzer.checkOnSave": true,
     "rust-analyzer.cargo.allTargets": true,
     "rust-analyzer.procMacro.enable": true,
     "editor.formatOnSave": true,
     "[rust]": {
       "editor.defaultFormatter": "rust-lang.rust-analyzer",
       "editor.formatOnSave": true
     },
     "files.watcherExclude": {
       "**/target/**": true
     }
   }
   ```

3. **Create `.vscode/launch.json`** (for debugging):
   ```json
   {
     "version": "0.2.0",
     "configurations": [
       {
         "type": "lldb",
         "request": "launch",
         "name": "Debug API Server",
         "cargo": {
           "args": ["build", "--bin=sctv-api", "--package=sctv-api"]
         },
         "args": [],
         "cwd": "${workspaceFolder}",
         "env": {
           "DATABASE_URL": "postgres://sctv:sctv@localhost:5432/sctv",
           "RUST_LOG": "debug"
         }
       },
       {
         "type": "lldb",
         "request": "launch",
         "name": "Debug Worker",
         "cargo": {
           "args": ["build", "--bin=sctv-worker", "--package=sctv-worker"]
         },
         "args": [],
         "cwd": "${workspaceFolder}",
         "env": {
           "DATABASE_URL": "postgres://sctv:sctv@localhost:5432/sctv",
           "RUST_LOG": "debug"
         }
       },
       {
         "type": "lldb",
         "request": "launch",
         "name": "Debug Tests",
         "cargo": {
           "args": ["test", "--no-run", "--workspace"]
         },
         "args": [],
         "cwd": "${workspaceFolder}"
       }
     ]
   }
   ```

### IntelliJ IDEA / CLion

1. **Install Plugin**: Rust plugin from JetBrains

2. **Configure Toolchain**:
   - Go to Settings → Languages & Frameworks → Rust
   - Set Standard library: `~/.rustup/toolchains/stable-x86_64-unknown-linux-gnu`
   - Enable external linter: Clippy

3. **Run Configurations**:
   - Create separate run configurations for `sctv-api` and `sctv-worker`
   - Set environment variables in each configuration
   - Add `DATABASE_URL` and `RUST_LOG` variables

4. **Enable Format on Save**:
   - Settings → Tools → Actions on Save
   - Enable "Reformat code" for Rust files

## Running Services Locally

### API Server

```bash
# Development mode (with debug logging)
RUST_LOG=debug cargo run --bin sctv-api

# With custom configuration
SCTV_HOST=127.0.0.1 \
SCTV_PORT=3000 \
SCTV_JWT_SECRET=dev-secret \
cargo run --bin sctv-api
```

Access the API:
- Health check: http://localhost:3000/health
- GraphQL endpoint (POST only): http://localhost:3000/graphql
- REST API: http://localhost:3000/api/v1

### Worker

```bash
# Development mode
RUST_LOG=debug cargo run --bin sctv-worker

# With custom worker count
SCTV_WORKER_COUNT=2 cargo run --bin sctv-worker
```

### CLI

```bash
# Show help
cargo run --bin sctv -- --help

# Run a scan
cargo run --bin sctv -- scan --project <project-id>

# Upload an SBOM
cargo run --bin sctv -- upload sbom --file sbom.json
```

## Database Setup and Migrations

### Creating Migrations

```bash
# Create a new migration
sqlx migrate add create_users_table

# This creates: migrations/<timestamp>_create_users_table.sql
```

### Running Migrations

```bash
# Run all pending migrations
sqlx migrate run

# Run migrations in Docker
docker compose exec api sqlx migrate run

# Or via Makefile
make db-migrate
```

### Rolling Back Migrations

```bash
# Revert the last migration
sqlx migrate revert
```

### Database Reset (DESTRUCTIVE)

```bash
# WARNING: This destroys all data!
make db-reset

# Or manually:
docker compose down -v
docker compose up -d postgres
sleep 5
sqlx migrate run
```

### Accessing the Database

```bash
# Using Docker
docker compose exec postgres psql -U sctv -d sctv

# Native PostgreSQL
psql -U sctv -d sctv

# Common queries
\dt         # List tables
\d+ users   # Describe users table
SELECT * FROM tenants LIMIT 10;
```

## Hot Reload Setup

For faster development iterations, use `cargo-watch`:

### Install cargo-watch

```bash
cargo install cargo-watch
```

### Watch and Auto-Restart API

```bash
# Watch and restart on file changes
cargo watch -x 'run --bin sctv-api'

# With clear screen
cargo watch -c -x 'run --bin sctv-api'

# Watch specific files only
cargo watch -w crates/sctv-api -w crates/sctv-core -x 'run --bin sctv-api'
```

### Watch and Run Tests

```bash
# Run tests on file changes
cargo watch -x test

# Run specific test
cargo watch -x 'test test_create_user'

# Run tests for specific crate
cargo watch -x 'test -p sctv-core'
```

### Watch with Linting

```bash
# Run check, clippy, and tests on changes
cargo watch -x check -x 'clippy -- -D warnings' -x test
```

## Debugging Tips

### Enable Debug Logging

Set the `RUST_LOG` environment variable:

```bash
# All debug logs
export RUST_LOG=debug

# Specific modules
export RUST_LOG=sctv_api=debug,sctv_db=trace,sqlx=info

# With tower_http for HTTP request logging
export RUST_LOG=info,sctv_api=debug,tower_http=debug
```

### Using rust-lldb (LLDB)

```bash
# Build with debug symbols
cargo build --bin sctv-api

# Start debugger
rust-lldb target/debug/sctv-api

# Set breakpoints
(lldb) b sctv_api::main
(lldb) b crates/sctv-api/src/main.rs:23

# Run
(lldb) run

# Step through
(lldb) n  # next line
(lldb) s  # step into
(lldb) c  # continue

# Inspect variables
(lldb) p variable_name
(lldb) fr v  # frame variables
```

### Using VS Code Debugger

1. Set breakpoints in the editor (click left of line numbers)
2. Press F5 or select "Debug API Server" from the debug panel
3. Use the debug toolbar to step through code

### Database Query Debugging

Enable SQLx query logging:

```bash
# Log all queries
export RUST_LOG=sqlx=debug

# Prettify query logs
export SQLX_LOGGING=true
```

### Common Issues and Solutions

#### Issue: "Database connection refused"

**Solution**:
```bash
# Check if PostgreSQL is running
docker compose ps postgres
# Or check native PostgreSQL
pg_isready -h localhost -p 5432
```

#### Issue: "Migration error: already applied"

**Solution**:
```bash
# Check migration status
sqlx migrate info

# Force revert if needed (be careful!)
sqlx migrate revert
```

#### Issue: "Cannot find -lpq"

**Solution** (Linux):
```bash
# Install PostgreSQL development libraries
sudo apt-get install libpq-dev
```

#### Issue: "Too many open files"

**Solution** (Linux/macOS):
```bash
# Increase file descriptor limit
ulimit -n 10000
```

#### Issue: Slow compilation

**Solutions**:
```bash
# Use faster linker (Linux)
sudo apt-get install lld clang
export RUSTFLAGS="-C link-arg=-fuse-ld=lld"

# Use sccache for caching
cargo install sccache
export RUSTC_WRAPPER=sccache

# Reduce optimization for debug builds (in .cargo/config.toml)
[profile.dev]
opt-level = 0
```

### Performance Profiling

```bash
# Install flamegraph
cargo install flamegraph

# Profile the API server
cargo flamegraph --bin sctv-api

# This generates flamegraph.svg
```

### Testing with curl

```bash
# Health check
curl http://localhost:3000/health

# GraphQL query
curl -X POST http://localhost:3000/graphql \
  -H "Content-Type: application/json" \
  -d '{"query": "{ health }"}'

# REST API with authentication
curl -X GET http://localhost:3000/api/v1/projects \
  -H "Authorization: Bearer YOUR_JWT_TOKEN"
```

## Next Steps

Once your environment is set up:

1. Read the [Code Standards](./code-standards.md) document
2. Review the [Testing Guide](./testing.md)
3. Check out [Architecture Documentation](../architecture/overview.md)
4. See [Contributing Guidelines](./contributing.md) before making changes

## Getting Help

- Check the [Troubleshooting section](../troubleshooting/common-issues.md)
- Review GitHub Issues for similar problems
- Ask in team chat or open a discussion

Happy coding!
