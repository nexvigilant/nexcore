// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! # Signal Theory Axioms
//!
//! The six fundamental axioms governing all signal detection.
//!
//! ## Axiom Hierarchy
//!
//! ```text
//! A1 (Data Generation) ──► A2 (Noise Dominance) ──► A3 (Signal Existence)
//!                                                        │
//!                                                        ▼
//! A6 (Causal Inference) ◄── A5 (Disproportionality) ◄── A4 (Boundary Requirement)
//! ```
//!
//! ## Curry-Howard Correspondence
//!
//! Each axiom has a type-level witness:
//! - Constructing the witness = proving the axiom holds
//! - Failed construction = axiom violation
//!
//! ## Core Thesis
//!
//! **"All detection is boundary drawing."**

use alloc::string::String;
use alloc::vec::Vec;

// ═══════════════════════════════════════════════════════════
// AXIOM TRAIT
// ═══════════════════════════════════════════════════════════

/// Marker trait for signal theory axiom witnesses.
///
/// An axiom is proven by constructing a value of the witness type.
/// This follows the Curry-Howard correspondence: types as propositions,
/// values as proofs.
pub trait Axiom {
    /// The axiom identifier (A1-A6).
    fn id() -> &'static str;

    /// Human-readable axiom name.
    fn name() -> &'static str;

    /// The axiom statement.
    fn statement() -> &'static str;

    /// The dominant T1 primitive for this axiom.
    fn dominant_symbol() -> char;
}

// ═══════════════════════════════════════════════════════════
// A1: DATA GENERATION
// ═══════════════════════════════════════════════════════════

/// **A1: Data Generation**
///
/// *Any observable system generates data at some frequency.*
///
/// The const generic `CAPACITY` encodes the maximum observation count.
/// Construction requires `observation_count >= 1`.
///
/// ## Primitive Grounding
///
/// T2-P (ν + N): Frequency + Quantity
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct A1DataGeneration<const CAPACITY: usize> {
    /// Number of observations collected.
    pub observation_count: usize,
    /// Rate of data generation (observations per unit time).
    pub generation_rate: u64,
}

impl<const CAPACITY: usize> A1DataGeneration<CAPACITY> {
    /// Attempt to construct a data generation witness.
    ///
    /// Returns `None` if `observation_count == 0` or exceeds `CAPACITY`.
    #[must_use]
    pub const fn try_new(observation_count: usize, generation_rate: u64) -> Option<Self> {
        if observation_count == 0 || observation_count > CAPACITY {
            None
        } else {
            Some(Self {
                observation_count,
                generation_rate,
            })
        }
    }

    /// The maximum capacity for observations.
    #[must_use]
    pub const fn capacity() -> usize {
        CAPACITY
    }

    /// Whether we are at capacity.
    #[must_use]
    pub const fn is_at_capacity(&self) -> bool {
        self.observation_count == CAPACITY
    }

    /// Fraction of capacity used.
    #[must_use]
    pub fn utilization(&self) -> f64 {
        if CAPACITY == 0 {
            return 0.0;
        }
        self.observation_count as f64 / CAPACITY as f64
    }
}

impl<const CAPACITY: usize> Axiom for A1DataGeneration<CAPACITY> {
    fn id() -> &'static str {
        "A1"
    }
    fn name() -> &'static str {
        "Data Generation"
    }
    fn statement() -> &'static str {
        "Any observable system generates data at some frequency"
    }
    fn dominant_symbol() -> char {
        'ν'
    }
}

// ═══════════════════════════════════════════════════════════
// A2: NOISE DOMINANCE
// ═══════════════════════════════════════════════════════════

/// **A2: Noise Dominance**
///
/// *In any dataset, noise dominates signal (noise_ratio > 0.5).*
///
/// This is the fundamental pessimistic assumption: most observations
/// are NOT signals. Detection must overcome this noise floor.
///
/// ## Primitive Grounding
///
/// T2-P (∅ + N): Void + Quantity
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct A2NoiseDominance {
    /// Proportion of data that is noise (must be > 0.5).
    pub noise_ratio: f64,
    /// Estimated background noise level.
    pub background_level: f64,
}

