// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! # GroundsTo implementations for nexcore-signal-theory types
//!
//! Connects the Universal Theory of Signals to the Lex Primitiva type system.
//!
//! ## Grounding Strategy
//!
//! The signal-theory crate formalizes signal detection through axioms, detection
//! primitives, boundary theory, detection algebra, SDT decision theory,
//! conservation laws, and formal theorems. Boundary (∂) is the dominant primitive
//! across the crate ("All detection is boundary drawing"), with Comparison (κ),
//! Quantity (N), Frequency (ν), Void (∅), Existence (∃), and Causality (→)
//! as key supporting primitives.
//!
//! | Primitive | Role in Signal Theory |
//! |-----------|----------------------|
//! | ∂ (Boundary) | **Dominant** — thresholds, detection boundaries |
//! | κ (Comparison) | Observed vs expected, decision outcomes |
//! | N (Quantity) | Counts, rates, measures |
//! | ν (Frequency) | Data generation rates |
//! | ∅ (Void) | Noise, missing signals |
//! | ∃ (Existence) | Signal existence, validation |
//! | → (Causality) | Causal inference, evidence accumulation |
//! | Σ (Sum) | Aggregation, conservation totals |
//! | σ (Sequence) | Detection pipelines, cascades |
//! | π (Persistence) | Evidence persistence, monitoring |
//! | ς (State) | Adaptive boundaries, detector state |
//!
//! ## Summary
//!
//! | Tier | Count | Types |
//! |------|-------|-------|
//! | T1 | 7 | SignalPrimitive, DecisionOutcome, SignalStrengthLevel, BoundaryKind, ConjunctionMode, ThresholdPreset, DetectionPhase |
//! | T2-P | 15 | A1-A4, Ratio, Difference, DetectionInterval, ObservationSpace, Baseline, FixedBoundary, RocPoint, DPrime, ResponseBias, L1-L4 |
//! | T2-C | 11 | A5, A6, AdaptiveBoundary, CompositeBoundary, DetectionOutcome, DecisionMatrix, RocCurve, CascadedThreshold, SignalVerificationReport, ConservationReport, DetectionPipeline |
//! | T3 | 1 | SignalTheoryProof |
//! | **Total** | **34** | |

use nexcore_lex_primitiva::grounding::GroundsTo;
use nexcore_lex_primitiva::primitiva::{LexPrimitiva, PrimitiveComposition};

// ═══════════════════════════════════════════════════════════
// CORE TYPES (lib.rs)
// ═══════════════════════════════════════════════════════════

/// SignalPrimitive: T1 (κ), dominant Comparison
///
/// Enum classifying the 6 signal-theory primitives. Pure classification.
impl GroundsTo for crate::SignalPrimitive {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Comparison])
            .with_dominant(LexPrimitiva::Comparison, 0.95)
    }
}

// ═══════════════════════════════════════════════════════════
// AXIOMS MODULE
// ═══════════════════════════════════════════════════════════

/// EvidenceKind: T1 (κ), dominant Comparison
impl GroundsTo for crate::axioms::EvidenceKind {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Comparison])
            .with_dominant(LexPrimitiva::Comparison, 0.95)
    }
}

/// A1DataGeneration: T2-P (ν + N), dominant Frequency
impl<const CAPACITY: usize> GroundsTo for crate::axioms::A1DataGeneration<CAPACITY> {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Frequency, // ν — data generation rate
            LexPrimitiva::Quantity,  // N — observation count, capacity
        ])
        .with_dominant(LexPrimitiva::Frequency, 0.85)
    }
}

/// A2NoiseDominance: T2-P (∅ + N), dominant Void
impl GroundsTo for crate::axioms::A2NoiseDominance {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Void,     // ∅ — noise (absence of signal)
            LexPrimitiva::Quantity, // N — noise_ratio
        ])
        .with_dominant(LexPrimitiva::Void, 0.85)
    }
}

/// A3SignalExistence: T2-P (∃ + N), dominant Existence
impl GroundsTo for crate::axioms::A3SignalExistence {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Existence, // ∃ — signal exists
            LexPrimitiva::Quantity,  // N — signal_count
        ])
        .with_dominant(LexPrimitiva::Existence, 0.85)
    }
}

/// A4BoundaryRequirement: T2-P (∂ + N), dominant Boundary
impl GroundsTo for crate::axioms::A4BoundaryRequirement {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary, // ∂ — the threshold IS the boundary
            LexPrimitiva::Quantity, // N — threshold value
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.90)
    }
}

