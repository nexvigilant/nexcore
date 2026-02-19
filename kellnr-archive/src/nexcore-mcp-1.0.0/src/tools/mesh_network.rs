//! Mesh Network MCP Tools — Runtime mesh networking queries
//!
//! Exposes nexcore-mesh capabilities: route quality computation, node simulation,
//! and T1 primitive grounding analysis.
//!
//! ## Primitive Foundation
//!
//! | Primitive | Role |
//! |-----------|------|
//! | λ Location | Node addresses, route destinations |
//! | μ Mapping | Routing tables, neighbor registries |
//! | σ Sequence | Hop paths, message relay chains |
//! | ∂ Boundary | TTL limits, circuit breakers |
//! | ρ Recursion | Multi-hop relay, self-correcting resilience |
//! | ν Frequency | Heartbeats, discovery/gossip intervals |
//! | ∃ Existence | Node liveness, neighbor verification |
//! | ς State | Node lifecycle, circuit breaker state |
//! | κ Comparison | Route quality scoring, best-path selection |
//! | → Causality | Message propagation, causal relay chains |

use crate::params::{
    MeshNetworkNodeInfoParams, MeshNetworkRouteQualityParams, MeshNetworkSimulateParams,
};
use nexcore_mesh::{
    MeshConfig, Node, NodeState, Route, TopologicalAddress, topology::RouteQuality,
};
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use serde_json::json;

/// Simulate a mesh network with given topology.
///
/// Creates N nodes, connects them per topology, and returns network state.
/// Tier: T3 (Domain-specific MCP tool)
pub fn simulate(params: MeshNetworkSimulateParams) -> Result<CallToolResult, McpError> {
    let node_count = params.node_count.clamp(2, 20);
    let topology = params.topology.to_lowercase();
    let duration_ms = params.duration_ms.clamp(10, 5000);

    // Build node IDs
    let node_ids: Vec<String> = (0..node_count).map(|i| format!("node-{}", i + 1)).collect();

    // Build adjacency based on topology
    let edges: Vec<(usize, usize)> = match topology.as_str() {
        "ring" => {
            let mut e = Vec::new();
            for i in 0..node_count {
                e.push((i, (i + 1) % node_count));
            }
            e
        }
        "star" => {
            let mut e = Vec::new();
            for i in 1..node_count {
                e.push((0, i));
            }
            e
        }
        "full" => {
            let mut e = Vec::new();
            for i in 0..node_count {
                for j in (i + 1)..node_count {
                    e.push((i, j));
                }
            }
            e
        }
        "line" => {
            let mut e = Vec::new();
            for i in 0..(node_count - 1) {
                e.push((i, i + 1));
            }
            e
        }
        "random" => {
            // Ensure connected: build spanning tree + random extras
            let mut e: Vec<(usize, usize)> = Vec::new();
            for i in 1..node_count {
                e.push((i - 1, i));
            }
            // Add ~30% extra edges
            let extra = node_count / 3;
            for k in 0..extra {
                let a = k % node_count;
                let b = (k * 3 + 2) % node_count;
                if a != b {
                    e.push((a, b));
                }
            }
            e
        }
        _ => {
            return Ok(CallToolResult::success(vec![Content::text(format!(
                "Unknown topology '{}'. Supported: ring, star, full, line, random",
                topology
            ))]));
        }
    };

    // Create nodes
    let config = MeshConfig::default();
    let nodes: Vec<Node> = node_ids
        .iter()
        .map(|id| Node::new(id.clone(), config.clone()))
        .collect();

    // Compute network statistics
    let edge_count = edges.len();
    let density = if node_count > 1 {
        (2.0 * edge_count as f64) / (node_count as f64 * (node_count as f64 - 1.0))
    } else {
        0.0
    };

    // Compute average degree
    let mut degrees = vec![0usize; node_count];
    for &(a, b) in &edges {
        if a < node_count {
            degrees[a] += 1;
        }
        if b < node_count {
            degrees[b] += 1;
        }
    }
    let avg_degree = if node_count > 0 {
        degrees.iter().sum::<usize>() as f64 / node_count as f64
    } else {
        0.0
    };

    // Compute diameter estimate (max shortest path for simple topologies)
    let diameter = match topology.as_str() {
        "ring" => node_count / 2,
        "star" => {
            if node_count > 2 {
                2
            } else {
                1
            }
        }
        "full" => 1,
        "line" => node_count - 1,
        _ => node_count - 1, // conservative estimate
    };

    // Build node info
    let node_info: Vec<serde_json::Value> = nodes
        .iter()
        .enumerate()
        .map(|(i, n)| {
            json!({
                "index": i,
                "id": n.id,
                "state": format!("{:?}", n.state),
                "neighbors": degrees.get(i).copied().unwrap_or(0),
            })
        })
        .collect();

    // Build edge info
    let edge_info: Vec<serde_json::Value> = edges
        .iter()
        .map(|&(a, b)| {
            json!({
                "from": node_ids.get(a).cloned().unwrap_or_default(),
                "to": node_ids.get(b).cloned().unwrap_or_default(),
            })
        })
        .collect();

    let result = json!({
        "topology": topology,
        "node_count": node_count,
        "edge_count": edge_count,
        "density": format!("{:.3}", density),
        "average_degree": format!("{:.2}", avg_degree),
        "diameter_estimate": diameter,
        "duration_ms": duration_ms,
        "nodes": node_info,
        "edges": edge_info,
        "primitive_grounding": {
            "dominant": "λ Location + μ Mapping",
            "composition": "λ(addresses) + μ(adjacency) + σ(paths) + ∂(TTL) + κ(quality)",
            "t1_coverage": "13/16"
        }
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{}".to_string()),
    )]))
}

