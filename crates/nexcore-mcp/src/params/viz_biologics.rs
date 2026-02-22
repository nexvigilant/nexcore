//! Typed parameter structs for Phase 2 biologics visualization MCP tools.
//!
//! Covers antibody topology, molecular interactions, higher-dimensional
//! projection, protein structure analysis, and mesh topology.

use rmcp::schemars::JsonSchema;
use rmcp::serde::Deserialize;

// ============================================================================
// viz_antibody_structure
// ============================================================================

/// Parameters for the `viz_antibody_structure` tool.
///
/// Accepts a JSON-serialized `Molecule` and an optional disulfide distance
/// cutoff. Runs full antibody topology analysis: chain classification,
/// domain identification, CDR loop location, fragment mapping, and
/// disulfide bond detection.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct VizAntibodyStructureParams {
    /// JSON-serialized `nexcore_viz::molecular::Molecule`.
    pub molecule_json: String,
    /// Maximum S–S distance in angstroms for disulfide detection (default 2.5).
    pub disulfide_cutoff: Option<f64>,
}

// ============================================================================
// viz_interaction_map
// ============================================================================

/// Parameters for the `viz_interaction_map` tool.
///
/// Detects all non-covalent interactions (H-bonds, salt bridges, hydrophobic
/// contacts, pi-stacking, cation-pi, halogen bonds, metal coordination)
/// in a molecule with configurable distance/angle cutoffs.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct VizInteractionMapParams {
    /// JSON-serialized `nexcore_viz::molecular::Molecule`.
    pub molecule_json: String,
    /// Maximum donor–acceptor distance for H-bonds (default 3.5 Å).
    pub hbond_distance: Option<f64>,
    /// Minimum D-H···A angle for H-bonds in degrees (default 120°).
    pub hbond_angle_min: Option<f64>,
    /// Maximum distance for salt bridges (default 4.0 Å).
    pub salt_bridge_distance: Option<f64>,
    /// Maximum distance for hydrophobic contacts (default 4.5 Å).
    pub hydrophobic_distance: Option<f64>,
    /// Maximum ring-centre distance for pi-stacking (default 5.5 Å).
    pub pi_stack_distance: Option<f64>,
    /// Maximum inter-plane angle for pi-stacking in degrees (default 30°).
    pub pi_stack_angle_max: Option<f64>,
    /// Maximum cation–ring distance for cation-pi (default 6.0 Å).
    pub cation_pi_distance: Option<f64>,
    /// Maximum halogen–acceptor distance (default 3.5 Å).
    pub halogen_bond_distance: Option<f64>,
    /// Maximum metal–ligand distance (default 2.8 Å).
    pub metal_coord_distance: Option<f64>,
}

// ============================================================================
// viz_projection
// ============================================================================

/// Parameters for the `viz_projection` tool.
///
/// Projects nD points or parametric 4D surfaces down to 3D using
/// stereographic, perspective, or orthographic projection.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct VizProjectionParams {
    /// Projection method: "stereographic", "perspective", or "orthographic".
    pub method: String,
    /// Focal distance for perspective projection (required when method = "perspective").
    pub focal_distance: Option<f64>,
    /// Three axis indices for orthographic projection (required when method = "orthographic").
    /// JSON array of 3 integers, e.g. "[0, 1, 2]".
    pub axes: Option<String>,
    /// Mode: "points" to project raw nD points, or "surface" to generate and
    /// project a parametric 4D shape. Default: "points".
    pub mode: Option<String>,
    /// JSON array of nD points (each a JSON array of floats).
    /// Required when mode = "points".
    pub points: Option<String>,
    /// Surface type: "tesseract", "hypersphere", "klein_bottle", "clifford_torus".
    /// Required when mode = "surface".
    pub surface: Option<String>,
    /// Radius for hypersphere (default 1.0).
    pub radius: Option<f64>,
    /// Scale for Klein bottle (default 1.0).
    pub scale: Option<f64>,
    /// r1 for Clifford torus (default 1.0).
    pub r1: Option<f64>,
    /// r2 for Clifford torus (default 1.0).
    pub r2: Option<f64>,
    /// Tessellation segments for surface generation (default 16).
    pub segments: Option<usize>,
}

// ============================================================================
// viz_protein_structure
// ============================================================================

/// Parameters for the `viz_protein_structure` tool.
///
/// Computes protein structural metrics: backbone geometry, phi/psi dihedrals,
/// Ramachandran classification, secondary structure assignment, contact map,
/// hydrogen bonds, and radius of gyration.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct VizProteinStructureParams {
    /// JSON-serialized `nexcore_viz::molecular::Molecule`.
    pub molecule_json: String,
    /// Distance threshold for contact map (CA–CA, default 8.0 Å).
    pub contact_threshold: Option<f64>,
    /// Distance cutoff for backbone H-bond detection (default 3.5 Å).
    pub hbond_cutoff: Option<f64>,
}

// ============================================================================
// viz_topology_analysis
// ============================================================================

/// Parameters for the `viz_topology_analysis` tool.
///
/// Computes topological invariants, curvatures, and verifies the Gauss–Bonnet
/// theorem on a triangle mesh.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct VizTopologyAnalysisParams {
    /// JSON array of 3D vertex positions, e.g. `[[0,0,0],[1,0,0],[0,1,0]]`.
    pub vertices: String,
    /// JSON array of triangle index triplets (CCW winding), e.g. `[[0,1,2]]`.
    pub triangles: String,
    /// Whether to compute per-vertex curvatures (default true).
    pub compute_curvatures: Option<bool>,
    /// Whether to verify the Gauss–Bonnet theorem (default true).
    pub verify_gauss_bonnet: Option<bool>,
}
