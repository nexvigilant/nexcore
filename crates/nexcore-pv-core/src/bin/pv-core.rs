//! # PV Core CLI
//!
//! Standalone micro-app for pharmacovigilance computation.
//! Signal detection, causality assessment, and safety margin analysis.

#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::process::ExitCode;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "pv-core",
    version,
    about = "Pharmacovigilance computation — signals, causality, safety margin"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Compute all disproportionality signals (PRR, ROR, IC, EBGM, Chi²)
    Signals {
        #[arg(long)]
        a: u64,
        #[arg(long)]
        b: u64,
        #[arg(long)]
        c: u64,
        #[arg(long)]
        d: u64,
    },

    /// Run Naranjo causality assessment (quick — 5 key questions, each 1/0/-1)
    Naranjo {
        #[arg(long)]
        temporal: i32,
        #[arg(long)]
        dechallenge: i32,
        #[arg(long)]
        rechallenge: i32,
        #[arg(long)]
        alternatives: i32,
        #[arg(long)]
        previous: i32,
    },

    /// Run WHO-UMC causality assessment (quick — 5 key questions, each 1/0/-1)
    WhoUmc {
        #[arg(long)]
        temporal: i32,
        #[arg(long)]
        dechallenge: i32,
        #[arg(long)]
        rechallenge: i32,
        #[arg(long)]
        alternatives: i32,
        #[arg(long)]
        plausibility: i32,
    },

    /// Compute safety margin d(s) from signal metrics
    SafetyMargin {
        #[arg(long)]
        prr: f64,
        #[arg(long, default_value = "1.0")]
        ror_lower: f64,
        #[arg(long, default_value = "0.0")]
        ic025: f64,
        #[arg(long, default_value = "1.0")]
        eb05: f64,
        #[arg(long, default_value = "10")]
        n: u32,
    },

    /// Full pipeline: signals + causality + safety margin
    Pipeline {
        #[arg(long)]
        drug: String,
        #[arg(long)]
        event: String,
        #[arg(long)]
        a: u64,
        #[arg(long)]
        b: u64,
        #[arg(long)]
        c: u64,
        #[arg(long)]
        d: u64,
        #[arg(long, default_value = "1")]
        temporal: i32,
        #[arg(long, default_value = "0")]
        dechallenge: i32,
        #[arg(long, default_value = "0")]
        rechallenge: i32,
        #[arg(long, default_value = "0")]
        alternatives: i32,
        #[arg(long, default_value = "1")]
        previous: i32,
    },
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match cli.command {
        Command::Signals { a, b, c, d } => cmd_signals(a, b, c, d),
        Command::Naranjo {
            temporal,
            dechallenge,
            rechallenge,
            alternatives,
            previous,
        } => cmd_naranjo(temporal, dechallenge, rechallenge, alternatives, previous),
        Command::WhoUmc {
            temporal,
            dechallenge,
            rechallenge,
            alternatives,
            plausibility,
        } => cmd_who_umc(
            temporal,
            dechallenge,
            rechallenge,
            alternatives,
            plausibility,
        ),
        Command::SafetyMargin {
            prr,
            ror_lower,
            ic025,
            eb05,
            n,
        } => cmd_safety_margin(prr, ror_lower, ic025, eb05, n),
        Command::Pipeline {
            drug,
            event,
            a,
            b,
            c,
            d,
            temporal,
            dechallenge,
            rechallenge,
            alternatives,
            previous,
        } => cmd_pipeline(
            &drug,
            &event,
            a,
            b,
            c,
            d,
            temporal,
            dechallenge,
            rechallenge,
            alternatives,
            previous,
        ),
    }
}

fn cmd_signals(a: u64, b: u64, c: u64, d: u64) -> ExitCode {
    use nexcore_pv_core::signals::evaluate_signal_complete;
    use nexcore_pv_core::thresholds::SignalCriteria;
    use nexcore_pv_core::types::ContingencyTable;

    let table = ContingencyTable::new(a, b, c, d);
    let criteria = SignalCriteria::evans();
    let result = evaluate_signal_complete(&table, &criteria);

    match serde_json::to_string_pretty(&result) {
        Ok(json) => {
            println!("{json}");
            eprintln!(
                "✓ PRR={:.2} ROR={:.2} IC={:.2} EBGM={:.2}",
                result.prr.point_estimate,
                result.ror.point_estimate,
                result.ic.point_estimate,
                result.ebgm.point_estimate,
            );
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("Error: {e}");
            ExitCode::from(1)
        }
    }
}

