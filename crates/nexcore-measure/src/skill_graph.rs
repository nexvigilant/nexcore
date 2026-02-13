//! Skill ecosystem dependency graph analysis.
//!
//! Builds directed graph from inter-skill references in SKILL.md files,
//! then computes betweenness centrality, cycle detection, and hub classification.
//!
//! ## Primitive Foundation
//!
//! | Primitive | Manifestation |
//! |-----------|---------------|
//! | T1: Mapping (μ) | skill_name → adjacency list |
//! | T1: Sequence (σ) | topological ordering |
//! | T1: Comparison (κ) | centrality ranking |
//! | T1: Recursion (ρ) | Tarjan DFS for SCCs |

use crate::error::MeasureResult;
use crate::types::*;
use std::collections::{HashMap, HashSet, VecDeque};
use std::path::Path;

// ---------------------------------------------------------------------------
// T2-C/T3: Graph types
// ---------------------------------------------------------------------------

/// Tier: T3 — A node in the skill dependency graph.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SkillNode {
    pub name: String,
    pub fan_in: usize,
    pub fan_out: usize,
    pub coupling_ratio: CouplingRatio,
    pub betweenness: Centrality,
    pub role: SkillRole,
}

/// Tier: T2-C — Skill role in the ecosystem graph.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum SkillRole {
    Foundation,
    Orchestrator,
    Hub,
    Leaf,
}

impl std::fmt::Display for SkillRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Foundation => write!(f, "Foundation"),
            Self::Orchestrator => write!(f, "Orchestrator"),
            Self::Hub => write!(f, "Hub"),
            Self::Leaf => write!(f, "Leaf"),
        }
    }
}

/// Tier: T3 — Full skill graph analysis result.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SkillGraphAnalysis {
    pub node_count: usize,
    pub edge_count: usize,
    pub density: Density,
    pub cycle_count: usize,
    pub cycles: Vec<Vec<String>>,
    pub max_depth: usize,
    pub nodes: Vec<SkillNode>,
}

// ---------------------------------------------------------------------------
// Adjacency list (T2-C)
// ---------------------------------------------------------------------------

/// Tier: T2-C — Directed graph via adjacency list.
#[derive(Debug, Clone)]
pub struct AdjacencyList {
    pub edges: HashMap<String, Vec<String>>,
    pub nodes: HashSet<String>,
}

impl AdjacencyList {
    #[must_use]
    pub fn new() -> Self {
        Self {
            edges: HashMap::new(),
            nodes: HashSet::new(),
        }
    }

    pub fn add_node(&mut self, name: String) {
        self.nodes.insert(name);
    }

    pub fn add_edge(&mut self, from: String, to: String) {
        self.nodes.insert(from.clone());
        self.nodes.insert(to.clone());
        self.edges.entry(from).or_default().push(to);
    }

    fn fan_out(&self, node: &str) -> usize {
        self.edges.get(node).map_or(0, |e| e.len())
    }

    fn fan_in(&self, node: &str) -> usize {
        self.edges
            .values()
            .filter(|t| t.contains(&node.to_string()))
            .count()
    }

    /// Run full analysis.
    pub fn analyze(&self) -> SkillGraphAnalysis {
        let stats = self.compute_stats();
        let betweenness = self.brandes_betweenness();
        let cycles = filter_real_cycles(self.tarjan_scc());
        let nodes = self.build_nodes(&betweenness);

        SkillGraphAnalysis {
            node_count: stats.0,
            edge_count: stats.1,
            density: Density::new(stats.2),
            cycle_count: cycles.len(),
            cycles,
            max_depth: self.compute_max_depth(),
            nodes,
        }
    }

    /// Compute basic graph statistics.
    fn compute_stats(&self) -> (usize, usize, f64) {
        let n = self.nodes.len();
        let edges: usize = self.edges.values().map(|v| v.len()).sum();
        let density = if n > 1 {
            edges as f64 / (n * (n - 1)) as f64
        } else {
            0.0
        };
        (n, edges, density)
    }

