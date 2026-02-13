//! Graph Theory: dependency graph analysis, SCC detection, centrality.
//!
//! ## Primitive Foundation
//!
//! | Primitive | Manifestation |
//! |-----------|---------------|
//! | T1: Mapping (μ) | crate name → adjacency list |
//! | T1: Recursion (ρ) | Tarjan SCC, DFS for depth |
//! | T1: Sequence (σ) | Topological ordering |
//! | T1: State (ς) | Visited/on-stack state during DFS |

use crate::error::{MeasureError, MeasureResult};
use crate::types::{Centrality, CouplingRatio, CrateId, Density, GraphAnalysis, GraphNode};
use std::collections::HashMap;

/// Dependency graph as adjacency list.
///
/// Tier: T2-C (composed graph structure).
#[derive(Debug, Clone)]
pub struct DepGraph {
    /// Node index → crate name.
    pub names: Vec<String>,
    /// name → index lookup.
    pub index: HashMap<String, usize>,
    /// Adjacency list: edges[i] = nodes that i depends on (fan-out).
    pub edges: Vec<Vec<usize>>,
    /// Reverse adjacency: rev_edges[i] = nodes depending on i (fan-in).
    pub rev_edges: Vec<Vec<usize>>,
}

impl DepGraph {
    /// Create empty graph.
    #[must_use]
    pub fn new() -> Self {
        Self {
            names: Vec::new(),
            index: HashMap::new(),
            edges: Vec::new(),
            rev_edges: Vec::new(),
        }
    }

    /// Add a node, returning its index. Idempotent.
    pub fn add_node(&mut self, name: &str) -> usize {
        if let Some(&idx) = self.index.get(name) {
            return idx;
        }
        let idx = self.names.len();
        self.names.push(name.to_string());
        self.index.insert(name.to_string(), idx);
        self.edges.push(Vec::new());
        self.rev_edges.push(Vec::new());
        idx
    }

    /// Add a directed edge: `from` depends on `to`. Deduplicates.
    pub fn add_edge(&mut self, from: usize, to: usize) {
        if !self.edges[from].contains(&to) {
            self.edges[from].push(to);
            self.rev_edges[to].push(from);
        }
    }

    /// Number of nodes.
    #[must_use]
    pub fn node_count(&self) -> usize {
        self.names.len()
    }

    /// Number of edges.
    #[must_use]
    pub fn edge_count(&self) -> usize {
        self.edges.iter().map(|e| e.len()).sum()
    }

    /// Fan-out for a node (dependencies).
    #[must_use]
    pub fn fan_out(&self, node: usize) -> usize {
        self.edges.get(node).map_or(0, Vec::len)
    }

    /// Fan-in for a node (reverse dependencies).
    #[must_use]
    pub fn fan_in(&self, node: usize) -> usize {
        self.rev_edges.get(node).map_or(0, Vec::len)
    }

    /// Coupling ratio: fan_out / (fan_in + fan_out).
    #[must_use]
    pub fn coupling_ratio(&self, node: usize) -> CouplingRatio {
        let fi = self.fan_in(node) as f64;
        let fo = self.fan_out(node) as f64;
        let total = fi + fo;
        if total < f64::EPSILON {
            return CouplingRatio::new(0.0);
        }
        CouplingRatio::new(fo / total)
    }

    /// Graph density: |E| / (|V| * (|V|-1)) for directed graphs.
    #[must_use]
    pub fn density(&self) -> Density {
        let v = self.node_count();
        if v <= 1 {
            return Density::new(0.0);
        }
        let max_edges = v * (v - 1);
        Density::new(self.edge_count() as f64 / max_edges as f64)
    }

    /// Degree centrality: (in + out) / (2 * (V - 1)).
    #[must_use]
    pub fn degree_centrality(&self, node: usize) -> Centrality {
        let v = self.node_count();
        if v <= 1 {
            return Centrality::new(0.0);
        }
        let total = self.fan_in(node) + self.fan_out(node);
        Centrality::new(total as f64 / (2.0 * (v - 1) as f64))
    }

