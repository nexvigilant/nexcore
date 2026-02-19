// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! # Theorems of Signal Theory
//!
//! Five formal theorems about signal detection systems.
//!
//! ## Fundamental Theorems
//!
//! | Theorem | Name | Statement |
//! |---------|------|-----------|
//! | T1 | Neyman-Pearson | Likelihood ratio is the optimal detector |
//! | T2 | Parallel Specificity | Parallel composition preserves specificity |
//! | T3 | Sequential FP Reduction | Sequential composition reduces false positives |
//! | T4 | Threshold Monotonicity | Higher threshold → higher specificity, lower sensitivity |
//! | T5 | Causal Accumulation | More evidence lines → stronger causal inference |

use alloc::string::String;
use alloc::vec::Vec;

// ═══════════════════════════════════════════════════════════
// THEOREM TRAIT
// ═══════════════════════════════════════════════════════════

/// A formal theorem about signal detection.
pub trait Theorem {
    /// Theorem identifier.
    fn id() -> &'static str;

    /// Theorem name.
    fn name() -> &'static str;

    /// Informal statement.
    fn statement() -> &'static str;

    /// Prerequisites (other theorems this depends on).
    fn prerequisites() -> Vec<&'static str>;
}

// ═══════════════════════════════════════════════════════════
// T1: NEYMAN-PEARSON OPTIMALITY
// ═══════════════════════════════════════════════════════════

/// **Theorem T1: Neyman-Pearson Optimality**
///
/// *For any given false positive rate, the likelihood ratio test
/// maximizes the true positive rate (is the most powerful test).*
///
/// This is the foundational theorem of signal detection: among all
/// detectors with the same false alarm rate, the likelihood ratio
/// detector has the highest sensitivity.
///
/// ## Implication
///
/// All other detectors are suboptimal in the Neyman-Pearson sense.
/// PRR, ROR, IC, EBGM are all approximations to the likelihood ratio.
pub struct T1NeymanPearson;

impl Theorem for T1NeymanPearson {
    fn id() -> &'static str {
        "T1"
    }
    fn name() -> &'static str {
        "Neyman-Pearson Optimality"
    }
    fn statement() -> &'static str {
        "The likelihood ratio test maximizes sensitivity for any given false positive rate"
    }
    fn prerequisites() -> Vec<&'static str> {
        Vec::new() // Fundamental
    }
}

// ═══════════════════════════════════════════════════════════
// T2: PARALLEL PRESERVES SPECIFICITY
// ═══════════════════════════════════════════════════════════

/// **Theorem T2: Parallel Composition Preserves Specificity**
///
/// *If D₁ and D₂ both require detection (AND), the composite
/// has specificity ≥ max(spec₁, spec₂).*
///
/// Requiring multiple independent detectors to agree reduces
/// false positives (increases specificity).
pub struct T2ParallelSpecificity;

impl Theorem for T2ParallelSpecificity {
    fn id() -> &'static str {
        "T2"
    }
    fn name() -> &'static str {
        "Parallel Preserves Specificity"
    }
    fn statement() -> &'static str {
        "AND-parallel composition has specificity >= max(spec1, spec2)"
    }
    fn prerequisites() -> Vec<&'static str> {
        alloc::vec!["T1"]
    }
}

impl T2ParallelSpecificity {
    /// Verify: combined specificity >= max of individual specificities.
    #[must_use]
    pub fn verify(spec1: f64, spec2: f64, combined_spec: f64) -> bool {
        let max_spec = if spec1 > spec2 { spec1 } else { spec2 };
        combined_spec >= max_spec - f64::EPSILON
    }
}

// ═══════════════════════════════════════════════════════════
// T3: SEQUENTIAL REDUCES FALSE POSITIVES
// ═══════════════════════════════════════════════════════════

/// **Theorem T3: Sequential Composition Reduces False Positives**
///
/// *In a screening→confirmation pipeline, the false positive rate
/// of the composite ≤ false positive rate of screening alone.*
///
/// Adding a confirmation step can only reduce (or maintain)
/// the false positive rate.
pub struct T3SequentialFPReduction;

impl Theorem for T3SequentialFPReduction {
    fn id() -> &'static str {
        "T3"
    }
    fn name() -> &'static str {
        "Sequential Reduces False Positives"
    }
    fn statement() -> &'static str {
        "Screening→confirmation pipeline has FPR <= FPR of screening alone"
    }
    fn prerequisites() -> Vec<&'static str> {
        alloc::vec!["T1"]
    }
}

