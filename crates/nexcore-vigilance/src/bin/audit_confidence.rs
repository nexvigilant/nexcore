//! # Audit Confidence CLI
//!
//! Command-line tool for calculating deletion confidence using
//! Bayesian evidence weighting and Wilson score intervals.
//!
//! ## Usage
//!
//! ```bash
//! # All evidence confirmed
//! audit-confidence D G B E A
//!
//! # Some partial
//! audit-confidence D G B --partial E A
//!
//! # Interactive mode
//! audit-confidence --interactive
//! ```

use clap::Parser;
use nexcore_vigilance::stark::audit_confidence::{
    run_interactive, run_with_args, standard_evidence,
};

/// Dead File Audit Confidence Calculator
///
/// Calculates deletion confidence using Bayesian evidence weighting
/// and Wilson score confidence intervals.
#[derive(Parser, Debug)]
#[command(name = "audit-confidence")]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Evidence codes that are FULLY confirmed (D, G, B, E, A)
    #[arg(value_name = "CODE")]
    evidence: Vec<String>,

    /// Evidence codes that are PARTIALLY confirmed
    #[arg(short, long, value_name = "CODE")]
    partial: Vec<String>,

    /// Run in interactive mode
    #[arg(short, long)]
    interactive: bool,

    /// Show available evidence codes
    #[arg(long)]
    list_codes: bool,
}

fn main() {
    let args = Args::parse();

    if args.list_codes {
        println!("\nAvailable Evidence Codes:\n");
        for e in standard_evidence() {
            println!("  {} - {}", e.code, e.name);
            println!("      {}\n", e.description);
        }
        return;
    }

    let assessment = if args.interactive || (args.evidence.is_empty() && args.partial.is_empty()) {
        match run_interactive() {
            Ok(a) => a,
            Err(e) => {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
    } else {
        let confirmed: Vec<&str> = args.evidence.iter().map(|s| s.as_str()).collect();
        let partial: Vec<&str> = args.partial.iter().map(|s| s.as_str()).collect();
        run_with_args(&confirmed, &partial)
    };

    assessment.print_report();
}
