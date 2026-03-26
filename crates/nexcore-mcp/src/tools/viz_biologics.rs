//! MCP tool wrappers for Phase 2 biologics visualization modules.
//!
//! Five tools wrapping `nexcore_viz` biologics analysis:
//! - `viz_antibody_structure` — antibody topology (chains, domains, CDRs, fragments)
//! - `viz_interaction_map` — non-covalent molecular interaction detection
//! - `viz_projection` — nD-to-3D projection (stereographic/perspective/orthographic)
//! - `viz_protein_structure` — protein structural metrics and geometry
//! - `viz_topology_analysis` — mesh topological invariants and curvature

use rmcp::ErrorData as McpError;
use rmcp::model::CallToolResult;

use crate::params::{
    VizAntibodyStructureParams, VizInteractionMapParams, VizProjectionParams,
    VizProteinStructureParams, VizTopologyAnalysisParams,
};

// ============================================================================
// Helpers
// ============================================================================

/// Build a success result containing a single JSON text content block.
fn json_success(value: &impl serde::Serialize) -> Result<CallToolResult, McpError> {
    let json = serde_json::to_string_pretty(value)
        .map_err(|e| McpError::invalid_params(format!("serialization failed: {e}"), None))?;
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        json.to_string(),
    )]))
}

/// Parse a JSON string into a typed value, mapping errors to `McpError`.
fn parse_json<T: serde::de::DeserializeOwned>(raw: &str, label: &str) -> Result<T, McpError> {
    serde_json::from_str(raw)
        .map_err(|e| McpError::invalid_params(format!("invalid {label} JSON: {e}"), None))
}

// ============================================================================
// viz_antibody_structure
// ============================================================================

/// Analyze antibody/immunoglobulin topology: chain classification, domain
/// identification (VH/VL/CH1-3/CL/Hinge), CDR loop location (Kabat),
/// Fab/Fc fragment mapping, and disulfide bond detection.
pub fn antibody_structure(params: VizAntibodyStructureParams) -> Result<CallToolResult, McpError> {
    let mol: nexcore_viz::molecular::Molecule = parse_json(&params.molecule_json, "molecule")?;

    let topology = nexcore_viz::antibody::analyze_antibody(&mol);

    json_success(&topology)
}

// ============================================================================
// viz_interaction_map
// ============================================================================

/// Detect all non-covalent molecular interactions: hydrogen bonds, salt
/// bridges, hydrophobic contacts, pi-stacking, cation-pi, halogen bonds,
/// and metal coordination. Returns counts, energies, and individual
/// interaction details.
pub fn interaction_map(params: VizInteractionMapParams) -> Result<CallToolResult, McpError> {
    let mol: nexcore_viz::molecular::Molecule = parse_json(&params.molecule_json, "molecule")?;

    let mut cutoffs = nexcore_viz::interaction::InteractionCutoffs::default();

    if let Some(v) = params.hbond_distance {
        cutoffs.hbond_distance = v;
    }
    if let Some(v) = params.hbond_angle_min {
        cutoffs.hbond_angle_min = v;
    }
    if let Some(v) = params.salt_bridge_distance {
        cutoffs.salt_bridge_distance = v;
    }
    if let Some(v) = params.hydrophobic_distance {
        cutoffs.hydrophobic_distance = v;
    }
    if let Some(v) = params.pi_stack_distance {
        cutoffs.pi_stack_distance = v;
    }
    if let Some(v) = params.pi_stack_angle_max {
        cutoffs.pi_stack_angle_max = v;
    }
    if let Some(v) = params.cation_pi_distance {
        cutoffs.cation_pi_distance = v;
    }
    if let Some(v) = params.halogen_bond_distance {
        cutoffs.halogen_bond_distance = v;
    }
    if let Some(v) = params.metal_coord_distance {
        cutoffs.metal_coord_distance = v;
    }

    let summary = nexcore_viz::interaction::detect_all_interactions(&mol, &cutoffs);

    json_success(&summary)
}

// ============================================================================
// viz_projection
// ============================================================================

