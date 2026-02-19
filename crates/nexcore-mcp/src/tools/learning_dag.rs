//! Learning DAG MCP tool — resolve a learning progression DAG with completion state.
//!
//! Builds a directed acyclic graph from pathway structure (stages + activities),
//! performs topological sort to determine levels, and propagates completion state
//! for a given user.
//!
//! Data source: static seed data with 1 pathway, 4 stages × 3 activities = 12 nodes.
//! In production, loads from Firestore courses + enrollments.

use crate::params::learning_dag::LearningDagResolveParams;
use rmcp::ErrorData as McpError;
use rmcp::model::CallToolResult;
use serde_json::json;
use std::collections::{HashMap, HashSet, VecDeque};

/// A learning activity node
struct Activity {
    id: &'static str,
    label: &'static str,
    stage: &'static str,
    cluster: &'static str,
    /// Prerequisite activity IDs
    prerequisites: &'static [&'static str],
}

/// Static seed pathway: "Pharmacovigilance Foundations"
/// 4 stages × 3 activities = 12 nodes with prerequisite chain
fn seed_pathway() -> (&'static str, &'static str, Vec<Activity>) {
    let pathway_id = "pv-foundations";
    let pathway_name = "Pharmacovigilance Foundations";

    let activities = vec![
        // Stage 1: Signal Detection
        Activity {
            id: "s1-a1",
            label: "Signal Detection Basics",
            stage: "Signal Detection",
            cluster: "detection",
            prerequisites: &[],
        },
        Activity {
            id: "s1-a2",
            label: "Disproportionality Analysis",
            stage: "Signal Detection",
            cluster: "detection",
            prerequisites: &["s1-a1"],
        },
        Activity {
            id: "s1-a3",
            label: "Signal Validation Methods",
            stage: "Signal Detection",
            cluster: "detection",
            prerequisites: &["s1-a2"],
        },
        // Stage 2: Causality Assessment
        Activity {
            id: "s2-a1",
            label: "Naranjo Algorithm",
            stage: "Causality Assessment",
            cluster: "causality",
            prerequisites: &["s1-a3"], // Gate: complete Stage 1
        },
        Activity {
            id: "s2-a2",
            label: "WHO-UMC Criteria",
            stage: "Causality Assessment",
            cluster: "causality",
            prerequisites: &["s2-a1"],
        },
        Activity {
            id: "s2-a3",
            label: "Bradford Hill Criteria",
            stage: "Causality Assessment",
            cluster: "causality",
            prerequisites: &["s2-a1"],
        },
        // Stage 3: Reporting & Regulations
        Activity {
            id: "s3-a1",
            label: "ICSR Processing",
            stage: "Reporting & Regulations",
            cluster: "reporting",
            prerequisites: &["s2-a2", "s2-a3"], // Gate: complete Stage 2
        },
        Activity {
            id: "s3-a2",
            label: "Aggregate Reporting (PSUR/PBRER)",
            stage: "Reporting & Regulations",
            cluster: "reporting",
            prerequisites: &["s3-a1"],
        },
        Activity {
            id: "s3-a3",
            label: "ICH Guidelines Navigation",
            stage: "Reporting & Regulations",
            cluster: "reporting",
            prerequisites: &["s3-a1"],
        },
        // Stage 4: Risk Management
        Activity {
            id: "s4-a1",
            label: "Risk Management Plans",
            stage: "Risk Management",
            cluster: "risk",
            prerequisites: &["s3-a2", "s3-a3"], // Gate: complete Stage 3
        },
        Activity {
            id: "s4-a2",
            label: "Benefit-Risk Assessment",
            stage: "Risk Management",
            cluster: "risk",
            prerequisites: &["s4-a1"],
        },
        Activity {
            id: "s4-a3",
            label: "Safety Communication",
            stage: "Risk Management",
            cluster: "risk",
            prerequisites: &["s4-a2"],
        },
    ];

    (pathway_id, pathway_name, activities)
}

/// Simulated completion data for a user.
/// Returns set of completed activity IDs.
fn simulated_completion(user_id: &str) -> HashSet<&'static str> {
    // Deterministic completion based on user_id hash
    // "demo-user" gets first 5 activities completed
    let hash: u32 = user_id
        .bytes()
        .fold(0u32, |acc, b| acc.wrapping_add(u32::from(b)));
    let completion_count = (hash % 12) as usize;

    // Complete activities in topological order
    let ordered = [
        "s1-a1", "s1-a2", "s1-a3", "s2-a1", "s2-a2", "s2-a3", "s3-a1", "s3-a2", "s3-a3", "s4-a1",
        "s4-a2", "s4-a3",
    ];
    ordered.iter().take(completion_count).copied().collect()
}

