# Backup and Recovery Guide

**Version:** 0.1.0

Complete guide for backup strategies, disaster recovery, and data protection in SCTV.

---

## Table of Contents

- [Overview](#overview)
- [Backup Strategy](#backup-strategy)
- [PostgreSQL Backup Methods](#postgresql-backup-methods)
- [Point-in-Time Recovery](#point-in-time-recovery)
- [Automated Backup Scripts](#automated-backup-scripts)
- [Backup Verification](#backup-verification)
- [Disaster Recovery Plan](#disaster-recovery-plan)
- [RTO/RPO Targets](#rtorpo-targets)
- [Multi-Region Backup](#multi-region-backup)
- [Restore Procedures](#restore-procedures)
- [Data Retention Policies](#data-retention-policies)

---

## Overview

Data protection is critical for SCTV deployments. This guide covers comprehensive backup and recovery strategies to ensure business continuity.

### Backup Goals

1. **Data Protection:** Prevent data loss from hardware failure, human error, or attacks
2. **Business Continuity:** Minimize downtime during disasters
3. **Compliance:** Meet regulatory requirements for data retention
4. **Version History:** Maintain historical data for auditing

### Key Concepts

**RTO (Recovery Time Objective):** Maximum acceptable downtime
- Target: < 1 hour for production systems

**RPO (Recovery Point Objective):** Maximum acceptable data loss
- Target: < 15 minutes for critical data

**Backup Types:**
- **Full Backup:** Complete copy of all data
- **Incremental Backup:** Only changes since last backup
- **Differential Backup:** Changes since last full backup
- **Continuous Backup:** Real-time replication

---

## Backup Strategy

### 3-2-1 Backup Rule

Follow the industry-standard 3-2-1 rule:

- **3** copies of data (1 primary + 2 backups)
- **2** different media types (disk + cloud)
- **1** copy offsite (different location)

### Backup Schedule

| Backup Type | Frequency | Retention | Storage |
|-------------|-----------|-----------|---------|
| Full Database | Daily at 2 AM | 30 days | Local + S3 |
| Incremental (WAL) | Continuous | 7 days | Local + S3 |
| Configuration | On change | 90 days | Git + S3 |
| Application Logs | Daily | 90 days | S3 |
| Audit Logs | Daily | 2 years | S3 Glacier |

### What to Back Up

**Essential Data:**
- PostgreSQL database (all tables)
- Configuration files (.env, docker-compose.yml)
- TLS certificates
- Application secrets (encrypted)

**Optional Data:**
- Application logs (can be regenerated)
- Temporary files
- Cache data

**Do NOT Back Up:**
- Container images (store in registry)
- Build artifacts (can be rebuilt)
- OS files (use infrastructure as code)

---

## PostgreSQL Backup Methods

### Method 1: pg_dump (Logical Backup)

**Pros:** Simple, portable, selective backup
**Cons:** Slower for large databases, locks tables

```bash
# Full database backup
pg_dump -U sctv -Fc -f backup.dump sctv

# Backup with compression
pg_dump -U sctv -Fc -Z9 -f backup-compressed.dump sctv

# Backup specific tables
pg_dump -U sctv -Fc -t projects -t dependencies -f backup-tables.dump sctv

# Backup to SQL file
pg_dump -U sctv -f backup.sql sctv

# Backup with schema only (no data)
pg_dump -U sctv -s -f schema-only.sql sctv
```

**Docker Compose:**

```bash
# Backup from running container
docker-compose exec postgres pg_dump -U sctv -Fc sctv > backup.dump

# Backup to container volume
docker-compose exec postgres pg_dump -U sctv -Fc -f /backups/backup-$(date +%Y%m%d-%H%M%S).dump sctv
```

### Method 2: pg_basebackup (Physical Backup)

**Pros:** Fast, includes all databases, supports PITR
**Cons:** Requires PostgreSQL running, larger size

```bash
# Base backup
pg_basebackup -U replication -D /backups/base -Ft -z -P

# With WAL files for PITR
pg_basebackup -U replication -D /backups/base -Ft -z -Xs -P
```

**Docker Compose:**

```bash
# Create replication user first
docker-compose exec postgres psql -U postgres -c "
CREATE USER replication WITH REPLICATION ENCRYPTED PASSWORD 'replication_password';
"

# Perform base backup
docker-compose exec postgres pg_basebackup \
  -U replication \
  -D /backups/base-$(date +%Y%m%d-%H%M%S) \
  -Ft -z -Xs -P
```

### Method 3: File System Snapshot

**Pros:** Very fast, atomic snapshot
**Cons:** Requires PostgreSQL stopped or specific filesystem (LVM, ZFS)

```bash
# Stop PostgreSQL
docker-compose stop postgres

# Create LVM snapshot
lvcreate --size 10G --snapshot --name postgres-snapshot /dev/vg0/postgres

# Start PostgreSQL
docker-compose start postgres

# Mount and backup snapshot
mkdir -p /mnt/snapshot
mount /dev/vg0/postgres-snapshot /mnt/snapshot
tar -czf postgres-snapshot-$(date +%Y%m%d).tar.gz /mnt/snapshot

# Remove snapshot
umount /mnt/snapshot
lvremove -f /dev/vg0/postgres-snapshot
```

### Method 4: Continuous Archiving (WAL)

**Pros:** Minimal data loss, supports PITR
**Cons:** Complex setup, requires storage space

```bash
# Configure in postgresql.conf
wal_level = replica
archive_mode = on
archive_command = 'cp %p /backups/wal/%f'
archive_timeout = 60  # Force WAL switch every 60 seconds

# Or archive to S3
archive_command = 'aws s3 cp %p s3://my-bucket/wal/%f'
```

---

## Point-in-Time Recovery

### Enable WAL Archiving

**1. Configure PostgreSQL**

Edit `postgresql.conf`:

```conf
# WAL settings
wal_level = replica
archive_mode = on
archive_command = 'test ! -f /backups/wal/%f && cp %p /backups/wal/%f'
archive_timeout = 60
max_wal_senders = 3
wal_keep_size = 1GB
```

**2. Create archive directory**

```bash
mkdir -p /backups/wal
chown postgres:postgres /backups/wal
chmod 700 /backups/wal
```

**3. Restart PostgreSQL**

```bash
docker-compose restart postgres
```

### Perform PITR Recovery

**1. Stop PostgreSQL**

```bash
docker-compose stop postgres
```

**2. Restore base backup**

```bash
# Remove old data
rm -rf /var/lib/postgresql/data/*

# Restore base backup
tar -xzf /backups/base-backup.tar.gz -C /var/lib/postgresql/data
```

**3. Create recovery.conf**

```bash
cat > /var/lib/postgresql/data/recovery.conf << EOF
restore_command = 'cp /backups/wal/%f %p'
recovery_target_time = '2026-01-15 14:30:00'
recovery_target_action = 'promote'
EOF
```

**4. Start PostgreSQL**

```bash
docker-compose start postgres
```

**5. Verify recovery**

```bash
docker-compose exec postgres psql -U sctv -c "SELECT pg_is_in_recovery();"
docker-compose exec postgres psql -U sctv -c "SELECT now();"
```

---

## Automated Backup Scripts

### Daily Backup Script

Create `/usr/local/bin/sctv-backup.sh`:

```bash
#!/bin/bash
set -e

# Configuration
BACKUP_DIR="/backups"
S3_BUCKET="s3://my-sctv-backups"
RETENTION_DAYS=30
DATE=$(date +%Y%m%d-%H%M%S)
LOG_FILE="/var/log/sctv-backup.log"

# Logging function
log() {
    echo "[$(date +'%Y-%m-%d %H:%M:%S')] $1" | tee -a "$LOG_FILE"
}

log "Starting SCTV backup..."

# Create backup directory
mkdir -p "$BACKUP_DIR/$DATE"

# 1. Backup PostgreSQL database
log "Backing up PostgreSQL database..."
docker-compose exec -T postgres pg_dump -U sctv -Fc sctv > "$BACKUP_DIR/$DATE/database.dump"

# Verify database backup
if [ ! -f "$BACKUP_DIR/$DATE/database.dump" ]; then
    log "ERROR: Database backup failed!"
    exit 1
fi

# 2. Backup configuration files
log "Backing up configuration files..."
tar -czf "$BACKUP_DIR/$DATE/config.tar.gz" \
    .env \
    docker-compose.yml \
    docker-compose.prod.yml \
    nginx.conf \
    prometheus.yml \
    grafana/

# 3. Backup TLS certificates
log "Backing up TLS certificates..."
tar -czf "$BACKUP_DIR/$DATE/certs.tar.gz" certs/

# 4. Backup application logs (last 7 days)
log "Backing up application logs..."
find /var/log/sctv -type f -mtime -7 -name "*.log" | \
    tar -czf "$BACKUP_DIR/$DATE/logs.tar.gz" -T -

# 5. Calculate checksums
log "Calculating checksums..."
cd "$BACKUP_DIR/$DATE"
sha256sum * > checksums.txt

# 6. Create backup manifest
log "Creating backup manifest..."
cat > "$BACKUP_DIR/$DATE/manifest.json" << EOF
{
  "timestamp": "$(date -Iseconds)",
  "version": "$(docker-compose exec -T api sctv-cli version 2>/dev/null || echo 'unknown')",
  "files": [
    "database.dump",
    "config.tar.gz",
    "certs.tar.gz",
    "logs.tar.gz",
    "checksums.txt"
  ],
  "size_bytes": $(du -sb "$BACKUP_DIR/$DATE" | cut -f1)
}
EOF

# 7. Compress entire backup
log "Compressing backup..."
cd "$BACKUP_DIR"
tar -czf "sctv-backup-$DATE.tar.gz" "$DATE/"
rm -rf "$DATE/"

# 8. Encrypt backup (optional)
log "Encrypting backup..."
gpg --encrypt --recipient backup@example.com "sctv-backup-$DATE.tar.gz"
rm "sctv-backup-$DATE.tar.gz"

# 9. Upload to S3
log "Uploading to S3..."
aws s3 cp "sctv-backup-$DATE.tar.gz.gpg" "$S3_BUCKET/daily/"

# 10. Verify upload
if aws s3 ls "$S3_BUCKET/daily/sctv-backup-$DATE.tar.gz.gpg" > /dev/null; then
    log "Backup uploaded successfully"
else
    log "ERROR: Backup upload failed!"
    exit 1
fi

# 11. Remove old local backups
log "Cleaning up old backups..."
find "$BACKUP_DIR" -name "sctv-backup-*.tar.gz.gpg" -mtime +7 -delete

# 12. Remove old S3 backups
log "Removing old S3 backups..."
aws s3 ls "$S3_BUCKET/daily/" | while read -r line; do
    createDate=$(echo "$line" | awk {'print $1" "$2'})
    createDate=$(date -d "$createDate" +%s)
    olderThan=$(date -d "-$RETENTION_DAYS days" +%s)
    if [[ $createDate -lt $olderThan ]]; then
        fileName=$(echo "$line" | awk {'print $4'})
        if [[ $fileName != "" ]]; then
            aws s3 rm "$S3_BUCKET/daily/$fileName"
            log "Deleted old backup: $fileName"
        fi
    fi
done

# 13. Send notification
log "Sending notification..."
BACKUP_SIZE=$(du -h "sctv-backup-$DATE.tar.gz.gpg" | cut -f1)
curl -X POST "$SLACK_WEBHOOK_URL" \
    -H "Content-Type: application/json" \
    -d "{\"text\":\"SCTV backup completed successfully\n• Date: $DATE\n• Size: $BACKUP_SIZE\n• Location: $S3_BUCKET/daily/\"}"

log "Backup completed successfully!"
```

### Set up cron job

```bash
# Make script executable
chmod +x /usr/local/bin/sctv-backup.sh

# Create cron job (daily at 2 AM)
cat > /etc/cron.d/sctv-backup << EOF
0 2 * * * root /usr/local/bin/sctv-backup.sh >> /var/log/sctv-backup.log 2>&1
EOF

# Test script
/usr/local/bin/sctv-backup.sh
```

### Continuous WAL Archiving Script

Create `/usr/local/bin/sctv-wal-archive.sh`:

```bash
#!/bin/bash
set -e

# Configuration
WAL_ARCHIVE_DIR="/backups/wal"
S3_BUCKET="s3://my-sctv-backups/wal"

# Archive WAL file
WAL_FILE="$1"
WAL_PATH="$2"

# Copy to local archive
cp "$WAL_PATH" "$WAL_ARCHIVE_DIR/$WAL_FILE"

# Upload to S3
aws s3 cp "$WAL_PATH" "$S3_BUCKET/$WAL_FILE"

# Verify upload
if aws s3 ls "$S3_BUCKET/$WAL_FILE" > /dev/null; then
    exit 0
else
    exit 1
fi
```

Update `postgresql.conf`:

```conf
archive_command = '/usr/local/bin/sctv-wal-archive.sh %f %p'
```

---

## Backup Verification

### Automated Verification Script

Create `/usr/local/bin/sctv-verify-backup.sh`:

```bash
#!/bin/bash
set -e

BACKUP_FILE="$1"
LOG_FILE="/var/log/sctv-backup-verify.log"

log() {
    echo "[$(date +'%Y-%m-%d %H:%M:%S')] $1" | tee -a "$LOG_FILE"
}

log "Verifying backup: $BACKUP_FILE"

# 1. Verify file exists
if [ ! -f "$BACKUP_FILE" ]; then
    log "ERROR: Backup file not found!"
    exit 1
fi

# 2. Verify file is not empty
if [ ! -s "$BACKUP_FILE" ]; then
    log "ERROR: Backup file is empty!"
    exit 1
fi

# 3. Decrypt backup
log "Decrypting backup..."
gpg --decrypt "$BACKUP_FILE" > "${BACKUP_FILE%.gpg}"

# 4. Verify tar integrity
log "Verifying archive integrity..."
if ! tar -tzf "${BACKUP_FILE%.gpg}" > /dev/null; then
    log "ERROR: Archive is corrupted!"
    exit 1
fi

# 5. Extract backup
VERIFY_DIR="/tmp/backup-verify-$$"
mkdir -p "$VERIFY_DIR"
tar -xzf "${BACKUP_FILE%.gpg}" -C "$VERIFY_DIR"

# 6. Verify checksums
log "Verifying checksums..."
cd "$VERIFY_DIR"/*
if ! sha256sum -c checksums.txt; then
    log "ERROR: Checksum verification failed!"
    exit 1
fi

# 7. Verify database backup
log "Verifying database backup..."
if [ -f "database.dump" ]; then
    # Check if it's a valid PostgreSQL dump
    if ! pg_restore --list database.dump > /dev/null 2>&1; then
        log "ERROR: Invalid database backup!"
        exit 1
    fi

    # Count objects in backup
    OBJECT_COUNT=$(pg_restore --list database.dump | grep -c "^[0-9]")
    log "Database backup contains $OBJECT_COUNT objects"
else
    log "ERROR: Database backup not found!"
    exit 1
fi

# 8. Verify manifest
log "Verifying manifest..."
if [ -f "manifest.json" ]; then
    jq '.' manifest.json > /dev/null || (log "ERROR: Invalid manifest JSON!" && exit 1)
    log "Backup timestamp: $(jq -r '.timestamp' manifest.json)"
    log "Backup size: $(jq -r '.size_bytes' manifest.json) bytes"
else
    log "WARNING: Manifest not found!"
fi

# Cleanup
rm -rf "$VERIFY_DIR"
rm "${BACKUP_FILE%.gpg}"

log "Backup verification completed successfully!"
exit 0
```

### Schedule verification

```bash
# Verify latest backup daily at 4 AM
cat > /etc/cron.d/sctv-backup-verify << EOF
0 4 * * * root /usr/local/bin/sctv-verify-backup.sh \$(ls -t /backups/sctv-backup-*.tar.gz.gpg | head -1) >> /var/log/sctv-backup-verify.log 2>&1
EOF
```

### Test Restore (Monthly)

```bash
#!/bin/bash
# test-restore.sh
# Perform test restore monthly to verify backup integrity

set -e

BACKUP_FILE="$1"
TEST_DB="sctv_restore_test"
LOG_FILE="/var/log/sctv-restore-test.log"

log() {
    echo "[$(date +'%Y-%m-%d %H:%M:%S')] $1" | tee -a "$LOG_FILE"
}

log "Starting restore test..."

# Extract and decrypt backup
TEMP_DIR=$(mktemp -d)
gpg --decrypt "$BACKUP_FILE" | tar -xz -C "$TEMP_DIR"

# Create test database
docker-compose exec -T postgres psql -U postgres -c "DROP DATABASE IF EXISTS $TEST_DB;"
docker-compose exec -T postgres psql -U postgres -c "CREATE DATABASE $TEST_DB;"

# Restore to test database
log "Restoring database..."
docker-compose exec -T postgres pg_restore \
    -U postgres \
    -d "$TEST_DB" \
    < "$TEMP_DIR/*/database.dump"

# Verify data
log "Verifying restored data..."
PROJECTS_COUNT=$(docker-compose exec -T postgres psql -U postgres -d "$TEST_DB" -t -c "SELECT COUNT(*) FROM projects;")
log "Projects count: $PROJECTS_COUNT"

ALERTS_COUNT=$(docker-compose exec -T postgres psql -U postgres -d "$TEST_DB" -t -c "SELECT COUNT(*) FROM alerts;")
log "Alerts count: $ALERTS_COUNT"

# Cleanup
docker-compose exec -T postgres psql -U postgres -c "DROP DATABASE $TEST_DB;"
rm -rf "$TEMP_DIR"

log "Restore test completed successfully!"
```

---

## Disaster Recovery Plan

### Disaster Recovery Scenarios

#### Scenario 1: Database Corruption

**Symptoms:**
- Database errors in logs
- Queries failing
- Data inconsistencies

**Recovery Steps:**

```bash
# 1. Stop applications
docker-compose stop api worker dashboard

# 2. Assess damage
docker-compose exec postgres pg_dump -U sctv sctv > /tmp/corrupted.dump

# 3. Restore from latest backup
LATEST_BACKUP=$(ls -t /backups/sctv-backup-*.tar.gz.gpg | head -1)
gpg --decrypt "$LATEST_BACKUP" | tar -xz -C /tmp/restore

# 4. Drop and recreate database
docker-compose exec postgres psql -U postgres -c "DROP DATABASE sctv;"
docker-compose exec postgres psql -U postgres -c "CREATE DATABASE sctv;"

# 5. Restore data
docker-compose exec -T postgres pg_restore -U sctv -d sctv < /tmp/restore/*/database.dump

# 6. Verify integrity
docker-compose exec postgres psql -U sctv -c "SELECT COUNT(*) FROM projects;"

# 7. Restart applications
docker-compose start api worker dashboard

# 8. Monitor logs
docker-compose logs -f
```

**Recovery Time:** ~30 minutes
**Data Loss:** < 24 hours (last full backup)

#### Scenario 2: Complete Server Failure

**Recovery Steps:**

```bash
# 1. Provision new server
# Use infrastructure as code (Terraform, etc.)

# 2. Install Docker and Docker Compose
curl -fsSL https://get.docker.com | sh
apt-get install docker-compose-plugin

# 3. Clone configuration from git
git clone https://github.com/example/sctv-config.git /opt/sctv
cd /opt/sctv

# 4. Download latest backup from S3
aws s3 cp s3://my-sctv-backups/daily/$(aws s3 ls s3://my-sctv-backups/daily/ | sort | tail -1 | awk '{print $4}') backup.tar.gz.gpg

# 5. Decrypt and extract
gpg --decrypt backup.tar.gz.gpg | tar -xz

# 6. Restore configuration
cp */config.tar.gz .
tar -xzf config.tar.gz

# 7. Restore certificates
tar -xzf */certs.tar.gz

# 8. Start PostgreSQL
docker-compose up -d postgres

# 9. Wait for PostgreSQL to be ready
while ! docker-compose exec postgres pg_isready -U sctv; do sleep 1; done

# 10. Restore database
docker-compose exec -T postgres pg_restore -U sctv -d sctv < */database.dump

# 11. Start all services
docker-compose up -d

# 12. Verify services
curl https://sctv.example.com/health

# 13. Monitor
docker-compose logs -f
```

**Recovery Time:** ~2 hours
**Data Loss:** < 24 hours

#### Scenario 3: Ransomware Attack

**Recovery Steps:**

```bash
# 1. Isolate infected systems immediately
# Disconnect from network, power off if necessary

# 2. Do NOT pay ransom

# 3. Identify infection vector and remove
# Scan with antivirus, check for backdoors

# 4. Provision clean infrastructure
# New servers, new credentials, new secrets

# 5. Restore from backup BEFORE infection date
# Use verified clean backup

# 6. Implement additional security measures
# See Security Hardening guide

# 7. Change all credentials
./rotate-all-secrets.sh

# 8. Monitor closely for reinfection
```

**Recovery Time:** ~4 hours
**Data Loss:** Depends on backup before infection

---

## RTO/RPO Targets

### Production SLAs

| Priority | RTO | RPO | Backup Method | Cost |
|----------|-----|-----|---------------|------|
| Critical | < 1 hour | < 15 min | Continuous replication | High |
| High | < 4 hours | < 1 hour | Incremental + WAL | Medium |
| Medium | < 24 hours | < 24 hours | Daily full backup | Low |
| Low | < 72 hours | < 7 days | Weekly backup | Very Low |

### Calculating RTO

```
RTO = Detection Time + Mobilization Time + Recovery Time + Verification Time

Example (Critical):
- Detection: 5 minutes (monitoring alerts)
- Mobilization: 10 minutes (team response)
- Recovery: 30 minutes (restore from replica)
- Verification: 15 minutes (health checks)
Total: 60 minutes
```

### Calculating RPO

```
RPO = Backup Frequency + Backup Duration

Example (High):
- Backup Frequency: 1 hour (incremental + WAL)
- Backup Duration: 5 minutes
Total: 65 minutes ≈ 1 hour

With continuous WAL archiving:
- WAL archive: every 60 seconds
- In-flight transactions: < 60 seconds
Total: < 1 minute
```

### Improve RTO/RPO

**To reduce RTO:**
1. Implement hot standby replicas
2. Automate failover procedures
3. Pre-stage recovery environments
4. Practice disaster recovery drills

**To reduce RPO:**
1. Enable continuous WAL archiving
2. Use synchronous replication
3. Increase backup frequency
4. Implement change data capture (CDC)

---

## Multi-Region Backup

### AWS Multi-Region Setup

```bash
# Primary region: us-east-1
# Backup region: us-west-2

# 1. Create S3 buckets in both regions
aws s3 mb s3://sctv-backups-us-east-1 --region us-east-1
aws s3 mb s3://sctv-backups-us-west-2 --region us-west-2

# 2. Enable versioning
aws s3api put-bucket-versioning \
    --bucket sctv-backups-us-east-1 \
    --versioning-configuration Status=Enabled

# 3. Configure cross-region replication
aws s3api put-bucket-replication \
    --bucket sctv-backups-us-east-1 \
    --replication-configuration file://replication-config.json

# replication-config.json
{
  "Role": "arn:aws:iam::ACCOUNT:role/s3-replication-role",
  "Rules": [
    {
      "Status": "Enabled",
      "Priority": 1,
      "Filter": {},
      "Destination": {
        "Bucket": "arn:aws:s3:::sctv-backups-us-west-2",
        "ReplicationTime": {
          "Status": "Enabled",
          "Time": {
            "Minutes": 15
          }
        },
        "Metrics": {
          "Status": "Enabled"
        }
      }
    }
  ]
}

# 4. Upload backups to primary region
aws s3 cp backup.tar.gz s3://sctv-backups-us-east-1/

# Backups automatically replicate to us-west-2
```

### PostgreSQL Streaming Replication

```yaml
# docker-compose.yml
version: '3.8'

services:
  postgres-primary:
    image: postgres:14
    environment:
      POSTGRES_USER: sctv
      POSTGRES_PASSWORD: ${POSTGRES_PASSWORD}
      POSTGRES_DB: sctv
    volumes:
      - postgres_primary_data:/var/lib/postgresql/data
      - ./postgresql-primary.conf:/etc/postgresql/postgresql.conf
    command: postgres -c config_file=/etc/postgresql/postgresql.conf

  postgres-standby:
    image: postgres:14
    environment:
      POSTGRES_USER: sctv
      POSTGRES_PASSWORD: ${POSTGRES_PASSWORD}
      PGDATA: /var/lib/postgresql/data/pgdata
    volumes:
      - postgres_standby_data:/var/lib/postgresql/data
      - ./postgresql-standby.conf:/etc/postgresql/postgresql.conf
    command: postgres -c config_file=/etc/postgresql/postgresql.conf
    depends_on:
      - postgres-primary

volumes:
  postgres_primary_data:
  postgres_standby_data:
```

**Configure primary** (`postgresql-primary.conf`):

```conf
wal_level = replica
max_wal_senders = 3
wal_keep_size = 1GB
synchronous_commit = on
synchronous_standby_names = 'standby1'
```

**Configure standby** (`postgresql-standby.conf`):

```conf
hot_standby = on
primary_conninfo = 'host=postgres-primary port=5432 user=replication password=replication_password'
primary_slot_name = 'standby1'
```

---

## Restore Procedures

### Full Database Restore

```bash
#!/bin/bash
# restore-database.sh

set -e

BACKUP_FILE="$1"

if [ -z "$BACKUP_FILE" ]; then
    echo "Usage: $0 <backup-file.tar.gz.gpg>"
    exit 1
fi

# Confirmation
read -p "This will restore database from $BACKUP_FILE. Continue? (yes/no): " CONFIRM
if [ "$CONFIRM" != "yes" ]; then
    echo "Aborted."
    exit 0
fi

echo "Starting database restore..."

# Stop services
echo "Stopping services..."
docker-compose stop api worker dashboard

# Extract backup
echo "Extracting backup..."
TEMP_DIR=$(mktemp -d)
gpg --decrypt "$BACKUP_FILE" | tar -xz -C "$TEMP_DIR"
BACKUP_DIR=$(ls -d $TEMP_DIR/*)

# Backup current database (just in case)
echo "Creating safety backup of current database..."
docker-compose exec -T postgres pg_dump -U sctv -Fc sctv > /tmp/pre-restore-backup.dump

# Drop and recreate database
echo "Recreating database..."
docker-compose exec -T postgres psql -U postgres << EOF
SELECT pg_terminate_backend(pid) FROM pg_stat_activity WHERE datname = 'sctv';
DROP DATABASE IF EXISTS sctv;
CREATE DATABASE sctv;
GRANT ALL PRIVILEGES ON DATABASE sctv TO sctv;
EOF

# Restore database
echo "Restoring database..."
docker-compose exec -T postgres pg_restore \
    -U sctv \
    -d sctv \
    --no-owner \
    --no-acl \
    < "$BACKUP_DIR/database.dump"

# Verify restore
echo "Verifying restore..."
PROJECTS_COUNT=$(docker-compose exec -T postgres psql -U sctv -d sctv -t -c "SELECT COUNT(*) FROM projects;")
echo "Projects: $PROJECTS_COUNT"

USERS_COUNT=$(docker-compose exec -T postgres psql -U sctv -d sctv -t -c "SELECT COUNT(*) FROM users;")
echo "Users: $USERS_COUNT"

# Restore configuration files
echo "Restoring configuration..."
if [ -f "$BACKUP_DIR/config.tar.gz" ]; then
    tar -xzf "$BACKUP_DIR/config.tar.gz"
fi

# Restart services
echo "Starting services..."
docker-compose up -d

# Wait for services to be healthy
echo "Waiting for services to be healthy..."
sleep 10
docker-compose exec api curl -f http://localhost:3000/health || echo "Warning: Health check failed"

# Cleanup
rm -rf "$TEMP_DIR"

echo "Restore completed successfully!"
echo "Safety backup stored at: /tmp/pre-restore-backup.dump"
```

### Partial Table Restore

```bash
# Restore specific tables only
docker-compose exec -T postgres pg_restore \
    -U sctv \
    -d sctv \
    --table=projects \
    --table=dependencies \
    < backup.dump

# Or restore to temporary database first
docker-compose exec -T postgres psql -U postgres -c "CREATE DATABASE sctv_temp;"
docker-compose exec -T postgres pg_restore -U sctv -d sctv_temp < backup.dump

# Copy specific data
docker-compose exec -T postgres psql -U sctv -d sctv << EOF
INSERT INTO projects SELECT * FROM sctv_temp.projects WHERE id = 'specific-project-id';
EOF

# Cleanup
docker-compose exec -T postgres psql -U postgres -c "DROP DATABASE sctv_temp;"
```

### Point-in-Time Restore

```bash
#!/bin/bash
# pitr-restore.sh

TARGET_TIME="$1"  # e.g., "2026-01-15 14:30:00"

if [ -z "$TARGET_TIME" ]; then
    echo "Usage: $0 'YYYY-MM-DD HH:MM:SS'"
    exit 1
fi

echo "Restoring to point in time: $TARGET_TIME"

# Stop PostgreSQL
docker-compose stop postgres

# Backup current data
mv /var/lib/postgresql/data /var/lib/postgresql/data.backup-$(date +%s)

# Restore base backup
mkdir -p /var/lib/postgresql/data
tar -xzf /backups/base-backup.tar.gz -C /var/lib/postgresql/data

# Create recovery configuration
cat > /var/lib/postgresql/data/recovery.signal << EOF
EOF

cat > /var/lib/postgresql/data/postgresql.auto.conf << EOF
restore_command = 'cp /backups/wal/%f %p'
recovery_target_time = '$TARGET_TIME'
recovery_target_action = 'promote'
EOF

# Start PostgreSQL in recovery mode
docker-compose start postgres

# Monitor recovery
echo "Monitoring recovery progress..."
while docker-compose exec postgres psql -U sctv -t -c "SELECT pg_is_in_recovery();" | grep -q "t"; do
    echo "Still recovering..."
    sleep 5
done

echo "Recovery completed!"
docker-compose exec postgres psql -U sctv -c "SELECT current_timestamp;"
```

---

## Data Retention Policies

### Retention Schedule

| Data Type | Retention Period | Storage Tier | Reason |
|-----------|------------------|--------------|--------|
| Full Backups | 30 days | S3 Standard | Operational recovery |
| Full Backups | 1 year | S3 Standard-IA | Compliance |
| Full Backups | 7 years | S3 Glacier | Legal/Audit |
| Incremental | 7 days | S3 Standard | Point-in-time recovery |
| WAL Archives | 7 days | S3 Standard | Point-in-time recovery |
| Audit Logs | 2 years | S3 Standard | Compliance |
| Application Logs | 90 days | S3 Standard | Troubleshooting |
| Scan Results | 1 year | Database | Historical analysis |

### Implement Lifecycle Policies

**AWS S3 Lifecycle Policy:**

```json
{
  "Rules": [
    {
      "Id": "Move to IA after 30 days",
      "Status": "Enabled",
      "Transitions": [
        {
          "Days": 30,
          "StorageClass": "STANDARD_IA"
        },
        {
          "Days": 90,
          "StorageClass": "GLACIER"
        }
      ],
      "Expiration": {
        "Days": 2555
      },
      "Filter": {
        "Prefix": "daily/"
      }
    },
    {
      "Id": "Delete old WAL files",
      "Status": "Enabled",
      "Expiration": {
        "Days": 7
      },
      "Filter": {
        "Prefix": "wal/"
      }
    },
    {
      "Id": "Archive audit logs",
      "Status": "Enabled",
      "Transitions": [
        {
          "Days": 90,
          "StorageClass": "GLACIER"
        }
      ],
      "Expiration": {
        "Days": 730
      },
      "Filter": {
        "Prefix": "audit/"
      }
    }
  ]
}
```

Apply policy:

```bash
aws s3api put-bucket-lifecycle-configuration \
    --bucket sctv-backups \
    --lifecycle-configuration file://lifecycle-policy.json
```

### Database Data Retention

```sql
-- Create retention policy function
CREATE OR REPLACE FUNCTION apply_retention_policy()
RETURNS void AS $$
BEGIN
  -- Delete old scan results (1 year)
  DELETE FROM scans
  WHERE created_at < now() - interval '1 year'
  AND status = 'completed';

  -- Delete old audit logs (2 years)
  DELETE FROM audit_log
  WHERE timestamp < now() - interval '2 years';

  -- Delete old job records (30 days)
  DELETE FROM jobs
  WHERE created_at < now() - interval '30 days'
  AND status IN ('completed', 'failed');

  -- Archive old alerts (90 days since resolved)
  UPDATE alerts
  SET archived = true
  WHERE resolved_at < now() - interval '90 days'
  AND archived = false;

  RAISE NOTICE 'Retention policy applied successfully';
END;
$$ LANGUAGE plpgsql;

-- Schedule with pg_cron (requires pg_cron extension)
CREATE EXTENSION IF NOT EXISTS pg_cron;

-- Run retention policy daily at 3 AM
SELECT cron.schedule('apply-retention-policy', '0 3 * * *', 'SELECT apply_retention_policy();');
```

Or create cron job:

```bash
# /etc/cron.d/sctv-retention
0 3 * * * postgres docker-compose exec -T postgres psql -U sctv -c "SELECT apply_retention_policy();" >> /var/log/sctv-retention.log 2>&1
```

---

## Backup Monitoring

### Monitor Backup Success

```bash
#!/bin/bash
# monitor-backups.sh

LOG_FILE="/var/log/sctv-backup.log"
ALERT_EMAIL="ops@example.com"

# Check last backup time
LAST_BACKUP=$(ls -t /backups/sctv-backup-*.tar.gz.gpg 2>/dev/null | head -1)
if [ -z "$LAST_BACKUP" ]; then
    echo "ERROR: No backups found!"
    echo "No backups found in /backups/" | mail -s "SCTV Backup Alert" $ALERT_EMAIL
    exit 1
fi

# Check backup age
BACKUP_AGE=$(($(date +%s) - $(stat -c %Y "$LAST_BACKUP")))
MAX_AGE=$((48 * 3600))  # 48 hours

if [ $BACKUP_AGE -gt $MAX_AGE ]; then
    echo "WARNING: Last backup is $(($BACKUP_AGE / 3600)) hours old!"
    echo "Last backup is too old: $(($BACKUP_AGE / 3600)) hours" | mail -s "SCTV Backup Alert" $ALERT_EMAIL
    exit 1
fi

# Check S3 sync
LATEST_LOCAL=$(basename "$LAST_BACKUP")
if ! aws s3 ls "s3://my-sctv-backups/daily/$LATEST_LOCAL" > /dev/null; then
    echo "ERROR: Latest backup not found in S3!"
    echo "Latest backup not uploaded to S3" | mail -s "SCTV Backup Alert" $ALERT_EMAIL
    exit 1
fi

echo "Backup monitoring: OK"
exit 0
```

### Prometheus Metrics

Add backup metrics to SCTV:

```
# Backup age (seconds since last backup)
sctv_backup_age_seconds 86400

# Backup size (bytes)
sctv_backup_size_bytes 5368709120

# Backup success
sctv_backup_success{type="full"} 1
sctv_backup_success{type="incremental"} 1

# Last backup timestamp
sctv_backup_last_success_timestamp 1705324800
```

Alert rules:

```yaml
- alert: BackupTooOld
  expr: time() - sctv_backup_last_success_timestamp > 86400
  for: 1h
  labels:
    severity: critical
  annotations:
    summary: "SCTV backup is too old"
    description: "Last successful backup was {{ $value | humanizeDuration }} ago"

- alert: BackupFailed
  expr: sctv_backup_success == 0
  for: 30m
  labels:
    severity: high
  annotations:
    summary: "SCTV backup failed"
    description: "Last backup attempt failed"
```

---

## Next Steps

- [Monitoring](monitoring.md) - Monitor backup health
- [Security](security.md) - Encrypt backups
- [Troubleshooting](troubleshooting.md) - Restore issues

---

**Protect your data!** Regular backups and tested restore procedures are essential for business continuity.
