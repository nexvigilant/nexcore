//! Lex Primitiva Parameters (T1 Symbolic Foundation)
//! Tier: T1 (Foundation)
//!
//! Grounding, composition, weight, and state-mode analysis for the 15 Lex Primitiva symbols.

use rmcp::schemars::{self, JsonSchema};
use rmcp::serde::{Deserialize, Serialize};

/// Parameters for listing all 16 Lex Primitiva symbols
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct LexPrimitivaListParams {
    /// If true, include symbol notation (σ, μ, ς, etc.) in output
    #[serde(default)]
    pub include_symbols: bool,
}

/// Parameters for getting details about a specific Lex Primitiva
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct LexPrimitivaGetParams {
    /// Name of the primitive (e.g., "Sequence", "Mapping", "State")
    pub name: String,
}

/// Parameters for classifying a type's grounding tier
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct LexPrimitivaTierParams {
    /// Type name to classify
    pub type_name: String,
}

/// Parameters for computing primitive composition of a grounded type
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct LexPrimitivaCompositionParams {
    /// Type name to analyze
    pub type_name: String,
}

/// Parameters for reverse-composing T1 primitives upward through the tier DAG.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct LexPrimitivaReverseComposeParams {
    /// Primitive names to compose
    pub primitives: Vec<String>,
    /// Optional target pattern name hint
    #[serde(default)]
    pub pattern_hint: Option<String>,
    /// Minimum coherence threshold
    #[serde(default)]
    pub min_coherence: Option<f64>,
}

/// Parameters for reverse-looking up grounded types by their T1 primitives.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct LexPrimitivaReverseLookupParams {
    /// Primitive names to search for
    pub primitives: Vec<String>,
    /// Match mode: "exact", "superset", "subset"
    #[serde(default)]
    pub match_mode: Option<String>,
}

/// Parameters for computing molecular weight of a word/concept.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct LexPrimitivaMolecularWeightParams {
    /// Primitive names composing the word
    pub primitives: Vec<String>,
    /// Optional concept name for labeling
    #[serde(default)]
    pub name: Option<String>,
    /// If true, include the full periodic table in the response
    #[serde(default)]
    pub include_periodic_table: bool,
}

/// Get the disambiguated State (ς) mode for a grounded type.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct LexPrimitivaStateModeParams {
    /// The type name to query
    pub type_name: String,
}

/// Parameters for computing a dominant shift (phase transition) analysis.
///
/// Given a base set of T1 primitives and one new primitive to add, detects
/// whether the dominant primitive changes — a "phase transition" in composition
/// character.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct LexPrimitivaDominantShiftParams {
    /// Base primitive names (e.g., ["Comparison", "Quantity"]).
    /// May be empty — an empty base has no old dominant.
    pub base_primitives: Vec<String>,
    /// The primitive being added (e.g., "Boundary").
    pub added_primitive: String,
}

/// Parameters for self-synthesis of new primitives
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct LexPrimitivaSynthParams {
    /// Natural language description of the observed pattern
    pub description: String,
    /// Sample data illustrating the new structure (JSON)
    pub sample_data: serde_json::Value,
}

// ============================================================================
// Compound Growth Parameters
// ============================================================================

/// Parameters for compound growth projection.
#[derive(Debug, Default, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct CompoundGrowthParams {
    /// Tier to add primitives to: "T1", "T2-P", "T2-C", "T3"
    #[serde(default)]
    pub add_tier: Option<String>,
    /// Number of primitives to add
    #[serde(default)]
    pub add_count: Option<u32>,
}

// ============================================================================
// Compound Growth Detector Parameters
// ============================================================================

/// Parameters for compound growth phase and bottleneck detection.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct CompoundDetectorParams {
    /// Array of basis snapshots in chronological order.
    pub snapshots: Vec<CompoundDetectorSnapshot>,
}

/// A single basis snapshot for compound growth detection.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct CompoundDetectorSnapshot {
    /// Session identifier
    pub session: String,
    /// T1 primitive count
    pub t1_count: u32,
    /// T2-P primitive count
    pub t2_p_count: u32,
    /// T2-C primitive count
    pub t2_c_count: u32,
    /// T3 primitive count
    pub t3_count: u32,
    /// Primitives reused from existing basis
    pub reused: u32,
    /// Total primitives needed for this session
    pub total_needed: u32,
}
