//! Query engine for the Learning Graph.
//!
//! Supports:
//! - Shortest path to a target capability (ALO or KSB)
//! - Capability surface (unlocked ALOs given completed set)
//! - Learning time estimates
//! - Critical path computation

use std::collections::{HashMap, HashSet, VecDeque};

use crate::ir::{AloEdgeType, AtomicLearningObject, LearningGraph};

/// Result of a shortest-path query.
#[non_exhaustive]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PathResult {
    /// Ordered list of ALO IDs from start to target.
    pub path: Vec<String>,
    /// Total estimated learning time in minutes.
    pub total_duration_min: u32,
    /// Number of ALOs in the path.
    pub alo_count: usize,
}

/// Result of a capability surface query.
#[non_exhaustive]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CapabilitySurface {
    /// ALO IDs now unlocked (all prereqs satisfied).
    pub unlocked: Vec<String>,
    /// ALO IDs still locked (missing prereqs).
    pub locked: Vec<String>,
    /// Percentage of total ALOs unlocked.
    pub unlock_ratio: f32,
}

/// Find the shortest path (minimum ALOs) to reach a target ALO.
///
/// Uses reverse BFS from the target, following Prereq edges backward,
/// then skipping any ALOs already in the `completed` set.
pub fn shortest_path_to_alo(
    graph: &LearningGraph,
    target_alo_id: &str,
    completed: &HashSet<String>,
) -> Option<PathResult> {
    // Build reverse adjacency (target → sources via Prereq)
    let mut reverse_adj: HashMap<&str, Vec<&str>> = HashMap::new();
    for node in &graph.nodes {
        reverse_adj.entry(node.id.as_str()).or_default();
    }
    for edge in &graph.edges {
        if edge.edge_type == AloEdgeType::Prereq {
            reverse_adj
                .entry(edge.to.as_str())
                .or_default()
                .push(edge.from.as_str());
        }
    }

    // BFS backward from target to find all required prereqs
    let mut required: Vec<&str> = Vec::new();
    let mut visited: HashSet<&str> = HashSet::new();
    let mut queue: VecDeque<&str> = VecDeque::new();

    queue.push_back(target_alo_id);
    visited.insert(target_alo_id);

    while let Some(node) = queue.pop_front() {
        required.push(node);
        if let Some(prereqs) = reverse_adj.get(node) {
            for &prereq in prereqs {
                if !visited.contains(prereq) && !completed.contains(prereq) {
                    visited.insert(prereq);
                    queue.push_back(prereq);
                }
            }
        }
    }

    // Filter out completed ALOs
    let path: Vec<String> = required
        .into_iter()
        .filter(|id| !completed.contains(*id))
        .map(|s| s.to_string())
        .collect();

    if path.is_empty() {
        return None;
    }

    // Topological sort of the path for correct ordering
    let path_set: HashSet<&str> = path.iter().map(|s| s.as_str()).collect();
    let sorted = topological_sort(&path_set, &graph.edges);

    let alo_map: HashMap<&str, &AtomicLearningObject> =
        graph.nodes.iter().map(|a| (a.id.as_str(), a)).collect();

    let total_duration: u32 = sorted
        .iter()
        .filter_map(|id| alo_map.get(id.as_str()))
        .map(|a| u32::from(a.estimated_duration))
        .sum();

    let alo_count = sorted.len();

    Some(PathResult {
        path: sorted,
        total_duration_min: total_duration,
        alo_count,
    })
}

/// Find the shortest path to reach a target KSB.
///
/// Finds all ALOs referencing the target KSB, then returns the shortest
/// path to any of them.
pub fn shortest_path_to_ksb(
    graph: &LearningGraph,
    target_ksb: &str,
    completed: &HashSet<String>,
) -> Option<PathResult> {
    // Find ALOs that address this KSB
    let target_alos: Vec<&str> = graph
        .nodes
        .iter()
        .filter(|a| a.ksb_refs.iter().any(|k| k == target_ksb))
        .map(|a| a.id.as_str())
        .collect();

    if target_alos.is_empty() {
        return None;
    }

    // Find shortest path to any target ALO
    let mut best: Option<PathResult> = None;

    for target in target_alos {
        if let Some(result) = shortest_path_to_alo(graph, target, completed) {
            let is_shorter = best
                .as_ref()
                .map_or(true, |b| result.alo_count < b.alo_count);
            if is_shorter {
                best = Some(result);
            }
        }
    }

    best
}