impl A2NoiseDominance {
    /// Attempt to construct a noise dominance witness.
    ///
    /// Returns `None` if `noise_ratio <= 0.5` (violates axiom).
    #[must_use]
    pub fn try_new(noise_ratio: f64, background_level: f64) -> Option<Self> {
        if noise_ratio > 0.5 && noise_ratio <= 1.0 && background_level >= 0.0 {
            Some(Self {
                noise_ratio,
                background_level,
            })
        } else {
            None
        }
    }

    /// The signal-to-noise ratio (1 - noise_ratio) / noise_ratio.
    #[must_use]
    pub fn signal_to_noise(&self) -> f64 {
        if self.noise_ratio >= 1.0 {
            return 0.0;
        }
        (1.0 - self.noise_ratio) / self.noise_ratio
    }

    /// Whether noise is overwhelmingly dominant (> 0.95).
    #[must_use]
    pub fn is_overwhelming(&self) -> bool {
        self.noise_ratio > 0.95
    }
}

impl Axiom for A2NoiseDominance {
    fn id() -> &'static str {
        "A2"
    }
    fn name() -> &'static str {
        "Noise Dominance"
    }
    fn statement() -> &'static str {
        "In any dataset, noise dominates signal (noise_ratio > 0.5)"
    }
    fn dominant_symbol() -> char {
        '∅'
    }
}

// ═══════════════════════════════════════════════════════════
// A3: SIGNAL EXISTENCE
// ═══════════════════════════════════════════════════════════

/// **A3: Signal Existence**
///
/// *There exists at least one true signal in the data.*
///
/// Without this axiom, detection is pointless. This is the
/// existential quantifier that justifies the search.
///
/// ## Primitive Grounding
///
/// T2-P (∃ + N): Existence + Quantity
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct A3SignalExistence {
    /// Number of confirmed or suspected true signals.
    pub signal_count: usize,
    /// Confidence that at least one signal is real (0.0-1.0 as fixed-point).
    pub confidence_pct: u32,
}

impl A3SignalExistence {
    /// Attempt to construct a signal existence witness.
    ///
    /// Returns `None` if `signal_count == 0`.
    #[must_use]
    pub const fn try_new(signal_count: usize, confidence_pct: u32) -> Option<Self> {
        if signal_count == 0 {
            None
        } else {
            Some(Self {
                signal_count,
                confidence_pct,
            })
        }
    }

    /// Confidence as a float (0.0-1.0).
    #[must_use]
    pub fn confidence(&self) -> f64 {
        self.confidence_pct as f64 / 100.0
    }

    /// Whether multiple signals exist.
    #[must_use]
    pub const fn is_plural(&self) -> bool {
        self.signal_count > 1
    }
}

impl Axiom for A3SignalExistence {
    fn id() -> &'static str {
        "A3"
    }
    fn name() -> &'static str {
        "Signal Existence"
    }
    fn statement() -> &'static str {
        "There exists at least one true signal in the data"
    }
    fn dominant_symbol() -> char {
        '∃'
    }
}

// ═══════════════════════════════════════════════════════════
// A4: BOUNDARY REQUIREMENT
// ═══════════════════════════════════════════════════════════

/// **A4: Boundary Requirement**
///
/// *Detection requires a finite, positive threshold (boundary).*
///
/// This is the central axiom: **"All detection is boundary drawing."**
/// Without a threshold, there is no decision. The threshold transforms
/// a continuous measure into a binary classification.
///
/// ## Primitive Grounding
///
/// T2-P (∂ + N): Boundary + Quantity
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct A4BoundaryRequirement {
    /// The threshold value (must be finite and positive).
    pub threshold: f64,
    /// Human-readable label for the boundary.
    threshold_label_len: u8,
}

