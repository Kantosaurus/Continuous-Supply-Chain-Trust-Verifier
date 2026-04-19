# SCTV Docker Deployment Guide

This directory contains Docker configuration for deploying the Supply Chain Trust Verifier (SCTV) platform.

## Quick Start

```bash
# Start development environment
docker-compose up -d

# Or use Make
make dev
```

Access the services:
- **API**: http://localhost:3000
- **GraphQL endpoint** (POST only): http://localhost:3000/graphql
- **Health Check**: http://localhost:3000/health

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                        Docker Network                        │
├──────────────┬──────────────┬──────────────┬────────────────┤
│   postgres   │     api      │    worker    │   dashboard    │
│   :5432      │    :3000     │   (no port)  │    :8080       │
└──────────────┴──────────────┴──────────────┴────────────────┘
```

### Services

| Service | Description | Port |
|---------|-------------|------|
| `postgres` | PostgreSQL database with extensions | 5432 |
| `api` | REST/GraphQL API server | 3000 |
| `worker` | Background job processor | - |
| `dashboard` | Web UI (optional) | 8080 |

## Configuration

### Environment Variables

Copy `.env.example` to `.env` and configure:

```bash
cp .env.example .env
```

Key variables:

| Variable | Description | Default |
|----------|-------------|---------|
| `POSTGRES_PASSWORD` | Database password | (required) |
| `SCTV_JWT_SECRET` | JWT signing secret | (required) |
| `SCTV_WORKER_COUNT` | Number of worker threads | `4` |
| `API_PORT` | API server port | `3000` |

### Development vs Production

**Development** (default):
```bash
docker-compose up -d
```

**Production**:
```bash
docker-compose -f docker-compose.yml -f docker-compose.prod.yml up -d
```

## Building Images

### Build All Services
```bash
# Using Make
make build

# Or directly
docker build -f docker/api.Dockerfile -t sctv-api .
docker build -f docker/worker.Dockerfile -t sctv-worker .
```

### Multi-Target Build
The main `docker/Dockerfile` supports multiple targets:

```bash
# API server
docker build --target sctv-api -t sctv-api -f docker/Dockerfile .

# Worker
docker build --target sctv-worker -t sctv-worker -f docker/Dockerfile .

# CLI tool
docker build --target sctv-cli -t sctv-cli -f docker/Dockerfile .

# Dashboard
docker build --target sctv-dashboard -t sctv-dashboard -f docker/Dockerfile .
```

## Operations

### Scaling Workers
```bash
docker-compose up -d --scale worker=3
```

### Viewing Logs
```bash
# All services
docker-compose logs -f

# Specific service
docker-compose logs -f api
```

### Database Access
```bash
# Connect to PostgreSQL
docker-compose exec postgres psql -U sctv -d sctv
```

### Health Checks
```bash
curl http://localhost:3000/health
```

## Files

| File | Purpose |
|------|---------|
| `Dockerfile` | Multi-stage build for all services |
| `api.Dockerfile` | Optimized build for API server |
| `worker.Dockerfile` | Optimized build for worker |
| `init-db.sql` | Database initialization script |

## Troubleshooting

### Database Connection Issues
```bash
# Check if postgres is healthy
docker-compose ps postgres

# View postgres logs
docker-compose logs postgres
```

### API Won't Start
```bash
# Check if migrations ran
docker-compose logs api | grep -i migration

# Manually run migrations
docker-compose exec api sqlx migrate run
```

### Worker Not Processing Jobs
```bash
# Check worker status
docker-compose logs worker

# Verify database connection
docker-compose exec worker env | grep DATABASE_URL
```

## Security Notes

1. **Never use default passwords in production**
2. **Set `SCTV_JWT_SECRET` to a strong random value**
3. **Disable CORS in production** (`SCTV_ENABLE_CORS=false`) unless the API is fronted by a gateway that handles CORS policy
4. **Use Docker secrets for sensitive values in Swarm/Kubernetes**
5. **Run containers as non-root user** (already configured)

## Resource Requirements

Minimum recommended resources:

| Service | CPU | Memory |
|---------|-----|--------|
| postgres | 0.5 | 512MB |
| api | 0.25 | 128MB |
| worker | 0.5 | 256MB |
| dashboard | 0.25 | 128MB |

Production recommendations:

| Service | CPU | Memory |
|---------|-----|--------|
| postgres | 2 | 2GB |
| api (x2) | 1 | 512MB |
| worker | 2 | 1GB |
| dashboard | 0.5 | 256MB |
