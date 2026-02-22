//! Visualization Foundation MCP tools
//!
//! Wraps pre-phase nexcore-viz modules: molecular, surface, spectral,
//! community, centrality, vdag.
//!
//! Each tool accepts a typed param struct and returns JSON or SVG via
//! `CallToolResult::success`.

use rmcp::ErrorData as McpError;
use rmcp::model::CallToolResult;

use crate::params::viz_foundation::{
    VizCentralityParams, VizCommunityDetectParams, VizMolecularInfoParams, VizSpectralAnalysisParams,
    VizSurfaceMeshParams, VizVdagOverlayParams,
};

// ─── Helpers ────────────────────────────────────────────────────────────────

/// Build a `GraphSpec` from JSON node and edge arrays.
fn parse_graph_spec(
    nodes_json: &str,
    edges_json: &str,
) -> Result<nexcore_viz::spectral::GraphSpec, McpError> {
    let node_ids: Vec<String> = serde_json::from_str(nodes_json)
        .map_err(|e| McpError::invalid_params(format!("Invalid nodes_json: {e}"), None))?;

    let raw_edges: Vec<(String, String)> = serde_json::from_str(edges_json)
        .map_err(|e| McpError::invalid_params(format!("Invalid edges_json: {e}"), None))?;

    Ok(nexcore_viz::spectral::GraphSpec {
        node_ids,
        edges: raw_edges,
    })
}

/// Format a `HashMap<String, f64>` as sorted JSON for consistent output.
fn centrality_to_json(
    map: &std::collections::HashMap<String, f64>,
) -> serde_json::Value {
    let mut entries: Vec<(&String, &f64)> = map.iter().collect();
    entries.sort_by(|a, b| a.0.cmp(b.0));
    let obj: serde_json::Map<String, serde_json::Value> = entries
        .into_iter()
        .map(|(k, v)| (k.clone(), serde_json::json!((*v * 1_000_000.0).round() / 1_000_000.0)))
        .collect();
    serde_json::Value::Object(obj)
}

fn text_result(s: String) -> Result<CallToolResult, McpError> {
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(s)]))
}

fn json_result(v: serde_json::Value) -> Result<CallToolResult, McpError> {
    text_result(serde_json::to_string_pretty(&v).unwrap_or_default())
}

// ─── 1. Molecular Info ──────────────────────────────────────────────────────

/// Look up element data (atomic mass, VdW radius, covalent radius, CPK color)
/// for a chemical element symbol.
pub fn molecular_info(params: VizMolecularInfoParams) -> Result<CallToolResult, McpError> {
    use nexcore_viz::molecular::Element;

    let element = Element::from_symbol(&params.symbol);
    let symbol = element.symbol();

    let json = serde_json::json!({
        "symbol": symbol,
        "atomic_mass_daltons": element.atomic_mass(),
        "vdw_radius_angstroms": element.vdw_radius(),
        "covalent_radius_angstroms": element.covalent_radius(),
        "cpk_color": element.cpk_color(),
        "is_known": symbol != "X",
    });

    json_result(json)
}

// ─── 2. Surface Mesh ────────────────────────────────────────────────────────

/// Atom entry parsed from JSON input for surface generation.
#[derive(serde::Deserialize)]
struct AtomInput {
    element: String,
    x: f64,
    y: f64,
    z: f64,
}

