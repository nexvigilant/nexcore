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
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use clap::Parser;
use nexcore_laboratory::{
    ExperimentResult, ReactionResult, Specimen, react, run_batch, run_experiment,
};
use nexcore_lex_primitiva::primitiva::LexPrimitiva;

/// Word Spectroscopy — Virtual Laboratory for Concept Analysis
#[derive(Parser, Debug)]
#[command(name = "word-spectroscopy")]
#[command(version = "0.1.0")]
#[command(about = "Analyze concepts through primitive decomposition and molecular weight")]
struct Args {
    /// Run only a specific experiment (1=spectroscopy, 2=batch, 3=reactions, 4=periodic-table)
    #[arg(short, long)]
    experiment: Option<u8>,

    /// Output as JSON instead of human-readable format
    #[arg(long)]
    json: bool,

    /// Custom word to analyze (comma-separated primitives after colon)
    /// e.g. "MyWord:state,boundary,causality"
    #[arg(short, long)]
    word: Vec<String>,
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
// Custom Word Parser
// ============================================================================

fn parse_custom_word(spec: &str) -> Option<Specimen> {
    let parts: Vec<&str> = spec.splitn(2, ':').collect();
    if parts.len() != 2 {
        eprintln!("Invalid format: use 'Name:prim1,prim2,...'");
        return None;
    }
    let name = parts[0];
    let prim_names: Vec<&str> = parts[1].split(',').collect();
    let mut prims = Vec::new();
    for pn in &prim_names {
        let pn = pn.trim();
        let mut found = false;
        for p in LexPrimitiva::all() {
            if p.symbol() == pn || p.name().eq_ignore_ascii_case(pn) {
                prims.push(p);
                found = true;
                break;
            }
        }
        if !found {
            eprintln!("Unknown primitive: '{pn}'");
            return None;
        }
    }
    if prims.is_empty() {
        return None;
    }
    Some(Specimen::new(name, prims))
}

// ============================================================================
// Main
// ============================================================================

fn main() {
    let args = Args::parse();

    // Build specimen library
    let mut specimens = specimen_library();

    // Add custom words if provided
    for w in &args.word {
        if let Some(s) = parse_custom_word(w) {
            specimens.push(s);
        }
    }

    match args.experiment {
        Some(1) => run_spectroscopy(&specimens, args.json),
        Some(2) => run_batch_analysis(&specimens, args.json),
        Some(3) => run_reactions(&specimens, args.json),
        Some(4) => run_periodic_table(args.json),
        Some(_) => eprintln!("Unknown experiment number. Use 1-4."),
        None => {
            // Run all experiments
            run_spectroscopy(&specimens, args.json);
            run_batch_analysis(&specimens, args.json);
            run_reactions(&specimens, args.json);
            run_periodic_table(args.json);
        }
    }
}
