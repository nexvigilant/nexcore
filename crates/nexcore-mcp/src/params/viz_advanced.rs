//! Advanced Visualization Parameters — Phase 5 "Eyes" modules
//!
//! Typed parameter structs for the Phase 5 viz MCP tools:
//! - `viz_manifold_sample`  — Calabi-Yau manifold mesh generation
//! - `viz_string_modes`     — String vibration mode computation
//! - `viz_render_pipeline`  — WebGPU pipeline config / shader codegen
//! - `viz_orbital_density`  — Electron density grid computation

use rmcp::schemars::{self, JsonSchema};
use rmcp::serde::Deserialize;

// ────────────────────────────────────────────────────────────────
// viz_manifold_sample
// ────────────────────────────────────────────────────────────────

/// Parameters for Calabi-Yau manifold sampling and mesh generation.
///
/// Wraps `nexcore_viz::manifold::generate_manifold_mesh` plus
/// topology helpers (`compute_genus`, `mesh_surface_area`, `mesh_bounding_box`).
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct VizManifoldSampleParams {
    /// Degree `n` of the Fermat polynomial z₁ⁿ + z₂ⁿ = 1. Must be >= 2. Default: 5.
    #[serde(default)]
    pub degree: Option<u32>,
    /// Grid resolution per parameter axis (total vertices = resolution²). Default: 50.
    #[serde(default)]
    pub resolution: Option<usize>,
    /// Shape-deformation parameter α. Default: 1.0.
    #[serde(default)]
    pub alpha: Option<f64>,
    /// Surface-selector integer k₁ ∈ {0, …, n−1}. Default: 0.
    #[serde(default)]
    pub k1: Option<u32>,
    /// Surface-selector integer k₂ ∈ {0, …, n−1}. Default: 1.
    #[serde(default)]
    pub k2: Option<u32>,
    /// Uniform scale applied to projected coordinates. Default: 2.0.
    #[serde(default)]
    pub scale: Option<f64>,
    /// Projection method: "stereographic" (default) or "orthographic".
    #[serde(default)]
    pub projection: Option<String>,
}

// ────────────────────────────────────────────────────────────────
// viz_string_modes
// ────────────────────────────────────────────────────────────────

/// Parameters for string vibration mode computation.
///
/// Wraps `nexcore_viz::string_modes` — standing-wave patterns,
/// harmonic frequencies, mode spectrum, and node/antinode positions.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct VizStringModesParams {
    /// Physical length of the string (arbitrary units). Default: 1.0.
    #[serde(default)]
    pub length: Option<f64>,
    /// String tension T (force units). Default: 1.0.
    #[serde(default)]
    pub tension: Option<f64>,
    /// Linear mass density μ (mass per unit length). Default: 1.0.
    #[serde(default)]
    pub linear_density: Option<f64>,
    /// Number of sample points along the string. Default: 100.
    #[serde(default)]
    pub num_points: Option<usize>,
    /// Boundary condition: "open" (Dirichlet, default) or "closed" (periodic).
    #[serde(default)]
    pub string_type: Option<String>,
    /// Maximum allowed modes. Default: 20.
    #[serde(default)]
    pub max_modes: Option<usize>,
    /// Number of harmonic modes to include in the spectrum. Default: 10.
    #[serde(default)]
    pub n_modes: Option<u32>,
    /// Specific mode number to query nodes/antinodes for. If omitted, returns full spectrum only.
    #[serde(default)]
    pub query_mode: Option<u32>,
}

// ────────────────────────────────────────────────────────────────
// viz_render_pipeline
// ────────────────────────────────────────────────────────────────

/// Parameters for WebGPU rendering pipeline configuration and shader generation.
///
/// Wraps `nexcore_viz::renderer` — pipeline state, quality presets,
/// adaptive quality, and WGSL shader code generation.  No actual GPU required.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct VizRenderPipelineParams {
    /// Action to perform:
    /// - "pipeline"   — return default pipeline state
    /// - "quality"    — return quality settings for given level
    /// - "adaptive"   — compute adaptive quality level from current/target FPS
    /// - "shaders"    — generate WGSL shader source for requested passes
    /// - "sss_profile" — compute SSS diffusion profile for a material
    #[serde(default)]
    pub action: Option<String>,

    /// Quality level: "low", "medium", "high", "ultra". Used by "quality" and "adaptive" actions.
    #[serde(default)]
    pub quality_level: Option<String>,

    /// Current FPS for adaptive quality computation.
    #[serde(default)]
    pub current_fps: Option<f64>,

    /// Target FPS for adaptive quality computation. Default: 60.0.
    #[serde(default)]
    pub target_fps: Option<f64>,

    /// Comma-separated shader passes to generate: "vertex", "fragment", "taa", "gtao", "sss".
    /// Used by "shaders" action.
    #[serde(default)]
    pub shader_passes: Option<String>,

    /// SSS material preset: "skin", "wax", "marble", "jade", "milk".
    /// Used by "sss_profile" action.
    #[serde(default)]
    pub sss_material: Option<String>,

    /// Thickness value for SSS transmittance computation.
    #[serde(default)]
    pub sss_thickness: Option<f64>,
}

// ────────────────────────────────────────────────────────────────
// viz_orbital_density
// ────────────────────────────────────────────────────────────────

/// Parameters for molecular orbital electron density grid computation.
///
/// Wraps `nexcore_viz::orbital` — STO-3G basis sets, electron density
/// grid generation, and overlap matrix computation.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct VizOrbitalDensityParams {
    /// JSON array of atoms: `[{"element": "O", "position": [0,0,0]}, ...]`.
    /// Supported elements: H, C, N, O, F, P, S, Cl.
    pub atoms: String,

    /// Molecule name. Default: "molecule".
    #[serde(default)]
    pub name: Option<String>,

    /// Grid spacing in angstroms. Default: 0.3.
    #[serde(default)]
    pub grid_spacing: Option<f64>,

    /// Padding around molecular bounding box in angstroms. Default: 3.0.
    #[serde(default)]
    pub padding: Option<f64>,

    /// Maximum allowed grid points. Default: 500_000.
    #[serde(default)]
    pub max_grid_points: Option<usize>,

    /// Isosurface value for downstream mesh generation. Default: 0.02.
    #[serde(default)]
    pub isovalue: Option<f64>,

    /// If true, also compute and return the overlap matrix. Default: false.
    #[serde(default)]
    pub include_overlap: Option<bool>,
}