/// Generate a molecular surface mesh via marching cubes.
///
/// Accepts atoms as JSON, builds a `Molecule`, runs `generate_surface`,
/// and returns vertex/triangle counts plus a mesh summary.
pub fn surface_mesh(params: VizSurfaceMeshParams) -> Result<CallToolResult, McpError> {
    use nexcore_viz::molecular::{Atom, Element, Molecule};
    use nexcore_viz::surface::{generate_surface, triangle_count, vertex_count, SurfaceType};

    let atoms: Vec<AtomInput> = serde_json::from_str(&params.atoms_json)
        .map_err(|e| McpError::invalid_params(format!("Invalid atoms_json: {e}"), None))?;

    if atoms.is_empty() {
        return json_result(serde_json::json!({
            "error": "No atoms provided",
            "vertex_count": 0,
            "triangle_count": 0,
        }));
    }

    let mut mol = Molecule::new("MCP Surface Input");
    for (i, a) in atoms.iter().enumerate() {
        mol.atoms.push(Atom::new(
            (i + 1) as u32,
            Element::from_symbol(&a.element),
            [a.x, a.y, a.z],
        ));
    }

    let surface_type = match params.surface_type.to_lowercase().as_str() {
        "ses" | "solvent_excluded" => SurfaceType::SolventExcluded {
            probe_radius: params.probe_radius.max(0.1),
        },
        _ => SurfaceType::VanDerWaals,
    };

    let resolution = params.resolution.clamp(0.1, 5.0);
    let mesh = generate_surface(&mol, surface_type, resolution);

    let vc = vertex_count(&mesh);
    let tc = triangle_count(&mesh);

    let json = serde_json::json!({
        "vertex_count": vc,
        "triangle_count": tc,
        "surface_type": params.surface_type,
        "resolution": resolution,
        "atom_count": atoms.len(),
        "formula": mol.formula(),
        "molecular_weight_daltons": mol.molecular_weight(),
        "has_normals": !mesh.normals.is_empty(),
        "has_colors": !mesh.colors.is_empty(),
    });

    json_result(json)
}

// ─── 3. Spectral Analysis ───────────────────────────────────────────────────

/// Compute spectral properties of a graph: adjacency matrix, Laplacian,
/// algebraic connectivity (Fiedler value), and dominant eigenvalue via
/// power iteration.
pub fn spectral_analysis(params: VizSpectralAnalysisParams) -> Result<CallToolResult, McpError> {
    use nexcore_viz::spectral::{
        adjacency_matrix, algebraic_connectivity, degree_matrix, laplacian_matrix, power_iteration,
    };

    let graph = parse_graph_spec(&params.nodes_json, &params.edges_json)?;
    let n = graph.n();

    if n == 0 {
        return json_result(serde_json::json!({
            "node_count": 0,
            "edge_count": 0,
            "algebraic_connectivity": 0.0,
            "dominant_eigenvalue": 0.0,
        }));
    }

    let adj = adjacency_matrix(&graph);
    let deg = degree_matrix(&graph);
    let lap = laplacian_matrix(&graph);
    let ac = algebraic_connectivity(&graph);
    let (dominant_eigenvalue, _eigenvector) = power_iteration(&adj, 1000, 1e-9);

    // Laplacian row sums (should be ~0 for validation)
    let lap_row_sums: Vec<f64> = lap.iter().map(|row| row.iter().sum()).collect();

    let json = serde_json::json!({
        "node_count": n,
        "edge_count": graph.edges.len(),
        "degree_vector": deg,
        "algebraic_connectivity": (ac * 1_000_000.0).round() / 1_000_000.0,
        "dominant_eigenvalue": (dominant_eigenvalue * 1_000_000.0).round() / 1_000_000.0,
        "laplacian_valid": lap_row_sums.iter().all(|s| s.abs() < 1e-10),
        "is_connected": ac > 1e-10,
        "adjacency_matrix_size": format!("{n}x{n}"),
    });

    json_result(json)
}

// ─── 4. Community Detection ─────────────────────────────────────────────────

/// Run Louvain community detection on a graph.
///
/// Returns community assignments (node IDs grouped by community) and
/// the modularity score Q.
pub fn community_detect(params: VizCommunityDetectParams) -> Result<CallToolResult, McpError> {
    use nexcore_viz::community::{detect_communities, modularity};

    let graph = parse_graph_spec(&params.nodes_json, &params.edges_json)?;

    let communities = detect_communities(&graph);
    let q = modularity(&graph, &communities);

    let community_summary: Vec<serde_json::Value> = communities
        .iter()
        .enumerate()
        .map(|(i, members)| {
            serde_json::json!({
                "community_id": i,
                "size": members.len(),
                "members": members,
            })
        })
        .collect();

    let json = serde_json::json!({
        "node_count": graph.n(),
        "edge_count": graph.edges.len(),
        "community_count": communities.len(),
        "modularity": (q * 1_000_000.0).round() / 1_000_000.0,
        "communities": community_summary,
    });

    json_result(json)
}

// ─── 5. Centrality ──────────────────────────────────────────────────────────

