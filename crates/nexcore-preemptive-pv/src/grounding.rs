//! # Lex Primitiva Grounding for nexcore-preemptive-pv
//!
//! GroundsTo implementations for all nexcore-preemptive-pv public types.
//! This crate implements the three-tier Preemptive PV Equation:
//! Psi(d,e,t) = DeltaG * Gamma * Omega * (1 - eta)
//!
//! ## Type Grounding Table
//!
//! | Type | Primitives | Dominant | Tier | Rationale |
//! |------|-----------|----------|------|-----------|
//! | Seriousness | Σ κ ∂ | ∂ | T2-P | Safety boundary classification of severity |
//! | SafetyLambda | ∂ N | ∂ | T2-P | Boundary multiplier lowering detection threshold |
//! | DrugEventPair | × ∃ | × | T2-P | Product of drug + event identities |
//! | ReportingCounts | N × | N | T2-P | Product of 4 numeric counts (2x2 table) |
//! | Decision | Σ → κ | Σ | T2-P | Sum of 3 decision tiers with causal outcome |
//! | ReportingDataPoint | N × | N | T2-P | Product of time + rate numerics |
//! | InterventionResult | N ∂ → | ∂ | T2-P | Boundary-driven numeric intervention outcome |
//! | GibbsParams | N × → | N | T2-P | Product of 3 thermodynamic parameters |
//! | NoiseParams | N ∂ ν | ∂ | T2-P | Boundary correction with frequency semantics |
//! | PredictiveConfig | ς N | ς | T2-P | Configuration state with numeric parameters |
//! | PredictiveResult | N → ∃ ν | N | T2-C | Numeric multi-component signal result |
//! | PreemptiveConfig | ς N → ∂ | ς | T2-C | Full domain configuration state |
//! | PreemptiveResult | N → ∂ ∝ ∃ κ | ∂ | T3 | Full 3-tier decision with irreversibility |

use nexcore_lex_primitiva::grounding::GroundsTo;
use nexcore_lex_primitiva::primitiva::{LexPrimitiva, PrimitiveComposition};

use crate::types::{
    Decision, DrugEventPair, GibbsParams, InterventionResult, NoiseParams, ReportingCounts,
    ReportingDataPoint, SafetyLambda, Seriousness,
};

use crate::predictive::{PredictiveConfig, PredictiveResult};
use crate::preemptive::{PreemptiveConfig, PreemptiveResult};

// ============================================================================
// T2-P (2-3 unique primitives)
// ============================================================================

/// Seriousness: ICH E2A seriousness categories.
/// Tier: T2Primitive. Dominant: ∂ Boundary.
/// WHY: Safety boundary classification -- determines detection threshold sensitivity.
/// The seriousness level IS the safety boundary: Fatal vs NonSerious defines the
/// boundary between aggressive and routine monitoring.
impl GroundsTo for Seriousness {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum,        // Σ -- one-of-5 variant
            LexPrimitiva::Comparison, // κ -- severity ordering
            LexPrimitiva::Boundary,   // ∂ -- safety boundary classification
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.75)
    }
}

/// SafetyLambda: Threshold multiplier derived from seriousness.
/// Tier: T2Primitive. Dominant: ∂ Boundary.
/// WHY: Directly adjusts the detection boundary threshold.
/// lambda_safety lowers theta_detection for severe outcomes.
impl GroundsTo for SafetyLambda {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary, // ∂ -- adjusts detection boundary
            LexPrimitiva::Quantity, // N -- numeric multiplier value
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.85)
    }
}

/// DrugEventPair: Product of drug identifier and event identifier.
/// Tier: T2Primitive. Dominant: × Product.
/// WHY: Pure conjunctive combination of two identity strings.
impl GroundsTo for DrugEventPair {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Product,   // × -- (drug_id, event_id) tuple
            LexPrimitiva::Existence, // ∃ -- identifies a specific drug-event pair
        ])
        .with_dominant(LexPrimitiva::Product, 0.80)
    }
}

/// ReportingCounts: 2x2 contingency table (a, b, c, d).
/// Tier: T2Primitive. Dominant: N Quantity.
/// WHY: Four numeric counts forming the foundation of disproportionality analysis.
impl GroundsTo for ReportingCounts {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity, // N -- four numeric counts
            LexPrimitiva::Product,  // × -- product of 4 cells
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.85)
    }
}

