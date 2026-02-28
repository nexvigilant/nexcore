//! Tests for the reconciliation engine.

use super::*;
use crate::{GovernancePolicy, Hold, Layer};
use std::collections::BTreeSet;

// ============================================================================
// Helpers
// ============================================================================

fn gov(owner: &str) -> GovernancePolicy {
    GovernancePolicy {
        owner: owner.to_owned(),
        review_required: true,
    }
}

fn hold(name: &str, members: &[&str], owner: &str) -> Hold {
    let members: BTreeSet<String> = members.iter().map(|s| (*s).to_owned()).collect();
    Hold::new(name, members, gov(owner)).unwrap_or_else(|e| panic!("hold '{name}' failed: {e}"))
}

fn bay(holds: Vec<Hold>) -> Bay {
    Bay::new(holds).unwrap_or_else(|e| panic!("bay failed: {e}"))
}

fn scan(crates: &[&str], edges: &[(&str, &str)]) -> WorkspaceScan {
    WorkspaceScan {
        crates: crates.iter().map(|s| (*s).to_owned()).collect(),
        edges: edges
            .iter()
            .map(|(a, b)| ((*a).to_owned(), (*b).to_owned()))
            .collect(),
        cycles: Vec::new(),
        parse_errors: Vec::new(),
    }
}

// ============================================================================
// ForgeAction Tests
// ============================================================================

#[test]
fn action_priority_ordering() {
    let actions = vec![
        ForgeAction::SuggestMove {
            crate_name: "a".into(),
            current_hold: "h1".into(),
            suggested_hold: "h2".into(),
            reason: "test".into(),
        },
        ForgeAction::OrphanCrate {
            crate_name: "b".into(),
        },
        ForgeAction::MissingCrate {
            crate_name: "c".into(),
            hold_name: "h1".into(),
        },
        ForgeAction::DirectionViolation {
            source_crate: "d".into(),
            source_hold: "h1".into(),
            target_crate: "e".into(),
            target_hold: "h2".into(),
            declared_direction: "up".into(),
        },
        ForgeAction::LayerViolation {
            crate_name: "f".into(),
            hold_name: "h1".into(),
            declared_layer: "Foundation".into(),
            actual_depth: 10,
        },
        ForgeAction::GovernanceDrift {
            hold_name: "h1".into(),
            field: "owner".into(),
            declared_value: "a".into(),
            actual_value: "b".into(),
        },
    ];

    let plan = ForgePlan::new(actions);
    let priorities: Vec<u8> = plan.actions().iter().map(|a| a.priority()).collect();
    // Must be sorted ascending
    for window in priorities.windows(2) {
        assert!(
            window[0] <= window[1],
            "Priority order violated: {} > {}",
            window[0],
            window[1]
        );
    }
}

#[test]
fn action_type_names() {
    assert_eq!(
        ForgeAction::OrphanCrate {
            crate_name: "x".into()
        }
        .action_type_name(),
        "OrphanCrate"
    );
    assert_eq!(
        ForgeAction::MissingCrate {
            crate_name: "x".into(),
            hold_name: "h".into()
        }
        .action_type_name(),
        "MissingCrate"
    );
    assert_eq!(
        ForgeAction::DirectionViolation {
            source_crate: "a".into(),
            source_hold: "h1".into(),
            target_crate: "b".into(),
            target_hold: "h2".into(),
            declared_direction: "down".into(),
        }
        .action_type_name(),
        "DirectionViolation"
    );
}

#[test]
fn action_target_returns_primary_entity() {
    let a = ForgeAction::OrphanCrate {
        crate_name: "foo".into(),
    };
    assert_eq!(a.target(), "foo");

    let b = ForgeAction::GovernanceDrift {
        hold_name: "bar-hold".into(),
        field: "owner".into(),
        declared_value: "x".into(),
        actual_value: "y".into(),
    };
    assert_eq!(b.target(), "bar-hold");
}

