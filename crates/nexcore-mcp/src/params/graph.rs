//! Graph analysis and construction MCP tool parameters
//! Tier: T3 (Domain-specific MCP tool parameters)
//!
//! Unified graph analysis covering centrality, components, shortest paths,
//! PageRank, community detection, SCC, and topological sort.

use rmcp::schemars::{self, JsonSchema};
use rmcp::serde::{Deserialize, Serialize};

/// Edge definition for graph construction from MCP input.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct EdgeInput {
    /// Source vertex index (0-based).
    pub from: usize,
    /// Target vertex index (0-based).
    pub to: usize,
    /// Optional edge weight (defaults to 1.0 for unweighted).
    #[serde(default)]
    pub weight: Option<f64>,
}

/// Vertex definition for graph construction from MCP input.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct VertexInput {
    /// Vertex label (optional).
    #[serde(default)]
    pub label: Option<String>,
}

/// Parameters for the unified graph_analyze MCP tool.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct GraphAnalyzeParams {
    /// Vertex definitions. If empty, vertices are auto-discovered from edges.
    #[serde(default)]
    pub vertices: Vec<VertexInput>,

    /// Edge definitions (required).
    pub edges: Vec<EdgeInput>,

    /// Analysis mode:
    /// - "centrality": betweenness centrality for all vertices
    /// - "components": connected components
    /// - "shortest_path": BFS shortest path (requires from_vertex, to_vertex)
    /// - "dijkstra": weighted shortest path (requires from_vertex, to_vertex)
    /// - "pagerank": iterative PageRank
    /// - "communities": Louvain community detection
    /// - "scc": strongly connected components
    /// - "topo_sort": topological sort (returns None if cycles exist)
    pub analysis: String,

    /// Source vertex for path queries (0-based index).
    #[serde(default)]
    pub from_vertex: Option<usize>,

    /// Target vertex for path queries (0-based index).
    #[serde(default)]
    pub to_vertex: Option<usize>,

    /// PageRank damping factor (default: 0.85).
    #[serde(default = "default_damping")]
    pub damping: f64,

    /// PageRank max iterations (default: 100).
    #[serde(default = "default_max_iterations")]
    pub max_iterations: usize,

    /// PageRank convergence tolerance (default: 1e-6).
    #[serde(default = "default_tolerance")]
    pub tolerance: f64,
}

fn default_damping() -> f64 {
    0.85
}

fn default_max_iterations() -> usize {
    100
}

fn default_tolerance() -> f64 {
    1e-6
}

/// Parameters for graph_construct MCP tool.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct GraphConstructParams {
    /// Construction format:
    /// - "edge_list": edges as array of {from, to, weight?}
    /// - "adjacency": adjacency list as array of arrays of neighbor indices
    /// - "co_occurrence": co-occurrence matrix (flat row-major, with row count)
    pub format: String,

    /// Edge list (for "edge_list" format).
    #[serde(default)]
    pub edges: Vec<EdgeInput>,

    /// Adjacency list (for "adjacency" format). Each inner vec is neighbor indices.
    #[serde(default)]
    pub adjacency: Vec<Vec<usize>>,

    /// Flat co-occurrence matrix data (for "co_occurrence" format).
    #[serde(default)]
    pub matrix_data: Vec<f64>,

    /// Number of rows in co-occurrence matrix.
    #[serde(default)]
    pub matrix_rows: Option<usize>,

    /// Number of vertices (required for adjacency and co_occurrence formats).
    #[serde(default)]
    pub vertex_count: Option<usize>,
}
