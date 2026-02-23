//! Parameter structs for topology MCP tools.

use rmcp::schemars::JsonSchema;
use serde::Deserialize;

/// Parameters for building a distance matrix and computing Vietoris-Rips complex
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct TopoVietorisRipsParams {
    /// Pairwise distances as a square matrix (array of arrays)
    pub distances: Vec<Vec<f64>>,
    /// Optional labels for each point (must match matrix size)
    pub labels: Option<Vec<String>>,
    /// Maximum simplex dimension (1=edges, 2=triangles, default 2)
    pub max_dim: Option<usize>,
    /// Maximum filtration value (default: f64::MAX — include all)
    pub max_filtration: Option<f64>,
}

/// Parameters for computing persistent homology from a distance matrix
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct TopoPersistenceParams {
    /// Pairwise distances as a square matrix
    pub distances: Vec<Vec<f64>>,
    /// Optional labels for each point
    pub labels: Option<Vec<String>>,
    /// Maximum simplex dimension (default 2)
    pub max_dim: Option<usize>,
    /// Maximum filtration value (default: auto from max distance)
    pub max_filtration: Option<f64>,
    /// Minimum persistence to report (filters noise, default 0.0)
    pub min_persistence: Option<f64>,
}

/// Parameters for computing Betti numbers at a specific filtration value
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct TopoBettiParams {
    /// Pairwise distances as a square matrix
    pub distances: Vec<Vec<f64>>,
    /// Filtration value at which to evaluate Betti numbers
    pub at_filtration: f64,
    /// Maximum simplex dimension (default 2)
    pub max_dim: Option<usize>,
}

/// Parameters for graph centrality (betweenness) computation
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct GraphCentralityParams {
    /// Node names
    pub nodes: Vec<String>,
    /// Edges as [from, to] pairs
    pub edges: Vec<(String, String)>,
}

/// Parameters for connected components
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct GraphComponentsParams {
    /// Node names
    pub nodes: Vec<String>,
    /// Edges as [from, to] pairs (treated as undirected)
    pub edges: Vec<(String, String)>,
}

/// Parameters for shortest path
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct GraphShortestPathParams {
    /// Edges as [from, to] pairs
    pub edges: Vec<(String, String)>,
    /// Source node
    pub from: String,
    /// Target node
    pub to: String,
}
