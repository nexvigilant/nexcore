//! Proof of Meaning — runnable demonstration program.
//!
//! Run with: cargo run -p nexcore-proof-of-meaning --example pom
//!
//! Demonstrates all 5 chemistry techniques + the Rosetta Proof.

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let command = args.get(1).map(|s| s.as_str()).unwrap_or("demo");

    match command {
        "distill" => {
            let expr =
                args_or_default(&args, 2, "serious unexpected cardiac adverse drug reaction");
            run_distillation(&expr);
        }
        "chromatograph" => {
            let expr = args_or_default(&args, 2, "serious cardiac adverse event");
            run_chromatography(&expr);
        }
        "titrate" => {
            let expr = args_or_default(&args, 2, "cardiac adverse event");
            run_titration(&expr);
        }
        "prove" => {
            let a = args_or_default(&args, 2, "adverse event following immunization");
            let b = args_or_default(&args, 3, "adverse reaction to vaccine");
            run_equivalence_proof(&a, &b);
        }
        "registry" => {
            run_registry_stats();
        }
        "rosetta" => {
            run_rosetta_proof();
        }
        "pipeline" => {
            let expr = args_or_default(&args, 2, "serious unexpected cardiac adverse reaction");
            run_full_pipeline(&expr);
        }
        "batch" => {
            run_batch_equivalence();
        }
        "demo" | _ => {
            run_full_demo();
        }
    }
}

fn args_or_default(args: &[String], idx: usize, default: &str) -> String {
    args.get(idx)
        .cloned()
        .unwrap_or_else(|| default.to_string())
}

// =============================================================================
// PROGRAMS
// =============================================================================

/// Full demonstration: all techniques in sequence.
fn run_full_demo() {
    println!("==========================================================");
    println!("  PROOF OF MEANING — Full Demonstration");
    println!("  Chemistry-Inspired Semantic Equivalence Engine");
    println!("==========================================================\n");

    println!("1. REGISTRY — The Periodic Table of Regulatory Semantics\n");
    run_registry_stats();

    println!("\n2. DISTILLATION — Decompose by Semantic Volatility\n");
    run_distillation("serious unexpected cardiac adverse drug reaction");

    println!("\n3. CHROMATOGRAPHY — Bind to Hierarchy Positions\n");
    run_chromatography("serious cardiac adverse event");

    println!("\n4. TITRATION — Measure Against Canonical Standards\n");
    run_titration("cardiac adverse event");

    println!("\n5. FULL PIPELINE — 3-Step Proof Trail\n");
    run_full_pipeline("serious unexpected cardiac adverse reaction");

    println!("\n6. ROSETTA PROOF — Cross-Jurisdiction Equivalence\n");
    run_rosetta_proof();

    println!("\n7. BATCH EQUIVALENCE — Regulatory Terminology Matrix\n");
    run_batch_equivalence();

    println!("\n==========================================================");
    println!("  All programs completed successfully.");
    println!("==========================================================");
}

/// Program 1: Distillation — separate by volatility.
fn run_distillation(expression: &str) {
    use nexcore_proof_of_meaning::distillation::Distiller;

    let distiller = Distiller::new();
    let result = distiller.distill(expression);

    println!("  INPUT:  \"{}\"", result.input_expression);
    println!("  METHOD: Fractional Distillation (separation by context-dependence)");
    println!();

    println!("  Fractions (most volatile first → most stable last):");
    println!(
        "  {:<4} {:<20} {:<12} {:<12} {:<8}",
        "#", "Atom", "Volatility", "Sep. Temp.", "Purity"
    );
    println!("  {}", "-".repeat(56));

    for f in &result.fractions {
        println!(
            "  {:<4} {:<20} {:<12.2} {:<12.2} {:<8.2}",
            f.fraction_number,
            f.atom.label,
            f.atom.volatility.into_inner(),
            f.separation_temperature.into_inner(),
            f.purity.into_inner(),
        );
    }

    println!();
    println!("  Mass Balance:");
    println!("    Input mass:     {:.1}", result.mass_balance.input_mass);
    println!(
        "    Recovered mass: {:.1}",
        result.mass_balance.recovered_mass
    );
    println!(
        "    Loss:           {:.1}%",
        result.mass_balance.loss_percent
    );
    println!("    Verdict:        {:?}", result.mass_balance.verdict);

    if !result.residue.is_empty() {
        println!();
        println!("  Residue (unseparable):");
        for r in &result.residue {
            println!("    \"{}\" — {:?}", r.fragment, r.reason);
        }
    }
}