impl A4BoundaryRequirement {
    /// Attempt to construct a boundary requirement witness.
    ///
    /// Returns `None` if threshold is not finite or not positive.
    #[must_use]
    pub fn try_new(threshold: f64) -> Option<Self> {
        if threshold.is_finite() && threshold > 0.0 {
            Some(Self {
                threshold,
                threshold_label_len: 0,
            })
        } else {
            None
        }
    }

    /// Whether a value exceeds this boundary.
    #[must_use]
    pub fn exceeds(&self, value: f64) -> bool {
        value >= self.threshold
    }

    /// Distance from the boundary (positive = above, negative = below).
    #[must_use]
    pub fn distance(&self, value: f64) -> f64 {
        value - self.threshold
    }

    /// Ratio of value to threshold.
    #[must_use]
    pub fn ratio(&self, value: f64) -> f64 {
        value / self.threshold
    }
}

impl Axiom for A4BoundaryRequirement {
    fn id() -> &'static str {
        "A4"
    }
    fn name() -> &'static str {
        "Boundary Requirement"
    }
    fn statement() -> &'static str {
        "Detection requires a finite, positive threshold (boundary)"
    }
    fn dominant_symbol() -> char {
        '∂'
    }
}

// ═══════════════════════════════════════════════════════════
// A5: DISPROPORTIONALITY
// ═══════════════════════════════════════════════════════════

/// **A5: Disproportionality**
///
/// *A signal is detected when observed frequency exceeds expected frequency.*
///
/// This axiom formalizes the comparison that all signal detection methods
/// share: observed vs. expected. PRR, ROR, IC, EBGM all reduce to this.
///
/// ## Primitive Grounding
///
/// T2-C (κ + N + ∂ + ν): Comparison + Quantity + Boundary + Frequency
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct A5Disproportionality {
    /// Observed count or frequency.
    pub observed: f64,
    /// Expected count or frequency (under null hypothesis).
    pub expected: f64,
    /// The ratio observed/expected.
    pub ratio: f64,
}

impl A5Disproportionality {
    /// Attempt to construct a disproportionality witness.
    ///
    /// Returns `None` if expected <= 0 or values aren't finite.
    #[must_use]
    pub fn try_new(observed: f64, expected: f64) -> Option<Self> {
        if expected > 0.0 && observed.is_finite() && expected.is_finite() {
            Some(Self {
                observed,
                expected,
                ratio: observed / expected,
            })
        } else {
            None
        }
    }

    /// Whether the ratio exceeds a given threshold.
    #[must_use]
    pub fn exceeds_threshold(&self, threshold: f64) -> bool {
        self.ratio >= threshold
    }

    /// The excess count (observed - expected).
    #[must_use]
    pub fn excess(&self) -> f64 {
        self.observed - self.expected
    }

    /// Information component: log2(observed/expected).
    #[must_use]
    pub fn information_component(&self) -> f64 {
        if self.ratio <= 0.0 {
            return f64::NEG_INFINITY;
        }
        self.ratio.ln() / core::f64::consts::LN_2
    }
}

impl Axiom for A5Disproportionality {
    fn id() -> &'static str {
        "A5"
    }
    fn name() -> &'static str {
        "Disproportionality"
    }
    fn statement() -> &'static str {
        "A signal is detected when observed frequency exceeds expected frequency"
    }
    fn dominant_symbol() -> char {
        'κ'
    }
}

// ═══════════════════════════════════════════════════════════
// A6: CAUSAL INFERENCE
// ═══════════════════════════════════════════════════════════

/// Evidence type for causal inference.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum EvidenceKind {
    /// Statistical association (correlation).
    Statistical,
    /// Temporal precedence (cause before effect).
    Temporal,
    /// Dose-response relationship.
    DoseResponse,
    /// Biological plausibility.
    Biological,
    /// Challenge/dechallenge/rechallenge.
    Experimental,
}

