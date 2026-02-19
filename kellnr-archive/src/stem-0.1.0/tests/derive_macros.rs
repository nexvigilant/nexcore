//! Derive Macro Integration Tests
//!
//! Tests `StemNewtype` derive macro for T2-P newtype generation.

use stem::StemNewtype;

// ============================================================================
// Test Types
// ============================================================================

/// Simple unclamped newtype
#[derive(Debug, Clone, Copy, PartialEq, StemNewtype)]
struct RawValue(f64);

/// Newtype with minimum clamp
#[derive(Debug, Clone, Copy, PartialEq, StemNewtype)]
#[stem(clamp_min = 0.0)]
struct NonNegative(f64);

/// Newtype with full range clamp [0.0, 1.0]
#[derive(Debug, Clone, Copy, PartialEq, StemNewtype)]
#[stem(clamp_min = 0.0, clamp_max = 1.0)]
struct Probability(f64);

/// Newtype with custom default
#[derive(Debug, Clone, Copy, PartialEq, StemNewtype)]
#[stem(clamp_min = 0.0, clamp_max = 1.0, default_value = 0.5)]
struct Entropy(f64);

/// Newtype with max-only clamp
#[derive(Debug, Clone, Copy, PartialEq, StemNewtype)]
#[stem(clamp_max = 100.0)]
struct Percentage(f64);

// ============================================================================
// Tests
// ============================================================================

#[test]
fn raw_value_no_clamping() {
    let v = RawValue::new(42.0);
    assert!((v.value() - 42.0).abs() < f64::EPSILON);

    let neg = RawValue::new(-100.0);
    assert!((neg.value() - (-100.0)).abs() < f64::EPSILON);
}

#[test]
fn non_negative_clamps_minimum() {
    let v = NonNegative::new(5.0);
    assert!((v.value() - 5.0).abs() < f64::EPSILON);

    let clamped = NonNegative::new(-10.0);
    assert!((clamped.value() - 0.0).abs() < f64::EPSILON);
}

#[test]
fn probability_clamps_range() {
    let normal = Probability::new(0.7);
    assert!((normal.value() - 0.7).abs() < f64::EPSILON);

    let over = Probability::new(1.5);
    assert!((over.value() - 1.0).abs() < f64::EPSILON);

    let under = Probability::new(-0.3);
    assert!((under.value() - 0.0).abs() < f64::EPSILON);
}

#[test]
fn entropy_custom_default() {
    let d = Entropy::default();
    assert!((d.value() - 0.5).abs() < f64::EPSILON);
}

#[test]
fn default_is_zero_when_unspecified() {
    let d = RawValue::default();
    assert!((d.value() - 0.0).abs() < f64::EPSILON);
}

#[test]
fn from_f64_conversion() {
    let p: Probability = 0.8.into();
    assert!((p.value() - 0.8).abs() < f64::EPSILON);

    // Clamping applies through From
    let over: Probability = 2.0.into();
    assert!((over.value() - 1.0).abs() < f64::EPSILON);
}

#[test]
fn into_f64_conversion() {
    let p = Probability::new(0.6);
    let raw: f64 = p.into();
    assert!((raw - 0.6).abs() < f64::EPSILON);
}

#[test]
fn percentage_clamps_max_only() {
    let v = Percentage::new(50.0);
    assert!((v.value() - 50.0).abs() < f64::EPSILON);

    let over = Percentage::new(200.0);
    assert!((over.value() - 100.0).abs() < f64::EPSILON);

    // Negative allowed (no clamp_min)
    let neg = Percentage::new(-50.0);
    assert!((neg.value() - (-50.0)).abs() < f64::EPSILON);
}
