//! Scan command - scan a project for supply chain threats.

use crate::OutputFormat;
use std::path::PathBuf;

pub async fn run(
    path: Option<PathBuf>,
    ecosystem: Option<String>,
    format: OutputFormat,
) -> anyhow::Result<()> {
    let project_path = path.unwrap_or_else(|| PathBuf::from("."));

    println!("Scanning project at: {}", project_path.display());

    if let Some(eco) = &ecosystem {
        println!("Ecosystem filter: {}", eco);
    }

    // TODO: Implement full project scanning
    // 1. Detect lock files (package-lock.json, Cargo.lock, requirements.txt, etc.)
    // 2. Parse dependencies
    // 3. Run all detectors
    // 4. Output results

    match format {
        OutputFormat::Json => {
            let output = serde_json::json!({
                "status": "not_implemented",
                "message": "Full project scanning coming soon",
            });
            println!("{}", serde_json::to_string_pretty(&output)?);
        }
        OutputFormat::Sarif => {
            println!("SARIF output not yet implemented");
        }
        OutputFormat::Text => {
            println!("\n⚠ Full project scanning not yet implemented.");
            println!("Use 'sctv check <package-name>' to check individual packages.");
        }
    }

    Ok(())
}
