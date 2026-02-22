//! Visualization Physics Parameters (dynamics, force_field, gpu_layout, hypergraph, lod, minimizer, particle, ae_overlay, coord_gen, bipartite)
//! Tier: T3 (Domain-specific MCP tool parameters)
//!
//! Phase 3 (Physics Engine) + Phase 4 (Nervous System) nexcore-viz modules.

use rmcp::schemars::{self, JsonSchema};
use rmcp::serde::{Deserialize, Serialize};

// ============================================================================
// dynamics — Molecular dynamics simulation
// ============================================================================

/// Parameters for running a molecular dynamics simulation step.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct VizDynamicsStepParams {
    /// Atom element symbols (e.g. ["O", "H", "H"])
    pub elements: Vec<String>,
    /// Atom positions as flat [x,y,z, x,y,z, ...] array
    pub positions: Vec<f64>,
    /// Bond pairs as [atom1, atom2, atom1, atom2, ...] (0-indexed)
    pub bonds: Vec<usize>,
    /// Integration timestep in femtoseconds (default: 1.0)
    #[serde(default = "default_timestep")]
    pub timestep: f64,
    /// Target temperature in Kelvin (default: 300.0)
    #[serde(default = "default_temperature")]
    pub temperature: f64,
    /// Number of simulation steps (default: 10)
    #[serde(default = "default_steps")]
    pub total_steps: usize,
}

fn default_timestep() -> f64 {
    1.0
}
fn default_temperature() -> f64 {
    300.0
}
fn default_steps() -> usize {
    10
}

// ============================================================================
// force_field — Energy computation
// ============================================================================

/// Parameters for computing molecular force field energy.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct VizForceFieldEnergyParams {
    /// Atom element symbols (e.g. ["O", "H", "H"])
    pub elements: Vec<String>,
    /// Atom positions as flat [x,y,z, x,y,z, ...] array
    pub positions: Vec<f64>,
    /// Bond pairs as [atom1, atom2, atom1, atom2, ...] (0-indexed)
    pub bonds: Vec<usize>,
    /// Non-bonded cutoff distance in angstroms (default: 10.0)
    #[serde(default = "default_cutoff")]
    pub cutoff: Option<f64>,
}

fn default_cutoff() -> Option<f64> {
    Some(10.0)
}

// ============================================================================
// gpu_layout — Force-directed graph layout
// ============================================================================

/// Parameters for force-directed graph layout computation.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct VizGpuLayoutParams {
    /// Number of nodes
    pub node_count: usize,
    /// Edge list as [source, target, source, target, ...] (0-indexed)
    pub edges: Vec<usize>,
    /// Optional edge weights (one per edge pair)
    #[serde(default)]
    pub weights: Option<Vec<f64>>,
    /// Canvas width (default: 800.0)
    #[serde(default = "default_width")]
    pub width: f64,
    /// Canvas height (default: 600.0)
    #[serde(default = "default_height")]
    pub height: f64,
    /// Maximum iterations (default: 300)
    #[serde(default = "default_iterations")]
    pub iterations: usize,
}

fn default_width() -> f64 {
    800.0
}
fn default_height() -> f64 {
    600.0
}
fn default_iterations() -> usize {
    300
}

// ============================================================================
// hypergraph — Hypergraph metrics and operations
// ============================================================================

/// Parameters for hypergraph construction and analysis.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct VizHypergraphParams {
    /// Node labels
    pub node_labels: Vec<String>,
    /// Hyperedges as JSON array of arrays of node indices, e.g. "[[0,1,2],[1,3]]"
    pub hyperedges: String,
    /// Whether the hypergraph is directed (default: false)
    #[serde(default)]
    pub directed: bool,
}

// ============================================================================
// lod — Level-of-detail selection
// ============================================================================

/// Parameters for LOD level selection based on atom count.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct VizLodSelectParams {
    /// Number of atoms in the molecule
    pub atom_count: usize,
    /// Optional atom count thresholds: [full_atom, calpha, secondary, blob, hull]
    #[serde(default)]
    pub thresholds: Option<Vec<usize>>,
}

// ============================================================================
// minimizer — Energy minimization
// ============================================================================

/// Parameters for energy minimization of a molecular structure.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct VizMinimizeEnergyParams {
    /// Atom element symbols
    pub elements: Vec<String>,
    /// Atom positions as flat [x,y,z, ...] array
    pub positions: Vec<f64>,
    /// Bond pairs as [atom1, atom2, ...] (0-indexed)
    pub bonds: Vec<usize>,
    /// Maximum minimization steps (default: 100)
    #[serde(default = "default_min_steps")]
    pub max_steps: usize,
    /// Convergence tolerance (default: 0.01)
    #[serde(default = "default_tolerance")]
    pub tolerance: f64,
}

fn default_min_steps() -> usize {
    100
}
fn default_tolerance() -> f64 {
    0.01
}

// ============================================================================
// particle — Particle system preset query
// ============================================================================

/// Parameters for particle system preset configuration.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct VizParticlePresetParams {
    /// Preset type: "solvent", "reaction", or "electron"
    pub preset: String,
    /// Center position [x, y, z] (default: [0,0,0])
    #[serde(default)]
    pub center: Option<[f64; 3]>,
    /// Radius (for solvent) or orbital_radius (for electron) (default: 2.0)
    #[serde(default)]
    pub radius: Option<f64>,
}

// ============================================================================
// ae_overlay — AE signal heatmap computation
// ============================================================================

/// Parameters for computing an AE signal overlay heatmap.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct VizAeOverlayParams {
    /// JSON array of drug nodes with signals. Each object:
    /// { "id": "drug_id", "label": "Drug Name", "signals": [
    ///   { "ae_name": "AE", "prr": 3.2, "ror": 3.5, "ic025": 0.8, "ebgm": 2.9, "case_count": 412 }
    /// ]}
    pub drugs: String,
    /// Score field to use: "prr", "ror", "ic025", or "ebgm" (default: "prr")
    #[serde(default = "default_score_field")]
    pub score_field: String,
    /// Normalization method: "min_max", "z_score", "percentile", or "log" (default: "min_max")
    #[serde(default = "default_normalization")]
    pub normalization: String,
}

fn default_score_field() -> String {
    "prr".to_string()
}
fn default_normalization() -> String {
    "min_max".to_string()
}

// ============================================================================
// coord_gen — Distance geometry coordinate generation
// ============================================================================

/// Parameters for generating 3D coordinates via distance geometry.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct VizCoordGenParams {
    /// Atom element symbols
    pub elements: Vec<String>,
    /// Bond pairs as [atom1, atom2, ...] (0-indexed)
    pub bonds: Vec<usize>,
    /// Random seed for reproducibility (default: 42)
    #[serde(default = "default_seed")]
    pub seed: u64,
    /// Maximum refinement iterations (default: 200)
    #[serde(default = "default_refine_iters")]
    pub max_iterations: usize,
}

fn default_seed() -> u64 {
    42
}
fn default_refine_iters() -> usize {
    200
}

// ============================================================================
// bipartite — Drug-AE bipartite network layout
// ============================================================================

/// Parameters for bipartite drug-AE network layout.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct VizBipartiteLayoutParams {
    /// JSON VDAG definition with nodes and edges:
    /// { "nodes": [...], "edges": [...], "title": "..." }
    pub vdag_json: String,
}