#[test]
fn action_detail_nonempty() {
    let actions = vec![
        ForgeAction::OrphanCrate {
            crate_name: "x".into(),
        },
        ForgeAction::MissingCrate {
            crate_name: "x".into(),
            hold_name: "h".into(),
        },
        ForgeAction::DirectionViolation {
            source_crate: "a".into(),
            source_hold: "h1".into(),
            target_crate: "b".into(),
            target_hold: "h2".into(),
            declared_direction: "down".into(),
        },
        ForgeAction::LayerViolation {
            crate_name: "x".into(),
            hold_name: "h".into(),
            declared_layer: "Foundation".into(),
            actual_depth: 10,
        },
        ForgeAction::GovernanceDrift {
            hold_name: "h".into(),
            field: "owner".into(),
            declared_value: "a".into(),
            actual_value: "b".into(),
        },
        ForgeAction::SuggestMove {
            crate_name: "x".into(),
            current_hold: "h1".into(),
            suggested_hold: "h2".into(),
            reason: "deps".into(),
        },
    ];
    for a in &actions {
        assert!(
            !a.detail().is_empty(),
            "detail empty for {}",
            a.action_type_name()
        );
    }
}

// ============================================================================
// Complexity Tests
// ============================================================================

#[test]
fn complexity_thresholds() {
    assert_eq!(Complexity::from_count(0), Complexity::Low);
    assert_eq!(Complexity::from_count(10), Complexity::Low);
    assert_eq!(Complexity::from_count(11), Complexity::Medium);
    assert_eq!(Complexity::from_count(50), Complexity::Medium);
    assert_eq!(Complexity::from_count(51), Complexity::High);
    assert_eq!(Complexity::from_count(1000), Complexity::High);
}

#[test]
fn complexity_display() {
    assert_eq!(Complexity::Low.to_string(), "Low");
    assert_eq!(Complexity::Medium.to_string(), "Medium");
    assert_eq!(Complexity::High.to_string(), "High");
}

// ============================================================================
// ForgePlan Tests
// ============================================================================

#[test]
fn empty_plan_is_clean() {
    let plan = ForgePlan::new(Vec::new());
    assert!(plan.is_clean());
    assert_eq!(plan.action_count(), 0);
    let summary = plan.summary();
    assert_eq!(summary.total_actions, 0);
    assert_eq!(summary.complexity, Complexity::Low);
}

#[test]
fn plan_summary_counts_by_type() {
    let actions = vec![
        ForgeAction::OrphanCrate {
            crate_name: "a".into(),
        },
        ForgeAction::OrphanCrate {
            crate_name: "b".into(),
        },
        ForgeAction::MissingCrate {
            crate_name: "c".into(),
            hold_name: "h1".into(),
        },
    ];
    let plan = ForgePlan::new(actions);
    let summary = plan.summary();
    assert_eq!(summary.total_actions, 3);
    assert_eq!(summary.by_type.get("OrphanCrate").copied(), Some(2));
    assert_eq!(summary.by_type.get("MissingCrate").copied(), Some(1));
    assert_eq!(summary.complexity, Complexity::Low);
}

// ============================================================================
// Serialization Tests
// ============================================================================

#[test]
fn markdown_contains_header_and_table() {
    let actions = vec![ForgeAction::OrphanCrate {
        crate_name: "orphan-crate".into(),
    }];
    let plan = ForgePlan::new(actions);
    let md = plan.to_markdown();
    assert!(md.contains("# Reconciliation Plan"), "missing header");
    assert!(
        md.contains("orphan-crate"),
        "missing crate name in markdown"
    );
    assert!(
        md.contains("OrphanCrate"),
        "missing action type in markdown"
    );
    assert!(md.contains("| # |"), "missing table header");
}

#[test]
fn markdown_empty_plan_says_no_actions() {
    let plan = ForgePlan::new(Vec::new());
    let md = plan.to_markdown();
    assert!(md.contains("No actions required"));
}

