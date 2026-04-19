//! Database models for `SQLx` queries.
//!
//! These models represent the database row structures and provide
//! conversions to/from domain models.

use sctv_core::{PackageEcosystem, ProjectStatus};

/// Parses an ecosystem string to the enum variant.
#[must_use]
pub fn parse_ecosystem(s: &str) -> Option<PackageEcosystem> {
    match s.to_lowercase().as_str() {
        "npm" => Some(PackageEcosystem::Npm),
        "pypi" => Some(PackageEcosystem::PyPi),
        "maven" => Some(PackageEcosystem::Maven),
        "nuget" => Some(PackageEcosystem::NuGet),
        "rubygems" => Some(PackageEcosystem::RubyGems),
        "cargo" => Some(PackageEcosystem::Cargo),
        "gomodules" | "go_modules" => Some(PackageEcosystem::GoModules),
        _ => None,
    }
}

/// Parses a status string to the enum variant.
#[must_use]
pub fn parse_status(s: &str) -> ProjectStatus {
    match s.to_lowercase().as_str() {
        "healthy" => ProjectStatus::Healthy,
        "warning" => ProjectStatus::Warning,
        "critical" => ProjectStatus::Critical,
        _ => ProjectStatus::Unknown,
    }
}

/// Converts a project status to a database string.
#[must_use]
pub const fn status_to_string(status: ProjectStatus) -> &'static str {
    match status {
        ProjectStatus::Healthy => "healthy",
        ProjectStatus::Warning => "warning",
        ProjectStatus::Critical => "critical",
        ProjectStatus::Unknown => "unknown",
    }
}
