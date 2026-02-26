//! Directed concept graph — adjacency list representation.
//!
//! Topological sort follows `foundation_graph_topsort` pattern.

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use serde::{Deserialize, Serialize};

/// Relation type between concepts.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum ConceptRelation {
    DependsOn,
    Contains,
    RelatedTo,
    DerivedFrom,
    Implements,
}

impl std::fmt::Display for ConceptRelation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DependsOn => write!(f, "depends_on"),
            Self::Contains => write!(f, "contains"),
            Self::RelatedTo => write!(f, "related_to"),
            Self::DerivedFrom => write!(f, "derived_from"),
            Self::Implements => write!(f, "implements"),
        }
    }
}

/// A node in the concept graph.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConceptNode {
    pub name: String,
    pub domain: Option<String>,
    pub frequency: usize,
}

/// A directed edge in the concept graph.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConceptEdge {
    pub from: String,
    pub to: String,
    pub relation: ConceptRelation,
    pub weight: f64,
}

/// Directed concept graph with adjacency list.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConceptGraph {
    pub nodes: BTreeMap<String, ConceptNode>,
    pub edges: Vec<ConceptEdge>,
    adjacency: BTreeMap<String, Vec<usize>>,
}

impl ConceptGraph {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add or update a concept node.
    pub fn add_concept(&mut self, name: &str, domain: Option<String>) {
        let entry = self.nodes.entry(name.to_string()).or_insert(ConceptNode {
            name: name.to_string(),
            domain,
            frequency: 0,
        });
        entry.frequency += 1;
    }

    /// Add a directed edge between concepts.
    ///
    /// If an edge with the same `(from, to, relation)` already exists, averages the
    /// weight rather than duplicating the edge. This prevents the edge list from
    /// growing O(fragments × concepts²) when multiple fragments share concepts.
    pub fn add_edge(&mut self, from: &str, to: &str, relation: ConceptRelation, weight: f64) {
        // Ensure both nodes exist
        if !self.nodes.contains_key(from) {
            self.add_concept(from, None);
        }
        if !self.nodes.contains_key(to) {
            self.add_concept(to, None);
        }

        // Deduplicate: if this exact (from, to, relation) edge exists, average weight
        if let Some(existing) = self
            .edges
            .iter_mut()
            .find(|e| e.from == from && e.to == to && e.relation == relation)
        {
            existing.weight = (existing.weight + weight) / 2.0;
            return;
        }

        let edge_idx = self.edges.len();
        self.edges.push(ConceptEdge {
            from: from.to_string(),
            to: to.to_string(),
            relation,
            weight,
        });
        self.adjacency
            .entry(from.to_string())
            .or_default()
            .push(edge_idx);
    }

    /// Get outgoing edges from a concept.
    pub fn outgoing(&self, name: &str) -> Vec<&ConceptEdge> {
        self.adjacency
            .get(name)
            .map(|indices| indices.iter().filter_map(|&i| self.edges.get(i)).collect())
            .unwrap_or_default()
    }

    /// Topological sort (Kahn's algorithm). Returns None if cycle detected.
    pub fn topsort(&self) -> Option<Vec<String>> {
        let mut in_degree: BTreeMap<&str, usize> = BTreeMap::new();
        for node in self.nodes.keys() {
            in_degree.entry(node).or_insert(0);
        }
        for edge in &self.edges {
            *in_degree.entry(&edge.to).or_insert(0) += 1;
        }

        let mut queue: VecDeque<String> = in_degree
            .iter()
            .filter(|&(_, deg)| *deg == 0)
            .map(|(name, _)| name.to_string())
            .collect();

        let mut result = Vec::new();
        let mut visited = 0;

        while let Some(node) = queue.pop_front() {
            result.push(node.clone());
            visited += 1;

            for edge in self.outgoing(&node) {
                if let Some(deg) = in_degree.get_mut(edge.to.as_str()) {
                    *deg -= 1;
                    if *deg == 0 {
                        queue.push_back(edge.to.clone());
                    }
                }
            }
        }

        if visited == self.nodes.len() {
            Some(result)
        } else {
            None // cycle
        }
    }

    /// Get all domains present in the graph.
    pub fn domains(&self) -> BTreeSet<String> {
        self.nodes
            .values()
            .filter_map(|n| n.domain.clone())
            .collect()
    }

    /// Count of nodes.
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Count of edges.
    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_and_query() {
        let mut g = ConceptGraph::new();
        g.add_concept("signal", Some("pv".to_string()));
        g.add_concept("detection", Some("pv".to_string()));
        g.add_edge("signal", "detection", ConceptRelation::RelatedTo, 0.9);

        assert_eq!(g.node_count(), 2);
        assert_eq!(g.edge_count(), 1);
        assert_eq!(g.outgoing("signal").len(), 1);
    }

    #[test]
    fn topsort_dag() {
        let mut g = ConceptGraph::new();
        g.add_concept("a", None);
        g.add_concept("b", None);
        g.add_concept("c", None);
        g.add_edge("a", "b", ConceptRelation::DependsOn, 1.0);
        g.add_edge("b", "c", ConceptRelation::DependsOn, 1.0);

        let sorted = g.topsort().unwrap();
        assert_eq!(sorted[0], "a");
        assert_eq!(sorted[2], "c");
    }

    #[test]
    fn topsort_cycle_detection() {
        let mut g = ConceptGraph::new();
        g.add_concept("a", None);
        g.add_concept("b", None);
        g.add_edge("a", "b", ConceptRelation::DependsOn, 1.0);
        g.add_edge("b", "a", ConceptRelation::DependsOn, 1.0);

        assert!(g.topsort().is_none());
    }
}
