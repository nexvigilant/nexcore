//! Topology tools: persistent homology, Betti numbers, Vietoris-Rips filtration.
//!
//! Wraps stem-topology for TDA (Topological Data Analysis) on crate DAGs,
//! PV signal networks, and dependency structures.

use crate::params::topology::{
    GraphCentralityParams, GraphComponentsParams, GraphShortestPathParams, TopoBettiParams,
    TopoPersistenceParams, TopoVietorisRipsParams,
};
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use serde_json::json;
use std::collections::{HashMap, HashSet, VecDeque};
use stem_topology::{
    BettiNumbers, DistanceMatrix, SimplicialComplex, betti_numbers, compute_persistence,
    vietoris_rips,
};

/// Build a Vietoris-Rips complex and return its structure
pub fn topo_vietoris_rips(params: TopoVietorisRipsParams) -> Result<CallToolResult, McpError> {
    let n = params.distances.len();
    if n == 0 {
        return Ok(CallToolResult::success(vec![Content::text(
            json!({"error": "Empty distance matrix"}).to_string(),
        )]));
    }

    let dm = DistanceMatrix::new(params.distances);
    let dm = if let Some(labels) = params.labels {
        dm.with_labels(labels)
    } else {
        dm
    };

    let max_dim = params.max_dim.unwrap_or(2);
    let max_filt = params.max_filtration.unwrap_or(f64::MAX);

    let complex = vietoris_rips(&dm, max_dim, max_filt);

    let dim_counts: Vec<(usize, usize)> = (0..=max_dim)
        .map(|d| (d, complex.simplices_of_dim(d).len()))
        .collect();

    let result = json!({
        "points": n,
        "total_simplices": complex.simplex_count(),
        "dimension": complex.dimension(),
        "simplex_counts_by_dim": dim_counts,
        "max_filtration_used": max_filt,
    });

    Ok(CallToolResult::success(vec![Content::text(
        result.to_string(),
    )]))
}

/// Compute persistent homology from a distance matrix
pub fn topo_persistence(params: TopoPersistenceParams) -> Result<CallToolResult, McpError> {
    let n = params.distances.len();
    if n == 0 {
        return Ok(CallToolResult::success(vec![Content::text(
            json!({"error": "Empty distance matrix"}).to_string(),
        )]));
    }

    // Auto-compute max filtration from distances if not provided
    let max_filt = params.max_filtration.unwrap_or_else(|| {
        params
            .distances
            .iter()
            .flat_map(|row| row.iter())
            .copied()
            .filter(|d| d.is_finite())
            .fold(0.0_f64, f64::max)
    });

    let dm = DistanceMatrix::new(params.distances);
    let dm = if let Some(labels) = params.labels {
        dm.with_labels(labels)
    } else {
        dm
    };

    let max_dim = params.max_dim.unwrap_or(2);
    let complex = vietoris_rips(&dm, max_dim, max_filt);
    let diagram = compute_persistence(&complex);

    let min_pers = params.min_persistence.unwrap_or(0.0);

    let points: Vec<serde_json::Value> = diagram
        .points
        .iter()
        .filter(|p| p.persistence() >= min_pers)
        .map(|p| {
            json!({
                "dimension": p.dimension,
                "birth": p.birth,
                "death": if p.is_infinite() { serde_json::Value::String("inf".into()) } else { serde_json::Value::from(p.death) },
                "persistence": if p.is_infinite() { serde_json::Value::String("inf".into()) } else { serde_json::Value::from(p.persistence()) },
                "is_essential": p.is_infinite(),
            })
        })
        .collect();

    // Summary by dimension
    let max_reported_dim = diagram
        .points
        .iter()
        .map(|p| p.dimension)
        .max()
        .unwrap_or(0);

    let summary: Vec<serde_json::Value> = (0..=max_reported_dim)
        .map(|dim| {
            let dim_points: Vec<_> = diagram
                .points
                .iter()
                .filter(|p| p.dimension == dim && p.persistence() >= min_pers)
                .collect();
            let essential = dim_points.iter().filter(|p| p.is_infinite()).count();
            let ephemeral = dim_points.len() - essential;
            json!({
                "dimension": dim,
                "total": dim_points.len(),
                "essential": essential,
                "ephemeral": ephemeral,
                "interpretation": match dim {
                    0 => "connected components (β₀)",
                    1 => "loops/cycles (β₁)",
                    2 => "voids/cavities (β₂)",
                    _ => "higher-dimensional holes",
                }
            })
        })
        .collect();

    let result = json!({
        "points": n,
        "features": points,
        "feature_count": points.len(),
        "summary_by_dimension": summary,
        "max_filtration": max_filt,
    });

    Ok(CallToolResult::success(vec![Content::text(
        result.to_string(),
    )]))
}