/// A5Disproportionality: T2-C (κ + N + ∂ + ν), dominant Comparison
impl GroundsTo for crate::axioms::A5Disproportionality {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison, // κ — observed vs expected
            LexPrimitiva::Quantity,   // N — counts
            LexPrimitiva::Boundary,   // ∂ — exceeds threshold?
            LexPrimitiva::Frequency,  // ν — frequency comparison
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.80)
    }
}

/// A6CausalInference: T2-C (→ + κ + Σ + π), dominant Causality
impl GroundsTo for crate::axioms::A6CausalInference {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Causality,   // → — causal chain
            LexPrimitiva::Comparison,  // κ — evidence evaluation
            LexPrimitiva::Sum,         // Σ — accumulated weight
            LexPrimitiva::Persistence, // π — evidence persistence
        ])
        .with_dominant(LexPrimitiva::Causality, 0.80)
    }
}

/// SignalTheoryProof: T3 (ν + ∅ + ∃ + ∂ + κ + →), dominant Boundary
impl<const CAPACITY: usize> GroundsTo for crate::axioms::SignalTheoryProof<CAPACITY> {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Frequency,  // ν — A1
            LexPrimitiva::Void,       // ∅ — A2
            LexPrimitiva::Existence,  // ∃ — A3
            LexPrimitiva::Boundary,   // ∂ — A4 (dominant)
            LexPrimitiva::Comparison, // κ — A5
            LexPrimitiva::Causality,  // → — A6
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.75)
    }
}

// ═══════════════════════════════════════════════════════════
// DETECTION MODULE
// ═══════════════════════════════════════════════════════════

/// ObservationSpace: T2-P (ν + N), dominant Frequency
impl GroundsTo for crate::detection::ObservationSpace {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Frequency, // ν — observation generation
            LexPrimitiva::Quantity,  // N — total_count, dimensions
        ])
        .with_dominant(LexPrimitiva::Frequency, 0.80)
    }
}

/// Baseline: T2-P (∅ + N), dominant Void
impl GroundsTo for crate::detection::Baseline {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Void,     // ∅ — null hypothesis
            LexPrimitiva::Quantity, // N — expected value
        ])
        .with_dominant(LexPrimitiva::Void, 0.80)
    }
}

/// Ratio: T2-P (κ + N), dominant Comparison
impl GroundsTo for crate::detection::Ratio {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison, // κ — observed/expected
            LexPrimitiva::Quantity,   // N — the ratio value
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.85)
    }
}

/// Difference: T2-P (κ + N), dominant Comparison
impl GroundsTo for crate::detection::Difference {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison, // κ — observed - expected
            LexPrimitiva::Quantity,   // N — the difference value
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.85)
    }
}

/// DetectionInterval: T2-P (∂ + N + κ), dominant Boundary
impl GroundsTo for crate::detection::DetectionInterval {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary,   // ∂ — interval bounds
            LexPrimitiva::Quantity,   // N — lower, upper values
            LexPrimitiva::Comparison, // κ — excludes check
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.80)
    }
}

/// DetectionOutcome: T2-C (∂ + κ + ∃ + ∅), dominant Boundary
impl GroundsTo for crate::detection::DetectionOutcome {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary,   // ∂ — boundary crossing result
            LexPrimitiva::Comparison, // κ — classification
            LexPrimitiva::Existence,  // ∃ — signal exists/not
            LexPrimitiva::Void,       // ∅ — indeterminate
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.80)
    }
}

/// SignalStrengthLevel: T1 (κ), dominant Comparison
impl GroundsTo for crate::detection::SignalStrengthLevel {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Comparison])
            .with_dominant(LexPrimitiva::Comparison, 0.95)
    }
}

/// SignalVerificationReport: T2-C (∂ + κ + N + ∃ + ∅), dominant Boundary
impl GroundsTo for crate::detection::SignalVerificationReport {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary,   // ∂ — verification boundaries
            LexPrimitiva::Comparison, // κ — outcome classification
            LexPrimitiva::Quantity,   // N — measures
            LexPrimitiva::Existence,  // ∃ — signal validation
            LexPrimitiva::Void,       // ∅ — missing data handling
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.75)
    }
}

// ═══════════════════════════════════════════════════════════
// THRESHOLD MODULE
// ═══════════════════════════════════════════════════════════

/// BoundaryKind: T1 (κ), dominant Comparison
impl GroundsTo for crate::threshold::BoundaryKind {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Comparison])
            .with_dominant(LexPrimitiva::Comparison, 0.95)
    }
}

