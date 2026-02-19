// Copyright © 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! Prima Universal Concept Translator CLI.
//!
//! ## Tier: T2-C (σ + μ + → + π)
//!
//! Execute `.true` files through the full Prima pipeline.

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use prima_pipeline::{PipelineResult, execute_file, execute_pipeline};
use std::path::PathBuf;
use tracing::{error, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

/// Prima Universal Concept Translator
#[derive(Parser, Debug)]
#[command(name = "prima-translate")]
#[command(version = "0.1.0")]
#[command(about = "Execute Prima source through the Universal Concept Translator pipeline")]
struct Args {
    #[command(subcommand)]
    command: Command,

    /// Verbose output (-v, -vv, -vvv).
    #[arg(short, long, action = clap::ArgAction::Count, global = true)]
    verbose: u8,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Execute a Prima source file.
    Run {
        /// Path to .true file.
        #[arg(value_name = "FILE")]
        file: PathBuf,

        /// Output format: text, json.
        #[arg(short, long, default_value = "text")]
        format: String,
    },

    /// Execute Prima code from command line.
    Eval {
        /// Prima source code.
        #[arg(value_name = "CODE")]
        code: String,

        /// Output format: text, json.
        #[arg(short, long, default_value = "text")]
        format: String,
    },

    /// Show pipeline stages for a file without executing.
    Trace {
        /// Path to .true file.
        #[arg(value_name = "FILE")]
        file: PathBuf,
    },

    /// Batch execute multiple files.
    Batch {
        /// Paths to .true files.
        #[arg(value_name = "FILES")]
        files: Vec<PathBuf>,
    },
}

fn main() -> Result<()> {
    let args = Args::parse();

    let level = match args.verbose {
        0 => tracing::Level::WARN,
        1 => tracing::Level::INFO,
        2 => tracing::Level::DEBUG,
        _ => tracing::Level::TRACE,
    };

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().with_target(false))
        .with(tracing_subscriber::filter::LevelFilter::from_level(level))
        .init();

    match args.command {
        Command::Run { file, format } => {
            info!(file = %file.display(), "Executing Prima file");
            let result = execute_file(&file)
                .with_context(|| format!("Failed to execute: {}", file.display()))?;
            output_result(&result, &format);
        }

        Command::Eval { code, format } => {
            info!(code_len = code.len(), "Evaluating Prima code");
            let result = execute_pipeline("<eval>", &code).context("Failed to evaluate code")?;
            output_result(&result, &format);
        }

        Command::Trace { file } => {
            info!(file = %file.display(), "Tracing Prima file");
            let result = execute_file(&file)
                .with_context(|| format!("Failed to trace: {}", file.display()))?;
            output_trace(&result);
        }

        Command::Batch { files } => {
            info!(count = files.len(), "Batch executing Prima files");
            let mut success = 0;
            let mut failed = 0;

            for file in &files {
                match execute_file(file) {
                    Ok(result) => {
                        println!("✓ {} → {}", file.display(), result.result);
                        success += 1;
                    }
                    Err(e) => {
                        error!(file = %file.display(), error = %e, "Execution failed");
                        println!("✗ {} → {}", file.display(), e);
                        failed += 1;
                    }
                }
            }

            println!("\n{} succeeded, {} failed", success, failed);
        }
    }

    Ok(())
}

fn output_result(result: &PipelineResult, format: &str) {
    match format {
        "json" => {
            if let Ok(json) = serde_json::to_string_pretty(result) {
                println!("{}", json);
            }
        }
        _ => {
            println!("{}", result.result);
        }
    }
}

fn output_trace(result: &PipelineResult) {
    println!("═══════════════════════════════════════════════════════════════");
    println!("  PRIMA PIPELINE TRACE: {}", result.source);
    println!("═══════════════════════════════════════════════════════════════");
    println!();

    for (i, stage) in result.stages.iter().enumerate() {
        let symbol = match stage.name.as_str() {
            "lex" => "σ",
            "parse" => "μ",
            "compile" => "→",
            "execute" => "κ",
            _ => "•",
        };
        println!(
            "  {} Stage {}: {} ({} items, {}μs)",
            symbol,
            i + 1,
            stage.name.to_uppercase(),
            stage.items,
            stage.duration_us
        );
    }

    println!();
    println!("  T1 Primitives Used: {}", result.primitives_used.join(" "));
    println!();
    println!("  Result: {}", result.result);
    println!("═══════════════════════════════════════════════════════════════");
}