/// Compute Betti numbers at a specific filtration value
pub fn topo_betti(params: TopoBettiParams) -> Result<CallToolResult, McpError> {
    let n = params.distances.len();
    if n == 0 {
        return Ok(CallToolResult::success(vec![Content::text(
            json!({"error": "Empty distance matrix"}).to_string(),
        )]));
    }

    let max_filt = params
        .distances
        .iter()
        .flat_map(|row| row.iter())
        .copied()
        .filter(|d| d.is_finite())
        .fold(0.0_f64, f64::max);

    let dm = DistanceMatrix::new(params.distances);
    let max_dim = params.max_dim.unwrap_or(2);
    let complex = vietoris_rips(&dm, max_dim, max_filt);
    let diagram = compute_persistence(&complex);
    let betti = betti_numbers(&diagram, params.at_filtration);

    let numbers: Vec<serde_json::Value> = betti
        .numbers
        .iter()
        .map(|(dim, count)| {
            json!({
                "dimension": dim,
                "betti": count,
                "interpretation": match *dim {
                    0 => format!("{} connected components", count),
                    1 => format!("{} independent loops/cycles", count),
                    2 => format!("{} enclosed voids", count),
                    d => format!("{} {}-dimensional holes", count, d),
                }
            })
        })
        .collect();

    let result = json!({
        "at_filtration": params.at_filtration,
        "betti_numbers": numbers,
        "beta_0": betti.at_dim(0),
        "beta_1": betti.at_dim(1),
    });

    Ok(CallToolResult::success(vec![Content::text(
        result.to_string(),
    )]))
}

// ========================================================================
// Graph Analysis Tools (pure algorithms, no external deps)
// ========================================================================

