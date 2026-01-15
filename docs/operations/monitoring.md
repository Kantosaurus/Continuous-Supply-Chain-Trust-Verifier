# Monitoring Guide

**Version:** 0.1.0

Complete guide for monitoring and observability in SCTV production environments.

---

## Table of Contents

- [Overview](#overview)
- [Metrics Overview](#metrics-overview)
- [Prometheus Integration](#prometheus-integration)
- [Key Metrics to Monitor](#key-metrics-to-monitor)
- [Grafana Dashboard Setup](#grafana-dashboard-setup)
- [Alert Rules](#alert-rules)
- [Log Aggregation](#log-aggregation)
- [Distributed Tracing](#distributed-tracing)
- [Health Check Endpoints](#health-check-endpoints)
- [SLA Monitoring](#sla-monitoring)
- [Capacity Planning](#capacity-planning)

---

## Overview

SCTV provides comprehensive observability features for production monitoring:

- **Metrics:** Prometheus-compatible metrics for performance tracking
- **Logs:** Structured JSON logs for analysis and debugging
- **Traces:** Distributed tracing for request flow visualization
- **Health Checks:** Liveness and readiness probes
- **Alerts:** Automated alerting for critical issues

### Monitoring Architecture

```
┌──────────────────────────────────────────────────────────────┐
│                    Monitoring Stack                          │
│                                                              │
│  ┌─────────────┐      ┌─────────────┐      ┌────────────┐  │
│  │ Prometheus  │──────│  Grafana    │──────│Alertmanager│  │
│  │  (Metrics)  │      │ (Dashboards)│      │  (Alerts)  │  │
│  └──────┬──────┘      └─────────────┘      └────────────┘  │
│         │                                                    │
└─────────┼────────────────────────────────────────────────────┘
          │
          │ /metrics endpoint
          │
┌─────────┴────────────────────────────────────────────────────┐
│                      SCTV Services                           │
│  ┌──────────┐      ┌──────────┐      ┌────────────────┐     │
│  │   API    │      │  Worker  │      │   Dashboard    │     │
│  │  Server  │      │  Service │      │                │     │
│  └──────────┘      └──────────┘      └────────────────┘     │
└──────────────────────────────────────────────────────────────┘

┌──────────────────────────────────────────────────────────────┐
│                    Logging Stack                             │
│  ┌─────────────┐      ┌─────────────┐      ┌────────────┐  │
│  │Elasticsearch│──────│   Kibana    │      │  Logstash  │  │
│  │    or       │      │     or      │      │     or     │  │
│  │    Loki     │      │   Grafana   │      │  Promtail  │  │
│  └─────────────┘      └─────────────┘      └────────────┘  │
└──────────────────────────────────────────────────────────────┘

┌──────────────────────────────────────────────────────────────┐
│                    Tracing Stack                             │
│  ┌─────────────┐      ┌─────────────┐                       │
│  │   Jaeger    │  or  │   Zipkin    │                       │
│  │             │      │             │                       │
│  └─────────────┘      └─────────────┘                       │
└──────────────────────────────────────────────────────────────┘
```

---

## Metrics Overview

SCTV exposes metrics at the `/metrics` endpoint in Prometheus format.

### Metric Categories

| Category | Description | Examples |
|----------|-------------|----------|
| **System** | Resource utilization | CPU, memory, disk, network |
| **API** | HTTP request metrics | Request rate, latency, errors |
| **Worker** | Job processing metrics | Job throughput, queue depth, failures |
| **Database** | Database operations | Query time, connections, transactions |
| **Business** | Application-specific | Scans completed, alerts generated, threats detected |

### Naming Convention

SCTV follows Prometheus naming conventions:

- **Counters:** `sctv_*_total` (e.g., `sctv_api_requests_total`)
- **Gauges:** `sctv_*` (e.g., `sctv_worker_pool_size`)
- **Histograms:** `sctv_*_duration_seconds` (e.g., `sctv_scan_duration_seconds`)
- **Summaries:** `sctv_*_quantile` (e.g., `sctv_api_latency_quantile`)

---

## Prometheus Integration

### Prometheus Configuration

Create `prometheus.yml`:

```yaml
global:
  scrape_interval: 15s
  evaluation_interval: 15s
  external_labels:
    cluster: 'sctv-production'
    environment: 'prod'

# Alertmanager configuration
alerting:
  alertmanagers:
    - static_configs:
        - targets: ['alertmanager:9093']

# Rule files
rule_files:
  - 'alerts/sctv-alerts.yml'

# Scrape configurations
scrape_configs:
  # SCTV API servers
  - job_name: 'sctv-api'
    static_configs:
      - targets: ['api-1:3000', 'api-2:3000', 'api-3:3000']
    metrics_path: '/metrics'
    scrape_interval: 10s
    scrape_timeout: 5s

  # SCTV Worker services
  - job_name: 'sctv-worker'
    static_configs:
      - targets: ['worker-1:3001', 'worker-2:3001']
    metrics_path: '/metrics'
    scrape_interval: 10s

  # PostgreSQL exporter
  - job_name: 'postgresql'
    static_configs:
      - targets: ['postgres-exporter:9187']
    scrape_interval: 15s

  # Node exporter (system metrics)
  - job_name: 'node'
    static_configs:
      - targets: ['node-exporter:9100']
    scrape_interval: 15s

  # Kubernetes service discovery (if using K8s)
  - job_name: 'kubernetes-pods'
    kubernetes_sd_configs:
      - role: pod
        namespaces:
          names:
            - sctv
    relabel_configs:
      - source_labels: [__meta_kubernetes_pod_annotation_prometheus_io_scrape]
        action: keep
        regex: true
      - source_labels: [__meta_kubernetes_pod_annotation_prometheus_io_path]
        action: replace
        target_label: __metrics_path__
        regex: (.+)
      - source_labels: [__address__, __meta_kubernetes_pod_annotation_prometheus_io_port]
        action: replace
        regex: ([^:]+)(?::\d+)?;(\d+)
        replacement: $1:$2
        target_label: __address__
```

### Docker Compose Monitoring Stack

Create `docker-compose.monitoring.yml`:

```yaml
version: '3.8'

services:
  # Prometheus
  prometheus:
    image: prom/prometheus:latest
    restart: always
    command:
      - '--config.file=/etc/prometheus/prometheus.yml'
      - '--storage.tsdb.path=/prometheus'
      - '--storage.tsdb.retention.time=30d'
      - '--web.enable-lifecycle'
    ports:
      - "9090:9090"
    volumes:
      - ./prometheus.yml:/etc/prometheus/prometheus.yml
      - ./alerts:/etc/prometheus/alerts
      - prometheus_data:/prometheus
    networks:
      - monitoring

  # Grafana
  grafana:
    image: grafana/grafana:latest
    restart: always
    environment:
      - GF_SECURITY_ADMIN_PASSWORD=${GRAFANA_PASSWORD}
      - GF_INSTALL_PLUGINS=grafana-piechart-panel
      - GF_SERVER_ROOT_URL=https://grafana.example.com
    ports:
      - "3003:3000"
    volumes:
      - grafana_data:/var/lib/grafana
      - ./grafana/provisioning:/etc/grafana/provisioning
      - ./grafana/dashboards:/var/lib/grafana/dashboards
    networks:
      - monitoring

  # Alertmanager
  alertmanager:
    image: prom/alertmanager:latest
    restart: always
    command:
      - '--config.file=/etc/alertmanager/alertmanager.yml'
      - '--storage.path=/alertmanager'
    ports:
      - "9093:9093"
    volumes:
      - ./alertmanager.yml:/etc/alertmanager/alertmanager.yml
      - alertmanager_data:/alertmanager
    networks:
      - monitoring

  # PostgreSQL Exporter
  postgres-exporter:
    image: prometheuscommunity/postgres-exporter:latest
    restart: always
    environment:
      DATA_SOURCE_NAME: "postgresql://sctv:${POSTGRES_PASSWORD}@postgres:5432/sctv?sslmode=disable"
    ports:
      - "9187:9187"
    networks:
      - monitoring
      - sctv-internal

  # Node Exporter
  node-exporter:
    image: prom/node-exporter:latest
    restart: always
    command:
      - '--path.procfs=/host/proc'
      - '--path.sysfs=/host/sys'
      - '--collector.filesystem.mount-points-exclude=^/(sys|proc|dev|host|etc)($$|/)'
    ports:
      - "9100:9100"
    volumes:
      - /proc:/host/proc:ro
      - /sys:/host/sys:ro
      - /:/rootfs:ro
    networks:
      - monitoring

volumes:
  prometheus_data:
  grafana_data:
  alertmanager_data:

networks:
  monitoring:
    driver: bridge
  sctv-internal:
    external: true
```

### Deploy Monitoring Stack

```bash
# Set Grafana password
export GRAFANA_PASSWORD=$(openssl rand -hex 16)
echo "Grafana password: $GRAFANA_PASSWORD" > .grafana-password

# Start monitoring stack
docker-compose -f docker-compose.monitoring.yml up -d

# Verify services
docker-compose -f docker-compose.monitoring.yml ps

# Check Prometheus targets
curl http://localhost:9090/api/v1/targets

# Access Grafana
echo "Grafana: http://localhost:3003"
echo "Username: admin"
echo "Password: $GRAFANA_PASSWORD"
```

---

## Key Metrics to Monitor

### API Server Metrics

#### HTTP Request Metrics

```promql
# Request rate (requests per second)
rate(sctv_api_requests_total[5m])

# Error rate
rate(sctv_api_requests_total{status=~"5.."}[5m])

# Request duration (p95, p99)
histogram_quantile(0.95, rate(sctv_api_request_duration_seconds_bucket[5m]))
histogram_quantile(0.99, rate(sctv_api_request_duration_seconds_bucket[5m]))

# Requests by endpoint
sum(rate(sctv_api_requests_total[5m])) by (endpoint)

# 4xx error rate
sum(rate(sctv_api_requests_total{status=~"4.."}[5m])) by (endpoint)
```

**Exported Metrics:**

```
# Request counter
sctv_api_requests_total{method="GET", endpoint="/api/v1/projects", status="200"} 12534

# Request duration histogram
sctv_api_request_duration_seconds_bucket{endpoint="/api/v1/projects", le="0.1"} 9821
sctv_api_request_duration_seconds_bucket{endpoint="/api/v1/projects", le="0.5"} 12234
sctv_api_request_duration_seconds_bucket{endpoint="/api/v1/projects", le="1.0"} 12500
sctv_api_request_duration_seconds_sum{endpoint="/api/v1/projects"} 892.34
sctv_api_request_duration_seconds_count{endpoint="/api/v1/projects"} 12534

# Active connections
sctv_api_active_connections 45
```

#### Authentication Metrics

```promql
# Authentication success rate
rate(sctv_auth_attempts_total{status="success"}[5m]) / rate(sctv_auth_attempts_total[5m])

# Failed login attempts
rate(sctv_auth_attempts_total{status="failed"}[5m])
```

**Exported Metrics:**

```
# Authentication attempts
sctv_auth_attempts_total{method="jwt", status="success"} 8234
sctv_auth_attempts_total{method="jwt", status="failed"} 45
sctv_auth_attempts_total{method="api_key", status="success"} 1234

# Active sessions
sctv_auth_active_sessions{tenant_id="tenant-123"} 12
```

### Worker Metrics

#### Job Processing Metrics

```promql
# Job completion rate
rate(sctv_jobs_completed_total[5m])

# Job failure rate
rate(sctv_jobs_failed_total[5m])

# Job queue depth
sctv_jobs_queued

# Job processing duration
histogram_quantile(0.95, rate(sctv_job_duration_seconds_bucket[5m]))

# Worker pool utilization
sctv_worker_pool_busy / sctv_worker_pool_size
```

**Exported Metrics:**

```
# Job counters
sctv_jobs_completed_total{job_type="ScanProject"} 3421
sctv_jobs_failed_total{job_type="ScanProject"} 23
sctv_jobs_retried_total{job_type="ScanProject"} 15

# Job queue metrics
sctv_jobs_queued{status="pending"} 45
sctv_jobs_queued{status="processing"} 8
sctv_jobs_queued{status="failed"} 2

# Job duration histogram
sctv_job_duration_seconds_bucket{job_type="ScanProject", le="30"} 2100
sctv_job_duration_seconds_bucket{job_type="ScanProject", le="60"} 3200
sctv_job_duration_seconds_bucket{job_type="ScanProject", le="120"} 3400
sctv_job_duration_seconds_sum{job_type="ScanProject"} 98234.56
sctv_job_duration_seconds_count{job_type="ScanProject"} 3421

# Worker pool metrics
sctv_worker_pool_size 8
sctv_worker_pool_busy 6
sctv_worker_pool_idle 2
```

#### Scan Metrics

```promql
# Scans per minute
rate(sctv_scans_completed_total[1m]) * 60

# Dependencies scanned
rate(sctv_dependencies_scanned_total[5m])

# Threats detected
rate(sctv_threats_detected_total[5m])
```

**Exported Metrics:**

```
# Scan metrics
sctv_scans_completed_total{ecosystem="npm"} 1234
sctv_scans_completed_total{ecosystem="pypi"} 876
sctv_dependencies_scanned_total 45678
sctv_threats_detected_total{threat_type="typosquatting"} 23
sctv_threats_detected_total{threat_type="tampering"} 5
```

### Database Metrics

```promql
# Active database connections
pg_stat_activity_count

# Query duration (p95)
histogram_quantile(0.95, rate(sctv_db_query_duration_seconds_bucket[5m]))

# Slow queries (> 1s)
rate(sctv_db_slow_queries_total[5m])

# Transaction rate
rate(pg_stat_database_xact_commit[5m])
```

**Exported Metrics:**

```
# Database connection pool
sctv_db_connections_active 12
sctv_db_connections_idle 8
sctv_db_connections_max 20

# Query metrics
sctv_db_query_duration_seconds_bucket{query="select_project", le="0.01"} 8234
sctv_db_query_duration_seconds_bucket{query="select_project", le="0.1"} 9821
sctv_db_slow_queries_total{query="complex_scan_results"} 5
```

### System Metrics

```promql
# CPU usage
100 - (avg by (instance) (rate(node_cpu_seconds_total{mode="idle"}[5m])) * 100)

# Memory usage
(1 - (node_memory_MemAvailable_bytes / node_memory_MemTotal_bytes)) * 100

# Disk usage
(1 - (node_filesystem_avail_bytes / node_filesystem_size_bytes)) * 100

# Network throughput
rate(node_network_receive_bytes_total[5m])
rate(node_network_transmit_bytes_total[5m])
```

### Business Metrics

```promql
# Active projects
sctv_projects_total

# Alerts by severity
sum(sctv_alerts_active) by (severity)

# SLA compliance
sctv_scans_on_time_total / sctv_scans_total

# Tenant usage
sum(sctv_dependencies_scanned_total) by (tenant_id)
```

**Exported Metrics:**

```
# Business metrics
sctv_projects_total 245
sctv_alerts_active{severity="critical"} 3
sctv_alerts_active{severity="high"} 12
sctv_alerts_active{severity="medium"} 45
sctv_scans_on_time_total 9876
sctv_scans_total 10000
sctv_tenant_users{tenant_id="tenant-123"} 25
```

---

## Grafana Dashboard Setup

### Add Prometheus Data Source

1. Access Grafana: `http://localhost:3003`
2. Navigate to **Configuration > Data Sources**
3. Click **Add data source**
4. Select **Prometheus**
5. Configure:
   - **Name:** SCTV Prometheus
   - **URL:** `http://prometheus:9090`
   - **Access:** Server (default)
6. Click **Save & Test**

### Import SCTV Dashboards

Create `grafana/provisioning/dashboards/sctv-dashboards.yml`:

```yaml
apiVersion: 1

providers:
  - name: 'SCTV Dashboards'
    orgId: 1
    folder: 'SCTV'
    type: file
    disableDeletion: false
    updateIntervalSeconds: 10
    allowUiUpdates: true
    options:
      path: /var/lib/grafana/dashboards
```

### Dashboard: SCTV Overview

Create `grafana/dashboards/sctv-overview.json`:

```json
{
  "dashboard": {
    "title": "SCTV Overview",
    "tags": ["sctv", "overview"],
    "timezone": "browser",
    "panels": [
      {
        "title": "Request Rate",
        "targets": [
          {
            "expr": "sum(rate(sctv_api_requests_total[5m]))",
            "legendFormat": "Requests/sec"
          }
        ],
        "type": "graph"
      },
      {
        "title": "Error Rate",
        "targets": [
          {
            "expr": "sum(rate(sctv_api_requests_total{status=~\"5..\"}[5m]))",
            "legendFormat": "Errors/sec"
          }
        ],
        "type": "graph"
      },
      {
        "title": "Job Queue Depth",
        "targets": [
          {
            "expr": "sctv_jobs_queued{status=\"pending\"}",
            "legendFormat": "Pending Jobs"
          }
        ],
        "type": "graph"
      },
      {
        "title": "Active Alerts by Severity",
        "targets": [
          {
            "expr": "sum(sctv_alerts_active) by (severity)",
            "legendFormat": "{{severity}}"
          }
        ],
        "type": "pie"
      }
    ]
  }
}
```

### Pre-built Dashboards

SCTV provides pre-built Grafana dashboards:

1. **SCTV Overview** - High-level system health
2. **API Performance** - Request rates, latency, errors
3. **Worker Performance** - Job processing, queue depth
4. **Database Performance** - Query performance, connections
5. **Business Metrics** - Scans, alerts, threats
6. **Tenant Usage** - Per-tenant resource consumption

Import dashboards:

```bash
# Download dashboards
curl -O https://grafana.com/api/dashboards/sctv-overview/revisions/1/download

# Or use Grafana provisioning (recommended)
cp dashboards/*.json /var/lib/grafana/dashboards/
```

---

## Alert Rules

### Alertmanager Configuration

Create `alertmanager.yml`:

```yaml
global:
  resolve_timeout: 5m
  smtp_smarthost: 'smtp.example.com:587'
  smtp_from: 'alertmanager@example.com'
  smtp_auth_username: 'alertmanager@example.com'
  smtp_auth_password: '${SMTP_PASSWORD}'

# Alert routing
route:
  receiver: 'default'
  group_by: ['alertname', 'cluster', 'service']
  group_wait: 10s
  group_interval: 10s
  repeat_interval: 12h
  routes:
    # Critical alerts go to PagerDuty
    - match:
        severity: critical
      receiver: 'pagerduty'
      continue: true

    # High severity to Slack
    - match:
        severity: high
      receiver: 'slack'
      continue: true

    # All alerts to email
    - match_re:
        severity: (warning|critical|high)
      receiver: 'email'

# Notification channels
receivers:
  - name: 'default'
    webhook_configs:
      - url: 'http://sctv-api:3000/api/v1/webhooks/alerts'

  - name: 'pagerduty'
    pagerduty_configs:
      - service_key: '${PAGERDUTY_SERVICE_KEY}'
        description: '{{ .GroupLabels.alertname }}'
        severity: '{{ .CommonLabels.severity }}'

  - name: 'slack'
    slack_configs:
      - api_url: '${SLACK_WEBHOOK_URL}'
        channel: '#sctv-alerts'
        title: '{{ .CommonAnnotations.summary }}'
        text: '{{ .CommonAnnotations.description }}'

  - name: 'email'
    email_configs:
      - to: 'ops-team@example.com'
        headers:
          Subject: '[SCTV] {{ .GroupLabels.alertname }}'

# Inhibition rules
inhibit_rules:
  # Silence warnings if critical alert is firing
  - source_match:
      severity: 'critical'
    target_match:
      severity: 'warning'
    equal: ['alertname', 'cluster', 'service']
```

### Prometheus Alert Rules

Create `alerts/sctv-alerts.yml`:

```yaml
groups:
  - name: sctv_api
    interval: 30s
    rules:
      # High error rate
      - alert: HighAPIErrorRate
        expr: |
          sum(rate(sctv_api_requests_total{status=~"5.."}[5m])) /
          sum(rate(sctv_api_requests_total[5m])) > 0.05
        for: 5m
        labels:
          severity: critical
          component: api
        annotations:
          summary: "High API error rate detected"
          description: "Error rate is {{ $value | humanizePercentage }} (threshold: 5%)"

      # High latency
      - alert: HighAPILatency
        expr: |
          histogram_quantile(0.95,
            rate(sctv_api_request_duration_seconds_bucket[5m])
          ) > 2.0
        for: 10m
        labels:
          severity: high
          component: api
        annotations:
          summary: "API latency is high"
          description: "P95 latency is {{ $value }}s (threshold: 2s)"

      # API service down
      - alert: APIServiceDown
        expr: up{job="sctv-api"} == 0
        for: 2m
        labels:
          severity: critical
          component: api
        annotations:
          summary: "API service is down"
          description: "API service {{ $labels.instance }} is unreachable"

  - name: sctv_worker
    interval: 30s
    rules:
      # High job failure rate
      - alert: HighJobFailureRate
        expr: |
          sum(rate(sctv_jobs_failed_total[5m])) /
          sum(rate(sctv_jobs_completed_total[5m]) + rate(sctv_jobs_failed_total[5m])) > 0.1
        for: 10m
        labels:
          severity: high
          component: worker
        annotations:
          summary: "High job failure rate"
          description: "Job failure rate is {{ $value | humanizePercentage }} (threshold: 10%)"

      # Job queue backing up
      - alert: JobQueueBackingUp
        expr: sctv_jobs_queued{status="pending"} > 100
        for: 15m
        labels:
          severity: warning
          component: worker
        annotations:
          summary: "Job queue is backing up"
          description: "{{ $value }} jobs pending (threshold: 100)"

      # Worker pool saturation
      - alert: WorkerPoolSaturated
        expr: |
          sctv_worker_pool_busy / sctv_worker_pool_size > 0.9
        for: 15m
        labels:
          severity: warning
          component: worker
        annotations:
          summary: "Worker pool is saturated"
          description: "Worker utilization is {{ $value | humanizePercentage }}"

      # Worker service down
      - alert: WorkerServiceDown
        expr: up{job="sctv-worker"} == 0
        for: 5m
        labels:
          severity: critical
          component: worker
        annotations:
          summary: "Worker service is down"
          description: "Worker {{ $labels.instance }} is unreachable"

  - name: sctv_database
    interval: 30s
    rules:
      # High database connection usage
      - alert: HighDatabaseConnectionUsage
        expr: |
          sctv_db_connections_active / sctv_db_connections_max > 0.8
        for: 10m
        labels:
          severity: warning
          component: database
        annotations:
          summary: "High database connection usage"
          description: "Connection pool is {{ $value | humanizePercentage }} utilized"

      # Slow queries
      - alert: SlowDatabaseQueries
        expr: rate(sctv_db_slow_queries_total[5m]) > 1
        for: 10m
        labels:
          severity: warning
          component: database
        annotations:
          summary: "Slow database queries detected"
          description: "{{ $value }} slow queries per second"

      # Database down
      - alert: DatabaseDown
        expr: up{job="postgresql"} == 0
        for: 2m
        labels:
          severity: critical
          component: database
        annotations:
          summary: "Database is down"
          description: "PostgreSQL database is unreachable"

  - name: sctv_system
    interval: 30s
    rules:
      # High CPU usage
      - alert: HighCPUUsage
        expr: |
          100 - (avg by (instance) (rate(node_cpu_seconds_total{mode="idle"}[5m])) * 100) > 80
        for: 10m
        labels:
          severity: warning
          component: system
        annotations:
          summary: "High CPU usage"
          description: "CPU usage is {{ $value }}% on {{ $labels.instance }}"

      # High memory usage
      - alert: HighMemoryUsage
        expr: |
          (1 - (node_memory_MemAvailable_bytes / node_memory_MemTotal_bytes)) * 100 > 85
        for: 10m
        labels:
          severity: warning
          component: system
        annotations:
          summary: "High memory usage"
          description: "Memory usage is {{ $value }}% on {{ $labels.instance }}"

      # Disk space low
      - alert: DiskSpaceLow
        expr: |
          (1 - (node_filesystem_avail_bytes{mountpoint="/"} / node_filesystem_size_bytes{mountpoint="/"})) * 100 > 80
        for: 10m
        labels:
          severity: warning
          component: system
        annotations:
          summary: "Disk space low"
          description: "Disk usage is {{ $value }}% on {{ $labels.instance }}"

      # Disk space critical
      - alert: DiskSpaceCritical
        expr: |
          (1 - (node_filesystem_avail_bytes{mountpoint="/"} / node_filesystem_size_bytes{mountpoint="/"})) * 100 > 90
        for: 5m
        labels:
          severity: critical
          component: system
        annotations:
          summary: "Disk space critical"
          description: "Disk usage is {{ $value }}% on {{ $labels.instance }}"

  - name: sctv_business
    interval: 1m
    rules:
      # SLA breach
      - alert: SLABreach
        expr: |
          (sctv_scans_on_time_total / sctv_scans_total) < 0.95
        for: 1h
        labels:
          severity: high
          component: business
        annotations:
          summary: "SLA breach detected"
          description: "Scan completion rate is {{ $value | humanizePercentage }} (SLA: 95%)"

      # Critical alerts not resolved
      - alert: CriticalAlertsNotResolved
        expr: |
          sctv_alerts_active{severity="critical"} > 0
        for: 4h
        labels:
          severity: high
          component: business
        annotations:
          summary: "Critical alerts not resolved"
          description: "{{ $value }} critical alerts open for >4 hours"
```

### Test Alert Rules

```bash
# Reload Prometheus configuration
curl -X POST http://localhost:9090/-/reload

# Check rules are loaded
curl http://localhost:9090/api/v1/rules | jq '.data.groups[].name'

# Trigger test alert (API)
for i in {1..100}; do
  curl -f http://localhost:3000/nonexistent || true
done

# Check firing alerts
curl http://localhost:9090/api/v1/alerts | jq '.data.alerts[] | select(.state=="firing")'

# Check Alertmanager
curl http://localhost:9093/api/v2/alerts | jq '.'
```

---

## Log Aggregation

### Option 1: ELK Stack (Elasticsearch, Logstash, Kibana)

#### Deploy ELK Stack

```yaml
# docker-compose.elk.yml
version: '3.8'

services:
  elasticsearch:
    image: docker.elastic.co/elasticsearch/elasticsearch:8.11.0
    environment:
      - discovery.type=single-node
      - "ES_JAVA_OPTS=-Xms2g -Xmx2g"
      - xpack.security.enabled=false
    ports:
      - "9200:9200"
    volumes:
      - elasticsearch_data:/usr/share/elasticsearch/data
    networks:
      - elk

  logstash:
    image: docker.elastic.co/logstash/logstash:8.11.0
    ports:
      - "5000:5000"
      - "9600:9600"
    volumes:
      - ./logstash/pipeline:/usr/share/logstash/pipeline
    networks:
      - elk
      - sctv-internal
    depends_on:
      - elasticsearch

  kibana:
    image: docker.elastic.co/kibana/kibana:8.11.0
    ports:
      - "5601:5601"
    environment:
      ELASTICSEARCH_HOSTS: http://elasticsearch:9200
    networks:
      - elk
    depends_on:
      - elasticsearch

volumes:
  elasticsearch_data:

networks:
  elk:
  sctv-internal:
    external: true
```

#### Logstash Configuration

Create `logstash/pipeline/sctv.conf`:

```ruby
input {
  # TCP input for JSON logs
  tcp {
    port => 5000
    codec => json_lines
  }

  # File input (if using file logging)
  file {
    path => "/var/log/sctv/*.log"
    codec => json
    start_position => "beginning"
  }
}

filter {
  # Parse SCTV logs
  if [app] == "sctv" {
    # Extract fields
    mutate {
      add_field => {
        "[@metadata][target_index]" => "sctv-logs-%{+YYYY.MM.dd}"
      }
    }

    # Parse log level
    if [level] {
      mutate {
        lowercase => [ "level" ]
      }
    }

    # Geoip for client IPs
    if [client_ip] {
      geoip {
        source => "client_ip"
        target => "geoip"
      }
    }

    # Add timestamp
    date {
      match => [ "timestamp", "ISO8601" ]
      target => "@timestamp"
    }
  }
}

output {
  elasticsearch {
    hosts => ["elasticsearch:9200"]
    index => "%{[@metadata][target_index]}"
  }

  # Debug output
  stdout {
    codec => rubydebug
  }
}
```

#### Configure SCTV to Send Logs

In `.env`:

```bash
# JSON logging to stdout
LOG_FORMAT=json
LOG_LEVEL=info

# Logstash output
LOGSTASH_HOST=logstash
LOGSTASH_PORT=5000
```

### Option 2: Grafana Loki + Promtail

#### Deploy Loki Stack

```yaml
# docker-compose.loki.yml
version: '3.8'

services:
  loki:
    image: grafana/loki:latest
    ports:
      - "3100:3100"
    command: -config.file=/etc/loki/local-config.yaml
    volumes:
      - loki_data:/loki
      - ./loki-config.yaml:/etc/loki/local-config.yaml
    networks:
      - monitoring

  promtail:
    image: grafana/promtail:latest
    command: -config.file=/etc/promtail/config.yml
    volumes:
      - ./promtail-config.yaml:/etc/promtail/config.yml
      - /var/log:/var/log
      - /var/lib/docker/containers:/var/lib/docker/containers:ro
    networks:
      - monitoring

volumes:
  loki_data:

networks:
  monitoring:
    external: true
```

#### Loki Configuration

Create `loki-config.yaml`:

```yaml
auth_enabled: false

server:
  http_listen_port: 3100

ingester:
  lifecycler:
    ring:
      kvstore:
        store: inmemory
      replication_factor: 1
  chunk_idle_period: 5m
  chunk_retain_period: 30s

schema_config:
  configs:
    - from: 2020-05-15
      store: boltdb
      object_store: filesystem
      schema: v11
      index:
        prefix: index_
        period: 24h

storage_config:
  boltdb:
    directory: /loki/index
  filesystem:
    directory: /loki/chunks

limits_config:
  enforce_metric_name: false
  reject_old_samples: true
  reject_old_samples_max_age: 168h

chunk_store_config:
  max_look_back_period: 0s

table_manager:
  retention_deletes_enabled: true
  retention_period: 720h
```

#### Promtail Configuration

Create `promtail-config.yaml`:

```yaml
server:
  http_listen_port: 9080
  grpc_listen_port: 0

positions:
  filename: /tmp/positions.yaml

clients:
  - url: http://loki:3100/loki/api/v1/push

scrape_configs:
  # Docker container logs
  - job_name: docker
    docker_sd_configs:
      - host: unix:///var/run/docker.sock
        refresh_interval: 5s
    relabel_configs:
      - source_labels: ['__meta_docker_container_name']
        regex: '/(.*)'
        target_label: 'container'
      - source_labels: ['__meta_docker_container_label_com_docker_compose_service']
        target_label: 'service'
    pipeline_stages:
      - json:
          expressions:
            level: level
            msg: message
            timestamp: timestamp
      - labels:
          level:
      - timestamp:
          source: timestamp
          format: RFC3339
```

### Query Logs in Grafana

Add Loki data source:
1. Configuration > Data Sources > Add Loki
2. URL: `http://loki:3100`

Example queries:

```logql
# All SCTV logs
{service="sctv-api"}

# Error logs only
{service="sctv-api"} |= "ERROR"

# Logs for specific tenant
{service="sctv-api"} | json | tenant_id="tenant-123"

# HTTP 500 errors
{service="sctv-api"} | json | status="500"

# Rate of errors
rate({service="sctv-api"} |= "ERROR" [5m])
```

---

## Distributed Tracing

### Option 1: Jaeger

#### Deploy Jaeger

```yaml
# docker-compose.jaeger.yml
version: '3.8'

services:
  jaeger:
    image: jaegertracing/all-in-one:latest
    environment:
      - COLLECTOR_ZIPKIN_HOST_PORT=:9411
      - COLLECTOR_OTLP_ENABLED=true
    ports:
      - "5775:5775/udp"
      - "6831:6831/udp"
      - "6832:6832/udp"
      - "5778:5778"
      - "16686:16686"  # UI
      - "14268:14268"
      - "14250:14250"
      - "9411:9411"
    networks:
      - monitoring
```

#### Configure SCTV for Tracing

In `.env`:

```bash
# Enable tracing
ENABLE_TRACING=true
JAEGER_AGENT_HOST=jaeger
JAEGER_AGENT_PORT=6831
TRACE_SAMPLE_RATE=0.1  # Sample 10% of requests
```

#### View Traces

Access Jaeger UI: `http://localhost:16686`

Search for:
- **Service:** sctv-api
- **Operation:** GET /api/v1/projects
- **Tags:** tenant_id=tenant-123

### Option 2: Zipkin

```yaml
zipkin:
  image: openzipkin/zipkin:latest
  ports:
    - "9411:9411"
  networks:
    - monitoring
```

Configure in `.env`:

```bash
ENABLE_TRACING=true
ZIPKIN_ENDPOINT=http://zipkin:9411/api/v2/spans
```

---

## Health Check Endpoints

SCTV provides comprehensive health check endpoints:

### Liveness Probe

**Endpoint:** `GET /health`

**Purpose:** Check if service is alive

```bash
curl http://localhost:3000/health
```

Response:

```json
{
  "status": "healthy",
  "timestamp": "2026-01-15T10:30:45Z"
}
```

### Readiness Probe

**Endpoint:** `GET /health/ready`

**Purpose:** Check if service is ready to accept traffic

```bash
curl http://localhost:3000/health/ready
```

Response:

```json
{
  "status": "ready",
  "checks": {
    "database": "ok",
    "job_queue": "ok",
    "cache": "ok"
  },
  "timestamp": "2026-01-15T10:30:45Z"
}
```

### Detailed Health Check

**Endpoint:** `GET /health/detailed`

**Purpose:** Comprehensive health information

```bash
curl http://localhost:3000/health/detailed
```

Response:

```json
{
  "status": "healthy",
  "version": "0.1.0",
  "uptime_seconds": 86400,
  "checks": {
    "database": {
      "status": "ok",
      "response_time_ms": 5,
      "connections_active": 12,
      "connections_max": 20
    },
    "job_queue": {
      "status": "ok",
      "pending_jobs": 45,
      "processing_jobs": 8
    },
    "external_services": {
      "npm_registry": "ok",
      "pypi_registry": "ok"
    }
  },
  "timestamp": "2026-01-15T10:30:45Z"
}
```

### Kubernetes Health Checks

```yaml
livenessProbe:
  httpGet:
    path: /health
    port: 3000
  initialDelaySeconds: 30
  periodSeconds: 10
  timeoutSeconds: 5
  failureThreshold: 3

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

## SLA Monitoring

### Define SLIs (Service Level Indicators)

```yaml
# SLI Definitions
sli:
  # Availability: % of successful requests
  availability:
    query: |
      sum(rate(sctv_api_requests_total{status!~"5.."}[5m])) /
      sum(rate(sctv_api_requests_total[5m]))
    target: 0.999  # 99.9%

  # Latency: % of requests < 500ms
  latency:
    query: |
      histogram_quantile(0.95,
        rate(sctv_api_request_duration_seconds_bucket[5m])
      ) < 0.5
    target: 0.95  # 95% of requests

  # Scan completion: % of scans completed on time
  scan_completion:
    query: |
      sctv_scans_on_time_total / sctv_scans_total
    target: 0.95  # 95%
```

### SLA Dashboard

Create Grafana dashboard panel:

```json
{
  "title": "SLA Compliance",
  "targets": [
    {
      "expr": "sum(rate(sctv_api_requests_total{status!~\"5..\"}[5m])) / sum(rate(sctv_api_requests_total[5m]))",
      "legendFormat": "Availability (target: 99.9%)"
    },
    {
      "expr": "histogram_quantile(0.95, rate(sctv_api_request_duration_seconds_bucket[5m])) < 0.5",
      "legendFormat": "Latency (target: 95%)"
    }
  ],
  "thresholds": [
    {
      "value": 0.999,
      "color": "green"
    },
    {
      "value": 0.99,
      "color": "yellow"
    },
    {
      "value": 0,
      "color": "red"
    }
  ]
}
```

### SLA Reporting

Generate monthly SLA report:

```bash
#!/bin/bash
# generate-sla-report.sh

MONTH=$(date -d "last month" +%Y-%m)
OUTPUT="sla-report-$MONTH.json"

# Query Prometheus
curl -G 'http://prometheus:9090/api/v1/query_range' \
  --data-urlencode 'query=sum(rate(sctv_api_requests_total{status!~"5.."}[1h])) / sum(rate(sctv_api_requests_total[1h]))' \
  --data-urlencode "start=$(date -d "$MONTH-01" +%s)" \
  --data-urlencode "end=$(date -d "$MONTH-01 +1 month" +%s)" \
  --data-urlencode 'step=3600' \
  | jq '.' > "$OUTPUT"

echo "SLA report generated: $OUTPUT"
```

---

## Capacity Planning

### Resource Utilization Tracking

```promql
# CPU utilization trend (7 days)
avg_over_time(
  (100 - (avg by (instance) (rate(node_cpu_seconds_total{mode="idle"}[5m])) * 100))
[7d:1h])

# Memory utilization trend
avg_over_time(
  ((1 - (node_memory_MemAvailable_bytes / node_memory_MemTotal_bytes)) * 100)
[7d:1h])

# Database connection pool trend
avg_over_time(sctv_db_connections_active[7d:1h])

# Job queue depth trend
max_over_time(sctv_jobs_queued{status="pending"}[7d:1h])
```

### Growth Metrics

```promql
# Project growth rate
rate(sctv_projects_total[7d])

# Scan volume growth
rate(sctv_scans_completed_total[7d])

# Storage growth (GB per week)
rate(pg_database_size_bytes[7d]) * 7 * 24 * 3600 / (1024^3)
```

### Capacity Thresholds

| Resource | Warning | Critical | Action |
|----------|---------|----------|--------|
| CPU | 70% | 85% | Add API server |
| Memory | 80% | 90% | Increase memory |
| Disk | 75% | 85% | Expand storage |
| DB Connections | 70% | 85% | Increase pool size |
| Job Queue | 100 | 500 | Add workers |

### Capacity Planning Report

```bash
#!/bin/bash
# capacity-planning-report.sh

echo "=== SCTV Capacity Planning Report ==="
echo "Date: $(date)"
echo ""

# Current resource usage
echo "## Current Resource Usage"
curl -s 'http://prometheus:9090/api/v1/query?query=100-(avg(rate(node_cpu_seconds_total{mode="idle"}[5m]))*100)' \
  | jq -r '.data.result[0].value[1]' \
  | xargs printf "CPU Usage: %.2f%%\n"

curl -s 'http://prometheus:9090/api/v1/query?query=(1-(node_memory_MemAvailable_bytes/node_memory_MemTotal_bytes))*100' \
  | jq -r '.data.result[0].value[1]' \
  | xargs printf "Memory Usage: %.2f%%\n"

echo ""

# Growth trends
echo "## Growth Trends (7 days)"
curl -s 'http://prometheus:9090/api/v1/query?query=rate(sctv_projects_total[7d])' \
  | jq -r '.data.result[0].value[1]' \
  | xargs printf "Projects per day: %.2f\n"

curl -s 'http://prometheus:9090/api/v1/query?query=rate(sctv_scans_completed_total[7d])' \
  | jq -r '.data.result[0].value[1]' \
  | xargs printf "Scans per day: %.2f\n"

echo ""

# Recommendations
echo "## Recommendations"
# Add logic based on thresholds
```

### Forecasting

Use Prometheus prediction functions:

```promql
# Predict when disk will be full (linear regression)
predict_linear(node_filesystem_avail_bytes{mountpoint="/"}[7d], 30*24*3600)

# Predict CPU usage in 7 days
predict_linear(
  avg(rate(node_cpu_seconds_total{mode="idle"}[5m]))[7d:1h],
  7*24*3600
)
```

---

## Next Steps

- [Troubleshooting](troubleshooting.md) - Diagnose and fix issues
- [Security Hardening](security.md) - Production security best practices
- [Backup and Recovery](backup.md) - Data protection strategies

---

**Monitor effectively!** Set up comprehensive monitoring before issues occur in production.
