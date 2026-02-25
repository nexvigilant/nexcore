//! Graph analysis and construction MCP tools
//!
//! Unified interface for graph algorithms: centrality, components,
//! shortest paths, PageRank, community detection, SCC, and topological sort.

use crate::params::{GraphAnalyzeParams, GraphConstructParams};
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use serde_json::json;
use stem_math::graph::{self, Graph, VertexId};

/// Build a Graph<String, f64> from MCP vertex/edge input.
fn build_graph(
    vertices: &[crate::params::VertexInput],
    edges: &[crate::params::EdgeInput],
) -> Result<Graph<String, f64>, McpError> {
    // Determine vertex count
    let max_idx = edges.iter().map(|e| e.from.max(e.to)).max().unwrap_or(0);
    let vertex_count = if vertices.is_empty() {
        max_idx + 1
    } else {
        vertices.len().max(max_idx + 1)
    };

    let mut g: Graph<String, f64> = Graph::with_capacity(vertex_count);

    // Add vertices
    for i in 0..vertex_count {
        let label = vertices
            .get(i)
            .and_then(|v| v.label.clone())
            .unwrap_or_else(|| format!("v{i}"));
        g.add_vertex(label);
    }

    // Add edges
    for edge in edges {
        if edge.from >= vertex_count || edge.to >= vertex_count {
            return Err(McpError::invalid_params(
                format!(
                    "Edge ({}, {}) references vertex beyond count {}",
                    edge.from, edge.to, vertex_count
                ),
                None,
            ));
        }
        g.add_edge(
            VertexId::new(edge.from),
            VertexId::new(edge.to),
            edge.weight.unwrap_or(1.0),
        );
    }

    Ok(g)
}

/// Analyze a graph structure.
pub fn graph_analyze(params: GraphAnalyzeParams) -> Result<CallToolResult, McpError> {
    let g = build_graph(&params.vertices, &params.edges)?;

    let mode = params.analysis.to_lowercase();
    match mode.as_str() {
        "centrality" => analyze_centrality(&g),
        "components" => analyze_components(&g),
        "shortest_path" => analyze_shortest_path(&g, params.from_vertex, params.to_vertex),
        "dijkstra" => analyze_dijkstra(&g, params.from_vertex, params.to_vertex),
        "pagerank" => analyze_pagerank(&g, params.damping, params.max_iterations, params.tolerance),
        "communities" => analyze_communities(&g),
        "scc" => analyze_scc(&g),
        "topo_sort" => analyze_topo_sort(&g),
        other => Err(McpError::invalid_params(
            format!(
                "Unknown analysis '{other}'. Use: centrality, components, shortest_path, dijkstra, pagerank, communities, scc, topo_sort"
            ),
            None,
        )),
    }
}

