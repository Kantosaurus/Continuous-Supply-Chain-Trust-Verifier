//! Integration tests for the SBOM repository.

mod common;

use common::{create_test_project, create_test_tenant, TestDb};
use sctv_core::traits::{ProjectRepository, SbomRepository, TenantRepository};
use sctv_core::{Sbom, SbomFormat};
use sctv_db::{PgProjectRepository, PgSbomRepository, PgTenantRepository};

fn create_test_sbom(
    project_id: sctv_core::ProjectId,
    tenant_id: sctv_core::TenantId,
    format: SbomFormat,
) -> Sbom {
    Sbom::new(
        project_id,
        tenant_id,
        format,
        Sbom::default_version(format).to_string(),
        serde_json::json!({
            "bomFormat": "CycloneDX",
            "specVersion": "1.5",
            "components": []
        }),
    )
}

#[tokio::test]
async fn test_create_and_find_sbom_by_id() {
    let db = TestDb::new().await;
    let tenant_repo = PgTenantRepository::new(db.pool.clone());
    let project_repo = PgProjectRepository::new(db.pool.clone());
    let sbom_repo = PgSbomRepository::new(db.pool.clone());

    let tenant = create_test_tenant();
    tenant_repo
        .create(&tenant)
        .await
        .expect("Failed to create tenant");

    let project = create_test_project(tenant.id);
    project_repo
        .create(&project)
        .await
        .expect("Failed to create project");

    let sbom = create_test_sbom(project.id, tenant.id, SbomFormat::CycloneDx);
    sbom_repo
        .create(&sbom)
        .await
        .expect("Failed to create SBOM");

    let found = sbom_repo
        .find_by_id(sbom.id)
        .await
        .expect("Failed to find SBOM")
        .expect("SBOM not found");

    assert_eq!(found.id, sbom.id);
    assert_eq!(found.format, SbomFormat::CycloneDx);
    assert_eq!(found.format_version, "1.5");
}

#[tokio::test]
async fn test_find_sboms_by_project() {
    let db = TestDb::new().await;
    let tenant_repo = PgTenantRepository::new(db.pool.clone());
    let project_repo = PgProjectRepository::new(db.pool.clone());
    let sbom_repo = PgSbomRepository::new(db.pool.clone());

    let tenant = create_test_tenant();
    tenant_repo
        .create(&tenant)
        .await
        .expect("Failed to create tenant");

    let project = create_test_project(tenant.id);
    project_repo
        .create(&project)
        .await
        .expect("Failed to create project");

    // Create multiple SBOMs
    let sbom1 = create_test_sbom(project.id, tenant.id, SbomFormat::CycloneDx);
    let sbom2 = create_test_sbom(project.id, tenant.id, SbomFormat::Spdx);
    sbom_repo
        .create(&sbom1)
        .await
        .expect("Failed to create SBOM 1");

    // Add small delay to ensure different timestamps
    tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    sbom_repo
        .create(&sbom2)
        .await
        .expect("Failed to create SBOM 2");

    let found_sboms = sbom_repo
        .find_by_project(project.id)
        .await
        .expect("Failed to find SBOMs");

    assert_eq!(found_sboms.len(), 2);
}

#[tokio::test]
async fn test_find_latest_sbom() {
    let db = TestDb::new().await;
    let tenant_repo = PgTenantRepository::new(db.pool.clone());
    let project_repo = PgProjectRepository::new(db.pool.clone());
    let sbom_repo = PgSbomRepository::new(db.pool.clone());

    let tenant = create_test_tenant();
    tenant_repo
        .create(&tenant)
        .await
        .expect("Failed to create tenant");

    let project = create_test_project(tenant.id);
    project_repo
        .create(&project)
        .await
        .expect("Failed to create project");

    // Create SBOMs with some delay between them
    let sbom1 = create_test_sbom(project.id, tenant.id, SbomFormat::CycloneDx);
    sbom_repo
        .create(&sbom1)
        .await
        .expect("Failed to create SBOM 1");

    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    let sbom2 = create_test_sbom(project.id, tenant.id, SbomFormat::CycloneDx);
    sbom_repo
        .create(&sbom2)
        .await
        .expect("Failed to create SBOM 2");

    // Find latest should return sbom2
    let latest = sbom_repo
        .find_latest(project.id)
        .await
        .expect("Failed to find latest SBOM")
        .expect("No SBOM found");

    assert_eq!(latest.id, sbom2.id);
}

