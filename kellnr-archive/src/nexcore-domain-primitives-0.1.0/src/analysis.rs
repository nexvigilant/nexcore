//! Topological sort, critical path, and bottleneck analysis.

use std::collections::{HashMap, HashSet, VecDeque};

use serde::{Deserialize, Serialize};

use crate::taxonomy::{DomainTaxonomy, Tier};

/// Topological ordering of primitives (dependencies before dependents).
pub fn topological_sort(tax: &DomainTaxonomy) -> Result<Vec<&str>, CycleError> {
    let graph = tax.dependency_graph();
    let mut in_degree: HashMap<&str, usize> = HashMap::new();

    // Initialize all nodes
    for p in &tax.primitives {
        in_degree.entry(p.name.as_str()).or_insert(0);
        for dep in &p.dependencies {
            *in_degree.entry(dep.as_str()).or_insert(0) += 0;
        }
    }

    // Count incoming edges
    for p in &tax.primitives {
        for dep in &p.dependencies {
            if let Some(entry) = in_degree.get_mut(p.name.as_str()) {
                // p depends on dep, so dep → p is an edge; p gets +1 in-degree? No.
                // Wait — in our graph, p.dependencies lists what p depends ON.
                // So dep → p in the "must come before" sense.
                // in_degree should count how many things p depends on.
                let _ = entry;
            }
        }
    }

    // Recalculate: in-degree = number of dependencies for each node
    let mut in_deg: HashMap<&str, usize> = HashMap::new();
    for p in &tax.primitives {
        in_deg.insert(p.name.as_str(), p.dependencies.len());
    }

    let mut queue: VecDeque<&str> = VecDeque::new();
    for (&name, &deg) in &in_deg {
        if deg == 0 {
            queue.push_back(name);
        }
    }

    let rev = tax.dependents_graph();
    let mut sorted = Vec::new();

    while let Some(node) = queue.pop_front() {
        sorted.push(node);
        if let Some(dependents) = rev.get(node) {
            for &dependent in dependents {
                if let Some(deg) = in_deg.get_mut(dependent) {
                    *deg = deg.saturating_sub(1);
                    if *deg == 0 {
                        queue.push_back(dependent);
                    }
                }
            }
        }
    }

    if sorted.len() == tax.primitives.len() {
        Ok(sorted)
    } else {
        Err(CycleError {
            sorted_count: sorted.len(),
            total: tax.primitives.len(),
        })
    }
}

/// Error when a cycle is detected in the dependency graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CycleError {
    pub sorted_count: usize,
    pub total: usize,
}

impl std::fmt::Display for CycleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Cycle detected: sorted {}/{} primitives",
            self.sorted_count, self.total
        )
    }
}

/// Find all root-to-leaf paths (critical paths) in the taxonomy.
/// Returns paths sorted longest-first.
pub fn critical_paths(tax: &DomainTaxonomy) -> Vec<Vec<&str>> {
    let rev = tax.dependents_graph();

    // Roots: nodes with no dependencies (T1 typically)
    let roots: Vec<&str> = tax
        .primitives
        .iter()
        .filter(|p| p.dependencies.is_empty())
        .map(|p| p.name.as_str())
        .collect();

    // Leaves: nodes that nothing depends on
    let has_dependents: HashSet<&str> = rev
        .iter()
        .filter(|(_, deps)| !deps.is_empty())
        .map(|(&name, _)| name)
        .collect();
    let all_names: HashSet<&str> = tax.primitives.iter().map(|p| p.name.as_str()).collect();
    let leaves: HashSet<&str> = all_names.difference(&has_dependents).copied().collect();

    let mut all_paths: Vec<Vec<&str>> = Vec::new();

    // DFS from each root
    for &root in &roots {
        let mut stack: Vec<(Vec<&str>,)> = vec![(vec![root],)];
        while let Some((path,)) = stack.pop() {
            let current = path[path.len() - 1];
            let children = rev.get(current).cloned().unwrap_or_default();

            if children.is_empty() || leaves.contains(current) && path.len() > 1 {
                all_paths.push(path.clone());
            }

            for &child in &children {
                if !path.contains(&child) {
                    // Avoid cycles
                    let mut new_path = path.clone();
                    new_path.push(child);
                    stack.push((new_path,));
                }
            }
        }
    }

    // Sort longest first
    all_paths.sort_by(|a, b| b.len().cmp(&a.len()));
    all_paths
}

/// Bottleneck analysis: primitives ranked by fan-out (most dependents first).
/// These are the primitives that, if understood, unlock the most downstream knowledge.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bottleneck {
    pub name: String,
    pub tier: Tier,
    pub fan_out: usize,
    /// What percentage of all primitives depend (directly or transitively) on this.
    pub reach_pct: f64,
}

