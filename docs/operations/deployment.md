# Deployment Guide

**Version:** 0.1.0

Complete guide for deploying SCTV to production environments.

---

## Table of Contents

- [Deployment Options](#deployment-options)
- [Prerequisites](#prerequisites)
- [Docker Deployment](#docker-deployment)
- [Kubernetes Deployment](#kubernetes-deployment)
- [Cloud Deployments](#cloud-deployments)
- [Configuration](#configuration)
- [Database Setup](#database-setup)
- [Load Balancing](#load-balancing)
- [TLS/SSL Setup](#tlsssl-setup)
- [High Availability](#high-availability)
- [Monitoring Setup](#monitoring-setup)
- [Backup Strategy](#backup-strategy)
- [Disaster Recovery](#disaster-recovery)

---

## Deployment Options

### Comparison

| Option | Best For | Complexity | Scalability | Cost |
|--------|----------|------------|-------------|------|
| Docker Compose | Small teams, development | Low | Limited | Low |
| Kubernetes | Production, enterprise | High | Excellent | Medium-High |
| Managed Cloud | Quick start, auto-scaling | Medium | Excellent | High |
| Bare Metal | Full control, compliance | High | Manual | Medium |

### Recommended Architecture

**Small Team (< 50 projects):**
- Single server with Docker Compose
- Managed PostgreSQL (RDS, Cloud SQL)
- 2-4 CPU cores, 8 GB RAM

**Medium Team (50-500 projects):**
- Kubernetes cluster (3+ nodes)
- Managed PostgreSQL with read replicas
- Load balancer with auto-scaling
- 4-8 CPU cores, 16-32 GB RAM

**Enterprise (500+ projects):**
- Multi-zone Kubernetes cluster
- PostgreSQL cluster with high availability
- Multi-region deployment
- CDN for static assets
- Dedicated monitoring stack

---

## Prerequisites

### Infrastructure

**Compute:**
- 2+ CPU cores (4+ recommended)
- 4 GB RAM minimum (8+ GB recommended)
- 50 GB disk space (SSD recommended)

**Network:**
- Static IP or domain name
- HTTPS (TLS 1.2+)
- Firewall rules configured
- Load balancer (optional, recommended)

**Database:**
- PostgreSQL 14+ (managed service recommended)
- 20 GB storage minimum
- Automatic backups enabled
- Connection pooling configured

**Credentials:**
- TLS certificates (Let's Encrypt or CA)
- Database credentials
- Registry credentials (if using private images)
- Cloud provider credentials (if applicable)

---

## Docker Deployment

### Production Docker Compose

Create `docker-compose.prod.yml`:

```yaml
version: '3.8'

services:
  # PostgreSQL Database
  postgres:
    image: postgres:14-alpine
    restart: always
    environment:
      POSTGRES_DB: sctv
      POSTGRES_USER: ${POSTGRES_USER}
      POSTGRES_PASSWORD: ${POSTGRES_PASSWORD}
    volumes:
      - postgres_data:/var/lib/postgresql/data
      - ./backups:/backups
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U ${POSTGRES_USER}"]
      interval: 10s
      timeout: 5s
      retries: 5
    networks:
      - sctv-internal

  # API Server
  api:
    image: ghcr.io/example/sctv-api:latest
    restart: always
    depends_on:
      postgres:
        condition: service_healthy
    environment:
      DATABASE_URL: postgresql://${POSTGRES_USER}:${POSTGRES_PASSWORD}@postgres:5432/sctv
      JWT_SECRET: ${JWT_SECRET}
      API_BIND_ADDR: 0.0.0.0:3000
      LOG_LEVEL: info
      LOG_FORMAT: json
      ENABLE_CORS: "false"
      ENABLE_GRAPHQL_PLAYGROUND: "false"
    ports:
      - "3000:3000"
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:3000/health"]
      interval: 30s
      timeout: 10s
      retries: 3
    networks:
      - sctv-internal
      - sctv-public
    deploy:
      resources:
        limits:
          cpus: '2'
          memory: 4G
        reservations:
          cpus: '1'
          memory: 2G

  # Worker Service
  worker:
    image: ghcr.io/example/sctv-worker:latest
    restart: always
    depends_on:
      postgres:
        condition: service_healthy
    environment:
      DATABASE_URL: postgresql://${POSTGRES_USER}:${POSTGRES_PASSWORD}@postgres:5432/sctv
      WORKER_POOL_SIZE: 4
      LOG_LEVEL: info
      LOG_FORMAT: json
    networks:
      - sctv-internal
    deploy:
      replicas: 2
      resources:
        limits:
          cpus: '2'
          memory: 4G
        reservations:
          cpus: '1'
          memory: 2G

  # Dashboard
  dashboard:
    image: ghcr.io/example/sctv-dashboard:latest
    restart: always
    depends_on:
      - api
    environment:
      API_URL: http://api:3000
    ports:
      - "3001:3000"
    networks:
      - sctv-internal
      - sctv-public

  # nginx Reverse Proxy
  nginx:
    image: nginx:alpine
    restart: always
    depends_on:
      - api
      - dashboard
    ports:
      - "80:80"
      - "443:443"
    volumes:
      - ./nginx.conf:/etc/nginx/nginx.conf:ro
      - ./certs:/etc/nginx/certs:ro
    networks:
      - sctv-public
    healthcheck:
      test: ["CMD", "wget", "-q", "--spider", "http://localhost/health"]
      interval: 30s
      timeout: 10s
      retries: 3

volumes:
  postgres_data:
    driver: local

networks:
  sctv-internal:
    driver: bridge
  sctv-public:
    driver: bridge
```

### nginx Configuration

Create `nginx.conf`:

```nginx
events {
    worker_connections 1024;
}

http {
    # Security headers
    add_header X-Frame-Options "DENY" always;
    add_header X-Content-Type-Options "nosniff" always;
    add_header X-XSS-Protection "1; mode=block" always;
    add_header Strict-Transport-Security "max-age=31536000; includeSubDomains" always;

    # Rate limiting
    limit_req_zone $binary_remote_addr zone=api_limit:10m rate=10r/s;
    limit_req_zone $binary_remote_addr zone=auth_limit:10m rate=5r/m;

    # Upstream servers
    upstream api_backend {
        least_conn;
        server api:3000 max_fails=3 fail_timeout=30s;
    }

    upstream dashboard_backend {
        least_conn;
        server dashboard:3000 max_fails=3 fail_timeout=30s;
    }

    # HTTP -> HTTPS redirect
    server {
        listen 80;
        server_name sctv.example.com;
        return 301 https://$server_name$request_uri;
    }

    # HTTPS server
    server {
        listen 443 ssl http2;
        server_name sctv.example.com;

        # TLS configuration
        ssl_certificate /etc/nginx/certs/fullchain.pem;
        ssl_certificate_key /etc/nginx/certs/privkey.pem;
        ssl_protocols TLSv1.2 TLSv1.3;
        ssl_ciphers 'ECDHE-ECDSA-AES128-GCM-SHA256:ECDHE-RSA-AES128-GCM-SHA256';
        ssl_prefer_server_ciphers on;
        ssl_session_cache shared:SSL:10m;
        ssl_session_timeout 10m;

        # API endpoints
        location /api/ {
            limit_req zone=api_limit burst=20 nodelay;

            proxy_pass http://api_backend;
            proxy_http_version 1.1;
            proxy_set_header Host $host;
            proxy_set_header X-Real-IP $remote_addr;
            proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
            proxy_set_header X-Forwarded-Proto $scheme;
            proxy_connect_timeout 60s;
            proxy_send_timeout 60s;
            proxy_read_timeout 60s;
        }

        # GraphQL endpoint
        location /graphql {
            limit_req zone=api_limit burst=20 nodelay;

            proxy_pass http://api_backend;
            proxy_http_version 1.1;
            proxy_set_header Upgrade $http_upgrade;
            proxy_set_header Connection "upgrade";
            proxy_set_header Host $host;
            proxy_set_header X-Real-IP $remote_addr;
            proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        }

        # Authentication (stricter rate limit)
        location /api/v1/auth/ {
            limit_req zone=auth_limit burst=5 nodelay;
            proxy_pass http://api_backend;
        }

        # Health check (no rate limit)
        location /health {
            access_log off;
            proxy_pass http://api_backend;
        }

        # Dashboard
        location / {
            proxy_pass http://dashboard_backend;
            proxy_http_version 1.1;
            proxy_set_header Host $host;
            proxy_set_header X-Real-IP $remote_addr;
            proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        }
    }
}
```

### Deployment Steps

```bash
# 1. Create environment file
cat > .env << EOF
POSTGRES_USER=sctv
POSTGRES_PASSWORD=$(openssl rand -hex 32)
JWT_SECRET=$(openssl rand -hex 32)
EOF

# 2. Set permissions
chmod 600 .env

# 3. Pull images
docker-compose -f docker-compose.prod.yml pull

# 4. Start services
docker-compose -f docker-compose.prod.yml up -d

# 5. Run migrations
docker-compose -f docker-compose.prod.yml exec api sctv-api migrate

# 6. Create admin user
docker-compose -f docker-compose.prod.yml exec api sctv-cli user create \
  --email admin@example.com \
  --role admin

# 7. Check status
docker-compose -f docker-compose.prod.yml ps
docker-compose -f docker-compose.prod.yml logs -f
```

---

## Kubernetes Deployment

### Helm Installation

```bash
# Add Helm repository
helm repo add sctv https://charts.sctv.example.com
helm repo update

# Create namespace
kubectl create namespace sctv

# Create secrets
kubectl create secret generic sctv-secrets \
  --from-literal=postgres-password=$(openssl rand -hex 32) \
  --from-literal=jwt-secret=$(openssl rand -hex 32) \
  --namespace sctv

# Install with Helm
helm install sctv sctv/supply-chain-trust-verifier \
  --namespace sctv \
  --values values.yaml \
  --wait
```

### Custom Values (values.yaml)

```yaml
# Global settings
global:
  image:
    registry: ghcr.io
    repository: example/sctv
    tag: "0.1.0"
    pullPolicy: IfNotPresent

# API server
api:
  replicaCount: 3

  image:
    repository: ghcr.io/example/sctv-api

  resources:
    requests:
      cpu: 500m
      memory: 1Gi
    limits:
      cpu: 2000m
      memory: 4Gi

  autoscaling:
    enabled: true
    minReplicas: 3
    maxReplicas: 10
    targetCPUUtilizationPercentage: 70

  service:
    type: ClusterIP
    port: 3000

  env:
    LOG_LEVEL: "info"
    LOG_FORMAT: "json"
    ENABLE_CORS: "false"

# Worker service
worker:
  replicaCount: 4

  image:
    repository: ghcr.io/example/sctv-worker

  resources:
    requests:
      cpu: 500m
      memory: 1Gi
    limits:
      cpu: 2000m
      memory: 4Gi

  autoscaling:
    enabled: true
    minReplicas: 4
    maxReplicas: 20
    targetCPUUtilizationPercentage: 80

  env:
    WORKER_POOL_SIZE: "8"
    JOB_MAX_RETRIES: "3"

# Dashboard
dashboard:
  replicaCount: 2

  image:
    repository: ghcr.io/example/sctv-dashboard

  resources:
    requests:
      cpu: 100m
      memory: 256Mi
    limits:
      cpu: 500m
      memory: 1Gi

# PostgreSQL (using Bitnami chart)
postgresql:
  enabled: true
  auth:
    username: sctv
    database: sctv
    existingSecret: sctv-secrets
    secretKeys:
      adminPasswordKey: postgres-password
      userPasswordKey: postgres-password

  primary:
    persistence:
      enabled: true
      size: 100Gi
      storageClass: "fast-ssd"

    resources:
      requests:
        cpu: 1000m
        memory: 2Gi
      limits:
        cpu: 4000m
        memory: 8Gi

    podAntiAffinity:
      preferredDuringSchedulingIgnoredDuringExecution:
        - weight: 100
          podAffinityTerm:
            topologyKey: kubernetes.io/hostname

  readReplicas:
    replicaCount: 2
    persistence:
      enabled: true
      size: 100Gi
    resources:
      requests:
        cpu: 500m
        memory: 1Gi
      limits:
        cpu: 2000m
        memory: 4Gi

# Ingress
ingress:
  enabled: true
  className: nginx
  annotations:
    cert-manager.io/cluster-issuer: letsencrypt-prod
    nginx.ingress.kubernetes.io/rate-limit: "100"
    nginx.ingress.kubernetes.io/ssl-redirect: "true"

  hosts:
    - host: sctv.example.com
      paths:
        - path: /
          pathType: Prefix
          backend:
            service:
              name: sctv-api
              port: 3000

  tls:
    - secretName: sctv-tls
      hosts:
        - sctv.example.com

# Service Monitor (Prometheus)
serviceMonitor:
  enabled: true
  interval: 30s
  scrapeTimeout: 10s

# Pod Disruption Budget
podDisruptionBudget:
  enabled: true
  minAvailable: 1

# Network Policies
networkPolicy:
  enabled: true
  ingress:
    - from:
        - namespaceSelector:
            matchLabels:
              name: ingress-nginx
```

### Deployment Steps

```bash
# 1. Verify cluster access
kubectl cluster-info

# 2. Create namespace and secrets
kubectl create namespace sctv
kubectl create secret generic sctv-secrets \
  --from-literal=postgres-password=$(openssl rand -hex 32) \
  --from-literal=jwt-secret=$(openssl rand -hex 32) \
  --namespace sctv

# 3. Install cert-manager (if not already installed)
kubectl apply -f https://github.com/cert-manager/cert-manager/releases/download/v1.13.0/cert-manager.yaml

# 4. Create ClusterIssuer for Let's Encrypt
cat <<EOF | kubectl apply -f -
apiVersion: cert-manager.io/v1
kind: ClusterIssuer
metadata:
  name: letsencrypt-prod
spec:
  acme:
    server: https://acme-v02.api.letsencrypt.org/directory
    email: admin@example.com
    privateKeySecretRef:
      name: letsencrypt-prod
    solvers:
    - http01:
        ingress:
          class: nginx
EOF

# 5. Install SCTV
helm install sctv sctv/supply-chain-trust-verifier \
  --namespace sctv \
  --values values.yaml \
  --wait \
  --timeout 10m

# 6. Run migrations
kubectl exec -n sctv deployment/sctv-api -- sctv-api migrate

# 7. Verify deployment
kubectl get pods -n sctv
kubectl get ingress -n sctv
kubectl logs -n sctv deployment/sctv-api -f

# 8. Create admin user
kubectl exec -n sctv deployment/sctv-api -- \
  sctv-cli user create \
    --email admin@example.com \
    --role admin
```

---

## Cloud Deployments

### AWS ECS/Fargate

```bash
# Install AWS CLI and ecs-cli
pip install awscli
ecs-cli --version

# Configure cluster
ecs-cli configure --cluster sctv-prod \
  --region us-east-1 \
  --default-launch-type FARGATE

# Create cluster
ecs-cli up --cluster-config sctv-prod \
  --ecs-profile sctv-profile

# Deploy with docker-compose
ecs-cli compose --file docker-compose.aws.yml \
  --project-name sctv \
  service up
```

### Google Cloud Run

```bash
# Build and push images
gcloud builds submit --tag gcr.io/PROJECT_ID/sctv-api
gcloud builds submit --tag gcr.io/PROJECT_ID/sctv-worker

# Deploy API
gcloud run deploy sctv-api \
  --image gcr.io/PROJECT_ID/sctv-api \
  --platform managed \
  --region us-central1 \
  --memory 2Gi \
  --cpu 2 \
  --max-instances 10 \
  --set-env-vars DATABASE_URL=postgres://... \
  --set-env-vars JWT_SECRET=...

# Deploy Worker
gcloud run deploy sctv-worker \
  --image gcr.io/PROJECT_ID/sctv-worker \
  --platform managed \
  --region us-central1 \
  --memory 2Gi \
  --cpu 2 \
  --min-instances 2 \
  --max-instances 20 \
  --set-env-vars DATABASE_URL=postgres://...
```

### Azure Container Instances

```bash
# Create resource group
az group create --name sctv-rg --location eastus

# Create container group
az container create \
  --resource-group sctv-rg \
  --name sctv \
  --image ghcr.io/example/sctv-api:latest \
  --dns-name-label sctv-prod \
  --ports 3000 \
  --cpu 2 \
  --memory 4 \
  --environment-variables \
    DATABASE_URL=postgres://... \
    JWT_SECRET=...
```

---

## High Availability

### Multi-Region Deployment

```
Region 1 (Primary):
  - API Servers (3 replicas)
  - Workers (4 replicas)
  - PostgreSQL Primary

Region 2 (Standby):
  - API Servers (3 replicas)
  - Workers (4 replicas)
  - PostgreSQL Replica

Global Load Balancer:
  - Route to nearest region
  - Health checks
  - Automatic failover
```

### Database Replication

```sql
-- On primary
CREATE PUBLICATION sctv_pub FOR ALL TABLES;

-- On replica
CREATE SUBSCRIPTION sctv_sub
CONNECTION 'host=primary.db port=5432 dbname=sctv user=replication password=...'
PUBLICATION sctv_pub;
```

### Health Checks

Configure comprehensive health checks:

```yaml
# Kubernetes liveness probe
livenessProbe:
  httpGet:
    path: /health
    port: 3000
  initialDelaySeconds: 30
  periodSeconds: 10
  timeoutSeconds: 5
  failureThreshold: 3

# Kubernetes readiness probe
readinessProbe:
  httpGet:
    path: /health/ready
    port: 3000
  initialDelaySeconds: 10
  periodSeconds: 5
  timeoutSeconds: 3
  failureThreshold: 2
```

---

## Monitoring Setup

See [Monitoring Guide](monitoring.md) for complete setup instructions.

**Quick setup:**

```bash
# Install Prometheus and Grafana
helm repo add prometheus-community https://prometheus-community.github.io/helm-charts
helm install prometheus prometheus-community/kube-prometheus-stack \
  --namespace monitoring \
  --create-namespace

# Import SCTV dashboards
kubectl apply -f monitoring/grafana-dashboards.yaml
```

---

## Next Steps

- [Monitoring](monitoring.md) - Set up observability
- [Security Hardening](security.md) - Production security
- [Troubleshooting](troubleshooting.md) - Common issues
- [Backup and Recovery](backup.md) - Data protection

---

**Ready for production!** 🚀 Continue with [Security Hardening](security.md) to ensure your deployment is secure.