/// Project nD points or parametric 4D surfaces down to 3D.
///
/// Supports two modes:
/// - `"points"` — provide raw nD coordinates, receive projected 3D points.
/// - `"surface"` — generate a named 4D shape (tesseract, hypersphere,
///   Klein bottle, Clifford torus) and project it.
pub fn projection(params: VizProjectionParams) -> Result<CallToolResult, McpError> {
    let method = match params.method.to_lowercase().as_str() {
        "stereographic" => nexcore_viz::projection::ProjectionMethod::Stereographic,
        "perspective" => {
            let fd = params.focal_distance.unwrap_or(3.0);
            nexcore_viz::projection::ProjectionMethod::Perspective { focal_distance: fd }
        }
        "orthographic" => {
            let axes_str = params.axes.as_deref().unwrap_or("[0, 1, 2]");
            let axes_vec: Vec<usize> = parse_json(axes_str, "axes")?;
            if axes_vec.len() < 3 {
                return Err(McpError::invalid_params(
                    "orthographic projection requires exactly 3 axis indices".to_string(),
                    None,
                ));
            }
            nexcore_viz::projection::ProjectionMethod::Orthographic {
                axes: [axes_vec[0], axes_vec[1], axes_vec[2]],
            }
        }
        other => {
            return Err(McpError::invalid_params(
                format!(
                    "unknown projection method: {other}. Use: stereographic, perspective, orthographic"
                ),
                None,
            ));
        }
    };

    let mode = params.mode.as_deref().unwrap_or("points");

    match mode {
        "points" => {
            let points_str = params.points.as_deref().unwrap_or("[]");
            let points: Vec<Vec<f64>> = parse_json(points_str, "points")?;

            let projected = nexcore_viz::projection::project_points(&points, &method)
                .map_err(|e| McpError::invalid_params(format!("projection error: {e}"), None))?;

            json_success(&projected)
        }
        "surface" => {
            let surface_name = params.surface.as_deref().unwrap_or("tesseract");
            let segments = params.segments.unwrap_or(16);

            let surface = match surface_name {
                "tesseract" => nexcore_viz::projection::ParametricSurface::Tesseract,
                "hypersphere" => nexcore_viz::projection::ParametricSurface::Hypersphere {
                    radius: params.radius.unwrap_or(1.0),
                },
                "klein_bottle" | "klein" => {
                    nexcore_viz::projection::ParametricSurface::KleinBottle {
                        scale: params.scale.unwrap_or(1.0),
                    }
                }
                "clifford_torus" | "clifford" => {
                    nexcore_viz::projection::ParametricSurface::CliffordTorus {
                        r1: params.r1.unwrap_or(1.0),
                        r2: params.r2.unwrap_or(1.0),
                    }
                }
                other => {
                    return Err(McpError::invalid_params(
                        format!(
                            "unknown surface: {other}. Use: tesseract, hypersphere, klein_bottle, clifford_torus"
                        ),
                        None,
                    ));
                }
            };

            let mesh = nexcore_viz::projection::project_surface(&surface, segments, &method)
                .map_err(|e| {
                    McpError::invalid_params(format!("surface projection error: {e}"), None)
                })?;

            json_success(&mesh)
        }
        other => Err(McpError::invalid_params(
            format!("unknown mode: {other}. Use: points, surface"),
            None,
        )),
    }
}

// ============================================================================
// viz_protein_structure
// ============================================================================

/// Compute protein structural metrics: backbone extraction, phi/psi dihedral
/// angles, Ramachandran classification, secondary structure assignment,
/// contact map, hydrogen bonds, radius of gyration, and aggregate metrics.
pub fn protein_structure(params: VizProteinStructureParams) -> Result<CallToolResult, McpError> {
    let mol: nexcore_viz::molecular::Molecule = parse_json(&params.molecule_json, "molecule")?;

    let contact_threshold = params.contact_threshold.unwrap_or(8.0);
    let hbond_cutoff = params.hbond_cutoff.unwrap_or(3.5);

    // Core analysis pipeline
    let backbone = nexcore_viz::protein::extract_backbone(&mol);
    let dihedrals = nexcore_viz::protein::compute_phi_psi(&mol, &backbone);
    let hbonds = nexcore_viz::protein::detect_hydrogen_bonds(&mol, hbond_cutoff);
    let secondary = nexcore_viz::protein::assign_secondary_structure(&mol, &hbonds);
    let contact_map = nexcore_viz::protein::compute_contact_map(&mol, contact_threshold);
    let metrics = nexcore_viz::protein::compute_protein_metrics(&mol);

    // Classify each dihedral into Ramachandran regions
    let ramachandran: Vec<serde_json::Value> = dihedrals
        .iter()
        .filter_map(|d| {
            let phi = d.phi?;
            let psi = d.psi?;
            let region = nexcore_viz::protein::classify_ramachandran(phi, psi);
            Some(serde_json::json!({
                "residue_index": d.residue_index,
                "phi": phi,
                "psi": psi,
                "region": format!("{region:?}"),
            }))
        })
        .collect();

    let result = serde_json::json!({
        "metrics": metrics,
        "backbone_count": backbone.len(),
        "dihedral_count": dihedrals.len(),
        "hbond_count": hbonds.len(),
        "contact_map_entries": contact_map.len(),
        "secondary_structure": secondary.iter()
            .map(|ss| format!("{ss:?}"))
            .collect::<Vec<_>>(),
        "ramachandran": ramachandran,
    });

    json_success(&result)
}

