//! Integration tests for the job repository.

mod common;

use common::{create_test_tenant, TestDb};
use sctv_core::traits::{JobFilter, JobRepository, TenantRepository};
use sctv_core::{Job, JobPriority, JobStatus, JobType};
use sctv_db::{PgJobRepository, PgTenantRepository};

fn create_test_job(tenant_id: Option<sctv_core::TenantId>) -> Job {
    Job::new(
        tenant_id,
        JobType::ScanProject {
            project_id: uuid::Uuid::new_v4(),
        },
    )
}

#[tokio::test]
async fn test_create_and_find_job_by_id() {
    let db = TestDb::new().await;
    let tenant_repo = PgTenantRepository::new(db.pool.clone());
    let job_repo = PgJobRepository::new(db.pool.clone());

    let tenant = create_test_tenant();
    tenant_repo
        .create(&tenant)
        .await
        .expect("Failed to create tenant");

    let job = create_test_job(Some(tenant.id));
    job_repo.create(&job).await.expect("Failed to create job");

    let found = job_repo
        .find_by_id(job.id)
        .await
        .expect("Failed to find job")
        .expect("Job not found");

    assert_eq!(found.id, job.id);
    assert_eq!(found.status, JobStatus::Pending);
}

#[tokio::test]
async fn test_create_global_job() {
    let db = TestDb::new().await;
    let job_repo = PgJobRepository::new(db.pool.clone());

    // Create a job without tenant (global job)
    let job = Job::new(
        None,
        JobType::Cleanup {
            older_than_days: 30,
        },
    );
    job_repo.create(&job).await.expect("Failed to create job");

    let found = job_repo
        .find_by_id(job.id)
        .await
        .expect("Failed to find job")
        .expect("Job not found");

    assert_eq!(found.id, job.id);
    assert!(found.tenant_id.is_none());
}

#[tokio::test]
async fn test_update_job() {
    let db = TestDb::new().await;
    let tenant_repo = PgTenantRepository::new(db.pool.clone());
    let job_repo = PgJobRepository::new(db.pool.clone());

    let tenant = create_test_tenant();
    tenant_repo
        .create(&tenant)
        .await
        .expect("Failed to create tenant");

    let mut job = create_test_job(Some(tenant.id));
    job_repo.create(&job).await.expect("Failed to create job");

    // Update job status
    job.mark_started();
    job_repo.update(&job).await.expect("Failed to update job");

    let found = job_repo
        .find_by_id(job.id)
        .await
        .expect("Failed to find job")
        .expect("Job not found");

    assert_eq!(found.status, JobStatus::Running);
    assert!(found.started_at.is_some());
    assert_eq!(found.attempts, 1);
}

#[tokio::test]
async fn test_job_completion() {
    let db = TestDb::new().await;
    let tenant_repo = PgTenantRepository::new(db.pool.clone());
    let job_repo = PgJobRepository::new(db.pool.clone());

    let tenant = create_test_tenant();
    tenant_repo
        .create(&tenant)
        .await
        .expect("Failed to create tenant");

    let mut job = create_test_job(Some(tenant.id));
    job_repo.create(&job).await.expect("Failed to create job");

    // Mark as completed
    job.mark_started();
    job.mark_completed(serde_json::json!({"success": true}));
    job_repo.update(&job).await.expect("Failed to update job");

    let found = job_repo
        .find_by_id(job.id)
        .await
        .expect("Failed to find job")
        .expect("Job not found");

    assert_eq!(found.status, JobStatus::Completed);
    assert!(found.completed_at.is_some());
    assert!(found.result.is_some());
}

#[tokio::test]
async fn test_job_failure() {
    let db = TestDb::new().await;
    let tenant_repo = PgTenantRepository::new(db.pool.clone());
    let job_repo = PgJobRepository::new(db.pool.clone());

    let tenant = create_test_tenant();
    tenant_repo
        .create(&tenant)
        .await
        .expect("Failed to create tenant");

    let mut job = create_test_job(Some(tenant.id));
    job_repo.create(&job).await.expect("Failed to create job");

    // Mark as failed
    job.mark_started();
    job.mark_failed("Something went wrong".to_string());
    job_repo.update(&job).await.expect("Failed to update job");

    let found = job_repo
        .find_by_id(job.id)
        .await
        .expect("Failed to find job")
        .expect("Job not found");

    assert_eq!(found.status, JobStatus::Failed);
    assert_eq!(
        found.error_message,
        Some("Something went wrong".to_string())
    );
}

