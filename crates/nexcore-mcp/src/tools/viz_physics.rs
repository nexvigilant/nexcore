//! Visualization Physics MCP tools
//!
//! Wraps Phase 3 (Physics Engine) + Phase 4 (Nervous System) nexcore-viz modules:
//! dynamics, force_field, gpu_layout, hypergraph, lod, minimizer, particle,
//! ae_overlay, coord_gen, bipartite.

use rmcp::ErrorData as McpError;
use rmcp::model::CallToolResult;

use crate::params::{
    VizAeOverlayParams, VizBipartiteLayoutParams, VizCoordGenParams, VizDynamicsStepParams,
    VizForceFieldEnergyParams, VizGpuLayoutParams, VizHypergraphParams, VizLodSelectParams,
    VizMinimizeEnergyParams, VizParticlePresetParams,
};

// ── helpers ──────────────────────────────────────────────────────────────────

/// Build a Molecule from element symbols, flat positions, and flat bond-pair list.
fn build_molecule(
    elements: &[String],
    positions: &[f64],
    bonds: &[usize],
) -> Result<nexcore_viz::molecular::Molecule, McpError> {
    use nexcore_viz::molecular::{Atom, Bond, BondOrder, Element, Molecule};

    if positions.len() != elements.len() * 3 {
        return Err(McpError::invalid_params(
            format!(
                "positions length {} does not match elements length {} * 3",
                positions.len(),
                elements.len()
            ),
            None,
        ));
    }
    if bonds.len() % 2 != 0 {
        return Err(McpError::invalid_params(
            "bonds array must have even length (pairs of atom indices)",
            None,
        ));
    }

    let mut mol = Molecule::new("input");
    for (i, sym) in elements.iter().enumerate() {
        let pos = [positions[i * 3], positions[i * 3 + 1], positions[i * 3 + 2]];
        mol.atoms
            .push(Atom::new((i + 1) as u32, Element::from_symbol(sym), pos));
    }
    for chunk in bonds.chunks(2) {
        if let [a1, a2] = chunk {
            mol.bonds.push(Bond {
                atom1: *a1,
                atom2: *a2,
                order: BondOrder::Single,
            });
        }
    }
    Ok(mol)
}

// ── dynamics ─────────────────────────────────────────────────────────────────

/// Run a molecular dynamics simulation and return trajectory summary.
pub fn dynamics_step(params: VizDynamicsStepParams) -> Result<CallToolResult, McpError> {
    use nexcore_viz::dynamics::{DynamicsConfig, run_simulation};
    use nexcore_viz::force_field::ForceFieldConfig;

    let mut mol = build_molecule(&params.elements, &params.positions, &params.bonds)?;

    let config = DynamicsConfig {
        timestep: params.timestep,
        temperature: params.temperature,
        friction: 0.1,
        total_steps: params.total_steps,
        save_interval: params.total_steps.max(1),
    };
    let ff_config = ForceFieldConfig::default();

    match run_simulation(&mut mol, &config, &ff_config) {
        Ok(result) => {
            let final_positions: Vec<[f64; 3]> = mol.atoms.iter().map(|a| a.position).collect();
            let json = serde_json::json!({
                "status": "completed",
                "steps_run": result.total_steps,
                "final_energy_kcal": result.final_energy,
                "final_positions": final_positions,
                "trajectory_frames": result.frames.len(),
            });
            Ok(CallToolResult::success(vec![rmcp::model::Content::text(
                serde_json::to_string_pretty(&json).unwrap_or_default(),
            )]))
        }
        Err(e) => Ok(CallToolResult::error(vec![rmcp::model::Content::text(
            format!("Dynamics error: {e}"),
        )])),
    }
}

// ── force_field ──────────────────────────────────────────────────────────────

