//! Integration test: runs nexcore-anatomy against the live nexcore workspace.
//!
//! This test uses `cargo_metadata::MetadataCommand` to analyze the real
//! workspace dependency graph and validates structural invariants.

use nexcore_anatomy::{
    AnatomyReport, BlastRadius, BlastRadiusReport, ChomskyLevel, ChomskyReport, DependencyGraph,
    HealthStatus, Layer, LayerMap, WorkspaceMetrics,
};

fn load_workspace_graph() -> DependencyGraph {
    let metadata = cargo_metadata::MetadataCommand::new()
        .manifest_path(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .parent()
                .and_then(|p| p.parent())
                .map(|p| p.join("Cargo.toml"))
                .unwrap_or_default(),
        )
        .exec();

    match metadata {
        Ok(m) => DependencyGraph::from_metadata(&m),
        Err(e) => {
            eprintln!("Skipping live workspace test: {e}");
            // Return empty graph — tests will check for this
            DependencyGraph {
                nodes: std::collections::HashMap::new(),
                cycles: vec![],
                topo_order: vec![],
                total_crates: 0,
            }
        }
    }
}

#[test]
fn test_workspace_has_crates() {
    let graph = load_workspace_graph();
    // NexCore workspace should have 80+ crates
    assert!(
        graph.total_crates > 50,
        "Expected 50+ workspace crates, found {}",
        graph.total_crates
    );
}

#[test]
fn test_workspace_no_cycles() {
    let graph = load_workspace_graph();
    if graph.total_crates == 0 {
        return;
    }
    assert!(
        !graph.has_cycles(),
        "Workspace has dependency cycles: {:?}",
        graph.cycles
    );
}

#[test]
fn test_workspace_topo_order_complete() {
    let graph = load_workspace_graph();
    if graph.total_crates == 0 {
        return;
    }
    assert_eq!(
        graph.topo_order.len(),
        graph.total_crates,
        "Topo order should include all crates (missing = cycles)"
    );
}

#[test]
fn test_lex_primitiva_is_bottleneck() {
    let graph = load_workspace_graph();
    if graph.total_crates == 0 {
        return;
    }

    // nexcore-lex-primitiva should be in the top 5 by fan-in
    let top5 = graph.top_by_fan_in(5);
    let names: Vec<&str> = top5.iter().map(|n| n.name.as_str()).collect();
    assert!(
        names.contains(&"nexcore-lex-primitiva"),
        "nexcore-lex-primitiva should be top-5 by fan-in, got: {names:?}"
    );
}

#[test]
fn test_lex_primitiva_blast_radius() {
    let graph = load_workspace_graph();
    if graph.total_crates == 0 {
        return;
    }

    let radius = BlastRadius::for_crate(&graph, "nexcore-lex-primitiva");
    assert!(
        radius.is_some(),
        "nexcore-lex-primitiva should exist in graph"
    );

    let r = radius.unwrap_or_else(|| unreachable!());

    // nexcore-lex-primitiva should affect > 20% of workspace (currently ~26%)
    assert!(
        r.impact_ratio > 0.20,
        "nexcore-lex-primitiva blast radius should exceed 20%, got {:.1}%",
        r.impact_ratio * 100.0
    );

    // Should have cascade depth >= 2 (direct deps → their deps → ...)
    assert!(
        r.cascade_depth >= 2,
        "Expected cascade depth >= 2, got {}",
        r.cascade_depth
    );

    eprintln!("--- nexcore-lex-primitiva Blast Radius ---");
    eprintln!("Direct dependents: {}", r.direct_count);
    eprintln!("Transitive dependents: {}", r.transitive_count);
    eprintln!("Impact ratio: {:.1}%", r.impact_ratio * 100.0);
    eprintln!("Cascade depth: {}", r.cascade_depth);
    eprintln!("Cascade width per level: {:?}", r.cascade_width);
    eprintln!("Containment: {:.1}%", r.containment * 100.0);
}

#[test]
fn test_blast_radius_report() {
    let graph = load_workspace_graph();
    if graph.total_crates == 0 {
        return;
    }

    let report = BlastRadiusReport::from_graph(&graph);

    // Should have bottlenecks
    assert!(
        !report.bottlenecks.is_empty(),
        "Workspace should have at least one bottleneck crate"
    );

    // Worst case should be identifiable
    assert!(
        !report.worst_case_crate.is_empty(),
        "Should identify worst-case crate"
    );

    eprintln!("--- Blast Radius Report ---");
    eprintln!("Total crates: {}", report.total_crates);
    eprintln!(
        "Worst case: {} ({} transitive deps, {:.1}%)",
        report.worst_case_crate,
        report.worst_case_count,
        report.worst_case_ratio * 100.0
    );
    eprintln!("Bottlenecks (>10%): {:?}", report.bottlenecks);
    eprintln!("Average impact: {:.1}%", report.avg_impact * 100.0);

    // Print top 10
    eprintln!("\n--- Top 10 by Blast Radius ---");
    for r in report.top(10) {
        eprintln!(
            "  {:<35} direct={:<3} transitive={:<3} impact={:.1}% depth={}",
            r.crate_name,
            r.direct_count,
            r.transitive_count,
            r.impact_ratio * 100.0,
            r.cascade_depth
        );
    }
}