/// Compute centrality metrics for a graph.
///
/// Supports degree, betweenness, closeness, eigenvector, or all four.
pub fn centrality(params: VizCentralityParams) -> Result<CallToolResult, McpError> {
    use nexcore_viz::centrality::{
        betweenness_centrality, closeness_centrality, degree_centrality, eigenvector_centrality,
    };

    let graph = parse_graph_spec(&params.nodes_json, &params.edges_json)?;

    if graph.n() == 0 {
        return json_result(serde_json::json!({
            "node_count": 0,
            "metric": params.metric,
            "scores": {},
        }));
    }

    let metric = params.metric.to_lowercase();

    let json = match metric.as_str() {
        "degree" => {
            let dc = degree_centrality(&graph);
            serde_json::json!({
                "metric": "degree",
                "node_count": graph.n(),
                "scores": centrality_to_json(&dc),
            })
        }
        "betweenness" => {
            let bc = betweenness_centrality(&graph);
            serde_json::json!({
                "metric": "betweenness",
                "node_count": graph.n(),
                "scores": centrality_to_json(&bc),
            })
        }
        "closeness" => {
            let cc = closeness_centrality(&graph);
            serde_json::json!({
                "metric": "closeness",
                "node_count": graph.n(),
                "scores": centrality_to_json(&cc),
            })
        }
        "eigenvector" => {
            let ec = eigenvector_centrality(&graph, 500);
            serde_json::json!({
                "metric": "eigenvector",
                "node_count": graph.n(),
                "scores": centrality_to_json(&ec),
            })
        }
        _ => {
            // "all" or unrecognised — compute all four
            let dc = degree_centrality(&graph);
            let bc = betweenness_centrality(&graph);
            let cc = closeness_centrality(&graph);
            let ec = eigenvector_centrality(&graph, 500);
            serde_json::json!({
                "metric": "all",
                "node_count": graph.n(),
                "degree": centrality_to_json(&dc),
                "betweenness": centrality_to_json(&bc),
                "closeness": centrality_to_json(&cc),
                "eigenvector": centrality_to_json(&ec),
            })
        }
    };

    json_result(json)
}

// ─── 6. VDAG Overlay ────────────────────────────────────────────────────────

/// Node entry parsed from JSON input for VDAG construction.
#[derive(serde::Deserialize)]
struct VdagNodeInput {
    id: String,
    label: String,
    node_type: String,
    #[serde(default)]
    atc_level: Option<String>,
}

/// Edge entry parsed from JSON input for VDAG construction.
#[derive(serde::Deserialize)]
struct VdagEdgeInput {
    from: String,
    to: String,
    edge_type: String,
    #[serde(default)]
    weight: Option<f64>,
}

/// Signal entry parsed from JSON input to attach to drug nodes.
#[derive(serde::Deserialize)]
struct VdagSignalInput {
    drug_id: String,
    ae_name: String,
    #[serde(default)]
    prr: Option<f64>,
    #[serde(default)]
    ror: Option<f64>,
    #[serde(default)]
    ic025: Option<f64>,
    #[serde(default)]
    ebgm: Option<f64>,
    #[serde(default)]
    case_count: Option<u32>,
    #[serde(default)]
    timestamp: Option<String>,
}

fn parse_node_type(s: &str) -> nexcore_viz::vdag::VdagNodeType {
    match s.to_lowercase().as_str() {
        "drug_class" | "drugclass" => nexcore_viz::vdag::VdagNodeType::DrugClass,
        "drug" => nexcore_viz::vdag::VdagNodeType::Drug,
        "adverse_event" | "adverseevent" | "ae" => nexcore_viz::vdag::VdagNodeType::AdverseEvent,
        "indication" => nexcore_viz::vdag::VdagNodeType::Indication,
        _ => nexcore_viz::vdag::VdagNodeType::Drug,
    }
}

fn parse_atc_level(s: &str) -> nexcore_viz::vdag::AtcLevel {
    match s.to_lowercase().as_str() {
        "anatomical" => nexcore_viz::vdag::AtcLevel::Anatomical,
        "therapeutic" => nexcore_viz::vdag::AtcLevel::Therapeutic,
        "pharmacological" => nexcore_viz::vdag::AtcLevel::Pharmacological,
        "chemical" => nexcore_viz::vdag::AtcLevel::Chemical,
        _ => nexcore_viz::vdag::AtcLevel::Substance,
    }
}