    /// Build SkillNode vec from betweenness map.
    fn build_nodes(&self, betweenness: &HashMap<&str, f64>) -> Vec<SkillNode> {
        let n = self.nodes.len();
        let mut nodes: Vec<SkillNode> = self
            .nodes
            .iter()
            .map(|name| {
                let fi = self.fan_in(name);
                let fo = self.fan_out(name);
                let cr = if fi + fo > 0 {
                    fo as f64 / (fi + fo) as f64
                } else {
                    0.5
                };
                let bc = betweenness.get(name.as_str()).copied().unwrap_or(0.0);
                SkillNode {
                    name: name.clone(),
                    fan_in: fi,
                    fan_out: fo,
                    coupling_ratio: CouplingRatio::new(cr),
                    betweenness: Centrality::new(bc),
                    role: classify_role(fi, fo, n),
                }
            })
            .collect();
        nodes.sort_by(|a, b| b.betweenness.value().total_cmp(&a.betweenness.value()));
        nodes
    }
}

// ---------------------------------------------------------------------------
// Graph building from filesystem
// ---------------------------------------------------------------------------

/// Build adjacency list from skill directory.
pub fn build_skill_graph(skills_dir: &Path) -> MeasureResult<AdjacencyList> {
    let contents = crate::skill::scan_skill_directory(skills_dir)?;
    let names: HashSet<String> = contents.iter().map(|(n, _)| n.clone()).collect();
    let mut adj = AdjacencyList::new();
    for (name, content) in &contents {
        adj.add_node(name.clone());
        let lower = content.to_lowercase();
        for target in find_refs(&lower, name, &names) {
            adj.add_edge(name.clone(), target);
        }
    }
    Ok(adj)
}

/// Find references to other skills in content.
fn find_refs(content: &str, self_name: &str, all: &HashSet<String>) -> Vec<String> {
    all.iter()
        .filter(|n| n.as_str() != self_name && content.contains(n.as_str()))
        .cloned()
        .collect()
}

// ---------------------------------------------------------------------------
// Brandes betweenness (simplified BFS-based)
// ---------------------------------------------------------------------------

impl AdjacencyList {
    fn brandes_betweenness(&self) -> HashMap<&str, f64> {
        let names: Vec<&str> = self.nodes.iter().map(|s| s.as_str()).collect();
        let n = names.len();
        let mut cb: HashMap<&str, f64> = names.iter().map(|&n| (n, 0.0)).collect();

        for &s in &names {
            let (sigma, pred, stack) = self.bfs_shortest_paths(s);
            self.accumulate_bc(&mut cb, s, &sigma, &pred, &stack);
        }

        normalize_betweenness(&mut cb, n);
        cb
    }

    /// BFS shortest paths from source.
    fn bfs_shortest_paths<'a>(
        &'a self,
        source: &'a str,
    ) -> (
        HashMap<&'a str, f64>,
        HashMap<&'a str, Vec<&'a str>>,
        Vec<&'a str>,
    ) {
        let mut sigma: HashMap<&str, f64> = HashMap::new();
        let mut dist: HashMap<&str, i64> = HashMap::new();
        let mut pred: HashMap<&str, Vec<&str>> = HashMap::new();
        let mut stack = Vec::new();
        let mut queue = VecDeque::new();

        for n in &self.nodes {
            sigma.insert(n.as_str(), 0.0);
            dist.insert(n.as_str(), -1);
            pred.insert(n.as_str(), vec![]);
        }
        sigma.insert(source, 1.0);
        dist.insert(source, 0);
        queue.push_back(source);

        while let Some(v) = queue.pop_front() {
            stack.push(v);
            self.relax_neighbors(v, &mut sigma, &mut dist, &mut pred, &mut queue);
        }
        (sigma, pred, stack)
    }

    /// Relax edges from v during BFS.
    fn relax_neighbors<'a>(
        &'a self,
        v: &'a str,
        sigma: &mut HashMap<&'a str, f64>,
        dist: &mut HashMap<&'a str, i64>,
        pred: &mut HashMap<&'a str, Vec<&'a str>>,
        queue: &mut VecDeque<&'a str>,
    ) {
        let v_dist = dist.get(v).copied().unwrap_or(-1);
        let Some(neighbors) = self.edges.get(v) else {
            return;
        };
        for w_str in neighbors {
            let w: &str = w_str.as_str();
            if dist.get(w).copied().unwrap_or(-1) < 0 {
                if let Some(d) = dist.get_mut(w) {
                    *d = v_dist + 1;
                }
                queue.push_back(w);
            }
            if dist.get(w).copied().unwrap_or(-1) == v_dist + 1 {
                let vs = sigma.get(v).copied().unwrap_or(0.0);
                if let Some(s) = sigma.get_mut(w) {
                    *s += vs;
                }
                if let Some(p) = pred.get_mut(w) {
                    p.push(v);
                }
            }
        }
    }

    /// Accumulate betweenness from BFS result.
    fn accumulate_bc<'a>(
        &self,
        cb: &mut HashMap<&'a str, f64>,
        source: &'a str,
        sigma: &HashMap<&'a str, f64>,
        pred: &HashMap<&'a str, Vec<&'a str>>,
        stack: &[&'a str],
    ) {
        let mut delta: HashMap<&str, f64> = stack.iter().map(|&n| (n, 0.0)).collect();
        for &w in stack.iter().rev() {
            let w_sigma = sigma.get(w).copied().unwrap_or(1.0);
            if let Some(preds) = pred.get(w) {
                for &v in preds {
                    let v_sigma = sigma.get(v).copied().unwrap_or(1.0);
                    let coeff = (v_sigma / w_sigma) * (1.0 + delta.get(w).copied().unwrap_or(0.0));
                    if let Some(d) = delta.get_mut(v) {
                        *d += coeff;
                    }
                }
            }
            if w != source {
                if let Some(val) = cb.get_mut(w) {
                    *val += delta.get(w).copied().unwrap_or(0.0);
                }
            }
        }
    }
}