impl EvidenceKind {
    /// All evidence kinds.
    #[must_use]
    pub const fn all() -> [Self; 5] {
        [
            Self::Statistical,
            Self::Temporal,
            Self::DoseResponse,
            Self::Biological,
            Self::Experimental,
        ]
    }
}

/// **A6: Causal Inference**
///
/// *Detection alone is insufficient; evidence must accumulate toward causality.*
///
/// This axiom captures the Bradford Hill criteria: multiple lines of evidence
/// converge to support a causal relationship, not just correlation.
///
/// ## Primitive Grounding
///
/// T2-C (→ + κ + Σ + π): Causality + Comparison + Sum + Persistence
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct A6CausalInference {
    /// Lines of evidence gathered.
    pub evidence: Vec<(EvidenceKind, f64)>,
    /// Accumulated evidence weight.
    pub total_weight: f64,
    /// Whether causal threshold is met.
    pub causal_threshold_met: bool,
}

impl A6CausalInference {
    /// Create a new causal inference witness.
    #[must_use]
    pub fn new(causal_threshold: f64) -> Self {
        Self {
            evidence: Vec::new(),
            total_weight: 0.0,
            causal_threshold_met: false,
        }
    }

    /// Add a line of evidence.
    pub fn add_evidence(&mut self, kind: EvidenceKind, weight: f64, causal_threshold: f64) {
        self.evidence.push((kind, weight));
        self.total_weight += weight;
        self.causal_threshold_met = self.total_weight >= causal_threshold;
    }

    /// Number of distinct evidence types.
    #[must_use]
    pub fn evidence_breadth(&self) -> usize {
        let mut seen = [false; 5];
        for &(kind, _) in &self.evidence {
            let idx = match kind {
                EvidenceKind::Statistical => 0,
                EvidenceKind::Temporal => 1,
                EvidenceKind::DoseResponse => 2,
                EvidenceKind::Biological => 3,
                EvidenceKind::Experimental => 4,
            };
            seen[idx] = true;
        }
        seen.iter().filter(|&&s| s).count()
    }

    /// Whether a specific evidence type has been gathered.
    #[must_use]
    pub fn has_evidence(&self, kind: EvidenceKind) -> bool {
        self.evidence.iter().any(|&(k, _)| k == kind)
    }
}

impl Axiom for A6CausalInference {
    fn id() -> &'static str {
        "A6"
    }
    fn name() -> &'static str {
        "Causal Inference"
    }
    fn statement() -> &'static str {
        "Detection alone is insufficient; evidence must accumulate toward causality"
    }
    fn dominant_symbol() -> char {
        '→'
    }
}

// ═══════════════════════════════════════════════════════════
// COMPLETE PROOF
// ═══════════════════════════════════════════════════════════

/// A complete proof that a signal detection system satisfies all six axioms.
///
/// Construction of this type witnesses the detection system is well-formed.
///
/// ## Tier Classification
///
/// T3 (ν + ∅ + ∃ + ∂ + κ + →): Full 6-primitive composition
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SignalTheoryProof<const CAPACITY: usize> {
    /// A1: Data generation witness.
    pub a1: A1DataGeneration<CAPACITY>,
    /// A2: Noise dominance witness.
    pub a2: A2NoiseDominance,
    /// A3: Signal existence witness.
    pub a3: A3SignalExistence,
    /// A4: Boundary requirement witness.
    pub a4: A4BoundaryRequirement,
    /// A5: Disproportionality witness.
    pub a5: A5Disproportionality,
    /// A6: Causal inference witness.
    pub a6: A6CausalInference,
}

impl<const CAPACITY: usize> SignalTheoryProof<CAPACITY> {
    /// Construct a complete signal theory proof.
    #[must_use]
    pub fn new(
        a1: A1DataGeneration<CAPACITY>,
        a2: A2NoiseDominance,
        a3: A3SignalExistence,
        a4: A4BoundaryRequirement,
        a5: A5Disproportionality,
        a6: A6CausalInference,
    ) -> Self {
        Self {
            a1,
            a2,
            a3,
            a4,
            a5,
            a6,
        }
    }

