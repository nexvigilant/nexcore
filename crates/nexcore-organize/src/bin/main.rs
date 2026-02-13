//! CLI binary for the ORGANIZE pipeline.
//!
//! Usage:
//!   nexcore-organize [OPTIONS] <ROOT>
//!   nexcore-organize --config organize.toml

use std::path::PathBuf;
use std::process::ExitCode;

use clap::Parser;

use nexcore_organize::config::OrganizeConfig;
use nexcore_organize::pipeline::OrganizePipeline;
use nexcore_organize::report;

/// ORGANIZE — 8-step file organization pipeline grounded to T1 primitives.
#[derive(Parser, Debug)]
#[command(name = "nexcore-organize", version, about)]
struct Cli {
    /// Root directory to organize.
    #[arg()]
    root: Option<PathBuf>,

    /// Path to TOML configuration file.
    #[arg(short, long)]
    config: Option<PathBuf>,

    /// Run in live mode (actually move/delete files).
    /// Without this flag, operates in dry-run mode.
    #[arg(long)]
    live: bool,

    /// Maximum directory depth (0 = unlimited).
    #[arg(short, long, default_value = "0")]
    depth: usize,

    /// Output format.
    #[arg(short, long, default_value = "markdown")]
    format: OutputFormat,

    /// Path to save/load state for drift detection.
    #[arg(short, long)]
    state: Option<PathBuf>,

    /// Analyze only (always dry-run, ignores --live).
    #[arg(long)]
    analyze: bool,
}

#[derive(Debug, Clone, clap::ValueEnum)]
enum OutputFormat {
    Markdown,
    Json,
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    let config = match build_config(&cli) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error: {e}");
            return ExitCode::FAILURE;
        }
    };

    let pipeline = OrganizePipeline::new(config);

    let result = if cli.analyze {
        pipeline.analyze()
    } else {
        pipeline.run()
    };

    match result {
        Ok(result) => {
            let output = match cli.format {
                OutputFormat::Markdown => report::markdown_report(&result),
                OutputFormat::Json => match report::json_report(&result) {
                    Ok(json) => json,
                    Err(e) => {
                        eprintln!("JSON error: {e}");
                        return ExitCode::FAILURE;
                    }
                },
            };
            println!("{output}");
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("Pipeline error: {e}");
            ExitCode::FAILURE
        }
    }
}

fn build_config(cli: &Cli) -> Result<OrganizeConfig, nexcore_organize::error::OrganizeError> {
    let mut config = if let Some(ref config_path) = cli.config {
        OrganizeConfig::from_toml(config_path)?
    } else if let Some(ref root) = cli.root {
        OrganizeConfig::default_for(root)
    } else {
        OrganizeConfig::default_for(".")
    };

    // CLI overrides
    if cli.live && !cli.analyze {
        config.dry_run = false;
    }

    if cli.depth > 0 {
        config.max_depth = cli.depth;
    }

    if let Some(ref state) = cli.state {
        config.state_path = Some(state.clone());
    }

    // Override root if provided alongside config
    if let Some(ref root) = cli.root {
        config.root = root.clone();
    }

    Ok(config)
}