#[test]
fn test_layer_classification() {
    let graph = load_workspace_graph();
    if graph.total_crates == 0 {
        return;
    }

    let layers = LayerMap::from_graph(&graph);

    // Known service crates should be classified correctly
    if let Some(mcp_layer) = layers.assignments.get("nexcore-mcp") {
        assert_eq!(
            *mcp_layer,
            Layer::Service,
            "nexcore-mcp should be Service layer"
        );
    }

    if let Some(primitives_layer) = layers.assignments.get("nexcore-primitives") {
        assert_eq!(
            *primitives_layer,
            Layer::Foundation,
            "nexcore-primitives should be Foundation layer"
        );
    }

    eprintln!("--- Layer Classification ---");
    eprintln!("Violations: {}", layers.violations.len());
    for v in &layers.violations {
        eprintln!(
            "  VIOLATION: {} ({:?}) → {} ({:?}) severity={}",
            v.from_crate, v.from_layer, v.to_crate, v.to_layer, v.severity
        );
    }
    eprintln!("Layer counts: {:?}", layers.layer_counts);
}

#[test]
fn test_criticality_distribution() {
    let graph = load_workspace_graph();
    if graph.total_crates == 0 {
        return;
    }

    let metrics = WorkspaceMetrics::from_graph(&graph);

    eprintln!("--- Criticality Distribution ---");
    eprintln!(
        "Bottleneck: {} (fan-in: {})",
        metrics.bottleneck_crate, metrics.max_fan_in
    );
    eprintln!("Avg instability: {:.3}", metrics.avg_instability);
    eprintln!("Graph density: {:.4}", metrics.graph_density);
    eprintln!("Total edges: {}", metrics.total_edges);

    let tiers = [
        ("Critical", nexcore_anatomy::CriticalityTier::Critical),
        ("Supporting", nexcore_anatomy::CriticalityTier::Supporting),
        ("Standard", nexcore_anatomy::CriticalityTier::Standard),
        (
            "Experimental",
            nexcore_anatomy::CriticalityTier::Experimental,
        ),
    ];

    for (label, tier) in &tiers {
        let count = metrics.crates_in_tier(*tier).len();
        eprintln!("  {label}: {count} crates");
    }
}

#[test]
fn test_full_anatomy_report() {
    let graph = load_workspace_graph();
    if graph.total_crates == 0 {
        return;
    }

    let report = AnatomyReport::from_graph(graph);

    // Workspace should not be Critical (no cycles expected)
    assert_ne!(
        report.summary.health,
        HealthStatus::Critical,
        "Workspace health should not be Critical"
    );

    eprintln!("--- Full Anatomy Report ---");
    eprintln!("Health: {:?}", report.summary.health);
    eprintln!("Total crates: {}", report.summary.total_crates);
    eprintln!("Critical: {}", report.summary.critical_count);
    eprintln!("Supporting: {}", report.summary.supporting_count);
    eprintln!("Experimental: {}", report.summary.experimental_count);
    eprintln!("Violations: {}", report.summary.violation_count);
    eprintln!("Cycles: {}", report.summary.cycle_count);
    eprintln!("Max depth: {}", report.summary.max_depth);
    eprintln!(
        "Bottleneck: {} (fan-in: {})",
        report.summary.bottleneck, report.summary.max_fan_in
    );
    eprintln!("Density: {:.4}", report.summary.graph_density);
}

#[test]
fn test_chomsky_classification() {
    let graph = load_workspace_graph();
    if graph.total_crates == 0 {
        return;
    }

    let chomsky = ChomskyReport::from_graph(&graph);

    // Should have profiles for every crate
    assert_eq!(chomsky.profiles.len(), graph.total_crates);

    // Should have all 4 levels represented in a diverse workspace
    let type3 = chomsky.at_level(ChomskyLevel::Type3Regular);
    let type2 = chomsky.at_level(ChomskyLevel::Type2ContextFree);
    let type1 = chomsky.at_level(ChomskyLevel::Type1ContextSensitive);
    let type0 = chomsky.at_level(ChomskyLevel::Type0Unrestricted);

    eprintln!("--- Chomsky Level Distribution ---");
    for (label, count) in &chomsky.level_distribution {
        eprintln!("  {label}: {count} crates");
    }
    eprintln!("Average generators: {:.2}", chomsky.avg_generators);

    eprintln!("\n--- Type-3 (Regular) crates ---");
    for p in &type3 {
        eprintln!("  {} [{}]", p.name, p.generators.join(", "));
    }

    eprintln!("\n--- Type-0 (Unrestricted) crates ---");
    for p in &type0 {
        eprintln!("  {} [{}]", p.name, p.generators.join(", "));
    }

    if !chomsky.overengineering_candidates.is_empty() {
        eprintln!(
            "\n--- Overengineering candidates ---\n  {:?}",
            chomsky.overengineering_candidates
        );
    }

    // Structural assertions
    assert!(
        !type0.is_empty(),
        "Should have Type-0 crates (engines, interpreters)"
    );
    assert!(
        type3.len() + type2.len() + type1.len() + type0.len() == graph.total_crates,
        "All crates should be classified"
    );
}
