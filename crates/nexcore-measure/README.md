# nexcore-measure

Workspace quality measurement engine for the NexVigilant Core kernel. It provides quantitative metrics for crate health, information entropy, dependency graph complexity, and statistical drift, all grounded in the Lex Primitiva foundation.

## Intent
To provide an objective, data-driven assessment of the workspace's structural and semantic integrity. It enables AI agents to detect "technical debt" or "structural decay" before it affects system safety or performance.

## T1 Grounding (Lex Primitiva)
Dominant Primitives:
- **N (Quantity)**: The primary primitive for all scalar measurements (LOC, complexity, entropy).
- **∂ (Boundary)**: Defines the "Healthy" vs "Unhealthy" zones for various metrics.
- **μ (Mapping)**: Maps raw source code counts to information-theoretic scores.
- **ρ (Recursion)**: Used for deep dependency graph analysis and cycle detection.
- **Σ (Sum)**: Root primitive for workspace-level aggregate health scores.

## Core Metrics
| Metric | Basis | Purpose |
| :--- | :--- | :--- |
| **Entropy** | Information Theory | Measures semantic density and predictability of code. |
| **Centrality** | Graph Theory | Identifies "hub" crates with high blast radius. |
| **Drift** | Statistics | Detects significant shifts in metrics over time (Welch t-test). |
| **Health Score** | Composite | 0-10 rating of a crate's adherence to Gold Standards. |

## SOPs for Use
### Measuring a Crate
```rust
use nexcore_measure::collect::measure_crate;
use std::path::Path;

let measurement = measure_crate(Path::new("crates/nexcore-vigilance"))?;
println!("Health Score: {}", measurement.health_score());
```

### Analyzing the Graph
```rust
use nexcore_measure::graph::analyze_workspace_graph;

let analysis = analyze_workspace_graph(Path::new("."))?;
for node in analysis.nodes {
    println!("Crate: {}, Centrality: {}", node.id, node.centrality);
}
```

## Key Components
- **CrateMeasurement**: Snapshot of a single crate's metrics.
- **WorkspaceHealth**: Aggregated health view across all members.
- **DriftResult**: Statistical significance of metric changes.

## License
Proprietary. Copyright (c) 2026 NexVigilant LLC. All Rights Reserved.
