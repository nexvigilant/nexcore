//! Disney Loop CLI: ρ(t) → ∂(¬σ⁻¹) → ∃(ν) → ρ(t+1)
//!
//! Forward-only compound discovery pipeline.
#![forbid(unsafe_code)]

use anyhow::{Context, Result};
use clap::Parser;
use polars::prelude::*;
use std::io::{self, BufRead};
use std::path::PathBuf;
use tracing::{error, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use disney_loop::*;

/// Disney Loop — forward-only compound discovery pipeline.
///
/// Reads JSON from stdin, filters backward regression, aggregates
/// novelty by domain, and writes new state to a JSON file.
#[derive(Parser, Debug)]
#[command(name = "disney-loop")]
#[command(version)]
#[command(about = "Forward-only compound discovery loop based on T1 primitive decomposition.")]
struct Args {
    /// Output file path for the new state JSON.
    #[arg(short, long, default_value = "output/state_next.json")]
    output: PathBuf,

    /// Mode: "discovery" or "humanize"
    #[arg(short, long, default_value = "discovery")]
    mode: String,

    /// Dry run — validate configuration only.
    #[arg(long)]
    dry_run: bool,

    /// Verbose output (-v, -vv, -vvv).
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,
}

#[derive(Debug, Default)]
struct PipelineStats {
    records_ingested: u64,
    records_written: u64,
    duration_secs: f64,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    let level = match args.verbose {
        0 => tracing::Level::INFO,
        1 => tracing::Level::DEBUG,
        _ => tracing::Level::TRACE,
    };
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().with_target(true))
        .with(tracing_subscriber::filter::LevelFilter::from_level(level))
        .init();

    info!(
        pipeline = "disney-loop",
        mode = %args.mode,
        "Starting: ρ(t) → ∂(¬σ⁻¹) → ∃(ν) → ρ(t+1)"
    );

    if args.dry_run {
        info!("Dry run — configuration valid, exiting");
        return Ok(());
    }

    let result = if args.mode == "humanize" {
        run_humanize_pipeline(&args.output).await
    } else {
        run_pipeline(&args.output).await
    };

    match result {
        Ok(stats) => {
            info!(
                ingested = stats.records_ingested,
                written = stats.records_written,
                duration = stats.duration_secs,
                "Pipeline complete — new state written"
            );
            Ok(())
        }
        Err(e) => {
            error!(error = %e, "Pipeline failed");
            Err(e)
        }
    }
}

async fn run_humanize_pipeline(output: &PathBuf) -> Result<PipelineStats> {
    let start = std::time::Instant::now();
    let mut stats = PipelineStats::default();

    info!(stage = "state-assessment", "Ingesting text for humanization");
    let stdin = io::stdin();
    let lines: Vec<String> = stdin
        .lock()
        .lines()
        .filter_map(|line| line.ok())
        .filter(|line| !line.trim().is_empty())
        .collect();

    let json_str = if lines.len() == 1 && lines[0].trim().starts_with('[') {
        lines[0].clone()
    } else {
        format!("[{}]", lines.join(","))
    };

    let cursor = std::io::Cursor::new(json_str);
    let df = JsonReader::new(cursor)
        .finish()
        .context("Failed to parse JSON from stdin")?;

    stats.records_ingested = df.height() as u64;
    let df = df.lazy();

    // Stage 2: ∂(¬σ⁻¹) — Humanization Gate
    let df = humanize::transform_humanization_gate(df, 0.65)?;

    // Stage 3: ∃(ν) — Phrasing Discovery
    let df = humanize::transform_phrasing_discovery(df)?;

    // Stage 4: ρ(t+1) — New State Sink
    stats.records_written = sink_new_state(df, output)?;

    stats.duration_secs = start.elapsed().as_secs_f64();
    Ok(stats)
}

async fn run_pipeline(output: &PathBuf) -> Result<PipelineStats> {
    let start = std::time::Instant::now();
    let mut stats = PipelineStats::default();

    // Stage 1: ρ(t) — State Assessment (stdin ingest)
    info!(
        stage = "state-assessment",
        "Reading current state from stdin"
    );
    let stdin = io::stdin();
    let lines: Vec<String> = stdin
        .lock()
        .lines()
        .filter_map(|line| line.ok())
        .filter(|line| !line.trim().is_empty())
        .collect();

    let json_str = if lines.len() == 1 && lines[0].trim().starts_with('[') {
        lines[0].clone()
    } else {
        format!("[{}]", lines.join(","))
    };

    let cursor = std::io::Cursor::new(json_str);
    let df = JsonReader::new(cursor)
        .finish()
        .context("Failed to parse JSON from stdin")?;

    stats.records_ingested = df.height() as u64;
    info!(records = stats.records_ingested, "State ingested");

    let df = df.lazy();

    // Stage 2: ∂(¬σ⁻¹) — Anti-Regression Gate
    info!(
        stage = "anti-regression-gate",
        "Filtering backward regression"
    );
    let df = transform_anti_regression_gate(df).context("Anti-regression gate failed")?;

    // Stage 3: ∃(ν) — Curiosity Search
    info!(stage = "curiosity-search", "Aggregating novelty by domain");
    let df = transform_curiosity_search(df).context("Curiosity search failed")?;

    // Stage 4: ρ(t+1) — New State Sink
    info!(stage = "new-state", path = %output.display(), "Writing new state");
    stats.records_written = sink_new_state(df, output).context("Sink failed")?;

    stats.duration_secs = start.elapsed().as_secs_f64();
    Ok(stats)
}
