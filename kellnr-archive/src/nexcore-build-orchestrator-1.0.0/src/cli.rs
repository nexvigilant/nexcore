//! CLI interface — clap subcommands for the orchestrator.
//!
//! ## Primitive Foundation
//! | Primitive | Manifestation |
//! |-----------|---------------|
//! | T1: Σ (Sum) | Subcommand variants |
//! | T1: μ (Mapping) | Args → action mapping |

use clap::{Parser, Subcommand};

/// Build orchestrator for the NexCore workspace.
///
/// Tier: T2-C (Σ + μ + σ + →, dominant Σ)
#[derive(Parser, Debug)]
#[command(name = "orchestrator-cli")]
#[command(about = "NexCore build orchestrator — CI/CD pipeline management")]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Workspace root override (default: auto-detect).
    #[arg(long, global = true)]
    pub workspace: Option<String>,

    /// Verbose output.
    #[arg(short, long, global = true)]
    pub verbose: bool,
}

/// Subcommands.
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Run a pipeline.
    Run {
        /// Pipeline name (validate, validate-quick, or custom).
        #[arg(default_value = "validate-quick")]
        pipeline: String,

        /// Force run even if sources unchanged.
        #[arg(long)]
        force: bool,
    },

    /// Show current pipeline status.
    Status,

    /// Show build history.
    History {
        /// Number of recent runs to show.
        #[arg(short = 'n', default_value = "10")]
        limit: usize,

        /// Filter by status (completed, failed, cancelled).
        #[arg(long)]
        status: Option<String>,
    },

    /// Scan workspace for crates and health.
    Workspace,

    /// Show execution plan without running (dry run).
    Plan {
        /// Pipeline name.
        #[arg(default_value = "validate")]
        pipeline: String,
    },

    /// Prune old history entries.
    Prune {
        /// Number of recent runs to keep.
        #[arg(default_value = "50")]
        keep: usize,
    },

    /// Start the web dashboard (requires ssr feature).
    Serve {
        /// Port for the dashboard.
        #[arg(short, long, default_value = "3100")]
        port: u16,
    },
}
