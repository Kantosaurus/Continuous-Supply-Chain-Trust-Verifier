//! Check command - verify a package name for typosquatting.

use crate::shared;
use crate::OutputFormat;
use sctv_core::PackageEcosystem;
use sctv_detectors::typosquatting::TyposquattingDetector;
use std::str::FromStr;

pub async fn run(name: &str, ecosystem: &str, format: OutputFormat) -> anyhow::Result<()> {
    let ecosystem = PackageEcosystem::from_str(ecosystem)
        .map_err(|_| anyhow::anyhow!("Unknown ecosystem: {ecosystem}"))?;

    let detector = TyposquattingDetector::new();
    let candidates = detector.check(ecosystem, name);

    match format {
        OutputFormat::Json => {
            let output = serde_json::json!({
                "package": name,
                "ecosystem": ecosystem.to_string(),
                "typosquatting_candidates": candidates,
            });
            println!("{}", serde_json::to_string_pretty(&output)?);
        }
        OutputFormat::Sarif => {
            let alerts: Vec<_> = candidates
                .iter()
                .map(|c| shared::typosquat_to_alert(c, ecosystem))
                .collect();
            shared::emit_sarif(&alerts)?;
        }
        OutputFormat::Text => {
            println!("Checking '{name}' ({ecosystem}) for typosquatting...\n");

            if candidates.is_empty() {
                println!("No typosquatting detected");
            } else {
                println!("Potential typosquatting detected!\n");
                for candidate in &candidates {
                    println!(
                        "  Similar to: {} (score: {:.2}, method: {:?})",
                        candidate.popular_name,
                        candidate.similarity_score,
                        candidate.detection_method
                    );
                }
                println!();
                println!(
                    "This package name is similar to {} popular package(s).",
                    candidates.len()
                );
                println!("Please verify this is the intended package before installing.");
            }
        }
    }

    Ok(())
}
