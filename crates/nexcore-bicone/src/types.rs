use serde::{Deserialize, Serialize};

/// A bicone shape defined by a sequence of widths at each level.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiconeProfile {
    /// Width at each hierarchical level (e.g. nodes, tools, users).
    pub width_sequence: Vec<f64>,
    /// Optional human-readable label for each level.
    pub level_labels: Option<Vec<String>>,
}

/// Aggregated geometric and information-theoretic metrics for a bicone.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiconeMetrics {
    /// Volume integral: π × Σ(w²).
    pub volume: f64,
    /// Shannon entropy H over the width distribution (bits).
    pub entropy: f64,
    /// Entropy normalised to [0, 1] by H_max = log₂(n).
    pub entropy_normalized: f64,
    /// Ratio of upper-cone mass to lower-cone mass.
    pub asymmetry_ratio: f64,
    /// Width decrease rate from peak to singularity (units/level).
    pub convergence_rate: f64,
    /// Level index of the bicone singularity (minimum width).
    pub singularity_index: usize,
    /// Sum of all widths (total node mass).
    pub total_nodes: f64,
    /// Number of levels in the profile.
    pub level_count: usize,
}

/// Shape-similarity comparison between two bicone profiles.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShapeComparison {
    /// Cosine similarity in [0, 1].
    pub overlap: f64,
    /// Qualitative classification ("identical", "similar", "divergent", …).
    pub classification: String,
    /// Level indices where the two profiles diverge most.
    pub divergent_levels: Vec<usize>,
}

/// Hill-function activation result for a single level.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HillActivation {
    /// Level index.
    pub level: usize,
    /// Raw width at this level.
    pub width: f64,
    /// Hill response in [0, 1].
    pub response: f64,
    /// True when response < 0.10 (throughput bottleneck).
    pub is_bottleneck: bool,
}
