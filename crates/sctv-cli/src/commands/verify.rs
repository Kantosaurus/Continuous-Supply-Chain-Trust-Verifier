//! Verify command - fetch package metadata and report its integrity fields.
//!
//! The CLI does not download package archives (that would require egress on
//! every run), so "verify" here means: fetch the registry's advertised
//! checksums for the given name/version and surface them. Hash verification
//! against a local file is follow-up work.

use crate::OutputFormat;
use sctv_core::PackageEcosystem;
use sctv_registries::{npm::NpmClient, RegistryClient};
use std::str::FromStr;

pub async fn run(
    name: &str,
    version: &str,
    ecosystem: &str,
    format: OutputFormat,
) -> anyhow::Result<()> {
    let ecosystem = PackageEcosystem::from_str(ecosystem)
        .map_err(|_| anyhow::anyhow!("Unknown ecosystem: {}", ecosystem))?;

    match ecosystem {
        PackageEcosystem::Npm => {}
        _ => anyhow::bail!(
            "`sctv verify` currently supports only the npm ecosystem; got {ecosystem}."
        ),
    }

    let client = NpmClient::new();
    let metadata = client
        .get_version(name, version)
        .await
        .map_err(|e| anyhow::anyhow!("Registry lookup failed for {name}@{version}: {e}"))?;

    let checksums = &metadata.version.checksums;
    match format {
        OutputFormat::Json => {
            let output = serde_json::json!({
                "package": name,
                "version": version,
                "ecosystem": ecosystem.to_string(),
                "checksums": {
                    "sha1": checksums.sha1,
                    "sha256": checksums.sha256,
                    "sha512": checksums.sha512,
                    "integrity": checksums.integrity,
                },
                "published_at": metadata.version.published_at,
                "download_url": metadata.download_url.as_ref().map(ToString::to_string),
            });
            println!("{}", serde_json::to_string_pretty(&output)?);
        }
        OutputFormat::Sarif => {
            // No findings to report unless we actually compared against a
            // local file; emit an empty SARIF run.
            crate::shared::emit_sarif(&[])?;
        }
        OutputFormat::Text => {
            println!("{} {}@{}", ecosystem, name, version);
            if let Some(sha1) = &checksums.sha1 {
                println!("  sha1:     {sha1}");
            }
            if let Some(sha256) = &checksums.sha256 {
                println!("  sha256:   {sha256}");
            }
            if let Some(sha512) = &checksums.sha512 {
                println!("  sha512:   {sha512}");
            }
            if let Some(integrity) = &checksums.integrity {
                println!("  integrity: {integrity}");
            }
            if !checksums.has_any() {
                println!("  (registry reported no checksums)");
            }
        }
    }

    Ok(())
}