/// Program 2: Chromatography — bind to hierarchy positions.
fn run_chromatography(expression: &str) {
    use nexcore_proof_of_meaning::chromatography::Column;

    let column = Column::pv_standard();
    let result = column.separate(expression);

    println!("  INPUT:  \"{}\"", result.input_expression);
    println!("  METHOD: Column Chromatography (binding by differential affinity)");
    println!();

    println!("  Bands (hierarchy bindings):");
    println!(
        "  {:<20} {:<20} {:<12} {:<10}",
        "Atom", "Bound Class", "Affinity", "Bandwidth"
    );
    println!("  {}", "-".repeat(62));

    for b in &result.bands {
        println!(
            "  {:<20} {:<20} {:<12.2} {:<10.2}",
            b.atom_label,
            format!("{:?}", b.bound_class),
            b.binding_affinity.into_inner(),
            b.bandwidth.into_inner(),
        );
        if !b.alternative_sites.is_empty() {
            for (class, aff) in &b.alternative_sites {
                println!("    alt: {:?} (affinity: {:.2})", class, aff.into_inner());
            }
        }
    }

    println!();
    println!("  Resolution:");
    for r in &result.resolution_scores {
        let status = if r.resolution.into_inner() >= 1.5 {
            "BASELINE"
        } else if r.resolution.into_inner() >= 1.0 {
            "partial"
        } else {
            "CO-ELUTING"
        };
        println!(
            "    {} vs {} — R={:.2} [{}]",
            r.band_a,
            r.band_b,
            r.resolution.into_inner(),
            status
        );
    }

    println!();
    println!("  Quality: {:?}", result.quality);
}

/// Program 3: Titration — measure against canonical standards.
fn run_titration(expression: &str) {
    use nexcore_proof_of_meaning::element::{Atom, ElementClass};
    use nexcore_proof_of_meaning::titration::Titrator;

    let pipeline = nexcore_proof_of_meaning::pipeline::ProofPipeline::pv_standard();
    // Access titrator through pipeline transform
    let titrants = vec![
        Atom::new("cardiac", ElementClass::OrganSystem, 0.15),
        Atom::new("hepatic", ElementClass::OrganSystem, 0.15),
        Atom::new("adverse", ElementClass::Modifier, 0.35),
        Atom::new("event", ElementClass::ObservationType, 0.25),
        Atom::new("reaction", ElementClass::ObservationType, 0.25),
        Atom::new("drug", ElementClass::Modifier, 0.05),
        Atom::new("serious", ElementClass::Severity, 0.55),
    ];
    let titrator = Titrator::new(titrants);
    let curve = titrator.titrate(expression);

    let _ = pipeline; // suppress unused warning

    println!("  INPUT:  \"{}\"", curve.analyte);
    println!("  METHOD: Acid-Base Titration (canonical standard neutralization)");
    println!();

    if !curve.points.is_empty() {
        println!("  Titration Curve:");
        println!(
            "  {:<4} {:<20} {:<10} {:<12} {:<8}",
            "#", "Titrant", "Volume", "Residual", "Delta"
        );
        println!("  {}", "-".repeat(54));

        for (i, p) in curve.points.iter().enumerate() {
            let marker = if p.delta.into_inner() > 0.1 {
                " <-- EP"
            } else {
                ""
            };
            println!(
                "  {:<4} {:<20} {:<10.2} {:<12.2} {:<8.2}{}",
                i + 1,
                p.titrant_label,
                p.volume_added.into_inner(),
                p.residual_meaning.into_inner(),
                p.delta.into_inner(),
                marker,
            );
        }
    }

    println!();
    println!("  Equivalence Points: {}", curve.equivalence_points.len());
    for ep in &curve.equivalence_points {
        println!(
            "    {} — coverage: {:.2}, sharpness: {:.2}",
            ep.matched_label,
            ep.coverage.into_inner(),
            ep.sharpness.into_inner(),
        );
    }

    println!();
    println!(
        "  Final Residual: {:.1}% unmatched meaning",
        curve.residual * 100.0
    );
}

