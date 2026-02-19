//! Dependency graph analysis.

use super::{DependencyCycle, LayerDefinition, LayerViolation, identify_layer};
use regex::Regex;
use std::collections::{HashMap, HashSet};

/// Dependency graph
#[derive(Debug, Default)]
pub struct DependencyGraph {
    nodes: HashSet<String>,
    edges: HashMap<String, Vec<String>>,
}

impl DependencyGraph {
    /// New graph
    pub fn new() -> Self {
        Self::default()
    }

    /// Add node
    pub fn add_node(&mut self, m: &str) {
        self.nodes.insert(m.to_string());
    }

    /// Add edge
    pub fn add_edge(&mut self, from: &str, to: &str) {
        self.nodes.insert(from.to_string());
        self.nodes.insert(to.to_string());
        self.edges
            .entry(from.to_string())
            .or_default()
            .push(to.to_string());
    }

    /// Neighbors
    pub fn neighbors(&self, n: &str) -> Vec<&String> {
        self.edges
            .get(n)
            .map(|v| v.iter().collect())
            .unwrap_or_default()
    }

    /// Incoming count
    pub fn count_incoming(&self, n: &str) -> usize {
        self.edges
            .values()
            .filter(|t| t.contains(&n.to_string()))
            .count()
    }

    /// Outgoing count
    pub fn count_outgoing(&self, n: &str) -> usize {
        self.edges.get(n).map(|v| v.len()).unwrap_or(0)
    }

    /// Detect cycles
    pub fn detect_cycles(&self) -> Vec<DependencyCycle> {
        let mut cycles = Vec::new();
        let mut visited = HashSet::new();
        let mut stack = HashSet::new();
        for node in &self.nodes {
            if !visited.contains(node) {
                self.dfs(node, &mut visited, &mut stack, &mut vec![], &mut cycles);
            }
        }
        cycles
    }

    fn dfs(
        &self,
        node: &str,
        visited: &mut HashSet<String>,
        stack: &mut HashSet<String>,
        path: &mut Vec<String>,
        cycles: &mut Vec<DependencyCycle>,
    ) {
        visited.insert(node.to_string());
        stack.insert(node.to_string());
        path.push(node.to_string());
        for nb in self.neighbors(node) {
            if !visited.contains(nb) {
                self.dfs(nb, visited, stack, path, cycles);
            } else if stack.contains(nb) {
                if let Some(i) = path.iter().position(|x| x == nb) {
                    let mut c: Vec<String> = path[i..].to_vec();
                    c.push(nb.clone());
                    cycles.push(DependencyCycle { modules: c });
                }
            }
        }
        path.pop();
        stack.remove(node);
    }

    /// Layer violations
    pub fn detect_layer_violations(&self, layers: &[LayerDefinition]) -> Vec<LayerViolation> {
        let mut v = Vec::new();
        for (from, targets) in &self.edges {
            let fl = identify_layer(from, layers);
            for to in targets {
                let tl = identify_layer(to, layers);
                if let (Some(ref f), Some(ref t)) = (&fl, &tl) {
                    if f != t {
                        if let Some(def) = layers.iter().find(|l| &l.name == f) {
                            if !def.can_depend_on.contains(t) {
                                v.push(LayerViolation {
                                    source: from.clone(),
                                    source_layer: f.clone(),
                                    target: to.clone(),
                                    target_layer: t.clone(),
                                    message: format!("{f} should not depend on {t}"),
                                });
                            }
                        }
                    }
                }
            }
        }
        v
    }
}

/// Extract use statements
pub fn extract_use_statements(code: &str) -> Vec<String> {
    let mut uses = Vec::new();
    for p in &[r"use\s+crate::(\w+)", r"use\s+super::(\w+)"] {
        if let Ok(re) = Regex::new(p) {
            for cap in re.captures_iter(code) {
                if let Some(m) = cap.get(1) {
                    uses.push(m.as_str().to_string());
                }
            }
        }
    }
    uses
}

/// Extract mod declarations
pub fn extract_mod_declarations(code: &str) -> Vec<String> {
    let mut mods = Vec::new();
    if let Ok(re) = Regex::new(r"(?:pub\s+)?mod\s+(\w+)") {
        for cap in re.captures_iter(code) {
            if let Some(m) = cap.get(1) {
                let n = m.as_str();
                if n != "tests" {
                    mods.push(n.to_string());
                }
            }
        }
    }
    mods
}
