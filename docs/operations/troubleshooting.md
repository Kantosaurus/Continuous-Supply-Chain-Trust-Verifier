# Troubleshooting Guide

**Version:** 0.1.0

Comprehensive troubleshooting guide for diagnosing and resolving common SCTV issues.

---

## Table of Contents

- [Overview](#overview)
- [Diagnostic Tools](#diagnostic-tools)
- [Common Issues](#common-issues)
  - [Database Connection Issues](#database-connection-issues)
  - [Worker Not Processing Jobs](#worker-not-processing-jobs)
  - [API Authentication Errors](#api-authentication-errors)
  - [Scan Failures](#scan-failures)
  - [Notification Delivery Issues](#notification-delivery-issues)
  - [Performance Problems](#performance-problems)
  - [Memory Issues](#memory-issues)
- [Log Analysis](#log-analysis)
- [Debug Mode](#debug-mode)
- [Support Escalation](#support-escalation)

---

## Overview

This guide provides step-by-step troubleshooting procedures for common SCTV issues. Each section includes:

- **Symptoms:** How to identify the issue
- **Diagnosis:** Steps to confirm the root cause
- **Resolution:** How to fix the problem
- **Prevention:** How to avoid the issue in the future

### Quick Diagnostic Checklist

Before diving into specific issues, run this quick checklist:

```bash
# 1. Check service health
curl http://localhost:3000/health

# 2. Check service status
docker-compose ps
# or
kubectl get pods -n sctv

# 3. Check recent logs
docker-compose logs --tail=100 api worker
# or
kubectl logs -n sctv --tail=100 -l app=sctv-api

# 4. Check database connectivity
docker-compose exec postgres psql -U sctv -c "SELECT version();"

# 5. Check disk space
df -h

# 6. Check memory usage
free -h

# 7. Check network connectivity
curl -I https://registry.npmjs.org
```

---

## Diagnostic Tools

### Built-in Diagnostics

```bash
# Health check with details
curl http://localhost:3000/health/detailed | jq '.'

# Metrics endpoint
curl http://localhost:3000/metrics

# Database connection test
docker-compose exec api sctv-cli db test-connection

# Worker queue status
docker-compose exec api sctv-cli queue status

# Check configuration
docker-compose exec api sctv-cli config show
```

### Log Viewing

```bash
# Follow all logs
docker-compose logs -f

# Follow specific service
docker-compose logs -f api
docker-compose logs -f worker

# View last N lines
docker-compose logs --tail=100 api

# Search logs for errors
docker-compose logs api | grep ERROR

# Kubernetes logs
kubectl logs -n sctv -f deployment/sctv-api
kubectl logs -n sctv -l app=sctv-worker --tail=100
```

### Database Diagnostics

```bash
# Connect to database
docker-compose exec postgres psql -U sctv

# Check active connections
docker-compose exec postgres psql -U sctv -c "
SELECT count(*), state
FROM pg_stat_activity
GROUP BY state;
"

# Check database size
docker-compose exec postgres psql -U sctv -c "
SELECT pg_size_pretty(pg_database_size('sctv')) AS size;
"

# Check slow queries
docker-compose exec postgres psql -U sctv -c "
SELECT pid, now() - pg_stat_activity.query_start AS duration, query
FROM pg_stat_activity
WHERE (now() - pg_stat_activity.query_start) > interval '1 minute'
AND state = 'active';
"

# Check locks
docker-compose exec postgres psql -U sctv -c "
SELECT * FROM pg_locks WHERE NOT granted;
"
```

---

## Common Issues

### Database Connection Issues

#### Symptom: "connection refused" or "could not connect to server"

**Error Messages:**
```
ERROR: could not connect to server: Connection refused
ERROR: password authentication failed for user "sctv"
ERROR: FATAL: too many connections for role "sctv"
```

#### Diagnosis

```bash
# 1. Check if PostgreSQL is running
docker-compose ps postgres
# or
kubectl get pods -n sctv -l app=postgresql

# 2. Check PostgreSQL logs
docker-compose logs postgres --tail=50

# 3. Test direct connection
docker-compose exec postgres psql -U sctv -c "SELECT 1;"

# 4. Check connection string in environment
docker-compose exec api env | grep DATABASE_URL

# 5. Check network connectivity
docker-compose exec api ping postgres
```

#### Resolution

**Problem: PostgreSQL not running**

```bash
# Restart PostgreSQL
docker-compose restart postgres

# Check status
docker-compose ps postgres

# Verify it's healthy
docker-compose exec postgres pg_isready
```

**Problem: Wrong credentials**

```bash
# Check environment variables
cat .env | grep POSTGRES

# Reset password
docker-compose exec postgres psql -U postgres -c "
ALTER USER sctv WITH PASSWORD 'new_password';
"

# Update .env file
nano .env
# Update POSTGRES_PASSWORD=new_password

# Restart services
docker-compose restart api worker
```

**Problem: Too many connections**

```bash
# Check current connections
docker-compose exec postgres psql -U sctv -c "
SELECT count(*) FROM pg_stat_activity;
"

# Check max connections
docker-compose exec postgres psql -U sctv -c "
SHOW max_connections;
"

# Kill idle connections
docker-compose exec postgres psql -U sctv -c "
SELECT pg_terminate_backend(pid)
FROM pg_stat_activity
WHERE state = 'idle'
AND query_start < now() - interval '10 minutes';
"

# Increase max_connections (edit postgresql.conf)
# or increase connection pool size in .env
echo "DB_POOL_MAX_SIZE=10" >> .env
docker-compose restart api worker
```

**Problem: Connection pool exhausted**

```bash
# Check pool metrics
curl http://localhost:3000/metrics | grep sctv_db_connections

# Increase pool size
echo "DB_POOL_MAX_SIZE=20" >> .env
echo "DB_POOL_MIN_SIZE=5" >> .env
docker-compose restart api worker
```

#### Prevention

1. **Configure connection pooling** properly
2. **Monitor connection usage** with Prometheus
3. **Set connection timeouts** to prevent leaks
4. **Use connection pool sizing** based on load

```bash
# Recommended settings in .env
DB_POOL_MAX_SIZE=20
DB_POOL_MIN_SIZE=5
DB_POOL_TIMEOUT_SECONDS=30
DB_POOL_MAX_LIFETIME_SECONDS=1800
```

---

### Worker Not Processing Jobs

#### Symptom: Jobs stuck in "pending" state

**Signs:**
- Jobs queue up but never complete
- `sctv_jobs_queued{status="pending"}` increasing
- No worker activity in logs

#### Diagnosis

```bash
# 1. Check worker status
docker-compose ps worker
kubectl get pods -n sctv -l app=sctv-worker

# 2. Check worker logs
docker-compose logs worker --tail=100

# 3. Check job queue
docker-compose exec postgres psql -U sctv -c "
SELECT status, count(*)
FROM jobs
GROUP BY status;
"

# 4. Check for stuck jobs
docker-compose exec postgres psql -U sctv -c "
SELECT id, job_type, status, created_at, updated_at
FROM jobs
WHERE status = 'processing'
AND updated_at < now() - interval '10 minutes'
ORDER BY created_at DESC
LIMIT 10;
"

# 5. Check worker pool metrics
curl http://localhost:3001/metrics | grep sctv_worker_pool
```

#### Resolution

**Problem: Worker service not running**

```bash
# Start worker
docker-compose up -d worker

# Scale workers (Docker Compose)
docker-compose up -d --scale worker=2

# Scale workers (Kubernetes)
kubectl scale deployment sctv-worker --replicas=4 -n sctv
```

**Problem: Worker crashed or hung**

```bash
# Check logs for errors
docker-compose logs worker --tail=200 | grep -E "ERROR|FATAL|panic"

# Restart worker
docker-compose restart worker

# Or force recreate
docker-compose up -d --force-recreate worker
```

**Problem: Jobs stuck in "processing" state**

```bash
# Mark stuck jobs as failed
docker-compose exec postgres psql -U sctv -c "
UPDATE jobs
SET status = 'failed',
    error_message = 'Job timed out',
    updated_at = now()
WHERE status = 'processing'
AND updated_at < now() - interval '30 minutes';
"

# Or retry stuck jobs
docker-compose exec postgres psql -U sctv -c "
UPDATE jobs
SET status = 'pending',
    retry_count = retry_count + 1,
    updated_at = now()
WHERE status = 'processing'
AND updated_at < now() - interval '30 minutes';
"
```

**Problem: Worker pool saturated**

```bash
# Check worker pool utilization
curl http://localhost:3001/metrics | grep sctv_worker_pool

# Increase worker pool size
echo "WORKER_POOL_SIZE=8" >> .env
docker-compose restart worker

# Or add more worker instances
docker-compose up -d --scale worker=3
```

**Problem: Database locks**

```bash
# Check for locks
docker-compose exec postgres psql -U sctv -c "
SELECT
  l.pid,
  l.mode,
  l.granted,
  a.query
FROM pg_locks l
JOIN pg_stat_activity a ON l.pid = a.pid
WHERE NOT l.granted;
"

# Kill blocking query
docker-compose exec postgres psql -U sctv -c "
SELECT pg_terminate_backend(pid)
FROM pg_stat_activity
WHERE pid = <PID>;
"
```

#### Prevention

1. **Set job timeouts** to prevent indefinite processing
2. **Monitor queue depth** and alert on backlog
3. **Auto-scale workers** based on queue depth
4. **Implement job recovery** for stuck jobs

```bash
# Add to .env
JOB_TIMEOUT_SECONDS=300
JOB_MAX_RETRIES=3
WORKER_POOL_SIZE=8

# Add cron job to recover stuck jobs
# /etc/cron.d/sctv-job-recovery
*/10 * * * * root /usr/local/bin/recover-stuck-jobs.sh
```

---

### API Authentication Errors

#### Symptom: 401 Unauthorized or 403 Forbidden

**Error Messages:**
```json
{
  "error": "Unauthorized",
  "message": "Invalid or expired token"
}
```

#### Diagnosis

```bash
# 1. Test authentication
curl -X POST http://localhost:3000/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email":"user@example.com","password":"password"}'

# 2. Verify JWT secret
docker-compose exec api env | grep JWT_SECRET

# 3. Check token expiration
# Decode JWT token (using jwt.io or jwt-cli)
jwt decode $TOKEN

# 4. Check API logs
docker-compose logs api | grep -E "auth|401|403"

# 5. Verify user exists
docker-compose exec postgres psql -U sctv -c "
SELECT id, email, role, is_active
FROM users
WHERE email = 'user@example.com';
"
```

#### Resolution

**Problem: Invalid JWT secret**

```bash
# Generate new JWT secret
JWT_SECRET=$(openssl rand -hex 32)
echo "JWT_SECRET=$JWT_SECRET" >> .env

# Restart API
docker-compose restart api

# All users need to re-authenticate
```

**Problem: Expired token**

```bash
# User needs to login again to get new token
curl -X POST http://localhost:3000/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email":"user@example.com","password":"password"}'

# Increase token expiration (in .env)
echo "JWT_EXPIRATION_HOURS=24" >> .env
docker-compose restart api
```

**Problem: User account disabled**

```bash
# Check user status
docker-compose exec postgres psql -U sctv -c "
SELECT id, email, is_active
FROM users
WHERE email = 'user@example.com';
"

# Enable user
docker-compose exec postgres psql -U sctv -c "
UPDATE users
SET is_active = true
WHERE email = 'user@example.com';
"
```

**Problem: Insufficient permissions**

```bash
# Check user role
docker-compose exec postgres psql -U sctv -c "
SELECT id, email, role
FROM users
WHERE email = 'user@example.com';
"

# Update user role
docker-compose exec postgres psql -U sctv -c "
UPDATE users
SET role = 'admin'
WHERE email = 'user@example.com';
"
```

**Problem: CORS issues**

```bash
# Enable CORS (in .env)
echo "ENABLE_CORS=true" >> .env
echo "CORS_ALLOWED_ORIGINS=https://dashboard.example.com" >> .env
docker-compose restart api

# Check CORS headers
curl -I -X OPTIONS http://localhost:3000/api/v1/projects \
  -H "Origin: https://dashboard.example.com"
```

#### Prevention

1. **Use long, random JWT secrets**
2. **Set appropriate token expiration**
3. **Implement token refresh** mechanism
4. **Monitor authentication failures**

```bash
# Recommended settings
JWT_SECRET=$(openssl rand -hex 32)
JWT_EXPIRATION_HOURS=8
JWT_REFRESH_ENABLED=true
```

---

### Scan Failures

#### Symptom: Scans fail with errors

**Error Messages:**
```
ERROR: Failed to fetch package metadata from registry
ERROR: Timeout while scanning project
ERROR: Invalid manifest file format
```

#### Diagnosis

```bash
# 1. Check failed scans
docker-compose exec postgres psql -U sctv -c "
SELECT id, project_id, status, error_message, created_at
FROM scans
WHERE status = 'failed'
ORDER BY created_at DESC
LIMIT 10;
"

# 2. Check worker logs
docker-compose logs worker | grep -E "ERROR|scan"

# 3. Test registry connectivity
curl -I https://registry.npmjs.org
curl -I https://pypi.org

# 4. Check manifest parsing
docker-compose exec api sctv-cli scan test \
  --file=/path/to/package.json \
  --dry-run

# 5. Check job metrics
curl http://localhost:3001/metrics | grep sctv_jobs_failed_total
```

#### Resolution

**Problem: Registry timeout or unavailable**

```bash
# Check registry status
curl -I https://registry.npmjs.org
# Check status page: https://status.npmjs.org

# Increase timeout (in .env)
echo "REGISTRY_TIMEOUT_SECONDS=60" >> .env
docker-compose restart worker

# Configure retry logic
echo "REGISTRY_MAX_RETRIES=3" >> .env
echo "REGISTRY_RETRY_DELAY_SECONDS=5" >> .env
docker-compose restart worker
```

**Problem: Invalid manifest file**

```bash
# Validate manifest locally
docker-compose exec api sctv-cli validate \
  --file=/path/to/package.json

# Check for common issues:
# - Invalid JSON syntax
# - Missing required fields
# - Unsupported version format

# Example: validate package.json
jq '.' package.json
```

**Problem: Rate limiting by registry**

```bash
# Check for rate limit errors in logs
docker-compose logs worker | grep -i "rate limit"

# Use authenticated registry access (npm) to get higher upstream limits
echo "NPM_TOKEN=your_token_here" >> .env
docker-compose restart worker
```

> **Note:** Client-side throttling of outbound registry requests
> (`REGISTRY_RATE_LIMIT`) is planned but not yet implemented. For now,
> mitigate upstream 429s with authenticated tokens and exponential
> backoff.

**Problem: Unsupported ecosystem**

```bash
# Check supported ecosystems
docker-compose exec api sctv-cli ecosystems list

# If ecosystem is missing, check if it's implemented
docker-compose exec api sctv-cli version
```

**Problem: Scan timeout**

```bash
# Check scan duration
docker-compose exec postgres psql -U sctv -c "
SELECT
  id,
  project_id,
  status,
  EXTRACT(EPOCH FROM (completed_at - started_at)) AS duration_seconds
FROM scans
WHERE completed_at IS NOT NULL
ORDER BY duration_seconds DESC
LIMIT 10;
"

# Increase scan timeout
echo "SCAN_TIMEOUT_SECONDS=600" >> .env
docker-compose restart worker
```

**Problem: Large dependency tree**

```bash
# Check dependency count
docker-compose exec postgres psql -U sctv -c "
SELECT s.id, s.project_id, count(d.*) AS dep_count
FROM scans s
JOIN dependencies d ON d.scan_id = s.id
GROUP BY s.id, s.project_id
ORDER BY dep_count DESC
LIMIT 10;
"

# Increase memory for worker
# In docker-compose.yml
services:
  worker:
    deploy:
      resources:
        limits:
          memory: 8G

# Restart
docker-compose up -d worker
```

#### Prevention

1. **Configure appropriate timeouts** for registries
2. **Implement retry logic** with exponential backoff
3. **Monitor scan success rate** with alerts
4. **Cache registry responses** to reduce load

```bash
# Recommended settings
REGISTRY_TIMEOUT_SECONDS=60
REGISTRY_MAX_RETRIES=3
REGISTRY_RETRY_DELAY_SECONDS=5
REGISTRY_CACHE_TTL_SECONDS=3600
SCAN_TIMEOUT_SECONDS=600
```

---

### Notification Delivery Issues

#### Symptom: Alerts not being delivered

**Signs:**
- Notifications not received in Slack/email
- `sctv_notifications_failed_total` increasing
- Notification jobs stuck

#### Diagnosis

```bash
# 1. Check notification jobs
docker-compose exec postgres psql -U sctv -c "
SELECT id, job_type, status, error_message
FROM jobs
WHERE job_type = 'SendNotification'
AND status = 'failed'
ORDER BY created_at DESC
LIMIT 10;
"

# 2. Check notification logs
docker-compose logs worker | grep -i notification

# 3. Test notification channels
docker-compose exec api sctv-cli notify test \
  --channel=slack \
  --message="Test notification"

# 4. Check notification metrics
curl http://localhost:3001/metrics | grep sctv_notifications
```

#### Resolution

**Problem: Invalid webhook URL**

```bash
# Check notification settings
docker-compose exec postgres psql -U sctv -c "
SELECT tenant_id, channel_type, channel_config
FROM notification_channels
WHERE is_active = true;
"

# Test webhook manually
curl -X POST $SLACK_WEBHOOK_URL \
  -H "Content-Type: application/json" \
  -d '{"text":"Test message"}'

# Update webhook URL
docker-compose exec postgres psql -U sctv -c "
UPDATE notification_channels
SET channel_config = jsonb_set(
  channel_config,
  '{webhook_url}',
  '\"https://hooks.slack.com/services/NEW/WEBHOOK/URL\"'
)
WHERE id = 'channel-id';
"
```

**Problem: SMTP authentication failure**

```bash
# Check SMTP settings
docker-compose exec api env | grep SMTP

# Test SMTP connection
docker-compose exec api sctv-cli smtp test \
  --host=$SMTP_HOST \
  --port=$SMTP_PORT \
  --user=$SMTP_USER \
  --password=$SMTP_PASSWORD

# Update SMTP credentials
echo "SMTP_HOST=smtp.gmail.com" >> .env
echo "SMTP_PORT=587" >> .env
echo "SMTP_USER=alerts@example.com" >> .env
echo "SMTP_PASSWORD=app_password" >> .env
docker-compose restart worker
```

**Problem: Rate limiting**

```bash
# Check for rate limit errors from upstream notification services
docker-compose logs worker | grep -i "rate limit"
```

> **Note:** Client-side throttling / batching of outbound notifications
> (`NOTIFICATION_RATE_LIMIT`, `NOTIFICATION_BATCH_SIZE`,
> `NOTIFICATION_BATCH_DELAY_SECONDS`) is planned but not yet
> implemented. For now, mitigate upstream 429s (Slack, PagerDuty, SMTP,
> etc.) by reducing alert volume via policy severity thresholds and
> tenant notification settings.

**Problem: Notification template error**

```bash
# Check template rendering
docker-compose logs worker | grep "template"

# Test template
docker-compose exec api sctv-cli template render \
  --template=alert \
  --data='{"severity":"high","message":"Test"}'
```

#### Prevention

1. **Test notification channels** after configuration
2. **Monitor delivery success rate**
3. **Implement retry logic** for failed notifications
4. **Reduce alert volume** via severity thresholds (notification batching /
   in-app rate limiting is planned, not yet implemented)

```bash
# Recommended settings
NOTIFICATION_MAX_RETRIES=3
NOTIFICATION_RETRY_DELAY_SECONDS=60
# NOTIFICATION_RATE_LIMIT / NOTIFICATION_BATCH_SIZE: planned, not yet honored
```

---

### Performance Problems

#### Symptom: Slow API responses or timeouts

**Signs:**
- High latency (>2s for API requests)
- Timeouts
- High CPU or memory usage

#### Diagnosis

```bash
# 1. Check API metrics
curl http://localhost:3000/metrics | grep sctv_api_request_duration

# 2. Check resource usage
docker stats

# 3. Check slow queries
docker-compose exec postgres psql -U sctv -c "
SELECT
  pid,
  now() - pg_stat_activity.query_start AS duration,
  query,
  state
FROM pg_stat_activity
WHERE (now() - pg_stat_activity.query_start) > interval '1 second'
AND state = 'active'
ORDER BY duration DESC;
"

# 4. Check database indexes
docker-compose exec postgres psql -U sctv -c "
SELECT schemaname, tablename, indexname
FROM pg_indexes
WHERE schemaname = 'public';
"

# 5. Profile API request
time curl http://localhost:3000/api/v1/projects
```

#### Resolution

**Problem: Missing database indexes**

```bash
# Identify slow queries
docker-compose exec postgres psql -U sctv -c "
SELECT query, mean_exec_time, calls
FROM pg_stat_statements
ORDER BY mean_exec_time DESC
LIMIT 10;
"

# Add indexes
docker-compose exec postgres psql -U sctv -c "
CREATE INDEX CONCURRENTLY idx_dependencies_project_id
ON dependencies(project_id);

CREATE INDEX CONCURRENTLY idx_alerts_severity
ON alerts(severity) WHERE status = 'active';

CREATE INDEX CONCURRENTLY idx_scans_created_at
ON scans(created_at DESC);
"
```

**Problem: High connection pool contention**

```bash
# Check connection pool metrics
curl http://localhost:3000/metrics | grep sctv_db_connections

# Increase pool size
echo "DB_POOL_MAX_SIZE=30" >> .env
docker-compose restart api
```

**Problem: Inefficient queries**

```bash
# Enable query logging
echo "LOG_SQL_QUERIES=true" >> .env
docker-compose restart api

# Analyze slow queries
docker-compose logs api | grep "slow query"

# Use EXPLAIN ANALYZE
docker-compose exec postgres psql -U sctv -c "
EXPLAIN ANALYZE
SELECT * FROM projects WHERE tenant_id = 'tenant-123';
"
```

**Problem: Insufficient resources**

```bash
# Check resource limits
docker-compose exec api cat /sys/fs/cgroup/memory/memory.limit_in_bytes

# Increase resources in docker-compose.yml
services:
  api:
    deploy:
      resources:
        limits:
          cpus: '4'
          memory: 8G

# Restart
docker-compose up -d api
```

**Problem: No caching**

```bash
# Enable Redis cache (optional)
# Add to docker-compose.yml
redis:
  image: redis:alpine
  ports:
    - "6379:6379"

# Configure caching in .env
echo "CACHE_ENABLED=true" >> .env
echo "CACHE_REDIS_URL=redis://redis:6379" >> .env
echo "CACHE_TTL_SECONDS=3600" >> .env
docker-compose up -d redis
docker-compose restart api
```

#### Prevention

1. **Add appropriate indexes** on frequently queried columns
2. **Monitor query performance** regularly
3. **Implement caching** for expensive operations
4. **Use connection pooling** efficiently
5. **Set up auto-scaling** based on load

---

### Memory Issues

#### Symptom: Out of memory errors or container crashes

**Error Messages:**
```
ERROR: Out of memory
FATAL: Killed by kernel OOM killer
```

#### Diagnosis

```bash
# 1. Check memory usage
docker stats

# 2. Check memory limits
docker-compose exec api cat /sys/fs/cgroup/memory/memory.limit_in_bytes

# 3. Check for memory leaks
# Monitor over time
watch -n 5 'docker stats --no-stream'

# 4. Check heap size (if applicable)
curl http://localhost:3000/metrics | grep process_resident_memory_bytes

# 5. Check logs for OOM
docker-compose logs api | grep -i "out of memory\|OOM"
dmesg | grep -i "out of memory"
```

#### Resolution

**Problem: Memory limit too low**

```bash
# Increase memory limit in docker-compose.yml
services:
  api:
    deploy:
      resources:
        limits:
          memory: 4G
        reservations:
          memory: 2G

# Restart
docker-compose up -d api
```

**Problem: Memory leak**

```bash
# Restart service to clear memory
docker-compose restart api

# Monitor memory over time
watch -n 10 'docker stats --no-stream api'

# If leak persists, check application code
# Enable heap profiling (Rust)
# Set RUSTFLAGS="-C target-cpu=native"
# Use valgrind or heaptrack for profiling
```

**Problem: Large scan consuming memory**

```bash
# Check scan sizes
docker-compose exec postgres psql -U sctv -c "
SELECT
  s.id,
  s.project_id,
  count(d.*) AS dep_count,
  pg_size_pretty(pg_total_relation_size('dependencies')) AS table_size
FROM scans s
LEFT JOIN dependencies d ON d.scan_id = s.id
GROUP BY s.id, s.project_id
ORDER BY dep_count DESC
LIMIT 10;
"

# Process large scans in batches
echo "SCAN_BATCH_SIZE=100" >> .env
docker-compose restart worker
```

**Problem: Too many concurrent jobs**

```bash
# Reduce worker pool size
echo "WORKER_POOL_SIZE=4" >> .env
docker-compose restart worker

# Limit concurrent jobs
echo "JOB_CONCURRENCY_LIMIT=10" >> .env
docker-compose restart worker
```

#### Prevention

1. **Set appropriate memory limits**
2. **Monitor memory usage** with alerts
3. **Implement memory-efficient algorithms**
4. **Process large datasets** in batches
5. **Regular restarts** if memory leaks persist

---

## Log Analysis

### View Logs by Severity

```bash
# Error logs only
docker-compose logs api | grep ERROR

# Warning and error logs
docker-compose logs api | grep -E "WARN|ERROR"

# With context (5 lines before/after)
docker-compose logs api | grep -B 5 -A 5 ERROR
```

### Search Logs by Pattern

```bash
# Find all authentication failures
docker-compose logs api | grep "auth.*failed"

# Find slow queries
docker-compose logs api | grep "slow query"

# Find specific tenant activity
docker-compose logs api | grep "tenant-123"

# Find request ID
docker-compose logs api | grep "request_id=abc123"
```

### Aggregate Logs

```bash
# Count errors by type
docker-compose logs api | grep ERROR | cut -d' ' -f5- | sort | uniq -c | sort -rn

# Error rate over time (last hour)
docker-compose logs --since 1h api | grep ERROR | wc -l

# Top 10 error messages
docker-compose logs api | grep ERROR | awk -F'message=' '{print $2}' | sort | uniq -c | sort -rn | head -10
```

### Export Logs

```bash
# Export to file
docker-compose logs api > api-logs.txt

# Export JSON logs for analysis
docker-compose logs --no-color api | jq '.' > api-logs.json

# Send to log aggregation service
docker-compose logs --no-color api | \
  jq -c '.' | \
  curl -X POST http://logstash:5000 --data-binary @-
```

---

## Debug Mode

### Enable Debug Logging

```bash
# In .env
echo "LOG_LEVEL=debug" >> .env
echo "LOG_SQL_QUERIES=true" >> .env
docker-compose restart api worker

# Temporarily enable debug mode
docker-compose exec api kill -USR1 1  # Send signal to increase log level
```

### Debug API Requests

```bash
# Enable request logging
echo "LOG_HTTP_REQUESTS=true" >> .env
docker-compose restart api

# View request logs
docker-compose logs -f api | grep "request_id"
```

### Debug Worker Jobs

```bash
# Enable job logging
echo "LOG_JOB_EXECUTION=true" >> .env
docker-compose restart worker

# View job logs
docker-compose logs -f worker | grep "job_id"

# Dry run a scan
docker-compose exec api sctv-cli scan \
  --project-id=proj-123 \
  --dry-run \
  --verbose
```

### Debug Database Queries

```bash
# Enable PostgreSQL query logging
docker-compose exec postgres psql -U postgres -c "
ALTER SYSTEM SET log_statement = 'all';
SELECT pg_reload_conf();
"

# View query logs
docker-compose logs postgres | grep "LOG:  execute"

# Disable query logging
docker-compose exec postgres psql -U postgres -c "
ALTER SYSTEM SET log_statement = 'none';
SELECT pg_reload_conf();
"
```

### Interactive Debugging

```bash
# Start shell in API container
docker-compose exec api /bin/bash

# Start shell in worker container
docker-compose exec worker /bin/bash

# Connect to PostgreSQL
docker-compose exec postgres psql -U sctv

# Run CLI commands
docker-compose exec api sctv-cli --help
docker-compose exec api sctv-cli scan test --file=/app/test/package.json
```

---

## Support Escalation

### Before Escalating

Collect the following information:

```bash
#!/bin/bash
# collect-diagnostics.sh

OUTPUT_DIR="sctv-diagnostics-$(date +%Y%m%d-%H%M%S)"
mkdir -p "$OUTPUT_DIR"

# System information
uname -a > "$OUTPUT_DIR/system-info.txt"
df -h > "$OUTPUT_DIR/disk-usage.txt"
free -h > "$OUTPUT_DIR/memory-usage.txt"

# Service status
docker-compose ps > "$OUTPUT_DIR/docker-ps.txt"

# Service logs (last 1000 lines)
docker-compose logs --tail=1000 api > "$OUTPUT_DIR/api-logs.txt"
docker-compose logs --tail=1000 worker > "$OUTPUT_DIR/worker-logs.txt"
docker-compose logs --tail=1000 postgres > "$OUTPUT_DIR/postgres-logs.txt"

# Configuration (sanitized)
docker-compose exec api env | grep -v PASSWORD | grep -v SECRET > "$OUTPUT_DIR/env-vars.txt"
docker-compose config > "$OUTPUT_DIR/docker-compose-config.yml"

# Health checks
curl -s http://localhost:3000/health/detailed > "$OUTPUT_DIR/health-check.json"

# Metrics
curl -s http://localhost:3000/metrics > "$OUTPUT_DIR/metrics-api.txt"
curl -s http://localhost:3001/metrics > "$OUTPUT_DIR/metrics-worker.txt"

# Database diagnostics
docker-compose exec postgres psql -U sctv -c "
SELECT version();" > "$OUTPUT_DIR/postgres-version.txt"

docker-compose exec postgres psql -U sctv -c "
SELECT count(*), state FROM pg_stat_activity GROUP BY state;" > "$OUTPUT_DIR/postgres-connections.txt"

# Compress
tar -czf "$OUTPUT_DIR.tar.gz" "$OUTPUT_DIR"
echo "Diagnostics collected: $OUTPUT_DIR.tar.gz"
```

### Contact Support

**Email:** support@sctv.example.com

**Include:**
1. Description of the issue
2. Steps to reproduce
3. Expected vs actual behavior
4. Diagnostics bundle (from script above)
5. SCTV version: `docker-compose exec api sctv-cli version`
6. Deployment type (Docker Compose, Kubernetes, etc.)

**Severity Levels:**

- **P0 (Critical):** Service completely down, data loss
- **P1 (High):** Major functionality broken, no workaround
- **P2 (Medium):** Feature not working, workaround available
- **P3 (Low):** Minor issue, enhancement request

### Emergency Contacts

**Production Issues:**
- **Slack:** #sctv-incidents
- **PagerDuty:** Page on-call engineer
- **Phone:** +1-xxx-xxx-xxxx (24/7)

---

## Next Steps

- [Monitoring](monitoring.md) - Set up proactive monitoring
- [Security Hardening](security.md) - Secure your deployment
- [Backup and Recovery](backup.md) - Protect your data

---

**Troubleshoot effectively!** Most issues can be resolved quickly with proper diagnostics.
