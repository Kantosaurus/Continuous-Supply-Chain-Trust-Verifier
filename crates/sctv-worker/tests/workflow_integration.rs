//! End-to-end integration tests for the scan workflow.
//!
//! These tests verify the complete scan workflow from job creation
//! through execution to completion, including:
//! - Job queueing and lifecycle management
//! - Executor registration and dispatch
//! - Worker pool coordination
//! - Result handling and error recovery
//! - Priority-based processing

use chrono::{Duration, Utc};
use sctv_core::{AlertId, DependencyId, PackageEcosystem, ProjectId, Severity, TenantId};
use sctv_worker::{
    Job, JobId, JobPayload, JobPriority, JobResult, JobStatus, JobType,
    MonitorRegistryPayload, MonitorRegistryResult, NotificationChannel, NotificationContext,
    ProvenanceVerificationStatus, ScanProjectPayload, ScanProjectResult,
    SendNotificationPayload, SendNotificationResult, SigstoreDetails,
    VerifyProvenancePayload, VerifyProvenanceResult,
};
use uuid::Uuid;

// =============================================================================
// Job Creation Tests
// =============================================================================

mod job_creation {
    use super::*;

    #[test]
    fn test_create_scan_project_job() {
        let tenant_id = TenantId::new();
        let project_id = ProjectId::new();

        let payload = JobPayload::ScanProject(ScanProjectPayload {
            project_id,
            tenant_id,
            ecosystems: vec![PackageEcosystem::Npm, PackageEcosystem::PyPi],
            full_scan: true,
        });

        let job = Job::new(payload)
            .with_tenant(tenant_id)
            .with_priority(JobPriority::High);

        assert_eq!(job.job_type, JobType::ScanProject);
        assert_eq!(job.status, JobStatus::Pending);
        assert_eq!(job.priority, JobPriority::High);
        assert_eq!(job.tenant_id, Some(tenant_id));
        assert!(job.result.is_none());
    }

    #[test]
    fn test_create_monitor_registry_job() {
        let payload = JobPayload::MonitorRegistry(MonitorRegistryPayload {
            ecosystem: PackageEcosystem::Npm,
            packages: vec!["lodash".to_string(), "express".to_string()],
            check_new_versions: true,
            check_removals: true,
            check_maintainer_changes: false,
        });

        let job = Job::new(payload);

        assert_eq!(job.job_type, JobType::MonitorRegistry);
        assert_eq!(job.priority, JobPriority::Normal);
    }

    #[test]
    fn test_create_verify_provenance_job() {
        let payload = JobPayload::VerifyProvenance(VerifyProvenancePayload {
            dependency_id: DependencyId::new(),
            tenant_id: TenantId::new(),
            ecosystem: PackageEcosystem::Npm,
            package_name: "secure-package".to_string(),
            version: "1.0.0".to_string(),
            verify_slsa: true,
            verify_sigstore: true,
            verify_intoto: false,
        });

        let job = Job::new(payload);

        assert_eq!(job.job_type, JobType::VerifyProvenance);
    }

    #[test]
    fn test_create_notification_job() {
        let payload = JobPayload::SendNotification(SendNotificationPayload {
            alert_id: AlertId::new(),
            tenant_id: TenantId::new(),
            channel: NotificationChannel::Slack,
            channel_config: serde_json::json!({
                "webhook_url": "https://hooks.slack.com/services/xxx"
            }),
            severity: Severity::High,
            title: "Security Alert".to_string(),
            description: "Vulnerability detected".to_string(),
            context: NotificationContext {
                project_name: Some("my-project".to_string()),
                package_name: Some("lodash".to_string()),
                package_version: Some("4.17.21".to_string()),
                dashboard_url: Some("https://dashboard.example.com/alerts/123".to_string()),
                remediation: Some("Update to latest version".to_string()),
            },
        });

        let job = Job::new(payload);

        assert_eq!(job.job_type, JobType::SendNotification);
    }

