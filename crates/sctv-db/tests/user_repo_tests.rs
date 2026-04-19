//! Integration tests for the user repository.

mod common;

use common::{create_test_tenant, create_test_user, TestDb};
use sctv_core::traits::{TenantRepository, UserRepository};
use sctv_core::UserRole;
use sctv_db::{PgTenantRepository, PgUserRepository};

#[tokio::test]
async fn test_create_and_find_user_by_id() {
    let db = TestDb::new().await;
    let tenant_repo = PgTenantRepository::new(db.pool.clone());
    let user_repo = PgUserRepository::new(db.pool.clone());

    // Create a tenant first
    let tenant = create_test_tenant();
    tenant_repo
        .create(&tenant)
        .await
        .expect("Failed to create tenant");

    // Create a user
    let user = create_test_user(tenant.id);
    user_repo
        .create(&user)
        .await
        .expect("Failed to create user");

    // Find by ID
    let found = user_repo
        .find_by_id(user.id)
        .await
        .expect("Failed to find user")
        .expect("User not found");

    assert_eq!(found.id, user.id);
    assert_eq!(found.email, user.email);
    assert_eq!(found.tenant_id, tenant.id);
}

#[tokio::test]
async fn test_find_user_by_email() {
    let db = TestDb::new().await;
    let tenant_repo = PgTenantRepository::new(db.pool.clone());
    let user_repo = PgUserRepository::new(db.pool.clone());

    let tenant = create_test_tenant();
    tenant_repo
        .create(&tenant)
        .await
        .expect("Failed to create tenant");

    let user = create_test_user(tenant.id);
    user_repo
        .create(&user)
        .await
        .expect("Failed to create user");

    // Find by email
    let found = user_repo
        .find_by_email(tenant.id, &user.email)
        .await
        .expect("Failed to find user")
        .expect("User not found");

    assert_eq!(found.id, user.id);
    assert_eq!(found.email, user.email);
}

#[tokio::test]
async fn test_find_user_by_email_not_found() {
    let db = TestDb::new().await;
    let tenant_repo = PgTenantRepository::new(db.pool.clone());
    let user_repo = PgUserRepository::new(db.pool.clone());

    let tenant = create_test_tenant();
    tenant_repo
        .create(&tenant)
        .await
        .expect("Failed to create tenant");

    let result = user_repo
        .find_by_email(tenant.id, "nonexistent@example.com")
        .await
        .expect("Failed to query");

    assert!(result.is_none());
}

#[tokio::test]
async fn test_find_users_by_tenant() {
    let db = TestDb::new().await;
    let tenant_repo = PgTenantRepository::new(db.pool.clone());
    let user_repo = PgUserRepository::new(db.pool.clone());

    let tenant = create_test_tenant();
    tenant_repo
        .create(&tenant)
        .await
        .expect("Failed to create tenant");

    // Create multiple users
    let user1 = create_test_user(tenant.id);
    let user2 = create_test_user(tenant.id);
    user_repo
        .create(&user1)
        .await
        .expect("Failed to create user1");
    user_repo
        .create(&user2)
        .await
        .expect("Failed to create user2");

    // Find all users for tenant
    let found_users = user_repo
        .find_by_tenant(tenant.id)
        .await
        .expect("Failed to find users");

    assert_eq!(found_users.len(), 2);
}

#[tokio::test]
async fn test_update_user() {
    let db = TestDb::new().await;
    let tenant_repo = PgTenantRepository::new(db.pool.clone());
    let user_repo = PgUserRepository::new(db.pool.clone());

    let tenant = create_test_tenant();
    tenant_repo
        .create(&tenant)
        .await
        .expect("Failed to create tenant");

    let mut user = create_test_user(tenant.id);
    user_repo
        .create(&user)
        .await
        .expect("Failed to create user");

    // Update user
    user.name = Some("Updated Name".to_string());
    user.role = UserRole::Admin;
    user_repo
        .update(&user)
        .await
        .expect("Failed to update user");

    // Verify update
    let found = user_repo
        .find_by_id(user.id)
        .await
        .expect("Failed to find user")
        .expect("User not found");

    assert_eq!(found.name, Some("Updated Name".to_string()));
    assert_eq!(found.role, UserRole::Admin);
}

