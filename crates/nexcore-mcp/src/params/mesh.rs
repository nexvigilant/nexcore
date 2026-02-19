//! Mesh Network + MeSH API Parameters
//! Tier: T2-T3 (Networking Primitives + Medical Subject Headings)

use rmcp::schemars::{self, JsonSchema};
use rmcp::serde::{Deserialize, Serialize};

// ============================================================================
// Mesh Network Simulation Parameters
// ============================================================================

/// Parameters for mesh network simulation.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct MeshNetworkSimulateParams {
    /// Number of nodes (2-20).
    pub node_count: usize,
    /// Topology: "ring", "star", etc.
    #[serde(default = "default_mesh_topology")]
    pub topology: String,
    /// Duration in ms.
    #[serde(default = "default_mesh_duration_ms")]
    pub duration_ms: u64,
}

fn default_mesh_topology() -> String {
    "ring".to_string()
}

fn default_mesh_duration_ms() -> u64 {
    100
}

/// Parameters for mesh route quality.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct MeshNetworkRouteQualityParams {
    /// Latency in ms.
    pub latency_ms: f64,
    /// Reliability ratio.
    pub reliability: f64,
    /// Number of hops.
    pub hop_count: u8,
}

/// Parameters for node info query.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct MeshNetworkNodeInfoParams {
    /// Node address segments.
    pub address: Vec<String>,
}

// ============================================================================
// NLM MeSH API Parameters
// ============================================================================

/// Parameters for MeSH descriptor lookup.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct MeshLookupParams {
    /// Descriptor UI or name to look up.
    pub identifier: String,
    /// Output format: "brief" or "full".
    #[serde(default)]
    pub format: Option<String>,
}

/// Parameters for MeSH search.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct MeshSearchParams {
    /// Search query.
    pub query: String,
    /// Maximum results (default: 10, max: 50).
    #[serde(default)]
    pub limit: Option<usize>,
    /// Include scope notes in results.
    #[serde(default)]
    pub include_scope_note: Option<bool>,
}

/// Parameters for MeSH tree navigation.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct MeshTreeParams {
    /// Descriptor UI to navigate from.
    pub descriptor_ui: String,
    /// Direction: "ancestors", "descendants", "siblings".
    #[serde(default)]
    pub direction: Option<String>,
    /// Maximum depth (default: 3, max: 10).
    #[serde(default)]
    pub depth: Option<u8>,
}

/// Parameters for cross-referencing terms across terminologies.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct MeshCrossrefParams {
    /// Term to cross-reference.
    pub term: String,
    /// Source terminology: "mesh", "meddra", "snomed", "ich".
    pub source: String,
    /// Target terminologies.
    pub targets: Vec<String>,
}

/// Parameters for PubMed MeSH enrichment.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct MeshEnrichPubmedParams {
    /// PubMed ID.
    pub pmid: String,
    /// Include qualifier terms.
    #[serde(default)]
    pub include_qualifiers: Option<bool>,
}

/// Parameters for terminology consistency check.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct MeshConsistencyParams {
    /// Terms to check for consistency.
    pub terms: Vec<String>,
    /// Corpora to check against: "mesh", "meddra", "snomed", "ich".
    pub corpora: Vec<String>,
}
