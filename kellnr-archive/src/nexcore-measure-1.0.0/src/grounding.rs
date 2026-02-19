//! # GroundsTo implementations for nexcore-measure types
//!
//! Connects information theory, graph theory, and statistics types
//! to the Lex Primitiva type system.
//!
//! ## Domain Signature
//!
//! - **N (Quantity)**: dominant across all measurement newtypes
//! - **∂ (Boundary)**: clamped ranges ([0,1], [0,10])
//! - **μ (Mapping)**: counts -> entropy, features -> scores

use nexcore_lex_primitiva::grounding::GroundsTo;
use nexcore_lex_primitiva::primitiva::{LexPrimitiva, PrimitiveComposition};
use nexcore_lex_primitiva::state_mode::StateMode;

use crate::error::MeasureError;
use crate::types::{
    BayesianPosterior, Centrality, ChemistryMapping, CodeDensityIndex, CouplingRatio, CrateHealth,
    CrateId, CrateMeasurement, Density, DriftDirection, DriftResult, Entropy, GraphAnalysis,
    GraphNode, HealthComponents, HealthRating, HealthScore, MeasureTimestamp, PoissonCi,
    Probability, RatingDistribution, RegressionResult, TestDensity, WelchResult, WorkspaceHealth,
    WorkspaceMeasurement,
};

// ---------------------------------------------------------------------------
// T2-P: Newtypes (2-3 primitives)
// ---------------------------------------------------------------------------

/// Entropy: T2-P (N + ∂), dominant N
///
/// Shannon entropy in bits, clamped to >= 0. Fundamentally a quantity.
impl GroundsTo for Entropy {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity, // N -- numeric value in bits
            LexPrimitiva::Boundary, // ∂ -- clamped to non-negative
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.9)
    }
}

/// Probability: T2-P (N + ∂), dominant ∂
///
/// Clamped to [0, 1]. The boundary constraint IS the type.
impl GroundsTo for Probability {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity, // N -- numeric value
            LexPrimitiva::Boundary, // ∂ -- [0,1] clamp
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.85)
    }
}

/// Density: T2-P (N + ∂), dominant N
///
/// Graph density [0, 1]. A ratio quantity with boundary constraint.
impl GroundsTo for Density {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity, // N -- ratio value
            LexPrimitiva::Boundary, // ∂ -- [0,1] clamp
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.85)
    }
}

/// TestDensity: T2-P (N + ∂), dominant N
///
/// Tests per KLOC, clamped to >= 0. Fundamentally a rate quantity.
impl GroundsTo for TestDensity {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity, // N -- tests/KLOC ratio
            LexPrimitiva::Boundary, // ∂ -- non-negative
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.9)
    }
}

/// Centrality: T2-P (N + ∂ + λ), dominant N
///
/// Node centrality [0, 1]. A positional metric in a graph.
impl GroundsTo for Centrality {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity, // N -- centrality value
            LexPrimitiva::Boundary, // ∂ -- [0,1] clamp
            LexPrimitiva::Location, // λ -- position in graph
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.85)
    }
}

/// CouplingRatio: T2-P (N + ∂), dominant N
///
/// fan_out / (fan_in + fan_out), [0, 1].
impl GroundsTo for CouplingRatio {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity, // N -- ratio
            LexPrimitiva::Boundary, // ∂ -- [0,1] clamp
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.85)
    }
}

/// CodeDensityIndex: T2-P (κ + N + ∂), dominant κ
///
/// Semantic density. Comparison-dominant: it measures code against an ideal density.
impl GroundsTo for CodeDensityIndex {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison, // κ -- relative density
            LexPrimitiva::Quantity,   // N -- raw ratio
            LexPrimitiva::Boundary,   // ∂ -- [0,1] clamp
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.85)
    }
}

/// HealthScore: T2-P (N + ∂ + κ), dominant κ
///
/// Composite score [0, 10] with rating derivation.
/// Comparison-dominant: the score exists to classify health.
impl GroundsTo for HealthScore {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity,   // N -- numeric score
            LexPrimitiva::Boundary,   // ∂ -- [0,10] clamp
            LexPrimitiva::Comparison, // κ -- health classification
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.8)
    }
}