/// Compute route quality from latency, reliability, and hop count.
///
/// Returns the quality score and component analysis.
/// Tier: T2-C (Cross-domain composite tool)
pub fn route_quality(params: MeshNetworkRouteQualityParams) -> Result<CallToolResult, McpError> {
    let latency_ms = params.latency_ms.clamp(0.0, 10000.0);
    let reliability = params.reliability.clamp(0.0, 1.0);
    let hop_count = params.hop_count.max(1);

    let quality = RouteQuality::new(latency_ms, reliability, hop_count);
    let score = quality.score();

    // Classify quality
    let classification = if score >= 0.8 {
        "Excellent"
    } else if score >= 0.6 {
        "Good"
    } else if score >= 0.4 {
        "Acceptable"
    } else if score >= 0.2 {
        "Poor"
    } else {
        "Critical"
    };

    // Match actual formula: reliability * (1/(1+latency/1000)) * (1/(1+hops))
    let latency_factor = 1.0 / (1.0 + latency_ms / 1000.0);
    let hop_factor = 1.0 / (1.0 + hop_count as f64);

    let result = json!({
        "score": format!("{:.4}", score),
        "classification": classification,
        "inputs": {
            "latency_ms": latency_ms,
            "reliability": reliability,
            "hop_count": hop_count,
        },
        "analysis": {
            "latency_factor": format!("{:.4}", latency_factor),
            "reliability_factor": format!("{:.4}", reliability),
            "hop_factor": format!("{:.4}", hop_factor),
            "formula": "score = reliability × (1/(1+latency/1000)) × (1/(1+hops))",
        },
        "primitive_grounding": {
            "dominant": "κ Comparison",
            "composition": "κ(scoring) + N(metrics) + ν(latency) + ∃(liveness)",
        }
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{}".to_string()),
    )]))
}

