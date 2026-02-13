//! # GroundsTo implementations for nexcore-value-mining types
//!
//! Connects economic value signal detection types to the Lex Primitiva type system.
//!
//! ## Domain Signature
//!
//! - **κ (Comparison)**: signal detection is fundamentally comparative
//! - **N (Quantity)**: scores, rates, confidence values
//! - **∂ (Boundary)**: signal thresholds

use nexcore_lex_primitiva::grounding::GroundsTo;
use nexcore_lex_primitiva::primitiva::{LexPrimitiva, PrimitiveComposition};

use crate::error::MiningError;
use crate::goals::{GoalMetric, GoalPortfolio, GoalProgress, GoalStatus, IntelligenceGoal};
use crate::intelligence::{
    IntelligenceState, MonitoringDashboard, SignalConvergence, SourceDiversity, TemporalAlignment,
    ValueIntelligence,
};
use crate::types::{Baseline, SignalStrength, SignalType, ValueSignal};

// ---------------------------------------------------------------------------
// T2-P: Classification types
// ---------------------------------------------------------------------------

/// SignalType: T2-P (Σ + κ), dominant Σ
///
/// Signal classification: Sentiment, Trend, Engagement, Virality, Controversy.
impl GroundsTo for SignalType {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum,        // Σ -- variant enumeration
            LexPrimitiva::Comparison, // κ -- PV algorithm analogs
        ])
        .with_dominant(LexPrimitiva::Sum, 0.85)
    }
}

/// SignalStrength: T2-P (κ + N), dominant κ
///
/// Strength classification: Weak, Moderate, Strong, VeryStrong.
impl GroundsTo for SignalStrength {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison, // κ -- ordinal strength ranking
            LexPrimitiva::Quantity,   // N -- confidence threshold
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.90)
    }
}

/// GoalStatus: T2-P (ς + κ), dominant ς
///
/// Goal lifecycle status: Active, Achieved, Failed, etc.
impl GroundsTo for GoalStatus {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::State,      // ς -- lifecycle position
            LexPrimitiva::Comparison, // κ -- status evaluation
        ])
        .with_dominant(LexPrimitiva::State, 0.85)
    }
}

/// GoalMetric: T2-P (N + κ), dominant N
///
/// Quantitative metric for goal progress tracking.
impl GroundsTo for GoalMetric {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity,   // N -- metric value
            LexPrimitiva::Comparison, // κ -- target comparison
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.85)
    }
}

// ---------------------------------------------------------------------------
// T2-C / T3: Domain types
// ---------------------------------------------------------------------------

/// ValueSignal: T3 (κ + N + σ + ∂ + λ + ν + ς), dominant κ
///
/// A detected value signal with score, confidence, and temporal window.
/// Comparison-dominant: detection IS comparison against baseline.
impl GroundsTo for ValueSignal {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison, // κ -- vs baseline comparison
            LexPrimitiva::Quantity,   // N -- score, confidence
            LexPrimitiva::Sequence,   // σ -- temporal window
            LexPrimitiva::Boundary,   // ∂ -- actionability threshold
            LexPrimitiva::Location,   // λ -- entity, source identity
            LexPrimitiva::Frequency,  // ν -- detection rate
            LexPrimitiva::State,      // ς -- strength classification
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.80)
    }
}

/// Baseline: T2-C (N + σ + ν + ∝), dominant N
///
/// Baseline statistics for comparison. Quantity-dominant: rates and counts.
impl GroundsTo for Baseline {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity,        // N -- rates, averages
            LexPrimitiva::Sequence,        // σ -- temporal computation
            LexPrimitiva::Frequency,       // ν -- posts per hour
            LexPrimitiva::Irreversibility, // ∝ -- EMA decay factor
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.85)
    }
}

/// GoalProgress: T2-C (N + κ + σ + ∂), dominant N
///
/// Progress toward a goal. Quantity-dominant: measurement tracking.
impl GroundsTo for GoalProgress {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity,   // N -- current vs target
            LexPrimitiva::Comparison, // κ -- progress evaluation
            LexPrimitiva::Sequence,   // σ -- temporal tracking
            LexPrimitiva::Boundary,   // ∂ -- target threshold
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.80)
    }
}

/// IntelligenceGoal: T3 (→ + ς + N + κ + σ + ∂), dominant →
///
/// Strategic intelligence goal. Causality-dominant: drives actions.
impl GroundsTo for IntelligenceGoal {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Causality,  // → -- goal drives action
            LexPrimitiva::State,      // ς -- status lifecycle
            LexPrimitiva::Quantity,   // N -- metrics
            LexPrimitiva::Comparison, // κ -- progress evaluation
            LexPrimitiva::Sequence,   // σ -- temporal tracking
            LexPrimitiva::Boundary,   // ∂ -- completion criteria
        ])
        .with_dominant(LexPrimitiva::Causality, 0.80)
    }
}