// ---------------------------------------------------------------------------
// T2-C: Composed types (4-5 primitives)
// ---------------------------------------------------------------------------

/// CrateId: T2-P (λ + ∃), dominant λ
///
/// Unique crate identifier. Location-dominant: naming is addressing.
impl GroundsTo for CrateId {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Location,  // λ -- named identity
            LexPrimitiva::Existence, // ∃ -- existence in workspace
        ])
        .with_dominant(LexPrimitiva::Location, 0.9)
    }
}

/// MeasureTimestamp: T2-P (σ + N), dominant σ
///
/// Unix epoch seconds — a point in a time sequence.
impl GroundsTo for MeasureTimestamp {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sequence, // σ -- temporal ordering
            LexPrimitiva::Quantity, // N -- epoch seconds
        ])
        .with_dominant(LexPrimitiva::Sequence, 0.85)
    }
}

/// HealthRating: T2-P (κ + ∂), dominant κ
///
/// Ordinal health categories derived from score boundaries.
impl GroundsTo for HealthRating {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison, // κ -- ordinal classification
            LexPrimitiva::Boundary,   // ∂ -- category thresholds
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.9)
    }
}

/// DriftDirection: T2-P (κ + →), dominant κ
///
/// Direction of statistical drift: improving, degrading, stable.
impl GroundsTo for DriftDirection {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison, // κ -- directional comparison
            LexPrimitiva::Causality,  // → -- change direction
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.85)
    }
}

// ---------------------------------------------------------------------------
// T3: Domain aggregates (6+ primitives)
// ---------------------------------------------------------------------------

/// CrateMeasurement: T3 (ς + N + σ + μ + ∂ + λ), dominant ς
///
/// Per-crate measurement snapshot. State-dominant: frozen point-in-time data.
impl GroundsTo for CrateMeasurement {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::State,    // ς -- snapshot at point in time
            LexPrimitiva::Quantity, // N -- LOC, test count, fan metrics
            LexPrimitiva::Sequence, // σ -- module distribution
            LexPrimitiva::Mapping,  // μ -- crate → measurements
            LexPrimitiva::Boundary, // ∂ -- clamped ratios
            LexPrimitiva::Location, // λ -- crate identity
        ])
        .with_dominant(LexPrimitiva::State, 0.8)
        .with_state_mode(StateMode::Accumulated)
    }

    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Accumulated)
    }
}

/// WorkspaceMeasurement: T3 (ς + Σ + N + σ + ρ + λ), dominant Σ
///
/// Workspace-level aggregate. Sum-dominant: aggregation over crates.
impl GroundsTo for WorkspaceMeasurement {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum,       // Σ -- aggregation
            LexPrimitiva::State,     // ς -- snapshot
            LexPrimitiva::Quantity,  // N -- totals
            LexPrimitiva::Sequence,  // σ -- crate list
            LexPrimitiva::Recursion, // ρ -- dependency depth
            LexPrimitiva::Location,  // λ -- workspace identity
        ])
        .with_dominant(LexPrimitiva::Sum, 0.8)
        .with_state_mode(StateMode::Accumulated)
    }

    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Accumulated)
    }
}

/// CrateHealth: T3 (κ + ς + N + ∂ + λ + Σ), dominant κ
///
/// Composite health assessment. Comparison-dominant: it exists to evaluate.
impl GroundsTo for CrateHealth {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison, // κ -- health classification
            LexPrimitiva::State,      // ς -- assessment snapshot
            LexPrimitiva::Quantity,   // N -- component scores
            LexPrimitiva::Boundary,   // ∂ -- score ranges
            LexPrimitiva::Location,   // λ -- crate identity
            LexPrimitiva::Sum,        // Σ -- composite aggregation
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.8)
        .with_state_mode(StateMode::Accumulated)
    }

    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Accumulated)
    }
}

