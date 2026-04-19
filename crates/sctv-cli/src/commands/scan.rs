//! Scan command - scan a project's manifest for typosquatting across its dependencies.
//!
//! Currently supports npm `package.json`. Other ecosystems are recognized but
//! reported as unsupported; full lockfile parsing is follow-up work.

use crate::shared;
use crate::OutputFormat;
use sctv_core::PackageEcosystem;
use sctv_detectors::typosquatting::TyposquattingDetector;
use std::path::{Path, PathBuf};

pub async fn run(
    path: Option<PathBuf>,
    ecosystem: Option<String>,
    format: OutputFormat,
) -> anyhow::Result<()> {
    let project_path = path.unwrap_or_else(|| PathBuf::from("."));

    let (names, detected_ecosystem) = load_dependency_names(&project_path, ecosystem.as_deref())?;

    let detector = TyposquattingDetector::new();
    let mut all_candidates = Vec::new();
    for name in &names {
        let candidates = detector.check(detected_ecosystem, name);
        all_candidates.extend(candidates);
    }

    match format {
        OutputFormat::Json => {
            let output = serde_json::json!({
                "project_path": project_path.display().to_string(),
                "ecosystem": detected_ecosystem.to_string(),
                "packages_scanned": names.len(),
                "typosquatting_candidates": all_candidates,
            });
            println!("{}", serde_json::to_string_pretty(&output)?);
        }
        OutputFormat::Sarif => {
            let alerts: Vec<_> = all_candidates
                .iter()
                .map(|c| shared::typosquat_to_alert(c, detected_ecosystem))
                .collect();
            shared::emit_sarif(&alerts)?;
        }
        OutputFormat::Text => {
            println!(
                "Scanned {} package(s) under {} ({})",
                names.len(),
                project_path.display(),
                detected_ecosystem
            );
            if all_candidates.is_empty() {
                println!("No typosquatting detected.");
            } else {
                println!("Potential typosquatting findings:");
                for c in &all_candidates {
                    println!(
                        "  - {} -> similar to {} (score {:.2})",
                        c.suspicious_name, c.popular_name, c.similarity_score
                    );
                }
            }
        }
    }

    Ok(())
}

/// Loads dependency names from a project manifest.
/// Returns (names, `resolved_ecosystem`) or an error.
fn load_dependency_names(
    project_path: &Path,
    requested_ecosystem: Option<&str>,
) -> anyhow::Result<(Vec<String>, PackageEcosystem)> {
    // If the caller specified an ecosystem, honor it; otherwise auto-detect
    // from files present in the project path.
    let ecosystem = match requested_ecosystem {
        Some("npm") => PackageEcosystem::Npm,
        Some(other) => {
            anyhow::bail!(
                "Ecosystem '{other}' not yet supported by `sctv scan`. Only 'npm' is wired up. \
                 Use `sctv check <name> --ecosystem {other}` for individual packages."
            );
        }
        None => {
            if project_path.join("package.json").exists() {
                PackageEcosystem::Npm
            } else {
                anyhow::bail!(
                    "Could not auto-detect ecosystem in {}. Pass --ecosystem.",
                    project_path.display()
                );
            }
        }
    };

    match ecosystem {
        PackageEcosystem::Npm => load_npm_package_names(project_path),
        _ => anyhow::bail!("Ecosystem {ecosystem} not yet supported by `sctv scan`."),
    }
}

fn load_npm_package_names(project_path: &Path) -> anyhow::Result<(Vec<String>, PackageEcosystem)> {
    let manifest = project_path.join("package.json");
    let raw = std::fs::read_to_string(&manifest)
        .map_err(|e| anyhow::anyhow!("Could not read {}: {e}", manifest.display()))?;
    let parsed: serde_json::Value =
        serde_json::from_str(&raw).map_err(|e| anyhow::anyhow!("Invalid package.json: {e}"))?;

    let mut names = Vec::new();
    for section in ["dependencies", "devDependencies", "peerDependencies"] {
        if let Some(obj) = parsed.get(section).and_then(|v| v.as_object()) {
            names.extend(obj.keys().cloned());
        }
    }

    Ok((names, PackageEcosystem::Npm))
}