/// GoalPortfolio: T3 (Σ + σ + κ + N + ς + →), dominant Σ
///
/// Collection of intelligence goals. Sum-dominant: portfolio aggregation.
impl GroundsTo for GoalPortfolio {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum,        // Σ -- goal aggregation
            LexPrimitiva::Sequence,   // σ -- priority ordering
            LexPrimitiva::Comparison, // κ -- priority comparison
            LexPrimitiva::Quantity,   // N -- progress metrics
            LexPrimitiva::State,      // ς -- portfolio state
            LexPrimitiva::Causality,  // → -- driving actions
        ])
        .with_dominant(LexPrimitiva::Sum, 0.80)
    }
}

/// IntelligenceState: T2-C (ς + N + σ + κ), dominant ς
///
/// Current intelligence engine state.
impl GroundsTo for IntelligenceState {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::State,      // ς -- current state
            LexPrimitiva::Quantity,   // N -- metrics
            LexPrimitiva::Sequence,   // σ -- history
            LexPrimitiva::Comparison, // κ -- evaluation
        ])
        .with_dominant(LexPrimitiva::State, 0.80)
    }
}

/// SignalConvergence: T2-P (κ + N), dominant κ
///
/// Convergence of multiple signal types. Comparison-dominant.
impl GroundsTo for SignalConvergence {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison, // κ -- convergence check
            LexPrimitiva::Quantity,   // N -- convergence score
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.90)
    }
}

/// SourceDiversity: T2-P (N + Σ), dominant N
///
/// Diversity of signal sources. Quantity-dominant.
impl GroundsTo for SourceDiversity {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity, // N -- diversity index
            LexPrimitiva::Sum,      // Σ -- source aggregation
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.85)
    }
}

/// TemporalAlignment: T2-P (σ + N), dominant σ
///
/// Temporal alignment of signals. Sequence-dominant.
impl GroundsTo for TemporalAlignment {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sequence, // σ -- temporal ordering
            LexPrimitiva::Quantity, // N -- alignment score
        ])
        .with_dominant(LexPrimitiva::Sequence, 0.85)
    }
}

/// ValueIntelligence: T3 (μ + κ + N + σ + → + ∂), dominant μ
///
/// Full intelligence assessment. Mapping-dominant: data → intelligence.
impl GroundsTo for ValueIntelligence {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping,    // μ -- data → intelligence
            LexPrimitiva::Comparison, // κ -- convergence, alignment
            LexPrimitiva::Quantity,   // N -- confidence scores
            LexPrimitiva::Sequence,   // σ -- temporal analysis
            LexPrimitiva::Causality,  // → -- recommendations
            LexPrimitiva::Boundary,   // ∂ -- actionability thresholds
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.80)
    }
}

/// MonitoringDashboard: T3 (ς + Σ + σ + N + κ + →), dominant ς
///
/// Live monitoring dashboard state.
impl GroundsTo for MonitoringDashboard {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::State,      // ς -- dashboard snapshot
            LexPrimitiva::Sum,        // Σ -- aggregated signals
            LexPrimitiva::Sequence,   // σ -- temporal display
            LexPrimitiva::Quantity,   // N -- metrics
            LexPrimitiva::Comparison, // κ -- status evaluation
            LexPrimitiva::Causality,  // → -- recommendations
        ])
        .with_dominant(LexPrimitiva::State, 0.80)
    }
}

// ---------------------------------------------------------------------------
// Error types
// ---------------------------------------------------------------------------

/// MiningError: T2-P (∂ + ∅), dominant ∂
///
/// Value mining errors: insufficient data, invalid baseline, detection failure.
impl GroundsTo for MiningError {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary, // ∂ -- constraint violations
            LexPrimitiva::Void,     // ∅ -- insufficient data
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.90)
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
    fn signal_type_is_sum_dominant() {
        assert_eq!(SignalType::dominant_primitive(), Some(LexPrimitiva::Sum));
    }

    #[test]
    fn signal_strength_is_comparison_dominant() {
        assert_eq!(
            SignalStrength::dominant_primitive(),
            Some(LexPrimitiva::Comparison)
        );
    }

    #[test]
    fn value_signal_is_t3() {
        assert_eq!(ValueSignal::tier(), Tier::T3DomainSpecific);
        assert_eq!(
            ValueSignal::dominant_primitive(),
            Some(LexPrimitiva::Comparison)
        );
    }

    #[test]
    fn baseline_is_quantity_dominant() {
        assert_eq!(Baseline::dominant_primitive(), Some(LexPrimitiva::Quantity));
    }

    #[test]
    fn intelligence_goal_is_causality_dominant() {
        assert_eq!(
            IntelligenceGoal::dominant_primitive(),
            Some(LexPrimitiva::Causality)
        );
    }

    #[test]
    fn value_intelligence_is_mapping_dominant() {
        assert_eq!(
            ValueIntelligence::dominant_primitive(),
            Some(LexPrimitiva::Mapping)
        );
        assert_eq!(ValueIntelligence::tier(), Tier::T3DomainSpecific);
    }

    #[test]
    fn mining_error_is_boundary_dominant() {
        assert_eq!(
            MiningError::dominant_primitive(),
            Some(LexPrimitiva::Boundary)
        );
    }
}