/// Show grounding analysis for all mesh types with GroundsTo impls.
///
/// Returns T1 primitive coverage, tier distribution, and per-type breakdown.
/// Tier: T3 (Domain-specific MCP tool)
pub fn grounding() -> Result<CallToolResult, McpError> {
    use nexcore_lex_primitiva::grounding::GroundsTo;
    use nexcore_mesh::{
        MeshError, MeshEvent, MeshHandle, MeshMessage, MeshRuntime, Neighbor,
        discovery::{DiscoveryAction, DiscoveryMessage},
        gossip::GossipMessage,
        resilience::{ResilienceAction, ResilienceState},
        topology::Path,
    };

    // Only types with GroundsTo impls (21 grounded, 5 config types excluded)
    let types_info: Vec<serde_json::Value> = vec![
        grounding_entry::<NodeState>("NodeState"),
        grounding_entry::<Path>("Path"),
        grounding_entry::<MeshError>("MeshError"),
        grounding_entry::<RouteQuality>("RouteQuality"),
        grounding_entry::<MeshMessage>("MeshMessage"),
        grounding_entry::<Neighbor>("Neighbor"),
        grounding_entry::<Route>("Route"),
        grounding_entry::<DiscoveryMessage>("DiscoveryMessage"),
        grounding_entry::<DiscoveryAction>("DiscoveryAction"),
        grounding_entry::<GossipMessage>("GossipMessage"),
        grounding_entry::<MeshEvent>("MeshEvent"),
        grounding_entry::<ResilienceAction>("ResilienceAction"),
        grounding_entry::<ResilienceState>("ResilienceState"),
        grounding_entry::<MeshHandle>("MeshHandle"),
        grounding_entry::<MeshRuntime>("MeshRuntime"),
    ];

    // Count tiers
    let mut t1 = 0usize;
    let mut t2p = 0usize;
    let mut t2c = 0usize;
    let mut t3 = 0usize;
    let mut all_primitives = std::collections::HashSet::new();

    for info in &types_info {
        let prim_count = info
            .get("primitive_count")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as usize;
        match prim_count {
            0..=1 => t1 += 1,
            2..=3 => t2p += 1,
            4..=5 => t2c += 1,
            _ => t3 += 1,
        }
        if let Some(prims) = info.get("primitives").and_then(|v| v.as_array()) {
            for p in prims {
                if let Some(s) = p.as_str() {
                    all_primitives.insert(s.to_string());
                }
            }
        }
    }

    let mut sorted_primitives: Vec<&String> = all_primitives.iter().collect();
    sorted_primitives.sort();

    let result = json!({
        "crate": "nexcore-mesh",
        "total_grounded_types": types_info.len(),
        "t1_coverage": format!("{}/16", all_primitives.len()),
        "tier_distribution": {
            "T1": t1,
            "T2-P": t2p,
            "T2-C": t2c,
            "T3": t3,
        },
        "primitives_used": {
            "count": all_primitives.len(),
            "symbols": sorted_primitives,
        },
        "types": types_info,
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{}".to_string()),
    )]))
}

/// Build a JSON entry for a GroundsTo type.
fn grounding_entry<T: nexcore_lex_primitiva::grounding::GroundsTo>(
    name: &str,
) -> serde_json::Value {
    let comp = T::primitive_composition();
    let primitives: Vec<String> = comp.primitives.iter().map(|p| format!("{:?}", p)).collect();
    let count = primitives.len();
    let tier = match count {
        0..=1 => "T1",
        2..=3 => "T2-P",
        4..=5 => "T2-C",
        _ => "T3",
    };
    let dominant = comp
        .dominant
        .map(|p| format!("{:?} ({:.0}%)", p, comp.confidence * 100.0))
        .unwrap_or_else(|| "none".to_string());

    json!({
        "name": name,
        "tier": tier,
        "dominant": dominant,
        "primitive_count": count,
        "primitives": primitives,
        "confidence": format!("{:.2}", comp.confidence),
    })
}

/// Get info about a specific mesh node address.
///
/// Returns address properties and topology characteristics.
/// Tier: T2-C (Cross-domain composite tool)
pub fn node_info(params: MeshNetworkNodeInfoParams) -> Result<CallToolResult, McpError> {
    let segments: Vec<String> = params.address.iter().map(|s| s.to_string()).collect();
    let addr = TopologicalAddress::new(segments, ".");

    let parent = if addr.segments.len() > 1 {
        let parent_segs = addr.segments[..addr.segments.len() - 1].to_vec();
        Some(TopologicalAddress::new(parent_segs, ".").render())
    } else {
        None
    };

    let result = json!({
        "address": addr.render(),
        "depth": addr.depth(),
        "segments": addr.segments,
        "is_root": addr.segments.len() <= 1,
        "parent": parent,
        "primitive_grounding": {
            "dominant": "λ Location",
            "composition": "λ(hierarchical address) + ∂(depth boundary) + σ(segment sequence)",
        }
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{}".to_string()),
    )]))
}