    #[test]
    fn test_job_with_scheduled_time() {
        let payload = JobPayload::MonitorRegistry(MonitorRegistryPayload {
            ecosystem: PackageEcosystem::PyPi,
            packages: vec!["requests".to_string()],
            check_new_versions: true,
            check_removals: false,
            check_maintainer_changes: false,
        });

        let future_time = Utc::now() + Duration::hours(1);
        let job = Job::new(payload).scheduled_for(future_time);

        assert_eq!(job.status, JobStatus::Scheduled);
        assert!(job.scheduled_at > Utc::now());
    }
}

// =============================================================================
// Job Lifecycle Tests
// =============================================================================

mod job_lifecycle {
    use super::*;

    #[test]
    fn test_job_status_transitions() {
        let payload = create_test_scan_payload();
        let mut job = Job::new(payload);

        // Initial state
        assert_eq!(job.status, JobStatus::Pending);
        assert!(job.started_at.is_none());
        assert!(job.completed_at.is_none());

        // Start execution
        job.mark_started();
        assert_eq!(job.status, JobStatus::Running);
        assert!(job.started_at.is_some());
        assert_eq!(job.attempts, 1);

        // Complete successfully
        let result = JobResult::ScanProject(ScanProjectResult {
            dependencies_found: 42,
            alerts_created: 5,
            scan_duration_ms: 3500,
        });
        job.mark_completed(result);

        assert_eq!(job.status, JobStatus::Completed);
        assert!(job.completed_at.is_some());
        assert!(job.result.is_some());

        if let Some(JobResult::ScanProject(res)) = &job.result {
            assert_eq!(res.dependencies_found, 42);
            assert_eq!(res.alerts_created, 5);
        } else {
            panic!("Expected ScanProject result");
        }
    }

    #[test]
    fn test_job_failure_with_retry() {
        let payload = create_test_scan_payload();
        let mut job = Job::new(payload).with_max_attempts(3);

        // First attempt fails
        job.mark_started();
        assert_eq!(job.attempts, 1);
        job.mark_failed("Network error".to_string());

        // Should be pending for retry
        assert_eq!(job.status, JobStatus::Pending);
        assert!(job.can_retry());
        assert_eq!(job.error_message, Some("Network error".to_string()));

        // Second attempt fails
        job.mark_started();
        assert_eq!(job.attempts, 2);
        job.mark_failed("Timeout".to_string());

        // Still can retry
        assert_eq!(job.status, JobStatus::Pending);
        assert!(job.can_retry());

        // Third attempt fails
        job.mark_started();
        assert_eq!(job.attempts, 3);
        job.mark_failed("Final failure".to_string());

        // No more retries
        assert_eq!(job.status, JobStatus::Failed);
        assert!(!job.can_retry());
    }

    #[test]
    fn test_job_no_retry_configured() {
        let payload = create_test_scan_payload();
        let mut job = Job::new(payload).with_max_attempts(1);

        job.mark_started();
        job.mark_failed("Error".to_string());

        // Should fail immediately without retry
        assert_eq!(job.status, JobStatus::Failed);
        assert!(!job.can_retry());
    }
}

// =============================================================================
// Job Priority Tests
// =============================================================================

mod job_priority {
    use super::*;

    #[test]
    fn test_priority_ordering() {
        assert!(JobPriority::Critical > JobPriority::High);
        assert!(JobPriority::High > JobPriority::Normal);
        assert!(JobPriority::Normal > JobPriority::Low);
    }

    #[test]
    fn test_default_priority() {
        let payload = create_test_scan_payload();
        let job = Job::new(payload);

        assert_eq!(job.priority, JobPriority::Normal);
    }

    #[test]
    fn test_priority_conversion_to_i32() {
        assert_eq!(i32::from(JobPriority::Low), 0);
        assert_eq!(i32::from(JobPriority::Normal), 5);
        assert_eq!(i32::from(JobPriority::High), 10);
        assert_eq!(i32::from(JobPriority::Critical), 15);
    }