/// Compute molecular force field energy breakdown.
pub fn force_field_energy(params: VizForceFieldEnergyParams) -> Result<CallToolResult, McpError> {
    use nexcore_viz::force_field::{ForceFieldConfig, compute_energy};

    let mol = build_molecule(&params.elements, &params.positions, &params.bonds)?;

    let config = ForceFieldConfig {
        cutoff: params.cutoff.unwrap_or(10.0),
        ..ForceFieldConfig::default()
    };

    match compute_energy(&mol, &config) {
        Ok(energy) => {
            let json = serde_json::json!({
                "total_energy_kcal": energy.total,
                "bond_stretch": energy.bond_stretch,
                "angle_bend": energy.angle_bend,
                "torsion": energy.torsion,
                "van_der_waals": energy.vdw,
                "electrostatic": energy.electrostatic,
                "atom_count": mol.atoms.len(),
                "bond_count": mol.bonds.len(),
            });
            Ok(CallToolResult::success(vec![rmcp::model::Content::text(
                serde_json::to_string_pretty(&json).unwrap_or_default(),
            )]))
        }
        Err(e) => Ok(CallToolResult::error(vec![rmcp::model::Content::text(
            format!("Force field error: {e}"),
        )])),
    }
}

// ── gpu_layout ───────────────────────────────────────────────────────────────

/// Run force-directed graph layout (CPU-side Fruchterman-Reingold / ForceAtlas2).
pub fn gpu_layout(params: VizGpuLayoutParams) -> Result<CallToolResult, McpError> {
    use nexcore_viz::gpu_layout::{LayoutConfig, LayoutEdge, compute_layout, random_layout};

    if params.node_count == 0 {
        return Ok(CallToolResult::error(vec![rmcp::model::Content::text(
            "Error: node_count must be > 0",
        )]));
    }
    if params.edges.len() % 2 != 0 {
        return Ok(CallToolResult::error(vec![rmcp::model::Content::text(
            "Error: edges array must have even length (pairs of node indices)",
        )]));
    }

    let nodes = random_layout(params.node_count, params.width, params.height);
    let edge_count = params.edges.len() / 2;
    let edges: Vec<LayoutEdge> = params
        .edges
        .chunks(2)
        .enumerate()
        .map(|(i, chunk)| LayoutEdge {
            source: chunk[0],
            target: chunk[1],
            weight: params
                .weights
                .as_ref()
                .and_then(|w| w.get(i).copied())
                .unwrap_or(1.0),
            ideal_length: 100.0,
        })
        .collect();

    let config = LayoutConfig {
        iterations: params.iterations,
        ..LayoutConfig::default()
    };

    match compute_layout(nodes, edges, &config) {
        Ok(result) => {
            let positions: Vec<[f64; 2]> = result.nodes.iter().map(|n| n.position).collect();
            let json = serde_json::json!({
                "node_count": result.nodes.len(),
                "edge_count": edge_count,
                "iterations_run": result.iterations_run,
                "final_energy": result.energy,
                "converged": result.converged,
                "positions": positions,
            });
            Ok(CallToolResult::success(vec![rmcp::model::Content::text(
                serde_json::to_string_pretty(&json).unwrap_or_default(),
            )]))
        }
        Err(e) => Ok(CallToolResult::error(vec![rmcp::model::Content::text(
            format!("Layout error: {e}"),
        )])),
    }
}

// ── hypergraph ───────────────────────────────────────────────────────────────

/// Construct a hypergraph and compute metrics, components, bipartiteness.
pub fn hypergraph(params: VizHypergraphParams) -> Result<CallToolResult, McpError> {
    use nexcore_viz::hypergraph::{
        HyperEdge, HyperNode, Hypergraph, compute_metrics, connected_components, detect_bipartite,
    };

    let raw_edges: Vec<Vec<usize>> = serde_json::from_str(&params.hyperedges)
        .map_err(|e| McpError::invalid_params(format!("Invalid hyperedges JSON: {e}"), None))?;

    let mut hg = Hypergraph::new(params.directed);
    for (i, label) in params.node_labels.iter().enumerate() {
        hg.add_node(HyperNode {
            id: i,
            label: label.clone(),
            weight: 1.0,
            metadata: None,
            position: None,
        });
    }
    for (i, edge_nodes) in raw_edges.iter().enumerate() {
        if let Err(e) = hg.add_edge(HyperEdge {
            id: i,
            nodes: edge_nodes.clone(),
            weight: 1.0,
            label: format!("e{i}"),
        }) {
            return Ok(CallToolResult::error(vec![rmcp::model::Content::text(
                format!("Hyperedge {i} error: {e}"),
            )]));
        }
    }

    let metrics = compute_metrics(&hg);
    let components = connected_components(&hg);
    let is_bipartite = detect_bipartite(&hg);

    let json = serde_json::json!({
        "node_count": metrics.node_count,
        "edge_count": metrics.edge_count,
        "avg_edge_size": metrics.avg_edge_size,
        "max_edge_size": metrics.max_edge_size,
        "density": metrics.density,
        "connected_components": components.len(),
        "component_sizes": components.iter().map(|c| c.len()).collect::<Vec<_>>(),
        "is_bipartite": is_bipartite,
        "directed": params.directed,
    });
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string_pretty(&json).unwrap_or_default(),
    )]))
}