fn analyze_centrality(g: &Graph<String, f64>) -> Result<CallToolResult, McpError> {
    let bc = graph::betweenness_centrality(g);
    let mut ranked: Vec<_> = g
        .vertices()
        .zip(bc.iter())
        .map(|((id, label), &score)| {
            json!({
                "vertex": id.index(),
                "label": label,
                "betweenness": (score * 1_000_000.0).round() / 1_000_000.0,
            })
        })
        .collect();
    ranked.sort_by(|a, b| {
        b["betweenness"]
            .as_f64()
            .unwrap_or(0.0)
            .partial_cmp(&a["betweenness"].as_f64().unwrap_or(0.0))
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let result = json!({
        "analysis": "centrality",
        "vertex_count": g.vertex_count(),
        "edge_count": g.edge_count(),
        "ranked": ranked,
        "grounding": "μ(Mapping) + λ(Location) + σ(Sequence)",
    });
    Ok(CallToolResult::success(vec![Content::text(
        result.to_string(),
    )]))
}

fn analyze_components(g: &Graph<String, f64>) -> Result<CallToolResult, McpError> {
    let comps = graph::connected_components(g);
    let component_data: Vec<_> = comps
        .iter()
        .enumerate()
        .map(|(i, comp)| {
            let labels: Vec<_> = comp
                .iter()
                .filter_map(|&id| g.vertex(id).cloned())
                .collect();
            json!({
                "component": i,
                "size": comp.len(),
                "vertices": labels,
            })
        })
        .collect();

    let result = json!({
        "analysis": "components",
        "component_count": comps.len(),
        "vertex_count": g.vertex_count(),
        "components": component_data,
        "grounding": "∂(Boundary) + μ(Mapping)",
    });
    Ok(CallToolResult::success(vec![Content::text(
        result.to_string(),
    )]))
}

fn analyze_shortest_path(
    g: &Graph<String, f64>,
    from: Option<usize>,
    to: Option<usize>,
) -> Result<CallToolResult, McpError> {
    let from_id = VertexId::new(
        from.ok_or_else(|| McpError::invalid_params("shortest_path requires from_vertex", None))?,
    );
    let to_id = VertexId::new(
        to.ok_or_else(|| McpError::invalid_params("shortest_path requires to_vertex", None))?,
    );

    match graph::bfs_shortest_path(g, from_id, to_id) {
        Some(path) => {
            let labels: Vec<_> = path
                .iter()
                .filter_map(|&id| g.vertex(id).cloned())
                .collect();
            let result = json!({
                "analysis": "shortest_path",
                "found": true,
                "path_length": path.len() - 1,
                "path": labels,
                "grounding": "σ(Sequence) + λ(Location)",
            });
            Ok(CallToolResult::success(vec![Content::text(
                result.to_string(),
            )]))
        }
        None => {
            let result = json!({
                "analysis": "shortest_path",
                "found": false,
                "interpretation": "No path exists between the specified vertices",
            });
            Ok(CallToolResult::success(vec![Content::text(
                result.to_string(),
            )]))
        }
    }
}

fn analyze_dijkstra(
    g: &Graph<String, f64>,
    from: Option<usize>,
    to: Option<usize>,
) -> Result<CallToolResult, McpError> {
    let from_id = VertexId::new(
        from.ok_or_else(|| McpError::invalid_params("dijkstra requires from_vertex", None))?,
    );
    let to_id = VertexId::new(
        to.ok_or_else(|| McpError::invalid_params("dijkstra requires to_vertex", None))?,
    );

    match graph::dijkstra(g, from_id, to_id) {
        Some((path, cost)) => {
            let labels: Vec<_> = path
                .iter()
                .filter_map(|&id| g.vertex(id).cloned())
                .collect();
            let result = json!({
                "analysis": "dijkstra",
                "found": true,
                "total_cost": (cost * 1_000_000.0).round() / 1_000_000.0,
                "path_length": path.len() - 1,
                "path": labels,
                "grounding": "σ(Sequence) + N(Quantity) + λ(Location)",
            });
            Ok(CallToolResult::success(vec![Content::text(
                result.to_string(),
            )]))
        }
        None => {
            let result = json!({
                "analysis": "dijkstra",
                "found": false,
                "interpretation": "No path exists between the specified vertices",
            });
            Ok(CallToolResult::success(vec![Content::text(
                result.to_string(),
            )]))
        }
    }
}

fn analyze_pagerank(
    g: &Graph<String, f64>,
    damping: f64,
    max_iterations: usize,
    tolerance: f64,
) -> Result<CallToolResult, McpError> {
    let ranks = graph::pagerank(g, damping, max_iterations, tolerance);
    let mut ranked: Vec<_> = g
        .vertices()
        .zip(ranks.iter())
        .map(|((id, label), measured)| {
            json!({
                "vertex": id.index(),
                "label": label,
                "rank": (measured.value * 1_000_000.0).round() / 1_000_000.0,
                "confidence": (measured.confidence.value() * 1_000_000.0).round() / 1_000_000.0,
            })
        })
        .collect();
    ranked.sort_by(|a, b| {
        b["rank"]
            .as_f64()
            .unwrap_or(0.0)
            .partial_cmp(&a["rank"].as_f64().unwrap_or(0.0))
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let result = json!({
        "analysis": "pagerank",
        "damping": damping,
        "vertex_count": g.vertex_count(),
        "ranked": ranked,
        "grounding": "ρ(Recursion) + N(Quantity) + μ(Mapping)",
    });
    Ok(CallToolResult::success(vec![Content::text(
        result.to_string(),
    )]))
}

fn analyze_communities(g: &Graph<String, f64>) -> Result<CallToolResult, McpError> {
    let communities = graph::louvain_communities(g);
    let community_data: Vec<_> = communities
        .iter()
        .enumerate()
        .map(|(i, comm)| {
            let labels: Vec<_> = comm
                .iter()
                .filter_map(|&id| g.vertex(id).cloned())
                .collect();
            json!({
                "community": i,
                "size": comm.len(),
                "vertices": labels,
            })
        })
        .collect();

    let result = json!({
        "analysis": "communities",
        "community_count": communities.len(),
        "vertex_count": g.vertex_count(),
        "communities": community_data,
        "grounding": "∂(Boundary) + μ(Mapping) + N(Quantity)",
    });
    Ok(CallToolResult::success(vec![Content::text(
        result.to_string(),
    )]))
}

fn analyze_scc(g: &Graph<String, f64>) -> Result<CallToolResult, McpError> {
    let sccs = graph::tarjan_scc(g);
    let multi_sccs: Vec<_> = sccs.iter().filter(|scc| scc.len() > 1).collect();
    let scc_data: Vec<_> = sccs
        .iter()
        .filter(|scc| scc.len() > 1)
        .enumerate()
        .map(|(i, scc)| {
            let labels: Vec<_> = scc.iter().filter_map(|&id| g.vertex(id).cloned()).collect();
            json!({
                "scc": i,
                "size": scc.len(),
                "vertices": labels,
            })
        })
        .collect();

    let result = json!({
        "analysis": "scc",
        "total_sccs": sccs.len(),
        "nontrivial_sccs": multi_sccs.len(),
        "vertex_count": g.vertex_count(),
        "nontrivial_components": scc_data,
        "interpretation": if multi_sccs.is_empty() {
            "No cycles detected — graph is a DAG".to_string()
        } else {
            format!("{} cycle(s) detected involving {} vertices",
                multi_sccs.len(),
                multi_sccs.iter().map(|s| s.len()).sum::<usize>())
        },
        "grounding": "ρ(Recursion) + ∂(Boundary) + μ(Mapping)",
    });
    Ok(CallToolResult::success(vec![Content::text(
        result.to_string(),
    )]))
}

fn analyze_topo_sort(g: &Graph<String, f64>) -> Result<CallToolResult, McpError> {
    match graph::topological_sort(g) {
        Some(order) => {
            let labels: Vec<_> = order
                .iter()
                .filter_map(|&id| g.vertex(id).cloned())
                .collect();
            let result = json!({
                "analysis": "topo_sort",
                "has_cycle": false,
                "order": labels,
                "grounding": "σ(Sequence) + μ(Mapping)",
            });
            Ok(CallToolResult::success(vec![Content::text(
                result.to_string(),
            )]))
        }
        None => {
            let result = json!({
                "analysis": "topo_sort",
                "has_cycle": true,
                "interpretation": "Graph contains cycles — no topological ordering exists",
            });
            Ok(CallToolResult::success(vec![Content::text(
                result.to_string(),
            )]))
        }
    }
}

/// Construct a graph from various data sources.
pub fn graph_construct(params: GraphConstructParams) -> Result<CallToolResult, McpError> {
    let format = params.format.to_lowercase();
    match format.as_str() {
        "edge_list" => construct_from_edges(&params.edges),
        "adjacency" => construct_from_adjacency(&params.adjacency, params.vertex_count),
        "co_occurrence" => construct_from_co_occurrence(&params.matrix_data, params.matrix_rows),
        other => Err(McpError::invalid_params(
            format!("Unknown format '{other}'. Use: edge_list, adjacency, co_occurrence"),
            None,
        )),
    }
}

fn construct_from_edges(edges: &[crate::params::EdgeInput]) -> Result<CallToolResult, McpError> {
    if edges.is_empty() {
        return Err(McpError::invalid_params("No edges provided", None));
    }
    let max_idx = edges.iter().map(|e| e.from.max(e.to)).max().unwrap_or(0);
    let vertex_count = max_idx + 1;
    let is_weighted = edges.iter().any(|e| e.weight.is_some());

    let result = json!({
        "format": "edge_list",
        "vertex_count": vertex_count,
        "edge_count": edges.len(),
        "is_directed": true,
        "is_weighted": is_weighted,
    });
    Ok(CallToolResult::success(vec![Content::text(
        result.to_string(),
    )]))
}

fn construct_from_adjacency(
    adjacency: &[Vec<usize>],
    vertex_count: Option<usize>,
) -> Result<CallToolResult, McpError> {
    let vc = vertex_count.unwrap_or(adjacency.len());
    let edge_count: usize = adjacency.iter().map(|adj| adj.len()).sum();

    let result = json!({
        "format": "adjacency",
        "vertex_count": vc,
        "edge_count": edge_count,
        "is_directed": true,
        "is_weighted": false,
    });
    Ok(CallToolResult::success(vec![Content::text(
        result.to_string(),
    )]))
}

fn construct_from_co_occurrence(
    matrix_data: &[f64],
    rows: Option<usize>,
) -> Result<CallToolResult, McpError> {
    let rows = rows.ok_or_else(|| {
        McpError::invalid_params("co_occurrence format requires matrix_rows", None)
    })?;
    if rows == 0 || matrix_data.len() % rows != 0 {
        return Err(McpError::invalid_params(
            format!(
                "{} elements not divisible by {} rows",
                matrix_data.len(),
                rows
            ),
            None,
        ));
    }
    let cols = matrix_data.len() / rows;
    let edge_count = matrix_data.iter().filter(|&&v| v > 0.0).count();

    let result = json!({
        "format": "co_occurrence",
        "vertex_count": rows,
        "columns": cols,
        "nonzero_edges": edge_count,
        "is_directed": rows != cols,
        "is_weighted": true,
    });
    Ok(CallToolResult::success(vec![Content::text(
        result.to_string(),
    )]))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::params::{EdgeInput, VertexInput};

    fn sample_edges() -> Vec<EdgeInput> {
        vec![
            EdgeInput {
                from: 0,
                to: 1,
                weight: Some(1.0),
            },
            EdgeInput {
                from: 0,
                to: 2,
                weight: Some(4.0),
            },
            EdgeInput {
                from: 1,
                to: 2,
                weight: Some(2.0),
            },
            EdgeInput {
                from: 1,
                to: 3,
                weight: Some(6.0),
            },
            EdgeInput {
                from: 2,
                to: 3,
                weight: Some(3.0),
            },
        ]
    }

    fn dag_edges() -> Vec<EdgeInput> {
        vec![
            EdgeInput {
                from: 0,
                to: 1,
                weight: None,
            },
            EdgeInput {
                from: 0,
                to: 2,
                weight: None,
            },
            EdgeInput {
                from: 1,
                to: 3,
                weight: None,
            },
            EdgeInput {
                from: 2,
                to: 3,
                weight: None,
            },
        ]
    }

    #[test]
    fn test_graph_centrality() {
        let params = GraphAnalyzeParams {
            vertices: vec![],
            edges: sample_edges(),
            analysis: "centrality".to_string(),
            from_vertex: None,
            to_vertex: None,
            damping: 0.85,
            max_iterations: 100,
            tolerance: 1e-6,
        };
        assert!(graph_analyze(params).is_ok());
    }

    #[test]
    fn test_graph_components() {
        let params = GraphAnalyzeParams {
            vertices: vec![],
            edges: dag_edges(),
            analysis: "components".to_string(),
            from_vertex: None,
            to_vertex: None,
            damping: 0.85,
            max_iterations: 100,
            tolerance: 1e-6,
        };
        assert!(graph_analyze(params).is_ok());
    }

    #[test]
    fn test_graph_shortest_path() {
        let params = GraphAnalyzeParams {
            vertices: vec![],
            edges: dag_edges(),
            analysis: "shortest_path".to_string(),
            from_vertex: Some(0),
            to_vertex: Some(3),
            damping: 0.85,
            max_iterations: 100,
            tolerance: 1e-6,
        };
        assert!(graph_analyze(params).is_ok());
    }

    #[test]
    fn test_graph_dijkstra() {
        let params = GraphAnalyzeParams {
            vertices: vec![],
            edges: sample_edges(),
            analysis: "dijkstra".to_string(),
            from_vertex: Some(0),
            to_vertex: Some(3),
            damping: 0.85,
            max_iterations: 100,
            tolerance: 1e-6,
        };
        assert!(graph_analyze(params).is_ok());
    }

    #[test]
    fn test_graph_pagerank() {
        let params = GraphAnalyzeParams {
            vertices: vec![],
            edges: sample_edges(),
            analysis: "pagerank".to_string(),
            from_vertex: None,
            to_vertex: None,
            damping: 0.85,
            max_iterations: 100,
            tolerance: 1e-6,
        };
        assert!(graph_analyze(params).is_ok());
    }

    #[test]
    fn test_graph_communities() {
        let params = GraphAnalyzeParams {
            vertices: vec![],
            edges: sample_edges(),
            analysis: "communities".to_string(),
            from_vertex: None,
            to_vertex: None,
            damping: 0.85,
            max_iterations: 100,
            tolerance: 1e-6,
        };
        assert!(graph_analyze(params).is_ok());
    }

    #[test]
    fn test_graph_scc() {
        let params = GraphAnalyzeParams {
            vertices: vec![],
            edges: vec![
                EdgeInput {
                    from: 0,
                    to: 1,
                    weight: None,
                },
                EdgeInput {
                    from: 1,
                    to: 0,
                    weight: None,
                },
                EdgeInput {
                    from: 2,
                    to: 3,
                    weight: None,
                },
            ],
            analysis: "scc".to_string(),
            from_vertex: None,
            to_vertex: None,
            damping: 0.85,
            max_iterations: 100,
            tolerance: 1e-6,
        };
        assert!(graph_analyze(params).is_ok());
    }

    #[test]
    fn test_graph_topo_sort() {
        let params = GraphAnalyzeParams {
            vertices: vec![],
            edges: dag_edges(),
            analysis: "topo_sort".to_string(),
            from_vertex: None,
            to_vertex: None,
            damping: 0.85,
            max_iterations: 100,
            tolerance: 1e-6,
        };
        assert!(graph_analyze(params).is_ok());
    }

    #[test]
    fn test_graph_construct_edge_list() {
        let params = GraphConstructParams {
            format: "edge_list".to_string(),
            edges: sample_edges(),
            adjacency: vec![],
            matrix_data: vec![],
            matrix_rows: None,
            vertex_count: None,
        };
        assert!(graph_construct(params).is_ok());
    }

    #[test]
    fn test_graph_construct_adjacency() {
        let params = GraphConstructParams {
            format: "adjacency".to_string(),
            edges: vec![],
            adjacency: vec![vec![1, 2], vec![2], vec![]],
            matrix_data: vec![],
            matrix_rows: None,
            vertex_count: Some(3),
        };
        assert!(graph_construct(params).is_ok());
    }

    #[test]
    fn test_invalid_analysis() {
        let params = GraphAnalyzeParams {
            vertices: vec![],
            edges: dag_edges(),
            analysis: "invalid".to_string(),
            from_vertex: None,
            to_vertex: None,
            damping: 0.85,
            max_iterations: 100,
            tolerance: 1e-6,
        };
        assert!(graph_analyze(params).is_err());
    }
}
