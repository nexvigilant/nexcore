//! Learning Graph: cross-pathway DAG builder and query engine.
//!
//! Merges multiple `AtomizedPathway` instances into a unified `LearningGraph`
//! with overlap detection, shortest-path queries, and spatial coordinates.
//!
//! ## Pipeline Position
//!
//! ```text
//! forge_atomize(pathway_1) ──┐
//! forge_atomize(pathway_2) ──┤──► forge_graph ──► LearningGraph JSON
//! forge_atomize(pathway_N) ──┘
//! ```

pub mod queries;

use std::collections::{HashMap, HashSet, VecDeque};

use crate::error::{ForgeError, ForgeResult};
use crate::ir::{
    AloEdge, AloEdgeType, AtomizedPathway, GraphMetadata, LearningGraph, OverlapCluster,
};

/// Build a unified learning graph from multiple atomized pathways.
///
/// Detects cross-pathway overlaps via KSB intersection and optional
/// learning-objective similarity. Validates DAG acyclicity.
pub fn build_graph(
    pathways: &[AtomizedPathway],
    include_fuzzy: bool,
    similarity_threshold: f32,
) -> ForgeResult<LearningGraph> {
    // Step 1: Merge all ALOs and edges
    let mut all_alos = Vec::new();
    let mut all_edges = Vec::new();
    let mut pathway_ids = Vec::new();

    for pathway in pathways {
        all_alos.extend(pathway.alos.clone());
        all_edges.extend(pathway.edges.clone());
        pathway_ids.push(pathway.id.clone());
    }

    // Step 2: Build KSB index for overlap detection
    let mut ksb_index: HashMap<String, Vec<String>> = HashMap::new();
    for alo in &all_alos {
        for ksb in &alo.ksb_refs {
            ksb_index
                .entry(ksb.clone())
                .or_default()
                .push(alo.id.clone());
        }
    }

    // Step 3: Detect cross-pathway overlaps
    let mut overlap_clusters = Vec::new();
    let alo_pathway_map: HashMap<String, String> = all_alos
        .iter()
        .map(|a| {
            let pw = extract_pathway_prefix(&a.id);
            (a.id.clone(), pw)
        })
        .collect();

    for (ksb, alo_ids) in &ksb_index {
        if alo_ids.len() < 2 {
            continue;
        }

        let involved_pathways: HashSet<&String> = alo_ids
            .iter()
            .filter_map(|id| alo_pathway_map.get(id))
            .collect();

        if involved_pathways.len() > 1 {
            // Cross-pathway overlap detected
            let canonical = select_canonical(alo_ids, &all_alos);

            let cluster = OverlapCluster {
                concept: ksb.clone(),
                alo_ids: alo_ids.clone(),
                pathways: involved_pathways.into_iter().cloned().collect(),
                canonical_alo_id: canonical,
            };
            overlap_clusters.push(cluster);

            // Add Strengthens edges between overlapping ALOs
            for i in 0..alo_ids.len() {
                for j in (i + 1)..alo_ids.len() {
                    // Only add cross-pathway edges (Rule E5)
                    let pw_i = alo_pathway_map.get(&alo_ids[i]);
                    let pw_j = alo_pathway_map.get(&alo_ids[j]);
                    if pw_i != pw_j {
                        all_edges.push(AloEdge {
                            from: alo_ids[i].clone(),
                            to: alo_ids[j].clone(),
                            edge_type: AloEdgeType::Strengthens,
                            strength: 0.5,
                        });
                    }
                }
            }
        }
    }

    // Step 4: Fuzzy similarity detection (optional)
    if include_fuzzy {
        let fuzzy_edges =
            detect_fuzzy_overlaps(&all_alos, &alo_pathway_map, similarity_threshold);
        all_edges.extend(fuzzy_edges);
    }

    // Step 5: Validate DAG (no cycles in Prereq subgraph)
    validate_dag_acyclicity(&all_alos, &all_edges)?;

    // Step 6: Compute metadata
    let metadata = compute_metadata(&all_alos, &all_edges, &overlap_clusters);

    Ok(LearningGraph {
        nodes: all_alos,
        edges: all_edges,
        pathways: pathway_ids,
        overlap_clusters,
        metadata,
    })
}