#[test]
fn json_serialization_round_trip() {
    let actions = vec![
        ForgeAction::OrphanCrate {
            crate_name: "x".into(),
        },
        ForgeAction::MissingCrate {
            crate_name: "y".into(),
            hold_name: "h".into(),
        },
    ];
    let plan = ForgePlan::new(actions);
    let json = plan.to_json().expect("serialization failed");
    assert!(json.contains("\"action_type\": \"OrphanCrate\""));
    assert!(json.contains("\"action_type\": \"MissingCrate\""));
    // Verify it's valid JSON
    let parsed: serde_json::Value = serde_json::from_str(&json).expect("JSON parse failed");
    assert!(parsed.get("actions").is_some());
}

// ============================================================================
// Reconciliation Engine Tests
// ============================================================================

#[test]
fn detects_orphan_crates() {
    let b = bay(vec![hold(
        "foundation",
        &["crate-a", "crate-b"],
        "foundation-team",
    )]);
    let s = scan(&["crate-a", "crate-b", "crate-orphan"], &[]);
    let plan = reconcile_with_scan(&b, &s).expect("reconcile failed");

    let orphans: Vec<&ForgeAction> = plan
        .actions()
        .iter()
        .filter(|a| matches!(a, ForgeAction::OrphanCrate { .. }))
        .collect();
    assert_eq!(orphans.len(), 1);
    assert_eq!(orphans[0].target(), "crate-orphan");
}

#[test]
fn detects_missing_crates() {
    let b = bay(vec![hold(
        "domain",
        &["crate-a", "crate-missing"],
        "domain-team",
    )]);
    let s = scan(&["crate-a"], &[]);
    let plan = reconcile_with_scan(&b, &s).expect("reconcile failed");

    let missing: Vec<&ForgeAction> = plan
        .actions()
        .iter()
        .filter(|a| matches!(a, ForgeAction::MissingCrate { .. }))
        .collect();
    assert_eq!(missing.len(), 1);
    assert_eq!(missing[0].target(), "crate-missing");
}

#[test]
fn detects_two_orphans_in_five_hold_bay() {
    let b = bay(vec![
        hold("foundation", &["stem-core", "stem-math"], "foundation-team"),
        hold("domain", &["nexcore-vigilance"], "domain-team"),
        hold("orchestration", &["nexcore-brain"], "orchestration-team"),
        hold("service", &["nexcore-mcp"], "service-team"),
        hold("bio", &["nexcore-cytokine"], "domain-team"),
    ]);
    let s = scan(
        &[
            "stem-core",
            "stem-math",
            "nexcore-vigilance",
            "nexcore-brain",
            "nexcore-mcp",
            "nexcore-cytokine",
            "orphan-alpha",
            "orphan-beta",
        ],
        &[],
    );
    let plan = reconcile_with_scan(&b, &s).expect("reconcile failed");
    let orphans: Vec<&str> = plan
        .actions()
        .iter()
        .filter_map(|a| match a {
            ForgeAction::OrphanCrate { crate_name } => Some(crate_name.as_str()),
            _ => None,
        })
        .collect();
    assert_eq!(orphans.len(), 2);
    assert!(orphans.contains(&"orphan-alpha"));
    assert!(orphans.contains(&"orphan-beta"));
}

#[test]
fn detects_direction_violation() {
    // Foundation crate depending on a Service crate = violation
    let b = bay(vec![
        hold("foundation", &["crate-low"], "foundation-team"),
        hold("service", &["crate-high"], "service-team"),
    ]);
    // crate-low depends on crate-high (Foundation → Service = violation)
    let s = scan(&["crate-low", "crate-high"], &[("crate-low", "crate-high")]);
    let plan = reconcile_with_scan(&b, &s).expect("reconcile failed");

    let violations: Vec<&ForgeAction> = plan
        .actions()
        .iter()
        .filter(|a| matches!(a, ForgeAction::DirectionViolation { .. }))
        .collect();
    assert_eq!(violations.len(), 1);
    assert_eq!(violations[0].target(), "crate-low");
}

