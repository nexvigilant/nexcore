// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! # Conservation Laws of Signal Detection
//!
//! Four fundamental laws that signal detection systems must preserve.
//!
//! ## The Four Laws
//!
//! | Law | Name | Statement |
//! |-----|------|-----------|
//! | **L1** | Total Count Conservation | The 2x2 matrix is exhaustive |
//! | **L2** | Base Rate Invariance | Prevalence is independent of threshold |
//! | **L3** | Sensitivity-Specificity Tradeoff | Improving one degrades the other |
//! | **L4** | Information Conservation | Detection cannot create signal information |
//!
//! ## Connection to Axioms
//!
//! - L1 → A1 (Data Generation): counts are preserved
//! - L2 → A2 (Noise Dominance): noise ratio is a property of reality
//! - L3 → A4 (Boundary Requirement): moving the boundary trades sens/spec
//! - L4 → A5 (Disproportionality): comparison, not creation

use alloc::string::String;

use crate::decision::DecisionMatrix;

// ═══════════════════════════════════════════════════════════
// LAW VERIFICATION
// ═══════════════════════════════════════════════════════════

/// Result of verifying a conservation law.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LawVerification {
    /// Law is satisfied.
    Satisfied,
    /// Law is violated with explanation.
    Violated(String),
}

impl LawVerification {
    /// Whether the law is satisfied.
    #[must_use]
    pub fn is_satisfied(&self) -> bool {
        matches!(self, Self::Satisfied)
    }

    /// Whether the law is violated.
    #[must_use]
    pub fn is_violated(&self) -> bool {
        matches!(self, Self::Violated(_))
    }
}

/// Trait for conservation laws.
pub trait ConservationLaw {
    /// Law identifier.
    fn id(&self) -> &'static str;

    /// Human-readable name.
    fn name(&self) -> &'static str;

    /// The law statement.
    fn statement(&self) -> &'static str;
}

// ═══════════════════════════════════════════════════════════
// L1: TOTAL COUNT CONSERVATION
// ═══════════════════════════════════════════════════════════

/// **L1: Total Count Conservation**
///
/// *The 2x2 decision matrix is exhaustive: every observation falls
/// into exactly one cell.*
///
/// hits + misses + false_alarms + correct_rejections = total
///
/// ## Primitive Grounding
///
/// T2-P (Σ + N): Sum + Quantity
#[derive(Debug, Clone, Copy, Default)]
pub struct L1TotalCountConservation;

impl ConservationLaw for L1TotalCountConservation {
    fn id(&self) -> &'static str {
        "L1"
    }
    fn name(&self) -> &'static str {
        "Total Count Conservation"
    }
    fn statement(&self) -> &'static str {
        "The 2x2 decision matrix is exhaustive: every observation falls into exactly one cell"
    }
}

impl L1TotalCountConservation {
    /// Verify that the matrix sums to the expected total.
    #[must_use]
    pub fn verify(&self, matrix: &DecisionMatrix, expected_total: u64) -> LawVerification {
        let actual = matrix.total();
        if actual == expected_total {
            LawVerification::Satisfied
        } else {
            LawVerification::Violated(alloc::format!(
                "Expected total {}, got {} (hits={} + misses={} + FA={} + CR={})",
                expected_total,
                actual,
                matrix.hits,
                matrix.misses,
                matrix.false_alarms,
                matrix.correct_rejections,
            ))
        }
    }
}

// ═══════════════════════════════════════════════════════════
// L2: BASE RATE INVARIANCE
// ═══════════════════════════════════════════════════════════

/// **L2: Base Rate Invariance**
///
/// *The true prevalence of signals is a property of reality,
/// not of the detection threshold.*
///
/// signal_present = hits + misses (invariant to threshold choice)
///
/// ## Primitive Grounding
///
/// T2-P (N + ∅): Quantity + Void (noise floor is fixed)
#[derive(Debug, Clone, Copy, Default)]
pub struct L2BaseRateInvariance;

impl ConservationLaw for L2BaseRateInvariance {
    fn id(&self) -> &'static str {
        "L2"
    }
    fn name(&self) -> &'static str {
        "Base Rate Invariance"
    }
    fn statement(&self) -> &'static str {
        "True prevalence is a property of reality, not of the detection threshold"
    }
}

