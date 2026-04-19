//! Monitor registry job executor.
//!
//! This executor monitors package registries for changes to watched packages.

use async_trait::async_trait;
use std::time::Instant;

use sctv_core::traits::PackageRepository;
use sctv_db::repositories::PgPackageRepository;
use sctv_registries::RegistryClientFactory;

use crate::error::{WorkerError, WorkerResult};
use crate::executor::{ExecutionContext, JobExecutor};
use crate::jobs::{
    Job, JobPayload, JobResult, JobType, MonitorRegistryPayload, MonitorRegistryResult,
};

/// Executor for monitoring package registries.
pub struct MonitorRegistryExecutor;

impl MonitorRegistryExecutor {
    /// Creates a new monitor registry executor.
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    async fn execute_monitor(
        &self,
        payload: &MonitorRegistryPayload,
        ctx: &ExecutionContext,
    ) -> WorkerResult<MonitorRegistryResult> {
        let start = Instant::now();

        let package_repo = PgPackageRepository::new(ctx.db_pool.clone());

        // Create registry client for the specified ecosystem
        let client = RegistryClientFactory::create(payload.ecosystem);

        tracing::info!(
            ecosystem = ?payload.ecosystem,
            package_count = payload.packages.len(),
            "Starting registry monitoring"
        );

        let mut packages_checked = 0u32;
        let mut new_versions_detected = 0u32;
        let mut removals_detected = 0u32;
        let mut maintainer_changes_detected = 0u32;
        let alerts_created = 0u32;

        // If specific packages are provided, check those
        // Otherwise, check all watched packages in the database
        let packages_to_check = if payload.packages.is_empty() {
            // In a real implementation, we'd query watched packages from the database
            tracing::debug!("No specific packages provided, would check all watched packages");
            Vec::new()
        } else {
            payload.packages.clone()
        };

        for package_name in &packages_to_check {
            packages_checked += 1;

            // Fetch package metadata from registry
            match client.get_package(package_name).await {
                Ok(metadata) => {
                    // Check if package exists in our cache
                    let cached = package_repo
                        .find_by_name(payload.ecosystem, package_name)
                        .await
                        .map_err(|e| WorkerError::Execution(e.to_string()))?;

                    if let Some(cached_pkg) = cached {
                        // Check for new versions
                        if payload.check_new_versions {
                            if let Some(new_updated) = metadata.package.last_updated {
                                if let Some(cached_updated) = cached_pkg.last_updated {
                                    if new_updated > cached_updated {
                                        new_versions_detected += 1;
                                        tracing::info!(
                                            package = package_name,
                                            "New version detected"
                                        );
                                    }
                                }
                            }
                        }

                        // Check for maintainer changes
                        if payload.check_maintainer_changes {
                            if metadata.package.maintainers != cached_pkg.maintainers {
                                maintainer_changes_detected += 1;
                                tracing::warn!(
                                    package = package_name,
                                    old_maintainers = ?cached_pkg.maintainers,
                                    new_maintainers = ?metadata.package.maintainers,
                                    "Maintainer change detected"
                                );
                                // In a real implementation, create an alert here
                            }
                        }

                        // Update cached package
                        if let Err(e) = package_repo.upsert(&metadata.package).await {
                            tracing::error!(
                                package = package_name,
                                error = %e,
                                "Failed to upsert cached package"
                            );
                        }
                    } else {
                        // New package, cache it
                        if let Err(e) = package_repo.upsert(&metadata.package).await {
                            tracing::error!(
                                package = package_name,
                                error = %e,
                                "Failed to insert new package"
                            );
                        }
                    }
                }
                Err(e) => {
                    // Check if package was removed/yanked
                    if payload.check_removals {
                        tracing::warn!(
                            package = package_name,
                            error = %e,
                            "Package may have been removed"
                        );
                        removals_detected += 1;
                    }
                }
            }
        }

        let duration = start.elapsed();

        tracing::info!(
            ecosystem = ?payload.ecosystem,
            packages_checked = packages_checked,
            new_versions = new_versions_detected,
            removals = removals_detected,
            maintainer_changes = maintainer_changes_detected,
            alerts_created = alerts_created,
            duration_ms = duration.as_millis(),
            "Registry monitoring completed"
        );

        Ok(MonitorRegistryResult {
            packages_checked,
            new_versions_detected,
            removals_detected,
            maintainer_changes_detected,
            alerts_created,
        })
    }
}

impl Default for MonitorRegistryExecutor {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl JobExecutor for MonitorRegistryExecutor {
    fn handles(&self) -> Vec<JobType> {
        vec![JobType::MonitorRegistry]
    }

    async fn execute(&self, job: &Job, ctx: &ExecutionContext) -> WorkerResult<JobResult> {
        let payload = match &job.payload {
            JobPayload::MonitorRegistry(p) => p,
            _ => {
                return Err(WorkerError::Execution(
                    "Invalid payload type for MonitorRegistry".into(),
                ))
            }
        };

        let result = self.execute_monitor(payload, ctx).await?;
        Ok(JobResult::MonitorRegistry(result))
    }

    fn default_timeout_secs(&self) -> u64 {
        300 // 5 minutes for registry monitoring
    }
}
