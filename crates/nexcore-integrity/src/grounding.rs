//! # GroundsTo Implementations for Integrity Types
//!
//! Connects integrity assessment types to the Lex Primitiva type system.

use nexcore_lex_primitiva::grounding::GroundsTo;
use nexcore_lex_primitiva::primitiva::{LexPrimitiva, PrimitiveComposition};

use crate::assessment::IntegrityAssessment;
use crate::bloom::BloomThresholds;
use crate::classify::{Classification, Verdict};
use crate::profile::CalibrationProfile;

/// IntegrityAssessment: T3 (σ · → · ∂ · N · Σ · ρ), dominant →
///
/// Full pipeline result: features → aggregate → classify. The causal chain
/// from raw text to verdict is the dominant operation.
impl GroundsTo for IntegrityAssessment {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sequence,  // σ — pipeline stage ordering
            LexPrimitiva::Causality, // → — features → score → verdict chain
            LexPrimitiva::Boundary,  // ∂ — threshold-based classification
            LexPrimitiva::Quantity,  // N — numeric scores and features
            LexPrimitiva::Sum,       // Σ — Beer-Lambert aggregation
            LexPrimitiva::Recursion, // ρ — Hill cooperative amplification
        ])
        .with_dominant(LexPrimitiva::Causality, 0.85)
    }
}

/// Classification: T2-C (∂ · → · N), dominant ∂
///
/// Verdict + probability + confidence. The boundary decision
/// (above/below threshold) is the dominant operation.
impl GroundsTo for Classification {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary,  // ∂ — threshold decision
            LexPrimitiva::Causality, // → — score → verdict mapping
            LexPrimitiva::Quantity,  // N — probability and confidence
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.90)
    }
}

/// Verdict: T1 (∂)
///
/// Binary classification outcome: Human or Generated.
/// Pure boundary — the most fundamental decision.
impl GroundsTo for Verdict {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary, // ∂ — binary partition
        ])
        .with_dominant(LexPrimitiva::Boundary, 1.0)
    }
}

/// BloomThresholds: T2-C (∂ · κ · N), dominant ∂
///
/// Threshold configuration indexed by cognitive level.
/// Boundary-dominant: each level defines a decision boundary.
impl GroundsTo for BloomThresholds {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary,   // ∂ — threshold values
            LexPrimitiva::Comparison, // κ — level ordering
            LexPrimitiva::Quantity,   // N — numeric thresholds
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.85)
    }
}

/// CalibrationProfile: T2-C (μ · N · ∂), dominant μ
///
/// Domain → baseline expectations mapping.
/// Mapping-dominant: the profile IS a domain → parameters mapping.
impl GroundsTo for CalibrationProfile {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping,  // μ — domain → baselines
            LexPrimitiva::Quantity, // N — numeric baselines
            LexPrimitiva::Boundary, // ∂ — domain-specific thresholds
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.85)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_integrity_assessment_tier() {
        let comp = IntegrityAssessment::primitive_composition();
        // 6 primitives = T3
        assert_eq!(comp.primitives.len(), 6);
        assert_eq!(IntegrityAssessment::tier().code(), "T3");
    }

    #[test]
    fn test_classification_tier() {
        let comp = Classification::primitive_composition();
        // 3 primitives = T2-P
        assert_eq!(comp.primitives.len(), 3);
    }

    #[test]
    fn test_verdict_tier() {
        let comp = Verdict::primitive_composition();
        // 1 primitive = T1
        assert_eq!(comp.primitives.len(), 1);
        assert_eq!(Verdict::tier().code(), "T1");
    }

    #[test]
    fn test_bloom_thresholds_tier() {
        let comp = BloomThresholds::primitive_composition();
        assert_eq!(comp.primitives.len(), 3);
    }

    #[test]
    fn test_calibration_profile_tier() {
        let comp = CalibrationProfile::primitive_composition();
        assert_eq!(comp.primitives.len(), 3);
    }

    #[test]
    fn test_dominant_primitives() {
        assert_eq!(
            IntegrityAssessment::primitive_composition().dominant,
            Some(LexPrimitiva::Causality)
        );
        assert_eq!(
            Classification::primitive_composition().dominant,
            Some(LexPrimitiva::Boundary)
        );
        assert_eq!(
            Verdict::primitive_composition().dominant,
            Some(LexPrimitiva::Boundary)
        );
        assert_eq!(
            BloomThresholds::primitive_composition().dominant,
            Some(LexPrimitiva::Boundary)
        );
        assert_eq!(
            CalibrationProfile::primitive_composition().dominant,
            Some(LexPrimitiva::Mapping)
        );
    }
}