impl T3SequentialFPReduction {
    /// Verify: combined FPR <= screening FPR.
    #[must_use]
    pub fn verify(screening_fpr: f64, combined_fpr: f64) -> bool {
        combined_fpr <= screening_fpr + f64::EPSILON
    }
}

// ═══════════════════════════════════════════════════════════
// T4: THRESHOLD MONOTONICITY
// ═══════════════════════════════════════════════════════════

/// **Theorem T4: Threshold Monotonicity**
///
/// *As the detection threshold increases:*
/// - *Specificity monotonically increases (fewer false alarms)*
/// - *Sensitivity monotonically decreases (more misses)*
///
/// This formalizes L3 (the sensitivity-specificity tradeoff)
/// as a monotonic function of threshold.
pub struct T4ThresholdMonotonicity;

impl Theorem for T4ThresholdMonotonicity {
    fn id() -> &'static str {
        "T4"
    }
    fn name() -> &'static str {
        "Threshold Monotonicity"
    }
    fn statement() -> &'static str {
        "Higher threshold → higher specificity, lower sensitivity (monotonically)"
    }
    fn prerequisites() -> Vec<&'static str> {
        alloc::vec!["T1"]
    }
}

impl T4ThresholdMonotonicity {
    /// Verify monotonicity: as threshold increases, sensitivity should decrease.
    ///
    /// Takes ordered pairs of (threshold, sensitivity).
    #[must_use]
    pub fn verify_sensitivity(pairs: &[(f64, f64)]) -> bool {
        for window in pairs.windows(2) {
            let (t1, s1) = window[0];
            let (t2, s2) = window[1];
            if t2 > t1 && s2 > s1 + f64::EPSILON {
                return false; // sensitivity increased with higher threshold
            }
        }
        true
    }

    /// Verify monotonicity: as threshold increases, specificity should increase.
    #[must_use]
    pub fn verify_specificity(pairs: &[(f64, f64)]) -> bool {
        for window in pairs.windows(2) {
            let (t1, sp1) = window[0];
            let (t2, sp2) = window[1];
            if t2 > t1 && sp2 < sp1 - f64::EPSILON {
                return false; // specificity decreased with higher threshold
            }
        }
        true
    }
}

// ═══════════════════════════════════════════════════════════
// T5: CAUSAL ACCUMULATION
// ═══════════════════════════════════════════════════════════

/// **Theorem T5: Causal Accumulation**
///
/// *The strength of causal inference monotonically increases
/// with the number of independent evidence lines.*
///
/// More Bradford Hill criteria satisfied → stronger causal claim.
/// This justifies the multi-criteria approach to causality assessment.
pub struct T5CausalAccumulation;

impl Theorem for T5CausalAccumulation {
    fn id() -> &'static str {
        "T5"
    }
    fn name() -> &'static str {
        "Causal Accumulation"
    }
    fn statement() -> &'static str {
        "Causal inference strength increases monotonically with independent evidence lines"
    }
    fn prerequisites() -> Vec<&'static str> {
        Vec::new() // Independent of T1-T4
    }
}

impl T5CausalAccumulation {
    /// Verify that more evidence lines → higher weight.
    ///
    /// Takes pairs of (evidence_count, total_weight).
    #[must_use]
    pub fn verify(pairs: &[(usize, f64)]) -> bool {
        for window in pairs.windows(2) {
            let (c1, w1) = window[0];
            let (c2, w2) = window[1];
            if c2 > c1 && w2 < w1 - f64::EPSILON {
                return false; // weight decreased with more evidence
            }
        }
        true
    }
}

// ═══════════════════════════════════════════════════════════
// THEOREM REGISTRY
// ═══════════════════════════════════════════════════════════

/// Summary of a theorem for registry display.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TheoremSummary {
    /// Theorem ID.
    pub id: String,
    /// Theorem name.
    pub name: String,
    /// Theorem statement.
    pub statement: String,
    /// Prerequisites.
    pub prerequisites: Vec<String>,
}

/// Registry of all signal theory theorems.
#[derive(Debug, Clone)]
pub struct TheoremRegistry {
    /// All theorem summaries.
    pub theorems: Vec<TheoremSummary>,
}