/// Validate that the Prereq edge subgraph has no cycles.
fn validate_dag_acyclicity(
    alos: &[crate::ir::AtomicLearningObject],
    edges: &[AloEdge],
) -> ForgeResult<()> {
    // Kahn's algorithm for topological sort
    let prereq_edges: Vec<&AloEdge> = edges
        .iter()
        .filter(|e| e.edge_type == AloEdgeType::Prereq)
        .collect();

    let node_ids: HashSet<&str> = alos.iter().map(|a| a.id.as_str()).collect();
    let mut in_degree: HashMap<&str, usize> = HashMap::new();
    let mut adj: HashMap<&str, Vec<&str>> = HashMap::new();

    for id in &node_ids {
        in_degree.insert(id, 0);
        adj.insert(id, Vec::new());
    }

    for edge in &prereq_edges {
        if node_ids.contains(edge.from.as_str()) && node_ids.contains(edge.to.as_str()) {
            *in_degree.entry(edge.to.as_str()).or_insert(0) += 1;
            adj.entry(edge.from.as_str())
                .or_default()
                .push(edge.to.as_str());
        }
    }

    let mut queue: VecDeque<&str> = VecDeque::new();
    for (&node, &deg) in &in_degree {
        if deg == 0 {
            queue.push_back(node);
        }
    }

    let mut visited = 0usize;
    while let Some(node) = queue.pop_front() {
        visited += 1;
        if let Some(neighbors) = adj.get(node) {
            for &neighbor in neighbors {
                if let Some(deg) = in_degree.get_mut(neighbor) {
                    *deg = deg.saturating_sub(1);
                    if *deg == 0 {
                        queue.push_back(neighbor);
                    }
                }
            }
        }
    }

    if visited < node_ids.len() {
        return Err(ForgeError::CycleDetected {
            cycle: format!(
                "{} nodes in cycle (of {} total)",
                node_ids.len() - visited,
                node_ids.len()
            ),
        });
    }

    Ok(())
}

/// Select the canonical ALO from a set of overlapping ALOs.
/// Prefers: highest Bloom level, then longest content.
fn select_canonical(
    alo_ids: &[String],
    all_alos: &[crate::ir::AtomicLearningObject],
) -> String {
    let mut best_id = alo_ids.first().cloned().unwrap_or_default();
    let mut best_bloom = 0u8;
    let mut best_content_len = 0usize;

    for id in alo_ids {
        if let Some(alo) = all_alos.iter().find(|a| a.id == *id) {
            let bloom_rank = alo.bloom_level.rank();
            let content_len = alo.content.len();

            if bloom_rank > best_bloom
                || (bloom_rank == best_bloom && content_len > best_content_len)
            {
                best_bloom = bloom_rank;
                best_content_len = content_len;
                best_id = id.clone();
            }
        }
    }

    best_id
}

/// Detect fuzzy overlaps between ALOs via learning-objective similarity.
fn detect_fuzzy_overlaps(
    alos: &[crate::ir::AtomicLearningObject],
    pathway_map: &HashMap<String, String>,
    threshold: f32,
) -> Vec<AloEdge> {
    let mut edges = Vec::new();

    // Only compare cross-pathway pairs of Concept ALOs
    let concepts: Vec<&crate::ir::AtomicLearningObject> = alos
        .iter()
        .filter(|a| a.alo_type == crate::ir::AloType::Concept)
        .collect();

    for i in 0..concepts.len() {
        for j in (i + 1)..concepts.len() {
            let pw_i = pathway_map.get(&concepts[i].id);
            let pw_j = pathway_map.get(&concepts[j].id);
            if pw_i == pw_j {
                continue; // Same pathway, skip
            }

            let sim = jaccard_similarity(
                &concepts[i].learning_objective,
                &concepts[j].learning_objective,
            );

            if sim >= threshold {
                edges.push(AloEdge {
                    from: concepts[i].id.clone(),
                    to: concepts[j].id.clone(),
                    edge_type: AloEdgeType::Strengthens,
                    strength: sim * 0.6,
                });
            }
        }
    }

    edges
}

/// Simple word-level Jaccard similarity between two strings.
fn jaccard_similarity(a: &str, b: &str) -> f32 {
    let set_a: HashSet<&str> = a.split_whitespace().map(|w| w.trim_matches(|c: char| !c.is_alphanumeric())).collect();
    let set_b: HashSet<&str> = b.split_whitespace().map(|w| w.trim_matches(|c: char| !c.is_alphanumeric())).collect();

    let intersection = set_a.intersection(&set_b).count();
    let union = set_a.union(&set_b).count();

    if union == 0 {
        return 0.0;
    }

    intersection as f32 / union as f32
}