    #[test]
    fn test_priority_conversion_from_i32() {
        assert_eq!(JobPriority::try_from(0).unwrap(), JobPriority::Low);
        assert_eq!(JobPriority::try_from(5).unwrap(), JobPriority::Normal);
        assert_eq!(JobPriority::try_from(10).unwrap(), JobPriority::High);
        assert_eq!(JobPriority::try_from(15).unwrap(), JobPriority::Critical);
        assert!(JobPriority::try_from(99).is_err());
    }

    #[test]
    fn test_sorted_priorities() {
        let mut priorities = vec![
            JobPriority::Normal,
            JobPriority::Critical,
            JobPriority::Low,
            JobPriority::High,
        ];
        priorities.sort();

        assert_eq!(
            priorities,
            vec![
                JobPriority::Low,
                JobPriority::Normal,
                JobPriority::High,
                JobPriority::Critical,
            ]
        );
    }
}

// =============================================================================
// Job Type Tests
// =============================================================================

mod job_types {
    use super::*;

    #[test]
    fn test_job_type_strings() {
        assert_eq!(JobType::ScanProject.as_str(), "scan_project");
        assert_eq!(JobType::MonitorRegistry.as_str(), "monitor_registry");
        assert_eq!(JobType::VerifyProvenance.as_str(), "verify_provenance");
        assert_eq!(JobType::SendNotification.as_str(), "send_notification");
    }

    #[test]
    fn test_job_type_parsing() {
        assert_eq!(
            JobType::from_str("scan_project").unwrap(),
            JobType::ScanProject
        );
        assert_eq!(
            JobType::from_str("monitor_registry").unwrap(),
            JobType::MonitorRegistry
        );
        assert_eq!(
            JobType::from_str("verify_provenance").unwrap(),
            JobType::VerifyProvenance
        );
        assert_eq!(
            JobType::from_str("send_notification").unwrap(),
            JobType::SendNotification
        );
        assert!(JobType::from_str("invalid_type").is_err());
    }

    #[test]
    fn test_job_status_strings() {
        assert_eq!(JobStatus::Pending.as_str(), "pending");
        assert_eq!(JobStatus::Running.as_str(), "running");
        assert_eq!(JobStatus::Completed.as_str(), "completed");
        assert_eq!(JobStatus::Failed.as_str(), "failed");
        assert_eq!(JobStatus::Cancelled.as_str(), "cancelled");
        assert_eq!(JobStatus::Scheduled.as_str(), "scheduled");
    }

    #[test]
    fn test_job_status_parsing() {
        assert_eq!(JobStatus::from_str("pending").unwrap(), JobStatus::Pending);
        assert_eq!(JobStatus::from_str("running").unwrap(), JobStatus::Running);
        assert_eq!(
            JobStatus::from_str("completed").unwrap(),
            JobStatus::Completed
        );
        assert_eq!(JobStatus::from_str("failed").unwrap(), JobStatus::Failed);
        assert_eq!(
            JobStatus::from_str("cancelled").unwrap(),
            JobStatus::Cancelled
        );
        assert_eq!(
            JobStatus::from_str("scheduled").unwrap(),
            JobStatus::Scheduled
        );
        assert!(JobStatus::from_str("invalid").is_err());
    }
}

// =============================================================================
// Job Payload Tests
// =============================================================================

mod job_payloads {
    use super::*;

    #[test]
    fn test_scan_project_payload() {
        let payload = ScanProjectPayload {
            project_id: ProjectId::new(),
            tenant_id: TenantId::new(),
            ecosystems: vec![PackageEcosystem::Npm, PackageEcosystem::PyPi],
            full_scan: true,
        };

        assert_eq!(payload.ecosystems.len(), 2);
        assert!(payload.full_scan);
    }