impl TheoremRegistry {
    /// Build the complete theorem registry.
    #[must_use]
    pub fn build() -> Self {
        let theorems = alloc::vec![
            TheoremSummary {
                id: T1NeymanPearson::id().into(),
                name: T1NeymanPearson::name().into(),
                statement: T1NeymanPearson::statement().into(),
                prerequisites: T1NeymanPearson::prerequisites()
                    .iter()
                    .map(|&s| s.into())
                    .collect(),
            },
            TheoremSummary {
                id: T2ParallelSpecificity::id().into(),
                name: T2ParallelSpecificity::name().into(),
                statement: T2ParallelSpecificity::statement().into(),
                prerequisites: T2ParallelSpecificity::prerequisites()
                    .iter()
                    .map(|&s| s.into())
                    .collect(),
            },
            TheoremSummary {
                id: T3SequentialFPReduction::id().into(),
                name: T3SequentialFPReduction::name().into(),
                statement: T3SequentialFPReduction::statement().into(),
                prerequisites: T3SequentialFPReduction::prerequisites()
                    .iter()
                    .map(|&s| s.into())
                    .collect(),
            },
            TheoremSummary {
                id: T4ThresholdMonotonicity::id().into(),
                name: T4ThresholdMonotonicity::name().into(),
                statement: T4ThresholdMonotonicity::statement().into(),
                prerequisites: T4ThresholdMonotonicity::prerequisites()
                    .iter()
                    .map(|&s| s.into())
                    .collect(),
            },
            TheoremSummary {
                id: T5CausalAccumulation::id().into(),
                name: T5CausalAccumulation::name().into(),
                statement: T5CausalAccumulation::statement().into(),
                prerequisites: T5CausalAccumulation::prerequisites()
                    .iter()
                    .map(|&s| s.into())
                    .collect(),
            },
        ];
        Self { theorems }
    }

    /// Number of theorems.
    #[must_use]
    pub fn count(&self) -> usize {
        self.theorems.len()
    }

    /// Get a theorem by ID.
    #[must_use]
    pub fn get(&self, id: &str) -> Option<&TheoremSummary> {
        self.theorems.iter().find(|t| t.id == id)
    }
}

impl Default for TheoremRegistry {
    fn default() -> Self {
        Self::build()
    }
}

// ═══════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theorem_ids() {
        assert_eq!(T1NeymanPearson::id(), "T1");
        assert_eq!(T2ParallelSpecificity::id(), "T2");
        assert_eq!(T3SequentialFPReduction::id(), "T3");
        assert_eq!(T4ThresholdMonotonicity::id(), "T4");
        assert_eq!(T5CausalAccumulation::id(), "T5");
    }

    #[test]
    fn test_t2_parallel_specificity_holds() {
        assert!(T2ParallelSpecificity::verify(0.9, 0.85, 0.95));
    }

    #[test]
    fn test_t2_parallel_specificity_violated() {
        assert!(!T2ParallelSpecificity::verify(0.9, 0.85, 0.80));
    }

    #[test]
    fn test_t3_sequential_fp_holds() {
        assert!(T3SequentialFPReduction::verify(0.10, 0.05));
    }

    #[test]
    fn test_t3_sequential_fp_violated() {
        assert!(!T3SequentialFPReduction::verify(0.05, 0.10));
    }

    #[test]
    fn test_t4_sensitivity_monotonicity() {
        // As threshold increases, sensitivity decreases
        let pairs = [(1.0, 0.95), (2.0, 0.80), (3.0, 0.60), (4.0, 0.40)];
        assert!(T4ThresholdMonotonicity::verify_sensitivity(&pairs));
    }

    #[test]
    fn test_t4_sensitivity_monotonicity_violated() {
        let pairs = [(1.0, 0.80), (2.0, 0.90)]; // sensitivity increased
        assert!(!T4ThresholdMonotonicity::verify_sensitivity(&pairs));
    }

    #[test]
    fn test_t4_specificity_monotonicity() {
        // As threshold increases, specificity increases
        let pairs = [(1.0, 0.70), (2.0, 0.85), (3.0, 0.95)];
        assert!(T4ThresholdMonotonicity::verify_specificity(&pairs));
    }

    #[test]
    fn test_t5_causal_accumulation() {
        let pairs = [(1, 1.5), (2, 2.5), (3, 3.5)];
        assert!(T5CausalAccumulation::verify(&pairs));
    }

    #[test]
    fn test_t5_causal_accumulation_violated() {
        let pairs = [(1, 2.0), (2, 1.0)]; // weight decreased
        assert!(!T5CausalAccumulation::verify(&pairs));
    }

    #[test]
    fn test_theorem_registry() {
        let registry = TheoremRegistry::build();
        assert_eq!(registry.count(), 5);
        assert!(registry.get("T1").is_some());
        assert!(registry.get("T5").is_some());
        assert!(registry.get("T99").is_none());
    }

    #[test]
    fn test_theorem_prerequisites() {
        assert!(T1NeymanPearson::prerequisites().is_empty());
        assert_eq!(T2ParallelSpecificity::prerequisites().len(), 1);
        assert!(T5CausalAccumulation::prerequisites().is_empty());
    }
}