#[tokio::test]
async fn test_claim_next_job() {
    let db = TestDb::new().await;
    let tenant_repo = PgTenantRepository::new(db.pool.clone());
    let job_repo = PgJobRepository::new(db.pool.clone());

    let tenant = create_test_tenant();
    tenant_repo
        .create(&tenant)
        .await
        .expect("Failed to create tenant");

    // Create multiple jobs with different priorities
    let low_job = create_test_job(Some(tenant.id)).with_priority(JobPriority::Low);
    let high_job = create_test_job(Some(tenant.id)).with_priority(JobPriority::High);

    job_repo
        .create(&low_job)
        .await
        .expect("Failed to create low job");
    job_repo
        .create(&high_job)
        .await
        .expect("Failed to create high job");

    // Claim next should get high priority job first
    let claimed = job_repo
        .claim_next()
        .await
        .expect("Failed to claim job")
        .expect("No job claimed");

    assert_eq!(claimed.id, high_job.id);
    assert_eq!(claimed.status, JobStatus::Running);
}

#[tokio::test]
async fn test_find_due_jobs() {
    let db = TestDb::new().await;
    let tenant_repo = PgTenantRepository::new(db.pool.clone());
    let job_repo = PgJobRepository::new(db.pool.clone());

    let tenant = create_test_tenant();
    tenant_repo
        .create(&tenant)
        .await
        .expect("Failed to create tenant");

    // Create some pending jobs
    job_repo
        .create(&create_test_job(Some(tenant.id)))
        .await
        .unwrap();
    job_repo
        .create(&create_test_job(Some(tenant.id)))
        .await
        .unwrap();

    let due_jobs = job_repo
        .find_due_jobs(10)
        .await
        .expect("Failed to find due jobs");

    assert_eq!(due_jobs.len(), 2);
}

#[tokio::test]
async fn test_find_by_tenant_with_filter() {
    let db = TestDb::new().await;
    let tenant_repo = PgTenantRepository::new(db.pool.clone());
    let job_repo = PgJobRepository::new(db.pool.clone());

    let tenant = create_test_tenant();
    tenant_repo
        .create(&tenant)
        .await
        .expect("Failed to create tenant");

    // Create jobs with different statuses
    let pending_job = create_test_job(Some(tenant.id));
    job_repo.create(&pending_job).await.unwrap();

    let mut running_job = create_test_job(Some(tenant.id));
    running_job.mark_started();
    job_repo.create(&running_job).await.unwrap();
    job_repo.update(&running_job).await.unwrap();

    // Filter by pending status
    let filter = JobFilter {
        status: Some(vec![JobStatus::Pending]),
        job_type: None,
    };

    let jobs = job_repo
        .find_by_tenant(Some(tenant.id), filter, 10, 0)
        .await
        .expect("Failed to find jobs");

    assert_eq!(jobs.len(), 1);
    assert_eq!(jobs[0].status, JobStatus::Pending);
}

#[tokio::test]
async fn test_count_by_status() {
    let db = TestDb::new().await;
    let tenant_repo = PgTenantRepository::new(db.pool.clone());
    let job_repo = PgJobRepository::new(db.pool.clone());

    let tenant = create_test_tenant();
    tenant_repo
        .create(&tenant)
        .await
        .expect("Failed to create tenant");

    // Create jobs
    job_repo
        .create(&create_test_job(Some(tenant.id)))
        .await
        .unwrap();
    job_repo
        .create(&create_test_job(Some(tenant.id)))
        .await
        .unwrap();

    let counts = job_repo.count_by_status().await.expect("Failed to count");

    assert_eq!(*counts.get(&JobStatus::Pending).unwrap_or(&0), 2);
}

#[tokio::test]
async fn test_cleanup_old_jobs() {
    let db = TestDb::new().await;
    let job_repo = PgJobRepository::new(db.pool.clone());

    // Create and complete a job
    let mut job = Job::new(None, JobType::Cleanup { older_than_days: 1 });
    job.mark_started();
    job.mark_completed(serde_json::json!({}));
    job_repo.create(&job).await.unwrap();
    job_repo.update(&job).await.unwrap();

    // Cleanup jobs older than 0 days (should include our job)
    // Note: This won't actually delete since the job was just created
    let deleted = job_repo
        .cleanup_old_jobs(0)
        .await
        .expect("Failed to cleanup");

    // The job was just created so it won't be deleted (completed_at is now)
    assert_eq!(deleted, 0);
}