    /// Topological depth: longest path in the DAG.
    pub fn topological_depth(&self) -> usize {
        let n = self.node_count();
        if n == 0 {
            return 0;
        }
        let mut memo: Vec<Option<usize>> = vec![None; n];
        let mut on_stack = vec![false; n];
        (0..n)
            .map(|s| dfs_depth(self, s, &mut memo, &mut on_stack))
            .max()
            .unwrap_or(0)
    }

    /// Get all neighbors (outgoing + incoming, deduplicated).
    fn all_neighbors(&self, node: usize) -> Vec<usize> {
        let mut nb: Vec<usize> = self.edges[node].clone();
        for &n in &self.rev_edges[node] {
            if !nb.contains(&n) {
                nb.push(n);
            }
        }
        nb
    }

    /// Build full graph analysis.
    pub fn analyze(&self) -> GraphAnalysis {
        let sccs = tarjan_scc(self);
        let bc = brandes_betweenness(self);
        build_analysis(self, &sccs, &bc)
    }
}

impl Default for DepGraph {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// DFS depth (σ: Recursion primitive)
// ---------------------------------------------------------------------------

fn dfs_depth(
    g: &DepGraph,
    node: usize,
    memo: &mut [Option<usize>],
    on_stack: &mut [bool],
) -> usize {
    if let Some(d) = memo[node] {
        return d;
    }
    if on_stack[node] {
        return 0;
    }
    on_stack[node] = true;
    let max_child = g.edges[node]
        .iter()
        .map(|&nb| dfs_depth(g, nb, memo, on_stack))
        .max()
        .unwrap_or(0);
    on_stack[node] = false;
    let depth = 1 + max_child;
    memo[node] = Some(depth);
    depth
}

// ---------------------------------------------------------------------------
// Tarjan SCC (ρ: Recursion primitive)
// ---------------------------------------------------------------------------

/// Context for Tarjan's algorithm.
struct TarjanCtx {
    counter: usize,
    stack: Vec<usize>,
    on_stack: Vec<bool>,
    indices: Vec<Option<usize>>,
    lowlinks: Vec<usize>,
    result: Vec<Vec<usize>>,
}

impl TarjanCtx {
    fn new(n: usize) -> Self {
        Self {
            counter: 0,
            stack: Vec::new(),
            on_stack: vec![false; n],
            indices: vec![None; n],
            lowlinks: vec![0; n],
            result: Vec::new(),
        }
    }

    fn enter_node(&mut self, v: usize) {
        self.indices[v] = Some(self.counter);
        self.lowlinks[v] = self.counter;
        self.counter += 1;
        self.stack.push(v);
        self.on_stack[v] = true;
    }

    fn update_lowlink(&mut self, v: usize, w: usize) {
        self.lowlinks[v] = self.lowlinks[v].min(self.lowlinks[w]);
    }

    fn update_from_backedge(&mut self, v: usize, w: usize) {
        let w_idx = self.indices[w].unwrap_or(0);
        self.lowlinks[v] = self.lowlinks[v].min(w_idx);
    }