/// Decision: Three-tier decision outcome (Monitor, Predict, Intervene).
/// Tier: T2Primitive. Dominant: Σ Sum.
/// WHY: Exclusive one-of-3 variant selection with causal action semantics.
impl GroundsTo for Decision {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum,        // Σ -- one-of-3 decision variant
            LexPrimitiva::Causality,  // → -- decision causes downstream action
            LexPrimitiva::Comparison, // κ -- tier comparison for escalation
        ])
        .with_dominant(LexPrimitiva::Sum, 0.70)
    }
}

/// ReportingDataPoint: Time-rate pair for temporal trajectory analysis.
/// Tier: T2Primitive. Dominant: N Quantity.
/// WHY: Product of two numeric measurements (time, rate).
impl GroundsTo for ReportingDataPoint {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity, // N -- numeric time and rate values
            LexPrimitiva::Product,  // × -- (time, rate) product
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.85)
    }
}

/// InterventionResult: Result of competitive inhibition model.
/// Tier: T2Primitive. Dominant: ∂ Boundary.
/// WHY: Measures harm rate reduction -- the effect of imposing safety boundaries.
/// The inhibited_rate is the new boundary after intervention.
impl GroundsTo for InterventionResult {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity,  // N -- numeric rate values
            LexPrimitiva::Boundary,  // ∂ -- safety boundary enforcement
            LexPrimitiva::Causality, // → -- intervention causes rate change
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.70)
    }
}

/// GibbsParams: Thermodynamic parameters for signal emergence feasibility.
/// Tier: T2Primitive. Dominant: N Quantity.
/// WHY: Product of three numeric parameters (enthalpy, temperature, entropy).
impl GroundsTo for GibbsParams {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity,  // N -- three numeric parameters
            LexPrimitiva::Product,   // × -- (deltaH, T, deltaS) product
            LexPrimitiva::Causality, // → -- determines signal feasibility
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.75)
    }
}

/// NoiseParams: Parameters for noise floor correction (Nernst sigmoid).
/// Tier: T2Primitive. Dominant: ∂ Boundary.
/// WHY: Distinguishes organic signal from noise -- a boundary detection problem.
/// The sigmoid defines the boundary between organic and stimulated reporting.
impl GroundsTo for NoiseParams {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity,  // N -- numeric rates and sensitivity
            LexPrimitiva::Boundary,  // ∂ -- noise/signal boundary
            LexPrimitiva::Frequency, // ν -- reporting rate frequency
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.70)
    }
}

/// PredictiveConfig: Configuration for predictive signal computation.
/// Tier: T2Primitive. Dominant: ς State.
/// WHY: Encapsulated configuration state with numeric parameters.
impl GroundsTo for PredictiveConfig {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::State,    // ς -- configuration state
            LexPrimitiva::Quantity, // N -- numeric parameters
        ])
        .with_dominant(LexPrimitiva::State, 0.80)
    }
}

// ============================================================================
// T2-C (4-5 unique primitives)
// ============================================================================

/// PredictiveResult: Multi-component result from predictive signal evaluation.
/// Tier: T2Composite. Dominant: N Quantity.
/// WHY: Composed of multiple numeric metrics (DeltaG, gamma, eta, psi).
impl GroundsTo for PredictiveResult {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity,  // N -- numeric signal metrics
            LexPrimitiva::Causality, // → -- feasibility causes signal potential
            LexPrimitiva::Existence, // ∃ -- signal existence test
            LexPrimitiva::Frequency, // ν -- temporal trajectory frequency
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.65)
    }
}

/// PreemptiveConfig: Full domain configuration for preemptive decision-making.
/// Tier: T2Composite. Dominant: ς State.
/// WHY: Nested configuration state containing sub-configs and boundary parameters.
impl GroundsTo for PreemptiveConfig {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::State,     // ς -- configuration state
            LexPrimitiva::Quantity,  // N -- numeric thresholds and costs
            LexPrimitiva::Causality, // → -- intervention causal model
            LexPrimitiva::Boundary,  // ∂ -- detection boundary thresholds
        ])
        .with_dominant(LexPrimitiva::State, 0.65)
    }
}

// ============================================================================
// T3 Domain-Specific (6+ unique primitives)
// ============================================================================

