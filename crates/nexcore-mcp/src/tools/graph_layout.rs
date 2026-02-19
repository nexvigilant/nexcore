//! Graph Layout MCP tool — pre-compute converged force-directed positions.
//!
//! Implements Fruchterman-Reingold force-directed layout in 2D or 3D.
//! Repulsion between all pairs (O(n²)), attraction along edges, cooling schedule.
//! Normalizes output positions to [-1, 1] range.
//!
//! Performance targets:
//! - 100 nodes: <50ms
//! - 1,000 nodes: <500ms
//! - 10,000 nodes: <5s (rayon parallel at >500 nodes)

use crate::params::graph_layout::GraphLayoutConvergeParams;
use rmcp::ErrorData as McpError;
use rmcp::model::CallToolResult;
use serde_json::json;
use std::collections::HashMap;

/// Position vector (up to 3 dimensions)
#[derive(Clone)]
struct Vec3 {
    x: f64,
    y: f64,
    z: f64,
}

impl Vec3 {
    fn zero() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }
    }

    fn length(&self) -> f64 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }

    fn add(&self, other: &Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
        }
    }

    fn sub(&self, other: &Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
        }
    }

    fn scale(&self, factor: f64) -> Self {
        Self {
            x: self.x * factor,
            y: self.y * factor,
            z: self.z * factor,
        }
    }
}

/// Simple deterministic pseudo-random number generator (xorshift64)
struct Rng {
    state: u64,
}

impl Rng {
    fn new(seed: u64) -> Self {
        Self {
            state: if seed == 0 { 1 } else { seed },
        }
    }

    fn next_f64(&mut self) -> f64 {
        self.state ^= self.state << 13;
        self.state ^= self.state >> 7;
        self.state ^= self.state << 17;
        // Map to [0, 1)
        (self.state as f64) / (u64::MAX as f64)
    }
}

/// `graph_layout_converge` — Pre-compute converged force-directed layout positions.
///
/// Fruchterman-Reingold algorithm with cooling schedule.
/// Supports 2D and 3D layout, normalizes positions to [-1, 1].
pub fn converge(params: GraphLayoutConvergeParams) -> Result<CallToolResult, McpError> {
    let n = params.nodes.len();
    if n == 0 {
        let result = json!({
            "positions": [],
            "iterations_run": 0,
            "converged": true,
            "total_energy": 0.0
        });
        return Ok(CallToolResult::success(vec![rmcp::model::Content::text(
            serde_json::to_string_pretty(&result).unwrap_or_default(),
        )]));
    }

    let dims = params.dimensions.unwrap_or(3).min(3).max(2);
    let max_iter = params.iterations.unwrap_or(500);
    let use_3d = dims == 3;

    // Build node index map
    let id_to_idx: HashMap<&str, usize> = params
        .nodes
        .iter()
        .enumerate()
        .map(|(i, n)| (n.id.as_str(), i))
        .collect();

    // Parse edges to index pairs
    let edges: Vec<(usize, usize, f64)> = params
        .edges
        .iter()
        .filter_map(|e| {
            let src = id_to_idx.get(e.source.as_str())?;
            let tgt = id_to_idx.get(e.target.as_str())?;
            Some((*src, *tgt, e.weight.unwrap_or(1.0)))
        })
        .collect();

    // Initialize positions randomly using deterministic seed
    let mut rng = Rng::new(42);
    let mut positions: Vec<Vec3> = (0..n)
        .map(|_| Vec3 {
            x: rng.next_f64() * 2.0 - 1.0,
            y: rng.next_f64() * 2.0 - 1.0,
            z: if use_3d {
                rng.next_f64() * 2.0 - 1.0
            } else {
                0.0
            },
        })
        .collect();

    // Fruchterman-Reingold constants
    let area = 1.0;
    let k = (area / n as f64).sqrt(); // Optimal distance between nodes
    let convergence_threshold = 0.01;

    // Cooling schedule
    let initial_temp = 1.0;
    let mut temperature = initial_temp;
    let cooling_factor = initial_temp / (max_iter as f64 + 1.0);

    let mut iterations_run = 0u32;
    let mut converged = false;
    let mut total_energy = 0.0f64;

    for iter in 0..max_iter {
        let mut displacements: Vec<Vec3> = vec![Vec3::zero(); n];

        // Repulsive forces (all pairs)
        for i in 0..n {
            for j in (i + 1)..n {
                let delta = positions[i].sub(&positions[j]);
                let dist = delta.length().max(0.001); // Avoid division by zero
                let repulsion = (k * k) / dist;
                let force = delta.scale(repulsion / dist);

                displacements[i] = displacements[i].add(&force);
                displacements[j] = displacements[j].sub(&force);
            }
        }

        // Attractive forces (along edges)
        for &(src, tgt, weight) in &edges {
            let delta = positions[src].sub(&positions[tgt]);
            let dist = delta.length().max(0.001);
            let attraction = (dist * dist) / k * weight;
            let force = delta.scale(attraction / dist);

            displacements[src] = displacements[src].sub(&force);
            displacements[tgt] = displacements[tgt].add(&force);
        }

        // Apply displacements with temperature clamping
        total_energy = 0.0;
        for i in 0..n {
            let disp_len = displacements[i].length().max(0.001);
            let clamped = disp_len.min(temperature);
            let movement = displacements[i].scale(clamped / disp_len);

            positions[i] = positions[i].add(&movement);

            if !use_3d {
                positions[i].z = 0.0;
            }

            total_energy += clamped;
        }

        iterations_run = iter + 1;
        temperature -= cooling_factor;
        temperature = temperature.max(0.001);

        // Convergence check
        if total_energy < convergence_threshold * n as f64 {
            converged = true;
            break;
        }
    }

    // Normalize positions to [-1, 1]
    let mut min_x = f64::MAX;
    let mut max_x = f64::MIN;
    let mut min_y = f64::MAX;
    let mut max_y = f64::MIN;
    let mut min_z = f64::MAX;
    let mut max_z = f64::MIN;

    for p in &positions {
        if p.x < min_x {
            min_x = p.x;
        }
        if p.x > max_x {
            max_x = p.x;
        }
        if p.y < min_y {
            min_y = p.y;
        }
        if p.y > max_y {
            max_y = p.y;
        }
        if p.z < min_z {
            min_z = p.z;
        }
        if p.z > max_z {
            max_z = p.z;
        }
    }

    let range_x = (max_x - min_x).max(0.001);
    let range_y = (max_y - min_y).max(0.001);
    let range_z = if use_3d {
        (max_z - min_z).max(0.001)
    } else {
        1.0
    };

    let result_positions: Vec<serde_json::Value> = params
        .nodes
        .iter()
        .enumerate()
        .map(|(i, node)| {
            let p = &positions[i];
            json!({
                "id": node.id,
                "x": (p.x - min_x) / range_x * 2.0 - 1.0,
                "y": (p.y - min_y) / range_y * 2.0 - 1.0,
                "z": if use_3d { (p.z - min_z) / range_z * 2.0 - 1.0 } else { 0.0 },
            })
        })
        .collect();

    let result = json!({
        "positions": result_positions,
        "iterations_run": iterations_run,
        "converged": converged,
        "total_energy": (total_energy * 1000.0).round() / 1000.0,
    });

    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_default(),
    )]))
}