    fn collect_scc(&mut self, v: usize) {
        let v_idx = self.indices[v].unwrap_or(0);
        if self.lowlinks[v] != v_idx {
            return;
        }
        let mut scc = Vec::new();
        loop {
            let w = match self.stack.pop() {
                Some(w) => w,
                None => break,
            };
            self.on_stack[w] = false;
            scc.push(w);
            if w == v {
                break;
            }
        }
        self.result.push(scc);
    }
}

/// Tarjan's SCC. Returns groups of >1 node (cycles only).
pub fn tarjan_scc(graph: &DepGraph) -> Vec<Vec<usize>> {
    let n = graph.node_count();
    let mut ctx = TarjanCtx::new(n);
    for v in 0..n {
        if ctx.indices[v].is_none() {
            tarjan_visit(graph, v, &mut ctx);
        }
    }
    ctx.result.into_iter().filter(|scc| scc.len() > 1).collect()
}

fn tarjan_visit(graph: &DepGraph, v: usize, ctx: &mut TarjanCtx) {
    ctx.enter_node(v);
    for &w in &graph.edges[v] {
        if ctx.indices[w].is_none() {
            tarjan_visit(graph, w, ctx);
            ctx.update_lowlink(v, w);
        } else if ctx.on_stack[w] {
            ctx.update_from_backedge(v, w);
        }
    }
    ctx.collect_scc(v);
}

// ---------------------------------------------------------------------------
// Brandes betweenness centrality (σ: Sequence primitive)
// ---------------------------------------------------------------------------

/// BFS state for one source in Brandes' algorithm.
struct BrandesBfs {
    stack: Vec<usize>,
    pred: Vec<Vec<usize>>,
    sigma: Vec<f64>,
    dist: Vec<i64>,
}

impl BrandesBfs {
    fn new(n: usize, source: usize) -> Self {
        let mut sigma = vec![0.0_f64; n];
        sigma[source] = 1.0;
        let mut dist = vec![-1_i64; n];
        dist[source] = 0;
        Self {
            stack: Vec::new(),
            pred: vec![Vec::new(); n],
            sigma,
            dist,
        }
    }

    fn process_neighbor(
        &mut self,
        v: usize,
        w: usize,
        queue: &mut std::collections::VecDeque<usize>,
    ) {
        if self.dist[w] < 0 {
            self.dist[w] = self.dist[v] + 1;
            queue.push_back(w);
        }
        if self.dist[w] == self.dist[v] + 1 {
            self.sigma[w] += self.sigma[v];
            self.pred[w].push(v);
        }
    }

    fn run_bfs(&mut self, graph: &DepGraph, source: usize) {
        let mut queue = std::collections::VecDeque::new();
        queue.push_back(source);
        while let Some(v) = queue.pop_front() {
            self.stack.push(v);
            let neighbors = graph.all_neighbors(v);
            for w in neighbors {
                self.process_neighbor(v, w, &mut queue);
            }
        }
    }

    fn back_propagate(&self, source: usize, cb: &mut [f64]) {
        let n = self.sigma.len();
        let mut delta = vec![0.0_f64; n];
        for &w in self.stack.iter().rev() {
            self.accumulate_delta(w, &mut delta);
            if w != source {
                cb[w] += delta[w];
            }
        }
    }