    /// Whether all axioms are satisfied and causal threshold met.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.a6.causal_threshold_met && self.a5.ratio > 1.0
    }

    /// Summary string.
    #[must_use]
    pub fn summary(&self) -> String {
        alloc::format!(
            "SignalTheoryProof<{}>: {} obs, noise={:.1}%, {} signals, threshold={:.2}, ratio={:.2}, causal={}",
            CAPACITY,
            self.a1.observation_count,
            self.a2.noise_ratio * 100.0,
            self.a3.signal_count,
            self.a4.threshold,
            self.a5.ratio,
            self.a6.causal_threshold_met,
        )
    }
}

// ═══════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_a1_data_generation() {
        let a1: Option<A1DataGeneration<1000>> = A1DataGeneration::try_new(500, 10);
        assert!(a1.is_some());
        let a1 = a1.unwrap_or_else(|| A1DataGeneration {
            observation_count: 0,
            generation_rate: 0,
        });
        assert_eq!(a1.observation_count, 500);
        assert!(!a1.is_at_capacity());
        assert!((a1.utilization() - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_a1_zero_observations_fails() {
        let a1: Option<A1DataGeneration<100>> = A1DataGeneration::try_new(0, 10);
        assert!(a1.is_none());
    }

    #[test]
    fn test_a1_exceeds_capacity_fails() {
        let a1: Option<A1DataGeneration<10>> = A1DataGeneration::try_new(20, 5);
        assert!(a1.is_none());
    }

    #[test]
    fn test_a2_noise_dominance() {
        let a2 = A2NoiseDominance::try_new(0.8, 1.0);
        assert!(a2.is_some());
        let a2 = a2.unwrap_or_else(|| A2NoiseDominance {
            noise_ratio: 0.51,
            background_level: 0.0,
        });
        assert!(!a2.is_overwhelming());
        assert!((a2.signal_to_noise() - 0.25).abs() < 1e-10);
    }

    #[test]
    fn test_a2_noise_below_half_fails() {
        let a2 = A2NoiseDominance::try_new(0.3, 1.0);
        assert!(a2.is_none());
    }

    #[test]
    fn test_a3_signal_existence() {
        let a3 = A3SignalExistence::try_new(3, 95);
        assert!(a3.is_some());
        let a3 = a3.unwrap_or_else(|| A3SignalExistence {
            signal_count: 1,
            confidence_pct: 0,
        });
        assert!(a3.is_plural());
        assert!((a3.confidence() - 0.95).abs() < f64::EPSILON);
    }

    #[test]
    fn test_a3_zero_signals_fails() {
        let a3 = A3SignalExistence::try_new(0, 50);
        assert!(a3.is_none());
    }

    #[test]
    fn test_a4_boundary_requirement() {
        let a4 = A4BoundaryRequirement::try_new(2.0);
        assert!(a4.is_some());
        let a4 = a4.unwrap_or_else(|| A4BoundaryRequirement {
            threshold: 1.0,
            threshold_label_len: 0,
        });
        assert!(a4.exceeds(3.0));
        assert!(!a4.exceeds(1.0));
        assert!((a4.distance(3.0) - 1.0).abs() < f64::EPSILON);
        assert!((a4.ratio(4.0) - 2.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_a4_zero_threshold_fails() {
        assert!(A4BoundaryRequirement::try_new(0.0).is_none());
        assert!(A4BoundaryRequirement::try_new(-1.0).is_none());
        assert!(A4BoundaryRequirement::try_new(f64::INFINITY).is_none());
    }

    #[test]
    fn test_a5_disproportionality() {
        let a5 = A5Disproportionality::try_new(15.0, 5.0);
        assert!(a5.is_some());
        let a5 = a5.unwrap_or_else(|| A5Disproportionality {
            observed: 1.0,
            expected: 1.0,
            ratio: 1.0,
        });
        assert!((a5.ratio - 3.0).abs() < f64::EPSILON);
        assert!((a5.excess() - 10.0).abs() < f64::EPSILON);
        assert!(a5.exceeds_threshold(2.0));
    }

    #[test]
    fn test_a5_zero_expected_fails() {
        assert!(A5Disproportionality::try_new(10.0, 0.0).is_none());
        assert!(A5Disproportionality::try_new(10.0, -1.0).is_none());
    }

    #[test]
    fn test_a6_causal_inference() {
        let mut a6 = A6CausalInference::new(3.0);
        assert!(!a6.causal_threshold_met);
        assert_eq!(a6.evidence_breadth(), 0);

        a6.add_evidence(EvidenceKind::Statistical, 1.5, 3.0);
        a6.add_evidence(EvidenceKind::Temporal, 1.0, 3.0);
        assert_eq!(a6.evidence_breadth(), 2);
        assert!(!a6.causal_threshold_met);

        a6.add_evidence(EvidenceKind::Biological, 1.0, 3.0);
        assert!(a6.causal_threshold_met);
        assert_eq!(a6.evidence_breadth(), 3);
    }

    #[test]
    fn test_evidence_kind_all() {
        assert_eq!(EvidenceKind::all().len(), 5);
    }

    #[test]
    fn test_signal_theory_proof() {
        let a1 =
            A1DataGeneration::<10000>::try_new(5000, 100).unwrap_or_else(|| A1DataGeneration {
                observation_count: 1,
                generation_rate: 1,
            });
        let a2 = A2NoiseDominance::try_new(0.95, 5.0).unwrap_or_else(|| A2NoiseDominance {
            noise_ratio: 0.51,
            background_level: 0.0,
        });
        let a3 = A3SignalExistence::try_new(3, 90).unwrap_or_else(|| A3SignalExistence {
            signal_count: 1,
            confidence_pct: 50,
        });
        let a4 = A4BoundaryRequirement::try_new(2.0).unwrap_or_else(|| A4BoundaryRequirement {
            threshold: 1.0,
            threshold_label_len: 0,
        });
        let a5 = A5Disproportionality::try_new(15.0, 5.0).unwrap_or_else(|| A5Disproportionality {
            observed: 1.0,
            expected: 1.0,
            ratio: 1.0,
        });
        let mut a6 = A6CausalInference::new(2.5);
        a6.add_evidence(EvidenceKind::Statistical, 1.5, 2.5);
        a6.add_evidence(EvidenceKind::Temporal, 1.0, 2.5);
        a6.add_evidence(EvidenceKind::Biological, 0.5, 2.5);

        let proof = SignalTheoryProof::new(a1, a2, a3, a4, a5, a6);
        assert!(proof.is_valid());
        assert!(proof.summary().contains("5000 obs"));
    }

    #[test]
    fn test_axiom_ids() {
        assert_eq!(A1DataGeneration::<10>::id(), "A1");
        assert_eq!(A2NoiseDominance::id(), "A2");
        assert_eq!(A3SignalExistence::id(), "A3");
        assert_eq!(A4BoundaryRequirement::id(), "A4");
        assert_eq!(A5Disproportionality::id(), "A5");
        assert_eq!(A6CausalInference::id(), "A6");
    }

    #[test]
    fn test_axiom_dominant_symbols() {
        assert_eq!(A1DataGeneration::<10>::dominant_symbol(), 'ν');
        assert_eq!(A2NoiseDominance::dominant_symbol(), '∅');
        assert_eq!(A3SignalExistence::dominant_symbol(), '∃');
        assert_eq!(A4BoundaryRequirement::dominant_symbol(), '∂');
        assert_eq!(A5Disproportionality::dominant_symbol(), 'κ');
        assert_eq!(A6CausalInference::dominant_symbol(), '→');
    }
}