// ── lod ──────────────────────────────────────────────────────────────────────

/// Select Level-of-Detail representation level based on atom count.
pub fn lod_select(params: VizLodSelectParams) -> Result<CallToolResult, McpError> {
    let (level, description) = if params.atom_count < 500 {
        ("FullAtom", "All atoms and bonds rendered")
    } else if params.atom_count < 5_000 {
        ("CalphaTrace", "C-alpha backbone trace only")
    } else if params.atom_count < 20_000 {
        (
            "SecondaryStructure",
            "Helix cylinders, sheet arrows, coil tubes",
        )
    } else if params.atom_count < 50_000 {
        ("DomainBlob", "One sphere per domain/chain segment")
    } else {
        ("ConvexHull", "Convex hull envelope of all atoms")
    };

    let reduction = if params.atom_count > 0 {
        match level {
            "FullAtom" => 1.0,
            "CalphaTrace" => 0.1,
            "SecondaryStructure" => 0.02,
            "DomainBlob" => 0.005,
            _ => 0.001,
        }
    } else {
        0.0
    };

    let json = serde_json::json!({
        "atom_count": params.atom_count,
        "lod_level": level,
        "description": description,
        "approximate_rendered_elements": (params.atom_count as f64 * reduction) as usize,
        "reduction_ratio": reduction,
    });
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string_pretty(&json).unwrap_or_default(),
    )]))
}

// ── minimizer ────────────────────────────────────────────────────────────────

/// Run energy minimization on a molecular structure.
pub fn minimize_energy(params: VizMinimizeEnergyParams) -> Result<CallToolResult, McpError> {
    use nexcore_viz::force_field::{ForceFieldConfig, compute_energy, compute_forces};

    let mut mol = build_molecule(&params.elements, &params.positions, &params.bonds)?;
    let ff_config = ForceFieldConfig::default();

    let initial_energy = compute_energy(&mol, &ff_config)
        .map(|e| e.total)
        .unwrap_or(f64::NAN);

    let step_size = 0.01_f64;
    let mut steps_taken = 0_usize;
    let mut converged = false;

    for step in 0..params.max_steps {
        let forces = match compute_forces(&mol, &ff_config) {
            Ok(f) => f,
            Err(e) => {
                return Ok(CallToolResult::error(vec![rmcp::model::Content::text(
                    format!("Force computation error at step {step}: {e}"),
                )]));
            }
        };

        let max_force: f64 = forces
            .iter()
            .map(|f| (f[0] * f[0] + f[1] * f[1] + f[2] * f[2]).sqrt())
            .fold(0.0_f64, f64::max);

        if max_force < params.tolerance {
            converged = true;
            steps_taken = step;
            break;
        }

        for (atom, force) in mol.atoms.iter_mut().zip(forces.iter()) {
            atom.position[0] += force[0] * step_size;
            atom.position[1] += force[1] * step_size;
            atom.position[2] += force[2] * step_size;
        }
        steps_taken = step + 1;
    }

    let final_energy = compute_energy(&mol, &ff_config)
        .map(|e| e.total)
        .unwrap_or(f64::NAN);

    let final_positions: Vec<[f64; 3]> = mol.atoms.iter().map(|a| a.position).collect();

    let json = serde_json::json!({
        "initial_energy_kcal": initial_energy,
        "final_energy_kcal": final_energy,
        "energy_reduction": initial_energy - final_energy,
        "steps_taken": steps_taken,
        "converged": converged,
        "tolerance": params.tolerance,
        "final_positions": final_positions,
    });
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string_pretty(&json).unwrap_or_default(),
    )]))
}

