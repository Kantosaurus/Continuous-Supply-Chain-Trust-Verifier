//! Integration tests for the audit log repository.

mod common;

use common::{create_test_tenant, create_test_user, TestDb};
use sctv_core::traits::{AuditLogRepository, TenantRepository, UserRepository};
use sctv_core::{AuditAction, AuditLog, AuditLogFilter, ResourceType};
use sctv_db::{PgAuditLogRepository, PgTenantRepository, PgUserRepository};
use std::net::{IpAddr, Ipv4Addr};

#[tokio::test]
async fn test_create_and_find_audit_log_by_id() {
    let db = TestDb::new().await;
    let tenant_repo = PgTenantRepository::new(db.pool.clone());
    let audit_repo = PgAuditLogRepository::new(db.pool.clone());

    let tenant = create_test_tenant();
    tenant_repo.create(&tenant).await.expect("Failed to create tenant");

    let audit_log = AuditLog::new(
        tenant.id,
        None,
        AuditAction::Created,
        ResourceType::Project,
    )
    .with_resource_id(uuid::Uuid::new_v4());

    audit_repo.create(&audit_log).await.expect("Failed to create audit log");

    let found = audit_repo
        .find_by_id(audit_log.id)
        .await
        .expect("Failed to find audit log")
        .expect("Audit log not found");

    assert_eq!(found.id, audit_log.id);
    assert_eq!(found.action, AuditAction::Created);
    assert_eq!(found.resource_type, ResourceType::Project);
}

#[tokio::test]
async fn test_audit_log_with_user() {
    let db = TestDb::new().await;
    let tenant_repo = PgTenantRepository::new(db.pool.clone());
    let user_repo = PgUserRepository::new(db.pool.clone());
    let audit_repo = PgAuditLogRepository::new(db.pool.clone());

    let tenant = create_test_tenant();
    tenant_repo.create(&tenant).await.expect("Failed to create tenant");

    let user = create_test_user(tenant.id);
    user_repo.create(&user).await.expect("Failed to create user");

    let audit_log = AuditLog::login(tenant.id, user.id);
    audit_repo.create(&audit_log).await.expect("Failed to create audit log");

    let found = audit_repo
        .find_by_id(audit_log.id)
        .await
        .expect("Failed to find audit log")
        .expect("Audit log not found");

    assert_eq!(found.user_id, Some(user.id));
    assert_eq!(found.action, AuditAction::Login);
}

#[tokio::test]
async fn test_audit_log_with_request_context() {
    let db = TestDb::new().await;
    let tenant_repo = PgTenantRepository::new(db.pool.clone());
    let audit_repo = PgAuditLogRepository::new(db.pool.clone());

    let tenant = create_test_tenant();
    tenant_repo.create(&tenant).await.expect("Failed to create tenant");

    let ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 100));
    let user_agent = "Mozilla/5.0 Test Browser".to_string();

    let audit_log = AuditLog::new(
        tenant.id,
        None,
        AuditAction::SettingsUpdated,
        ResourceType::Settings,
    )
    .with_request_context(ip, user_agent.clone());

    audit_repo.create(&audit_log).await.expect("Failed to create audit log");

    let found = audit_repo
        .find_by_id(audit_log.id)
        .await
        .expect("Failed to find audit log")
        .expect("Audit log not found");

    assert_eq!(found.user_agent, Some(user_agent));
    // Note: IP address handling may vary based on PostgreSQL INET type parsing
}

#[tokio::test]
async fn test_find_audit_logs_by_tenant() {
    let db = TestDb::new().await;
    let tenant_repo = PgTenantRepository::new(db.pool.clone());
    let audit_repo = PgAuditLogRepository::new(db.pool.clone());

    let tenant = create_test_tenant();
    tenant_repo.create(&tenant).await.expect("Failed to create tenant");

    // Create multiple audit logs
    for _ in 0..5 {
        let log = AuditLog::new(tenant.id, None, AuditAction::Created, ResourceType::Project);
        audit_repo.create(&log).await.expect("Failed to create audit log");
    }

    let logs = audit_repo
        .find_by_tenant(tenant.id, AuditLogFilter::default(), 10, 0)
        .await
        .expect("Failed to find audit logs");

    assert_eq!(logs.len(), 5);
}

