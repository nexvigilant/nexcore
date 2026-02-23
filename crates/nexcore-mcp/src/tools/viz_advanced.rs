//! Advanced Visualization MCP tools — Phase 5 "Eyes"
//!
//! Provides MCP tool wrappers for the four Phase 5 nexcore-viz modules:
//! - `viz_manifold_sample`  — Calabi-Yau manifold mesh generation
//! - `viz_string_modes`     — String vibration mode computation
//! - `viz_render_pipeline`  — Pipeline config / quality / WGSL shaders
//! - `viz_orbital_density`  — Electron density grid computation

use rmcp::ErrorData as McpError;
use rmcp::model::CallToolResult;

use crate::params::{
    VizManifoldSampleParams, VizOrbitalDensityParams, VizRenderPipelineParams, VizStringModesParams,
};

// ════════════════════════════════════════════════════════════════
// viz_manifold_sample
// ════════════════════════════════════════════════════════════════

/// Generate a Calabi-Yau manifold mesh with topology metrics.
///
/// Returns: vertex/triangle counts, genus, surface area, bounding box,
/// and sample surface points with normals and curvature.
pub fn manifold_sample(params: VizManifoldSampleParams) -> Result<CallToolResult, McpError> {
    use nexcore_viz::manifold::{self, CalabiYauConfig, ProjectionMethod};

    let projection = match params
        .projection
        .as_deref()
        .unwrap_or("stereographic")
        .to_lowercase()
        .as_str()
    {
        "stereographic" => ProjectionMethod::Stereographic,
        "orthographic" => ProjectionMethod::Orthographic,
        other => {
            return Err(McpError::invalid_params(
                format!("Unknown projection method: {other}. Use: stereographic, orthographic"),
                None,
            ));
        }
    };

    let config = CalabiYauConfig {
        degree: params.degree.unwrap_or(5),
        resolution: params.resolution.unwrap_or(50),
        alpha: params.alpha.unwrap_or(1.0),
        k1: params.k1.unwrap_or(0),
        k2: params.k2.unwrap_or(1),
        scale: params.scale.unwrap_or(2.0),
        projection_method: projection,
    };

    let mesh = manifold::generate_manifold_mesh(&config)
        .map_err(|e| McpError::invalid_params(format!("Manifold generation failed: {e}"), None))?;

    let genus = manifold::compute_genus(config.degree);
    let area = manifold::mesh_surface_area(&mesh);
    let (bb_min, bb_max) = manifold::mesh_bounding_box(&mesh);

    // Sample a few surface points with normals and curvature
    let sample_thetas = [0.3, 0.7, 1.0];
    let sample_phi = 0.5;
    let mut sample_points = Vec::with_capacity(sample_thetas.len());
    for &theta in &sample_thetas {
        let pt = manifold::evaluate_surface_point(theta, sample_phi, sample_phi, &config);
        let normal = manifold::compute_surface_normal(theta, sample_phi, sample_phi, &config);
        let curvature = manifold::estimate_curvature(theta, sample_phi, sample_phi, &config);
        sample_points.push(serde_json::json!({
            "theta": theta,
            "point": pt,
            "normal": normal,
            "curvature": curvature,
        }));
    }

    let result = serde_json::json!({
        "degree": config.degree,
        "resolution": config.resolution,
        "projection": format!("{:?}", config.projection_method),
        "vertices": mesh.vertices.len(),
        "triangles": mesh.triangles.len(),
        "genus": genus,
        "mesh_genus": mesh.genus,
        "surface_area": area,
        "bounding_box": {
            "min": bb_min,
            "max": bb_max,
        },
        "sample_points": sample_points,
    });

    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_default(),
    )]))
}

// ════════════════════════════════════════════════════════════════
// viz_string_modes
// ════════════════════════════════════════════════════════════════

