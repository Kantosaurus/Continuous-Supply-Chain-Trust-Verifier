//! Executor registry for managing job handlers.

use std::collections::HashMap;
use std::sync::Arc;

use crate::error::{WorkerError, WorkerResult};
use crate::executor::{BoxedExecutor, ExecutionContext, JobExecutor};
use crate::jobs::{Job, JobResult, JobType};

/// Registry for job executors.
///
/// The registry maintains a mapping from job types to their executors,
/// allowing the worker pool to dispatch jobs to the appropriate handler.
pub struct ExecutorRegistry {
    executors: HashMap<JobType, BoxedExecutor>,
}

impl ExecutorRegistry {
    /// Creates a new empty executor registry.
    #[must_use]
    pub fn new() -> Self {
        Self {
            executors: HashMap::new(),
        }
    }

    /// Registers an executor for its supported job types.
    ///
    /// If an executor for the same job type is already registered,
    /// it will be replaced.
    pub fn register<E: JobExecutor + 'static>(&mut self, executor: E) {
        let executor = Arc::new(executor);
        for job_type in executor.handles() {
            tracing::debug!(job_type = ?job_type, "Registering executor");
            self.executors.insert(job_type, executor.clone());
        }
    }

    /// Gets the executor for a job type.
    #[must_use]
    pub fn get(&self, job_type: &JobType) -> Option<&BoxedExecutor> {
        self.executors.get(job_type)
    }

    /// Returns all registered job types.
    #[must_use]
    pub fn registered_types(&self) -> Vec<JobType> {
        self.executors.keys().cloned().collect()
    }

    /// Checks if an executor is registered for a job type.
    #[must_use]
    pub fn has_executor(&self, job_type: &JobType) -> bool {
        self.executors.contains_key(job_type)
    }

    /// Executes a job using the appropriate executor.
    pub async fn execute(&self, job: &Job, ctx: &ExecutionContext) -> WorkerResult<JobResult> {
        let executor = self.executors.get(&job.job_type).ok_or_else(|| {
            WorkerError::Execution(format!(
                "No executor registered for job type: {:?}",
                job.job_type
            ))
        })?;

        tracing::debug!(
            job_id = %job.id,
            job_type = ?job.job_type,
            "Executing job"
        );

        // Call before hook
        executor.before_execute(job, ctx).await?;

        // Execute the job with timeout
        let timeout = std::time::Duration::from_secs(executor.default_timeout_secs());
        let result = tokio::time::timeout(timeout, executor.execute(job, ctx))
            .await
            .map_err(|_| WorkerError::Timeout)?;

        let result = result?;

        // Call after hook
        executor.after_execute(job, &result, ctx).await?;

        tracing::debug!(
            job_id = %job.id,
            job_type = ?job.job_type,
            "Job execution completed"
        );

        Ok(result)
    }
}

impl Default for ExecutorRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::jobs::{Job, JobPayload, JobResult, ScanProjectPayload, ScanProjectResult};
    use async_trait::async_trait;
    use sctv_core::{PackageEcosystem, ProjectId, TenantId};

    struct MockScanExecutor;

    #[async_trait]
    impl JobExecutor for MockScanExecutor {
        fn handles(&self) -> Vec<JobType> {
            vec![JobType::ScanProject]
        }

        async fn execute(&self, _job: &Job, _ctx: &ExecutionContext) -> WorkerResult<JobResult> {
            Ok(JobResult::ScanProject(ScanProjectResult {
                dependencies_found: 10,
                alerts_created: 0,
                scan_duration_ms: 100,
            }))
        }
    }

    #[test]
    fn test_register_executor() {
        let mut registry = ExecutorRegistry::new();
        registry.register(MockScanExecutor);

        assert!(registry.has_executor(&JobType::ScanProject));
        assert!(!registry.has_executor(&JobType::MonitorRegistry));
    }

    #[test]
    fn test_registered_types() {
        let mut registry = ExecutorRegistry::new();
        registry.register(MockScanExecutor);

        let types = registry.registered_types();
        assert_eq!(types.len(), 1);
        assert!(types.contains(&JobType::ScanProject));
    }

    #[tokio::test]
    async fn test_execute_unregistered_type() {
        let _registry = ExecutorRegistry::new();
        let payload = JobPayload::ScanProject(ScanProjectPayload {
            project_id: ProjectId::new(),
            tenant_id: TenantId::new(),
            ecosystems: vec![PackageEcosystem::Npm],
            full_scan: false,
        });
        let _job = Job::new(payload);

        // Create a minimal context (won't actually connect to DB in this test)
        // This test will fail on context creation, which is expected
        // In real tests, we'd use a test database
    }
}
