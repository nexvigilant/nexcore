//! Word Spectroscopy — Virtual Laboratory Program
//!
//! Runs concept analysis through the nexcore-laboratory pipeline:
//! 1. Decompose — resolve words to T1 primitives (μ Mapping)
//! 2. Weigh — compute molecular weight in daltons (Σ Sum + N Quantity)
//! 3. Classify — determine tier and transfer confidence (κ Comparison)
//! 4. Analyze — spectral analysis of constituent masses (∂ Boundary)
//! 5. Report — structured result with interpretation (σ Sequence)
//!
//! ## Tier: T2-C (σ + μ + κ + ∂ + Σ)

#![forbid(unsafe_code)]
#![cfg_attr(
    not(test),
    deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)
)]

use clap::Parser;
use nexcore_laboratory::{
    ExperimentResult, ReactionResult, Specimen, react, run_batch, run_experiment,
};
use nexcore_lex_primitiva::primitiva::LexPrimitiva;

/// Word Spectroscopy — Virtual Laboratory for Concept Analysis
#[derive(Parser, Debug)]
#[command(name = "word-spectroscopy")]
#[command(version = "1.1.0")]
#[command(about = "Analyze concepts through primitive decomposition and molecular weight")]
struct Args {
    /// Output as JSON instead of human-readable format
    #[arg(long, global = true)]
    json: bool,

    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(clap::Subcommand, Debug)]
enum Command {
    /// Run spectroscopy on specimens (decompose, weigh, classify, analyze)
    #[command(visible_alias = "spec")]
    Analyze {
        /// Custom word name to analyze
        name: Option<String>,
        /// Primitives composing the word (names, symbols, or short forms)
        primitives: Vec<String>,
    },

    /// Run batch statistical analysis across all specimens
    #[command(visible_alias = "stats")]
    Batch,

    /// Run chemical reactions between specimen pairs
    #[command(visible_alias = "rxn")]
    React,

    /// Show periodic table of all 16 T1 primitives
    #[command(visible_alias = "periodic")]
    Table,

    /// Display reference guide for all 16 Lex Primitiva
    #[command(visible_alias = "prims")]
    Primitives,
}

// ============================================================================
// Specimen Library — domain concepts grounded to T1 primitives
// ============================================================================

/// Build the standard specimen library of 10 domain concepts.
///
/// Tier: T2-P (μ Mapping + σ Sequence)
fn specimen_library() -> Vec<Specimen> {
    vec![
        // Core PV concepts
        Specimen::new(
            "Vigilance",
            vec![
                LexPrimitiva::State,
                LexPrimitiva::Boundary,
                LexPrimitiva::Frequency,
                LexPrimitiva::Causality,
            ],
        ),
        Specimen::new(
            "Signal",
            vec![LexPrimitiva::Boundary, LexPrimitiva::Quantity],
        ),
        Specimen::new(
            "Guardian",
            vec![
                LexPrimitiva::State,
                LexPrimitiva::Boundary,
                LexPrimitiva::Comparison,
            ],
        ),
        Specimen::new(
            "Cascade",
            vec![
                LexPrimitiva::Sequence,
                LexPrimitiva::Causality,
                LexPrimitiva::Irreversibility,
            ],
        ),
        Specimen::new(
            "Polypharmacy",
            vec![
                LexPrimitiva::Product,
                LexPrimitiva::Boundary,
                LexPrimitiva::Comparison,
                LexPrimitiva::Quantity,
            ],
        ),
        // Extended concepts
        Specimen::new(
            "Homeostasis",
            vec![
                LexPrimitiva::State,
                LexPrimitiva::Boundary,
                LexPrimitiva::Frequency,
                LexPrimitiva::Comparison,
            ],
        ),
        Specimen::new(
            "Causality",
            vec![LexPrimitiva::Causality, LexPrimitiva::Sequence],
        ),
        Specimen::new(
            "Persistence",
            vec![
                LexPrimitiva::Persistence,
                LexPrimitiva::State,
                LexPrimitiva::Existence,
            ],
        ),
        Specimen::new(
            "Recursion",
            vec![
                LexPrimitiva::Recursion,
                LexPrimitiva::Sequence,
                LexPrimitiva::State,
            ],
        ),
        Specimen::new(
            "Measurement",
            vec![
                LexPrimitiva::Quantity,
                LexPrimitiva::Comparison,
                LexPrimitiva::Boundary,
            ],
        ),
    ]
}

// ============================================================================
// Experiment 1: Individual Word Spectroscopy
// ============================================================================