#[test]
fn no_direction_violation_for_correct_flow() {
    // Service depending on Foundation = correct
    let b = bay(vec![
        hold("foundation", &["crate-low"], "foundation-team"),
        hold("service", &["crate-high"], "service-team"),
    ]);
    let s = scan(&["crate-low", "crate-high"], &[("crate-high", "crate-low")]);
    let plan = reconcile_with_scan(&b, &s).expect("reconcile failed");

    let violations: Vec<&ForgeAction> = plan
        .actions()
        .iter()
        .filter(|a| matches!(a, ForgeAction::DirectionViolation { .. }))
        .collect();
    assert_eq!(violations.len(), 0);
}

#[test]
fn detects_layer_violation() {
    // Crate in foundation hold but with many internal deps = should be Domain
    let b = bay(vec![hold(
        "foundation",
        &["crate-heavy"],
        "foundation-team",
    )]);

    // >3 deps triggers Domain layer, but hold declares Foundation → violation
    let edges: Vec<(&str, &str)> = (0..10)
        .map(|i| {
            // We'll reuse the edge names; the scan just counts
            if i % 2 == 0 {
                ("crate-heavy", "dep1")
            } else {
                ("crate-heavy", "dep2")
            }
        })
        .collect();

    let crates = vec!["crate-heavy", "dep1", "dep2"];
    let s2 = scan(&crates, &edges);
    let plan = reconcile_with_scan(&b, &s2).expect("reconcile failed");

    let layer_violations: Vec<&ForgeAction> = plan
        .actions()
        .iter()
        .filter(|a| matches!(a, ForgeAction::LayerViolation { .. }))
        .collect();
    assert_eq!(layer_violations.len(), 1);
    assert_eq!(layer_violations[0].target(), "crate-heavy");
}

#[test]
fn clean_plan_for_matching_state() {
    // Both holds at foundation layer, correct direction, matching dep counts
    let b = bay(vec![
        hold("foundation-core", &["crate-a"], "foundation-team"),
        hold("foundation-math", &["crate-b"], "foundation-team"),
    ]);
    // Workspace matches exactly, correct direction, both have ≤3 deps (Foundation)
    let s = scan(&["crate-a", "crate-b"], &[("crate-b", "crate-a")]);
    let plan = reconcile_with_scan(&b, &s).expect("reconcile failed");
    assert!(
        plan.is_clean(),
        "expected clean plan, got {} actions: {:?}",
        plan.action_count(),
        plan.actions()
    );
}

#[test]
fn scan_error_produces_topology_error() {
    let b = bay(vec![hold("test", &["c"], "team")]);
    let result = reconcile(&b, std::path::Path::new("/nonexistent/path"));
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        matches!(err, TopologyError::ScanFailed(_)),
        "expected ScanFailed, got: {err}"
    );
}

#[test]
fn plan_with_mixed_actions_orders_correctly() {
    let b = bay(vec![
        hold(
            "foundation",
            &["crate-a", "crate-missing"],
            "foundation-team",
        ),
        hold("service", &["crate-b"], "service-team"),
    ]);
    let s = scan(
        &["crate-a", "crate-b", "crate-orphan"],
        &[("crate-a", "crate-b")],
    );
    let plan = reconcile_with_scan(&b, &s).expect("reconcile failed");

    // Should have: OrphanCrate (crate-orphan), MissingCrate (crate-missing),
    // DirectionViolation (crate-a → crate-b = Foundation → Service)
    assert!(plan.action_count() >= 2);

    // Verify priority ordering
    let priorities: Vec<u8> = plan.actions().iter().map(|a| a.priority()).collect();
    for window in priorities.windows(2) {
        assert!(window[0] <= window[1], "priority order violated");
    }
}

