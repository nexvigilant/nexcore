//! Mesh networking endpoints.
//!
//! Exposes nexcore-mesh adaptive networking: topology simulation,
//! route quality scoring, grounding coverage, and node address operations.
//!
//! ## Endpoints
//! - `GET  /health`        — Mesh subsystem health
//! - `GET  /topology`      — Current topology (nodes + edges)
//! - `POST /simulate`      — Simulate a mesh with given topology
//! - `POST /route-quality`  — Compute route quality score
//! - `GET  /grounding`     — T1 primitive coverage
//! - `POST /node-info`     — Query node address properties

use axum::{
    Json, Router,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use nexcore_mesh::Route;
use nexcore_mesh::routing::RoutingTable;
use nexcore_mesh::topology::{AddressExt, Path as MeshPath, RouteQuality};
use nexcore_primitives::transfer::TopologicalAddress;

use super::common::ApiError;

// ─── Request/Response Types ──────────────────────────────────

/// Mesh health response.
#[derive(Debug, Serialize, ToSchema)]
pub struct MeshHealthResponse {
    /// Whether the mesh subsystem is available
    pub available: bool,
    /// Mesh crate version description
    pub version: String,
    /// Number of GroundsTo implementations
    pub groundings: u32,
    /// T1 primitive coverage ratio (e.g. "14/16")
    pub t1_coverage: String,
}

/// Simulated node in topology response.
#[derive(Debug, Serialize, ToSchema)]
pub struct TopologyNode {
    /// Node identifier
    pub id: String,
    /// Current lifecycle state
    pub state: String,
    /// Number of neighbors
    pub neighbor_count: usize,
}

/// Edge between two nodes.
#[derive(Debug, Serialize, ToSchema)]
pub struct TopologyEdge {
    /// Source node ID
    pub from: String,
    /// Target node ID
    pub to: String,
    /// Route quality score (0.0 - 1.0)
    pub quality: f64,
}

/// Topology response.
#[derive(Debug, Serialize, ToSchema)]
pub struct TopologyResponse {
    /// All nodes in the mesh
    pub nodes: Vec<TopologyNode>,
    /// All edges (routes) between nodes
    pub edges: Vec<TopologyEdge>,
    /// Total node count
    pub node_count: usize,
    /// Total edge count
    pub edge_count: usize,
}

/// Simulate request — describe a topology to build.
#[derive(Debug, Deserialize, ToSchema)]
pub struct SimulateRequest {
    /// Node IDs to include in the mesh
    pub nodes: Vec<String>,
    /// Edges as (from, to, latency_ms, reliability) tuples
    pub edges: Vec<SimulateEdge>,
}

/// Edge specification for simulation.
#[derive(Debug, Deserialize, ToSchema)]
pub struct SimulateEdge {
    /// Source node ID
    pub from: String,
    /// Target node ID
    pub to: String,
    /// Latency in milliseconds
    pub latency_ms: f64,
    /// Reliability (0.0 - 1.0)
    pub reliability: f64,
}

/// Simulate response.
#[derive(Debug, Serialize, ToSchema)]
pub struct SimulateResponse {
    /// Number of nodes in the simulated mesh
    pub node_count: usize,
    /// Number of routes created
    pub route_count: usize,
    /// Routes with computed quality scores
    pub routes: Vec<SimulateRouteResult>,
}

/// A route result from simulation.
#[derive(Debug, Serialize, ToSchema)]
pub struct SimulateRouteResult {
    /// Destination node ID
    pub destination: String,
    /// Next hop node ID
    pub next_hop: String,
    /// Computed quality score
    pub quality_score: f64,
    /// Latency in ms
    pub latency_ms: f64,
    /// Reliability (0.0 - 1.0)
    pub reliability: f64,
    /// Hop count
    pub hop_count: u8,
}

/// Route quality computation request.
#[derive(Debug, Deserialize, ToSchema)]
pub struct RouteQualityRequest {
    /// Latency in milliseconds
    pub latency_ms: f64,
    /// Packet delivery reliability (0.0 - 1.0)
    pub reliability: f64,
    /// Number of hops
    pub hop_count: u8,
}

/// Route quality response.
#[derive(Debug, Serialize, ToSchema)]
pub struct RouteQualityResponse {
    /// Composite quality score (higher = better)
    pub score: f64,
    /// Whether the route is considered usable
    pub usable: bool,
    /// Input latency
    pub latency_ms: f64,
    /// Input reliability
    pub reliability: f64,
    /// Input hop count
    pub hop_count: u8,
    /// Score formula description
    pub formula: String,
}

/// Grounding entry for API response.
#[derive(Debug, Serialize, ToSchema)]
pub struct GroundingEntry {
    /// Type name
    pub type_name: String,
    /// Tier classification (T1, T2-P, T2-C, T3)
    pub tier: String,
    /// Dominant primitive symbol
    pub dominant: String,
    /// All primitives in composition
    pub primitives: Vec<String>,
    /// Confidence score
    pub confidence: f64,
}

/// Grounding response.
#[derive(Debug, Serialize, ToSchema)]
pub struct GroundingResponse {
    /// Total GroundsTo implementations
    pub total: usize,
    /// T1 primitive coverage
    pub t1_coverage: String,
    /// Tier distribution
    pub tier_distribution: TierDistribution,
    /// All grounding entries
    pub entries: Vec<GroundingEntry>,
}

/// Distribution across tiers.
#[derive(Debug, Serialize, ToSchema)]
pub struct TierDistribution {
    /// T1 count
    pub t1: usize,
    /// T2-P count
    pub t2_p: usize,
    /// T2-C count
    pub t2_c: usize,
    /// T3 count
    pub t3: usize,
}

/// Node info request.
#[derive(Debug, Deserialize, ToSchema)]
pub struct NodeInfoRequest {
    /// Node address (dot-separated hierarchical path)
    pub address: String,
    /// Address separator (default ".")
    pub separator: Option<String>,
    /// Optional second address for distance calculation
    pub compare_to: Option<String>,
}

/// Node info response.
#[derive(Debug, Serialize, ToSchema)]
pub struct NodeInfoResponse {
    /// Parsed address
    pub address: String,
    /// Address depth (number of segments)
    pub depth: usize,
    /// Address segments
    pub segments: Vec<String>,
    /// Ancestor addresses
    pub ancestors: Vec<String>,
    /// Distance to compare_to address (if provided)
    pub distance: Option<usize>,
    /// Whether the two addresses are neighbors (if compare_to provided)
    pub is_neighbor: Option<bool>,
}

// ─── Router ──────────────────────────────────────────────────

/// Mesh networking router. Nested under `/api/v1/mesh`.
pub fn router() -> axum::Router<crate::ApiState> {
    Router::new()
        .route("/health", get(mesh_health))
        .route("/topology", get(mesh_topology))
        .route("/simulate", post(mesh_simulate))
        .route("/route-quality", post(mesh_route_quality))
        .route("/grounding", get(mesh_grounding))
        .route("/node-info", post(mesh_node_info))
}

// ─── Handlers ────────────────────────────────────────────────

/// Mesh subsystem health.
#[utoipa::path(
    get,
    path = "/api/v1/mesh/health",
    tag = "mesh",
    responses(
        (status = 200, description = "Mesh subsystem health", body = MeshHealthResponse)
    )
)]
pub async fn mesh_health() -> Json<MeshHealthResponse> {
    Json(MeshHealthResponse {
        available: true,
        version: "nexcore-mesh 14/16 T1 coverage".to_string(),
        groundings: 28,
        t1_coverage: "14/16".to_string(),
    })
}