#[tokio::test]
async fn test_delete_user() {
    let db = TestDb::new().await;
    let tenant_repo = PgTenantRepository::new(db.pool.clone());
    let user_repo = PgUserRepository::new(db.pool.clone());

    let tenant = create_test_tenant();
    tenant_repo
        .create(&tenant)
        .await
        .expect("Failed to create tenant");

    let user = create_test_user(tenant.id);
    user_repo
        .create(&user)
        .await
        .expect("Failed to create user");

    // Delete user
    user_repo
        .delete(user.id)
        .await
        .expect("Failed to delete user");

    // Verify deletion
    let result = user_repo
        .find_by_id(user.id)
        .await
        .expect("Failed to query");

    assert!(result.is_none());
}

#[tokio::test]
async fn test_update_last_login() {
    let db = TestDb::new().await;
    let tenant_repo = PgTenantRepository::new(db.pool.clone());
    let user_repo = PgUserRepository::new(db.pool.clone());

    let tenant = create_test_tenant();
    tenant_repo
        .create(&tenant)
        .await
        .expect("Failed to create tenant");

    let user = create_test_user(tenant.id);
    user_repo
        .create(&user)
        .await
        .expect("Failed to create user");

    // Initially no last login
    let found = user_repo
        .find_by_id(user.id)
        .await
        .expect("Failed to find user")
        .expect("User not found");
    assert!(found.last_login_at.is_none());

    // Update last login
    user_repo
        .update_last_login(user.id)
        .await
        .expect("Failed to update last login");

    // Verify last login is set
    let found = user_repo
        .find_by_id(user.id)
        .await
        .expect("Failed to find user")
        .expect("User not found");
    assert!(found.last_login_at.is_some());
}

#[tokio::test]
async fn test_count_by_tenant() {
    let db = TestDb::new().await;
    let tenant_repo = PgTenantRepository::new(db.pool.clone());
    let user_repo = PgUserRepository::new(db.pool.clone());

    let tenant = create_test_tenant();
    tenant_repo
        .create(&tenant)
        .await
        .expect("Failed to create tenant");

    // Initially zero users
    let count = user_repo
        .count_by_tenant(tenant.id)
        .await
        .expect("Failed to count");
    assert_eq!(count, 0);

    // Create users
    user_repo
        .create(&create_test_user(tenant.id))
        .await
        .unwrap();
    user_repo
        .create(&create_test_user(tenant.id))
        .await
        .unwrap();
    user_repo
        .create(&create_test_user(tenant.id))
        .await
        .unwrap();

    let count = user_repo
        .count_by_tenant(tenant.id)
        .await
        .expect("Failed to count");
    assert_eq!(count, 3);
}

#[tokio::test]
async fn test_find_user_by_api_key() {
    let db = TestDb::new().await;
    let tenant_repo = PgTenantRepository::new(db.pool.clone());
    let user_repo = PgUserRepository::new(db.pool.clone());

    let tenant = create_test_tenant();
    tenant_repo
        .create(&tenant)
        .await
        .expect("Failed to create tenant");

    let mut user = create_test_user(tenant.id);
    user.api_key_hash = Some("hashed_api_key_12345".to_string());
    user_repo
        .create(&user)
        .await
        .expect("Failed to create user");

    // Find by API key
    let found = user_repo
        .find_by_api_key("hashed_api_key_12345")
        .await
        .expect("Failed to find user")
        .expect("User not found");

    assert_eq!(found.id, user.id);

    // Not found with wrong key
    let not_found = user_repo
        .find_by_api_key("wrong_key")
        .await
        .expect("Failed to query");
    assert!(not_found.is_none());
}

#[tokio::test]
async fn test_duplicate_email_same_tenant_fails() {
    let db = TestDb::new().await;
    let tenant_repo = PgTenantRepository::new(db.pool.clone());
    let user_repo = PgUserRepository::new(db.pool.clone());

    let tenant = create_test_tenant();
    tenant_repo
        .create(&tenant)
        .await
        .expect("Failed to create tenant");

    let user1 = create_test_user(tenant.id);
    user_repo
        .create(&user1)
        .await
        .expect("Failed to create user1");

    // Try to create another user with same email
    let mut user2 = create_test_user(tenant.id);
    user2.email = user1.email.clone();

    let result = user_repo.create(&user2).await;
    assert!(result.is_err());
}