/// HealthComponents: T2-C (N + ∂ + κ + Σ), dominant N
///
/// Normalized component scores for health computation.
impl GroundsTo for HealthComponents {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity,   // N -- normalized values
            LexPrimitiva::Boundary,   // ∂ -- normalization range
            LexPrimitiva::Comparison, // κ -- optimal comparisons
            LexPrimitiva::Sum,        // Σ -- aggregation into composite
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.8)
    }
}

/// WorkspaceHealth: T3 (Σ + κ + ς + σ + N + ∂), dominant Σ
///
/// Workspace aggregate health. Sum-dominant: mean over crate health.
impl GroundsTo for WorkspaceHealth {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum,        // Σ -- mean aggregation
            LexPrimitiva::Comparison, // κ -- health distribution
            LexPrimitiva::State,      // ς -- snapshot
            LexPrimitiva::Sequence,   // σ -- crate list
            LexPrimitiva::Quantity,   // N -- mean score
            LexPrimitiva::Boundary,   // ∂ -- score ranges
        ])
        .with_dominant(LexPrimitiva::Sum, 0.8)
        .with_state_mode(StateMode::Accumulated)
    }

    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Accumulated)
    }
}

/// RatingDistribution: T2-C (Σ + N + κ), dominant Σ
///
/// Count of crates in each health tier. Aggregation over categories.
impl GroundsTo for RatingDistribution {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum,        // Σ -- counting accumulation
            LexPrimitiva::Quantity,   // N -- count values
            LexPrimitiva::Comparison, // κ -- category buckets
        ])
        .with_dominant(LexPrimitiva::Sum, 0.85)
    }
}

/// DriftResult: T3 (κ + N + ∂ + → + ς + ν), dominant κ
///
/// Welch t-test drift result. Comparison-dominant: detects significant change.
impl GroundsTo for DriftResult {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison, // κ -- statistical significance test
            LexPrimitiva::Quantity,   // N -- t-statistic, p-value, dof
            LexPrimitiva::Boundary,   // ∂ -- significance threshold
            LexPrimitiva::Causality,  // → -- direction of drift
            LexPrimitiva::State,      // ς -- before/after state
            LexPrimitiva::Frequency,  // ν -- rate of change
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.8)
        .with_state_mode(StateMode::Accumulated)
    }

    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Accumulated)
    }
}

/// RegressionResult: T2-C (N + → + κ + ∂), dominant N
///
/// OLS regression coefficients. Fundamentally quantitative.
impl GroundsTo for RegressionResult {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity,   // N -- slope, intercept, r_squared
            LexPrimitiva::Causality,  // → -- slope direction
            LexPrimitiva::Comparison, // κ -- significance test
            LexPrimitiva::Boundary,   // ∂ -- p-value threshold
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.85)
    }
}

/// WelchResult: T2-P (N + κ), dominant N
///
/// Raw Welch t-test output: t-statistic, dof, p-value.
impl GroundsTo for WelchResult {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity,   // N -- t, dof, p-value
            LexPrimitiva::Comparison, // κ -- statistical test
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.85)
    }
}

/// PoissonCi: T2-C (N + ∂ + ν + κ), dominant N
///
/// Poisson confidence interval: rate, lower, upper, alpha.
impl GroundsTo for PoissonCi {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity,   // N -- rate, bounds
            LexPrimitiva::Boundary,   // ∂ -- CI bounds
            LexPrimitiva::Frequency,  // ν -- event rate
            LexPrimitiva::Comparison, // κ -- confidence level
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.8)
    }
}

/// BayesianPosterior: T2-C (N + ς + →), dominant N
///
/// Gamma-Poisson conjugate posterior parameters.
impl GroundsTo for BayesianPosterior {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity,  // N -- mean, variance, alpha, beta
            LexPrimitiva::State,     // ς -- posterior state (updated belief)
            LexPrimitiva::Causality, // → -- prior → posterior update
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.85)
        .with_state_mode(StateMode::Accumulated)
    }

    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Accumulated)
    }
}

/// GraphNode: T2-C (λ + N + ∂ + μ), dominant λ
///
/// A node in the dependency graph with centrality metrics.
impl GroundsTo for GraphNode {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Location, // λ -- node identity
            LexPrimitiva::Quantity, // N -- fan metrics, centrality
            LexPrimitiva::Boundary, // ∂ -- clamped ratios
            LexPrimitiva::Mapping,  // μ -- connections
        ])
        .with_dominant(LexPrimitiva::Location, 0.8)
    }
}

