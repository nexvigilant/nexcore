//! # Signal Pipeline CLI
//!
//! Standalone micro-app for PV signal detection.
//! Every public function in `nexcore-signal-pipeline` exposed as a subcommand.

#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::process::ExitCode;

use clap::{Parser, Subcommand};
use nexcore_signal_pipeline::core::ContingencyTable;
use nexcore_signal_pipeline::relay;
use nexcore_signal_pipeline::stats;
use nexcore_signal_pipeline::threshold::EvansThreshold;

#[derive(Parser)]
#[command(
    name = "signal-pipeline",
    version,
    about = "PV signal detection pipeline — PRR/ROR/IC/EBGM + Evans thresholds + relay fidelity"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Compute all disproportionality metrics (PRR, ROR, IC, EBGM, Chi²) from a 2×2 table
    Compute {
        /// Drug-event co-occurrence count
        #[arg(long)]
        a: u64,
        /// Drug without event count
        #[arg(long)]
        b: u64,
        /// Event without drug count
        #[arg(long)]
        c: u64,
        /// Neither drug nor event count
        #[arg(long)]
        d: u64,
    },

    /// Apply Evans criteria threshold to a contingency table
    Threshold {
        /// Drug-event co-occurrence count
        #[arg(long)]
        a: u64,
        /// Drug without event count
        #[arg(long)]
        b: u64,
        /// Event without drug count
        #[arg(long)]
        c: u64,
        /// Neither drug nor event count
        #[arg(long)]
        d: u64,
    },

    /// Display the PV pipeline relay chain with fidelity targets
    Relay,

    /// Display the core detection relay chain
    DetectionChain,

    /// Full pipeline: compute metrics + apply Evans threshold + relay chain
    Pipeline {
        /// Drug name
        #[arg(long)]
        drug: String,
        /// Event name
        #[arg(long)]
        event: String,
        /// Drug-event co-occurrence count
        #[arg(long)]
        a: u64,
        /// Drug without event count
        #[arg(long)]
        b: u64,
        /// Event without drug count
        #[arg(long)]
        c: u64,
        /// Neither drug nor event count
        #[arg(long)]
        d: u64,
    },
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match cli.command {
        Command::Compute { a, b, c, d } => cmd_compute(a, b, c, d),
        Command::Threshold { a, b, c, d } => cmd_threshold(a, b, c, d),
        Command::Relay => cmd_relay(),
        Command::DetectionChain => cmd_detection_chain(),
        Command::Pipeline {
            drug,
            event,
            a,
            b,
            c,
            d,
        } => cmd_pipeline(&drug, &event, a, b, c, d),
    }
}

fn cmd_compute(a: u64, b: u64, c: u64, d: u64) -> ExitCode {
    let table = ContingencyTable::new(a, b, c, d);
    let metrics = stats::compute_all(&table);
    match serde_json::to_string_pretty(&metrics) {
        Ok(json) => {
            println!("{json}");
            eprintln!("✓ Computed PRR/ROR/IC/EBGM/Chi² for table ({a}, {b}, {c}, {d})");
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("Error serializing metrics: {e}");
            ExitCode::from(1)
        }
    }
}

fn build_detection_result(
    drug: &str,
    event: &str,
    table: ContingencyTable,
    metrics: &stats::SignalMetrics,
) -> nexcore_signal_pipeline::core::DetectionResult {
    use nexcore_signal_pipeline::core::{DetectionResult, DrugEventPair};

    DetectionResult::new(
        DrugEventPair::new(drug, event),
        table,
        metrics.prr,
        metrics.ror,
        Some(metrics.ic),
        Some(metrics.ebgm),
        metrics.chi_square,
        metrics.strength,
        nexcore_chrono::DateTime::now(),
    )
}

fn cmd_threshold(a: u64, b: u64, c: u64, d: u64) -> ExitCode {
    use nexcore_signal_pipeline::core::Threshold as _;

    let table = ContingencyTable::new(a, b, c, d);
    let metrics = stats::compute_all(&table);
    let result = build_detection_result("drug", "event", table, &metrics);

    let threshold = EvansThreshold::new();
    let passes = threshold.apply(&result);

    let output = serde_json::json!({
        "table": { "a": a, "b": b, "c": c, "d": d },
        "prr": metrics.prr.map(|p| p.0),
        "ror": metrics.ror.map(|r| r.0),
        "chi_square": metrics.chi_square.0,
        "evans_pass": passes,
        "criteria": {
            "prr_min": 2.0,
            "chi_square_min": 3.841,
            "case_count_min": 3
        }
    });

    match serde_json::to_string_pretty(&output) {
        Ok(json) => {
            println!("{json}");
            let icon = if passes { "✓" } else { "✗" };
            eprintln!(
                "{icon} Evans threshold: {}",
                if passes { "PASS" } else { "FAIL" }
            );
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("Error: {e}");
            ExitCode::from(1)
        }
    }
}

fn cmd_relay() -> ExitCode {
    let chain = relay::pv_pipeline_chain();
    match serde_json::to_string_pretty(&chain) {
        Ok(json) => {
            println!("{json}");
            eprintln!("✓ PV pipeline relay chain ({} hops)", chain.hop_count());
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("Error: {e}");
            ExitCode::from(1)
        }
    }
}

fn cmd_detection_chain() -> ExitCode {
    let chain = relay::core_detection_chain();
    match serde_json::to_string_pretty(&chain) {
        Ok(json) => {
            println!("{json}");
            eprintln!("✓ Core detection chain ({} hops)", chain.hop_count());
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("Error: {e}");
            ExitCode::from(1)
        }
    }
}

fn cmd_pipeline(drug: &str, event: &str, a: u64, b: u64, c: u64, d: u64) -> ExitCode {
    use nexcore_signal_pipeline::core::Threshold as _;

    eprintln!("── Signal Pipeline: {drug} + {event} ──");

    // Stage 1: Compute
    eprintln!("  [1/4] Computing disproportionality metrics...");
    let table = ContingencyTable::new(a, b, c, d);
    let metrics = stats::compute_all(&table);

    // Stage 2: Threshold
    eprintln!("  [2/4] Applying Evans criteria...");
    let result = build_detection_result(drug, event, table, &metrics);
    let threshold = EvansThreshold::new();
    let passes = threshold.apply(&result);

    // Stage 3: Relay
    eprintln!("  [3/4] Building relay chain...");
    let chain = relay::pv_pipeline_chain();

    // Stage 4: Report
    eprintln!("  [4/4] Generating report...");
    let output = serde_json::json!({
        "drug": drug,
        "event": event,
        "contingency": { "a": a, "b": b, "c": c, "d": d },
        "metrics": metrics,
        "evans_threshold": {
            "pass": passes,
            "criteria": { "prr_min": 2.0, "chi_square_min": 3.841, "case_count_min": 3 }
        },
        "relay": {
            "hops": chain.hop_count(),
            "total_fidelity": chain.total_fidelity(),
            "signal_loss": chain.signal_loss(),
        },
        "verdict": if passes { "SIGNAL_DETECTED" } else { "NO_SIGNAL" }
    });

    match serde_json::to_string_pretty(&output) {
        Ok(json) => {
            println!("{json}");
            let icon = if passes { "✓" } else { "✗" };
            eprintln!(
                "{icon} Verdict: {} — PRR={:.2}, Chi²={:.2}",
                if passes {
                    "SIGNAL DETECTED"
                } else {
                    "No signal"
                },
                metrics.prr.map_or(0.0, |p| p.0),
                metrics.chi_square.0,
            );
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("Error: {e}");
            ExitCode::from(1)
        }
    }
}