impl L2BaseRateInvariance {
    /// Verify that two matrices (different thresholds) have the same prevalence.
    #[must_use]
    pub fn verify(&self, m1: &DecisionMatrix, m2: &DecisionMatrix) -> LawVerification {
        let sp1 = m1.signal_present();
        let sp2 = m2.signal_present();
        if sp1 == sp2 {
            LawVerification::Satisfied
        } else {
            LawVerification::Violated(alloc::format!(
                "Signal-present count changed: {} vs {} (threshold should not affect prevalence)",
                sp1,
                sp2,
            ))
        }
    }
}

// ═══════════════════════════════════════════════════════════
// L3: SENSITIVITY-SPECIFICITY TRADEOFF
// ═══════════════════════════════════════════════════════════

/// **L3: Sensitivity-Specificity Tradeoff**
///
/// *For any fixed data distribution, improving sensitivity
/// degrades specificity and vice versa.*
///
/// This is the fundamental tradeoff: you cannot simultaneously
/// maximize both without changing the underlying signal.
///
/// ## Primitive Grounding
///
/// T2-P (κ + ∂): Comparison + Boundary
#[derive(Debug, Clone, Copy, Default)]
pub struct L3SensitivitySpecificityTradeoff;

impl ConservationLaw for L3SensitivitySpecificityTradeoff {
    fn id(&self) -> &'static str {
        "L3"
    }
    fn name(&self) -> &'static str {
        "Sensitivity-Specificity Tradeoff"
    }
    fn statement(&self) -> &'static str {
        "For fixed data, improving sensitivity degrades specificity and vice versa"
    }
}

impl L3SensitivitySpecificityTradeoff {
    /// Verify the tradeoff between two operating points.
    ///
    /// If sensitivity increased, specificity should decrease (and vice versa).
    /// Allows for equal values (on the same point).
    #[must_use]
    pub fn verify(&self, m1: &DecisionMatrix, m2: &DecisionMatrix) -> LawVerification {
        let sens1 = m1.sensitivity();
        let sens2 = m2.sensitivity();
        let spec1 = m1.specificity();
        let spec2 = m2.specificity();

        let sens_increased = sens2 > sens1 + f64::EPSILON;
        let spec_increased = spec2 > spec1 + f64::EPSILON;

        if sens_increased && spec_increased {
            LawVerification::Violated(alloc::format!(
                "Both sensitivity ({:.3} → {:.3}) and specificity ({:.3} → {:.3}) increased \
                 — this violates the fundamental tradeoff",
                sens1,
                sens2,
                spec1,
                spec2,
            ))
        } else {
            LawVerification::Satisfied
        }
    }
}

// ═══════════════════════════════════════════════════════════
// L4: INFORMATION CONSERVATION
// ═══════════════════════════════════════════════════════════

/// **L4: Information Conservation**
///
/// *Detection cannot create signal information — it can only
/// identify what is already present in the data.*
///
/// A detector's discriminability (d') is bounded by the actual
/// signal-to-noise ratio in the data.
///
/// ## Primitive Grounding
///
/// T2-P (→ + ∃): Causality + Existence
#[derive(Debug, Clone, Copy, Default)]
pub struct L4InformationConservation;

impl ConservationLaw for L4InformationConservation {
    fn id(&self) -> &'static str {
        "L4"
    }
    fn name(&self) -> &'static str {
        "Information Conservation"
    }
    fn statement(&self) -> &'static str {
        "Detection cannot create signal information; it can only identify what is already present"
    }
}

impl L4InformationConservation {
    /// Verify that discriminability does not exceed theoretical maximum.
    ///
    /// `max_dprime` is the theoretical maximum d' given the SNR.
    #[must_use]
    pub fn verify(&self, observed_dprime: f64, max_dprime: f64) -> LawVerification {
        if observed_dprime <= max_dprime + f64::EPSILON {
            LawVerification::Satisfied
        } else {
            LawVerification::Violated(alloc::format!(
                "Observed d'={:.3} exceeds theoretical maximum d'={:.3} \
                 — detection cannot create information",
                observed_dprime,
                max_dprime,
            ))
        }
    }
}