/// Compute betweenness centrality for all nodes
pub fn graph_centrality(params: GraphCentralityParams) -> Result<CallToolResult, McpError> {
    let nodes = &params.nodes;
    let n = nodes.len();
    let node_idx: HashMap<&str, usize> = nodes
        .iter()
        .enumerate()
        .map(|(i, n)| (n.as_str(), i))
        .collect();

    // Adjacency list (directed)
    let mut adj: Vec<Vec<usize>> = vec![vec![]; n];
    for (from, to) in &params.edges {
        if let (Some(&fi), Some(&ti)) = (node_idx.get(from.as_str()), node_idx.get(to.as_str())) {
            adj[fi].push(ti);
        }
    }

    // In-degree and out-degree
    let mut in_degree = vec![0usize; n];
    let mut out_degree = vec![0usize; n];
    for (from, to) in &params.edges {
        if let Some(&fi) = node_idx.get(from.as_str()) {
            out_degree[fi] += 1;
        }
        if let Some(&ti) = node_idx.get(to.as_str()) {
            in_degree[ti] += 1;
        }
    }

    // BFS-based betweenness centrality (Brandes algorithm simplified)
    let mut centrality = vec![0.0_f64; n];

    for s in 0..n {
        let mut stack = Vec::new();
        let mut sigma = vec![0.0_f64; n];
        sigma[s] = 1.0;
        let mut dist: Vec<i64> = vec![-1; n];
        dist[s] = 0;
        let mut predecessors: Vec<Vec<usize>> = vec![vec![]; n];
        let mut queue = VecDeque::new();
        queue.push_back(s);

        while let Some(v) = queue.pop_front() {
            stack.push(v);
            for &w in &adj[v] {
                if dist[w] < 0 {
                    dist[w] = dist[v] + 1;
                    queue.push_back(w);
                }
                if dist[w] == dist[v] + 1 {
                    sigma[w] += sigma[v];
                    predecessors[w].push(v);
                }
            }
        }

        let mut delta = vec![0.0_f64; n];
        while let Some(w) = stack.pop() {
            for &v in &predecessors[w] {
                delta[v] += (sigma[v] / sigma[w]) * (1.0 + delta[w]);
            }
            if w != s {
                centrality[w] += delta[w];
            }
        }
    }

    // Normalize
    let norm = if n > 2 {
        ((n - 1) * (n - 2)) as f64
    } else {
        1.0
    };

    let mut ranked: Vec<serde_json::Value> = nodes
        .iter()
        .enumerate()
        .map(|(i, name)| {
            json!({
                "node": name,
                "centrality": centrality[i] / norm,
                "in_degree": in_degree[i],
                "out_degree": out_degree[i],
            })
        })
        .collect();

    ranked.sort_by(|a, b| {
        b["centrality"]
            .as_f64()
            .unwrap_or(0.0)
            .partial_cmp(&a["centrality"].as_f64().unwrap_or(0.0))
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let top_10: Vec<_> = ranked.iter().take(10).cloned().collect();

    let result = json!({
        "node_count": n,
        "edge_count": params.edges.len(),
        "top_central_nodes": top_10,
        "all_centrality": ranked,
    });

    Ok(CallToolResult::success(vec![Content::text(
        result.to_string(),
    )]))
}

/// Find connected components (treating edges as undirected)
pub fn graph_components(params: GraphComponentsParams) -> Result<CallToolResult, McpError> {
    let nodes = &params.nodes;
    let n = nodes.len();
    let node_idx: HashMap<&str, usize> = nodes
        .iter()
        .enumerate()
        .map(|(i, n)| (n.as_str(), i))
        .collect();

    // Undirected adjacency
    let mut adj: Vec<Vec<usize>> = vec![vec![]; n];
    for (from, to) in &params.edges {
        if let (Some(&fi), Some(&ti)) = (node_idx.get(from.as_str()), node_idx.get(to.as_str())) {
            adj[fi].push(ti);
            adj[ti].push(fi);
        }
    }

    let mut visited = vec![false; n];
    let mut components: Vec<Vec<String>> = Vec::new();

    for start in 0..n {
        if visited[start] {
            continue;
        }
        let mut component = Vec::new();
        let mut queue = VecDeque::new();
        queue.push_back(start);
        visited[start] = true;

        while let Some(v) = queue.pop_front() {
            component.push(nodes[v].clone());
            for &w in &adj[v] {
                if !visited[w] {
                    visited[w] = true;
                    queue.push_back(w);
                }
            }
        }
        component.sort();
        components.push(component);
    }

    components.sort_by(|a, b| b.len().cmp(&a.len()));

    let standalone: Vec<_> = components
        .iter()
        .filter(|c| c.len() == 1)
        .map(|c| c[0].clone())
        .collect();

    let result = json!({
        "component_count": components.len(),
        "largest_component_size": components.first().map(|c| c.len()).unwrap_or(0),
        "standalone_count": standalone.len(),
        "standalone_nodes": standalone,
        "components": components.iter().map(|c| json!({"size": c.len(), "nodes": c})).collect::<Vec<_>>(),
    });

    Ok(CallToolResult::success(vec![Content::text(
        result.to_string(),
    )]))
}

/// Find shortest path between two nodes (BFS, unweighted)
pub fn graph_shortest_path(params: GraphShortestPathParams) -> Result<CallToolResult, McpError> {
    let mut all_nodes: HashSet<String> = HashSet::new();
    for (from, to) in &params.edges {
        all_nodes.insert(from.clone());
        all_nodes.insert(to.clone());
    }
    all_nodes.insert(params.from.clone());
    all_nodes.insert(params.to.clone());

    let nodes: Vec<String> = all_nodes.into_iter().collect();
    let node_idx: HashMap<&str, usize> = nodes
        .iter()
        .enumerate()
        .map(|(i, n)| (n.as_str(), i))
        .collect();

    let n = nodes.len();
    let mut adj: Vec<Vec<usize>> = vec![vec![]; n];
    for (from, to) in &params.edges {
        if let (Some(&fi), Some(&ti)) = (node_idx.get(from.as_str()), node_idx.get(to.as_str())) {
            adj[fi].push(ti);
        }
    }

    let start = match node_idx.get(params.from.as_str()) {
        Some(&i) => i,
        None => {
            return Ok(CallToolResult::success(vec![Content::text(
                json!({"error": "Source node not found"}).to_string(),
            )]));
        }
    };
    let end = match node_idx.get(params.to.as_str()) {
        Some(&i) => i,
        None => {
            return Ok(CallToolResult::success(vec![Content::text(
                json!({"error": "Target node not found"}).to_string(),
            )]));
        }
    };

    // BFS
    let mut dist: Vec<i64> = vec![-1; n];
    let mut prev: Vec<Option<usize>> = vec![None; n];
    dist[start] = 0;
    let mut queue = VecDeque::new();
    queue.push_back(start);

    while let Some(v) = queue.pop_front() {
        if v == end {
            break;
        }
        for &w in &adj[v] {
            if dist[w] < 0 {
                dist[w] = dist[v] + 1;
                prev[w] = Some(v);
                queue.push_back(w);
            }
        }
    }

    if dist[end] < 0 {
        let result = json!({
            "reachable": false,
            "from": params.from,
            "to": params.to,
            "path": null,
            "distance": null,
        });
        return Ok(CallToolResult::success(vec![Content::text(
            result.to_string(),
        )]));
    }

    // Reconstruct path
    let mut path = Vec::new();
    let mut current = end;
    while current != start {
        path.push(nodes[current].clone());
        match prev[current] {
            Some(p) => current = p,
            None => break,
        }
    }
    path.push(nodes[start].clone());
    path.reverse();

    let result = json!({
        "reachable": true,
        "from": params.from,
        "to": params.to,
        "path": path,
        "distance": dist[end],
    });

    Ok(CallToolResult::success(vec![Content::text(
        result.to_string(),
    )]))
}