/// PreemptiveResult: Complete 3-tier preemptive evaluation result.
/// Tier: T3DomainSpecific. Dominant: ∂ Boundary.
/// WHY: The ultimate safety boundary decision. Integrates predictive metrics,
/// irreversibility, causal assessment, and comparison to determine intervention.
/// This is THE domain type -- patient safety boundary enforcement.
impl GroundsTo for PreemptiveResult {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity,        // N -- pi, omega, threshold numerics
            LexPrimitiva::Causality,       // → -- intervention decision
            LexPrimitiva::Boundary,        // ∂ -- safety boundary enforcement
            LexPrimitiva::Irreversibility, // ∝ -- omega irreversibility weighting
            LexPrimitiva::Existence,       // ∃ -- signal existence determination
            LexPrimitiva::Comparison,      // κ -- threshold comparison
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.70)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nexcore_lex_primitiva::grounding::GroundsTo;
    use nexcore_lex_primitiva::tier::Tier;

    #[test]
    fn seriousness_is_t2p_boundary() {
        assert_eq!(Seriousness::tier(), Tier::T2Primitive);
        assert_eq!(
            Seriousness::dominant_primitive(),
            Some(LexPrimitiva::Boundary)
        );
    }

    #[test]
    fn safety_lambda_is_t2p_boundary() {
        assert_eq!(SafetyLambda::tier(), Tier::T2Primitive);
        assert_eq!(
            SafetyLambda::dominant_primitive(),
            Some(LexPrimitiva::Boundary)
        );
    }

    #[test]
    fn drug_event_pair_is_t2p_product() {
        assert_eq!(DrugEventPair::tier(), Tier::T2Primitive);
        assert_eq!(
            DrugEventPair::dominant_primitive(),
            Some(LexPrimitiva::Product)
        );
    }

    #[test]
    fn reporting_counts_is_t2p_quantity() {
        assert_eq!(ReportingCounts::tier(), Tier::T2Primitive);
        assert_eq!(
            ReportingCounts::dominant_primitive(),
            Some(LexPrimitiva::Quantity)
        );
    }

    #[test]
    fn decision_is_t2p_sum() {
        // Decision has its own tier(&self) -> u8 method, so use fully-qualified syntax
        assert_eq!(<Decision as GroundsTo>::tier(), Tier::T2Primitive);
        assert_eq!(
            <Decision as GroundsTo>::dominant_primitive(),
            Some(LexPrimitiva::Sum)
        );
    }

    #[test]
    fn reporting_data_point_is_t2p_quantity() {
        assert_eq!(ReportingDataPoint::tier(), Tier::T2Primitive);
        assert_eq!(
            ReportingDataPoint::dominant_primitive(),
            Some(LexPrimitiva::Quantity)
        );
    }

    #[test]
    fn intervention_result_is_t2p_boundary() {
        assert_eq!(InterventionResult::tier(), Tier::T2Primitive);
        assert_eq!(
            InterventionResult::dominant_primitive(),
            Some(LexPrimitiva::Boundary)
        );
    }

    #[test]
    fn gibbs_params_is_t2p_quantity() {
        assert_eq!(GibbsParams::tier(), Tier::T2Primitive);
        assert_eq!(
            GibbsParams::dominant_primitive(),
            Some(LexPrimitiva::Quantity)
        );
    }

    #[test]
    fn noise_params_is_t2p_boundary() {
        assert_eq!(NoiseParams::tier(), Tier::T2Primitive);
        assert_eq!(
            NoiseParams::dominant_primitive(),
            Some(LexPrimitiva::Boundary)
        );
    }

    #[test]
    fn predictive_config_is_t2p_state() {
        assert_eq!(PredictiveConfig::tier(), Tier::T2Primitive);
        assert_eq!(
            PredictiveConfig::dominant_primitive(),
            Some(LexPrimitiva::State)
        );
    }

    #[test]
    fn predictive_result_is_t2c_quantity() {
        assert_eq!(PredictiveResult::tier(), Tier::T2Composite);
        assert_eq!(
            PredictiveResult::dominant_primitive(),
            Some(LexPrimitiva::Quantity)
        );
    }

    #[test]
    fn preemptive_config_is_t2c_state() {
        assert_eq!(PreemptiveConfig::tier(), Tier::T2Composite);
        assert_eq!(
            PreemptiveConfig::dominant_primitive(),
            Some(LexPrimitiva::State)
        );
    }

    #[test]
    fn preemptive_result_is_t3_boundary() {
        assert_eq!(PreemptiveResult::tier(), Tier::T3DomainSpecific);
        assert_eq!(
            PreemptiveResult::dominant_primitive(),
            Some(LexPrimitiva::Boundary)
        );
    }
}