/// Extract pathway prefix from ALO ID (e.g., "tov-01" from "tov-01-03-c02").
fn extract_pathway_prefix(alo_id: &str) -> String {
    // Pathway prefix is everything before the stage number pattern
    // Format: {pathway}-{stage_num:02}-{type}{seq}
    let parts: Vec<&str> = alo_id.rsplitn(3, '-').collect();
    if parts.len() >= 3 {
        parts[2].to_string()
    } else {
        alo_id.to_string()
    }
}

/// Compute graph metadata.
fn compute_metadata(
    alos: &[crate::ir::AtomicLearningObject],
    edges: &[AloEdge],
    overlap_clusters: &[OverlapCluster],
) -> GraphMetadata {
    let node_count = alos.len();
    let edge_count = edges.len();

    let total_duration: u32 = alos.iter().map(|a| a.estimated_duration as u32).sum();
    let avg_duration = if node_count > 0 {
        total_duration as f32 / node_count as f32
    } else {
        0.0
    };

    let overlap_alo_ids: HashSet<&str> = overlap_clusters
        .iter()
        .flat_map(|c| c.alo_ids.iter().map(|s| s.as_str()))
        .collect();
    let overlap_ratio = if node_count > 0 {
        overlap_alo_ids.len() as f32 / node_count as f32
    } else {
        0.0
    };

    // Connected components via BFS
    let connected_components = count_connected_components(alos, edges);

    // Diameter: longest shortest-path (simplified — uses BFS from each source)
    let diameter = compute_diameter(alos, edges);

    GraphMetadata {
        node_count,
        edge_count,
        connected_components,
        diameter,
        avg_duration_min: avg_duration,
        total_duration_min: total_duration,
        overlap_ratio,
    }
}

/// Count connected components (treating all edge types as connections).
fn count_connected_components(
    alos: &[crate::ir::AtomicLearningObject],
    edges: &[AloEdge],
) -> usize {
    let mut adj: HashMap<&str, Vec<&str>> = HashMap::new();
    for alo in alos {
        adj.entry(alo.id.as_str()).or_default();
    }
    for edge in edges {
        adj.entry(edge.from.as_str())
            .or_default()
            .push(edge.to.as_str());
        adj.entry(edge.to.as_str())
            .or_default()
            .push(edge.from.as_str());
    }

    let mut visited: HashSet<&str> = HashSet::new();
    let mut components = 0usize;

    for alo in alos {
        if visited.contains(alo.id.as_str()) {
            continue;
        }
        components += 1;
        let mut queue = VecDeque::new();
        queue.push_back(alo.id.as_str());
        while let Some(node) = queue.pop_front() {
            if !visited.insert(node) {
                continue;
            }
            if let Some(neighbors) = adj.get(node) {
                for &n in neighbors {
                    if !visited.contains(n) {
                        queue.push_back(n);
                    }
                }
            }
        }
    }

    components
}

/// Compute graph diameter (longest shortest-path via BFS, Prereq edges only).
fn compute_diameter(
    alos: &[crate::ir::AtomicLearningObject],
    edges: &[AloEdge],
) -> usize {
    // Build adjacency for Prereq edges only
    let mut adj: HashMap<&str, Vec<&str>> = HashMap::new();
    for alo in alos {
        adj.entry(alo.id.as_str()).or_default();
    }
    for edge in edges {
        if edge.edge_type == AloEdgeType::Prereq {
            adj.entry(edge.from.as_str())
                .or_default()
                .push(edge.to.as_str());
        }
    }

    let mut max_dist = 0usize;

    // BFS from each node (only on small graphs; for large graphs use sampling)
    if alos.len() > 500 {
        // Sample 50 nodes for performance
        for alo in alos.iter().step_by(alos.len() / 50 + 1) {
            let dist = bfs_max_distance(alo.id.as_str(), &adj);
            if dist > max_dist {
                max_dist = dist;
            }
        }
    } else {
        for alo in alos {
            let dist = bfs_max_distance(alo.id.as_str(), &adj);
            if dist > max_dist {
                max_dist = dist;
            }
        }
    }

    max_dist
}