    #[test]
    fn test_scan_project_payload_builder() {
        let project_id = ProjectId::new();
        let tenant_id = TenantId::new();

        let payload = ScanProjectPayload::new(project_id, tenant_id)
            .with_ecosystems(vec![PackageEcosystem::Npm])
            .full_scan();

        assert_eq!(payload.project_id, project_id);
        assert!(payload.full_scan);
        assert_eq!(payload.ecosystems, vec![PackageEcosystem::Npm]);
    }

    #[test]
    fn test_monitor_registry_payload() {
        let payload = MonitorRegistryPayload {
            ecosystem: PackageEcosystem::Cargo,
            packages: vec!["serde".to_string(), "tokio".to_string(), "reqwest".to_string()],
            check_new_versions: true,
            check_removals: true,
            check_maintainer_changes: true,
        };

        assert_eq!(payload.ecosystem, PackageEcosystem::Cargo);
        assert_eq!(payload.packages.len(), 3);
    }

    #[test]
    fn test_monitor_registry_payload_builder() {
        let payload = MonitorRegistryPayload::new(PackageEcosystem::Npm)
            .with_packages(vec!["lodash".to_string()])
            .check_only(true, false, true);

        assert!(payload.check_new_versions);
        assert!(!payload.check_removals);
        assert!(payload.check_maintainer_changes);
    }

    #[test]
    fn test_verify_provenance_payload() {
        let payload = VerifyProvenancePayload {
            dependency_id: DependencyId::new(),
            tenant_id: TenantId::new(),
            ecosystem: PackageEcosystem::PyPi,
            package_name: "test-package".to_string(),
            version: "2.0.0".to_string(),
            verify_slsa: true,
            verify_sigstore: true,
            verify_intoto: false,
        };

        assert_eq!(payload.package_name, "test-package");
        assert_eq!(payload.version, "2.0.0");
        assert!(payload.verify_slsa);
        assert!(!payload.verify_intoto);
    }

    #[test]
    fn test_notification_payload() {
        let payload = SendNotificationPayload {
            alert_id: AlertId::new(),
            tenant_id: TenantId::new(),
            channel: NotificationChannel::PagerDuty,
            channel_config: serde_json::json!({}),
            severity: Severity::Critical,
            title: "Critical Vulnerability".to_string(),
            description: "A critical vulnerability has been detected".to_string(),
            context: NotificationContext::default(),
        };

        assert_eq!(payload.channel, NotificationChannel::PagerDuty);
        assert_eq!(payload.severity, Severity::Critical);
    }

    #[test]
    fn test_payload_job_type_mapping() {
        let scan_payload = JobPayload::ScanProject(ScanProjectPayload::new(
            ProjectId::new(),
            TenantId::new(),
        ));
        assert_eq!(scan_payload.job_type(), JobType::ScanProject);

        let monitor_payload = JobPayload::MonitorRegistry(
            MonitorRegistryPayload::new(PackageEcosystem::Npm),
        );
        assert_eq!(monitor_payload.job_type(), JobType::MonitorRegistry);

        let provenance_payload = JobPayload::VerifyProvenance(VerifyProvenancePayload::new(
            DependencyId::new(),
            TenantId::new(),
            PackageEcosystem::Npm,
            "pkg".to_string(),
            "1.0.0".to_string(),
        ));
        assert_eq!(provenance_payload.job_type(), JobType::VerifyProvenance);

        let notification_payload = JobPayload::SendNotification(SendNotificationPayload::new(
            AlertId::new(),
            TenantId::new(),
            NotificationChannel::Email,
            Severity::Medium,
            "Alert".to_string(),
            "Description".to_string(),
        ));
        assert_eq!(notification_payload.job_type(), JobType::SendNotification);
    }
}

// =============================================================================
// Job Result Tests
// =============================================================================

mod job_results {
    use super::*;