// ============================================================================
// Layer Inference Tests
// ============================================================================

#[test]
fn infer_layer_from_depth_thresholds() {
    assert_eq!(infer_layer_from_depth(0), Layer::Foundation);
    assert_eq!(infer_layer_from_depth(3), Layer::Foundation);
    assert_eq!(infer_layer_from_depth(4), Layer::Domain);
    assert_eq!(infer_layer_from_depth(25), Layer::Domain);
    assert_eq!(infer_layer_from_depth(26), Layer::Orchestration);
    assert_eq!(infer_layer_from_depth(50), Layer::Orchestration);
    assert_eq!(infer_layer_from_depth(51), Layer::Service);
    assert_eq!(infer_layer_from_depth(100), Layer::Service);
}

#[test]
fn infer_layer_from_name_keywords() {
    assert_eq!(
        infer_layer_from_name("foundation-primitives"),
        Layer::Foundation
    );
    assert_eq!(infer_layer_from_name("stem-math"), Layer::Foundation);
    assert_eq!(infer_layer_from_name("domain-vigilance"), Layer::Domain);
    assert_eq!(
        infer_layer_from_name("orchestration-brain"),
        Layer::Orchestration
    );
    assert_eq!(infer_layer_from_name("service-api"), Layer::Service);
    // Unknown defaults to Domain (most holds are domain-level)
    assert_eq!(infer_layer_from_name("unknown-hold"), Layer::Domain);
}

// ============================================================================
// TopologyError Display Tests
// ============================================================================

#[test]
fn scan_failed_error_display() {
    let err = TopologyError::ScanFailed("test error".into());
    assert_eq!(err.to_string(), "workspace scan failed: test error");
}

#[test]
fn serialize_failed_error_display() {
    let err = TopologyError::SerializeFailed("json broke".into());
    assert_eq!(err.to_string(), "serialization failed: json broke");
}

// ============================================================================
// Integration: Real Workspace Reconciliation
// ============================================================================

#[test]
fn real_workspace_reconciliation() {
    use crate::bridge::{bootstrap, scanner};
    use std::path::Path;

    let workspace_root = Path::new("/home/matthew/Projects/nexcore");
    let json_path = Path::new("/home/matthew/.claude/knowledge/workspace-topology.json");

    if !workspace_root.join("crates").is_dir() || !json_path.is_file() {
        // Skip if not in expected environment
        return;
    }

    // Step 1: Bootstrap Bay from topology JSON
    let bootstrap_result = bootstrap::from_topology_file(json_path).expect("bootstrap failed");
    let declared = bootstrap_result.bay;

    // Step 2: Scan workspace
    let scan_result = scanner::scan_workspace(workspace_root).expect("scan failed");

    // Step 3: Reconcile
    let plan = reconcile_with_scan(&declared, &scan_result).expect("reconcile failed");

    let summary = plan.summary();

    // Print summary for human review
    eprintln!("\n=== Real Workspace Reconciliation ===");
    eprintln!("Total actions: {}", summary.total_actions);
    eprintln!("Complexity: {}", summary.complexity);
    for (action_type, count) in &summary.by_type {
        eprintln!("  {action_type}: {count}");
    }
    eprintln!("====================================\n");

    // Dump full plan for triage analysis (visible with --nocapture)
    eprintln!("{}", plan.to_markdown());

    // Structural assertions (the plan should produce SOMETHING for a
    // 224-crate workspace — perfect alignment is unlikely)
    assert!(declared.hold_count() > 0, "bootstrap produced no holds");
    assert!(
        scan_result.crates.len() > 100,
        "expected >100 crates in workspace scan, got {}",
        scan_result.crates.len()
    );

    // The plan should serialize cleanly
    let json = plan.to_json().expect("JSON serialization failed");
    assert!(!json.is_empty());

    let md = plan.to_markdown();
    assert!(md.contains("# Reconciliation Plan"));
}
