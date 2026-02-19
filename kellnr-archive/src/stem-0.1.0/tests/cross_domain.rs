//! Cross-domain integration tests proving STEM primitives compose.
//!
//! These tests demonstrate that types from different STEM domains
//! interact correctly through shared stem-core types (Confidence, Tier, Measured).

use stem::prelude::*;

// ============================================================================
// Cross-Domain Type Sharing
// ============================================================================

#[test]
fn confidence_shared_across_all_domains() {
    // One Confidence type, four domains
    let conf = Confidence::new(0.9);

    let chem = MeasuredRatio::new(Ratio::new(2.0), conf);
    let phys = MeasuredQuantity::new(Quantity::new(50.0), conf);
    let math = MeasuredBound::new(Bounded::new(5.0, Some(0.0), Some(10.0)), conf);

    // All reference the same confidence semantics
    assert!((chem.confidence.value() - 0.9).abs() < f64::EPSILON);
    assert!((phys.confidence.value() - 0.9).abs() < f64::EPSILON);
    assert!((math.confidence.value() - 0.9).abs() < f64::EPSILON);
}

#[test]
fn tier_classification_applies_everywhere() {
    // T1 primitive → direct transfer (1.0x)
    assert!((Tier::T1Universal.transfer_multiplier() - 1.0).abs() < f64::EPSILON);

    // T2-P (all STEM traits) → 0.9x transfer
    assert!((Tier::T2Primitive.transfer_multiplier() - 0.9).abs() < f64::EPSILON);

    // T3 domain-specific → 0.4x (lossy)
    assert!((Tier::T3DomainSpecific.transfer_multiplier() - 0.4).abs() < f64::EPSILON);
}

// ============================================================================
// Cross-Domain Analogies (Same Primitive, Different Domain)
// ============================================================================

#[test]
fn bound_analogy_across_domains() {
    // Mathematics: Bounded value
    let math_bound = Bounded::new(5.0, Some(0.0), Some(10.0));
    assert!(math_bound.in_bounds());

    // Chemistry: Fraction is inherently bounded [0, 1]
    let chem_fraction = Fraction::new(0.5);
    assert!(!chem_fraction.is_saturated());

    // Physics: Mass is bounded to non-negative
    let phys_mass = Mass::new(-5.0);
    assert!((phys_mass.value() - 0.0).abs() < f64::EPSILON); // clamped

    // All three express BOUND (T2-P) grounded in T1 STATE
}

#[test]
fn rate_analogy_across_domains() {
    // Chemistry: Reaction rate
    let chem_rate = Rate::new(2.5);
    assert!((chem_rate.value() - 2.5).abs() < f64::EPSILON);

    // Physics: Frequency (rate of oscillation)
    let phys_freq = Frequency::new(2.5);
    assert!((phys_freq.value() - 2.5).abs() < f64::EPSILON);

    // Both express RATE grounded in T1 MAPPING: time → change
}

#[test]
fn conservation_analogy_across_domains() {
    // Physics: Quantity conservation
    let before = Quantity::new(100.0);
    let after = Quantity::new(100.001);
    assert!(before.conserved_with(&after, 0.01));

    // Chemistry: Balance (equilibrium = conservation of reaction rates)
    let balance = Balance::new(Rate::new(1.0), Rate::new(1.0));
    assert!(balance.is_equilibrium(0.01));

    // Mathematics: Relation symmetry (structural conservation)
    assert!(Relation::Equal.is_symmetric());

    // All express PRESERVE grounded in T1 STATE
}

// ============================================================================
// Composition Tests
// ============================================================================

#[test]
fn measured_confidence_combines_multiplicatively() {
    let a = Confidence::new(0.9);
    let b = Confidence::new(0.8);
    let combined = a.combine(b);
    assert!((combined.value() - 0.72).abs() < f64::EPSILON);

    // Apply combined confidence to any domain
    let result = MeasuredForce::new(Force::new(10.0), combined);
    assert!((result.confidence.value() - 0.72).abs() < f64::EPSILON);
}

#[test]
fn correction_works_with_any_domain_type() {
    // Correct a physics quantity
    let correction = Correction::now(
        Quantity::new(100.0),
        Quantity::new(105.0),
        "Calibration drift detected",
    );
    assert!((correction.original().value() - 100.0).abs() < f64::EPSILON);
    assert!((correction.corrected().value() - 105.0).abs() < f64::EPSILON);

    // Correct a chemistry ratio
    let ratio_fix = Correction::now(Ratio::new(2.0), Ratio::new(2.3), "Dilution factor updated");
    assert!((ratio_fix.apply().value() - 2.3).abs() < f64::EPSILON);
}

// ============================================================================
// Negation / Inverse Symmetry
// ============================================================================

#[test]
fn physics_force_negation_mirrors_math_relation_inversion() {
    // Physics: Force negation
    let f = Force::new(10.0);
    let neg_f = -f;
    assert!((neg_f.value() - (-10.0)).abs() < f64::EPSILON);

    // Mathematics: Relation inversion
    let r = Relation::LessThan;
    assert_eq!(r.invert(), Relation::GreaterThan);

    // Both express SYMMETRIC grounded in T1 MAPPING
}

// ============================================================================
// Scale / Transform Analogy
// ============================================================================

#[test]
fn scale_transform_across_domains() {
    // Physics: ScaleFactor
    let scale = ScaleFactor::new(2.0);
    assert!((scale.apply(5.0) - 10.0).abs() < f64::EPSILON);

    // Mathematics: Bounded clamp (constraining transform)
    let bounded = Bounded::new(15, Some(0), Some(10));
    assert_eq!(bounded.clamp(), 10);

    // Chemistry: Ratio (concentration = amount/volume scaling)
    let ratio = Ratio::new(3.0);
    assert!((ratio.value() - 3.0).abs() < f64::EPSILON);
}

// ============================================================================
// F = ma across domains
// ============================================================================

#[test]
fn cause_effect_relationship() {
    // Physics: F = ma → a = F/m
    let force = Force::new(10.0);
    let mass = Mass::new(2.0);
    let accel = Acceleration::from_force_and_mass(force, mass);
    assert!((accel.value() - 5.0).abs() < f64::EPSILON);

    // Chemistry: K = forward/reverse
    let balance = Balance::new(Rate::new(10.0), Rate::new(2.0));
    assert!((balance.constant - 5.0).abs() < f64::EPSILON);

    // Both produce ratio of 5.0 from same input proportions
    // Grounded in T1 MAPPING: cause → proportional effect
}