/// Current mesh topology (static view).
///
/// Returns a sample topology since there is no live mesh runtime
/// in the API server. Use `/simulate` for dynamic topology building.
#[utoipa::path(
    get,
    path = "/api/v1/mesh/topology",
    tag = "mesh",
    responses(
        (status = 200, description = "Current mesh topology", body = TopologyResponse)
    )
)]
pub async fn mesh_topology() -> Json<TopologyResponse> {
    // Return empty topology — no live mesh in API server
    Json(TopologyResponse {
        nodes: Vec::new(),
        edges: Vec::new(),
        node_count: 0,
        edge_count: 0,
    })
}

/// Simulate a mesh with given nodes and edges.
#[utoipa::path(
    post,
    path = "/api/v1/mesh/simulate",
    tag = "mesh",
    request_body = SimulateRequest,
    responses(
        (status = 200, description = "Simulation complete", body = SimulateResponse),
        (status = 400, description = "Invalid topology", body = super::common::ApiError)
    )
)]
pub async fn mesh_simulate(
    Json(req): Json<SimulateRequest>,
) -> Result<Json<SimulateResponse>, ApiError> {
    if req.nodes.is_empty() {
        return Err(ApiError::new(
            "VALIDATION_ERROR",
            "at least one node required",
        ));
    }

    let routing = RoutingTable::with_defaults();

    for edge in &req.edges {
        // Validate node references
        if !req.nodes.contains(&edge.from) {
            return Err(ApiError::new(
                "VALIDATION_ERROR",
                format!("edge source '{}' not in node list", edge.from),
            ));
        }
        if !req.nodes.contains(&edge.to) {
            return Err(ApiError::new(
                "VALIDATION_ERROR",
                format!("edge target '{}' not in node list", edge.to),
            ));
        }

        let mut path = MeshPath::new(&edge.from, 16);
        let _ = path.add_hop(&edge.to);
        let quality = RouteQuality::new(edge.latency_ms, edge.reliability, 1);
        let route = Route::new(&edge.to, &edge.from, path, quality);
        routing.upsert(route);
    }

    let routes: Vec<SimulateRouteResult> = routing
        .snapshot()
        .iter()
        .flat_map(|(dest, route_list)| {
            route_list.iter().map(move |r| SimulateRouteResult {
                destination: dest.clone(),
                next_hop: r.next_hop.clone(),
                quality_score: r.quality.score(),
                latency_ms: r.quality.latency_ms,
                reliability: r.quality.reliability,
                hop_count: r.quality.hop_count,
            })
        })
        .collect();

    let route_count = routes.len();
    Ok(Json(SimulateResponse {
        node_count: req.nodes.len(),
        route_count,
        routes,
    }))
}

