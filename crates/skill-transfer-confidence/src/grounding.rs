//! # GroundsTo implementations for skill-transfer-confidence types
//!
//! Transfer confidence computation types grounded to the Lex Primitiva type system.
//!
//! ## Dominant Primitive Distribution
//!
//! - `TransferConfidence` -- Mapping (mu) dominant as it maps dimension scores to confidence.
//! - `TransferScore` -- Quantity (N) dominant as it holds numeric scores.
//! - `ConfidenceTier` -- Comparison (kappa) dominant as it classifies confidence levels.
//! - `TransferConfidenceSkill` -- Mapping (mu) dominant as it wraps the computation as a skill.

use nexcore_lex_primitiva::grounding::GroundsTo;
use nexcore_lex_primitiva::primitiva::{LexPrimitiva, PrimitiveComposition};

use crate::TransferConfidenceSkill;
use crate::compute::{ConfidenceTier, TransferConfidence, TransferScore};

/// TransferConfidence: T2-P (mu + N + kappa), dominant mu
///
/// Weighted confidence calculator. Maps three dimension scores (structural,
/// functional, contextual) to a composite confidence via weighted sum.
/// Mapping-dominant because the core operation is (s, f, c) -> TransferScore.
/// Quantity is secondary (f64 weights). Comparison is tertiary (threshold-based tier).
impl GroundsTo for TransferConfidence {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping,    // mu -- dimension scores -> confidence
            LexPrimitiva::Quantity,   // N -- f64 weights
            LexPrimitiva::Comparison, // kappa -- tier classification
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.85)
    }
}

/// TransferScore: T2-C (N + kappa + mu + partial), dominant N
///
/// Result of confidence computation with individual dimension scores,
/// overall confidence, and tier classification.
/// Quantity-dominant because the type carries four f64 scores.
/// Comparison is secondary (tier classification).
/// Mapping is tertiary (scores -> tier). Boundary is quaternary (0.0-1.0 clamping).
impl GroundsTo for TransferScore {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity,   // N -- four f64 scores
            LexPrimitiva::Comparison, // kappa -- tier classification
            LexPrimitiva::Mapping,    // mu -- scores -> tier
            LexPrimitiva::Boundary,   // partial -- 0.0-1.0 clamping
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.80)
    }
}

/// ConfidenceTier: T1-Universal (kappa), dominant kappa
///
/// Four-level tier: High, Medium, Low, VeryLow.
/// Pure comparison -- discriminates confidence magnitude.
impl GroundsTo for ConfidenceTier {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison, // kappa -- confidence level comparison
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.95)
    }
}

/// TransferConfidenceSkill: T2-P (mu + kappa + N), dominant mu
///
/// Skill wrapper around the transfer confidence computation.
/// Mapping-dominant because it maps skill context inputs to confidence output.
/// Comparison is secondary (trigger matching). Quantity is tertiary (parsed score args).
impl GroundsTo for TransferConfidenceSkill {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping,    // mu -- context -> confidence output
            LexPrimitiva::Comparison, // kappa -- trigger matching
            LexPrimitiva::Quantity,   // N -- parsed score arguments
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.85)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use nexcore_lex_primitiva::tier::Tier;

    #[test]
    fn transfer_confidence_is_t2p() {
        assert_eq!(TransferConfidence::tier(), Tier::T2Primitive);
    }

    #[test]
    fn transfer_score_is_t2c() {
        assert_eq!(TransferScore::tier(), Tier::T2Composite);
    }

    #[test]
    fn confidence_tier_is_t1() {
        assert_eq!(ConfidenceTier::tier(), Tier::T1Universal);
    }

    #[test]
    fn transfer_confidence_skill_is_t2p() {
        assert_eq!(TransferConfidenceSkill::tier(), Tier::T2Primitive);
    }

    #[test]
    fn transfer_confidence_dominant_is_mapping() {
        let comp = TransferConfidence::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Mapping));
    }

    #[test]
    fn transfer_score_dominant_is_quantity() {
        let comp = TransferScore::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Quantity));
    }

    #[test]
    fn confidence_tier_dominant_is_comparison() {
        let comp = ConfidenceTier::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Comparison));
    }
}
