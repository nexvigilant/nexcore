//! # Graph Algorithms
//!
//! DAG operations for skill dependency management and execution scheduling.
//!
//! ## Algorithms
//!
//! - **Topological sort** - Kahn's algorithm with cycle detection
//! - **Level parallelization** - Compute parallel execution levels
//! - **Shortest path** - Dijkstra's algorithm for weighted graphs
//! - **Resource conflict detection** - Identify concurrent write conflicts
//!
//! ## Example
//!
//! ```rust
//! use nexcore_vigilance::foundation::algorithms::graph::{SkillGraph, SkillNode};
//!
//! let mut graph = SkillGraph::new();
//! graph.add_node(SkillNode::simple("a", vec![]));
//! graph.add_node(SkillNode::simple("b", vec!["a"]));
//! graph.add_node(SkillNode::simple("c", vec!["a"]));
//! graph.add_node(SkillNode::simple("d", vec!["b", "c"]));
//!
//! // Get execution order
//! let order = graph.topological_sort().unwrap();
//! assert!(order.iter().position(|x| x == "a") < order.iter().position(|x| x == "d"));
//!
//! // Get parallel levels
//! let levels = graph.level_parallelization().unwrap();
//! assert_eq!(levels.len(), 3); // [a], [b, c], [d]
//! ```

use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap, HashSet, VecDeque};

/// A node in the skill dependency graph
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct SkillNode {
    /// Unique name of this skill
    pub name: String,
    /// Skills that must complete before this one
    pub dependencies: Vec<String>,
    /// Outputs produced by this skill (for conflict detection)
    #[serde(default)]
    pub outputs: Vec<String>,
    /// Weighted edges to other skills
    pub adjacencies: Vec<Adjacency>,
}

impl SkillNode {
    /// Create a simple node with just name and dependencies
    #[must_use]
    pub fn simple(name: &str, deps: Vec<&str>) -> Self {
        Self {
            name: name.to_string(),
            dependencies: deps.into_iter().map(String::from).collect(),
            outputs: vec![],
            adjacencies: vec![],
        }
    }

    /// Create a node with adjacencies for shortest path calculations
    #[must_use]
    pub fn with_adjacencies(name: &str, deps: Vec<&str>, adjs: Vec<(&str, f32)>) -> Self {
        Self {
            name: name.to_string(),
            dependencies: deps.into_iter().map(String::from).collect(),
            outputs: vec![],
            adjacencies: adjs
                .into_iter()
                .map(|(t, w)| Adjacency {
                    target: t.to_string(),
                    weight: w,
                    when: "success".to_string(),
                    action: String::new(),
                })
                .collect(),
        }
    }
}

/// Weighted edge in the skill graph
#[derive(Debug, Serialize, Clone)]
pub struct Adjacency {
    /// Target node name
    pub target: String,
    /// Edge weight (for shortest path)
    pub weight: f32,
    /// Condition for traversal (e.g., "success", "failure")
    pub when: String,
    /// Action to perform on traversal
    pub action: String,
}

impl<'de> serde::Deserialize<'de> for Adjacency {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct AdjacencyVisitor;

        impl<'de> serde::de::Visitor<'de> for AdjacencyVisitor {
            type Value = Adjacency;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("string or map for Adjacency")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Adjacency {
                    target: value.to_string(),
                    weight: 0.5,
                    when: "success".to_string(),
                    action: String::new(),
                })
            }

            fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error>
            where
                M: serde::de::MapAccess<'de>,
            {
                let mut target = String::new();
                let mut weight = 0.5;
                let mut when = "success".to_string();
                let mut action = String::new();

                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "target" => target = map.next_value()?,
                        "weight" => weight = map.next_value()?,
                        "when" => when = map.next_value()?,
                        "action" => action = map.next_value()?,
                        _ => {
                            let _: serde::de::IgnoredAny = map.next_value()?;
                        }
                    }
                }

                Ok(Adjacency {
                    target,
                    weight,
                    when,
                    action,
                })
            }
        }

        deserializer.deserialize_any(AdjacencyVisitor)
    }
}

/// Directed graph of skill dependencies
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct SkillGraph {
    /// Map of node name to node data
    pub nodes: HashMap<String, SkillNode>,
}