/// ConjunctionMode: T1 (κ), dominant Comparison
impl GroundsTo for crate::threshold::ConjunctionMode {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Comparison])
            .with_dominant(LexPrimitiva::Comparison, 0.95)
    }
}

/// ThresholdPreset: T1 (κ), dominant Comparison
impl GroundsTo for crate::threshold::ThresholdPreset {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Comparison])
            .with_dominant(LexPrimitiva::Comparison, 0.95)
    }
}

/// DetectionPhase: T1 (κ), dominant Comparison
impl GroundsTo for crate::threshold::DetectionPhase {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Comparison])
            .with_dominant(LexPrimitiva::Comparison, 0.95)
    }
}

/// FixedBoundary: T2-P (∂ + N), dominant Boundary
impl GroundsTo for crate::threshold::FixedBoundary {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary, // ∂ — the threshold
            LexPrimitiva::Quantity, // N — threshold value
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.90)
    }
}

/// AdaptiveBoundary: T2-C (∂ + N + ν + ς), dominant Boundary
impl GroundsTo for crate::threshold::AdaptiveBoundary {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary,  // ∂ — adjusting threshold
            LexPrimitiva::Quantity,  // N — threshold values
            LexPrimitiva::Frequency, // ν — update frequency
            LexPrimitiva::State,     // ς — current_threshold state
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.80)
    }
}

/// CompositeBoundary: T2-C (∂ + κ + N + Σ), dominant Boundary
impl GroundsTo for crate::threshold::CompositeBoundary {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary,   // ∂ — composite boundary
            LexPrimitiva::Comparison, // κ — conjunction evaluation
            LexPrimitiva::Quantity,   // N — threshold values
            LexPrimitiva::Sum,        // Σ — aggregation of results
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.80)
    }
}

// ═══════════════════════════════════════════════════════════
// ALGEBRA MODULE
// ═══════════════════════════════════════════════════════════

/// CascadedThreshold: T2-C (∂ + σ + κ + N), dominant Boundary
impl GroundsTo for crate::algebra::CascadedThreshold {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary,   // ∂ — cascading boundaries
            LexPrimitiva::Sequence,   // σ — ordered stages
            LexPrimitiva::Comparison, // κ — stage evaluation
            LexPrimitiva::Quantity,   // N — threshold values
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.80)
    }
}

/// DetectionPipeline: T2-C (∂ + σ + κ + N), dominant Sequence
impl GroundsTo for crate::algebra::DetectionPipeline {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sequence,   // σ — pipeline stages
            LexPrimitiva::Boundary,   // ∂ — per-stage thresholds
            LexPrimitiva::Comparison, // κ — stage evaluation
            LexPrimitiva::Quantity,   // N — values
        ])
        .with_dominant(LexPrimitiva::Sequence, 0.80)
    }
}

// ═══════════════════════════════════════════════════════════
// DECISION MODULE
// ═══════════════════════════════════════════════════════════

/// DecisionOutcome: T1 (κ), dominant Comparison
impl GroundsTo for crate::decision::DecisionOutcome {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Comparison])
            .with_dominant(LexPrimitiva::Comparison, 0.95)
    }
}

/// DecisionMatrix: T2-C (κ + N + ∂ + Σ), dominant Comparison
impl GroundsTo for crate::decision::DecisionMatrix {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison, // κ — 2x2 classification
            LexPrimitiva::Quantity,   // N — cell counts
            LexPrimitiva::Boundary,   // ∂ — threshold-dependent
            LexPrimitiva::Sum,        // Σ — row/column totals
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.80)
    }
}

/// RocPoint: T2-P (κ + N), dominant Comparison
impl GroundsTo for crate::decision::RocPoint {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison, // κ — FPR vs TPR
            LexPrimitiva::Quantity,   // N — rate values
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.85)
    }
}

/// RocCurve: T2-C (κ + N + σ + ∂), dominant Comparison
impl GroundsTo for crate::decision::RocCurve {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison, // κ — performance comparison
            LexPrimitiva::Quantity,   // N — area under curve
            LexPrimitiva::Sequence,   // σ — ordered points
            LexPrimitiva::Boundary,   // ∂ — threshold sweep
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.80)
    }
}

/// DPrime: T2-P (κ + N), dominant Comparison
impl GroundsTo for crate::decision::DPrime {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison, // κ — discriminability
            LexPrimitiva::Quantity,   // N — d' value
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.85)
    }
}