// ═══════════════════════════════════════════════════════════
// VERIFICATION REPORT
// ═══════════════════════════════════════════════════════════

/// Aggregated conservation law verification results.
#[derive(Debug, Clone)]
pub struct ConservationReport {
    /// Results keyed by law ID.
    pub results: alloc::vec::Vec<(String, LawVerification)>,
}

impl ConservationReport {
    /// Create an empty report.
    #[must_use]
    pub fn new() -> Self {
        Self {
            results: alloc::vec::Vec::new(),
        }
    }

    /// Add a verification result.
    pub fn add(&mut self, law_id: &str, result: LawVerification) {
        self.results.push((law_id.into(), result));
    }

    /// Whether all laws are satisfied.
    #[must_use]
    pub fn all_satisfied(&self) -> bool {
        self.results.iter().all(|(_, r)| r.is_satisfied())
    }

    /// Get violations.
    #[must_use]
    pub fn violations(&self) -> alloc::vec::Vec<(&str, &str)> {
        self.results
            .iter()
            .filter_map(|(id, r)| {
                if let LawVerification::Violated(msg) = r {
                    Some((id.as_str(), msg.as_str()))
                } else {
                    None
                }
            })
            .collect()
    }
}

impl Default for ConservationReport {
    fn default() -> Self {
        Self::new()
    }
}

// ═══════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_l1_satisfied() {
        let m = DecisionMatrix::new(80, 20, 10, 90);
        let result = L1TotalCountConservation.verify(&m, 200);
        assert!(result.is_satisfied());
    }

    #[test]
    fn test_l1_violated() {
        let m = DecisionMatrix::new(80, 20, 10, 90);
        let result = L1TotalCountConservation.verify(&m, 100);
        assert!(result.is_violated());
    }

    #[test]
    fn test_l2_satisfied() {
        // Same data, different thresholds → same signal_present
        let m1 = DecisionMatrix::new(80, 20, 10, 90);
        let m2 = DecisionMatrix::new(90, 10, 20, 80);
        let result = L2BaseRateInvariance.verify(&m1, &m2);
        assert!(result.is_satisfied());
    }

    #[test]
    fn test_l2_violated() {
        let m1 = DecisionMatrix::new(80, 20, 10, 90);
        let m2 = DecisionMatrix::new(70, 10, 10, 110); // signal_present=80 vs 100
        let result = L2BaseRateInvariance.verify(&m1, &m2);
        assert!(result.is_violated());
    }

    #[test]
    fn test_l3_satisfied_tradeoff() {
        // Higher sensitivity, lower specificity → valid tradeoff
        let m1 = DecisionMatrix::new(70, 30, 10, 90);
        let m2 = DecisionMatrix::new(90, 10, 30, 70);
        let result = L3SensitivitySpecificityTradeoff.verify(&m1, &m2);
        assert!(result.is_satisfied());
    }

    #[test]
    fn test_l3_violated_both_improve() {
        // Both sensitivity AND specificity improved → violation
        let m1 = DecisionMatrix::new(70, 30, 30, 70);
        let m2 = DecisionMatrix::new(90, 10, 10, 90);
        let result = L3SensitivitySpecificityTradeoff.verify(&m1, &m2);
        assert!(result.is_violated());
    }

    #[test]
    fn test_l4_satisfied() {
        let result = L4InformationConservation.verify(1.5, 3.0);
        assert!(result.is_satisfied());
    }

    #[test]
    fn test_l4_violated() {
        let result = L4InformationConservation.verify(4.0, 3.0);
        assert!(result.is_violated());
    }

    #[test]
    fn test_conservation_report() {
        let mut report = ConservationReport::new();
        report.add("L1", LawVerification::Satisfied);
        report.add("L3", LawVerification::Violated("test".into()));

        assert!(!report.all_satisfied());
        assert_eq!(report.violations().len(), 1);
    }

    #[test]
    fn test_conservation_all_satisfied() {
        let mut report = ConservationReport::new();
        report.add("L1", LawVerification::Satisfied);
        report.add("L2", LawVerification::Satisfied);
        assert!(report.all_satisfied());
    }
}