/// Compute string vibration mode spectrum, frequencies, and standing-wave features.
///
/// Returns: fundamental frequency, wave speed, mode spectrum, and
/// optionally nodes/antinodes for a specific mode.
pub fn string_modes(params: VizStringModesParams) -> Result<CallToolResult, McpError> {
    use nexcore_viz::string_modes::{self, StringConfig, StringType};

    let string_type = match params
        .string_type
        .as_deref()
        .unwrap_or("open")
        .to_lowercase()
        .as_str()
    {
        "open" => StringType::Open,
        "closed" => StringType::Closed,
        other => {
            return Err(McpError::invalid_params(
                format!("Unknown string type: {other}. Use: open, closed"),
                None,
            ));
        }
    };

    let config = StringConfig {
        length: params.length.unwrap_or(1.0),
        tension: params.tension.unwrap_or(1.0),
        linear_density: params.linear_density.unwrap_or(1.0),
        num_points: params.num_points.unwrap_or(100),
        string_type,
        max_modes: params.max_modes.unwrap_or(20),
    };

    let wave_speed = string_modes::wave_speed(&config);
    let fundamental = string_modes::fundamental_frequency(&config);
    let n_modes = params.n_modes.unwrap_or(10);
    let spectrum = string_modes::mode_spectrum(&config, n_modes);

    let spectrum_json: Vec<serde_json::Value> = spectrum
        .iter()
        .map(|s| {
            serde_json::json!({
                "mode_number": s.mode_number,
                "frequency": s.frequency,
                "energy": s.energy,
                "wavelength": s.wavelength,
            })
        })
        .collect();

    let mut result = serde_json::json!({
        "string_type": format!("{:?}", config.string_type),
        "length": config.length,
        "tension": config.tension,
        "linear_density": config.linear_density,
        "wave_speed": wave_speed,
        "fundamental_frequency": fundamental,
        "spectrum": spectrum_json,
    });

    // Optionally query nodes/antinodes for a specific mode
    if let Some(mode_num) = params.query_mode {
        let nodes = string_modes::standing_wave_nodes(mode_num, &config);
        let antinodes = string_modes::standing_wave_antinodes(mode_num, &config);
        let mode_freq = string_modes::mode_frequency(mode_num, &config);

        if let Some(obj) = result.as_object_mut() {
            obj.insert(
                "query_mode".to_string(),
                serde_json::json!({
                    "mode_number": mode_num,
                    "frequency": mode_freq,
                    "nodes": nodes,
                    "antinodes": antinodes,
                    "node_count": nodes.len(),
                    "antinode_count": antinodes.len(),
                }),
            );
        }
    }

    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_default(),
    )]))
}

// ════════════════════════════════════════════════════════════════
// viz_render_pipeline
// ════════════════════════════════════════════════════════════════

/// Query WebGPU rendering pipeline configuration, quality settings,
/// and generate WGSL shader source code.  No actual GPU required.
pub fn render_pipeline(params: VizRenderPipelineParams) -> Result<CallToolResult, McpError> {
    use nexcore_viz::renderer::{self, QualityLevel, SssMaterial};

    let action = params.action.as_deref().unwrap_or("pipeline");

    let result = match action.to_lowercase().as_str() {
        "pipeline" => {
            let ps = renderer::default_pipeline();
            serde_json::json!({
                "action": "pipeline",
                "blend_mode": format!("{:?}", ps.blend_mode),
                "depth_func": format!("{:?}", ps.depth_func),
                "depth_write": ps.depth_write,
                "cull_mode": format!("{:?}", ps.cull_mode),
                "sample_count": ps.sample_count,
            })
        }

        "quality" => {
            let level = parse_quality_level(params.quality_level.as_deref().unwrap_or("high"))?;
            let qs = renderer::quality_settings(level.clone());
            serde_json::json!({
                "action": "quality",
                "level": format!("{level:?}"),
                "shadow_map_size": qs.shadow_map_size,
                "ao_samples": qs.ao_samples,
                "aa_method": format!("{:?}", qs.aa_method),
                "sss_enabled": qs.sss_enabled,
                "max_lights": qs.max_lights,
            })
        }

        "adaptive" => {
            let current_fps = params.current_fps.ok_or_else(|| {
                McpError::invalid_params(
                    "current_fps is required for adaptive action".to_string(),
                    None,
                )
            })?;
            let target_fps = params.target_fps.unwrap_or(60.0);
            let current_level =
                parse_quality_level(params.quality_level.as_deref().unwrap_or("high"))?;
            let new_level =
                renderer::adaptive_quality(current_fps, target_fps, current_level.clone());
            serde_json::json!({
                "action": "adaptive",
                "current_fps": current_fps,
                "target_fps": target_fps,
                "current_level": format!("{current_level:?}"),
                "recommended_level": format!("{new_level:?}"),
            })
        }

        "shaders" => {
            let passes = params.shader_passes.as_deref().unwrap_or("vertex,fragment");
            let mut shaders = serde_json::Map::new();
            for pass in passes.split(',').map(|s| s.trim()) {
                let code = match pass.to_lowercase().as_str() {
                    "vertex" => renderer::generate_molecular_vertex_shader(),
                    "fragment" => renderer::generate_molecular_fragment_shader(),
                    "taa" => renderer::generate_taa_resolve_shader(),
                    "gtao" => renderer::generate_gtao_shader(),
                    "sss" => renderer::generate_sss_shader(),
                    other => {
                        return Err(McpError::invalid_params(
                            format!(
                                "Unknown shader pass: {other}. \
                                 Use: vertex, fragment, taa, gtao, sss"
                            ),
                            None,
                        ));
                    }
                };
                shaders.insert(pass.to_string(), serde_json::Value::String(code));
            }
            serde_json::json!({
                "action": "shaders",
                "passes": shaders,
            })
        }

        "sss_profile" => {
            let material = parse_sss_material(params.sss_material.as_deref().unwrap_or("skin"))?;
            let profile = renderer::preset_sss_profile(&material);
            let diffusion = renderer::sss_diffusion_profile(&material);

            let mut result = serde_json::json!({
                "action": "sss_profile",
                "material": params.sss_material.as_deref().unwrap_or("skin"),
                "scatter_distance": profile.scatter_distance,
                "scatter_color": profile.scatter_color,
                "diffusion_samples": diffusion.len(),
            });

            // Optionally compute transmittance at a given thickness
            if let Some(thickness) = params.sss_thickness {
                let transmittance = renderer::sss_transmittance(thickness, &profile);
                if let Some(obj) = result.as_object_mut() {
                    obj.insert(
                        "transmittance".to_string(),
                        serde_json::json!({
                            "thickness": thickness,
                            "rgb": transmittance,
                        }),
                    );
                }
            }

            result
        }

        other => {
            return Err(McpError::invalid_params(
                format!(
                    "Unknown action: {other}. \
                     Use: pipeline, quality, adaptive, shaders, sss_profile"
                ),
                None,
            ));
        }
    };

    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_default(),
    )]))
}

