//! # Aggregation Pipeline — Multi-Stage Causal Feature Integration
//!
//! Tier: T2-C | Primitives: Σ Sum, ρ Recursion, ∂ Boundary, → Causality
//!
//! Models the Beer-Lambert → Hill → Arrhenius pipeline pattern where:
//! - **Beer-Lambert** (Σ Sum): weighted feature summation (absorptivity × path × concentration)
//! - **Hill** (ρ Recursion): cooperative amplification via recursive binding feedback
//! - **Arrhenius** (∂ Boundary): threshold gate — activation energy barrier for classification
//! - **Pipeline** (→ Causality): directed causal chain binding all stages
//!
//! This composition fills a gap in the Lex Primitiva registry — {Σ, ρ, ∂, →}
//! was previously unrepresented despite being the canonical pattern for
//! multi-feature aggregation with nonlinear amplification and threshold gating.
//!
//! ## Cross-Domain Transfer
//!
//! | Domain | Instantiation |
//! |--------|---------------|
//! | AI Text Detection | 5 features → Beer-Lambert → Hill → Arrhenius → verdict |
//! | Signal Detection | disproportionality metrics → aggregation → amplification → alert |
//! | Clinical Trials | endpoints → composite → significance → decision |
//! | Manufacturing QC | sensor readings → weighted sum → control chart → alarm |

/// Three-stage aggregation pipeline: summation → amplification → gating.
///
/// Tier: T2-C
///
/// Captures the universal pattern of collecting multiple weighted signals,
/// amplifying via cooperative (Hill) dynamics, and gating through an
/// activation energy threshold (Arrhenius).
#[derive(Debug, Clone, PartialEq)]
pub struct AggregationPipeline {
    /// Number of input features entering the summation stage.
    pub feature_count: usize,

    /// Beer-Lambert composite score after weighted summation.
    /// A = Σ(εᵢ × lᵢ × cᵢ) across all features.
    pub beer_lambert_composite: f64,

    /// Hill coefficient controlling cooperative amplification.
    /// nH > 1: positive cooperativity (signal amplification)
    /// nH = 1: no cooperativity (linear passthrough)
    /// nH < 1: negative cooperativity (signal dampening)
    pub hill_coefficient: f64,

    /// Hill EC50 — concentration at half-maximal response.
    pub hill_ec50: f64,

    /// Arrhenius activation energy threshold.
    /// Pipeline output must exceed this to trigger classification.
    pub activation_energy: f64,

    /// Final probability after full pipeline traversal.
    pub output_probability: f64,
}

impl AggregationPipeline {
    /// Create a new pipeline result from the three stages.
    #[must_use]
    pub fn new(
        feature_count: usize,
        beer_lambert_composite: f64,
        hill_coefficient: f64,
        hill_ec50: f64,
        activation_energy: f64,
        output_probability: f64,
    ) -> Self {
        Self {
            feature_count,
            beer_lambert_composite,
            hill_coefficient,
            hill_ec50,
            activation_energy,
            output_probability,
        }
    }

    /// Check whether the pipeline output exceeds the activation threshold.
    ///
    /// This is the ∂ Boundary primitive in action — the Arrhenius gate.
    #[must_use]
    pub fn exceeds_threshold(&self, threshold: f64) -> bool {
        self.output_probability >= threshold
    }

    /// Compute the confidence distance from the threshold.
    ///
    /// Positive = above threshold (classified as generated).
    /// Negative = below threshold (classified as human).
    #[must_use]
    pub fn threshold_distance(&self, threshold: f64) -> f64 {
        self.output_probability - threshold
    }
}
