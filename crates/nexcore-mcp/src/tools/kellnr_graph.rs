//! Kellnr Graph Theory computation tools (4).
//! Consolidated from kellnr-mcp/src/graph.rs.

use crate::params::kellnr::{KellnrGraphEdgesParams, KellnrGraphMutualInfoParams};
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use serde_json::json;
use std::collections::VecDeque;

fn json_result(value: serde_json::Value) -> CallToolResult {
    CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&value).unwrap_or_else(|_| "{}".into()),
    )])
}

fn edges_to_tuples(edges: &[Vec<usize>]) -> Vec<(usize, usize)> {
    edges
        .iter()
        .filter_map(|e| {
            if e.len() == 2 {
                Some((e[0], e[1]))
            } else {
                None
            }
        })
        .collect()
}

/// Betweenness centrality using Brandes algorithm.
pub fn compute_graph_betweenness(
    params: KellnrGraphEdgesParams,
) -> Result<CallToolResult, McpError> {
    let edge_tuples = edges_to_tuples(&params.edges);
    let n = params.node_count;
    let mut adj = vec![vec![]; n];
    for &(u, v) in &edge_tuples {
        if u < n && v < n {
            adj[u].push(v);
            adj[v].push(u);
        }
    }
    let mut cb = vec![0.0f64; n];
    for s in 0..n {
        let mut stack = Vec::new();
        let mut pred: Vec<Vec<usize>> = vec![vec![]; n];
        let mut sigma = vec![0.0f64; n];
        sigma[s] = 1.0;
        let mut dist = vec![-1i64; n];
        dist[s] = 0;
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
                    pred[w].push(v);
                }
            }
        }
        let mut delta = vec![0.0f64; n];
        while let Some(w) = stack.pop() {
            for &v in &pred[w] {
                delta[v] += (sigma[v] / sigma[w]) * (1.0 + delta[w]);
            }
            if w != s {
                cb[w] += delta[w];
            }
        }
    }
    let norm = if n > 2 {
        2.0 / ((n - 1) * (n - 2)) as f64
    } else {
        1.0
    };
    let normalized: Vec<f64> = cb.iter().map(|&c| c * norm).collect();
    let max_node = cb
        .iter()
        .enumerate()
        .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
        .map(|(i, _)| i)
        .unwrap_or(0);
    Ok(json_result(json!({
        "success": true,
        "betweenness": cb,
        "normalized": normalized,
        "most_central_node": max_node,
        "node_count": n,
        "edge_count": edge_tuples.len()
    })))
}

/// Mutual information between two discrete variables.
pub fn compute_graph_mutual_info(
    params: KellnrGraphMutualInfoParams,
) -> Result<CallToolResult, McpError> {
    let x = &params.x;
    let y = &params.y;
    if x.len() != y.len() || x.is_empty() {
        return Ok(json_result(
            json!({"success": false, "error": "x and y must have equal non-zero length"}),
        ));
    }
    let n = x.len() as f64;
    let x_max = *x.iter().max().unwrap_or(&0) as usize + 1;
    let y_max = *y.iter().max().unwrap_or(&0) as usize + 1;
    let mut joint = vec![vec![0u64; y_max]; x_max];
    let mut px = vec![0u64; x_max];
    let mut py = vec![0u64; y_max];
    for i in 0..x.len() {
        let xi = x[i] as usize;
        let yi = y[i] as usize;
        joint[xi][yi] += 1;
        px[xi] += 1;
        py[yi] += 1;
    }
    let mut mi = 0.0;
    for xi in 0..x_max {
        for yi in 0..y_max {
            if joint[xi][yi] > 0 {
                let pxy = joint[xi][yi] as f64 / n;
                let px_v = px[xi] as f64 / n;
                let py_v = py[yi] as f64 / n;
                mi += pxy * (pxy / (px_v * py_v)).ln();
            }
        }
    }
    let hx: f64 = px
        .iter()
        .filter(|&&c| c > 0)
        .map(|&c| {
            let p = c as f64 / n;
            -p * p.ln()
        })
        .sum();
    let hy: f64 = py
        .iter()
        .filter(|&&c| c > 0)
        .map(|&c| {
            let p = c as f64 / n;
            -p * p.ln()
        })
        .sum();
    let nmi = if hx.min(hy) > 0.0 {
        mi / hx.min(hy)
    } else {
        0.0
    };
    Ok(json_result(json!({
        "success": true,
        "mutual_information_nats": mi,
        "mutual_information_bits": mi / 2.0_f64.ln(),
        "normalized_mi": nmi,
        "entropy_x_nats": hx,
        "entropy_y_nats": hy,
        "n": x.len()
    })))
}