#[tokio::test]
async fn test_find_audit_logs_with_action_filter() {
    let db = TestDb::new().await;
    let tenant_repo = PgTenantRepository::new(db.pool.clone());
    let user_repo = PgUserRepository::new(db.pool.clone());
    let audit_repo = PgAuditLogRepository::new(db.pool.clone());

    let tenant = create_test_tenant();
    tenant_repo.create(&tenant).await.expect("Failed to create tenant");

    let user = create_test_user(tenant.id);
    user_repo.create(&user).await.expect("Failed to create user");

    // Create different types of audit logs
    audit_repo
        .create(&AuditLog::login(tenant.id, user.id))
        .await
        .unwrap();
    audit_repo
        .create(&AuditLog::new(tenant.id, Some(user.id), AuditAction::Created, ResourceType::Project))
        .await
        .unwrap();
    audit_repo
        .create(&AuditLog::new(tenant.id, Some(user.id), AuditAction::Updated, ResourceType::Project))
        .await
        .unwrap();

    // Filter by login action
    let filter = AuditLogFilter {
        action: Some(vec![AuditAction::Login]),
        ..Default::default()
    };

    let logs = audit_repo
        .find_by_tenant(tenant.id, filter, 10, 0)
        .await
        .expect("Failed to find audit logs");

    assert_eq!(logs.len(), 1);
    assert_eq!(logs[0].action, AuditAction::Login);
}

#[tokio::test]
async fn test_find_audit_logs_with_user_filter() {
    let db = TestDb::new().await;
    let tenant_repo = PgTenantRepository::new(db.pool.clone());
    let user_repo = PgUserRepository::new(db.pool.clone());
    let audit_repo = PgAuditLogRepository::new(db.pool.clone());

    let tenant = create_test_tenant();
    tenant_repo.create(&tenant).await.expect("Failed to create tenant");

    let user1 = create_test_user(tenant.id);
    let user2 = create_test_user(tenant.id);
    user_repo.create(&user1).await.unwrap();
    user_repo.create(&user2).await.unwrap();

    // Create logs for different users
    audit_repo
        .create(&AuditLog::login(tenant.id, user1.id))
        .await
        .unwrap();
    audit_repo
        .create(&AuditLog::login(tenant.id, user1.id))
        .await
        .unwrap();
    audit_repo
        .create(&AuditLog::login(tenant.id, user2.id))
        .await
        .unwrap();

    // Filter by user1
    let filter = AuditLogFilter {
        user_id: Some(user1.id),
        ..Default::default()
    };

    let logs = audit_repo
        .find_by_tenant(tenant.id, filter, 10, 0)
        .await
        .expect("Failed to find audit logs");

    assert_eq!(logs.len(), 2);
    for log in logs {
        assert_eq!(log.user_id, Some(user1.id));
    }
}

#[tokio::test]
async fn test_find_audit_logs_with_resource_type_filter() {
    let db = TestDb::new().await;
    let tenant_repo = PgTenantRepository::new(db.pool.clone());
    let audit_repo = PgAuditLogRepository::new(db.pool.clone());

    let tenant = create_test_tenant();
    tenant_repo.create(&tenant).await.expect("Failed to create tenant");

    // Create logs for different resource types
    audit_repo
        .create(&AuditLog::new(tenant.id, None, AuditAction::Created, ResourceType::Project))
        .await
        .unwrap();
    audit_repo
        .create(&AuditLog::new(tenant.id, None, AuditAction::Created, ResourceType::Policy))
        .await
        .unwrap();
    audit_repo
        .create(&AuditLog::new(tenant.id, None, AuditAction::Created, ResourceType::Project))
        .await
        .unwrap();

    // Filter by Project resource type
    let filter = AuditLogFilter {
        resource_type: Some(ResourceType::Project),
        ..Default::default()
    };

    let logs = audit_repo
        .find_by_tenant(tenant.id, filter, 10, 0)
        .await
        .expect("Failed to find audit logs");

    assert_eq!(logs.len(), 2);
}

