// Copyright © 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! CLI for the nexcore-transcriptase engine.
//!
//! Reads JSON from stdin or a file, infers schemas, synthesizes
//! boundary violations, and optionally verifies round-trip fidelity.

#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use clap::Parser;
use nexcore_transcriptase::{Config, DiagnosticLevel, Engine, TranscriptaseError};
use std::io::Read;

/// Reverse transcriptase engine — observe JSON, infer schemas,
/// synthesize boundary violations.
#[derive(Parser, Debug)]
#[command(name = "nexcore-transcriptase")]
#[command(
    about = "Bidirectional data-to-schema pipeline with fidelity verification and violation synthesis"
)]
struct Args {
    /// Input file (reads from stdin if omitted).
    file: Option<String>,

    /// Synthesize boundary violations.
    #[arg(long, short = 'V')]
    violations: bool,

    /// Verify round-trip fidelity.
    #[arg(long)]
    verify: bool,

    /// Output as JSON (default: human-readable).
    #[arg(long)]
    json: bool,

    /// Generate N synthetic records from observed schema.
    #[arg(long, short = 'g')]
    generate: Option<usize>,

    /// Show engine statistics after processing.
    #[arg(long)]
    stats: bool,
}

fn main() -> std::result::Result<(), TranscriptaseError> {
    let args = Args::parse();

    let input = read_input(&args)?;

    let config = Config {
        synthesize_violations: args.violations,
        verify_fidelity: args.verify,
        source_name: args.file.as_deref().unwrap_or("stdin").to_string(),
    };

    let mut engine = Engine::with_config(config);
    let output = engine.process(&input)?;

    if args.json {
        let json = serde_json::to_string_pretty(&output).map_err(TranscriptaseError::Json)?;
        println!("{json}");
    } else {
        print_human(&output, &args);
    }

    // Generation mode: observe input, then emit synthetic records
    if let Some(count) = args.generate {
        let count = if count == 0 { 1 } else { count };
        match engine.generate_batch(count) {
            Some(records) => {
                if args.json {
                    let json =
                        serde_json::to_string_pretty(&records).map_err(TranscriptaseError::Json)?;
                    println!("{json}");
                } else {
                    println!("\nGenerated {} synthetic record(s):", records.len());
                    for (i, record) in records.iter().enumerate() {
                        let json = serde_json::to_string_pretty(record)
                            .map_err(TranscriptaseError::Json)?;
                        println!("  [{i}] {json}");
                    }
                }
            }
            None => {
                eprintln!("No schema observed — cannot generate data.");
            }
        }
    }

    if args.stats {
        eprintln!("\n{}", engine.stats());
    }

    Ok(())
}

fn read_input(args: &Args) -> Result<String, TranscriptaseError> {
    match &args.file {
        Some(path) => std::fs::read_to_string(path).map_err(TranscriptaseError::Io),
        None => {
            let mut buf = String::new();
            std::io::stdin()
                .read_to_string(&mut buf)
                .map_err(TranscriptaseError::Io)?;
            Ok(buf)
        }
    }
}

fn print_human(output: &nexcore_transcriptase::TranscriptionOutput, args: &Args) {
    println!(
        "Schema ({} records observed):",
        output.stats.records_observed
    );
    print_schema(&output.schema, 0);

    if args.violations && !output.violations.is_empty() {
        println!("\nViolations ({}):", output.violations.len());
        for v in &output.violations {
            let marker = match v.severity {
                DiagnosticLevel::Critical => "!!",
                DiagnosticLevel::Warning => " !",
                DiagnosticLevel::Info => "  ",
            };
            println!("  {marker} {v}");
        }
    }

    if args.verify && !output.fidelity.is_empty() {
        println!("\nFidelity:");
        for (i, f) in output.fidelity.iter().enumerate() {
            println!("  record {i}: {f}");
        }
    }
}

fn print_schema(schema: &nexcore_transcriptase::Schema, indent: usize) {
    let pad = "  ".repeat(indent);
    let name = schema.name.as_deref().unwrap_or("<root>");

    match &schema.kind {
        nexcore_transcriptase::SchemaKind::Null => {
            println!("{pad}{name}: null");
        }
        nexcore_transcriptase::SchemaKind::Bool {
            true_count,
            false_count,
        } => {
            println!("{pad}{name}: bool (true:{true_count} false:{false_count})");
        }
        nexcore_transcriptase::SchemaKind::Int { min, max, sum } => {
            let avg = if schema.observations > 0 {
                *sum as f64 / schema.observations as f64
            } else {
                0.0
            };
            println!(
                "{pad}{name}: int [{min}..{max}] avg={avg:.1} n={}",
                schema.observations
            );
        }
        nexcore_transcriptase::SchemaKind::Float { min, max, sum } => {
            let avg = if schema.observations > 0 {
                *sum / schema.observations as f64
            } else {
                0.0
            };
            println!(
                "{pad}{name}: float [{min}..{max}] avg={avg:.2} n={}",
                schema.observations
            );
        }
        nexcore_transcriptase::SchemaKind::Str {
            min_len,
            max_len,
            unique_count,
        } => {
            println!("{pad}{name}: string len=[{min_len}..{max_len}] unique~{unique_count}");
        }
        nexcore_transcriptase::SchemaKind::Array {
            element,
            min_len,
            max_len,
        } => {
            println!("{pad}{name}: array len=[{min_len}..{max_len}]");
            print_schema(element, indent + 1);
        }
        nexcore_transcriptase::SchemaKind::Record(fields) => {
            println!("{pad}{name}: record ({} fields)", fields.len());
            for (_, field_schema) in fields {
                print_schema(field_schema, indent + 1);
            }
        }
        nexcore_transcriptase::SchemaKind::Mixed => {
            println!("{pad}{name}: mixed (heterogeneous)");
        }
    }
}