fn cmd_naranjo(
    temporal: i32,
    dechallenge: i32,
    rechallenge: i32,
    alternatives: i32,
    previous: i32,
) -> ExitCode {
    use nexcore_pv_core::causality::calculate_naranjo_quick;

    let result =
        calculate_naranjo_quick(temporal, dechallenge, rechallenge, alternatives, previous);

    match serde_json::to_string_pretty(&result) {
        Ok(json) => {
            println!("{json}");
            eprintln!(
                "✓ Naranjo: score={}, category={:?}",
                result.score, result.category
            );
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("Error: {e}");
            ExitCode::from(1)
        }
    }
}

fn cmd_who_umc(
    temporal: i32,
    dechallenge: i32,
    rechallenge: i32,
    alternatives: i32,
    plausibility: i32,
) -> ExitCode {
    use nexcore_pv_core::causality::calculate_who_umc_quick;

    let result = calculate_who_umc_quick(
        temporal,
        dechallenge,
        rechallenge,
        alternatives,
        plausibility,
    );

    match serde_json::to_string_pretty(&result) {
        Ok(json) => {
            println!("{json}");
            eprintln!("✓ WHO-UMC: category={:?}", result.category);
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("Error: {e}");
            ExitCode::from(1)
        }
    }
}

fn cmd_safety_margin(prr: f64, ror_lower: f64, ic025: f64, eb05: f64, n: u32) -> ExitCode {
    let margin = nexcore_pv_core::SafetyMargin::calculate(prr, ror_lower, ic025, eb05, n);

    match serde_json::to_string_pretty(&margin) {
        Ok(json) => {
            println!("{json}");
            eprintln!("✓ d(s)={:.4} — {}", margin.distance, margin.interpretation);
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("Error: {e}");
            ExitCode::from(1)
        }
    }
}

fn cmd_pipeline(
    drug: &str,
    event: &str,
    a: u64,
    b: u64,
    c: u64,
    d: u64,
    temporal: i32,
    dechallenge: i32,
    rechallenge: i32,
    alternatives: i32,
    previous: i32,
) -> ExitCode {
    use nexcore_pv_core::causality::{calculate_naranjo_quick, calculate_who_umc_quick};
    use nexcore_pv_core::signals::evaluate_signal_complete;
    use nexcore_pv_core::thresholds::SignalCriteria;
    use nexcore_pv_core::types::ContingencyTable;

    eprintln!("── PV Core Pipeline: {drug} + {event} ──");

    eprintln!("  [1/4] Computing disproportionality signals...");
    let table = ContingencyTable::new(a, b, c, d);
    let criteria = SignalCriteria::evans();
    let signals = evaluate_signal_complete(&table, &criteria);

    eprintln!("  [2/4] Naranjo causality assessment...");
    let naranjo =
        calculate_naranjo_quick(temporal, dechallenge, rechallenge, alternatives, previous);

    eprintln!("  [3/4] WHO-UMC causality assessment...");
    let who_umc =
        calculate_who_umc_quick(temporal, dechallenge, rechallenge, alternatives, previous);

    eprintln!("  [4/4] Computing safety margin...");
    let margin = nexcore_pv_core::SafetyMargin::calculate(
        signals.prr.point_estimate,
        signals.ror.lower_ci,
        signals.ic.lower_ci,
        signals.ebgm.lower_ci,
        u32::try_from(signals.n).unwrap_or(u32::MAX),
    );

    let output = serde_json::json!({
        "drug": drug,
        "event": event,
        "contingency": { "a": a, "b": b, "c": c, "d": d },
        "signals": {
            "prr": signals.prr.point_estimate,
            "ror": signals.ror.point_estimate,
            "ic": signals.ic.point_estimate,
            "ebgm": signals.ebgm.point_estimate,
            "chi_square": signals.chi_square,
            "n": signals.n,
        },
        "causality": {
            "naranjo": { "score": naranjo.score, "category": format!("{:?}", naranjo.category) },
            "who_umc": { "category": format!("{:?}", who_umc.category) },
        },
        "safety_margin": {
            "distance": margin.distance,
            "interpretation": margin.interpretation,
            "action": margin.action,
        },
    });

    match serde_json::to_string_pretty(&output) {
        Ok(json) => {
            println!("{json}");
            eprintln!(
                "✓ PRR={:.2} | Naranjo={} ({:?}) | d(s)={:.4}",
                signals.prr.point_estimate, naranjo.score, naranjo.category, margin.distance,
            );
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("Error: {e}");
            ExitCode::from(1)
        }
    }
}