/// Find bottleneck primitives ordered by fan-out.
pub fn bottlenecks(tax: &DomainTaxonomy) -> Vec<Bottleneck> {
    let rev = tax.dependents_graph();
    let total = tax.primitives.len() as f64;

    let mut results: Vec<Bottleneck> = tax
        .primitives
        .iter()
        .map(|p| {
            // Compute transitive reach via BFS
            let mut visited = HashSet::new();
            let mut queue = VecDeque::new();
            queue.push_back(p.name.as_str());

            while let Some(node) = queue.pop_front() {
                if let Some(deps) = rev.get(node) {
                    for &dep in deps {
                        if visited.insert(dep) {
                            queue.push_back(dep);
                        }
                    }
                }
            }

            Bottleneck {
                name: p.name.clone(),
                tier: p.tier,
                fan_out: visited.len(),
                reach_pct: if total > 0.0 {
                    visited.len() as f64 / total * 100.0
                } else {
                    0.0
                },
            }
        })
        .collect();

    results.sort_by(|a, b| b.fan_out.cmp(&a.fan_out));
    results
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::golden_dome::golden_dome;

    #[test]
    fn topological_sort_succeeds() {
        let tax = golden_dome();
        let sorted = topological_sort(&tax);
        assert!(sorted.is_ok());
        let sorted = sorted.unwrap_or_default();
        assert_eq!(sorted.len(), 30);

        // T1 should appear before T2-P, T2-P before T2-C, etc.
        let pos = |name: &str| sorted.iter().position(|&n| n == name);
        let det = pos("detection").unwrap_or(0);
        let sf = pos("sensor-fusion").unwrap_or(0);
        let md = pos("midcourse-discrimination").unwrap_or(0);
        assert!(det < sf, "detection should precede sensor-fusion");
        assert!(
            sf < md,
            "sensor-fusion should precede midcourse-discrimination"
        );
    }

    #[test]
    fn critical_paths_found() {
        let tax = golden_dome();
        let paths = critical_paths(&tax);
        assert!(!paths.is_empty());
        // Longest path should be at least 3 hops (T1 → T2-P → T2-C → T3)
        assert!(paths[0].len() >= 3);
    }

    #[test]
    fn critical_paths_start_at_t1() {
        let tax = golden_dome();
        let paths = critical_paths(&tax);
        for path in &paths {
            let first = path[0];
            let prim = tax.get(first);
            assert!(prim.is_some());
            let prim = prim.unwrap_or_else(|| &tax.primitives[0]);
            assert_eq!(prim.tier, Tier::T1, "Path should start at T1: {first}");
        }
    }

    #[test]
    fn bottleneck_detection_high() {
        let tax = golden_dome();
        let bn = bottlenecks(&tax);
        assert!(!bn.is_empty());

        // detection/tracking/threshold should be among top bottlenecks (many dependents)
        let top_5: Vec<&str> = bn.iter().take(5).map(|b| b.name.as_str()).collect();
        // At least one T1 in top 5
        let has_t1 = bn.iter().take(5).any(|b| b.tier == Tier::T1);
        assert!(has_t1, "T1 primitives should dominate bottleneck ranking");
    }

    #[test]
    fn t3_lower_fanout_than_t1() {
        let tax = golden_dome();
        let bn = bottlenecks(&tax);
        // T3 can depend on other T3 (e.g., NGI → midcourse-discrimination)
        // but T3 avg fan-out should be strictly less than T1 avg fan-out
        let t1_avg = avg_fanout(&bn, Tier::T1);
        let t3_avg = avg_fanout(&bn, Tier::T3);
        assert!(
            t3_avg < t1_avg,
            "T3 avg fan-out ({t3_avg:.1}) should be less than T1 ({t1_avg:.1})"
        );
    }

    fn avg_fanout(bn: &[Bottleneck], tier: Tier) -> f64 {
        let (sum, count) = bn
            .iter()
            .filter(|b| b.tier == tier)
            .fold((0usize, 0usize), |(s, c), b| (s + b.fan_out, c + 1));
        if count > 0 {
            sum as f64 / count as f64
        } else {
            0.0
        }
    }

    #[test]
    fn bottleneck_reach_bounded() {
        let tax = golden_dome();
        let bn = bottlenecks(&tax);
        for b in &bn {
            assert!(
                b.reach_pct <= 100.0,
                "Reach should be <= 100%: {} at {}%",
                b.name,
                b.reach_pct
            );
        }
    }
}