fn parse_edge_type(s: &str) -> nexcore_viz::vdag::VdagEdgeType {
    match s.to_lowercase().as_str() {
        "contains" => nexcore_viz::vdag::VdagEdgeType::Contains,
        "interacts_with" | "interactswith" => nexcore_viz::vdag::VdagEdgeType::InteractsWith,
        "contraindicates" => nexcore_viz::vdag::VdagEdgeType::Contraindicates,
        "class_of" | "classof" => nexcore_viz::vdag::VdagEdgeType::ClassOf,
        "has_adverse_event" | "hasadverseevent" => nexcore_viz::vdag::VdagEdgeType::HasAdverseEvent,
        _ => nexcore_viz::vdag::VdagEdgeType::Contains,
    }
}

/// Build a VDAG from JSON inputs, compute signal summary, render SVG,
/// and produce 3D layout coordinates.
pub fn vdag_overlay(params: VizVdagOverlayParams) -> Result<CallToolResult, McpError> {
    use nexcore_viz::vdag::{SignalScore, Vdag, VdagEdge, VdagNode};

    // Parse nodes
    let node_inputs: Vec<VdagNodeInput> = serde_json::from_str(&params.nodes_json)
        .map_err(|e| McpError::invalid_params(format!("Invalid nodes_json: {e}"), None))?;

    // Parse edges
    let edge_inputs: Vec<VdagEdgeInput> = serde_json::from_str(&params.edges_json)
        .map_err(|e| McpError::invalid_params(format!("Invalid edges_json: {e}"), None))?;

    // Parse signals (optional)
    let signal_inputs: Vec<VdagSignalInput> = if let Some(ref sj) = params.signals_json {
        serde_json::from_str(sj)
            .map_err(|e| McpError::invalid_params(format!("Invalid signals_json: {e}"), None))?
    } else {
        vec![]
    };

    // Build signal lookup: drug_id -> Vec<SignalScore>
    let mut signal_map: std::collections::HashMap<String, Vec<SignalScore>> =
        std::collections::HashMap::new();
    for s in &signal_inputs {
        signal_map.entry(s.drug_id.clone()).or_default().push(SignalScore {
            ae_name: s.ae_name.clone(),
            prr: s.prr,
            ror: s.ror,
            ic025: s.ic025,
            ebgm: s.ebgm,
            case_count: s.case_count,
            timestamp: s.timestamp.clone(),
        });
    }

    // Build VDAG nodes
    let nodes: Vec<VdagNode> = node_inputs
        .into_iter()
        .map(|ni| VdagNode {
            id: ni.id.clone(),
            label: ni.label,
            node_type: parse_node_type(&ni.node_type),
            atc_level: ni.atc_level.as_deref().map(parse_atc_level),
            signals: signal_map.remove(&ni.id).unwrap_or_default(),
            color: None,
            metadata: std::collections::HashMap::new(),
        })
        .collect();

    // Build VDAG edges
    let edges: Vec<VdagEdge> = edge_inputs
        .into_iter()
        .map(|ei| VdagEdge {
            from: ei.from,
            to: ei.to,
            edge_type: parse_edge_type(&ei.edge_type),
            weight: ei.weight,
        })
        .collect();

    let vdag = Vdag {
        nodes,
        edges,
        title: params.title.clone(),
    };

    // Compute summary
    let summary = vdag.signal_summary();
    let drug_count = vdag.drugs().len();
    let ae_count = vdag.adverse_events().len();
    let layout_3d = vdag.to_3d_layout();

    // Render SVG
    let svg = vdag.render_svg(&nexcore_viz::Theme::default());

    let json = serde_json::json!({
        "title": params.title,
        "node_count": vdag.nodes.len(),
        "edge_count": vdag.edges.len(),
        "drug_node_count": drug_count,
        "adverse_event_node_count": ae_count,
        "signal_summary": summary.map(|s| serde_json::json!({
            "total_signals": s.total_signals,
            "active_signals": s.active_signals,
            "max_prr": s.max_prr,
            "max_case_count": s.max_case_count,
        })),
        "layout_3d_node_count": layout_3d.len(),
        "svg_length_bytes": svg.len(),
        "svg": svg,
    });

    json_result(json)
}

