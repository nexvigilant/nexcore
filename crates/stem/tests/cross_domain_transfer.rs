//! Cross-Domain Transfer Examples
//!
//! Demonstrates the STEM thesis: T2-P primitives transfer across domains
//! with predictable confidence based on tier classification.
//!
//! ## The Thesis
//!
//! ```text
//! Transfer Confidence = Tier.transfer_multiplier()
//!   T1 Universal  → 1.0  (direct)
//!   T2-P Primitive → 0.9  (find instantiation)
//!   T2-C Composite → 0.7  (map components)
//!   T3 Domain      → 0.4  (find equivalent, lossy)
//! ```

use stem::prelude::*;

// ============================================================================
// Example 1: THRESHOLD transfers across PV, Economics, Software
// ============================================================================
// T2-P: Bounded (BOUND trait) — grounded in T1 STATE

/// PV: Signal threshold (PRR ≥ 2.0, Evans criteria)
fn pv_signal_threshold(prr: f64) -> bool {
    let threshold = Bounded::new(prr, Some(2.0), None);
    threshold.in_bounds()
}

/// Economics: Price ceiling/floor
fn econ_price_bounds(price: f64, floor: f64, ceiling: f64) -> f64 {
    let bounded = Bounded::new(price, Some(floor), Some(ceiling));
    bounded.clamp()
}

/// Software: Rate limit (requests per second)
fn software_rate_limit(rps: f64, max_rps: f64) -> bool {
    let bounded = Bounded::new(rps, Some(0.0), Some(max_rps));
    bounded.in_bounds()
}

#[test]
fn threshold_transfers_across_three_domains() {
    // Same Bounded<f64> type, three domain instantiations
    assert!(pv_signal_threshold(3.5)); // PRR = 3.5 → signal
    assert!(!pv_signal_threshold(1.2)); // PRR = 1.2 → no signal

    assert!((econ_price_bounds(150.0, 100.0, 200.0) - 150.0).abs() < f64::EPSILON);
    assert!((econ_price_bounds(50.0, 100.0, 200.0) - 100.0).abs() < f64::EPSILON); // clamped to floor

    assert!(software_rate_limit(500.0, 1000.0));
    assert!(!software_rate_limit(1500.0, 1000.0));

    // Transfer confidence: 0.9 (T2-P)
    assert!((Tier::T2Primitive.transfer_multiplier() - 0.9).abs() < f64::EPSILON);
}

// ============================================================================
// Example 2: EQUILIBRIUM transfers across Chemistry, Economics, Software
// ============================================================================
// T2-P: Balance (Harmonize trait) — grounded in T1 STATE

/// Chemistry: Reaction equilibrium K = forward/reverse
fn chem_equilibrium(forward_rate: f64, reverse_rate: f64) -> Balance {
    Balance::new(Rate::new(forward_rate), Rate::new(reverse_rate))
}

/// Economics: Supply-demand equilibrium
fn econ_supply_demand(supply_rate: f64, demand_rate: f64) -> Balance {
    Balance::new(Rate::new(supply_rate), Rate::new(demand_rate))
}

/// Software: Load balancer equilibrium
fn software_load_balance(incoming_rps: f64, processing_rps: f64) -> Balance {
    Balance::new(Rate::new(incoming_rps), Rate::new(processing_rps))
}

#[test]
fn equilibrium_transfers_across_three_domains() {
    let chem = chem_equilibrium(2.0, 1.0);
    assert!(chem.products_favored()); // K > 1

    let econ = econ_supply_demand(100.0, 100.0);
    assert!(econ.is_equilibrium(0.01)); // balanced

    let sw = software_load_balance(500.0, 1000.0);
    assert!(!sw.products_favored()); // K < 1 → processing exceeds incoming (good)

    // All three use the same Balance type from stem-chem
}

// ============================================================================
// Example 3: CONSERVATION transfers across Physics, Accounting, Data
// ============================================================================
// T2-P: Quantity (Preserve trait) — grounded in T1 STATE

/// Physics: Energy conservation
fn physics_energy_conserved(before: f64, after: f64, tolerance: f64) -> bool {
    let q_before = Quantity::new(before);
    let q_after = Quantity::new(after);
    q_before.conserved_with(&q_after, tolerance)
}

/// Accounting: Budget conservation (debits = credits)
fn accounting_balanced(debits: f64, credits: f64, tolerance: f64) -> bool {
    let q_debits = Quantity::new(debits);
    let q_credits = Quantity::new(credits);
    q_debits.conserved_with(&q_credits, tolerance)
}

/// Data integrity: Row count conservation through ETL
fn etl_rows_conserved(input_rows: f64, output_rows: f64, tolerance: f64) -> bool {
    let q_in = Quantity::new(input_rows);
    let q_out = Quantity::new(output_rows);
    q_in.conserved_with(&q_out, tolerance)
}

#[test]
fn conservation_transfers_across_three_domains() {
    assert!(physics_energy_conserved(100.0, 100.001, 0.01));
    assert!(!physics_energy_conserved(100.0, 105.0, 0.01));

    assert!(accounting_balanced(50000.0, 50000.0, 0.01));
    assert!(!accounting_balanced(50000.0, 49000.0, 0.01));

    assert!(etl_rows_conserved(1000.0, 1000.0, 0.01));
    assert!(!etl_rows_conserved(1000.0, 999.0, 0.01));
}

// ============================================================================
// Example 4: PROOF transfers across Math, PV, Legal
// ============================================================================
// T2-P: Proof<P,C> (Prove trait) — grounded in T1 SEQUENCE

#[test]
fn proof_structure_transfers() {
    // Mathematics: Logical proof
    let math_proof: Proof<&str, &str> = Proof::valid(
        vec!["All men are mortal", "Socrates is a man"],
        "Socrates is mortal",
    );
    assert!(math_proof.valid);

    // PV: Causality assessment (premises → conclusion)
    let pv_proof: Proof<&str, &str> = Proof::valid(
        vec![
            "Temporal relationship established",
            "Dechallenge positive",
            "No alternative explanation",
        ],
        "Probable causal relationship",
    );
    assert!(pv_proof.valid);

    // Legal: Evidence chain
    let legal_proof: Proof<&str, &str> = Proof::invalid(
        vec!["Circumstantial evidence only"],
        "Guilty beyond reasonable doubt",
    );
    assert!(!legal_proof.valid);
}

// ============================================================================
// Example 5: Measured<T> wraps any domain value with Confidence
// ============================================================================

#[test]
fn measured_wraps_any_domain() {
    // Science domain
    let science_result = Measured::certain(42.0);
    assert!((science_result.confidence.value() - 1.0).abs() < f64::EPSILON);

    // Chemistry domain
    let chem_result = Measured::uncertain(Ratio::new(2.5), Confidence::default());
    assert!((chem_result.confidence.value() - 0.5).abs() < f64::EPSILON);

    // Map preserves confidence across transformation
    let doubled = Measured::new(5.0, Confidence::new(0.8)).map(|x| x * 2.0);
    assert_eq!(doubled.value, 10.0);
    assert!((doubled.confidence.value() - 0.8).abs() < f64::EPSILON);
}