fn bfs_max_distance<'a>(start: &'a str, adj: &HashMap<&'a str, Vec<&'a str>>) -> usize {
    let mut visited: HashSet<&str> = HashSet::new();
    let mut queue: VecDeque<(&str, usize)> = VecDeque::new();
    queue.push_back((start, 0));
    visited.insert(start);
    let mut max_dist = 0usize;

    while let Some((node, dist)) = queue.pop_front() {
        if dist > max_dist {
            max_dist = dist;
        }
        if let Some(neighbors) = adj.get(node) {
            for &n in neighbors {
                if visited.insert(n) {
                    queue.push_back((n, dist + 1));
                }
            }
        }
    }

    max_dist
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::{AloType, AtomicLearningObject, BloomLevel};

    fn make_alo(id: &str, alo_type: AloType, ksbs: &[&str]) -> AtomicLearningObject {
        AtomicLearningObject {
            id: id.to_string(),
            title: format!("ALO {id}"),
            alo_type,
            learning_objective: "Test objective".to_string(),
            estimated_duration: 5,
            bloom_level: BloomLevel::Remember,
            content: "Test content".to_string(),
            ksb_refs: ksbs.iter().map(|s| s.to_string()).collect(),
            source_stage_id: "test-01-01".to_string(),
            source_activity_id: None,
            assessment: None,
        }
    }

    #[test]
    fn test_build_graph_single_pathway() {
        let pathway = AtomizedPathway {
            id: "test-01".to_string(),
            title: "Test".to_string(),
            source_pathway_id: "test-01".to_string(),
            alos: vec![
                make_alo("test-01-01-h01", AloType::Hook, &[]),
                make_alo("test-01-01-c01", AloType::Concept, &["KSB-001"]),
            ],
            edges: vec![AloEdge {
                from: "test-01-01-h01".to_string(),
                to: "test-01-01-c01".to_string(),
                edge_type: AloEdgeType::Prereq,
                strength: 1.0,
            }],
        };

        let graph = build_graph(&[pathway], false, 0.75);
        assert!(graph.is_ok());
        let graph = graph.ok();
        assert!(graph.is_some());
        let g = graph.as_ref();
        assert_eq!(g.map(|g| g.nodes.len()), Some(2));
        assert_eq!(g.map(|g| g.metadata.connected_components), Some(1));
    }

    #[test]
    fn test_overlap_detection() {
        let pw1 = AtomizedPathway {
            id: "pw1-01".to_string(),
            title: "PW1".to_string(),
            source_pathway_id: "pw1-01".to_string(),
            alos: vec![make_alo("pw1-01-01-c01", AloType::Concept, &["KSB-SHARED"])],
            edges: Vec::new(),
        };
        let pw2 = AtomizedPathway {
            id: "pw2-01".to_string(),
            title: "PW2".to_string(),
            source_pathway_id: "pw2-01".to_string(),
            alos: vec![make_alo("pw2-01-01-c01", AloType::Concept, &["KSB-SHARED"])],
            edges: Vec::new(),
        };

        let graph = build_graph(&[pw1, pw2], false, 0.75);
        assert!(graph.is_ok());
        let g = graph.as_ref().ok();
        assert!(g.is_some());
        assert_eq!(g.map(|g| g.overlap_clusters.len()), Some(1));
        assert_eq!(
            g.and_then(|g| g.overlap_clusters.first().map(|c| c.concept.as_str())),
            Some("KSB-SHARED")
        );
    }

    #[test]
    fn test_cycle_detection() {
        let pathway = AtomizedPathway {
            id: "cycle-01".to_string(),
            title: "Cyclic".to_string(),
            source_pathway_id: "cycle-01".to_string(),
            alos: vec![
                make_alo("cycle-01-01-c01", AloType::Concept, &[]),
                make_alo("cycle-01-01-c02", AloType::Concept, &[]),
            ],
            edges: vec![
                AloEdge {
                    from: "cycle-01-01-c01".to_string(),
                    to: "cycle-01-01-c02".to_string(),
                    edge_type: AloEdgeType::Prereq,
                    strength: 1.0,
                },
                AloEdge {
                    from: "cycle-01-01-c02".to_string(),
                    to: "cycle-01-01-c01".to_string(),
                    edge_type: AloEdgeType::Prereq,
                    strength: 1.0,
                },
            ],
        };

        let result = build_graph(&[pathway], false, 0.75);
        assert!(result.is_err());
    }

    #[test]
    fn test_jaccard_similarity() {
        let a = "Define the five axioms of vigilance theory";
        let b = "Define the five axioms of safety theory";
        let sim = jaccard_similarity(a, b);
        assert!(sim > 0.5, "Expected >0.5, got {sim}");
    }
}
