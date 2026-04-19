//! Supply Chain Trust Verifier CLI.
//!
//! A command-line tool for detecting supply chain threats in your dependencies.

mod commands;
mod shared;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "sctv")]
#[command(author, version, about = "Supply Chain Trust Verifier", long_about = None)]
struct Cli {
    /// Enable verbose output
    #[arg(short, long, global = true)]
    verbose: bool,

    /// Output format
    #[arg(short, long, global = true, default_value = "text")]
    format: OutputFormat,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Clone, Copy, Debug, Default, clap::ValueEnum)]
enum OutputFormat {
    #[default]
    Text,
    Json,
    Sarif,
}

#[derive(Subcommand)]
enum Commands {
    /// Scan dependencies for supply chain threats
    Scan {
        /// Path to project directory (default: current directory)
        #[arg(short, long)]
        path: Option<std::path::PathBuf>,

        /// Package ecosystem to scan
        #[arg(short, long)]
        ecosystem: Option<String>,
    },

    /// Check a specific package for typosquatting
    Check {
        /// Package name to check
        name: String,

        /// Package ecosystem
        #[arg(short, long, default_value = "npm")]
        ecosystem: String,
    },

    /// Verify package integrity
    Verify {
        /// Package name
        name: String,

        /// Package version
        version: String,

        /// Package ecosystem
        #[arg(short, long, default_value = "npm")]
        ecosystem: String,
    },

    /// Evaluate a policy against dependencies
    Policy {
        /// Path to policy file
        #[arg(short, long)]
        policy: std::path::PathBuf,

        /// Path to project directory
        #[arg(short, long)]
        path: Option<std::path::PathBuf>,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Initialize tracing
    let filter = if cli.verbose { "debug" } else { "info" };
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .init();

    match cli.command {
        Commands::Scan { path, ecosystem } => {
            commands::scan::run(path, ecosystem.as_deref(), cli.format)?;
        }
        Commands::Check { name, ecosystem } => {
            commands::check::run(&name, &ecosystem, cli.format)?;
        }
        Commands::Verify {
            name,
            version,
            ecosystem,
        } => {
            commands::verify::run(&name, &version, &ecosystem, cli.format).await?;
        }
        Commands::Policy { policy, path } => {
            commands::policy::run(&policy, path, cli.format)?;
        }
    }

    Ok(())
}
