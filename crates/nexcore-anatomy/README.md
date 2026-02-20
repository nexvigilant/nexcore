# nexcore-anatomy

Structural anatomy analysis for Rust workspaces. This crate provides deep insights into the dependency graph, classifying members into logical layers, detecting boundary violations, and computing criticality scores to identify architectural bottlenecks.

## Intent
To enforce the **Layered Architecture** of the NexCore workspace. It ensures that the "one-way" dependency flow (Service → Orchestration → Domain → Foundation) is strictly maintained and that no circular dependencies or "skip-layer" violations occur.

## T1 Grounding (Lex Primitiva)
Dominant Primitives:
- **σ (Sequence)**: Manages the topological ordering and layer hierarchy of the workspace.
- **∂ (Boundary)**: The primary primitive for detecting layer violations and enforcing isolation.
- **ρ (Recursion)**: Used for recursive graph traversal and cycle detection (Kahn's algorithm).
- **μ (Mapping)**: Maps crates to their appropriate `Layer`, `ChomskyLevel`, and `CriticalityScore`.
- **N (Quantity)**: Computes fan-in/out counts and numeric health metrics.

## Architectural Layers
| Layer | Description | Rules |
| :--- | :--- | :--- |
| **Service** | External APIs, CLIs | May depend on all layers below. |
| **Orchestration**| Schedulers, Event Bus | May depend on Domain and Foundation. |
| **Domain** | Business Logic, PV | May depend on Foundation only. |
| **Foundation** | Primitives, Macros | Leaf level: No internal workspace deps. |

## SOPs for Use
### Running an Anatomy Report
```rust
use nexcore_anatomy::DependencyGraph;
use nexcore_anatomy::report::AnatomyReport;

let graph = DependencyGraph::from_workspace_root(".")?;
let report = AnatomyReport::from_graph(graph);

if report.summary.violation_count > 0 {
    println!("Architectural violations detected!");
}
```

### Analyzing Blast Radius
Use the `BlastRadius` module to determine how many crates are affected if a specific leaf crate is modified.

## Key Components
- **DependencyGraph**: The recursive representation of all crate relationships.
- **LayerMap**: Logic for assigning crates to the 4-layer hierarchy.
- **ChomskyAnalyzer**: Classifies code complexity based on the generators used.
- **CriticalityScore**: Identifies "single points of failure" in the graph.

## License
Proprietary. Copyright (c) 2026 NexVigilant LLC. All Rights Reserved.