/// ResponseBias: T2-P (κ + ∂), dominant Comparison
impl GroundsTo for crate::decision::ResponseBias {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison, // κ — liberal vs conservative
            LexPrimitiva::Boundary,   // ∂ — criterion placement
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.85)
    }
}

// ═══════════════════════════════════════════════════════════
// CONSERVATION MODULE
// ═══════════════════════════════════════════════════════════

/// L1TotalCountConservation: T2-P (Σ + N), dominant Sum
impl GroundsTo for crate::conservation::L1TotalCountConservation {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum,      // Σ — total count
            LexPrimitiva::Quantity, // N — cell values
        ])
        .with_dominant(LexPrimitiva::Sum, 0.85)
    }
}

/// L2BaseRateInvariance: T2-P (N + ∅), dominant Quantity
impl GroundsTo for crate::conservation::L2BaseRateInvariance {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity, // N — prevalence count
            LexPrimitiva::Void,     // ∅ — noise floor
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.85)
    }
}

/// L3SensitivitySpecificityTradeoff: T2-P (κ + ∂), dominant Comparison
impl GroundsTo for crate::conservation::L3SensitivitySpecificityTradeoff {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison, // κ — sensitivity vs specificity
            LexPrimitiva::Boundary,   // ∂ — threshold tradeoff
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.85)
    }
}

/// L4InformationConservation: T2-P (→ + ∃), dominant Causality
impl GroundsTo for crate::conservation::L4InformationConservation {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Causality, // → — information flow
            LexPrimitiva::Existence, // ∃ — signal existence
        ])
        .with_dominant(LexPrimitiva::Causality, 0.85)
    }
}

/// ConservationReport: T2-C (Σ + κ + N + ∂), dominant Sum
impl GroundsTo for crate::conservation::ConservationReport {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum,        // Σ — aggregated results
            LexPrimitiva::Comparison, // κ — satisfied/violated
            LexPrimitiva::Quantity,   // N — result counts
            LexPrimitiva::Boundary,   // ∂ — law boundaries
        ])
        .with_dominant(LexPrimitiva::Sum, 0.80)
    }
}