/// Compute route quality score from metrics.
#[utoipa::path(
    post,
    path = "/api/v1/mesh/route-quality",
    tag = "mesh",
    request_body = RouteQualityRequest,
    responses(
        (status = 200, description = "Route quality computed", body = RouteQualityResponse)
    )
)]
pub async fn mesh_route_quality(
    Json(req): Json<RouteQualityRequest>,
) -> Json<RouteQualityResponse> {
    let quality = RouteQuality::new(req.latency_ms, req.reliability, req.hop_count);
    Json(RouteQualityResponse {
        score: quality.score(),
        usable: quality.is_usable(),
        latency_ms: quality.latency_ms,
        reliability: quality.reliability,
        hop_count: quality.hop_count,
        formula: "reliability * (1/(1+latency_ms/1000)) * (1/(1+hop_count))".to_string(),
    })
}

/// Show T1 primitive grounding coverage for mesh types.
#[utoipa::path(
    get,
    path = "/api/v1/mesh/grounding",
    tag = "mesh",
    responses(
        (status = 200, description = "Grounding coverage", body = GroundingResponse)
    )
)]
pub async fn mesh_grounding() -> Json<GroundingResponse> {
    // Static grounding data from nexcore-mesh grounding.rs (28 impls)
    let entries = vec![
        // T1
        grounding_entry("NodeState", "T1", "ς", &["ς"], 1.0),
        grounding_entry("TlsTier", "T1", "ς", &["ς"], 1.0),
        // T2-P
        grounding_entry("Path", "T2-P", "σ", &["σ", "∂"], 0.80),
        grounding_entry("MeshError", "T2-P", "∂", &["∂", "∅"], 0.85),
        grounding_entry("RouteQuality", "T2-P", "κ", &["κ", "N"], 0.75),
        grounding_entry("Neighbor", "T2-P", "∃", &["∃", "κ"], 0.80),
        grounding_entry("Route", "T2-P", "μ", &["μ", "σ", "κ"], 0.75),
        grounding_entry("PeerIdentity", "T2-P", "λ", &["λ", "∂"], 0.85),
        grounding_entry("DiscoveryAction", "T2-P", "ν", &["ν", "∃"], 0.80),
        // T2-C
        grounding_entry("MeshMessage", "T2-C", "σ", &["σ", "λ", "μ", "→"], 0.75),
        grounding_entry("RoutingTable", "T2-C", "μ", &["μ", "σ", "κ", "∂"], 0.80),
        grounding_entry("NeighborRegistry", "T2-C", "μ", &["μ", "∃", "∂", "κ"], 0.75),
        grounding_entry("DiscoveryMessage", "T2-C", "ν", &["ν", "∃", "λ", "→"], 0.70),
        grounding_entry("GossipMessage", "T2-C", "ν", &["ν", "μ", "σ", "→"], 0.70),
        grounding_entry("ResilienceState", "T2-C", "ρ", &["ρ", "ς", "∂", "ν"], 0.70),
        grounding_entry("ResilienceAction", "T2-C", "ρ", &["ρ", "∂", "ν", "ς"], 0.70),
        grounding_entry("SecurityPolicy", "T2-C", "∂", &["∂", "κ", "ς", "→"], 0.80),
        grounding_entry(
            "ChemotacticRouteSelector",
            "T2-C",
            "κ",
            &["κ", "N", "Σ", "→"],
            0.75,
        ),
        grounding_entry("MeshSnapshot", "T2-C", "π", &["π", "ς", "μ", "σ"], 0.80),
        // T3
        grounding_entry("Node", "T3", "ς", &["ς", "μ", "σ", "∂", "ν", "ρ"], 0.90),
        grounding_entry(
            "DiscoveryLoop",
            "T3",
            "ν",
            &["ν", "∃", "μ", "ρ", "→", "∂"],
            0.85,
        ),
        grounding_entry(
            "GossipLoop",
            "T3",
            "ν",
            &["ν", "μ", "ρ", "σ", "→", "∂"],
            0.85,
        ),
        grounding_entry(
            "ResilienceLoop",
            "T3",
            "ρ",
            &["ρ", "ς", "∂", "ν", "→", "μ"],
            0.85,
        ),
        grounding_entry(
            "MeshEvent",
            "T3",
            "→",
            &["→", "ς", "∃", "μ", "ν", "∂"],
            0.80,
        ),
        grounding_entry(
            "MeshHandle",
            "T3",
            "→",
            &["→", "ς", "ν", "∂", "μ", "ρ"],
            0.75,
        ),
        grounding_entry(
            "MeshRuntime",
            "T3",
            "ρ",
            &["ρ", "ν", "μ", "ς", "∂", "→"],
            0.85,
        ),
        grounding_entry(
            "SnapshotStore",
            "T3",
            "π",
            &["π", "μ", "∂", "→", "λ", "ς"],
            0.80,
        ),
        grounding_entry(
            "ChemotacticRouter",
            "T3",
            "ρ",
            &["ρ", "→", "λ", "κ", "μ", "Σ"],
            0.80,
        ),
    ];

    let total = entries.len();
    let t1 = entries.iter().filter(|e| e.tier == "T1").count();
    let t2_p = entries.iter().filter(|e| e.tier == "T2-P").count();
    let t2_c = entries.iter().filter(|e| e.tier == "T2-C").count();
    let t3 = entries.iter().filter(|e| e.tier == "T3").count();

    Json(GroundingResponse {
        total,
        t1_coverage: "14/16".to_string(),
        tier_distribution: TierDistribution { t1, t2_p, t2_c, t3 },
        entries,
    })
}