// ── particle ─────────────────────────────────────────────────────────────────

/// Get a particle system preset configuration (solvent, reaction, electron).
pub fn particle_preset(params: VizParticlePresetParams) -> Result<CallToolResult, McpError> {
    use nexcore_viz::particle::{electron_cloud, reaction_burst, solvent_emitter};

    let center = params.center.unwrap_or([0.0, 0.0, 0.0]);
    let radius = params.radius.unwrap_or(2.0);

    let (config, preset_name) = match params.preset.to_lowercase().as_str() {
        "solvent" => (solvent_emitter(center, radius), "solvent"),
        "reaction" => (reaction_burst(center), "reaction_burst"),
        "electron" => (electron_cloud(center, radius), "electron_cloud"),
        other => {
            return Ok(CallToolResult::error(vec![rmcp::model::Content::text(
                format!("Unknown preset: {other}. Use: solvent, reaction, electron"),
            )]));
        }
    };

    let json = serde_json::json!({
        "preset": preset_name,
        "max_particles": config.max_particles,
        "rate": config.rate,
        "lifetime": config.lifetime,
        "initial_speed": config.initial_speed,
        "initial_color": config.initial_color,
        "initial_size": config.initial_size,
        "center": center,
        "radius": radius,
    });
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string_pretty(&json).unwrap_or_default(),
    )]))
}

// ── ae_overlay ───────────────────────────────────────────────────────────────

/// Compute an AE signal heatmap from drug signal data.
pub fn ae_overlay(params: VizAeOverlayParams) -> Result<CallToolResult, McpError> {
    use nexcore_viz::ae_overlay::{
        NormalizationMethod, OverlayConfig, ScoreField, compute_heatmap,
    };
    use nexcore_viz::vdag::{SignalScore, VdagNode, VdagNodeType};
    use std::collections::HashMap;

    let raw_drugs: Vec<serde_json::Value> = serde_json::from_str(&params.drugs)
        .map_err(|e| McpError::invalid_params(format!("Invalid drugs JSON: {e}"), None))?;

    let nodes: Vec<VdagNode> = raw_drugs
        .iter()
        .map(|d| {
            let signals_raw = d["signals"].as_array().cloned().unwrap_or_default();
            let signals: Vec<SignalScore> = signals_raw
                .iter()
                .map(|s| SignalScore {
                    ae_name: s["ae_name"].as_str().unwrap_or("unknown").to_string(),
                    prr: s["prr"].as_f64(),
                    ror: s["ror"].as_f64(),
                    ic025: s["ic025"].as_f64(),
                    ebgm: s["ebgm"].as_f64(),
                    case_count: s["case_count"].as_u64().map(|v| v as u32),
                    timestamp: s["timestamp"].as_str().map(String::from),
                })
                .collect();
            VdagNode {
                id: d["id"].as_str().unwrap_or("drug").to_string(),
                label: d["label"].as_str().unwrap_or("Drug").to_string(),
                node_type: VdagNodeType::Drug,
                atc_level: None,
                signals,
                color: None,
                metadata: HashMap::new(),
            }
        })
        .collect();

    let score_field = match params.score_field.to_lowercase().as_str() {
        "ror" => ScoreField::Ror,
        "ic025" => ScoreField::Ic025,
        "ebgm" => ScoreField::Ebgm,
        _ => ScoreField::Prr,
    };

    let normalization = match params.normalization.to_lowercase().as_str() {
        "z_score" | "zscore" => NormalizationMethod::ZScore,
        "percentile" => NormalizationMethod::Percentile,
        "log" => NormalizationMethod::Log,
        _ => NormalizationMethod::MinMax,
    };

    let config = OverlayConfig {
        score_field,
        normalization,
        ..nexcore_viz::ae_overlay::default_config()
    };

    match compute_heatmap(&nodes, &config) {
        Ok(heatmap) => {
            let signal_cells: Vec<_> = heatmap
                .cells
                .iter()
                .filter(|c| c.is_signal)
                .map(|c| {
                    serde_json::json!({
                        "drug": c.drug_id,
                        "ae": c.ae_name,
                        "score": c.raw_score,
                        "color": c.color,
                    })
                })
                .collect();

            let json = serde_json::json!({
                "drug_count": heatmap.drug_ids.len(),
                "ae_count": heatmap.ae_names.len(),
                "total_cells": heatmap.cells.len(),
                "signal_cells": signal_cells.len(),
                "signals": signal_cells,
                "score_range": [heatmap.min_score, heatmap.max_score],
                "score_field": format!("{score_field:?}"),
                "normalization": format!("{normalization:?}"),
            });
            Ok(CallToolResult::success(vec![rmcp::model::Content::text(
                serde_json::to_string_pretty(&json).unwrap_or_default(),
            )]))
        }
        Err(e) => Ok(CallToolResult::error(vec![rmcp::model::Content::text(
            format!("AE overlay error: {e}"),
        )])),
    }
}