// ============================================================================
// viz_topology_analysis
// ============================================================================

/// Compute topological invariants (Euler characteristic, genus, Betti numbers),
/// per-vertex Gaussian and mean curvatures, and verify the Gauss–Bonnet
/// theorem on a triangle mesh.
pub fn topology_analysis(params: VizTopologyAnalysisParams) -> Result<CallToolResult, McpError> {
    let vertices: Vec<[f64; 3]> = parse_json(&params.vertices, "vertices")?;
    let triangles: Vec<[usize; 3]> = parse_json(&params.triangles, "triangles")?;

    let mesh = nexcore_viz::topology::TriangleMesh {
        vertices,
        triangles,
    };

    // Invariants
    let invariants = nexcore_viz::topology::compute_invariants(&mesh)
        .map_err(|e| McpError::invalid_params(format!("topology error: {e}"), None))?;

    let compute_curvatures = params.compute_curvatures.unwrap_or(true);
    let verify_gb = params.verify_gauss_bonnet.unwrap_or(true);

    let mut result = serde_json::json!({
        "invariants": invariants,
    });

    // Curvatures (optional)
    if compute_curvatures {
        let curvatures = nexcore_viz::topology::compute_all_curvatures(&mesh)
            .map_err(|e| McpError::invalid_params(format!("curvature error: {e}"), None))?;

        if verify_gb {
            let gb_result = nexcore_viz::topology::verify_gauss_bonnet(&mesh, &curvatures);
            result["gauss_bonnet"] = serde_json::json!(gb_result);
        }

        result["curvatures"] = serde_json::json!(curvatures);
    }

    json_success(&result)
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper: empty molecule JSON for testing.
    fn empty_mol_json() -> String {
        r#"{"name":"test","atoms":[],"bonds":[],"chains":[],"source_format":null}"#.to_string()
    }

    /// Helper: tetrahedron mesh vertices + triangles.
    fn tetrahedron_vertices() -> String {
        "[[0,0,0],[1,0,0],[0.5,1,0],[0.5,0.5,1]]".to_string()
    }

    fn tetrahedron_triangles() -> String {
        "[[0,1,2],[0,1,3],[0,2,3],[1,2,3]]".to_string()
    }

    #[test]
    fn antibody_structure_empty_molecule() {
        let params = VizAntibodyStructureParams {
            molecule_json: empty_mol_json(),
            disulfide_cutoff: None,
        };
        let result = antibody_structure(params);
        assert!(result.is_ok());
    }

    #[test]
    fn antibody_structure_invalid_json() {
        let params = VizAntibodyStructureParams {
            molecule_json: "not json".to_string(),
            disulfide_cutoff: None,
        };
        let result = antibody_structure(params);
        assert!(result.is_err());
    }

    #[test]
    fn interaction_map_empty_molecule() {
        let params = VizInteractionMapParams {
            molecule_json: empty_mol_json(),
            hbond_distance: None,
            hbond_angle_min: None,
            salt_bridge_distance: None,
            hydrophobic_distance: None,
            pi_stack_distance: None,
            pi_stack_angle_max: None,
            cation_pi_distance: None,
            halogen_bond_distance: None,
            metal_coord_distance: None,
        };
        let result = interaction_map(params);
        assert!(result.is_ok());
    }

    #[test]
    fn interaction_map_custom_cutoffs() {
        let params = VizInteractionMapParams {
            molecule_json: empty_mol_json(),
            hbond_distance: Some(4.0),
            hbond_angle_min: Some(110.0),
            salt_bridge_distance: Some(5.0),
            hydrophobic_distance: Some(5.0),
            pi_stack_distance: Some(6.0),
            pi_stack_angle_max: Some(35.0),
            cation_pi_distance: Some(7.0),
            halogen_bond_distance: Some(4.0),
            metal_coord_distance: Some(3.0),
        };
        let result = interaction_map(params);
        assert!(result.is_ok());
    }

    #[test]
    fn projection_stereographic_points() {
        let params = VizProjectionParams {
            method: "stereographic".to_string(),
            focal_distance: None,
            axes: None,
            mode: Some("points".to_string()),
            points: Some("[[1.0, 0.0, 0.0, 2.0]]".to_string()),
            surface: None,
            radius: None,
            scale: None,
            r1: None,
            r2: None,
            segments: None,
        };
        let result = projection(params);
        assert!(result.is_ok());
    }

    #[test]
    fn projection_perspective_surface_tesseract() {
        let params = VizProjectionParams {
            method: "perspective".to_string(),
            focal_distance: Some(3.0),
            axes: None,
            mode: Some("surface".to_string()),
            points: None,
            surface: Some("tesseract".to_string()),
            radius: None,
            scale: None,
            r1: None,
            r2: None,
            segments: Some(8),
        };
        let result = projection(params);
        assert!(result.is_ok());
    }

    #[test]
    fn projection_orthographic() {
        let params = VizProjectionParams {
            method: "orthographic".to_string(),
            focal_distance: None,
            axes: Some("[0, 1, 2]".to_string()),
            mode: Some("points".to_string()),
            points: Some("[[1.0, 2.0, 3.0, 4.0]]".to_string()),
            surface: None,
            radius: None,
            scale: None,
            r1: None,
            r2: None,
            segments: None,
        };
        let result = projection(params);
        assert!(result.is_ok());
    }

    #[test]
    fn projection_unknown_method() {
        let params = VizProjectionParams {
            method: "fish_eye".to_string(),
            focal_distance: None,
            axes: None,
            mode: None,
            points: Some("[[1.0, 2.0, 3.0]]".to_string()),
            surface: None,
            radius: None,
            scale: None,
            r1: None,
            r2: None,
            segments: None,
        };
        let result = projection(params);
        assert!(result.is_err());
    }

    #[test]
    fn protein_structure_empty_molecule() {
        let params = VizProteinStructureParams {
            molecule_json: empty_mol_json(),
            contact_threshold: None,
            hbond_cutoff: None,
        };
        let result = protein_structure(params);
        assert!(result.is_ok());
    }

    #[test]
    fn protein_structure_custom_thresholds() {
        let params = VizProteinStructureParams {
            molecule_json: empty_mol_json(),
            contact_threshold: Some(10.0),
            hbond_cutoff: Some(4.0),
        };
        let result = protein_structure(params);
        assert!(result.is_ok());
    }

    #[test]
    fn topology_analysis_tetrahedron() {
        let params = VizTopologyAnalysisParams {
            vertices: tetrahedron_vertices(),
            triangles: tetrahedron_triangles(),
            compute_curvatures: Some(true),
            verify_gauss_bonnet: Some(true),
        };
        let result = topology_analysis(params);
        assert!(result.is_ok());
    }

    #[test]
    fn topology_analysis_no_curvatures() {
        let params = VizTopologyAnalysisParams {
            vertices: tetrahedron_vertices(),
            triangles: tetrahedron_triangles(),
            compute_curvatures: Some(false),
            verify_gauss_bonnet: Some(false),
        };
        let result = topology_analysis(params);
        assert!(result.is_ok());
    }

    #[test]
    fn topology_analysis_invalid_vertices() {
        let params = VizTopologyAnalysisParams {
            vertices: "not json".to_string(),
            triangles: "[[0,1,2]]".to_string(),
            compute_curvatures: None,
            verify_gauss_bonnet: None,
        };
        let result = topology_analysis(params);
        assert!(result.is_err());
    }
}
