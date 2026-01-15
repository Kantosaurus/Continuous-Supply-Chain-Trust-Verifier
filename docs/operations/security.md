# Security Hardening Guide

**Version:** 0.1.0

Comprehensive security hardening guide for production SCTV deployments.

---

## Table of Contents

- [Overview](#overview)
- [Production Security Checklist](#production-security-checklist)
- [TLS Configuration](#tls-configuration)
- [Network Security](#network-security)
- [Database Security](#database-security)
- [API Security Headers](#api-security-headers)
- [Rate Limiting](#rate-limiting)
- [Secret Management](#secret-management)
- [Audit Logging](#audit-logging)
- [Vulnerability Scanning](#vulnerability-scanning)
- [Incident Response](#incident-response)
- [Compliance](#compliance)

---

## Overview

Security is a critical aspect of any production deployment. This guide provides comprehensive security hardening recommendations for SCTV.

### Security Principles

1. **Defense in Depth:** Multiple layers of security controls
2. **Least Privilege:** Minimal permissions required
3. **Secure by Default:** Security features enabled out-of-the-box
4. **Zero Trust:** Verify all access explicitly
5. **Audit Everything:** Complete audit trail for compliance

### Threat Model

SCTV must protect against:

- **External attacks:** Unauthorized API access, DDoS, injection attacks
- **Supply chain attacks:** Compromised dependencies (ironic!)
- **Data breaches:** Unauthorized access to tenant data
- **Insider threats:** Malicious or compromised operators
- **Infrastructure compromise:** Container escape, privilege escalation

---

## Production Security Checklist

Use this checklist before going to production:

### Authentication & Authorization

- [ ] Strong JWT secrets configured (32+ random bytes)
- [ ] JWT tokens expire within 8 hours
- [ ] API keys use secure random generation
- [ ] Password requirements enforced (12+ chars, complexity)
- [ ] Multi-factor authentication (MFA) enabled for admins
- [ ] Role-based access control (RBAC) properly configured
- [ ] Service accounts use minimal permissions

### Network Security

- [ ] TLS 1.2+ enabled, TLS 1.0/1.1 disabled
- [ ] Valid TLS certificates from trusted CA
- [ ] HTTPS enforced (HTTP redirects to HTTPS)
- [ ] Firewall rules restrict unnecessary ports
- [ ] Services isolated in private networks
- [ ] API gateway/load balancer configured
- [ ] DDoS protection enabled

### Database Security

- [ ] PostgreSQL password authentication enabled
- [ ] Strong database passwords (32+ random chars)
- [ ] Row-level security (RLS) enabled
- [ ] Database backups encrypted
- [ ] Database access restricted to application only
- [ ] SSL/TLS for database connections
- [ ] Regular security updates applied

### Application Security

- [ ] All secrets stored in secure secret management
- [ ] Environment variables sanitized (no secrets in logs)
- [ ] Input validation on all API endpoints
- [ ] Output encoding to prevent XSS
- [ ] SQL injection protection (parameterized queries)
- [ ] CSRF protection enabled
- [ ] Security headers configured
- [ ] Rate limiting enabled
- [ ] CORS properly configured

### Monitoring & Logging

- [ ] Audit logging enabled for all critical operations
- [ ] Log aggregation configured
- [ ] Security alerts configured
- [ ] Failed authentication attempts monitored
- [ ] Anomaly detection enabled
- [ ] Logs retained for compliance period (90+ days)

### Container Security

- [ ] Containers run as non-root user
- [ ] Minimal base images (Alpine/Distroless)
- [ ] No unnecessary packages installed
- [ ] Container images scanned for vulnerabilities
- [ ] Image signing and verification enabled
- [ ] Resource limits configured
- [ ] Read-only root filesystem where possible

### Compliance

- [ ] Privacy policy published
- [ ] Data retention policies configured
- [ ] GDPR compliance measures implemented
- [ ] SOC 2 controls documented
- [ ] Incident response plan documented
- [ ] Security training completed

---

## TLS Configuration

### Generate TLS Certificates

#### Option 1: Let's Encrypt (Recommended)

```bash
# Install certbot
sudo apt-get update
sudo apt-get install certbot

# Generate certificate
sudo certbot certonly --standalone \
  --preferred-challenges http \
  -d sctv.example.com \
  --email admin@example.com \
  --agree-tos

# Certificates stored in:
# /etc/letsencrypt/live/sctv.example.com/fullchain.pem
# /etc/letsencrypt/live/sctv.example.com/privkey.pem

# Auto-renewal
sudo certbot renew --dry-run
```

#### Option 2: Self-Signed (Development Only)

```bash
# Generate private key
openssl genrsa -out privkey.pem 4096

# Generate certificate signing request
openssl req -new -key privkey.pem -out csr.pem \
  -subj "/C=US/ST=State/L=City/O=Organization/CN=sctv.example.com"

# Generate self-signed certificate (valid 1 year)
openssl x509 -req -days 365 -in csr.pem \
  -signkey privkey.pem -out fullchain.pem

# Create directory and copy certificates
mkdir -p ./certs
cp fullchain.pem privkey.pem ./certs/
chmod 600 ./certs/privkey.pem
```

### nginx TLS Configuration

Create `nginx-tls.conf`:

```nginx
# Modern TLS configuration
ssl_protocols TLSv1.2 TLSv1.3;
ssl_ciphers 'ECDHE-ECDSA-AES128-GCM-SHA256:ECDHE-RSA-AES128-GCM-SHA256:ECDHE-ECDSA-AES256-GCM-SHA384:ECDHE-RSA-AES256-GCM-SHA384:ECDHE-ECDSA-CHACHA20-POLY1305:ECDHE-RSA-CHACHA20-POLY1305';
ssl_prefer_server_ciphers off;

# TLS session cache
ssl_session_cache shared:SSL:10m;
ssl_session_timeout 10m;
ssl_session_tickets off;

# OCSP stapling
ssl_stapling on;
ssl_stapling_verify on;
ssl_trusted_certificate /etc/nginx/certs/fullchain.pem;
resolver 8.8.8.8 8.8.4.4 valid=300s;
resolver_timeout 5s;

# Diffie-Hellman parameters
ssl_dhparam /etc/nginx/dhparam.pem;

# Security headers
add_header Strict-Transport-Security "max-age=63072000; includeSubDomains; preload" always;
add_header X-Frame-Options "DENY" always;
add_header X-Content-Type-Options "nosniff" always;
add_header X-XSS-Protection "1; mode=block" always;
add_header Referrer-Policy "strict-origin-when-cross-origin" always;
add_header Content-Security-Policy "default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; font-src 'self'; connect-src 'self'; frame-ancestors 'none';" always;
add_header Permissions-Policy "geolocation=(), microphone=(), camera=()" always;

# Server configuration
server {
    listen 80;
    server_name sctv.example.com;
    return 301 https://$server_name$request_uri;
}

server {
    listen 443 ssl http2;
    server_name sctv.example.com;

    # TLS certificates
    ssl_certificate /etc/nginx/certs/fullchain.pem;
    ssl_certificate_key /etc/nginx/certs/privkey.pem;

    # Security settings
    client_max_body_size 10M;
    client_body_timeout 60s;
    client_header_timeout 60s;

    # Logging
    access_log /var/log/nginx/sctv-access.log;
    error_log /var/log/nginx/sctv-error.log;

    # API proxy
    location /api/ {
        proxy_pass http://api:3000;
        proxy_http_version 1.1;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_connect_timeout 60s;
        proxy_send_timeout 60s;
        proxy_read_timeout 60s;

        # Security
        proxy_hide_header X-Powered-By;
    }

    # Dashboard
    location / {
        proxy_pass http://dashboard:3000;
        proxy_http_version 1.1;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}
```

### Generate Diffie-Hellman Parameters

```bash
# Generate strong DH parameters (takes several minutes)
openssl dhparam -out dhparam.pem 4096

# Copy to nginx
cp dhparam.pem ./nginx/dhparam.pem
```

### Test TLS Configuration

```bash
# Test SSL configuration
curl -I https://sctv.example.com

# Check certificate
openssl s_client -connect sctv.example.com:443 -showcerts

# Use SSL Labs test
# Visit: https://www.ssllabs.com/ssltest/analyze.html?d=sctv.example.com

# Check HSTS preload eligibility
# Visit: https://hstspreload.org/?domain=sctv.example.com
```

---

## Network Security

### Firewall Configuration

#### UFW (Ubuntu)

```bash
# Install UFW
sudo apt-get install ufw

# Default policies
sudo ufw default deny incoming
sudo ufw default allow outgoing

# Allow SSH (change port if needed)
sudo ufw allow 22/tcp

# Allow HTTPS
sudo ufw allow 443/tcp

# Allow HTTP (for Let's Encrypt)
sudo ufw allow 80/tcp

# Optional: Allow specific IPs only
sudo ufw allow from 203.0.113.0/24 to any port 22

# Enable firewall
sudo ufw enable

# Check status
sudo ufw status verbose
```

#### iptables

```bash
#!/bin/bash
# firewall-rules.sh

# Flush existing rules
iptables -F
iptables -X
iptables -Z

# Default policies
iptables -P INPUT DROP
iptables -P FORWARD DROP
iptables -P OUTPUT ACCEPT

# Allow loopback
iptables -A INPUT -i lo -j ACCEPT

# Allow established connections
iptables -A INPUT -m state --state ESTABLISHED,RELATED -j ACCEPT

# Allow SSH (change port if needed)
iptables -A INPUT -p tcp --dport 22 -j ACCEPT

# Allow HTTPS
iptables -A INPUT -p tcp --dport 443 -j ACCEPT

# Allow HTTP (for Let's Encrypt)
iptables -A INPUT -p tcp --dport 80 -j ACCEPT

# Drop invalid packets
iptables -A INPUT -m state --state INVALID -j DROP

# Rate limit SSH
iptables -A INPUT -p tcp --dport 22 -m state --state NEW -m recent --set
iptables -A INPUT -p tcp --dport 22 -m state --state NEW -m recent --update --seconds 60 --hitcount 4 -j DROP

# Log dropped packets
iptables -A INPUT -j LOG --log-prefix "iptables-dropped: "

# Save rules
iptables-save > /etc/iptables/rules.v4
```

### Docker Network Isolation

Create separate networks for services:

```yaml
# docker-compose.yml
version: '3.8'

services:
  api:
    networks:
      - public
      - internal

  worker:
    networks:
      - internal

  postgres:
    networks:
      - internal
    # No external access

  nginx:
    networks:
      - public
    ports:
      - "80:80"
      - "443:443"

networks:
  # Public-facing network
  public:
    driver: bridge

  # Internal services only
  internal:
    driver: bridge
    internal: true  # No external access
```

### Kubernetes Network Policies

```yaml
# network-policy.yaml
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: sctv-network-policy
  namespace: sctv
spec:
  podSelector:
    matchLabels:
      app: sctv

  policyTypes:
    - Ingress
    - Egress

  ingress:
    # Allow from ingress controller
    - from:
        - namespaceSelector:
            matchLabels:
              name: ingress-nginx
      ports:
        - protocol: TCP
          port: 3000

    # Allow internal communication
    - from:
        - podSelector:
            matchLabels:
              app: sctv
      ports:
        - protocol: TCP
          port: 3000

  egress:
    # Allow DNS
    - to:
        - namespaceSelector:
            matchLabels:
              name: kube-system
        - podSelector:
            matchLabels:
              k8s-app: kube-dns
      ports:
        - protocol: UDP
          port: 53

    # Allow database access
    - to:
        - podSelector:
            matchLabels:
              app: postgresql
      ports:
        - protocol: TCP
          port: 5432

    # Allow external HTTPS (registries)
    - to:
        - namespaceSelector: {}
      ports:
        - protocol: TCP
          port: 443
```

### DDoS Protection

#### Rate Limiting at nginx

```nginx
# Rate limiting zones
limit_req_zone $binary_remote_addr zone=api_limit:10m rate=10r/s;
limit_req_zone $binary_remote_addr zone=auth_limit:10m rate=5r/m;
limit_req_zone $binary_remote_addr zone=scan_limit:10m rate=1r/s;

# Connection limiting
limit_conn_zone $binary_remote_addr zone=conn_limit:10m;

server {
    # Apply rate limits
    location /api/ {
        limit_req zone=api_limit burst=20 nodelay;
        limit_conn conn_limit 10;
        proxy_pass http://api:3000;
    }

    location /api/v1/auth/ {
        limit_req zone=auth_limit burst=5 nodelay;
        proxy_pass http://api:3000;
    }

    location /api/v1/scans {
        limit_req zone=scan_limit burst=5 nodelay;
        proxy_pass http://api:3000;
    }
}
```

#### Cloudflare DDoS Protection

```bash
# Use Cloudflare as reverse proxy
# 1. Point DNS to Cloudflare
# 2. Enable "Under Attack" mode for DDoS
# 3. Configure rate limiting rules
# 4. Enable Bot Fight Mode
# 5. Set up Page Rules for sensitive endpoints
```

---

## Database Security

### PostgreSQL Hardening

#### 1. Authentication Configuration

Edit `pg_hba.conf`:

```conf
# TYPE  DATABASE        USER            ADDRESS                 METHOD

# Local connections
local   all             postgres                                peer
local   all             all                                     md5

# IPv4 local connections
host    all             all             127.0.0.1/32            md5

# Application connections (require SSL)
hostssl sctv            sctv            10.0.0.0/8              md5

# Reject all other connections
host    all             all             0.0.0.0/0               reject
```

#### 2. Enable SSL/TLS

Edit `postgresql.conf`:

```conf
# SSL Configuration
ssl = on
ssl_cert_file = '/etc/postgresql/certs/server.crt'
ssl_key_file = '/etc/postgresql/certs/server.key'
ssl_ca_file = '/etc/postgresql/certs/ca.crt'
ssl_min_protocol_version = 'TLSv1.2'
ssl_ciphers = 'HIGH:MEDIUM:+3DES:!aNULL'
ssl_prefer_server_ciphers = on
```

Generate certificates:

```bash
# Generate CA key and certificate
openssl req -new -x509 -days 3650 -nodes -out ca.crt -keyout ca.key \
  -subj "/CN=PostgreSQL CA"

# Generate server key
openssl genrsa -out server.key 4096
chmod 600 server.key

# Generate server certificate signing request
openssl req -new -key server.key -out server.csr \
  -subj "/CN=postgres.example.com"

# Sign server certificate
openssl x509 -req -in server.csr -CA ca.crt -CAkey ca.key \
  -CAcreateserial -out server.crt -days 365

# Copy to PostgreSQL
cp server.crt server.key ca.crt /etc/postgresql/certs/
chown postgres:postgres /etc/postgresql/certs/*
chmod 600 /etc/postgresql/certs/server.key
```

#### 3. Row-Level Security (RLS)

Enable multi-tenant isolation:

```sql
-- Enable RLS on all tables
ALTER TABLE projects ENABLE ROW LEVEL SECURITY;
ALTER TABLE dependencies ENABLE ROW LEVEL SECURITY;
ALTER TABLE alerts ENABLE ROW LEVEL SECURITY;
ALTER TABLE scans ENABLE ROW LEVEL SECURITY;
ALTER TABLE users ENABLE ROW LEVEL SECURITY;

-- Create policies
CREATE POLICY tenant_isolation_policy ON projects
  USING (tenant_id = current_setting('app.current_tenant_id', true)::uuid);

CREATE POLICY tenant_isolation_policy ON dependencies
  USING (tenant_id = current_setting('app.current_tenant_id', true)::uuid);

CREATE POLICY tenant_isolation_policy ON alerts
  USING (tenant_id = current_setting('app.current_tenant_id', true)::uuid);

CREATE POLICY tenant_isolation_policy ON scans
  USING (tenant_id = current_setting('app.current_tenant_id', true)::uuid);

CREATE POLICY tenant_isolation_policy ON users
  USING (tenant_id = current_setting('app.current_tenant_id', true)::uuid);

-- Function to set tenant context
CREATE OR REPLACE FUNCTION set_tenant_id(tenant_id uuid)
RETURNS void AS $$
BEGIN
  PERFORM set_config('app.current_tenant_id', tenant_id::text, false);
END;
$$ LANGUAGE plpgsql SECURITY DEFINER;
```

#### 4. Database User Permissions

```sql
-- Create dedicated user with minimal permissions
CREATE USER sctv_app WITH PASSWORD 'strong_random_password';

-- Grant only necessary permissions
GRANT CONNECT ON DATABASE sctv TO sctv_app;
GRANT USAGE ON SCHEMA public TO sctv_app;

-- Table permissions
GRANT SELECT, INSERT, UPDATE, DELETE ON projects TO sctv_app;
GRANT SELECT, INSERT, UPDATE, DELETE ON dependencies TO sctv_app;
GRANT SELECT, INSERT, UPDATE, DELETE ON alerts TO sctv_app;
GRANT SELECT, INSERT, UPDATE, DELETE ON scans TO sctv_app;
GRANT SELECT, INSERT, UPDATE, DELETE ON users TO sctv_app;

-- Sequence permissions
GRANT USAGE, SELECT ON ALL SEQUENCES IN SCHEMA public TO sctv_app;

-- Prevent table creation
REVOKE CREATE ON SCHEMA public FROM sctv_app;
```

#### 5. Database Encryption

```bash
# Encrypt PostgreSQL data directory (using LUKS)
# Warning: Do this before initializing database

# Create encrypted volume
cryptsetup luksFormat /dev/sdb1
cryptsetup luksOpen /dev/sdb1 postgres_data

# Format and mount
mkfs.ext4 /dev/mapper/postgres_data
mkdir -p /var/lib/postgresql/data
mount /dev/mapper/postgres_data /var/lib/postgresql/data

# Add to /etc/crypttab
echo "postgres_data /dev/sdb1 none" >> /etc/crypttab

# Add to /etc/fstab
echo "/dev/mapper/postgres_data /var/lib/postgresql/data ext4 defaults 0 2" >> /etc/fstab
```

#### 6. Backup Encryption

```bash
# Encrypt backups with GPG
pg_dump -U sctv -Fc sctv | gpg --encrypt --recipient backup@example.com > backup.sql.gpg

# Or use pgcrypto for column-level encryption
CREATE EXTENSION IF NOT EXISTS pgcrypto;

-- Encrypt sensitive columns
CREATE TABLE sensitive_data (
  id UUID PRIMARY KEY,
  encrypted_value BYTEA,
  -- Store encrypted
  INSERT INTO sensitive_data (id, encrypted_value)
  VALUES (gen_random_uuid(), pgp_sym_encrypt('secret', 'encryption_key'))
);

-- Decrypt when querying
SELECT id, pgp_sym_decrypt(encrypted_value, 'encryption_key') AS value
FROM sensitive_data;
```

---

## API Security Headers

### Configure Security Headers

In `nginx.conf` or application configuration:

```nginx
# Strict-Transport-Security (HSTS)
add_header Strict-Transport-Security "max-age=63072000; includeSubDomains; preload" always;

# X-Frame-Options (prevent clickjacking)
add_header X-Frame-Options "DENY" always;

# X-Content-Type-Options (prevent MIME sniffing)
add_header X-Content-Type-Options "nosniff" always;

# X-XSS-Protection (legacy XSS protection)
add_header X-XSS-Protection "1; mode=block" always;

# Referrer-Policy
add_header Referrer-Policy "strict-origin-when-cross-origin" always;

# Content-Security-Policy (CSP)
add_header Content-Security-Policy "default-src 'self'; script-src 'self' 'unsafe-inline' 'unsafe-eval'; style-src 'self' 'unsafe-inline'; img-src 'self' data: https:; font-src 'self'; connect-src 'self'; frame-ancestors 'none'; base-uri 'self'; form-action 'self';" always;

# Permissions-Policy (formerly Feature-Policy)
add_header Permissions-Policy "geolocation=(), microphone=(), camera=(), payment=(), usb=(), magnetometer=(), gyroscope=(), speaker=()" always;

# Remove server information
server_tokens off;
more_clear_headers 'Server';
more_clear_headers 'X-Powered-By';
```

### Test Security Headers

```bash
# Test with curl
curl -I https://sctv.example.com

# Use security headers checker
# Visit: https://securityheaders.com/?q=sctv.example.com

# Or use observatory
# Visit: https://observatory.mozilla.org/analyze/sctv.example.com
```

---

## Rate Limiting

### Application-Level Rate Limiting

Configure in `.env`:

```bash
# Global rate limits
RATE_LIMIT_ENABLED=true
RATE_LIMIT_REQUESTS_PER_MINUTE=60
RATE_LIMIT_BURST=20

# Per-endpoint limits
API_RATE_LIMIT_AUTH=5      # 5 requests per minute for /auth
API_RATE_LIMIT_SCAN=10     # 10 requests per minute for /scan
API_RATE_LIMIT_WEBHOOK=100 # 100 requests per minute for webhooks

# Per-tenant limits
TENANT_RATE_LIMIT_ENABLED=true
TENANT_RATE_LIMIT_PROJECTS=100      # Max 100 projects per tenant
TENANT_RATE_LIMIT_SCANS_PER_DAY=500 # Max 500 scans per day
```

### nginx Rate Limiting

```nginx
# Define rate limit zones
limit_req_zone $binary_remote_addr zone=general:10m rate=60r/m;
limit_req_zone $binary_remote_addr zone=auth:10m rate=5r/m;
limit_req_zone $binary_remote_addr zone=api:10m rate=100r/m;

# Define connection limits
limit_conn_zone $binary_remote_addr zone=conn_limit:10m;

http {
    # Apply limits
    server {
        location / {
            limit_req zone=general burst=10 nodelay;
            limit_conn conn_limit 5;
        }

        location /api/v1/auth/ {
            limit_req zone=auth burst=3 nodelay;
        }

        location /api/ {
            limit_req zone=api burst=20 nodelay;
        }
    }

    # Custom error page for rate limiting
    error_page 429 /rate_limit_exceeded.html;
    location = /rate_limit_exceeded.html {
        internal;
        default_type application/json;
        return 429 '{"error": "Rate limit exceeded", "message": "Too many requests. Please try again later."}';
    }
}
```

### Redis-Based Rate Limiting

```bash
# Add Redis to docker-compose.yml
redis:
  image: redis:alpine
  restart: always
  command: redis-server --requirepass ${REDIS_PASSWORD}
  networks:
    - internal

# Configure in .env
REDIS_ENABLED=true
REDIS_URL=redis://:${REDIS_PASSWORD}@redis:6379
RATE_LIMIT_BACKEND=redis
```

---

## Secret Management

### Environment Variables

```bash
# Generate strong secrets
JWT_SECRET=$(openssl rand -hex 32)
POSTGRES_PASSWORD=$(openssl rand -hex 32)
API_KEY_SECRET=$(openssl rand -hex 32)

# Store in .env (NEVER commit to git)
cat > .env << EOF
# Database
POSTGRES_PASSWORD=$POSTGRES_PASSWORD
DATABASE_URL=postgresql://sctv:$POSTGRES_PASSWORD@postgres:5432/sctv

# Authentication
JWT_SECRET=$JWT_SECRET
JWT_EXPIRATION_HOURS=8

# API Keys
API_KEY_SECRET=$API_KEY_SECRET

# External Services
SMTP_PASSWORD=<secure_password>
SLACK_WEBHOOK_URL=https://hooks.slack.com/services/xxx/yyy/zzz
EOF

# Secure permissions
chmod 600 .env
```

### Docker Secrets (Docker Swarm)

```bash
# Create secrets
echo "strong_db_password" | docker secret create db_password -
echo "strong_jwt_secret" | docker secret create jwt_secret -

# Use in docker-compose.yml
version: '3.8'
services:
  api:
    secrets:
      - db_password
      - jwt_secret
    environment:
      DATABASE_URL: postgresql://sctv:$(cat /run/secrets/db_password)@postgres:5432/sctv
      JWT_SECRET_FILE: /run/secrets/jwt_secret

secrets:
  db_password:
    external: true
  jwt_secret:
    external: true
```

### Kubernetes Secrets

```bash
# Create secrets
kubectl create secret generic sctv-secrets \
  --from-literal=postgres-password=$(openssl rand -hex 32) \
  --from-literal=jwt-secret=$(openssl rand -hex 32) \
  --namespace sctv

# Use in deployment
apiVersion: apps/v1
kind: Deployment
metadata:
  name: sctv-api
spec:
  template:
    spec:
      containers:
        - name: api
          env:
            - name: POSTGRES_PASSWORD
              valueFrom:
                secretKeyRef:
                  name: sctv-secrets
                  key: postgres-password
            - name: JWT_SECRET
              valueFrom:
                secretKeyRef:
                  name: sctv-secrets
                  key: jwt-secret
```

### HashiCorp Vault (Enterprise)

```bash
# Install Vault
curl -fsSL https://apt.releases.hashicorp.com/gpg | sudo apt-key add -
sudo apt-add-repository "deb [arch=amd64] https://apt.releases.hashicorp.com $(lsb_release -cs) main"
sudo apt-get update && sudo apt-get install vault

# Start Vault
vault server -dev

# Store secrets
vault kv put secret/sctv/database password="strong_password"
vault kv put secret/sctv/jwt secret="jwt_secret"

# Retrieve in application
export VAULT_ADDR='http://127.0.0.1:8200'
export VAULT_TOKEN='root'
vault kv get -field=password secret/sctv/database
```

---

## Audit Logging

### Enable Audit Logging

Configure in `.env`:

```bash
# Audit logging
AUDIT_LOGGING_ENABLED=true
AUDIT_LOG_LEVEL=info
AUDIT_LOG_FILE=/var/log/sctv/audit.log
AUDIT_LOG_FORMAT=json

# What to audit
AUDIT_LOG_AUTH=true           # Authentication events
AUDIT_LOG_API_CALLS=true      # All API calls
AUDIT_LOG_DATA_ACCESS=true    # Data access (reads)
AUDIT_LOG_DATA_CHANGES=true   # Data modifications (writes)
AUDIT_LOG_ADMIN_ACTIONS=true  # Administrative actions
```

### Audit Log Format

```json
{
  "timestamp": "2026-01-15T10:30:45Z",
  "event_type": "api_call",
  "actor": {
    "user_id": "user-123",
    "tenant_id": "tenant-456",
    "ip_address": "203.0.113.10",
    "user_agent": "Mozilla/5.0..."
  },
  "action": "project.create",
  "resource": {
    "type": "project",
    "id": "proj-789"
  },
  "result": "success",
  "metadata": {
    "endpoint": "/api/v1/projects",
    "method": "POST",
    "status_code": 201,
    "duration_ms": 145
  }
}
```

### Database Audit Trigger

```sql
-- Create audit log table
CREATE TABLE audit_log (
  id BIGSERIAL PRIMARY KEY,
  timestamp TIMESTAMPTZ NOT NULL DEFAULT now(),
  table_name TEXT NOT NULL,
  operation TEXT NOT NULL,
  user_id UUID,
  tenant_id UUID,
  old_data JSONB,
  new_data JSONB,
  changed_fields TEXT[]
);

-- Create audit trigger function
CREATE OR REPLACE FUNCTION audit_trigger_func()
RETURNS TRIGGER AS $$
BEGIN
  IF (TG_OP = 'DELETE') THEN
    INSERT INTO audit_log (table_name, operation, user_id, tenant_id, old_data)
    VALUES (TG_TABLE_NAME, 'DELETE', current_setting('app.current_user_id', true)::uuid,
            current_setting('app.current_tenant_id', true)::uuid, row_to_json(OLD));
    RETURN OLD;
  ELSIF (TG_OP = 'UPDATE') THEN
    INSERT INTO audit_log (table_name, operation, user_id, tenant_id, old_data, new_data)
    VALUES (TG_TABLE_NAME, 'UPDATE', current_setting('app.current_user_id', true)::uuid,
            current_setting('app.current_tenant_id', true)::uuid,
            row_to_json(OLD), row_to_json(NEW));
    RETURN NEW;
  ELSIF (TG_OP = 'INSERT') THEN
    INSERT INTO audit_log (table_name, operation, user_id, tenant_id, new_data)
    VALUES (TG_TABLE_NAME, 'INSERT', current_setting('app.current_user_id', true)::uuid,
            current_setting('app.current_tenant_id', true)::uuid, row_to_json(NEW));
    RETURN NEW;
  END IF;
END;
$$ LANGUAGE plpgsql;

-- Apply to sensitive tables
CREATE TRIGGER audit_trigger AFTER INSERT OR UPDATE OR DELETE ON projects
FOR EACH ROW EXECUTE FUNCTION audit_trigger_func();

CREATE TRIGGER audit_trigger AFTER INSERT OR UPDATE OR DELETE ON users
FOR EACH ROW EXECUTE FUNCTION audit_trigger_func();

CREATE TRIGGER audit_trigger AFTER INSERT OR UPDATE OR DELETE ON policies
FOR EACH ROW EXECUTE FUNCTION audit_trigger_func();
```

### Query Audit Logs

```sql
-- Recent authentication events
SELECT * FROM audit_log
WHERE action = 'auth.login'
ORDER BY timestamp DESC
LIMIT 100;

-- Failed login attempts
SELECT user_id, ip_address, count(*)
FROM audit_log
WHERE action = 'auth.login' AND result = 'failure'
GROUP BY user_id, ip_address
HAVING count(*) > 5;

-- Data modifications by user
SELECT action, resource_type, count(*)
FROM audit_log
WHERE user_id = 'user-123'
AND action LIKE '%.update' OR action LIKE '%.delete'
GROUP BY action, resource_type;

-- Compliance report (last 90 days)
SELECT
  date_trunc('day', timestamp) AS day,
  count(*) AS total_events,
  count(*) FILTER (WHERE result = 'failure') AS failures
FROM audit_log
WHERE timestamp > now() - interval '90 days'
GROUP BY day
ORDER BY day;
```

---

## Vulnerability Scanning

### Container Image Scanning

#### Trivy

```bash
# Install Trivy
sudo apt-get install trivy

# Scan image
trivy image ghcr.io/example/sctv-api:latest

# Scan with severity filter
trivy image --severity HIGH,CRITICAL ghcr.io/example/sctv-api:latest

# Generate report
trivy image --format json --output report.json ghcr.io/example/sctv-api:latest

# Scan in CI/CD
# .github/workflows/security-scan.yml
- name: Scan image
  run: |
    trivy image --exit-code 1 --severity CRITICAL ghcr.io/example/sctv-api:${{ github.sha }}
```

#### Clair

```bash
# Run Clair
docker run -d --name clair-db postgres:latest
docker run -d --name clair --link clair-db:postgres arminc/clair-local-scan

# Scan image
docker run --rm --link clair:clair arminc/clair-scanner:latest \
  --clair=http://clair:6060 \
  --ip=$(hostname -i) \
  ghcr.io/example/sctv-api:latest
```

### Dependency Scanning

```bash
# Scan Rust dependencies with cargo-audit
cargo install cargo-audit
cargo audit

# Scan in CI/CD
# .github/workflows/security-scan.yml
- name: Audit dependencies
  run: cargo audit --deny warnings

# Generate SBOM
cargo install cargo-sbom
cargo sbom > sbom.json
```

### SAST (Static Application Security Testing)

```bash
# Clippy (Rust linter)
cargo clippy -- -D warnings

# Semgrep
semgrep --config=auto .

# CodeQL (GitHub)
# Enable in repository settings: Security > Code scanning
```

---

## Incident Response

### Incident Response Plan

#### 1. Detection

**Indicators of Compromise (IoC):**
- Unusual authentication failures
- Unexpected API calls
- High resource usage
- Database anomalies
- Security alert triggers

**Monitoring:**
```bash
# Monitor failed logins
docker-compose logs api | grep "auth.*failed"

# Monitor suspicious activity
curl http://prometheus:9090/api/v1/query?query=rate(sctv_auth_attempts_total{status="failed"}[5m])

# Check security alerts
curl http://alertmanager:9093/api/v2/alerts | jq '.[] | select(.labels.severity=="critical")'
```

#### 2. Containment

**Immediate Actions:**
```bash
# 1. Isolate affected systems
docker-compose stop api worker

# 2. Block suspicious IPs at firewall
sudo ufw deny from 203.0.113.100

# 3. Revoke compromised credentials
docker-compose exec postgres psql -U sctv -c "
UPDATE users SET is_active = false WHERE id = 'compromised-user-id';
"

# 4. Rotate secrets
JWT_SECRET=$(openssl rand -hex 32)
echo "JWT_SECRET=$JWT_SECRET" >> .env
docker-compose restart api

# 5. Enable additional logging
echo "LOG_LEVEL=debug" >> .env
echo "AUDIT_LOG_LEVEL=debug" >> .env
```

#### 3. Investigation

**Collect Evidence:**
```bash
#!/bin/bash
# incident-investigation.sh

INCIDENT_ID="incident-$(date +%Y%m%d-%H%M%S)"
EVIDENCE_DIR="/var/log/incidents/$INCIDENT_ID"
mkdir -p "$EVIDENCE_DIR"

# Collect logs
docker-compose logs > "$EVIDENCE_DIR/docker-logs.txt"
cp /var/log/sctv/* "$EVIDENCE_DIR/"

# Database snapshot
docker-compose exec postgres pg_dump -U sctv > "$EVIDENCE_DIR/database-snapshot.sql"

# Audit logs
docker-compose exec postgres psql -U sctv -c "
COPY (SELECT * FROM audit_log WHERE timestamp > now() - interval '24 hours')
TO STDOUT CSV HEADER" > "$EVIDENCE_DIR/audit-log.csv"

# System state
docker-compose ps > "$EVIDENCE_DIR/container-status.txt"
docker stats --no-stream > "$EVIDENCE_DIR/resource-usage.txt"

# Network connections
netstat -tuln > "$EVIDENCE_DIR/network-connections.txt"

# Preserve evidence
tar -czf "$EVIDENCE_DIR.tar.gz" "$EVIDENCE_DIR"
sha256sum "$EVIDENCE_DIR.tar.gz" > "$EVIDENCE_DIR.tar.gz.sha256"

echo "Evidence collected: $EVIDENCE_DIR.tar.gz"
```

#### 4. Eradication

**Remove Threat:**
```bash
# 1. Patch vulnerabilities
docker-compose pull  # Get latest images
docker-compose up -d --force-recreate

# 2. Clean compromised data
docker-compose exec postgres psql -U sctv -c "
DELETE FROM sessions WHERE user_id IN (SELECT id FROM users WHERE is_compromised = true);
"

# 3. Reset affected accounts
docker-compose exec postgres psql -U sctv -c "
UPDATE users SET password_hash = NULL, must_change_password = true
WHERE id IN ('user1', 'user2');
"

# 4. Review and remove backdoors
# Manual code review and security audit
```

#### 5. Recovery

**Restore Normal Operations:**
```bash
# 1. Restore from clean backup (if needed)
./restore-backup.sh backup-20260115.tar.gz

# 2. Restart services
docker-compose up -d

# 3. Verify integrity
docker-compose exec api sctv-cli health-check
docker-compose exec postgres psql -U sctv -c "SELECT version();"

# 4. Monitor closely
docker-compose logs -f | tee recovery-monitor.log
```

#### 6. Post-Incident

**Lessons Learned:**
```markdown
# Incident Post-Mortem

**Incident ID:** INC-2026-001
**Date:** 2026-01-15
**Severity:** High

## Summary
Brief description of what happened.

## Timeline
- 10:00 - Detection
- 10:15 - Containment
- 10:30 - Investigation started
- 12:00 - Eradication completed
- 14:00 - Recovery completed

## Root Cause
What caused the incident.

## Impact
- Systems affected
- Data compromised
- Downtime duration

## Actions Taken
1. Immediate containment
2. Evidence collection
3. Threat removal
4. System recovery

## Lessons Learned
- What went well
- What could be improved
- Gaps identified

## Action Items
- [ ] Update firewall rules
- [ ] Implement additional monitoring
- [ ] Security training for team
- [ ] Update incident response plan
```

---

## Compliance

### GDPR Compliance

#### Data Protection Measures

```bash
# Data encryption at rest
# Already covered in Database Security section

# Data encryption in transit
# TLS 1.2+ configured

# Right to access
docker-compose exec api sctv-cli gdpr export-data --user-id=user-123 > user-data.json

# Right to erasure ("right to be forgotten")
docker-compose exec api sctv-cli gdpr delete-user --user-id=user-123 --confirm

# Data retention
docker-compose exec postgres psql -U sctv -c "
DELETE FROM audit_log WHERE timestamp < now() - interval '2 years';
DELETE FROM scans WHERE created_at < now() - interval '1 year' AND status = 'completed';
"
```

#### Privacy Policy

Ensure your privacy policy includes:
- What data is collected
- How data is used
- Data retention periods
- User rights (access, deletion, portability)
- Contact information for DPO

### SOC 2 Compliance

#### Control Implementation

**Access Controls:**
```bash
# CC6.1 - Logical and physical access controls
# - Multi-factor authentication enabled
# - Role-based access control implemented
# - Audit logging enabled

# Verify
docker-compose exec api sctv-cli security audit-access-controls
```

**Change Management:**
```bash
# CC8.1 - Change management process
# - All changes reviewed and approved
# - Testing before deployment
# - Rollback procedures documented

# Document in version control
git log --oneline --graph --decorate
```

**Monitoring:**
```bash
# CC7.2 - System monitoring
# - Prometheus metrics configured
# - Alerting enabled
# - Log aggregation setup

# Verify
curl http://prometheus:9090/api/v1/targets
```

### Compliance Audit Script

```bash
#!/bin/bash
# compliance-audit.sh

echo "=== SCTV Compliance Audit ==="
echo "Date: $(date)"
echo ""

# 1. Authentication
echo "## Authentication Security"
echo -n "JWT secret configured: "
[ -n "$JWT_SECRET" ] && echo "✓ Yes" || echo "✗ No"

echo -n "Strong passwords enforced: "
docker-compose exec postgres psql -U sctv -t -c "
SELECT COUNT(*) FROM users WHERE password_hash IS NOT NULL;" | xargs echo

# 2. Encryption
echo ""
echo "## Encryption"
echo -n "TLS enabled: "
curl -Is https://sctv.example.com | grep -q "Strict-Transport-Security" && echo "✓ Yes" || echo "✗ No"

echo -n "Database SSL enabled: "
docker-compose exec postgres psql -U sctv -t -c "SHOW ssl;" | xargs echo

# 3. Audit Logging
echo ""
echo "## Audit Logging"
echo -n "Audit logs enabled: "
[ "$AUDIT_LOGGING_ENABLED" = "true" ] && echo "✓ Yes" || echo "✗ No"

echo -n "Audit log entries (last 24h): "
docker-compose exec postgres psql -U sctv -t -c "
SELECT COUNT(*) FROM audit_log WHERE timestamp > now() - interval '24 hours';" | xargs echo

# 4. Backups
echo ""
echo "## Backups"
echo -n "Backup configured: "
[ -f "/etc/cron.d/sctv-backup" ] && echo "✓ Yes" || echo "✗ No"

echo -n "Last backup: "
ls -t /backups/*.tar.gz | head -1 | xargs stat -c %y 2>/dev/null || echo "None"

# 5. Access Controls
echo ""
echo "## Access Controls"
echo -n "Row-level security enabled: "
docker-compose exec postgres psql -U sctv -t -c "
SELECT COUNT(*) FROM pg_tables WHERE schemaname = 'public' AND rowsecurity = true;" | xargs echo

# 6. Vulnerability Scanning
echo ""
echo "## Security Scanning"
echo "Running Trivy scan..."
trivy image --severity HIGH,CRITICAL --quiet ghcr.io/example/sctv-api:latest | wc -l | xargs echo -n "Vulnerabilities found: "

echo ""
echo "=== Audit Complete ==="
```

---

## Next Steps

- [Monitoring](monitoring.md) - Set up security monitoring
- [Troubleshooting](troubleshooting.md) - Security incident response
- [Backup and Recovery](backup.md) - Data protection

---

**Security is a continuous process!** Regularly review and update security measures as threats evolve.