// ─── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── molecular_info ──────────────────────────────────────────────────

    #[test]
    fn molecular_info_carbon() {
        let result = molecular_info(VizMolecularInfoParams {
            symbol: "C".to_string(),
        });
        assert!(result.is_ok());
        let call = result.unwrap_or_else(|_| CallToolResult::success(vec![]));
        let text = call
            .content
            .first()
            .and_then(|c| match &c.raw {
                rmcp::model::RawContent::Text(t) => Some(t.text.clone()),
                _ => None,
            })
            .unwrap_or_default();
        assert!(text.contains("12.011"), "should contain carbon atomic mass");
        assert!(text.contains("#909090"), "should contain carbon CPK color");
    }

    #[test]
    fn molecular_info_unknown_element() {
        let result = molecular_info(VizMolecularInfoParams {
            symbol: "Uu".to_string(),
        });
        assert!(result.is_ok());
        let call = result.unwrap_or_else(|_| CallToolResult::success(vec![]));
        let text = call
            .content
            .first()
            .and_then(|c| match &c.raw {
                rmcp::model::RawContent::Text(t) => Some(t.text.clone()),
                _ => None,
            })
            .unwrap_or_default();
        assert!(text.contains("\"is_known\": false"));
    }

    // ── surface_mesh ────────────────────────────────────────────────────

    #[test]
    fn surface_mesh_single_atom() {
        let result = surface_mesh(VizSurfaceMeshParams {
            atoms_json: r#"[{"element": "C", "x": 0.0, "y": 0.0, "z": 0.0}]"#.to_string(),
            surface_type: "vdw".to_string(),
            probe_radius: 1.4,
            resolution: 0.5,
        });
        assert!(result.is_ok());
        let call = result.unwrap_or_else(|_| CallToolResult::success(vec![]));
        let text = call
            .content
            .first()
            .and_then(|c| match &c.raw {
                rmcp::model::RawContent::Text(t) => Some(t.text.clone()),
                _ => None,
            })
            .unwrap_or_default();
        assert!(text.contains("\"vertex_count\""));
        assert!(text.contains("\"triangle_count\""));
    }

    #[test]
    fn surface_mesh_empty_atoms() {
        let result = surface_mesh(VizSurfaceMeshParams {
            atoms_json: "[]".to_string(),
            surface_type: "vdw".to_string(),
            probe_radius: 1.4,
            resolution: 0.5,
        });
        assert!(result.is_ok());
        let call = result.unwrap_or_else(|_| CallToolResult::success(vec![]));
        let text = call
            .content
            .first()
            .and_then(|c| match &c.raw {
                rmcp::model::RawContent::Text(t) => Some(t.text.clone()),
                _ => None,
            })
            .unwrap_or_default();
        assert!(text.contains("\"vertex_count\": 0"));
    }

    // ── spectral_analysis ───────────────────────────────────────────────

    #[test]
    fn spectral_analysis_triangle() {
        let result = spectral_analysis(VizSpectralAnalysisParams {
            nodes_json: r#"["a", "b", "c"]"#.to_string(),
            edges_json: r#"[["a","b"], ["b","c"], ["a","c"]]"#.to_string(),
        });
        assert!(result.is_ok());
        let call = result.unwrap_or_else(|_| CallToolResult::success(vec![]));
        let text = call
            .content
            .first()
            .and_then(|c| match &c.raw {
                rmcp::model::RawContent::Text(t) => Some(t.text.clone()),
                _ => None,
            })
            .unwrap_or_default();
        assert!(text.contains("\"is_connected\": true"));
        assert!(text.contains("\"node_count\": 3"));
    }

    #[test]
    fn spectral_analysis_empty_graph() {
        let result = spectral_analysis(VizSpectralAnalysisParams {
            nodes_json: "[]".to_string(),
            edges_json: "[]".to_string(),
        });
        assert!(result.is_ok());
    }

    // ── community_detect ────────────────────────────────────────────────

    #[test]
    fn community_detect_two_cliques() {
        let result = community_detect(VizCommunityDetectParams {
            nodes_json: r#"["a","b","c","x","y","z"]"#.to_string(),
            edges_json: r#"[["a","b"],["b","c"],["a","c"],["x","y"],["y","z"],["x","z"],["c","x"]]"#
                .to_string(),
        });
        assert!(result.is_ok());
        let call = result.unwrap_or_else(|_| CallToolResult::success(vec![]));
        let text = call
            .content
            .first()
            .and_then(|c| match &c.raw {
                rmcp::model::RawContent::Text(t) => Some(t.text.clone()),
                _ => None,
            })
            .unwrap_or_default();
        assert!(text.contains("\"community_count\""));
        assert!(text.contains("\"modularity\""));
    }

    // ── centrality ──────────────────────────────────────────────────────

    #[test]
    fn centrality_degree_star() {
        let result = centrality(VizCentralityParams {
            nodes_json: r#"["hub","a","b","c"]"#.to_string(),
            edges_json: r#"[["hub","a"],["hub","b"],["hub","c"]]"#.to_string(),
            metric: "degree".to_string(),
        });
        assert!(result.is_ok());
        let call = result.unwrap_or_else(|_| CallToolResult::success(vec![]));
        let text = call
            .content
            .first()
            .and_then(|c| match &c.raw {
                rmcp::model::RawContent::Text(t) => Some(t.text.clone()),
                _ => None,
            })
            .unwrap_or_default();
        assert!(text.contains("\"hub\": 1.0"), "hub should have degree centrality 1.0");
    }

    #[test]
    fn centrality_all_metrics() {
        let result = centrality(VizCentralityParams {
            nodes_json: r#"["a","b","c"]"#.to_string(),
            edges_json: r#"[["a","b"],["b","c"]]"#.to_string(),
            metric: "all".to_string(),
        });
        assert!(result.is_ok());
        let call = result.unwrap_or_else(|_| CallToolResult::success(vec![]));
        let text = call
            .content
            .first()
            .and_then(|c| match &c.raw {
                rmcp::model::RawContent::Text(t) => Some(t.text.clone()),
                _ => None,
            })
            .unwrap_or_default();
        assert!(text.contains("\"degree\""));
        assert!(text.contains("\"betweenness\""));
        assert!(text.contains("\"closeness\""));
        assert!(text.contains("\"eigenvector\""));
    }

    // ── vdag_overlay ────────────────────────────────────────────────────

    #[test]
    fn vdag_overlay_basic() {
        let result = vdag_overlay(VizVdagOverlayParams {
            nodes_json: r#"[
                {"id": "analgesics", "label": "Analgesics (N02)", "node_type": "drug_class"},
                {"id": "aspirin", "label": "Aspirin", "node_type": "drug"}
            ]"#
            .to_string(),
            edges_json: r#"[{"from": "analgesics", "to": "aspirin", "edge_type": "contains"}]"#
                .to_string(),
            signals_json: Some(
                r#"[{"drug_id": "aspirin", "ae_name": "GI Bleeding", "prr": 3.2, "ror": 3.5, "ic025": 0.8, "ebgm": 2.9, "case_count": 412}]"#
                    .to_string(),
            ),
            title: "Test VDAG".to_string(),
        });
        assert!(result.is_ok());
        let call = result.unwrap_or_else(|_| CallToolResult::success(vec![]));
        let text = call
            .content
            .first()
            .and_then(|c| match &c.raw {
                rmcp::model::RawContent::Text(t) => Some(t.text.clone()),
                _ => None,
            })
            .unwrap_or_default();
        assert!(text.contains("\"drug_node_count\": 2"));
        assert!(text.contains("\"total_signals\": 1"));
        assert!(text.contains("<svg"));
    }

    #[test]
    fn vdag_overlay_no_signals() {
        let result = vdag_overlay(VizVdagOverlayParams {
            nodes_json: r#"[{"id": "a", "label": "Drug A", "node_type": "drug"}]"#.to_string(),
            edges_json: "[]".to_string(),
            signals_json: None,
            title: "Empty VDAG".to_string(),
        });
        assert!(result.is_ok());
    }
}