fn run_spectroscopy(specimens: &[Specimen], json: bool) {
    let results: Vec<ExperimentResult> = specimens.iter().map(run_experiment).collect();

    if json {
        if let Ok(j) = serde_json::to_string_pretty(&results) {
            println!("{j}");
        }
        return;
    }

    println!("\n{:=<70}", "");
    println!(" EXPERIMENT 1: WORD SPECTROSCOPY");
    println!(" Pipeline: Decompose -> Weigh -> Classify -> Analyze -> Report");
    println!("{:=<70}\n", "");

    for r in &results {
        println!(
            "  {:<15} [{:<6}] = {:>6.2} Da | {:<8} | tier: {:<4} | transfer: {:>3.0}%",
            r.name,
            r.formula,
            r.molecular_weight,
            r.transfer_class,
            r.tier_prediction,
            r.hybrid_transfer * 100.0,
        );
        for line in &r.spectrum {
            println!(
                "    {} {:<16} = {:>5.2} bits (freq={:>4}, p={:.3})",
                line.symbol, line.primitive, line.mass_bits, line.frequency, line.probability,
            );
        }
        println!();
    }
}

// ============================================================================
// Experiment 2: Batch Statistical Analysis
// ============================================================================

fn run_batch_analysis(specimens: &[Specimen], json: bool) {
    let batch = run_batch(specimens);

    if json {
        if let Ok(j) = serde_json::to_string_pretty(&batch) {
            println!("{j}");
        }
        return;
    }

    println!("{:=<70}", "");
    println!(" EXPERIMENT 2: BATCH STATISTICAL ANALYSIS");
    println!("{:=<70}\n", "");

    println!("  Specimens:   {}", batch.experiments.len());
    println!("  Lightest:    {}", batch.lightest);
    println!("  Heaviest:    {}", batch.heaviest);
    println!(
        "  Average MW:  {:.2} Da (sigma={:.2})",
        batch.average_weight, batch.weight_std_dev
    );
    println!(
        "  Distribution: {} light, {} medium, {} heavy",
        batch.class_distribution.light,
        batch.class_distribution.medium,
        batch.class_distribution.heavy,
    );

    // Weight ranking
    println!("\n  Weight Ranking:");
    let mut ranked: Vec<&ExperimentResult> = batch.experiments.iter().collect();
    ranked.sort_by(|a, b| {
        a.molecular_weight
            .partial_cmp(&b.molecular_weight)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    for (i, r) in ranked.iter().enumerate() {
        let bar_len = (r.molecular_weight * 2.0) as usize;
        let bar: String = (0..bar_len).map(|_| '#').collect();
        println!(
            "    {:>2}. {:<15} {:>6.2} Da  {}",
            i + 1,
            r.name,
            r.molecular_weight,
            bar,
        );
    }
    println!();
}

// ============================================================================
// Experiment 3: Chemical Reactions
// ============================================================================

fn run_reactions(specimens: &[Specimen], json: bool) {
    // React interesting pairs
    let pairs: Vec<(usize, usize)> = vec![
        (0, 1), // Vigilance + Signal
        (2, 1), // Guardian + Signal
        (0, 2), // Vigilance + Guardian
        (3, 4), // Cascade + Polypharmacy
        (5, 2), // Homeostasis + Guardian
    ];

    let reactions: Vec<ReactionResult> = pairs
        .iter()
        .filter(|(a, b)| *a < specimens.len() && *b < specimens.len())
        .map(|(a, b)| react(&specimens[*a], &specimens[*b]))
        .collect();

    if json {
        if let Ok(j) = serde_json::to_string_pretty(&reactions) {
            println!("{j}");
        }
        return;
    }

    println!("{:=<70}", "");
    println!(" EXPERIMENT 3: CHEMICAL REACTIONS");
    println!(" Shared primitives = catalysts, unique = reactants");
    println!("{:=<70}\n", "");

    for rxn in &reactions {
        println!("  {}", rxn.equation);
        println!(
            "    Catalyst:  {:?}   Unique A: {:?}   Unique B: {:?}",
            rxn.catalyst, rxn.unique_a, rxn.unique_b,
        );
        println!(
            "    DeltaH = {:>+6.2} Da ({})   Jaccard = {:.2}",
            rxn.enthalpy,
            if rxn.exothermic {
                "exothermic"
            } else {
                "endothermic"
            },
            rxn.jaccard_similarity,
        );
        println!("    {}", rxn.interpretation);
        println!();
    }
}

// ============================================================================
// Experiment 4: Periodic Table of Primitives
// ============================================================================

fn run_periodic_table(json: bool) {
    let all_prims = LexPrimitiva::all();

    if json {
        let table: Vec<serde_json::Value> = all_prims
            .iter()
            .map(|p| {
                let s = Specimen::new(p.name(), vec![*p]);
                let r = run_experiment(&s);
                serde_json::json!({
                    "symbol": p.symbol(),
                    "name": p.name(),
                    "mass_bits": r.molecular_weight,
                    "transfer_class": r.transfer_class,
                    "transfer_confidence": r.transfer_confidence,
                })
            })
            .collect();
        if let Ok(j) = serde_json::to_string_pretty(&table) {
            println!("{j}");
        }
        return;
    }

    println!("{:=<70}", "");
    println!(" EXPERIMENT 4: PERIODIC TABLE OF T1 PRIMITIVES");
    println!(" Each primitive as a single-atom specimen");
    println!("{:=<70}\n", "");

    println!(
        "  {:<3} {:<18} {:>8} {:>10} {:>10}",
        "Sym", "Name", "Mass(Da)", "Class", "Transfer%"
    );
    println!(
        "  {:-<3} {:-<18} {:->8} {:->10} {:->10}",
        "", "", "", "", ""
    );

    for p in &all_prims {
        let s = Specimen::new(p.name(), vec![*p]);
        let r = run_experiment(&s);
        println!(
            "  {:<3} {:<18} {:>8.3} {:>10} {:>9.0}%",
            p.symbol(),
            p.name(),
            r.molecular_weight,
            r.transfer_class,
            r.transfer_confidence * 100.0,
        );
    }
    println!();
}

// ============================================================================
// Primitive Parser (accepts names, symbols, and short forms)
// ============================================================================

fn parse_single_primitive(input: &str) -> Option<LexPrimitiva> {
    let input = input.trim();
    // Try exact symbol or name match first
    for p in LexPrimitiva::all() {
        if p.symbol() == input || p.name().eq_ignore_ascii_case(input) {
            return Some(p);
        }
    }
    // Try short forms
    let lower = input.to_ascii_lowercase();
    match lower.as_str() {
        "cause" | "causal" => Some(LexPrimitiva::Causality),
        "qty" | "num" | "number" => Some(LexPrimitiva::Quantity),
        "exist" | "exists" => Some(LexPrimitiva::Existence),
        "cmp" | "compare" => Some(LexPrimitiva::Comparison),
        "st" | "ctx" => Some(LexPrimitiva::State),
        "map" | "transform" => Some(LexPrimitiva::Mapping),
        "seq" | "order" => Some(LexPrimitiva::Sequence),
        "rec" | "recurse" | "self-ref" => Some(LexPrimitiva::Recursion),
        "nil" | "none" | "absent" => Some(LexPrimitiva::Void),
        "bound" | "limit" | "edge" => Some(LexPrimitiva::Boundary),
        "freq" | "rate" => Some(LexPrimitiva::Frequency),
        "loc" | "pos" | "position" => Some(LexPrimitiva::Location),
        "persist" | "store" | "save" => Some(LexPrimitiva::Persistence),
        "irrev" | "oneway" | "one-way" => Some(LexPrimitiva::Irreversibility),
        "sum" | "either" | "variant" => Some(LexPrimitiva::Sum),
        "prod" | "tuple" | "struct" => Some(LexPrimitiva::Product),
        _ => None,
    }
}

// ============================================================================
// Experiment 5: Primitives Reference Guide
// ============================================================================

fn run_primitives_reference(json: bool) {
    let all_prims = LexPrimitiva::all();

    if json {
        let table: Vec<serde_json::Value> = all_prims
            .iter()
            .map(|p| {
                serde_json::json!({
                    "symbol": p.symbol(),
                    "name": p.name(),
                    "description": p.description(),
                })
            })
            .collect();
        if let Ok(j) = serde_json::to_string_pretty(&table) {
            println!("{j}");
        }
        return;
    }

    println!("{:=<70}", "");
    println!(" LEX PRIMITIVA — 16 T1 IRREDUCIBLE PRIMITIVES");
    println!("{:=<70}\n", "");

    println!("  {:<4} {:<18} Description", "Sym", "Name");
    println!("  {:-<4} {:-<18} {:-<44}", "", "", "");

    for p in &all_prims {
        println!("  {:<4} {:<18} {}", p.symbol(), p.name(), p.description(),);
    }
    println!();
}

// ============================================================================
// Main
// ============================================================================

fn main() {
    let args = Args::parse();
    let specimens = specimen_library();

    match args.command {
        Some(Command::Analyze { name, primitives }) => {
            if let Some(word_name) = name {
                // Parse primitives from positional args
                let mut prims = Vec::new();
                for p_str in &primitives {
                    match parse_single_primitive(p_str) {
                        Some(p) => prims.push(p),
                        None => {
                            eprintln!(
                                "Unknown primitive: '{p_str}'. Use 'word-spectroscopy primitives' for reference."
                            );
                            return;
                        }
                    }
                }
                if prims.is_empty() {
                    eprintln!(
                        "Provide at least one primitive. Example: word-spectroscopy analyze MyWord state boundary causality"
                    );
                    return;
                }
                let custom = vec![Specimen::new(&word_name, prims)];
                run_spectroscopy(&custom, args.json);
            } else {
                run_spectroscopy(&specimens, args.json);
            }
        }
        Some(Command::Batch) => run_batch_analysis(&specimens, args.json),
        Some(Command::React) => run_reactions(&specimens, args.json),
        Some(Command::Table) => run_periodic_table(args.json),
        Some(Command::Primitives) => run_primitives_reference(args.json),
        None => {
            eprintln!(
                "Hint: use subcommands (analyze, batch, react, table, primitives) or --help\n"
            );
            run_spectroscopy(&specimens, args.json);
            run_batch_analysis(&specimens, args.json);
            run_reactions(&specimens, args.json);
            run_periodic_table(args.json);
        }
    }
}
