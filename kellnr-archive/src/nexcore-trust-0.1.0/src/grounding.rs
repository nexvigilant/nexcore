//! # GroundsTo implementations for trust domain types
//!
//! Connects trust engine types to the Lex Primitiva type system.
//!
//! ## Product (×) Grounding
//!
//! Multi-dimensional trust is the canonical Product primitive in the social domain:
//! - **Ability × Benevolence × Integrity** — independent conjunctive factors
//! - Missing ANY one dimension collapses trust (AND logic, not OR)
//! - This mirrors physics product types: tuples, structs, payoff matrices

use nexcore_lex_primitiva::grounding::GroundsTo;
use nexcore_lex_primitiva::primitiva::{LexPrimitiva, PrimitiveComposition};

use crate::dimension::{DimensionWeights, MultiTrustEngine, TrustDimension};
use crate::engine::{TrustConfig, TrustEngine};
use crate::evidence::Evidence;
use crate::level::TrustLevel;
use crate::volatility::TrustVelocity;

/// MultiTrustEngine: T3 (× · κ · N · ς · μ), dominant × (Product)
///
/// Three independent Beta engines composed conjunctively.
/// Ability × Benevolence × Integrity — all must be present for full trust.
/// Product is dominant because the conjunction IS the engine's purpose.
impl GroundsTo for MultiTrustEngine {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Product,    // × — conjunctive composition of 3 dimensions
            LexPrimitiva::Comparison, // κ — threshold-based level classification
            LexPrimitiva::Quantity,   // N — Beta(α,β) parameters, scores
            LexPrimitiva::State,      // ς — mutable trust state per dimension
            LexPrimitiva::Mapping,    // μ — evidence → score transformation
        ])
        .with_dominant(LexPrimitiva::Product, 0.90)
    }
}

/// TrustDimension: T2-P (× · ς), dominant ×
///
/// Enum of three independent capability axes.
/// Product because dimensions are conjunctive slots in the product type.
impl GroundsTo for TrustDimension {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Product, // × — one factor in the product space
            LexPrimitiva::State,   // ς — each dimension has independent state
        ])
        .with_dominant(LexPrimitiva::Product, 0.90)
    }
}

/// DimensionWeights: T2-C (μ · N · ×), dominant μ
///
/// Maps dimensions to their relative importance.
/// Mapping because it transforms dimension → weight.
impl GroundsTo for DimensionWeights {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping,  // μ — dimension → weight function
            LexPrimitiva::Quantity, // N — numeric weight values
            LexPrimitiva::Product,  // × — weights apply to conjunctive factors
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.85)
    }
}

/// TrustEngine: T2-C (ς · N · μ · ∂), dominant ς
///
/// Single-dimension Beta(α,β) trust tracker.
/// State-dominant: the engine's purpose is maintaining and updating trust state.
impl GroundsTo for TrustEngine {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::State,    // ς — Beta distribution state
            LexPrimitiva::Quantity, // N — α, β parameters
            LexPrimitiva::Mapping,  // μ — evidence → update function
            LexPrimitiva::Boundary, // ∂ — score thresholds for level classification
        ])
        .with_dominant(LexPrimitiva::State, 0.85)
    }
}

/// Evidence: T2-P (κ · N), dominant κ
///
/// Binary positive/negative signal with optional weight.
/// Comparison-dominant: the evidence IS a positive/negative judgment.
impl GroundsTo for Evidence {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison, // κ — positive vs negative
            LexPrimitiva::Quantity,   // N — weight magnitude
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.90)
    }
}

/// TrustLevel: T2-P (∂ · κ), dominant ∂
///
/// Categorical trust classification (NoTrust through HighlyTrusted).
/// Boundary-dominant: levels are threshold boundaries on a continuous score.
impl GroundsTo for TrustLevel {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary,   // ∂ — threshold-defined categories
            LexPrimitiva::Comparison, // κ — ordered comparison between levels
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.90)
    }
}

/// TrustVelocity: T2-C (ν · κ · N), dominant ν
///
/// Rate of trust change with direction detection.
/// Frequency-dominant: measures speed of trust movement over time.
impl GroundsTo for TrustVelocity {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Frequency,  // ν — rate of change
            LexPrimitiva::Comparison, // κ — direction (increasing/decreasing)
            LexPrimitiva::Quantity,   // N — magnitude of velocity
        ])
        .with_dominant(LexPrimitiva::Frequency, 0.85)
    }
}

/// TrustConfig: T2-C (μ · N · ∂ · ν), dominant μ
///
/// Configuration mapping for trust engine behavior.
/// Mapping-dominant: defines parameter → behavior relationships.
impl GroundsTo for TrustConfig {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping,   // μ — config → behavior
            LexPrimitiva::Quantity,  // N — numeric parameters
            LexPrimitiva::Boundary,  // ∂ — thresholds
            LexPrimitiva::Frequency, // ν — decay rate
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.80)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nexcore_lex_primitiva::tier::Tier;

    #[test]
    fn multi_trust_engine_grounds_to_product() {
        let comp = MultiTrustEngine::primitive_composition();
        assert!(comp.primitives.contains(&LexPrimitiva::Product));
        assert_eq!(comp.dominant, Some(LexPrimitiva::Product));
    }

    #[test]
    fn multi_trust_engine_is_t2c() {
        // 5 primitives = T2-C (Cross-Domain Composite)
        assert_eq!(MultiTrustEngine::tier(), Tier::T2Composite);
    }

    #[test]
    fn trust_dimension_grounds_to_product() {
        let comp = TrustDimension::primitive_composition();
        assert!(comp.primitives.contains(&LexPrimitiva::Product));
        assert_eq!(comp.dominant, Some(LexPrimitiva::Product));
    }

    #[test]
    fn trust_dimension_is_t2p() {
        // 2 primitives = T2-P
        assert_eq!(TrustDimension::tier(), Tier::T2Primitive);
    }

    #[test]
    fn dimension_weights_includes_product() {
        let comp = DimensionWeights::primitive_composition();
        assert!(comp.primitives.contains(&LexPrimitiva::Product));
        assert_eq!(comp.dominant, Some(LexPrimitiva::Mapping));
    }

    #[test]
    fn trust_engine_is_state_dominant() {
        let comp = TrustEngine::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::State));
        assert!(!comp.primitives.contains(&LexPrimitiva::Product));
    }

    #[test]
    fn evidence_is_comparison_dominant() {
        let comp = Evidence::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Comparison));
    }

    #[test]
    fn trust_level_is_boundary_dominant() {
        let comp = TrustLevel::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Boundary));
    }

    #[test]
    fn trust_velocity_is_frequency_dominant() {
        let comp = TrustVelocity::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Frequency));
    }
}