/// Compute the capability surface: which ALOs are now unlocked given completed set.
pub fn capability_surface(graph: &LearningGraph, completed: &HashSet<String>) -> CapabilitySurface {
    // Build prereq map: ALO → set of prereq ALO IDs
    let mut prereq_map: HashMap<&str, HashSet<&str>> = HashMap::new();
    for node in &graph.nodes {
        prereq_map.entry(node.id.as_str()).or_default();
    }
    for edge in &graph.edges {
        if edge.edge_type == AloEdgeType::Prereq {
            prereq_map
                .entry(edge.to.as_str())
                .or_default()
                .insert(edge.from.as_str());
        }
    }

    let mut unlocked = Vec::new();
    let mut locked = Vec::new();

    for node in &graph.nodes {
        if completed.contains(&node.id) {
            continue; // Already done
        }

        let prereqs = prereq_map
            .get(node.id.as_str())
            .cloned()
            .unwrap_or_default();
        let all_prereqs_met = prereqs.iter().all(|p| completed.contains(*p));

        if all_prereqs_met {
            unlocked.push(node.id.clone());
        } else {
            locked.push(node.id.clone());
        }
    }

    let total = graph.nodes.len();
    let completed_and_unlocked = completed.len().saturating_add(unlocked.len());
    #[allow(
        clippy::as_conversions,
        clippy::cast_precision_loss,
        reason = "usize->f32 cast for ratio computation; node count fits in f32 for practical dataset sizes"
    )]
    let unlock_ratio = if total > 0 {
        completed_and_unlocked as f32 / total as f32
    } else {
        0.0
    };

    CapabilitySurface {
        unlocked,
        locked,
        unlock_ratio,
    }
}

/// Estimate remaining learning time to reach a target ALO.
pub fn learning_time_estimate(
    graph: &LearningGraph,
    target_alo_id: &str,
    completed: &HashSet<String>,
) -> Option<u32> {
    shortest_path_to_alo(graph, target_alo_id, completed).map(|r| r.total_duration_min)
}