/// Parse a quality level string into the enum variant.
fn parse_quality_level(s: &str) -> Result<nexcore_viz::renderer::QualityLevel, McpError> {
    use nexcore_viz::renderer::QualityLevel;
    match s.to_lowercase().as_str() {
        "low" => Ok(QualityLevel::Low),
        "medium" | "med" => Ok(QualityLevel::Medium),
        "high" => Ok(QualityLevel::High),
        "ultra" => Ok(QualityLevel::Ultra),
        other => Err(McpError::invalid_params(
            format!("Unknown quality level: {other}. Use: low, medium, high, ultra"),
            None,
        )),
    }
}

/// Parse an SSS material string into the enum variant.
fn parse_sss_material(s: &str) -> Result<nexcore_viz::renderer::SssMaterial, McpError> {
    use nexcore_viz::renderer::SssMaterial;
    match s.to_lowercase().as_str() {
        "skin" => Ok(SssMaterial::Skin),
        "wax" => Ok(SssMaterial::Wax),
        "marble" => Ok(SssMaterial::Marble),
        "jade" => Ok(SssMaterial::Jade),
        "milk" => Ok(SssMaterial::Milk),
        other => Err(McpError::invalid_params(
            format!("Unknown SSS material: {other}. Use: skin, wax, marble, jade, milk"),
            None,
        )),
    }
}

// ════════════════════════════════════════════════════════════════
// viz_orbital_density
// ════════════════════════════════════════════════════════════════