/// Query node address properties.
#[utoipa::path(
    post,
    path = "/api/v1/mesh/node-info",
    tag = "mesh",
    request_body = NodeInfoRequest,
    responses(
        (status = 200, description = "Node info", body = NodeInfoResponse),
        (status = 400, description = "Invalid address", body = super::common::ApiError)
    )
)]
pub async fn mesh_node_info(
    Json(req): Json<NodeInfoRequest>,
) -> Result<Json<NodeInfoResponse>, ApiError> {
    let sep = req.separator.as_deref().unwrap_or(".");

    if req.address.is_empty() {
        return Err(ApiError::new(
            "VALIDATION_ERROR",
            "address must not be empty",
        ));
    }

    let addr = TopologicalAddress::parse(&req.address, sep);
    let ancestors: Vec<String> = addr.ancestors().iter().map(|a| a.render()).collect();

    let (distance, is_neighbor) = if let Some(ref compare) = req.compare_to {
        let other = TopologicalAddress::parse(compare, sep);
        (
            Some(addr.mesh_distance(&other)),
            Some(addr.is_neighbor(&other)),
        )
    } else {
        (None, None)
    };

    Ok(Json(NodeInfoResponse {
        address: addr.render(),
        depth: addr.depth(),
        segments: addr.segments.clone(),
        ancestors,
        distance,
        is_neighbor,
    }))
}

// ─── Helpers ─────────────────────────────────────────────────