#[tokio::test]
async fn test_find_latest_sbom_by_format() {
    let db = TestDb::new().await;
    let tenant_repo = PgTenantRepository::new(db.pool.clone());
    let project_repo = PgProjectRepository::new(db.pool.clone());
    let sbom_repo = PgSbomRepository::new(db.pool.clone());

    let tenant = create_test_tenant();
    tenant_repo
        .create(&tenant)
        .await
        .expect("Failed to create tenant");

    let project = create_test_project(tenant.id);
    project_repo
        .create(&project)
        .await
        .expect("Failed to create project");

    // Create CycloneDX SBOM
    let cyclonedx = create_test_sbom(project.id, tenant.id, SbomFormat::CycloneDx);
    sbom_repo
        .create(&cyclonedx)
        .await
        .expect("Failed to create CycloneDX SBOM");

    // Create SPDX SBOM
    let spdx = create_test_sbom(project.id, tenant.id, SbomFormat::Spdx);
    sbom_repo
        .create(&spdx)
        .await
        .expect("Failed to create SPDX SBOM");

    // Find latest CycloneDX
    let found_cyclonedx = sbom_repo
        .find_latest_by_format(project.id, SbomFormat::CycloneDx)
        .await
        .expect("Failed to find SBOM")
        .expect("CycloneDX SBOM not found");

    assert_eq!(found_cyclonedx.format, SbomFormat::CycloneDx);

    // Find latest SPDX
    let found_spdx = sbom_repo
        .find_latest_by_format(project.id, SbomFormat::Spdx)
        .await
        .expect("Failed to find SBOM")
        .expect("SPDX SBOM not found");

    assert_eq!(found_spdx.format, SbomFormat::Spdx);
}

#[tokio::test]
async fn test_delete_sbom() {
    let db = TestDb::new().await;
    let tenant_repo = PgTenantRepository::new(db.pool.clone());
    let project_repo = PgProjectRepository::new(db.pool.clone());
    let sbom_repo = PgSbomRepository::new(db.pool.clone());

    let tenant = create_test_tenant();
    tenant_repo
        .create(&tenant)
        .await
        .expect("Failed to create tenant");

    let project = create_test_project(tenant.id);
    project_repo
        .create(&project)
        .await
        .expect("Failed to create project");

    let sbom = create_test_sbom(project.id, tenant.id, SbomFormat::CycloneDx);
    sbom_repo
        .create(&sbom)
        .await
        .expect("Failed to create SBOM");

    // Delete SBOM
    sbom_repo
        .delete(sbom.id)
        .await
        .expect("Failed to delete SBOM");

    // Verify deletion
    let result = sbom_repo
        .find_by_id(sbom.id)
        .await
        .expect("Failed to query");

    assert!(result.is_none());
}

#[tokio::test]
async fn test_cleanup_old_sboms() {
    let db = TestDb::new().await;
    let tenant_repo = PgTenantRepository::new(db.pool.clone());
    let project_repo = PgProjectRepository::new(db.pool.clone());
    let sbom_repo = PgSbomRepository::new(db.pool.clone());

    let tenant = create_test_tenant();
    tenant_repo
        .create(&tenant)
        .await
        .expect("Failed to create tenant");

    let project = create_test_project(tenant.id);
    project_repo
        .create(&project)
        .await
        .expect("Failed to create project");

    // Create 5 SBOMs
    for i in 0..5 {
        let sbom = create_test_sbom(project.id, tenant.id, SbomFormat::CycloneDx);
        sbom_repo
            .create(&sbom)
            .await
            .unwrap_or_else(|_| panic!("Failed to create SBOM {i}"));
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }

    // Keep only 2 most recent
    let deleted = sbom_repo
        .cleanup_old_sboms(project.id, 2)
        .await
        .expect("Failed to cleanup SBOMs");

    assert_eq!(deleted, 3);

    // Verify only 2 remain
    let remaining = sbom_repo
        .find_by_project(project.id)
        .await
        .expect("Failed to find SBOMs");

    assert_eq!(remaining.len(), 2);
}

#[tokio::test]
async fn test_sbom_with_scan_id() {
    let db = TestDb::new().await;
    let tenant_repo = PgTenantRepository::new(db.pool.clone());
    let project_repo = PgProjectRepository::new(db.pool.clone());
    let sbom_repo = PgSbomRepository::new(db.pool.clone());

    let tenant = create_test_tenant();
    tenant_repo
        .create(&tenant)
        .await
        .expect("Failed to create tenant");

    let project = create_test_project(tenant.id);
    project_repo
        .create(&project)
        .await
        .expect("Failed to create project");

    let scan_id = uuid::Uuid::new_v4();
    let sbom = Sbom::from_scan(
        project.id,
        tenant.id,
        SbomFormat::CycloneDx,
        "1.5".to_string(),
        serde_json::json!({"components": []}),
        scan_id,
    );

    sbom_repo
        .create(&sbom)
        .await
        .expect("Failed to create SBOM");

    let found = sbom_repo
        .find_by_id(sbom.id)
        .await
        .expect("Failed to find SBOM")
        .expect("SBOM not found");

    assert_eq!(found.scan_id, Some(scan_id));
}

#[tokio::test]
async fn test_find_latest_returns_none_for_empty_project() {
    let db = TestDb::new().await;
    let tenant_repo = PgTenantRepository::new(db.pool.clone());
    let project_repo = PgProjectRepository::new(db.pool.clone());
    let sbom_repo = PgSbomRepository::new(db.pool.clone());

    let tenant = create_test_tenant();
    tenant_repo
        .create(&tenant)
        .await
        .expect("Failed to create tenant");

    let project = create_test_project(tenant.id);
    project_repo
        .create(&project)
        .await
        .expect("Failed to create project");

    // No SBOMs created
    let result = sbom_repo
        .find_latest(project.id)
        .await
        .expect("Failed to query");

    assert!(result.is_none());
}