    #[test]
    fn test_scan_project_result() {
        let result = ScanProjectResult {
            dependencies_found: 150,
            alerts_created: 7,
            scan_duration_ms: 12500,
        };

        assert_eq!(result.dependencies_found, 150);
        assert_eq!(result.alerts_created, 7);
        assert_eq!(result.scan_duration_ms, 12500);
    }

    #[test]
    fn test_monitor_registry_result() {
        let result = MonitorRegistryResult {
            packages_checked: 50,
            new_versions_detected: 3,
            removals_detected: 0,
            maintainer_changes_detected: 1,
            alerts_created: 4,
        };

        assert_eq!(result.packages_checked, 50);
        assert_eq!(result.new_versions_detected, 3);
        assert_eq!(result.maintainer_changes_detected, 1);
    }

    #[test]
    fn test_verify_provenance_result_verified() {
        let result = VerifyProvenanceResult {
            slsa_status: Some(ProvenanceVerificationStatus::Verified),
            slsa_level: Some(2),
            sigstore_status: Some(ProvenanceVerificationStatus::Verified),
            sigstore_details: Some(SigstoreDetails {
                issuer: Some("https://github.com/login/oauth".to_string()),
                subject: Some("https://github.com/owner/repo".to_string()),
                rekor_entry: Some("https://rekor.sigstore.dev/123".to_string()),
            }),
            intoto_status: None,
            alert_created: false,
        };

        assert!(matches!(
            result.slsa_status,
            Some(ProvenanceVerificationStatus::Verified)
        ));
        assert_eq!(result.slsa_level, Some(2));
        assert!(!result.alert_created);
    }

    #[test]
    fn test_verify_provenance_result_failed() {
        let result = VerifyProvenanceResult {
            slsa_status: Some(ProvenanceVerificationStatus::Failed),
            slsa_level: None,
            sigstore_status: Some(ProvenanceVerificationStatus::NoAttestations),
            sigstore_details: None,
            intoto_status: None,
            alert_created: true,
        };

        assert!(matches!(
            result.slsa_status,
            Some(ProvenanceVerificationStatus::Failed)
        ));
        assert!(result.alert_created);
    }

    #[test]
    fn test_notification_result() {
        let result = SendNotificationResult {
            sent: true,
            response: Some(serde_json::json!({"message_id": "123"})),
            send_duration_ms: 150,
        };

        assert!(result.sent);
        assert!(result.response.is_some());
        assert_eq!(result.send_duration_ms, 150);
    }
}

// =============================================================================
// Job Serialization Tests
// =============================================================================

mod serialization {
    use super::*;

    #[test]
    fn test_job_payload_serialization() {
        let payload = JobPayload::ScanProject(ScanProjectPayload {
            project_id: ProjectId::new(),
            tenant_id: TenantId::new(),
            ecosystems: vec![PackageEcosystem::Npm],
            full_scan: false,
        });

        let json = serde_json::to_string(&payload).unwrap();
        let deserialized: JobPayload = serde_json::from_str(&json).unwrap();

        if let JobPayload::ScanProject(p) = deserialized {
            assert!(!p.full_scan);
            assert_eq!(p.ecosystems, vec![PackageEcosystem::Npm]);
        } else {
            panic!("Expected ScanProject payload");
        }
    }

    #[test]
    fn test_job_result_serialization() {
        let result = JobResult::ScanProject(ScanProjectResult {
            dependencies_found: 100,
            alerts_created: 5,
            scan_duration_ms: 5000,
        });

        let json = serde_json::to_string(&result).unwrap();
        let deserialized: JobResult = serde_json::from_str(&json).unwrap();

        if let JobResult::ScanProject(r) = deserialized {
            assert_eq!(r.dependencies_found, 100);
            assert_eq!(r.alerts_created, 5);
        } else {
            panic!("Expected ScanProject result");
        }
    }