/// Normalize betweenness values to [0,1].
fn normalize_betweenness(cb: &mut HashMap<&str, f64>, n: usize) {
    let norm = if n > 2 {
        ((n - 1) * (n - 2)) as f64
    } else {
        1.0
    };
    for val in cb.values_mut() {
        *val /= norm;
        *val = val.clamp(0.0, 1.0);
    }
}

// ---------------------------------------------------------------------------
// Tarjan SCC (ρ: recursion)
// ---------------------------------------------------------------------------

struct TarjanState<'a> {
    index: usize,
    indices: HashMap<&'a str, usize>,
    lowlinks: HashMap<&'a str, usize>,
    on_stack: HashSet<&'a str>,
    stack: Vec<&'a str>,
    visited: HashSet<&'a str>,
    sccs: Vec<Vec<String>>,
}

impl<'a> TarjanState<'a> {
    fn new() -> Self {
        Self {
            index: 0,
            indices: HashMap::new(),
            lowlinks: HashMap::new(),
            on_stack: HashSet::new(),
            stack: Vec::new(),
            visited: HashSet::new(),
            sccs: Vec::new(),
        }
    }
}

impl AdjacencyList {
    fn tarjan_scc(&self) -> Vec<Vec<String>> {
        let mut state = TarjanState::new();
        for name in &self.nodes {
            if !state.visited.contains(name.as_str()) {
                self.tarjan_visit(name, &mut state);
            }
        }
        state.sccs
    }

    fn tarjan_visit<'a>(&'a self, node: &'a str, s: &mut TarjanState<'a>) {
        let idx = s.index;
        s.index += 1;
        s.indices.insert(node, idx);
        s.lowlinks.insert(node, idx);
        s.on_stack.insert(node);
        s.stack.push(node);
        s.visited.insert(node);

        self.tarjan_process_neighbors(node, s);
        self.tarjan_check_root(node, s);
    }

    fn tarjan_process_neighbors<'a>(&'a self, node: &'a str, s: &mut TarjanState<'a>) {
        let Some(neighbors) = self.edges.get(node) else {
            return;
        };
        for neighbor in neighbors {
            let n: &str = neighbor.as_str();
            if !s.visited.contains(n) {
                self.tarjan_visit(n, s);
                let n_low = s.lowlinks.get(n).copied().unwrap_or(usize::MAX);
                update_lowlink(s, node, n_low);
            } else if s.on_stack.contains(n) {
                let n_idx = s.indices.get(n).copied().unwrap_or(usize::MAX);
                update_lowlink(s, node, n_idx);
            }
        }
    }

    fn tarjan_check_root<'a>(&'a self, node: &'a str, s: &mut TarjanState<'a>) {
        if s.indices.get(node) != s.lowlinks.get(node) {
            return;
        }
        let mut scc = Vec::new();
        while let Some(w) = s.stack.pop() {
            s.on_stack.remove(w);
            scc.push(w.to_string());
            if w == node {
                break;
            }
        }
        s.sccs.push(scc);
    }

    fn compute_max_depth(&self) -> usize {
        let incoming: HashSet<&str> = self
            .edges
            .values()
            .flat_map(|t| t.iter().map(|s| s.as_str()))
            .collect();
        self.nodes
            .iter()
            .map(|n| n.as_str())
            .filter(|n| !incoming.contains(n))
            .map(|root| bfs_depth(root, &self.edges))
            .max()
            .unwrap_or(0)
    }
}

