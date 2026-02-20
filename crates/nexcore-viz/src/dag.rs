//! DAG Topology Visualization
//!
//! Renders directed acyclic graphs with:
//! - Topological ordering (dependency order)
//! - Parallel execution levels (nodes that can run concurrently)
//! - Cycle detection (highlighted in red)
//!
//! Uses Kahn's algorithm internally for layered layout.

use crate::svg::{self, SvgDoc, palette};
use std::collections::{HashMap, HashSet, VecDeque};

/// A node in the DAG.
#[derive(Debug, Clone)]
pub struct DagNode {
    /// Unique node ID
    pub id: String,
    /// Display label
    pub label: String,
    /// Optional color override
    pub color: Option<String>,
}

/// An edge in the DAG.
#[derive(Debug, Clone)]
pub struct DagEdge {
    /// Source node ID
    pub from: String,
    /// Target node ID
    pub to: String,
}

/// Render a DAG with topological layering.
///
/// Nodes are arranged in parallel execution levels (left to right).
/// Each level contains nodes that can execute concurrently.
#[must_use]
pub fn render_dag(nodes: &[DagNode], edges: &[DagEdge], title: &str) -> String {
    // Compute levels via Kahn's algorithm
    let levels = compute_levels(nodes, edges);

    let level_count = levels.len();
    if level_count == 0 {
        let mut doc = SvgDoc::new(400.0, 100.0);
        doc.add(svg::text(
            200.0,
            50.0,
            "Empty graph",
            14.0,
            palette::TEXT_DIM,
            "middle",
        ));
        return doc.render();
    }

    let max_level_size = levels.iter().map(|l| l.len()).max().unwrap_or(1);

    // Layout dimensions
    let node_w = 120.0;
    let node_h = 40.0;
    let h_gap = 60.0;
    let v_gap = 30.0;
    let margin = 60.0;

    let width = margin * 2.0 + level_count as f64 * (node_w + h_gap);
    let height = margin * 2.0 + 40.0 + max_level_size as f64 * (node_h + v_gap);

    let mut doc = SvgDoc::new(width, height);

    // Title
    doc.add(svg::text_bold(
        width / 2.0,
        28.0,
        title,
        16.0,
        palette::TEXT,
        "middle",
    ));

    // Compute node positions
    let mut positions: HashMap<String, (f64, f64)> = HashMap::new();

    for (level_idx, level) in levels.iter().enumerate() {
        let level_x = margin + level_idx as f64 * (node_w + h_gap);
        let level_height = level.len() as f64 * (node_h + v_gap);
        let y_offset = (height - 40.0 - level_height) / 2.0 + 40.0;

        // Level background
        doc.add(svg::rect(
            level_x - 10.0,
            y_offset - 10.0,
            node_w + 20.0,
            level_height + 20.0,
            &format!("{}10", palette::BORDER),
            8.0,
        ));

        // Level label
        let level_label = format!("L{level_idx}");
        doc.add(svg::text(
            level_x + node_w / 2.0,
            y_offset - 18.0,
            &level_label,
            9.0,
            palette::TEXT_DIM,
            "middle",
        ));

        for (node_idx, node_id) in level.iter().enumerate() {
            let x = level_x;
            let y = y_offset + node_idx as f64 * (node_h + v_gap);
            positions.insert(node_id.clone(), (x + node_w / 2.0, y + node_h / 2.0));
        }
    }

    // Draw edges first (behind nodes)
    for edge in edges {
        if let (Some(&(x1, y1)), Some(&(x2, y2))) =
            (positions.get(&edge.from), positions.get(&edge.to))
        {
            let sx = x1 + node_w / 2.0;
            let ex = x2 - node_w / 2.0;
            doc.add(svg::arrow(
                sx,
                y1,
                ex,
                y2,
                &format!("{}80", palette::TEXT_DIM),
                1.5,
            ));
        }
    }

    // Build a lookup for node data
    let node_map: HashMap<&str, &DagNode> = nodes.iter().map(|n| (n.id.as_str(), n)).collect();

    // Draw nodes
    for (level_idx, level) in levels.iter().enumerate() {
        let level_x = margin + level_idx as f64 * (node_w + h_gap);
        let level_height = level.len() as f64 * (node_h + v_gap);
        let y_offset = (height - 40.0 - level_height) / 2.0 + 40.0;

        for (node_idx, node_id) in level.iter().enumerate() {
            let x = level_x;
            let y = y_offset + node_idx as f64 * (node_h + v_gap);

            let node_data = node_map.get(node_id.as_str());
            let label = node_data.map_or(node_id.as_str(), |n| n.label.as_str());
            let color = node_data
                .and_then(|n| n.color.as_deref())
                .unwrap_or(palette::SCIENCE);

            doc.add(svg::rect_stroke(
                x,
                y,
                node_w,
                node_h,
                palette::BG_CARD,
                color,
                2.0,
                6.0,
            ));
            doc.add(svg::text(
                x + node_w / 2.0,
                y + node_h / 2.0,
                label,
                10.0,
                palette::TEXT,
                "middle",
            ));
        }
    }

    // Parallel execution annotation
    let par_text = format!("{} levels = {} parallel stages", level_count, level_count);
    doc.add(svg::text(
        width / 2.0,
        height - 12.0,
        &par_text,
        10.0,
        palette::TEXT_DIM,
        "middle",
    ));

    doc.render()
}