fn grounding_entry(
    type_name: &str,
    tier: &str,
    dominant: &str,
    primitives: &[&str],
    confidence: f64,
) -> GroundingEntry {
    GroundingEntry {
        type_name: type_name.to_string(),
        tier: tier.to_string(),
        dominant: dominant.to_string(),
        primitives: primitives.iter().map(|p| p.to_string()).collect(),
        confidence,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn health_returns_available() {
        let Json(resp) = mesh_health().await;
        assert!(resp.available);
        assert_eq!(resp.groundings, 28);
        assert_eq!(resp.t1_coverage, "14/16");
    }

    #[tokio::test]
    async fn topology_returns_empty() {
        let Json(resp) = mesh_topology().await;
        assert_eq!(resp.node_count, 0);
        assert_eq!(resp.edge_count, 0);
    }

    #[tokio::test]
    async fn simulate_ring_topology() {
        let req = SimulateRequest {
            nodes: vec!["a".into(), "b".into(), "c".into()],
            edges: vec![
                SimulateEdge {
                    from: "a".into(),
                    to: "b".into(),
                    latency_ms: 10.0,
                    reliability: 0.95,
                },
                SimulateEdge {
                    from: "b".into(),
                    to: "c".into(),
                    latency_ms: 20.0,
                    reliability: 0.90,
                },
                SimulateEdge {
                    from: "c".into(),
                    to: "a".into(),
                    latency_ms: 15.0,
                    reliability: 0.92,
                },
            ],
        };
        let result = mesh_simulate(Json(req)).await;
        assert!(result.is_ok());
        if let Ok(Json(resp)) = result {
            assert_eq!(resp.node_count, 3);
            assert_eq!(resp.route_count, 3);
        }
    }

    #[tokio::test]
    async fn simulate_empty_nodes_fails() {
        let req = SimulateRequest {
            nodes: Vec::new(),
            edges: Vec::new(),
        };
        let result = mesh_simulate(Json(req)).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn simulate_invalid_edge_ref_fails() {
        let req = SimulateRequest {
            nodes: vec!["a".into()],
            edges: vec![SimulateEdge {
                from: "a".into(),
                to: "z".into(),
                latency_ms: 10.0,
                reliability: 0.9,
            }],
        };
        let result = mesh_simulate(Json(req)).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn route_quality_computation() {
        let req = RouteQualityRequest {
            latency_ms: 50.0,
            reliability: 0.95,
            hop_count: 2,
        };
        let Json(resp) = mesh_route_quality(Json(req)).await;
        assert!(resp.score > 0.0);
        assert!(resp.score < 1.0);
        assert!(resp.usable);
        assert!(resp.formula.contains("reliability"));
    }

    #[tokio::test]
    async fn route_quality_unusable() {
        let req = RouteQualityRequest {
            latency_ms: 10000.0,
            reliability: 0.05,
            hop_count: 10,
        };
        let Json(resp) = mesh_route_quality(Json(req)).await;
        assert!(!resp.usable);
    }

    #[tokio::test]
    async fn grounding_returns_28_entries() {
        let Json(resp) = mesh_grounding().await;
        assert_eq!(resp.total, 28);
        assert_eq!(resp.t1_coverage, "14/16");
        assert_eq!(resp.tier_distribution.t1, 2);
        // T2-P + T2-C + T3 = 26
        let sum =
            resp.tier_distribution.t2_p + resp.tier_distribution.t2_c + resp.tier_distribution.t3;
        assert_eq!(sum, 26);
    }

    #[tokio::test]
    async fn node_info_basic() {
        let req = NodeInfoRequest {
            address: "mesh.region1.node1".into(),
            separator: Some(".".into()),
            compare_to: None,
        };
        let result = mesh_node_info(Json(req)).await;
        assert!(result.is_ok());
        if let Ok(Json(resp)) = result {
            assert_eq!(resp.depth, 3);
            assert_eq!(resp.segments.len(), 3);
            assert_eq!(resp.ancestors.len(), 2);
            assert!(resp.distance.is_none());
        }
    }

    #[tokio::test]
    async fn node_info_with_comparison() {
        let req = NodeInfoRequest {
            address: "mesh.r1.n1".into(),
            separator: Some(".".into()),
            compare_to: Some("mesh.r1.n2".into()),
        };
        let result = mesh_node_info(Json(req)).await;
        assert!(result.is_ok());
        if let Ok(Json(resp)) = result {
            assert_eq!(resp.distance, Some(2)); // siblings: 1 up + 1 down
            assert_eq!(resp.is_neighbor, Some(true));
        }
    }

    #[tokio::test]
    async fn node_info_empty_address_fails() {
        let req = NodeInfoRequest {
            address: String::new(),
            separator: None,
            compare_to: None,
        };
        let result = mesh_node_info(Json(req)).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn node_info_different_regions_not_neighbors() {
        let req = NodeInfoRequest {
            address: "mesh.r1.n1".into(),
            separator: None,
            compare_to: Some("mesh.r2.n1".into()),
        };
        let result = mesh_node_info(Json(req)).await;
        assert!(result.is_ok());
        if let Ok(Json(resp)) = result {
            assert_eq!(resp.is_neighbor, Some(false));
            assert_eq!(resp.distance, Some(4)); // 2 up + 2 down
        }
    }
}
