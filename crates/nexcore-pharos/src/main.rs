//! PHAROS CLI — run the surveillance pipeline from the command line.
//!
//! Usage:
//!   pharos run --faers-dir /path/to/faers [--config pharos.toml]
//!   pharos run --faers-dir /path/to/faers --strict
//!   pharos run --faers-dir /path/to/faers --sensitive --min-cases 2

use std::path::PathBuf;

use nexcore_error::Result;
use clap::{Parser, Subcommand};
use tracing_subscriber::EnvFilter;

use nexcore_pharos::{PharosConfig, PharosPipeline, SignalThresholds};

#[derive(Parser)]
#[command(
    name = "pharos",
    about = "PHAROS — Pharmacovigilance Autonomous Reconnaissance and Observation System"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run the full surveillance pipeline.
    Run {
        /// Path to FAERS quarterly ASCII data directory.
        #[arg(long)]
        faers_dir: PathBuf,

        /// Output directory for results.
        #[arg(long, default_value = "./output/pharos")]
        output_dir: PathBuf,

        /// Path to TOML config file (optional, overrides defaults).
        #[arg(long)]
        config: Option<PathBuf>,

        /// Minimum case count threshold.
        #[arg(long, default_value = "3")]
        min_cases: i64,

        /// Include all drug roles (not just primary suspect).
        #[arg(long)]
        all_roles: bool,

        /// Use strict thresholds (fewer false positives).
        #[arg(long)]
        strict: bool,

        /// Use sensitive thresholds (more signals detected).
        #[arg(long)]
        sensitive: bool,

        /// Top N signals to include in report.
        #[arg(long, default_value = "50")]
        top_n: usize,

        /// Skip Guardian signal injection.
        #[arg(long)]
        no_guardian: bool,

        /// Skip cytokine emission.
        #[arg(long)]
        no_cytokines: bool,
    },
}

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Run {
            faers_dir,
            output_dir,
            config,
            min_cases,
            all_roles,
            strict,
            sensitive,
            top_n,
            no_guardian,
            no_cytokines,
        } => {
            let mut pharos_config = if let Some(config_path) = config {
                PharosConfig::from_file(&config_path)?
            } else {
                PharosConfig::default()
            };

            // CLI overrides
            pharos_config.faers_dir = faers_dir;
            pharos_config.output_dir = output_dir;
            pharos_config.min_cases = min_cases;
            pharos_config.include_all_roles = all_roles;
            pharos_config.top_n_report = top_n;
            pharos_config.inject_guardian = !no_guardian;
            pharos_config.emit_cytokines = !no_cytokines;

            if strict {
                pharos_config.thresholds = SignalThresholds::strict();
            } else if sensitive {
                pharos_config.thresholds = SignalThresholds::sensitive();
            }

            pharos_config.validate()?;

            let pipeline = PharosPipeline::new(pharos_config);
            let output = pipeline.execute()?;

            tracing::info!(
                actionable = output.report.actionable_signals,
                duration_ms = output.report.duration_ms,
                "PHAROS complete"
            );

            // Print summary to stdout
            println!("{}", output.report.summary());

            if !output.actionable_signals.is_empty() {
                println!("\nTop actionable signals:");
                for (i, sig) in output.actionable_signals.iter().take(10).enumerate() {
                    println!(
                        "  {}. {} + {} | cases={} PRR={:.1} EB05={:.1} [{}]",
                        i + 1,
                        sig.drug,
                        sig.event,
                        sig.case_count,
                        sig.prr,
                        sig.eb05,
                        sig.threat_level(),
                    );
                }
            }

            Ok(())
        }
    }
}