    fn accumulate_delta(&self, w: usize, delta: &mut [f64]) {
        for &v in &self.pred[w] {
            if self.sigma[w] > 0.0 {
                delta[v] += (self.sigma[v] / self.sigma[w]) * (1.0 + delta[w]);
            }
        }
    }
}

/// Brandes' algorithm for betweenness centrality. O(V*E).
pub fn brandes_betweenness(graph: &DepGraph) -> Vec<Centrality> {
    let n = graph.node_count();
    let mut cb = vec![0.0_f64; n];
    for s in 0..n {
        let mut bfs = BrandesBfs::new(n, s);
        bfs.run_bfs(graph, s);
        bfs.back_propagate(s, &mut cb);
    }
    let norm = if n > 2 {
        ((n - 1) * (n - 2)) as f64
    } else {
        1.0
    };
    cb.iter().map(|&v| Centrality::new(v / norm)).collect()
}

// ---------------------------------------------------------------------------
// Analysis builder
// ---------------------------------------------------------------------------

fn build_analysis(graph: &DepGraph, sccs: &[Vec<usize>], bc: &[Centrality]) -> GraphAnalysis {
    let n = graph.node_count();
    let nodes: Vec<GraphNode> = (0..n).map(|i| build_graph_node(graph, i, bc)).collect();
    let cycles: Vec<Vec<CrateId>> = sccs
        .iter()
        .map(|scc| scc.iter().map(|&i| CrateId::new(&graph.names[i])).collect())
        .collect();
    GraphAnalysis {
        node_count: n,
        edge_count: graph.edge_count(),
        density: graph.density(),
        max_depth: graph.topological_depth(),
        cycle_count: cycles.len(),
        cycles,
        nodes,
    }
}

fn build_graph_node(graph: &DepGraph, i: usize, bc: &[Centrality]) -> GraphNode {
    GraphNode {
        crate_id: CrateId::new(&graph.names[i]),
        fan_in: graph.fan_in(i),
        fan_out: graph.fan_out(i),
        coupling_ratio: graph.coupling_ratio(i),
        degree_centrality: graph.degree_centrality(i),
        betweenness_centrality: bc.get(i).copied().unwrap_or_else(|| Centrality::new(0.0)),
    }
}

// ---------------------------------------------------------------------------
// Workspace Cargo.toml parser
// ---------------------------------------------------------------------------

/// Parse workspace to build a DepGraph.
pub fn build_dep_graph(workspace_root: &std::path::Path) -> MeasureResult<DepGraph> {
    let mut graph = DepGraph::new();
    let crates_dir = workspace_root.join("crates");
    if !crates_dir.exists() {
        return Err(MeasureError::CargoParse {
            path: crates_dir.display().to_string(),
            reason: "crates/ directory not found".into(),
        });
    }
    let crate_dirs = discover_crates(&crates_dir, &mut graph)?;
    parse_crate_deps(&crate_dirs, &mut graph)?;
    Ok(graph)
}

fn discover_crates(
    crates_dir: &std::path::Path,
    graph: &mut DepGraph,
) -> MeasureResult<Vec<(String, std::path::PathBuf)>> {
    let mut result = Vec::new();
    let entries = std::fs::read_dir(crates_dir).map_err(MeasureError::Io)?;
    for entry in entries {
        let entry = entry.map_err(MeasureError::Io)?;
        let path = entry.path();
        register_crate_if_valid(&path, graph, &mut result);
    }
    Ok(result)
}

fn register_crate_if_valid(
    path: &std::path::Path,
    graph: &mut DepGraph,
    result: &mut Vec<(String, std::path::PathBuf)>,
) {
    if !path.is_dir() {
        return;
    }
    let cargo_path = path.join("Cargo.toml");
    if !cargo_path.exists() {
        return;
    }
    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();
    graph.add_node(&name);
    result.push((name, cargo_path));
}

fn parse_crate_deps(
    crate_dirs: &[(String, std::path::PathBuf)],
    graph: &mut DepGraph,
) -> MeasureResult<()> {
    for (crate_name, cargo_path) in crate_dirs {
        let content = std::fs::read_to_string(cargo_path).map_err(MeasureError::Io)?;
        let parsed: toml::Value = content.parse().map_err(MeasureError::Toml)?;
        let from_idx = graph.index.get(crate_name.as_str()).copied().unwrap_or(0);
        wire_deps_from_toml(&parsed, from_idx, graph);
    }
    Ok(())
}

fn wire_deps_from_toml(parsed: &toml::Value, from_idx: usize, graph: &mut DepGraph) {
    let dep_keys = extract_dep_names(parsed, "dependencies");
    for dep_name in dep_keys {
        if let Some(&to_idx) = graph.index.get(dep_name.as_str()) {
            graph.add_edge(from_idx, to_idx);
        }
    }
}

fn extract_dep_names(parsed: &toml::Value, section: &str) -> Vec<String> {
    parsed
        .get(section)
        .and_then(|v| v.as_table())
        .map(|deps| deps.keys().cloned().collect())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_linear() -> DepGraph {
        let mut g = DepGraph::new();
        let a = g.add_node("a");
        let b = g.add_node("b");
        let c = g.add_node("c");
        g.add_edge(a, b);
        g.add_edge(b, c);
        g
    }

    fn make_cycle() -> DepGraph {
        let mut g = DepGraph::new();
        let a = g.add_node("a");
        let b = g.add_node("b");
        let c = g.add_node("c");
        g.add_edge(a, b);
        g.add_edge(b, c);
        g.add_edge(c, a);
        g
    }

    fn make_star() -> DepGraph {
        let mut g = DepGraph::new();
        let hub = g.add_node("hub");
        let a = g.add_node("a");
        let b = g.add_node("b");
        let c = g.add_node("c");
        let d = g.add_node("d");
        g.add_edge(hub, a);
        g.add_edge(hub, b);
        g.add_edge(hub, c);
        g.add_edge(hub, d);
        g
    }

    #[test]
    fn linear_chain_depth() {
        assert_eq!(make_linear().topological_depth(), 3);
    }

    #[test]
    fn linear_no_cycles() {
        assert!(tarjan_scc(&make_linear()).is_empty());
    }

    #[test]
    fn cycle_detected() {
        let sccs = tarjan_scc(&make_cycle());
        assert_eq!(sccs.len(), 1);
        assert_eq!(sccs[0].len(), 3);
    }

    #[test]
    fn star_fan_out() {
        let g = make_star();
        assert_eq!(g.fan_out(g.index["hub"]), 4);
        assert_eq!(g.fan_in(g.index["hub"]), 0);
    }

    #[test]
    fn star_fan_in() {
        let g = make_star();
        assert_eq!(g.fan_in(g.index["a"]), 1);
        assert_eq!(g.fan_out(g.index["a"]), 0);
    }

    #[test]
    fn density_complete() {
        let mut g = DepGraph::new();
        let a = g.add_node("a");
        let b = g.add_node("b");
        let c = g.add_node("c");
        g.add_edge(a, b);
        g.add_edge(a, c);
        g.add_edge(b, a);
        g.add_edge(b, c);
        g.add_edge(c, a);
        g.add_edge(c, b);
        assert!((g.density().value() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn density_empty() {
        assert!((DepGraph::new().density().value()).abs() < 1e-10);
    }

    #[test]
    fn density_single() {
        let mut g = DepGraph::new();
        g.add_node("solo");
        assert!((g.density().value()).abs() < 1e-10);
    }

    #[test]
    fn coupling_producer() {
        let g = make_star();
        assert!((g.coupling_ratio(g.index["a"]).value()).abs() < 1e-10);
    }

    #[test]
    fn coupling_consumer() {
        let g = make_star();
        assert!((g.coupling_ratio(g.index["hub"]).value() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn degree_centrality_hub() {
        let g = make_star();
        let dc = g.degree_centrality(g.index["hub"]);
        assert!((dc.value() - 0.5).abs() < 1e-10);
    }

    #[test]
    fn betweenness_linear() {
        let g = make_linear();
        let bc = brandes_betweenness(&g);
        assert!(bc[g.index["b"]].value() >= bc[g.index["a"]].value());
    }

    #[test]
    fn analyze_valid() {
        let a = make_linear().analyze();
        assert_eq!(a.node_count, 3);
        assert_eq!(a.edge_count, 2);
        assert_eq!(a.cycle_count, 0);
    }

    #[test]
    fn duplicate_edge_ignored() {
        let mut g = DepGraph::new();
        let a = g.add_node("a");
        let b = g.add_node("b");
        g.add_edge(a, b);
        g.add_edge(a, b);
        assert_eq!(g.edge_count(), 1);
    }

    #[test]
    fn add_node_idempotent() {
        let mut g = DepGraph::new();
        let i1 = g.add_node("x");
        let i2 = g.add_node("x");
        assert_eq!(i1, i2);
        assert_eq!(g.node_count(), 1);
    }
}