// ── coord_gen ────────────────────────────────────────────────────────────────

/// Generate 3D molecular coordinates via distance geometry.
pub fn coord_gen(params: VizCoordGenParams) -> Result<CallToolResult, McpError> {
    use nexcore_viz::coord_gen::{CoordGenConfig, generate_coordinates};
    use nexcore_viz::molecular::{Atom, Bond, BondOrder, Element, Molecule};

    if params.bonds.len() % 2 != 0 {
        return Ok(CallToolResult::error(vec![rmcp::model::Content::text(
            "Error: bonds array must have even length",
        )]));
    }

    let mut mol = Molecule::new("input");
    for (i, sym) in params.elements.iter().enumerate() {
        mol.atoms.push(Atom::new(
            (i + 1) as u32,
            Element::from_symbol(sym),
            [0.0, 0.0, 0.0],
        ));
    }
    for chunk in params.bonds.chunks(2) {
        if let [a1, a2] = chunk {
            mol.bonds.push(Bond {
                atom1: *a1,
                atom2: *a2,
                order: BondOrder::Single,
            });
        }
    }

    let config = CoordGenConfig {
        seed: params.seed,
        max_refinement_steps: params.max_iterations,
        ..CoordGenConfig::default()
    };

    match generate_coordinates(&mol, &config) {
        Ok(result) => {
            let json = serde_json::json!({
                "atom_count": result.coordinates.len(),
                "coordinates": result.coordinates,
                "final_stress": result.stress,
                "iterations": result.iterations,
                "converged": result.converged,
            });
            Ok(CallToolResult::success(vec![rmcp::model::Content::text(
                serde_json::to_string_pretty(&json).unwrap_or_default(),
            )]))
        }
        Err(e) => Ok(CallToolResult::error(vec![rmcp::model::Content::text(
            format!("Coordinate generation error: {e}"),
        )])),
    }
}

// ── bipartite ────────────────────────────────────────────────────────────────