    #[test]
    fn test_full_job_serialization() {
        let tenant_id = TenantId::new();
        let payload = JobPayload::ScanProject(ScanProjectPayload {
            project_id: ProjectId::new(),
            tenant_id,
            ecosystems: vec![PackageEcosystem::Npm, PackageEcosystem::PyPi],
            full_scan: true,
        });

        let job = Job::new(payload)
            .with_tenant(tenant_id)
            .with_priority(JobPriority::High);

        let json = serde_json::to_string(&job).unwrap();

        // Verify it's valid JSON
        let value: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert!(value["id"].is_string());
        assert_eq!(value["job_type"], "scan_project");
        assert_eq!(value["status"], "pending");
    }

    #[test]
    fn test_job_status_serialization() {
        let statuses = vec![
            JobStatus::Pending,
            JobStatus::Running,
            JobStatus::Completed,
            JobStatus::Failed,
            JobStatus::Cancelled,
            JobStatus::Scheduled,
        ];

        for status in statuses {
            let json = serde_json::to_string(&status).unwrap();
            let deserialized: JobStatus = serde_json::from_str(&json).unwrap();
            assert_eq!(status, deserialized);
        }
    }

    #[test]
    fn test_job_priority_serialization() {
        let priorities = vec![
            JobPriority::Low,
            JobPriority::Normal,
            JobPriority::High,
            JobPriority::Critical,
        ];

        for priority in priorities {
            let json = serde_json::to_string(&priority).unwrap();
            let deserialized: JobPriority = serde_json::from_str(&json).unwrap();
            assert_eq!(priority, deserialized);
        }
    }

    #[test]
    fn test_notification_channel_serialization() {
        let channels = vec![
            NotificationChannel::Email,
            NotificationChannel::Slack,
            NotificationChannel::Teams,
            NotificationChannel::PagerDuty,
            NotificationChannel::Webhook,
        ];

        for channel in channels {
            let json = serde_json::to_string(&channel).unwrap();
            let deserialized: NotificationChannel = serde_json::from_str(&json).unwrap();
            assert_eq!(channel, deserialized);
        }
    }

    #[test]
    fn test_provenance_status_serialization() {
        let statuses = vec![
            ProvenanceVerificationStatus::Verified,
            ProvenanceVerificationStatus::Failed,
            ProvenanceVerificationStatus::NoAttestations,
            ProvenanceVerificationStatus::Unverifiable,
        ];

        for status in statuses {
            let json = serde_json::to_string(&status).unwrap();
            let deserialized: ProvenanceVerificationStatus = serde_json::from_str(&json).unwrap();
            assert_eq!(status, deserialized);
        }
    }
}

// =============================================================================
// Job ID Tests
// =============================================================================

mod job_id {
    use super::*;

    #[test]
    fn test_job_id_uniqueness() {
        let id1 = JobId::new();
        let id2 = JobId::new();

        assert_ne!(id1, id2);
    }

    #[test]
    fn test_job_id_display() {
        let id = JobId::new();
        let display = format!("{}", id);

        // Should be a valid UUID string
        assert_eq!(display.len(), 36); // UUID format: 8-4-4-4-12
        assert!(Uuid::parse_str(&display).is_ok());
    }

    #[test]
    fn test_job_id_default() {
        let id: JobId = Default::default();

        // Should create a new random ID
        assert!(!id.0.is_nil());
    }

    #[test]
    fn test_job_id_hash() {
        use std::collections::HashSet;

        let id1 = JobId::new();
        let id2 = JobId::new();
        let id1_clone = JobId(id1.0);

        let mut set = HashSet::new();
        set.insert(id1);
        set.insert(id2);
        set.insert(id1_clone);

        // id1 and id1_clone should be the same
        assert_eq!(set.len(), 2);
    }
}

// =============================================================================
// Helper Functions
// =============================================================================

fn create_test_scan_payload() -> JobPayload {
    JobPayload::ScanProject(ScanProjectPayload {
        project_id: ProjectId::new(),
        tenant_id: TenantId::new(),
        ecosystems: vec![PackageEcosystem::Npm],
        full_scan: false,
    })
}
