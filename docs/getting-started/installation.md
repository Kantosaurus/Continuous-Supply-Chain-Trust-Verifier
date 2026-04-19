# Installation Guide

**Version:** 0.1.0

This guide walks you through installing and setting up Supply Chain Trust Verifier (SCTV) in your environment.

---

## Table of Contents

- [Prerequisites](#prerequisites)
- [Installation Methods](#installation-methods)
- [Docker Installation (Recommended)](#docker-installation-recommended)
- [Native Installation](#native-installation)
- [Kubernetes Installation](#kubernetes-installation)
- [Database Setup](#database-setup)
- [Verification](#verification)
- [Next Steps](#next-steps)

---

## Prerequisites

### System Requirements

**Minimum:**
- CPU: 2 cores
- RAM: 4 GB
- Disk: 20 GB
- OS: Linux (Ubuntu 20.04+, Debian 11+, RHEL 8+), macOS 11+, Windows 10+

**Recommended for Production:**
- CPU: 4+ cores
- RAM: 8+ GB
- Disk: 100+ GB SSD
- OS: Linux (Ubuntu 22.04 LTS)

### Software Dependencies

**Required:**
- PostgreSQL 14+ (for database)
- Docker 20+ and Docker Compose 2+ (for containerized deployment)

**Optional:**
- Rust 1.75+ (for native builds)
- Node.js 18+ (for dashboard development)

---

## Installation Methods

SCTV can be installed using:

1. **Docker/Docker Compose** - Recommended for most users
2. **Native binaries** - For custom deployments
3. **Kubernetes** - For production clusters
4. **From source** - For development

---

## Docker Installation (Recommended)

Docker Compose provides the fastest way to get SCTV running.

### Step 1: Clone the Repository

```bash
git clone https://github.com/example/supply-chain-trust-verifier.git
cd supply-chain-trust-verifier
```

### Step 2: Configure Environment

Create a `.env` file:

```bash
cp .env.example .env
```

Edit `.env` with your configuration:

```env
# Database Configuration
POSTGRES_HOST=postgres
POSTGRES_PORT=5432
POSTGRES_DB=sctv
POSTGRES_USER=sctv
POSTGRES_PASSWORD=your-secure-password-here

# API Configuration
API_BIND_ADDR=0.0.0.0:3000
JWT_SECRET=your-jwt-secret-change-in-production
ENABLE_CORS=true
ENABLE_GRAPHQL_PLAYGROUND=true

# Worker Configuration
WORKER_POOL_SIZE=4
JOB_MAX_RETRIES=3

# Notification Configuration (Optional)
SLACK_WEBHOOK_URL=
EMAIL_SMTP_HOST=
EMAIL_SMTP_PORT=587
EMAIL_FROM_ADDRESS=alerts@example.com

# Monitoring (Optional)
LOG_LEVEL=info
ENABLE_METRICS=true
METRICS_PORT=9090
```

### Step 3: Start Services

```bash
docker-compose up -d
```

This starts:
- PostgreSQL database
- SCTV API server
- SCTV worker pool
- SCTV dashboard (web UI)

### Step 4: Run Database Migrations

```bash
docker-compose exec api sctv-api migrate
```

### Step 5: Create Your First Tenant

```bash
docker-compose exec api sctv-cli tenant create \
  --name "My Organization" \
  --slug "my-org" \
  --plan enterprise
```

### Step 6: Access the Dashboard

Open your browser to:

```
http://localhost:3000
```

**Default credentials** (change immediately):
- Username: `admin@example.com`
- Password: `admin`

---

## Native Installation

For custom deployments without Docker.

### Step 1: Install Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

### Step 2: Clone and Build

```bash
git clone https://github.com/example/supply-chain-trust-verifier.git
cd supply-chain-trust-verifier

# Build release binaries
cargo build --release

# Binaries will be in target/release/
# - sctv-api (API server)
# - sctv-worker (background worker)
# - sctv-cli (command-line tool)
```

### Step 3: Install Binaries

```bash
sudo cp target/release/sctv-api /usr/local/bin/
sudo cp target/release/sctv-worker /usr/local/bin/
sudo cp target/release/sctv-cli /usr/local/bin/

# Verify installation
sctv-cli --version
```

### Step 4: Create Configuration Directory

```bash
sudo mkdir -p /etc/sctv
sudo chown $USER:$USER /etc/sctv
```

Create `/etc/sctv/config.toml`:

```toml
[database]
host = "localhost"
port = 5432
database = "sctv"
username = "sctv"
password = "your-password"
max_connections = 20

[api]
bind_addr = "0.0.0.0:3000"
jwt_secret = "your-jwt-secret"
enable_cors = true

[worker]
pool_size = 4
poll_interval_seconds = 5
max_retries = 3

[logging]
level = "info"
format = "json"

[metrics]
enabled = true
bind_addr = "0.0.0.0:9090"
```

### Step 5: Create Systemd Services

Create `/etc/systemd/system/sctv-api.service`:

```ini
[Unit]
Description=SCTV API Server
After=network.target postgresql.service
Requires=postgresql.service

[Service]
Type=simple
User=sctv
Group=sctv
ExecStart=/usr/local/bin/sctv-api --config /etc/sctv/config.toml
Restart=on-failure
RestartSec=5s

[Install]
WantedBy=multi-user.target
```

Create `/etc/systemd/system/sctv-worker.service`:

```ini
[Unit]
Description=SCTV Worker Service
After=network.target postgresql.service sctv-api.service
Requires=postgresql.service

[Service]
Type=simple
User=sctv
Group=sctv
ExecStart=/usr/local/bin/sctv-worker --config /etc/sctv/config.toml
Restart=on-failure
RestartSec=5s

[Install]
WantedBy=multi-user.target
```

### Step 6: Create Service User

```bash
sudo useradd -r -s /bin/false sctv
```

### Step 7: Enable and Start Services

```bash
sudo systemctl daemon-reload
sudo systemctl enable sctv-api sctv-worker
sudo systemctl start sctv-api sctv-worker

# Check status
sudo systemctl status sctv-api
sudo systemctl status sctv-worker
```

---

## Kubernetes Installation

For production deployments on Kubernetes.

### Prerequisites

- Kubernetes 1.24+ cluster
- `kubectl` configured
- Helm 3.0+ (optional but recommended)

### Using Helm (Recommended)

```bash
# Add the SCTV Helm repository
helm repo add sctv https://charts.sctv.example.com
helm repo update

# Install with default values
helm install sctv sctv/supply-chain-trust-verifier \
  --namespace sctv \
  --create-namespace

# Or with custom values
helm install sctv sctv/supply-chain-trust-verifier \
  --namespace sctv \
  --create-namespace \
  --values custom-values.yaml
```

Example `custom-values.yaml`:

```yaml
replicaCount: 3

image:
  repository: ghcr.io/example/sctv
  tag: "0.1.0"
  pullPolicy: IfNotPresent

api:
  resources:
    requests:
      memory: "512Mi"
      cpu: "500m"
    limits:
      memory: "1Gi"
      cpu: "1000m"

worker:
  replicas: 4
  resources:
    requests:
      memory: "512Mi"
      cpu: "500m"
    limits:
      memory: "1Gi"
      cpu: "1000m"

postgresql:
  enabled: true
  auth:
    username: sctv
    password: "your-secure-password"
    database: sctv
  primary:
    persistence:
      size: 100Gi

ingress:
  enabled: true
  className: nginx
  hosts:
    - host: sctv.example.com
      paths:
        - path: /
          pathType: Prefix
  tls:
    - secretName: sctv-tls
      hosts:
        - sctv.example.com

config:
  jwtSecret: "your-jwt-secret"
  logLevel: "info"

  notifications:
    slack:
      webhookUrl: "https://hooks.slack.com/services/xxx/yyy/zzz"
```

### Using kubectl (Manual)

Apply the Kubernetes manifests:

```bash
kubectl apply -f k8s/namespace.yaml
kubectl apply -f k8s/configmap.yaml
kubectl apply -f k8s/secret.yaml
kubectl apply -f k8s/postgres.yaml
kubectl apply -f k8s/api-deployment.yaml
kubectl apply -f k8s/worker-deployment.yaml
kubectl apply -f k8s/dashboard-deployment.yaml
kubectl apply -f k8s/service.yaml
kubectl apply -f k8s/ingress.yaml
```

---

## Database Setup

SCTV requires PostgreSQL 14 or later.

### Option 1: Docker PostgreSQL

Already included in `docker-compose.yml`.

### Option 2: Managed Database

Use a managed PostgreSQL service (AWS RDS, Google Cloud SQL, Azure Database):

1. Create a PostgreSQL 14+ instance
2. Enable the following extensions:
   ```sql
   CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
   CREATE EXTENSION IF NOT EXISTS "pg_trgm";
   ```
3. Create a database and user:
   ```sql
   CREATE DATABASE sctv;
   CREATE USER sctv WITH ENCRYPTED PASSWORD 'your-password';
   GRANT ALL PRIVILEGES ON DATABASE sctv TO sctv;
   ```
4. Update connection string in your configuration

### Option 3: Self-Hosted PostgreSQL

#### Ubuntu/Debian

```bash
# Install PostgreSQL 14
sudo apt update
sudo apt install -y postgresql-14 postgresql-contrib-14

# Start and enable service
sudo systemctl start postgresql
sudo systemctl enable postgresql

# Create database and user
sudo -u postgres psql << EOF
CREATE DATABASE sctv;
CREATE USER sctv WITH ENCRYPTED PASSWORD 'your-password';
GRANT ALL PRIVILEGES ON DATABASE sctv TO sctv;
\c sctv
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pg_trgm";
EOF
```

#### Configure PostgreSQL

Edit `/etc/postgresql/14/main/postgresql.conf`:

```conf
max_connections = 100
shared_buffers = 256MB
effective_cache_size = 1GB
maintenance_work_mem = 64MB
checkpoint_completion_target = 0.9
wal_buffers = 16MB
default_statistics_target = 100
random_page_cost = 1.1
effective_io_concurrency = 200
work_mem = 2621kB
min_wal_size = 1GB
max_wal_size = 4GB
```

Edit `/etc/postgresql/14/main/pg_hba.conf`:

```conf
# Allow SCTV to connect
host    sctv    sctv    127.0.0.1/32    scram-sha-256
```

Restart PostgreSQL:

```bash
sudo systemctl restart postgresql
```

### Run Migrations

After database setup, run migrations:

```bash
# Using CLI
sctv-cli migrate --config /etc/sctv/config.toml

# Or using Docker
docker-compose exec api sctv-api migrate

# Or manually with psql
psql -h localhost -U sctv -d sctv -f migrations/001_initial_schema.sql
```

---

## Verification

### Check API Health

```bash
curl http://localhost:3000/health
```

Expected response:

```json
{
  "status": "healthy",
  "version": "0.1.0",
  "timestamp": "2026-01-15T10:30:00Z"
}
```

### Check Worker Status

```bash
# View worker logs
docker-compose logs -f worker

# Or for systemd
sudo journalctl -u sctv-worker -f
```

### Check Database Connection

```bash
sctv-cli db check --config /etc/sctv/config.toml
```

### Run a Test Scan

```bash
sctv-cli scan \
  --path ./test-project \
  --ecosystem npm \
  --format json
```

---

## Next Steps

Now that SCTV is installed:

1. **Configure your first project** - See [Quick Start Guide](quickstart.md)
2. **Set up notifications** - See [Configuration Guide](configuration.md)
3. **Create security policies** - See [User Guide: Policies](../user-guide/policies.md)
4. **Integrate with CI/CD** - See [Webhooks Documentation](../api/webhooks.md)

---

## Troubleshooting

### Database Connection Failed

**Issue:** SCTV can't connect to PostgreSQL

**Solutions:**
1. Verify PostgreSQL is running: `systemctl status postgresql`
2. Check credentials in configuration
3. Verify network connectivity: `psql -h HOST -U USER -d DATABASE`
4. Check firewall rules

### Port Already in Use

**Issue:** Port 3000 is already in use

**Solutions:**
1. Change the bind address in configuration
2. Stop the conflicting service
3. Use a different port: `API_BIND_ADDR=0.0.0.0:8080`

### Migrations Failed

**Issue:** Database migrations error

**Solutions:**
1. Check PostgreSQL logs: `journalctl -u postgresql`
2. Verify extensions are installed
3. Ensure database user has sufficient privileges
4. Run migrations manually to see detailed errors

### Worker Not Processing Jobs

**Issue:** Background jobs remain pending

**Solutions:**
1. Check worker logs for errors
2. Verify worker service is running
3. Check database connection from worker
4. Ensure sufficient resources (CPU, memory)

For more troubleshooting, see [Operations: Troubleshooting](../operations/troubleshooting.md).

---

## Support

Need help?

- **Documentation:** [SCTV Docs](../README.md)
- **Issues:** [GitHub Issues](https://github.com/example/supply-chain-trust-verifier/issues)
- **Community:** [Discussions](https://github.com/example/supply-chain-trust-verifier/discussions)