/// Tarjan's SCC algorithm.
pub fn compute_graph_tarjan_scc(
    params: KellnrGraphEdgesParams,
) -> Result<CallToolResult, McpError> {
    let edge_tuples = edges_to_tuples(&params.edges);
    let n = params.node_count;
    let mut adj = vec![vec![]; n];
    for &(u, v) in &edge_tuples {
        if u < n && v < n {
            adj[u].push(v);
        }
    }
    let mut index_counter = 0usize;
    let mut stack = Vec::new();
    let mut on_stack = vec![false; n];
    let mut indices = vec![usize::MAX; n];
    let mut lowlinks = vec![0usize; n];
    let mut sccs: Vec<Vec<usize>> = Vec::new();

    #[allow(clippy::too_many_arguments)]
    fn strongconnect(
        v: usize,
        adj: &[Vec<usize>],
        counter: &mut usize,
        stack: &mut Vec<usize>,
        on_stack: &mut [bool],
        indices: &mut [usize],
        lowlinks: &mut [usize],
        sccs: &mut Vec<Vec<usize>>,
    ) {
        indices[v] = *counter;
        lowlinks[v] = *counter;
        *counter += 1;
        stack.push(v);
        on_stack[v] = true;
        for &w in &adj[v] {
            if indices[w] == usize::MAX {
                strongconnect(w, adj, counter, stack, on_stack, indices, lowlinks, sccs);
                lowlinks[v] = lowlinks[v].min(lowlinks[w]);
            } else if on_stack[w] {
                lowlinks[v] = lowlinks[v].min(indices[w]);
            }
        }
        if lowlinks[v] == indices[v] {
            let mut scc = Vec::new();
            while let Some(w) = stack.pop() {
                on_stack[w] = false;
                scc.push(w);
                if w == v {
                    break;
                }
            }
            sccs.push(scc);
        }
    }

    for v in 0..n {
        if indices[v] == usize::MAX {
            strongconnect(
                v,
                &adj,
                &mut index_counter,
                &mut stack,
                &mut on_stack,
                &mut indices,
                &mut lowlinks,
                &mut sccs,
            );
        }
    }
    let has_cycle = sccs.iter().any(|scc| scc.len() > 1);
    Ok(json_result(json!({
        "success": true,
        "sccs": sccs,
        "scc_count": sccs.len(),
        "has_cycle": has_cycle,
        "largest_scc_size": sccs.iter().map(|s| s.len()).max().unwrap_or(0),
        "node_count": n
    })))
}

/// Topological sort (Kahn's algorithm). Returns error if cycle detected.
pub fn compute_graph_topsort(params: KellnrGraphEdgesParams) -> Result<CallToolResult, McpError> {
    let edge_tuples = edges_to_tuples(&params.edges);
    let n = params.node_count;
    let mut adj = vec![vec![]; n];
    let mut indegree = vec![0usize; n];
    for &(u, v) in &edge_tuples {
        if u < n && v < n {
            adj[u].push(v);
            indegree[v] += 1;
        }
    }
    let mut queue: VecDeque<usize> = indegree
        .iter()
        .enumerate()
        .filter(|&(_, &d)| d == 0)
        .map(|(i, _)| i)
        .collect();
    let mut order = Vec::with_capacity(n);
    while let Some(u) = queue.pop_front() {
        order.push(u);
        for &v in &adj[u] {
            indegree[v] -= 1;
            if indegree[v] == 0 {
                queue.push_back(v);
            }
        }
    }
    if order.len() == n {
        Ok(json_result(json!({
            "success": true,
            "order": order,
            "is_dag": true,
            "node_count": n
        })))
    } else {
        Ok(json_result(json!({
            "success": false,
            "error": "Graph contains a cycle",
            "is_dag": false,
            "sorted_before_cycle": order,
            "node_count": n
        })))
    }
}