/// Compute electron density grid for a molecule using STO-3G basis sets.
///
/// Returns: basis function count, grid dimensions, density statistics,
/// and optionally the overlap matrix.
pub fn orbital_density(params: VizOrbitalDensityParams) -> Result<CallToolResult, McpError> {
    use nexcore_viz::molecular::{Atom, Element, Molecule};
    use nexcore_viz::orbital::{self, OrbitalConfig};

    // Parse atoms JSON
    let raw_atoms: Vec<serde_json::Value> = serde_json::from_str(&params.atoms)
        .map_err(|e| McpError::invalid_params(format!("Invalid atoms JSON: {e}"), None))?;

    if raw_atoms.is_empty() {
        return Err(McpError::invalid_params(
            "atoms array must contain at least one atom".to_string(),
            None,
        ));
    }

    let mol_name = params.name.as_deref().unwrap_or("molecule");
    let mut mol = Molecule::new(mol_name);

    for (i, raw) in raw_atoms.iter().enumerate() {
        let element_str = raw["element"].as_str().ok_or_else(|| {
            McpError::invalid_params(
                format!("Atom {i}: missing or invalid 'element' field"),
                None,
            )
        })?;

        let element = parse_element(element_str)?;
        let pos = parse_position(&raw["position"], i)?;
        mol.atoms.push(Atom::new((i as u32) + 1, element, pos));
    }

    let config = OrbitalConfig {
        grid_spacing: params.grid_spacing.unwrap_or(0.3),
        padding: params.padding.unwrap_or(3.0),
        max_grid_points: params.max_grid_points.unwrap_or(500_000),
        isovalue: params.isovalue.unwrap_or(0.02),
    };

    let basis_set = orbital::build_basis_set(&mol).map_err(|e| {
        McpError::invalid_params(format!("Basis set construction failed: {e}"), None)
    })?;

    let coefficients = orbital::default_coefficients(&basis_set);

    let grid = orbital::density_grid(&mol, &basis_set, &coefficients, &config).map_err(|e| {
        McpError::invalid_params(format!("Density grid computation failed: {e}"), None)
    })?;

    let mut result = serde_json::json!({
        "molecule": mol_name,
        "atom_count": mol.atoms.len(),
        "basis_functions": basis_set.len(),
        "grid": {
            "origin": grid.origin,
            "spacing": grid.spacing,
            "nx": grid.nx,
            "ny": grid.ny,
            "nz": grid.nz,
            "total_points": grid.values.len(),
        },
        "density": {
            "min_value": grid.min_value,
            "max_value": grid.max_value,
            "isovalue": config.isovalue,
        },
    });

    if params.include_overlap.unwrap_or(false) {
        let overlap = orbital::compute_overlap_matrix(&basis_set);
        // Return a compact representation (only unique upper-triangle values)
        let n = overlap.len();
        let mut overlap_entries = Vec::new();
        for i in 0..n {
            for j in i..n {
                let val = overlap
                    .get(i)
                    .and_then(|row| row.get(j))
                    .copied()
                    .unwrap_or(0.0);
                if val.abs() > 1e-10 {
                    overlap_entries.push(serde_json::json!({
                        "i": i, "j": j, "value": val,
                    }));
                }
            }
        }
        if let Some(obj) = result.as_object_mut() {
            obj.insert(
                "overlap_matrix".to_string(),
                serde_json::json!({
                    "size": n,
                    "nonzero_entries": overlap_entries.len(),
                    "entries": overlap_entries,
                }),
            );
        }
    }

    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_default(),
    )]))
}

/// Parse an element symbol string into the `Element` enum.
fn parse_element(s: &str) -> Result<nexcore_viz::molecular::Element, McpError> {
    use nexcore_viz::molecular::Element;
    match s.to_uppercase().as_str() {
        "H" => Ok(Element::H),
        "C" => Ok(Element::C),
        "N" => Ok(Element::N),
        "O" => Ok(Element::O),
        "F" => Ok(Element::F),
        "P" => Ok(Element::P),
        "S" => Ok(Element::S),
        "CL" => Ok(Element::Cl),
        other => Err(McpError::invalid_params(
            format!("Unsupported element '{other}'. Supported: H, C, N, O, F, P, S, Cl"),
            None,
        )),
    }
}

/// Parse a JSON position array `[x, y, z]` into `[f64; 3]`.
fn parse_position(val: &serde_json::Value, atom_idx: usize) -> Result<[f64; 3], McpError> {
    let arr = val.as_array().ok_or_else(|| {
        McpError::invalid_params(
            format!("Atom {atom_idx}: 'position' must be a [x, y, z] array"),
            None,
        )
    })?;

    if arr.len() < 3 {
        return Err(McpError::invalid_params(
            format!("Atom {atom_idx}: position array must have 3 elements"),
            None,
        ));
    }

    let x = arr[0].as_f64().unwrap_or(0.0);
    let y = arr[1].as_f64().unwrap_or(0.0);
    let z = arr[2].as_f64().unwrap_or(0.0);

    Ok([x, y, z])
}

