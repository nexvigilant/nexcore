//! # sop-reactor CLI
//!
//! Command-line interface for the SOP-Anatomy-Code triple mapping,
//! I.R.O.N.M.A.N. reactor phases, and Capability Transfer Protocol.

#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use clap::{Parser, Subcommand};
use nexcore_sop_anatomy::audit::audit_path;
use nexcore_sop_anatomy::mapping::{CoverageReport, Domain, SopSection};
use nexcore_sop_anatomy::reactor::{self, IronmanPhase};
use nexcore_sop_anatomy::transfer;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "sop-reactor", version, about = "SOP-Anatomy-Code reactor")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Look up the triple mapping for an SOP section
    Map {
        /// Section number (1-18), or omit for all
        #[arg(short, long)]
        section: Option<u8>,
    },
    /// Cross-domain transfer using Capability Transfer Protocol
    Bridge {
        /// Source domain: sop, anatomy, or code
        source: String,
        /// Concept to transfer
        concept: String,
        /// Target domain: sop, anatomy, or code
        target: String,
    },
    /// Audit a crate/project against 18 governance sections
    Audit {
        /// Path to the project root
        path: PathBuf,
    },
    /// Full 18-section coverage report with bio-crate wiring
    Coverage,
    /// Run an I.R.O.N.M.A.N. reactor phase on input
    Phase {
        /// Phase: identify, react, optimize, navigate, manufacture, assay, negotiate
        phase: String,
        /// Input concept to process
        input: String,
    },
    /// Run the full I.R.O.N.M.A.N. pipeline
    Pipeline {
        /// Input concept to process through all 7 phases
        input: String,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Map { section } => cmd_map(section),
        Commands::Bridge {
            source,
            concept,
            target,
        } => cmd_bridge(&source, &concept, &target),
        Commands::Audit { path } => cmd_audit(&path),
        Commands::Coverage => cmd_coverage(),
        Commands::Phase { phase, input } => cmd_phase(&phase, &input),
        Commands::Pipeline { input } => cmd_pipeline(&input),
    }
}

fn cmd_map(section: Option<u8>) {
    match section {
        Some(n) => match SopSection::from_number(n) {
            Some(s) => {
                let json = serde_json::to_string_pretty(&s.mapping()).unwrap_or_default();
                println!("{json}");
            }
            None => eprintln!("Invalid section number: {n}. Valid range: 1-18."),
        },
        None => {
            let all: Vec<_> = SopSection::ALL.iter().map(|s| s.mapping()).collect();
            let json = serde_json::to_string_pretty(&all).unwrap_or_default();
            println!("{json}");
        }
    }
}

fn cmd_bridge(source: &str, concept: &str, target: &str) {
    let src = match Domain::from_str_loose(source) {
        Some(d) => d,
        None => {
            eprintln!("Unknown source domain: '{source}'. Use: sop, anatomy, or code.");
            return;
        }
    };
    let tgt = match Domain::from_str_loose(target) {
        Some(d) => d,
        None => {
            eprintln!("Unknown target domain: '{target}'. Use: sop, anatomy, or code.");
            return;
        }
    };

    let result = transfer::transfer(src, concept, tgt);
    let json = serde_json::to_string_pretty(&result).unwrap_or_default();
    println!("{json}");
}

fn cmd_audit(path: &PathBuf) {
    let report = audit_path(path);
    let json = serde_json::to_string_pretty(&report).unwrap_or_default();
    println!("{json}");
}

fn cmd_coverage() {
    let report = CoverageReport::generate();
    let json = serde_json::to_string_pretty(&report).unwrap_or_default();
    println!("{json}");
}

fn cmd_phase(phase_name: &str, input: &str) {
    let phase = match phase_name.to_lowercase().as_str() {
        "i" | "identify" => IronmanPhase::Identify,
        "r" | "react" => IronmanPhase::React,
        "o" | "optimize" => IronmanPhase::Optimize,
        "n" | "navigate" => IronmanPhase::Navigate,
        "m" | "manufacture" => IronmanPhase::Manufacture,
        "a" | "assay" => IronmanPhase::Assay,
        "n2" | "negotiate" => IronmanPhase::Negotiate,
        _ => {
            eprintln!(
                "Unknown phase: '{phase_name}'. Use: identify, react, optimize, navigate, manufacture, assay, negotiate."
            );
            return;
        }
    };

    let output = phase.apply(input);
    let json = serde_json::to_string_pretty(&output).unwrap_or_default();
    println!("{json}");
}

fn cmd_pipeline(input: &str) {
    let outputs = reactor::run_full_pipeline(input);
    let json = serde_json::to_string_pretty(&outputs).unwrap_or_default();
    println!("{json}");
}