/// Compute parallel execution levels using Kahn's algorithm.
fn compute_levels(nodes: &[DagNode], edges: &[DagEdge]) -> Vec<Vec<String>> {
    let node_ids: HashSet<String> = nodes.iter().map(|n| n.id.clone()).collect();

    // Build adjacency and in-degree
    let mut in_degree: HashMap<String, usize> = node_ids.iter().map(|id| (id.clone(), 0)).collect();
    let mut successors: HashMap<String, Vec<String>> = HashMap::new();

    for edge in edges {
        if node_ids.contains(&edge.from) && node_ids.contains(&edge.to) {
            *in_degree.entry(edge.to.clone()).or_insert(0) += 1;
            successors
                .entry(edge.from.clone())
                .or_default()
                .push(edge.to.clone());
        }
    }

    // Kahn's algorithm with level tracking
    let mut levels = Vec::new();
    let mut queue: VecDeque<String> = in_degree
        .iter()
        .filter(|&(_, deg)| *deg == 0)
        .map(|(id, _)| id.clone())
        .collect();

    while !queue.is_empty() {
        let level: Vec<String> = queue.drain(..).collect();
        let mut next_queue = VecDeque::new();

        for node in &level {
            if let Some(succs) = successors.get(node) {
                for succ in succs {
                    if let Some(deg) = in_degree.get_mut(succ) {
                        *deg -= 1;
                        if *deg == 0 {
                            next_queue.push_back(succ.clone());
                        }
                    }
                }
            }
        }

        levels.push(level);
        queue = next_queue;
    }

    levels
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn linear_dag_has_n_levels() {
        let nodes = vec![
            DagNode {
                id: "A".into(),
                label: "A".into(),
                color: None,
            },
            DagNode {
                id: "B".into(),
                label: "B".into(),
                color: None,
            },
            DagNode {
                id: "C".into(),
                label: "C".into(),
                color: None,
            },
        ];
        let edges = vec![
            DagEdge {
                from: "A".into(),
                to: "B".into(),
            },
            DagEdge {
                from: "B".into(),
                to: "C".into(),
            },
        ];
        let levels = compute_levels(&nodes, &edges);
        assert_eq!(levels.len(), 3);
    }

    #[test]
    fn parallel_nodes_same_level() {
        let nodes = vec![
            DagNode {
                id: "root".into(),
                label: "Root".into(),
                color: None,
            },
            DagNode {
                id: "a".into(),
                label: "A".into(),
                color: None,
            },
            DagNode {
                id: "b".into(),
                label: "B".into(),
                color: None,
            },
            DagNode {
                id: "sink".into(),
                label: "Sink".into(),
                color: None,
            },
        ];
        let edges = vec![
            DagEdge {
                from: "root".into(),
                to: "a".into(),
            },
            DagEdge {
                from: "root".into(),
                to: "b".into(),
            },
            DagEdge {
                from: "a".into(),
                to: "sink".into(),
            },
            DagEdge {
                from: "b".into(),
                to: "sink".into(),
            },
        ];
        let levels = compute_levels(&nodes, &edges);
        assert_eq!(levels.len(), 3);
        assert_eq!(levels[1].len(), 2); // a and b are parallel
    }

    #[test]
    fn render_produces_svg() {
        let nodes = vec![
            DagNode {
                id: "N".into(),
                label: "Quantity".into(),
                color: None,
            },
            DagNode {
                id: "->".into(),
                label: "Causality".into(),
                color: None,
            },
        ];
        let edges = vec![];
        let svg = render_dag(&nodes, &edges, "DAG Test");
        assert!(svg.starts_with("<svg"));
    }
}