#[tokio::test]
async fn test_audit_log_pagination() {
    let db = TestDb::new().await;
    let tenant_repo = PgTenantRepository::new(db.pool.clone());
    let audit_repo = PgAuditLogRepository::new(db.pool.clone());

    let tenant = create_test_tenant();
    tenant_repo.create(&tenant).await.expect("Failed to create tenant");

    // Create 10 audit logs
    for _ in 0..10 {
        let log = AuditLog::new(tenant.id, None, AuditAction::Created, ResourceType::Project);
        audit_repo.create(&log).await.unwrap();
    }

    // Get first page (5 items)
    let page1 = audit_repo
        .find_by_tenant(tenant.id, AuditLogFilter::default(), 5, 0)
        .await
        .expect("Failed to find audit logs");

    assert_eq!(page1.len(), 5);

    // Get second page (5 items)
    let page2 = audit_repo
        .find_by_tenant(tenant.id, AuditLogFilter::default(), 5, 5)
        .await
        .expect("Failed to find audit logs");

    assert_eq!(page2.len(), 5);

    // Ensure no overlap
    let page1_ids: Vec<_> = page1.iter().map(|l| l.id).collect();
    let page2_ids: Vec<_> = page2.iter().map(|l| l.id).collect();
    for id in &page2_ids {
        assert!(!page1_ids.contains(id));
    }
}

#[tokio::test]
async fn test_count_by_tenant() {
    let db = TestDb::new().await;
    let tenant_repo = PgTenantRepository::new(db.pool.clone());
    let audit_repo = PgAuditLogRepository::new(db.pool.clone());

    let tenant = create_test_tenant();
    tenant_repo.create(&tenant).await.expect("Failed to create tenant");

    // Initially zero
    let count = audit_repo
        .count_by_tenant(tenant.id)
        .await
        .expect("Failed to count");
    assert_eq!(count, 0);

    // Create some logs
    for _ in 0..7 {
        let log = AuditLog::new(tenant.id, None, AuditAction::Created, ResourceType::Project);
        audit_repo.create(&log).await.unwrap();
    }

    let count = audit_repo
        .count_by_tenant(tenant.id)
        .await
        .expect("Failed to count");
    assert_eq!(count, 7);
}

#[tokio::test]
async fn test_cleanup_old_logs() {
    let db = TestDb::new().await;
    let tenant_repo = PgTenantRepository::new(db.pool.clone());
    let audit_repo = PgAuditLogRepository::new(db.pool.clone());

    let tenant = create_test_tenant();
    tenant_repo.create(&tenant).await.expect("Failed to create tenant");

    // Create a log
    let log = AuditLog::new(tenant.id, None, AuditAction::Created, ResourceType::Project);
    audit_repo.create(&log).await.unwrap();

    // Cleanup logs older than 0 days
    // Note: This won't delete the log since it was just created
    let deleted = audit_repo
        .cleanup_old_logs(0)
        .await
        .expect("Failed to cleanup");

    assert_eq!(deleted, 0);
}

#[tokio::test]
async fn test_audit_log_with_details() {
    let db = TestDb::new().await;
    let tenant_repo = PgTenantRepository::new(db.pool.clone());
    let user_repo = PgUserRepository::new(db.pool.clone());
    let audit_repo = PgAuditLogRepository::new(db.pool.clone());

    let tenant = create_test_tenant();
    tenant_repo.create(&tenant).await.expect("Failed to create tenant");

    let user = create_test_user(tenant.id);
    user_repo.create(&user).await.expect("Failed to create user");

    let resource_id = uuid::Uuid::new_v4();
    let changes = serde_json::json!({
        "field": "name",
        "old_value": "Old Name",
        "new_value": "New Name"
    });

    let audit_log = AuditLog::updated(tenant.id, user.id, ResourceType::Project, resource_id, changes.clone());
    audit_repo.create(&audit_log).await.expect("Failed to create audit log");

    let found = audit_repo
        .find_by_id(audit_log.id)
        .await
        .expect("Failed to find audit log")
        .expect("Audit log not found");

    assert_eq!(found.resource_id, Some(resource_id));
    assert_eq!(found.details, changes);
}