/// Compute bipartite drug-AE network layout from a VDAG definition.
pub fn bipartite_layout(params: VizBipartiteLayoutParams) -> Result<CallToolResult, McpError> {
    use nexcore_viz::bipartite::{
        Side, default_config, from_vdag, layout_bipartite, render_bipartite_svg,
    };
    use nexcore_viz::theme::Theme;
    use nexcore_viz::vdag::{Vdag, VdagEdge, VdagEdgeType, VdagNode, VdagNodeType};
    use std::collections::HashMap;

    let raw: serde_json::Value = serde_json::from_str(&params.vdag_json)
        .map_err(|e| McpError::invalid_params(format!("Invalid VDAG JSON: {e}"), None))?;

    let title = raw["title"]
        .as_str()
        .unwrap_or("Drug-AE Network")
        .to_string();

    let raw_nodes = raw["nodes"].as_array().cloned().unwrap_or_default();
    let raw_edges = raw["edges"].as_array().cloned().unwrap_or_default();

    let nodes: Vec<VdagNode> = raw_nodes
        .iter()
        .map(|n| {
            let nt = match n["type"].as_str().unwrap_or("drug") {
                "ae" | "adverse_event" | "AdverseEvent" => VdagNodeType::AdverseEvent,
                "class" | "drug_class" | "DrugClass" => VdagNodeType::DrugClass,
                _ => VdagNodeType::Drug,
            };
            VdagNode {
                id: n["id"].as_str().unwrap_or("").to_string(),
                label: n["label"].as_str().unwrap_or("").to_string(),
                node_type: nt,
                atc_level: None,
                signals: vec![],
                color: None,
                metadata: HashMap::new(),
            }
        })
        .collect();

    let edges: Vec<VdagEdge> = raw_edges
        .iter()
        .map(|e| VdagEdge {
            from: e["from"].as_str().unwrap_or("").to_string(),
            to: e["to"].as_str().unwrap_or("").to_string(),
            edge_type: VdagEdgeType::HasAdverseEvent,
            weight: e["weight"].as_f64(),
        })
        .collect();

    let vdag = Vdag {
        nodes,
        edges,
        title: title.clone(),
    };

    match from_vdag(&vdag) {
        Ok((all_nodes, biedges)) => {
            let (mut left, mut right): (Vec<_>, Vec<_>) = all_nodes
                .into_iter()
                .partition(|n| matches!(n.side, Side::Left));

            let config = default_config();
            let layout = layout_bipartite(&mut left, &mut right, &biedges, &config);
            let svg = render_bipartite_svg(&layout, &title, &Theme::default());

            let json = serde_json::json!({
                "left_count": layout.left_nodes.len(),
                "right_count": layout.right_nodes.len(),
                "edge_count": layout.edges.len(),
                "crossings": layout.crossing_count,
                "svg_length": svg.len(),
                "svg": svg,
            });
            Ok(CallToolResult::success(vec![rmcp::model::Content::text(
                serde_json::to_string_pretty(&json).unwrap_or_default(),
            )]))
        }
        Err(e) => Ok(CallToolResult::error(vec![rmcp::model::Content::text(
            format!("Bipartite layout error: {e}"),
        )])),
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_force_field_energy_water() {
        let params = VizForceFieldEnergyParams {
            elements: vec!["O".into(), "H".into(), "H".into()],
            positions: vec![0.0, 0.0, 0.0, 0.96, 0.0, 0.0, -0.24, 0.93, 0.0],
            bonds: vec![0, 1, 0, 2],
            cutoff: None,
        };
        let result = force_field_energy(params);
        assert!(result.is_ok());
    }

    #[test]
    fn test_gpu_layout_triangle() {
        let params = VizGpuLayoutParams {
            node_count: 3,
            edges: vec![0, 1, 1, 2, 2, 0],
            weights: None,
            width: 400.0,
            height: 300.0,
            iterations: 50,
        };
        let result = gpu_layout(params);
        assert!(result.is_ok());
    }

    #[test]
    fn test_hypergraph_basic() {
        let params = VizHypergraphParams {
            node_labels: vec!["A".into(), "B".into(), "C".into()],
            hyperedges: "[[0,1,2]]".into(),
            directed: false,
        };
        let result = hypergraph(params);
        assert!(result.is_ok());
    }

    #[test]
    fn test_lod_select_small() {
        let params = VizLodSelectParams {
            atom_count: 100,
            thresholds: None,
        };
        let result = lod_select(params);
        assert!(result.is_ok());
    }

    #[test]
    fn test_particle_solvent() {
        let params = VizParticlePresetParams {
            preset: "solvent".into(),
            center: Some([1.0, 2.0, 3.0]),
            radius: Some(5.0),
        };
        let result = particle_preset(params);
        assert!(result.is_ok());
    }

    #[test]
    fn test_lod_select_large() {
        let params = VizLodSelectParams {
            atom_count: 55_000,
            thresholds: None,
        };
        let result = lod_select(params);
        assert!(result.is_ok());
    }
}
