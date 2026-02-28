//! Integration test: Reconcile workspace from ferro-forge hold manifests.
//!
//! This test loads the Bay from individual `ferro-forge/holds/*.toml` files
//! (the editable source of truth) rather than the bootstrap JSON path.
//!
//! Run with: cargo test --test reconcile_from_manifests -- --nocapture

use nexcore_topology::bridge::{manifest, scanner};
use nexcore_topology::reconcile::reconcile_with_scan;
use std::path::Path;

const WORKSPACE_ROOT: &str = "/home/matthew/Projects/nexcore";
const HOLDS_DIR: &str = "/home/matthew/Projects/nexcore/ferro-forge/holds";
const BAY_PATH: &str = "/home/matthew/Projects/nexcore/ferro-forge/bay.toml";

#[test]
fn reconcile_from_manifests() {
    let workspace_root = Path::new(WORKSPACE_ROOT);
    let holds_dir = Path::new(HOLDS_DIR);

    if !workspace_root.join("crates").is_dir() || !holds_dir.is_dir() {
        eprintln!("SKIP: workspace or ferro-forge/holds/ not found");
        return;
    }

    // Step 1: Load Bay from individual hold manifests
    let bay = manifest::load_bay_from_holds_dir(holds_dir)
        .unwrap_or_else(|e| panic!("load_bay_from_holds_dir failed: {e}"));

    eprintln!(
        "Loaded bay: {} holds, {} crates",
        bay.hold_count(),
        bay.crate_count()
    );

    // Step 2: Scan workspace
    let scan = scanner::scan_workspace(workspace_root)
        .unwrap_or_else(|e| panic!("scan_workspace failed: {e}"));

    eprintln!(
        "Scanned workspace: {} crates, {} edges",
        scan.crates.len(),
        scan.edges.len()
    );

    // Step 3: Reconcile
    let plan = reconcile_with_scan(&bay, &scan).unwrap_or_else(|e| panic!("reconcile failed: {e}"));

    let summary = plan.summary();

    eprintln!("\n=== Manifest-Based Reconciliation ===");
    eprintln!("Total actions: {}", summary.total_actions);
    eprintln!("Complexity: {}", summary.complexity);
    for (action_type, count) in &summary.by_type {
        eprintln!("  {action_type}: {count}");
    }
    eprintln!("=====================================\n");

    // Dump full plan
    eprintln!("{}", plan.to_markdown());

    // After P8 directional filter: SuggestMoves are suppressed when the
    // suggested hold is at a lower layer or is a foundation-keyword hold.
    // This eliminates all 59 original foundation-gravity false positives.
    let suggest_moves = summary.by_type.get("SuggestMove").copied().unwrap_or(0);
    eprintln!("SuggestMove: {suggest_moves}");
    assert!(
        suggest_moves < 10,
        "SuggestMove count should be < 10 after directional filter, got {suggest_moves}"
    );

    // Total actions should be < 30 (DV + LV + orphans only, no SuggestMoves).
    eprintln!("Total actions: {}", summary.total_actions);
    assert!(
        summary.total_actions < 30,
        "Expected < 30 total actions, got {}",
        summary.total_actions
    );
}

#[test]
fn regenerate_bay_from_holds() {
    let holds_dir = Path::new(HOLDS_DIR);
    let bay_path = Path::new(BAY_PATH);

    if !holds_dir.is_dir() {
        eprintln!("SKIP: ferro-forge/holds/ not found");
        return;
    }

    // Load Bay from hold manifests
    let bay = manifest::load_bay_from_holds_dir(holds_dir)
        .unwrap_or_else(|e| panic!("load_bay_from_holds_dir failed: {e}"));

    // Regenerate bay.toml
    let bay_toml = manifest::bay_to_toml(&bay, "nexcore-workspace");
    std::fs::write(bay_path, &bay_toml).unwrap_or_else(|e| panic!("write bay.toml failed: {e}"));

    eprintln!(
        "Regenerated bay.toml: {} holds, {} crates, {} bytes",
        bay.hold_count(),
        bay.crate_count(),
        bay_toml.len()
    );
}