/// Program 4: Full 3-step pipeline with proof trail.
fn run_full_pipeline(expression: &str) {
    use nexcore_proof_of_meaning::pipeline::ProofPipeline;

    let pipeline = ProofPipeline::pv_standard();
    let trail = pipeline.transform(expression);

    println!("  INPUT:  \"{}\"", trail.input_expression);
    println!("  METHOD: Full 3-Step Proof Pipeline");
    println!("  ID:     {}", trail.id);
    println!();

    for step in &trail.steps {
        let status = match &step.verification {
            nexcore_proof_of_meaning::pipeline::StepVerification::Verified { confidence } => {
                format!("VERIFIED (conf: {:.2})", confidence)
            }
            nexcore_proof_of_meaning::pipeline::StepVerification::VerifiedWithWarnings {
                warnings,
            } => format!("WARN ({})", warnings.join("; ")),
            nexcore_proof_of_meaning::pipeline::StepVerification::Failed { reason } => {
                format!("FAILED: {}", reason)
            }
        };

        println!("  Step {}: {:?}", step.step_number, step.method);
        println!("    In:  {}", step.input_description);
        println!("    Out: {}", step.output_description);
        println!("    =>   {}", status);
        println!();
    }

    println!("  Trail Valid: {}", trail.trail_valid);
    if !trail.warnings.is_empty() {
        println!("  Warnings:");
        for w in &trail.warnings {
            println!("    - {}", w);
        }
    }
}

/// Program 5: Equivalence proof between two expressions.
fn run_equivalence_proof(expression_a: &str, expression_b: &str) {
    use nexcore_proof_of_meaning::pipeline::ProofPipeline;

    let pipeline = ProofPipeline::pv_standard();
    let proof = pipeline.prove_equivalence(expression_a, expression_b);

    println!("  EXPRESSION A: \"{}\"", expression_a);
    println!("  EXPRESSION B: \"{}\"", expression_b);
    println!("  METHOD:       Comparative Titration + Dual Trail");
    println!("  PROOF ID:     {}", proof.id);
    println!();

    println!(
        "  Trail A: {} steps, valid={}",
        proof.trail_a.steps.len(),
        proof.trail_a.trail_valid
    );
    println!(
        "  Trail B: {} steps, valid={}",
        proof.trail_b.steps.len(),
        proof.trail_b.trail_valid
    );
    println!();

    println!("  Equivalence Analysis:");
    println!("    Shared atoms:  {}", proof.equivalence.shared_atoms);
    println!("    Unique to A:   {}", proof.equivalence.unique_to_a);
    println!("    Unique to B:   {}", proof.equivalence.unique_to_b);
    println!(
        "    Score:         {:.2}",
        proof.equivalence.equivalence_score.into_inner()
    );
    println!("    Verdict:       {:?}", proof.equivalence.verdict);
    println!();

    println!("  PROOF VALID: {}", proof.proof_valid);
}