/// GraphAnalysis: T3 (ρ + λ + N + ∂ + σ + μ), dominant ρ
///
/// Dependency graph analysis. Recursion-dominant: graph traversal.
impl GroundsTo for GraphAnalysis {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Recursion, // ρ -- SCC, depth traversal
            LexPrimitiva::Location,  // λ -- node positions
            LexPrimitiva::Quantity,  // N -- counts, density
            LexPrimitiva::Boundary,  // ∂ -- cycle detection limits
            LexPrimitiva::Sequence,  // σ -- topological ordering
            LexPrimitiva::Mapping,   // μ -- edges
        ])
        .with_dominant(LexPrimitiva::Recursion, 0.8)
    }
}

/// ChemistryMapping: T2-C (μ + N + κ + λ), dominant μ
///
/// Cross-domain chemistry bridge mapping. Mapping-dominant: transformation.
impl GroundsTo for ChemistryMapping {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping,    // μ -- source → target mapping
            LexPrimitiva::Quantity,   // N -- mapped value, confidence
            LexPrimitiva::Comparison, // κ -- transfer confidence
            LexPrimitiva::Location,   // λ -- source/target identity
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.85)
    }
}

// ---------------------------------------------------------------------------
// Error types — ∂ dominant
// ---------------------------------------------------------------------------

/// MeasureError: T2-P (∂ + ∅), dominant ∂
///
/// Measurement error variants: boundary violations, missing data.
impl GroundsTo for MeasureError {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary, // ∂ -- constraint violations
            LexPrimitiva::Void,     // ∅ -- empty input, not found
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.9)
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
    fn entropy_is_t2p() {
        assert_eq!(Entropy::tier(), Tier::T2Primitive);
        assert_eq!(Entropy::dominant_primitive(), Some(LexPrimitiva::Quantity));
    }

    #[test]
    fn probability_is_boundary_dominant() {
        assert_eq!(
            Probability::dominant_primitive(),
            Some(LexPrimitiva::Boundary)
        );
    }

    #[test]
    fn health_score_is_comparison_dominant() {
        assert_eq!(
            HealthScore::dominant_primitive(),
            Some(LexPrimitiva::Comparison)
        );
        assert_eq!(HealthScore::tier(), Tier::T2Primitive);
    }

    #[test]
    fn crate_measurement_is_t3() {
        assert_eq!(CrateMeasurement::tier(), Tier::T3DomainSpecific);
        assert_eq!(
            CrateMeasurement::dominant_primitive(),
            Some(LexPrimitiva::State)
        );
    }

    #[test]
    fn workspace_measurement_is_t3() {
        assert_eq!(WorkspaceMeasurement::tier(), Tier::T3DomainSpecific);
        assert_eq!(
            WorkspaceMeasurement::dominant_primitive(),
            Some(LexPrimitiva::Sum)
        );
    }

    #[test]
    fn graph_analysis_is_recursion_dominant() {
        let comp = GraphAnalysis::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Recursion));
        assert!(comp.primitives.contains(&LexPrimitiva::Location));
    }

    #[test]
    fn drift_result_is_t3() {
        assert_eq!(DriftResult::tier(), Tier::T3DomainSpecific);
        assert_eq!(
            DriftResult::dominant_primitive(),
            Some(LexPrimitiva::Comparison)
        );
    }

    #[test]
    fn measure_error_is_boundary_dominant() {
        assert_eq!(
            MeasureError::dominant_primitive(),
            Some(LexPrimitiva::Boundary)
        );
    }

    #[test]
    fn crate_id_is_location_dominant() {
        assert_eq!(CrateId::dominant_primitive(), Some(LexPrimitiva::Location));
    }

    #[test]
    fn chemistry_mapping_is_mapping_dominant() {
        assert_eq!(
            ChemistryMapping::dominant_primitive(),
            Some(LexPrimitiva::Mapping)
        );
    }
}
