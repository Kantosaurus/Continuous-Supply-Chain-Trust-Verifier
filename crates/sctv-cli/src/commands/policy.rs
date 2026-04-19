//! Policy command - load a policy JSON file and report its structure.
//!
//! Full policy evaluation (enforcing rules against scanned dependencies)
//! requires the scan pipeline to be wired across all ecosystems and is
//! follow-up work. This command validates the policy file and summarizes
//! its rules so operators can at least catch schema mistakes locally.

use crate::OutputFormat;
use sctv_core::Policy;
use std::path::{Path, PathBuf};

pub async fn run(
    policy_path: &Path,
    project_path: Option<PathBuf>,
    format: OutputFormat,
) -> anyhow::Result<()> {
    let project = project_path.unwrap_or_else(|| PathBuf::from("."));

    let raw = std::fs::read_to_string(policy_path)
        .map_err(|e| anyhow::anyhow!("Could not read {}: {e}", policy_path.display()))?;
    let policy: Policy =
        serde_json::from_str(&raw).map_err(|e| anyhow::anyhow!("Invalid policy JSON: {e}"))?;

    match format {
        OutputFormat::Json => {
            let output = serde_json::json!({
                "policy_file": policy_path.display().to_string(),
                "project": project.display().to_string(),
                "policy": {
                    "name": policy.name,
                    "rule_count": policy.rules.len(),
                    "severity_override_count": policy.severity_overrides.len(),
                    "is_default": policy.is_default,
                    "enabled": policy.enabled,
                },
                "evaluation": "not_implemented",
            });
            println!("{}", serde_json::to_string_pretty(&output)?);
        }
        OutputFormat::Sarif => {
            // Evaluation not implemented yet — emit an empty SARIF run.
            crate::shared::emit_sarif(&[])?;
        }
        OutputFormat::Text => {
            println!(
                "Loaded policy '{}' from {}",
                policy.name,
                policy_path.display()
            );
            println!("  rules: {}", policy.rules.len());
            println!("  severity overrides: {}", policy.severity_overrides.len());
            println!("  enabled: {}", policy.enabled);
            println!("  project: {}", project.display());
            println!();
            println!(
                "Policy evaluation against project dependencies is not yet implemented; \
                 this command currently validates the policy file's shape only."
            );
        }
    }

    Ok(())
}