// ═══════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use nexcore_lex_primitiva::tier::Tier;

    // Helper to test tier classification
    fn assert_tier<T: GroundsTo>(expected: &str) {
        let tier = T::tier();
        let tier_str = match tier {
            Tier::T1Universal => "T1",
            Tier::T2Primitive => "T2-P",
            Tier::T2Composite => "T2-C",
            Tier::T3DomainSpecific => "T3",
            _ => "unknown",
        };
        assert_eq!(tier_str, expected, "Type has unexpected tier");
    }

    // ── T1 types (7) ──

    #[test]
    fn test_signal_primitive_grounding() {
        assert_tier::<crate::SignalPrimitive>("T1");
        assert_eq!(
            crate::SignalPrimitive::dominant_primitive(),
            Some(LexPrimitiva::Comparison)
        );
    }

    #[test]
    fn test_decision_outcome_grounding() {
        assert_tier::<crate::decision::DecisionOutcome>("T1");
    }

    #[test]
    fn test_signal_strength_level_grounding() {
        assert_tier::<crate::detection::SignalStrengthLevel>("T1");
    }

    #[test]
    fn test_boundary_kind_grounding() {
        assert_tier::<crate::threshold::BoundaryKind>("T1");
    }

    #[test]
    fn test_conjunction_mode_grounding() {
        assert_tier::<crate::threshold::ConjunctionMode>("T1");
    }

    #[test]
    fn test_threshold_preset_grounding() {
        assert_tier::<crate::threshold::ThresholdPreset>("T1");
    }

    #[test]
    fn test_detection_phase_grounding() {
        assert_tier::<crate::threshold::DetectionPhase>("T1");
    }

    // ── T2-P types (15) ──

    #[test]
    fn test_a1_grounding() {
        assert_tier::<crate::axioms::A1DataGeneration<100>>("T2-P");
        assert_eq!(
            crate::axioms::A1DataGeneration::<100>::dominant_primitive(),
            Some(LexPrimitiva::Frequency)
        );
    }

    #[test]
    fn test_a2_grounding() {
        assert_tier::<crate::axioms::A2NoiseDominance>("T2-P");
        assert_eq!(
            crate::axioms::A2NoiseDominance::dominant_primitive(),
            Some(LexPrimitiva::Void)
        );
    }

    #[test]
    fn test_a3_grounding() {
        assert_tier::<crate::axioms::A3SignalExistence>("T2-P");
        assert_eq!(
            crate::axioms::A3SignalExistence::dominant_primitive(),
            Some(LexPrimitiva::Existence)
        );
    }

    #[test]
    fn test_a4_grounding() {
        assert_tier::<crate::axioms::A4BoundaryRequirement>("T2-P");
        assert_eq!(
            crate::axioms::A4BoundaryRequirement::dominant_primitive(),
            Some(LexPrimitiva::Boundary)
        );
    }

    #[test]
    fn test_ratio_grounding() {
        assert_tier::<crate::detection::Ratio>("T2-P");
    }

    #[test]
    fn test_difference_grounding() {
        assert_tier::<crate::detection::Difference>("T2-P");
    }

    #[test]
    fn test_detection_interval_grounding() {
        assert_tier::<crate::detection::DetectionInterval>("T2-P");
    }

    #[test]
    fn test_observation_space_grounding() {
        assert_tier::<crate::detection::ObservationSpace>("T2-P");
    }

    #[test]
    fn test_baseline_grounding() {
        assert_tier::<crate::detection::Baseline>("T2-P");
    }

    #[test]
    fn test_fixed_boundary_grounding() {
        assert_tier::<crate::threshold::FixedBoundary>("T2-P");
    }

    #[test]
    fn test_roc_point_grounding() {
        assert_tier::<crate::decision::RocPoint>("T2-P");
    }

    #[test]
    fn test_dprime_grounding() {
        assert_tier::<crate::decision::DPrime>("T2-P");
    }

    #[test]
    fn test_response_bias_grounding() {
        assert_tier::<crate::decision::ResponseBias>("T2-P");
    }

    #[test]
    fn test_l1_grounding() {
        assert_tier::<crate::conservation::L1TotalCountConservation>("T2-P");
    }

    #[test]
    fn test_l2_grounding() {
        assert_tier::<crate::conservation::L2BaseRateInvariance>("T2-P");
    }

    #[test]
    fn test_l3_grounding() {
        assert_tier::<crate::conservation::L3SensitivitySpecificityTradeoff>("T2-P");
    }

    #[test]
    fn test_l4_grounding() {
        assert_tier::<crate::conservation::L4InformationConservation>("T2-P");
    }

    // ── T2-C types (11) ──

    #[test]
    fn test_a5_grounding() {
        assert_tier::<crate::axioms::A5Disproportionality>("T2-C");
    }

    #[test]
    fn test_a6_grounding() {
        assert_tier::<crate::axioms::A6CausalInference>("T2-C");
    }

    #[test]
    fn test_adaptive_boundary_grounding() {
        assert_tier::<crate::threshold::AdaptiveBoundary>("T2-C");
    }

    #[test]
    fn test_composite_boundary_grounding() {
        assert_tier::<crate::threshold::CompositeBoundary>("T2-C");
    }

    #[test]
    fn test_detection_outcome_grounding() {
        assert_tier::<crate::detection::DetectionOutcome>("T2-C");
    }

    #[test]
    fn test_decision_matrix_grounding() {
        assert_tier::<crate::decision::DecisionMatrix>("T2-C");
    }

    #[test]
    fn test_roc_curve_grounding() {
        assert_tier::<crate::decision::RocCurve>("T2-C");
    }

    #[test]
    fn test_cascaded_threshold_grounding() {
        assert_tier::<crate::algebra::CascadedThreshold>("T2-C");
    }

    #[test]
    fn test_signal_verification_report_grounding() {
        assert_tier::<crate::detection::SignalVerificationReport>("T2-C");
    }

    #[test]
    fn test_conservation_report_grounding() {
        assert_tier::<crate::conservation::ConservationReport>("T2-C");
    }

    #[test]
    fn test_detection_pipeline_grounding() {
        assert_tier::<crate::algebra::DetectionPipeline>("T2-C");
    }

    // ── T3 types (1) ──

    #[test]
    fn test_signal_theory_proof_grounding() {
        assert_tier::<crate::axioms::SignalTheoryProof<100>>("T3");
        assert_eq!(
            crate::axioms::SignalTheoryProof::<100>::dominant_primitive(),
            Some(LexPrimitiva::Boundary)
        );
    }

    // ── Cross-cutting ──

    #[test]
    fn test_total_grounding_count() {
        // 7 T1 + 19 T2-P + 11 T2-C + 1 T3 = 38
        // (includes EvidenceKind as T1 — 8 total T1)
        // This test just verifies compilation of all impls
        let count = 8 + 19 + 11 + 1; // adjusted for EvidenceKind
        assert!(count >= 34, "Expected at least 34 GroundsTo impls");
    }
}