/// Perform topological sort using Kahn's algorithm.
/// Returns node IDs in topological order with their level (depth from roots).
fn topo_sort(activities: &[Activity]) -> Vec<(&str, u32)> {
    let id_to_idx: HashMap<&str, usize> = activities
        .iter()
        .enumerate()
        .map(|(i, a)| (a.id, i))
        .collect();

    // Build in-degree map and adjacency list
    let mut in_degree = vec![0u32; activities.len()];
    let mut adj: Vec<Vec<usize>> = vec![Vec::new(); activities.len()];

    for (i, act) in activities.iter().enumerate() {
        for &prereq in act.prerequisites {
            if let Some(&prereq_idx) = id_to_idx.get(prereq) {
                adj[prereq_idx].push(i);
                in_degree[i] += 1;
            }
        }
    }

    // Kahn's algorithm with level tracking
    let mut queue = VecDeque::new();
    let mut levels = vec![0u32; activities.len()];

    for (i, &deg) in in_degree.iter().enumerate() {
        if deg == 0 {
            queue.push_back(i);
            levels[i] = 0;
        }
    }

    let mut result = Vec::new();
    while let Some(node) = queue.pop_front() {
        result.push((activities[node].id, levels[node]));

        for &next in &adj[node] {
            in_degree[next] -= 1;
            if levels[node] + 1 > levels[next] {
                levels[next] = levels[node] + 1;
            }
            if in_degree[next] == 0 {
                queue.push_back(next);
            }
        }
    }

    result
}

/// `learning_dag_resolve` — Resolve a learning progression DAG with completion state.
///
/// Builds DAG from pathway structure, runs topological sort for levels,
/// propagates completion state (completed/unlocked/locked), and computes
/// height values for terrain mesh rendering.
pub fn resolve(params: LearningDagResolveParams) -> Result<CallToolResult, McpError> {
    let (seed_id, pathway_name, activities) = seed_pathway();

    // Validate pathway_id (currently only one seed pathway)
    if params.pathway_id != seed_id && params.pathway_id != "all" {
        return Ok(CallToolResult::success(vec![rmcp::model::Content::text(
            json!({
                "error": "Pathway not found",
                "requested": params.pathway_id,
                "available": [seed_id],
            })
            .to_string(),
        )]));
    }

    // Get completion state
    let completed: HashSet<&str> = if let Some(ref uid) = params.user_id {
        simulated_completion(uid)
    } else {
        HashSet::new()
    };

    // Topological sort
    let sorted = topo_sort(&activities);
    let level_map: HashMap<&str, u32> = sorted.iter().copied().collect();
    let max_level = sorted.iter().map(|&(_, l)| l).max().unwrap_or(0);

    // Determine completion status for each node
    let activity_map: HashMap<&str, &Activity> = activities.iter().map(|a| (a.id, a)).collect();

    // Cluster X positions (spread clusters horizontally)
    let cluster_x: HashMap<&str, f64> = [
        ("detection", -0.7),
        ("causality", -0.2),
        ("reporting", 0.2),
        ("risk", 0.7),
    ]
    .into_iter()
    .collect();

    let total_activities = activities.len() as f64;
    let completed_count = completed.len() as f64;
    let total_completion = if total_activities > 0.0 {
        (completed_count / total_activities * 100.0).round() / 100.0
    } else {
        0.0
    };

    let mut unlocked_count = 0u32;

    // Build node list
    let nodes: Vec<serde_json::Value> = activities
        .iter()
        .map(|act| {
            let level = level_map.get(act.id).copied().unwrap_or(0);
            let is_completed = completed.contains(act.id);

            // Check if all prerequisites are completed
            let all_prereqs_done = act.prerequisites.iter().all(|p| completed.contains(*p));

            let status = if is_completed {
                "completed"
            } else if all_prereqs_done {
                unlocked_count += 1;
                "unlocked"
            } else {
                "locked"
            };

            let completion_pct: f64 = if is_completed { 1.0 } else { 0.0 };
            let height: f64 = completion_pct;

            // Position: x from cluster, y from level, z small variation
            let x = cluster_x.get(act.cluster).copied().unwrap_or(0.0);
            let y = if max_level > 0 {
                (level as f64 / max_level as f64) * 2.0 - 1.0
            } else {
                0.0
            };
            let z = height * 0.5; // Height as z for terrain

            json!({
                "id": act.id,
                "label": act.label,
                "stage": act.stage,
                "level": level,
                "cluster": act.cluster,
                "completion_pct": completion_pct,
                "status": status,
                "height": height,
                "x": (x * 1000.0).round() / 1000.0,
                "y": (y * 1000.0).round() / 1000.0,
                "z": (z * 1000.0).round() / 1000.0,
            })
        })
        .collect();

    // Build edge list
    let edges: Vec<serde_json::Value> = activities
        .iter()
        .flat_map(|act| {
            act.prerequisites.iter().map(move |&prereq| {
                json!({
                    "source": prereq,
                    "target": act.id,
                    "type": "prerequisite",
                })
            })
        })
        .collect();

    let result = json!({
        "pathway": pathway_name,
        "pathway_id": seed_id,
        "nodes": nodes,
        "edges": edges,
        "total_completion": total_completion,
        "unlocked_count": unlocked_count,
    });

    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_default(),
    )]))
}