// ════════════════════════════════════════════════════════════════
// Tests
// ════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manifold_sample_default() {
        let params = VizManifoldSampleParams {
            degree: None,
            resolution: Some(10),
            alpha: None,
            k1: None,
            k2: None,
            scale: None,
            projection: None,
        };
        let result = manifold_sample(params);
        assert!(
            result.is_ok(),
            "manifold_sample should succeed with defaults"
        );
    }

    #[test]
    fn test_manifold_sample_orthographic() {
        let params = VizManifoldSampleParams {
            degree: Some(3),
            resolution: Some(8),
            alpha: Some(0.5),
            k1: Some(0),
            k2: Some(1),
            scale: Some(1.0),
            projection: Some("orthographic".to_string()),
        };
        let result = manifold_sample(params);
        assert!(result.is_ok());
    }

    #[test]
    fn test_manifold_sample_invalid_projection() {
        let params = VizManifoldSampleParams {
            degree: None,
            resolution: Some(10),
            alpha: None,
            k1: None,
            k2: None,
            scale: None,
            projection: Some("mercator".to_string()),
        };
        let result = manifold_sample(params);
        assert!(result.is_err());
    }

    #[test]
    fn test_string_modes_open() {
        let params = VizStringModesParams {
            length: Some(2.0),
            tension: Some(4.0),
            linear_density: Some(1.0),
            num_points: Some(50),
            string_type: Some("open".to_string()),
            max_modes: Some(10),
            n_modes: Some(5),
            query_mode: Some(3),
        };
        let result = string_modes(params);
        assert!(
            result.is_ok(),
            "string_modes should succeed for open string"
        );
    }

    #[test]
    fn test_string_modes_closed() {
        let params = VizStringModesParams {
            length: None,
            tension: None,
            linear_density: None,
            num_points: None,
            string_type: Some("closed".to_string()),
            max_modes: None,
            n_modes: None,
            query_mode: None,
        };
        let result = string_modes(params);
        assert!(result.is_ok());
    }

    #[test]
    fn test_render_pipeline_default() {
        let params = VizRenderPipelineParams {
            action: None,
            quality_level: None,
            current_fps: None,
            target_fps: None,
            shader_passes: None,
            sss_material: None,
            sss_thickness: None,
        };
        let result = render_pipeline(params);
        assert!(result.is_ok(), "pipeline action should succeed");
    }

    #[test]
    fn test_render_pipeline_quality() {
        let params = VizRenderPipelineParams {
            action: Some("quality".to_string()),
            quality_level: Some("ultra".to_string()),
            current_fps: None,
            target_fps: None,
            shader_passes: None,
            sss_material: None,
            sss_thickness: None,
        };
        let result = render_pipeline(params);
        assert!(result.is_ok());
    }

    #[test]
    fn test_render_pipeline_shaders() {
        let params = VizRenderPipelineParams {
            action: Some("shaders".to_string()),
            quality_level: None,
            current_fps: None,
            target_fps: None,
            shader_passes: Some("vertex,fragment".to_string()),
            sss_material: None,
            sss_thickness: None,
        };
        let result = render_pipeline(params);
        assert!(result.is_ok());
    }

    #[test]
    fn test_render_pipeline_sss() {
        let params = VizRenderPipelineParams {
            action: Some("sss_profile".to_string()),
            quality_level: None,
            current_fps: None,
            target_fps: None,
            shader_passes: None,
            sss_material: Some("jade".to_string()),
            sss_thickness: Some(0.5),
        };
        let result = render_pipeline(params);
        assert!(result.is_ok());
    }

    #[test]
    fn test_render_pipeline_adaptive() {
        let params = VizRenderPipelineParams {
            action: Some("adaptive".to_string()),
            quality_level: Some("high".to_string()),
            current_fps: Some(30.0),
            target_fps: Some(60.0),
            shader_passes: None,
            sss_material: None,
            sss_thickness: None,
        };
        let result = render_pipeline(params);
        assert!(result.is_ok());
    }

    #[test]
    fn test_orbital_density_water() {
        let atoms_json = r#"[
            {"element": "O", "position": [0.0, 0.0, 0.0]},
            {"element": "H", "position": [0.757, 0.586, 0.0]},
            {"element": "H", "position": [-0.757, 0.586, 0.0]}
        ]"#;

        let params = VizOrbitalDensityParams {
            atoms: atoms_json.to_string(),
            name: Some("water".to_string()),
            grid_spacing: Some(0.5),
            padding: Some(2.0),
            max_grid_points: Some(100_000),
            isovalue: Some(0.02),
            include_overlap: Some(true),
        };
        let result = orbital_density(params);
        assert!(result.is_ok(), "orbital_density should succeed for water");
    }

    #[test]
    fn test_orbital_density_invalid_element() {
        let atoms_json = r#"[{"element": "Unobtainium", "position": [0,0,0]}]"#;
        let params = VizOrbitalDensityParams {
            atoms: atoms_json.to_string(),
            name: None,
            grid_spacing: None,
            padding: None,
            max_grid_points: None,
            isovalue: None,
            include_overlap: None,
        };
        let result = orbital_density(params);
        assert!(result.is_err());
    }

    #[test]
    fn test_orbital_density_empty_atoms() {
        let params = VizOrbitalDensityParams {
            atoms: "[]".to_string(),
            name: None,
            grid_spacing: None,
            padding: None,
            max_grid_points: None,
            isovalue: None,
            include_overlap: None,
        };
        let result = orbital_density(params);
        assert!(result.is_err());
    }
}
