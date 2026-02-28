//! Integration test: Generate all ferro-forge manifest files from topology JSON.
//!
//! Run with: cargo test --test generate_ferro_forge -- --nocapture

use nexcore_topology::bridge::{bootstrap, manifest};
use std::path::Path;

#[test]
fn generate_ferro_forge_manifests() {
    let json_path = Path::new("/home/matthew/.claude/knowledge/workspace-topology.json");
    let ferro_dir = Path::new("/home/matthew/Projects/nexcore/ferro-forge");

    if !json_path.is_file() {
        eprintln!("SKIP: topology JSON not found");
        return;
    }

    // Bootstrap
    let result = bootstrap::from_topology_file(json_path).expect("bootstrap failed");

    // Write bay.toml
    let bay_toml = manifest::bay_to_toml(&result.bay, "nexcore-workspace");
    std::fs::write(ferro_dir.join("bay.toml"), &bay_toml).expect("write bay.toml");
    eprintln!("Wrote bay.toml ({} bytes)", bay_toml.len());

    // Write holds/*.toml
    let holds_dir = ferro_dir.join("holds");
    std::fs::create_dir_all(&holds_dir).expect("create holds/");

    let mut hold_count = 0;
    for (hold_name, hold_toml) in &result.hold_tomls {
        let filename = format!("{hold_name}.toml");
        std::fs::write(holds_dir.join(&filename), hold_toml).expect("write hold toml");
        hold_count += 1;
    }
    eprintln!("Wrote {hold_count} hold manifests to holds/");

    // Summary
    eprintln!("\n=== ferro-forge Generation Summary ===");
    eprintln!(
        "Bay: {} holds, {} crates",
        result.bay.hold_count(),
        result.bay.crate_count()
    );
    eprintln!("Coverage: {:.1}%", result.stats.coverage_pct);
    eprintln!("Unassigned: {}", result.unassigned.len());
    eprintln!("Hold manifest files: {hold_count}");
    eprintln!("========================================");
}