impl From<HashMap<String, Vec<String>>> for SkillGraph {
    fn from(map: HashMap<String, Vec<String>>) -> Self {
        let mut graph = Self::new();

        let mut all_nodes = HashSet::new();
        for (node, neighbors) in &map {
            all_nodes.insert(node.clone());
            for neighbor in neighbors {
                all_nodes.insert(neighbor.clone());
            }
        }

        for node_name in all_nodes {
            let adjacencies = map
                .get(&node_name)
                .map(|neighbors| {
                    neighbors
                        .iter()
                        .map(|n| Adjacency {
                            target: n.clone(),
                            weight: 1.0,
                            when: "success".to_string(),
                            action: String::new(),
                        })
                        .collect()
                })
                .unwrap_or_default();

            graph.add_node(SkillNode {
                name: node_name,
                dependencies: vec![],
                outputs: vec![],
                adjacencies,
            });
        }
        graph
    }
}

impl SkillGraph {
    /// Create a new empty graph
    #[must_use]
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
        }
    }

    /// Add a node to the graph
    pub fn add_node(&mut self, node: SkillNode) {
        self.nodes.insert(node.name.clone(), node);
    }

    /// Returns a list of skill names in topological order.
    ///
    /// # Errors
    ///
    /// Returns `Err(Vec<String>)` containing the cycle if a cycle is detected,
    /// or an error message if a dependency is missing.
    pub fn topological_sort(&self) -> Result<Vec<String>, Vec<String>> {
        let mut in_degree: HashMap<String, usize> = HashMap::new();
        let mut adj: HashMap<String, Vec<String>> = HashMap::new();

        for name in self.nodes.keys() {
            in_degree.insert(name.clone(), 0);
        }

        for (name, node) in &self.nodes {
            for dep in &node.dependencies {
                if !self.nodes.contains_key(dep) {
                    return Err(vec![format!("Missing dependency: {dep} for node {name}")]);
                }
                adj.entry(dep.clone()).or_default().push(name.clone());
                *in_degree.entry(name.clone()).or_insert(0) += 1;
            }
        }

        let mut queue: VecDeque<String> = in_degree
            .iter()
            .filter(|&(_, &degree)| degree == 0)
            .map(|(name, _)| name.clone())
            .collect();

        let mut sorted = Vec::new();
        while let Some(u) = queue.pop_front() {
            sorted.push(u.clone());
            if let Some(neighbors) = adj.get(&u) {
                for v in neighbors {
                    if let Some(degree) = in_degree.get_mut(v) {
                        *degree -= 1;
                        if *degree == 0 {
                            queue.push_back(v.clone());
                        }
                    }
                }
            }
        }

        if sorted.len() == self.nodes.len() {
            Ok(sorted)
        } else {
            Err(self.find_cycle())
        }
    }

    /// Find a cycle in the graph (helper for error reporting)
    fn find_cycle(&self) -> Vec<String> {
        let mut visited = HashSet::new();
        let mut on_stack = Vec::new();
        let mut cycle = Vec::new();

        fn dfs(
            u: &str,
            nodes: &HashMap<String, SkillNode>,
            visited: &mut HashSet<String>,
            on_stack: &mut Vec<String>,
            cycle: &mut Vec<String>,
        ) -> bool {
            visited.insert(u.to_string());
            on_stack.push(u.to_string());

            if let Some(node) = nodes.get(u) {
                for dep in &node.dependencies {
                    if on_stack.contains(dep) {
                        let pos = on_stack.iter().position(|x| x == dep).unwrap_or(0);
                        *cycle = on_stack[pos..].to_vec();
                        return true;
                    }
                    if !visited.contains(dep) && dfs(dep, nodes, visited, on_stack, cycle) {
                        return true;
                    }
                }
            }

            on_stack.pop();
            false
        }

        for node in self.nodes.keys() {
            if !visited.contains(node)
                && dfs(node, &self.nodes, &mut visited, &mut on_stack, &mut cycle)
            {
                break;
            }
        }

        cycle
    }

    /// Computes parallel execution levels for DAG vertices.
    ///
    /// Vertices at the same level can be executed concurrently.
    ///
    /// # Errors
    ///
    /// Returns `Err(Vec<String>)` containing the cycle if a cycle is detected.
    pub fn level_parallelization(&self) -> Result<Vec<Vec<String>>, Vec<String>> {
        if self.nodes.is_empty() {
            return Ok(vec![]);
        }

        let mut in_degree: HashMap<String, usize> = HashMap::new();
        let mut adj: HashMap<String, Vec<String>> = HashMap::new();

        for name in self.nodes.keys() {
            in_degree.insert(name.clone(), 0);
        }

        for (name, node) in &self.nodes {
            for dep in &node.dependencies {
                if !self.nodes.contains_key(dep) {
                    return Err(vec![format!("Missing dependency: {dep} for node {name}")]);
                }
                adj.entry(dep.clone()).or_default().push(name.clone());
                *in_degree.entry(name.clone()).or_insert(0) += 1;
            }
        }

        let mut levels: Vec<Vec<String>> = Vec::new();
        let mut processed_count = 0;

        let mut current_level: Vec<String> = in_degree
            .iter()
            .filter(|&(_, &degree)| degree == 0)
            .map(|(name, _)| name.clone())
            .collect();

        while !current_level.is_empty() {
            current_level.sort();
            processed_count += current_level.len();

            let mut next_level: Vec<String> = Vec::new();

            for node in &current_level {
                if let Some(neighbors) = adj.get(node) {
                    for neighbor in neighbors {
                        if let Some(degree) = in_degree.get_mut(neighbor) {
                            *degree -= 1;
                            if *degree == 0 {
                                next_level.push(neighbor.clone());
                            }
                        }
                    }
                }
            }

            levels.push(current_level);
            current_level = next_level;
        }

        if processed_count == self.nodes.len() {
            Ok(levels)
        } else {
            self.topological_sort().map(|_| levels)
        }
    }

    /// Detects resource conflicts (multiple nodes at the same level writing to the same output)
    #[must_use]
    pub fn detect_resource_conflicts(&self) -> Vec<String> {
        let mut conflicts = Vec::new();
        if let Ok(levels) = self.level_parallelization() {
            for level in levels {
                let mut level_outputs = HashMap::new();
                for node_name in level {
                    if let Some(node) = self.nodes.get(&node_name) {
                        for output in &node.outputs {
                            if let Some(other_node) =
                                level_outputs.insert(output.clone(), node_name.clone())
                            {
                                conflicts.push(format!(
                                    "Resource conflict: nodes '{other_node}' and '{node_name}' both write to output '{output}'"
                                ));
                            }
                        }
                    }
                }
            }
        }
        conflicts
    }

    /// Finds the shortest path between two skills based on adjacency weights.
    ///
    /// Uses Dijkstra's algorithm.
    ///
    /// # Returns
    ///
    /// `Some((path, cost))` if a path exists, `None` otherwise.
    #[must_use]
    pub fn shortest_path(&self, start: &str, end: &str) -> Option<(Vec<String>, f32)> {
        if !self.nodes.contains_key(start) || !self.nodes.contains_key(end) {
            return None;
        }

        let mut distances: HashMap<String, f32> = HashMap::new();
        let mut previous: HashMap<String, String> = HashMap::new();
        let mut visited = HashSet::new();
        let mut pq = BinaryHeap::new();

        #[derive(PartialEq)]
        struct NodeScore(f32, String);
        impl Eq for NodeScore {}
        impl PartialOrd for NodeScore {
            fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
                Some(self.cmp(other))
            }
        }
        impl Ord for NodeScore {
            fn cmp(&self, other: &Self) -> Ordering {
                other.0.partial_cmp(&self.0).unwrap_or(Ordering::Equal)
            }
        }

        distances.insert(start.to_string(), 0.0);
        pq.push(NodeScore(0.0, start.to_string()));

        while let Some(NodeScore(dist, u)) = pq.pop() {
            if u == end {
                let mut path = Vec::new();
                let mut curr = end.to_string();
                while let Some(prev) = previous.get(&curr) {
                    path.push(curr);
                    curr = prev.clone();
                }
                path.push(start.to_string());
                path.reverse();
                return Some((path, dist));
            }

            if visited.contains(&u) {
                continue;
            }
            visited.insert(u.clone());

            if let Some(node) = self.nodes.get(&u) {
                for adj in &node.adjacencies {
                    let v = &adj.target;
                    let weight = adj.weight;
                    let new_dist = dist + weight;

                    if new_dist < *distances.get(v).unwrap_or(&f32::INFINITY) {
                        distances.insert(v.clone(), new_dist);
                        previous.insert(v.clone(), u.clone());
                        pq.push(NodeScore(new_dist, v.clone()));
                    }
                }
            }
        }

        None
    }

    /// Traces a failure from a target node back to its root cause in the dependencies.
    ///
    /// Returns the chain of dependency nodes that led to the failed target.
    #[must_use]
    pub fn trace_failure(&self, failed_node: &str) -> Vec<String> {
        let mut path = Vec::new();
        let mut current = failed_node.to_string();

        while let Some(node) = self.nodes.get(&current) {
            path.push(current.clone());
            // Simplistic trace: pick the first dependency that exists
            // In production, this would use execution logs to find the ACTUAL failed dependency
            if let Some(first_dep) = node.dependencies.first() {
                current = first_dep.clone();
            } else {
                break;
            }
        }

        path.reverse();
        path
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_topsort_linear_chain() {
        let mut graph = SkillGraph::new();
        graph.add_node(SkillNode::simple("c", vec!["b"]));
        graph.add_node(SkillNode::simple("b", vec!["a"]));
        graph.add_node(SkillNode::simple("a", vec![]));

        let result = graph.topological_sort().unwrap();
        let pos_a = result.iter().position(|x| x == "a").unwrap();
        let pos_b = result.iter().position(|x| x == "b").unwrap();
        let pos_c = result.iter().position(|x| x == "c").unwrap();

        assert!(pos_a < pos_b);
        assert!(pos_b < pos_c);
    }

    #[test]
    fn test_topsort_diamond() {
        let mut graph = SkillGraph::new();
        graph.add_node(SkillNode::simple("a", vec![]));
        graph.add_node(SkillNode::simple("b", vec!["a"]));
        graph.add_node(SkillNode::simple("c", vec!["a"]));
        graph.add_node(SkillNode::simple("d", vec!["b", "c"]));

        let result = graph.topological_sort().unwrap();
        let pos_a = result.iter().position(|x| x == "a").unwrap();
        let pos_d = result.iter().position(|x| x == "d").unwrap();

        assert!(pos_a < pos_d);
    }

    #[test]
    fn test_topsort_cycle() {
        let mut graph = SkillGraph::new();
        graph.add_node(SkillNode::simple("a", vec!["b"]));
        graph.add_node(SkillNode::simple("b", vec!["a"]));

        assert!(graph.topological_sort().is_err());
    }

    #[test]
    fn test_level_parallelization_diamond() {
        let mut graph = SkillGraph::new();
        graph.add_node(SkillNode::simple("a", vec![]));
        graph.add_node(SkillNode::simple("b", vec!["a"]));
        graph.add_node(SkillNode::simple("c", vec!["a"]));
        graph.add_node(SkillNode::simple("d", vec!["b", "c"]));

        let levels = graph.level_parallelization().unwrap();
        assert_eq!(levels.len(), 3);
        assert_eq!(levels[0], vec!["a"]);
        assert!(levels[1].contains(&"b".to_string()));
        assert!(levels[1].contains(&"c".to_string()));
        assert_eq!(levels[2], vec!["d"]);
    }

    #[test]
    fn test_shortest_path_direct() {
        let mut graph = SkillGraph::new();
        graph.add_node(SkillNode::with_adjacencies("a", vec![], vec![("b", 5.0)]));
        graph.add_node(SkillNode::with_adjacencies("b", vec![], vec![]));

        let (path, cost) = graph.shortest_path("a", "b").unwrap();
        assert_eq!(path, vec!["a", "b"]);
        assert_eq!(cost, 5.0);
    }

    #[test]
    fn test_resource_conflict() {
        let mut graph = SkillGraph::new();
        graph.add_node(SkillNode {
            name: "a".to_string(),
            outputs: vec!["shared_var".to_string()],
            ..Default::default()
        });
        graph.add_node(SkillNode {
            name: "b".to_string(),
            outputs: vec!["shared_var".to_string()],
            ..Default::default()
        });

        let conflicts = graph.detect_resource_conflicts();
        assert_eq!(conflicts.len(), 1);
        assert!(conflicts[0].contains("shared_var"));
    }
}