/// Program 6: The Rosetta Proof — resolving cross-jurisdictional terminology.
fn run_rosetta_proof() {
    use nexcore_proof_of_meaning::pipeline::ProofPipeline;

    let pipeline = ProofPipeline::pv_standard();

    let cases = vec![
        // FDA vs EMA terminology
        (
            "adverse event following immunization",
            "adverse reaction to vaccine",
        ),
        // ICH vs regional variations
        ("serious adverse event", "serious adverse reaction"),
        // Different granularity
        ("cardiac adverse event", "heart-related adverse experience"),
        // Same concept, different modifiers
        (
            "unexpected adverse drug reaction",
            "unlisted adverse reaction",
        ),
    ];

    println!("  THE ROSETTA PROOF");
    println!("  Cross-jurisdictional regulatory terminology equivalence");
    println!("  {}", "=".repeat(60));

    for (i, (a, b)) in cases.iter().enumerate() {
        let proof = pipeline.prove_equivalence(a, b);
        println!();
        println!("  Case {}: ", i + 1);
        println!("    A: \"{}\"", a);
        println!("    B: \"{}\"", b);
        println!(
            "    Shared: {}  Unique(A): {}  Unique(B): {}",
            proof.equivalence.shared_atoms,
            proof.equivalence.unique_to_a,
            proof.equivalence.unique_to_b,
        );
        println!(
            "    Score: {:.2}  Verdict: {:?}",
            proof.equivalence.equivalence_score.into_inner(),
            proof.equivalence.verdict,
        );
        println!("    Proof valid: {}", proof.proof_valid);
    }
}

/// Program 7: Batch equivalence — all-pairs matrix.
fn run_batch_equivalence() {
    use nexcore_proof_of_meaning::pipeline::ProofPipeline;

    let pipeline = ProofPipeline::pv_standard();

    let expressions = vec![
        "cardiac adverse event",
        "cardiac adverse reaction",
        "cardiac adverse experience",
        "hepatic adverse event",
        "serious cardiac event",
    ];

    println!("  BATCH EQUIVALENCE MATRIX");
    println!("  {}", "-".repeat(60));

    // Header
    print!("  {:<30}", "");
    for (i, _) in expressions.iter().enumerate() {
        print!("{:<8}", format!("[{}]", i + 1));
    }
    println!();

    for (i, a) in expressions.iter().enumerate() {
        // Truncate label for display
        let label = if a.len() > 28 { &a[..28] } else { a };
        print!("  [{}] {:<27}", i + 1, label);

        for (j, b) in expressions.iter().enumerate() {
            if i == j {
                print!("{:<8}", "---");
            } else if j > i {
                let proof = pipeline.prove_equivalence(a, b);
                let s = proof.equivalence.equivalence_score.into_inner();
                let marker = match proof.equivalence.verdict {
                    nexcore_proof_of_meaning::titration::EquivalenceVerdict::Equivalent => "=",
                    nexcore_proof_of_meaning::titration::EquivalenceVerdict::PartialOverlap => "~",
                    nexcore_proof_of_meaning::titration::EquivalenceVerdict::Distinct => "X",
                };
                print!("{:<8}", format!("{:.2}{}", s, marker));
            } else {
                print!("{:<8}", "");
            }
        }
        println!();
    }

    println!();
    println!("  Legend: = Equivalent  ~ Partial Overlap  X Distinct");
}

/// Program 8: Registry statistics.
fn run_registry_stats() {
    use nexcore_proof_of_meaning::element::ElementClass;
    use nexcore_proof_of_meaning::registry::AtomRegistry;

    let mut registry = AtomRegistry::new();
    registry.seed_all();

    println!("  THE PERIODIC TABLE OF REGULATORY SEMANTICS");
    println!("  {}", "-".repeat(50));
    println!(
        "  {:<25} {:<10} {:<15}",
        "Element Class", "Count", "Dominant T1"
    );
    println!("  {}", "-".repeat(50));

    for class in ElementClass::all() {
        let count = registry.count_by_class(class);
        let prim = class.dominant_primitive();
        println!(
            "  {:<25} {:<10} {:<15}",
            format!("{:?}", class),
            count,
            format!("{:?}", prim),
        );
    }

    println!("  {}", "-".repeat(50));
    println!("  {:<25} {:<10}", "TOTAL", registry.count());
}
