//! Visualization Foundation Parameters
//!
//! Typed parameter structs for pre-phase nexcore-viz MCP tools:
//! molecular, surface, spectral, community, centrality, vdag.
//!
//! Tier: T3 (Domain-specific MCP tool parameters)

use rmcp::schemars::{self, JsonSchema};
use rmcp::serde::Deserialize;

// ─── Molecular ──────────────────────────────────────────────────────────────

/// Parameters for molecular element info lookup.
///
/// Returns atomic number, mass, VdW radius, covalent radius, and CPK color
/// for a single element symbol.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct VizMolecularInfoParams {
    /// Element symbol (e.g. "H", "C", "O", "Fe", "Cl").
    /// Case-sensitive — use standard periodic table symbols.
    pub symbol: String,
}

// ─── Surface ────────────────────────────────────────────────────────────────

/// Parameters for molecular surface mesh generation via marching cubes.
///
/// Accepts a JSON array of atoms (each with x, y, z, element) and generates
/// a triangle mesh for VdW or solvent-excluded surface.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct VizSurfaceMeshParams {
    /// JSON array of atoms: `[{"element": "O", "x": 0.0, "y": 0.0, "z": 0.0}, ...]`
    pub atoms_json: String,

    /// Surface type: "vdw" (Van der Waals) or "ses" (Solvent-Excluded Surface).
    /// Defaults to "vdw".
    #[serde(default = "default_surface_type")]
    pub surface_type: String,

    /// Probe radius in angstroms for SES (ignored for VdW). Default: 1.4
    #[serde(default = "default_probe_radius")]
    pub probe_radius: f64,

    /// Grid resolution in angstroms. Lower = more detail, slower. Default: 0.5
    #[serde(default = "default_resolution")]
    pub resolution: f64,
}

fn default_surface_type() -> String {
    "vdw".to_string()
}

fn default_probe_radius() -> f64 {
    1.4
}

fn default_resolution() -> f64 {
    0.5
}

// ─── Spectral ───────────────────────────────────────────────────────────────

/// Parameters for spectral graph analysis.
///
/// Computes adjacency matrix, Laplacian, algebraic connectivity (Fiedler value),
/// and power iteration on a graph specified by node IDs and edge pairs.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct VizSpectralAnalysisParams {
    /// JSON array of node ID strings: `["a", "b", "c"]`
    pub nodes_json: String,

    /// JSON array of edge pairs: `[["a","b"], ["b","c"]]`
    pub edges_json: String,
}

// ─── Community ──────────────────────────────────────────────────────────────

/// Parameters for Louvain community detection.
///
/// Runs the Louvain algorithm and returns community assignments with modularity.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct VizCommunityDetectParams {
    /// JSON array of node ID strings: `["a", "b", "c", "x", "y", "z"]`
    pub nodes_json: String,

    /// JSON array of edge pairs: `[["a","b"], ["b","c"], ["c","x"]]`
    pub edges_json: String,
}

// ─── Centrality ─────────────────────────────────────────────────────────────

/// Parameters for graph centrality computation.
///
/// Computes degree, betweenness, closeness, or eigenvector centrality
/// (or all four at once).
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct VizCentralityParams {
    /// JSON array of node ID strings: `["hub", "a", "b", "c"]`
    pub nodes_json: String,

    /// JSON array of edge pairs: `[["hub","a"], ["hub","b"]]`
    pub edges_json: String,

    /// Centrality metric: "degree", "betweenness", "closeness", "eigenvector", or "all".
    /// Default: "all"
    #[serde(default = "default_centrality_metric")]
    pub metric: String,
}

fn default_centrality_metric() -> String {
    "all".to_string()
}

// ─── VDAG ───────────────────────────────────────────────────────────────────

/// Parameters for VDAG (Validated DAG) overlay creation and querying.
///
/// Builds a pharmacovigilance knowledge graph from drug classes and AE signals,
/// then returns a summary with signal statistics, SVG rendering, and 3D layout.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct VizVdagOverlayParams {
    /// JSON array of drug class nodes:
    /// ```json
    /// [
    ///   {"id": "analgesics", "label": "Analgesics (N02)", "node_type": "drug_class"},
    ///   {"id": "aspirin", "label": "Aspirin", "node_type": "drug"}
    /// ]
    /// ```
    /// Supported `node_type` values: "drug_class", "drug", "adverse_event", "indication"
    pub nodes_json: String,

    /// JSON array of edges:
    /// ```json
    /// [
    ///   {"from": "analgesics", "to": "aspirin", "edge_type": "contains"}
    /// ]
    /// ```
    /// Supported `edge_type` values: "contains", "interacts_with", "contraindicates",
    /// "class_of", "has_adverse_event"
    pub edges_json: String,

    /// JSON array of AE signal scores to attach to drug nodes:
    /// ```json
    /// [
    ///   {"drug_id": "aspirin", "ae_name": "GI Bleeding", "prr": 3.2, "ror": 3.5,
    ///    "ic025": 0.8, "ebgm": 2.9, "case_count": 412}
    /// ]
    /// ```
    #[serde(default)]
    pub signals_json: Option<String>,

    /// Graph title. Default: "VDAG Overlay"
    #[serde(default = "default_vdag_title")]
    pub title: String,
}

fn default_vdag_title() -> String {
    "VDAG Overlay".to_string()
}