/// Topological sort of a subset of nodes using graph edges.
fn topological_sort(node_ids: &HashSet<&str>, edges: &[crate::ir::AloEdge]) -> Vec<String> {
    let relevant_edges: Vec<&crate::ir::AloEdge> = edges
        .iter()
        .filter(|e| {
            e.edge_type == AloEdgeType::Prereq
                && node_ids.contains(e.from.as_str())
                && node_ids.contains(e.to.as_str())
        })
        .collect();

    let mut in_degree: HashMap<&str, usize> = HashMap::new();
    let mut adj: HashMap<&str, Vec<&str>> = HashMap::new();

    for &id in node_ids {
        in_degree.insert(id, 0);
        adj.insert(id, Vec::new());
    }

    for edge in &relevant_edges {
        let deg = in_degree.entry(edge.to.as_str()).or_insert(0);
        *deg = deg.saturating_add(1);
        adj.entry(edge.from.as_str())
            .or_default()
            .push(edge.to.as_str());
    }

    // Collect into sorted vec first for deterministic topological order
    let mut zero_degree: Vec<&str> = in_degree
        .iter()
        .filter_map(|(&node, &deg)| if deg == 0 { Some(node) } else { None })
        .collect();
    zero_degree.sort_unstable();

    let mut queue: VecDeque<&str> = VecDeque::from(zero_degree);

    let mut sorted = Vec::new();
    while let Some(node) = queue.pop_front() {
        sorted.push(node.to_string());
        if let Some(neighbors) = adj.get(node) {
            let mut next_zero: Vec<&str> = Vec::new();
            for &neighbor in neighbors {
                if let Some(deg) = in_degree.get_mut(neighbor) {
                    *deg = deg.saturating_sub(1);
                    if *deg == 0 {
                        next_zero.push(neighbor);
                    }
                }
            }
            next_zero.sort_unstable();
            for n in next_zero {
                queue.push_back(n);
            }
        }
    }

    sorted
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::*;

    fn test_graph() -> LearningGraph {
        let alos = vec![
            AtomicLearningObject {
                id: "p1-01-h01".to_string(),
                title: "Hook".to_string(),
                alo_type: AloType::Hook,
                learning_objective: "Recognize".to_string(),
                estimated_duration: 2,
                bloom_level: BloomLevel::Remember,
                content: String::new(),
                ksb_refs: Vec::new(),
                source_stage_id: "p1-01".to_string(),
                source_activity_id: None,
                assessment: None,
            },
            AtomicLearningObject {
                id: "p1-01-c01".to_string(),
                title: "Concept 1".to_string(),
                alo_type: AloType::Concept,
                learning_objective: "Explain concept".to_string(),
                estimated_duration: 8,
                bloom_level: BloomLevel::Understand,
                content: String::new(),
                ksb_refs: vec!["KSB-001".to_string()],
                source_stage_id: "p1-01".to_string(),
                source_activity_id: None,
                assessment: None,
            },
            AtomicLearningObject {
                id: "p1-01-a01".to_string(),
                title: "Activity 1".to_string(),
                alo_type: AloType::Activity,
                learning_objective: "Apply concept".to_string(),
                estimated_duration: 10,
                bloom_level: BloomLevel::Apply,
                content: String::new(),
                ksb_refs: vec!["KSB-001".to_string()],
                source_stage_id: "p1-01".to_string(),
                source_activity_id: None,
                assessment: None,
            },
            AtomicLearningObject {
                id: "p1-01-r01".to_string(),
                title: "Reflection".to_string(),
                alo_type: AloType::Reflection,
                learning_objective: "Evaluate".to_string(),
                estimated_duration: 4,
                bloom_level: BloomLevel::Evaluate,
                content: String::new(),
                ksb_refs: Vec::new(),
                source_stage_id: "p1-01".to_string(),
                source_activity_id: None,
                assessment: None,
            },
        ];

        let edges = vec![
            AloEdge {
                from: "p1-01-h01".to_string(),
                to: "p1-01-c01".to_string(),
                edge_type: AloEdgeType::Prereq,
                strength: 1.0,
            },
            AloEdge {
                from: "p1-01-c01".to_string(),
                to: "p1-01-a01".to_string(),
                edge_type: AloEdgeType::Prereq,
                strength: 1.0,
            },
            AloEdge {
                from: "p1-01-a01".to_string(),
                to: "p1-01-r01".to_string(),
                edge_type: AloEdgeType::Prereq,
                strength: 1.0,
            },
        ];

        LearningGraph {
            nodes: alos,
            edges,
            pathways: vec!["p1".to_string()],
            overlap_clusters: Vec::new(),
            metadata: GraphMetadata {
                node_count: 4,
                edge_count: 3,
                connected_components: 1,
                diameter: 3,
                avg_duration_min: 6.0,
                total_duration_min: 24,
                overlap_ratio: 0.0,
            },
        }
    }

    #[test]
    fn test_shortest_path_from_empty() {
        let graph = test_graph();
        let completed = HashSet::new();
        let result = shortest_path_to_alo(&graph, "p1-01-r01", &completed);
        assert!(result.is_some());
        let r = result.as_ref();
        // Should include all 4 ALOs (h01 → c01 → a01 → r01)
        assert_eq!(r.map(|r| r.alo_count), Some(4));
        assert_eq!(r.map(|r| r.total_duration_min), Some(24));
    }

    #[test]
    fn test_shortest_path_with_completed() {
        let graph = test_graph();
        let mut completed = HashSet::new();
        completed.insert("p1-01-h01".to_string());
        completed.insert("p1-01-c01".to_string());

        let result = shortest_path_to_alo(&graph, "p1-01-r01", &completed);
        assert!(result.is_some());
        let r = result.as_ref();
        // Should only need a01 → r01
        assert_eq!(r.map(|r| r.alo_count), Some(2));
    }

    #[test]
    fn test_capability_surface_empty() {
        let graph = test_graph();
        let completed = HashSet::new();
        let surface = capability_surface(&graph, &completed);
        // Only hook should be unlocked (no prereqs)
        assert!(surface.unlocked.contains(&"p1-01-h01".to_string()));
        assert!(!surface.unlocked.contains(&"p1-01-c01".to_string()));
    }

    #[test]
    fn test_capability_surface_after_hook() {
        let graph = test_graph();
        let mut completed = HashSet::new();
        completed.insert("p1-01-h01".to_string());
        let surface = capability_surface(&graph, &completed);
        // c01 should now be unlocked
        assert!(surface.unlocked.contains(&"p1-01-c01".to_string()));
    }

    #[test]
    fn test_shortest_path_to_ksb() {
        let graph = test_graph();
        let completed = HashSet::new();
        let result = shortest_path_to_ksb(&graph, "KSB-001", &completed);
        assert!(result.is_some());
    }
}