fn update_lowlink(s: &mut TarjanState<'_>, node: &str, candidate: usize) {
    let my = s.lowlinks.get(node).copied().unwrap_or(usize::MAX);
    if let Some(l) = s.lowlinks.get_mut(node) {
        *l = my.min(candidate);
    }
}

fn filter_real_cycles(sccs: Vec<Vec<String>>) -> Vec<Vec<String>> {
    sccs.into_iter().filter(|c| c.len() > 1).collect()
}

fn bfs_depth(root: &str, edges: &HashMap<String, Vec<String>>) -> usize {
    let mut visited = HashSet::new();
    let mut queue = VecDeque::new();
    queue.push_back((root, 0_usize));
    visited.insert(root.to_string());
    let mut max_d = 0;
    while let Some((node, depth)) = queue.pop_front() {
        if depth > max_d {
            max_d = depth;
        }
        if let Some(neighbors) = edges.get(node) {
            for n in neighbors {
                if !visited.contains(n.as_str()) {
                    visited.insert(n.clone());
                    queue.push_back((n.as_str(), depth + 1));
                }
            }
        }
    }
    max_d
}

fn classify_role(fan_in: usize, fan_out: usize, total: usize) -> SkillRole {
    let threshold = (total as f64 * 0.1).max(2.0) as usize;
    match (fan_in >= threshold, fan_out >= threshold) {
        (true, true) => SkillRole::Hub,
        (true, false) => SkillRole::Foundation,
        (false, true) => SkillRole::Orchestrator,
        (false, false) => SkillRole::Leaf,
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_graph() -> AdjacencyList {
        let mut g = AdjacencyList::new();
        g.add_edge("forge".into(), "primitive-extractor".into());
        g.add_edge("forge".into(), "rust-anatomy-expert".into());
        g.add_edge("forge".into(), "strat-dev".into());
        g.add_edge("skill-advisor".into(), "forge".into());
        g.add_edge("skill-advisor".into(), "primitive-extractor".into());
        g.add_edge("primitive-extractor".into(), "ctvp-validator".into());
        g.add_node("standalone".into());
        g
    }

    #[test]
    fn fan_counts() {
        let g = sample_graph();
        assert_eq!(g.fan_out("forge"), 3);
        assert_eq!(g.fan_in("primitive-extractor"), 2);
        assert_eq!(g.fan_out("standalone"), 0);
    }

    #[test]
    fn analysis_stats() {
        let a = sample_graph().analyze();
        assert_eq!(a.node_count, 7);
        assert_eq!(a.edge_count, 6);
    }

    #[test]
    fn density_valid() {
        let a = sample_graph().analyze();
        assert!(a.density.value() >= 0.0 && a.density.value() <= 1.0);
    }

    #[test]
    fn dag_no_cycles() {
        assert_eq!(sample_graph().analyze().cycle_count, 0);
    }

    #[test]
    fn cycle_detected() {
        let mut g = AdjacencyList::new();
        g.add_edge("a".into(), "b".into());
        g.add_edge("b".into(), "c".into());
        g.add_edge("c".into(), "a".into());
        assert!(g.analyze().cycle_count > 0);
    }

    #[test]
    fn role_classification() {
        assert_eq!(classify_role(10, 10, 58), SkillRole::Hub);
        assert_eq!(classify_role(10, 0, 58), SkillRole::Foundation);
        assert_eq!(classify_role(0, 10, 58), SkillRole::Orchestrator);
        assert_eq!(classify_role(1, 1, 58), SkillRole::Leaf);
    }

    #[test]
    fn linear_chain_depth() {
        let mut g = AdjacencyList::new();
        g.add_edge("a".into(), "b".into());
        g.add_edge("b".into(), "c".into());
        g.add_edge("c".into(), "d".into());
        assert_eq!(g.analyze().max_depth, 3);
    }

    #[test]
    fn empty_graph_ok() {
        let a = AdjacencyList::new().analyze();
        assert_eq!(a.node_count, 0);
        assert_eq!(a.edge_count, 0);
    }

    #[test]
    fn betweenness_normalized() {
        for node in &sample_graph().analyze().nodes {
            assert!(node.betweenness.value() >= 0.0 && node.betweenness.value() <= 1.0);
        }
    }
}
