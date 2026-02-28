//! Tests for the bridge module: manifest, scanner, bootstrap.

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "Tests use unwrap/assert/unreachable for validation"
)]
mod tests {
    use crate::Layer;
    use crate::bridge::bootstrap;
    use crate::bridge::manifest;

    // ========================================================================
    // Manifest — Hold
    // ========================================================================

    const HOLD_TOML: &str = r#"
[hold]
name = "core-primitives"
owner = "foundation-team"
review_required = true
members = ["nexcore-primitives", "nexcore-id", "nexcore-constants"]
"#;

    const HOLD_WITH_CYCLES_TOML: &str = r#"
[hold]
name = "stem-foundation"
owner = "stem-team"
review_required = false
members = ["stem-math", "nexcore-lex-primitiva", "stem-core"]

[[hold.cycle_partners]]
a = "stem-math"
b = "nexcore-lex-primitiva"
"#;

    #[test]
    fn manifest_load_hold() {
        let hold = manifest::load_hold(HOLD_TOML).unwrap();
        assert_eq!(hold.name(), "core-primitives");
        assert_eq!(hold.member_count(), 3);
        assert!(hold.contains("nexcore-primitives"));
        assert!(hold.contains("nexcore-id"));
        assert!(hold.contains("nexcore-constants"));
        assert_eq!(hold.governance().owner, "foundation-team");
        assert!(hold.governance().review_required);
        assert!(!hold.has_cycles());
    }

    #[test]
    fn manifest_load_hold_with_cycles() {
        let hold = manifest::load_hold(HOLD_WITH_CYCLES_TOML).unwrap();
        assert_eq!(hold.name(), "stem-foundation");
        assert!(hold.has_cycles());
        assert_eq!(hold.cycle_partners().len(), 1);
        let (a, b) = hold.cycle_partners().first().unwrap();
        assert_eq!(a, "stem-math");
        assert_eq!(b, "nexcore-lex-primitiva");
    }

    #[test]
    fn manifest_hold_rejects_empty_members() {
        let toml_str = r#"
[hold]
name = "empty"
owner = "nobody"
members = []
"#;
        let result = manifest::load_hold(toml_str);
        assert!(result.is_err());
    }

    #[test]
    fn manifest_hold_rejects_bad_toml() {
        let result = manifest::load_hold("this is not valid toml {{{");
        assert!(result.is_err());
    }

    // ========================================================================
    // Manifest — Hold Round-Trip
    // ========================================================================

    #[test]
    fn manifest_hold_round_trip() {
        let original = manifest::load_hold(HOLD_TOML).unwrap();
        let toml_str = manifest::hold_to_toml(&original);
        let reloaded = manifest::load_hold(&toml_str).unwrap();
        assert_eq!(original.name(), reloaded.name());
        assert_eq!(original.member_count(), reloaded.member_count());
        assert_eq!(original.governance().owner, reloaded.governance().owner);
        assert_eq!(
            original.governance().review_required,
            reloaded.governance().review_required
        );
        // Members must match exactly (BTreeSet is ordered)
        for member in original.members() {
            assert!(reloaded.contains(member));
        }
    }

    #[test]
    fn manifest_hold_with_cycles_round_trip() {
        let original = manifest::load_hold(HOLD_WITH_CYCLES_TOML).unwrap();
        let toml_str = manifest::hold_to_toml(&original);
        let reloaded = manifest::load_hold(&toml_str).unwrap();
        assert_eq!(original.name(), reloaded.name());
        assert_eq!(
            original.cycle_partners().len(),
            reloaded.cycle_partners().len()
        );
    }

    // ========================================================================
    // Manifest — Compartment
    // ========================================================================

    const COMPARTMENT_TOML: &str = r#"
[compartment]
name = "foundation"
layer = "Foundation"

[[compartment.holds]]
name = "core"
owner = "core-team"
review_required = true
members = ["nexcore-primitives", "nexcore-id"]

[[compartment.holds]]
name = "stem"
owner = "stem-team"
review_required = true
members = ["stem-math", "stem-core"]

[[compartment.edges]]
from = "stem"
to = "core"
"#;

    #[test]
    fn manifest_load_compartment() {
        let comp = manifest::load_compartment(COMPARTMENT_TOML).unwrap();
        assert_eq!(comp.name(), "foundation");
        assert_eq!(comp.layer(), Layer::Foundation);
        assert_eq!(comp.hold_count(), 2);
        assert_eq!(comp.crate_count(), 4);
        assert_eq!(comp.edges().len(), 1);
    }

    #[test]
    fn manifest_compartment_rejects_cycle() {
        let toml_str = r#"
[compartment]
name = "cyclic"
layer = "Domain"

[[compartment.holds]]
name = "a"
owner = "team"
members = ["crate-a"]

[[compartment.holds]]
name = "b"
owner = "team"
members = ["crate-b"]

[[compartment.edges]]
from = "a"
to = "b"

[[compartment.edges]]
from = "b"
to = "a"
"#;
        let result = manifest::load_compartment(toml_str);
        assert!(result.is_err());
    }

    #[test]
    fn manifest_compartment_rejects_bad_layer() {
        let toml_str = r#"
[compartment]
name = "bad"
layer = "NotALayer"

[[compartment.holds]]
name = "x"
owner = "team"
members = ["crate-x"]
"#;
        let result = manifest::load_compartment(toml_str);
        assert!(result.is_err());
    }

    #[test]
    fn manifest_compartment_topological_order() {
        let comp = manifest::load_compartment(COMPARTMENT_TOML).unwrap();
        let order = comp.topological_order().unwrap();
        // "core" is a dependency of "stem", so core comes before stem
        let core_pos = order.iter().position(|n| n == "core").unwrap();
        let stem_pos = order.iter().position(|n| n == "stem").unwrap();
        assert!(core_pos < stem_pos);
    }

    // ========================================================================
    // Manifest — Bay
    // ========================================================================

    const BAY_TOML: &str = r#"
[bay]
name = "test-workspace"

[[bay.holds]]
name = "core"
owner = "core-team"
review_required = true
members = ["nexcore-primitives", "nexcore-id"]

[[bay.holds]]
name = "pv"
owner = "pv-team"
review_required = true
members = ["nexcore-vigilance", "nexcore-pvos"]
"#;

    #[test]
    fn manifest_load_bay() {
        let bay = manifest::load_bay(BAY_TOML).unwrap();
        assert_eq!(bay.hold_count(), 2);
        assert_eq!(bay.crate_count(), 4);
        assert_eq!(bay.find_crate("nexcore-primitives"), Some("core"));
        assert_eq!(bay.find_crate("nexcore-vigilance"), Some("pv"));
    }

    #[test]
    fn manifest_bay_rejects_duplicate_crate() {
        let toml_str = r#"
[bay]
name = "bad"

[[bay.holds]]
name = "a"
owner = "team"
members = ["shared"]

[[bay.holds]]
name = "b"
owner = "team"
members = ["shared"]
"#;
        let result = manifest::load_bay(toml_str);
        assert!(result.is_err());
    }

    #[test]
    fn manifest_bay_round_trip() {
        let original = manifest::load_bay(BAY_TOML).unwrap();
        let toml_str = manifest::bay_to_toml(&original, "test-workspace");
        let reloaded = manifest::load_bay(&toml_str).unwrap();
        assert_eq!(original.hold_count(), reloaded.hold_count());
        assert_eq!(original.crate_count(), reloaded.crate_count());
        // Every crate in original should be findable in reloaded
        for hold in original.holds().values() {
            for member in hold.members() {
                assert!(
                    reloaded.find_crate(member).is_some(),
                    "crate '{member}' not found after round-trip"
                );
            }
        }
    }

    // ========================================================================
    // Manifest — Layer Parsing
    // ========================================================================

    #[test]
    fn manifest_parse_layer_all_variants() {
        assert_eq!(
            manifest::parse_layer("Foundation").unwrap(),
            Layer::Foundation
        );
        assert_eq!(manifest::parse_layer("Domain").unwrap(), Layer::Domain);
        assert_eq!(
            manifest::parse_layer("Orchestration").unwrap(),
            Layer::Orchestration
        );
        assert_eq!(manifest::parse_layer("Service").unwrap(), Layer::Service);
        // Case insensitive
        assert_eq!(
            manifest::parse_layer("foundation").unwrap(),
            Layer::Foundation
        );
        assert_eq!(manifest::parse_layer("DOMAIN").unwrap(), Layer::Domain);
    }

    #[test]
    fn manifest_parse_layer_rejects_unknown() {
        assert!(manifest::parse_layer("Unknown").is_err());
        assert!(manifest::parse_layer("").is_err());
    }

    // ========================================================================
    // Manifest — Error Display
    // ========================================================================

    #[test]
    fn manifest_error_display() {
        let e = manifest::ManifestError::TomlParse("bad input".to_owned());
        assert!(format!("{e}").contains("bad input"));

        let e = manifest::ManifestError::UnknownLayer("Weird".to_owned());
        assert!(format!("{e}").contains("Weird"));

        let e = manifest::ManifestError::Io("file not found".to_owned());
        assert!(format!("{e}").contains("file not found"));
    }

    // ========================================================================
    // Bootstrap — Minimal JSON
    // ========================================================================

    fn minimal_topology_json() -> String {
        serde_json::json!({
            "metadata": { "total_crates": 4, "source": "test", "generated": "2026-01-01T00:00:00Z", "method": "test" },
            "summary": {
                "layers": { "Foundation": 2, "Domain": 2 },
                "clusters": [
                    { "name": "core", "count": 2, "members": ["crate-a", "crate-b"] },
                    { "name": "pv", "count": 2, "members": ["crate-c", "crate-d"] }
                ],
                "capability_paths": []
            },
            "crates": [
                { "name": "crate-a", "layer": "Foundation", "layer_depth": 0, "domain_cluster": "core", "internal_dep_count": 0, "dependent_count": 1, "dependencies": [], "dependents": ["crate-b"] },
                { "name": "crate-b", "layer": "Foundation", "layer_depth": 0, "domain_cluster": "core", "internal_dep_count": 1, "dependent_count": 0, "dependencies": ["crate-a"], "dependents": [] },
                { "name": "crate-c", "layer": "Domain", "layer_depth": 1, "domain_cluster": "pv", "internal_dep_count": 0, "dependent_count": 1, "dependencies": [], "dependents": ["crate-d"] },
                { "name": "crate-d", "layer": "Domain", "layer_depth": 1, "domain_cluster": "pv", "internal_dep_count": 1, "dependent_count": 0, "dependencies": ["crate-c"], "dependents": [] }
            ]
        })
        .to_string()
    }

    #[test]
    fn bootstrap_from_minimal_json() {
        let json = minimal_topology_json();
        let result = bootstrap::from_topology_json(&json).unwrap();
        assert_eq!(result.bay.hold_count(), 2);
        assert_eq!(result.bay.crate_count(), 4);
        assert_eq!(result.stats.total_crates, 4);
        assert_eq!(result.stats.assigned_crates, 4);
        assert_eq!(result.stats.hold_count, 2);
        assert!((result.stats.coverage_pct - 100.0).abs() < 0.01);
        assert!(result.unassigned.is_empty());
    }

    #[test]
    fn bootstrap_bay_round_trip() {
        let json = minimal_topology_json();
        let result = bootstrap::from_topology_json(&json).unwrap();

        // The bay_toml should round-trip through load_bay
        let reloaded = manifest::load_bay(&result.bay_toml).unwrap();
        assert_eq!(result.bay.hold_count(), reloaded.hold_count());
        assert_eq!(result.bay.crate_count(), reloaded.crate_count());
    }

    #[test]
    fn bootstrap_hold_tomls_round_trip() {
        let json = minimal_topology_json();
        let result = bootstrap::from_topology_json(&json).unwrap();

        // Each hold TOML should round-trip through load_hold
        for (name, toml_str) in &result.hold_tomls {
            let hold = manifest::load_hold(toml_str)
                .unwrap_or_else(|e| panic!("hold '{name}' failed to round-trip: {e}"));
            assert_eq!(hold.name(), name.as_str());
        }
    }

    #[test]
    fn bootstrap_detects_cycles() {
        let json = serde_json::json!({
            "metadata": { "total_crates": 2, "source": "test", "generated": "2026-01-01T00:00:00Z", "method": "test" },
            "summary": {
                "layers": { "Foundation": 2 },
                "clusters": [{ "name": "cyclic", "count": 2, "members": ["a", "b"] }],
                "capability_paths": []
            },
            "crates": [
                { "name": "a", "layer": "Foundation", "layer_depth": 0, "domain_cluster": "cyclic", "internal_dep_count": 1, "dependent_count": 1, "dependencies": ["b"], "dependents": ["b"] },
                { "name": "b", "layer": "Foundation", "layer_depth": 0, "domain_cluster": "cyclic", "internal_dep_count": 1, "dependent_count": 1, "dependencies": ["a"], "dependents": ["a"] }
            ]
        })
        .to_string();

        let result = bootstrap::from_topology_json(&json).unwrap();
        let cyclic_hold = result.bay.find_hold("cyclic").unwrap();
        assert!(cyclic_hold.has_cycles());
        assert_eq!(cyclic_hold.cycle_partners().len(), 1);
    }

    #[test]
    fn bootstrap_handles_unassigned() {
        let json = serde_json::json!({
            "metadata": { "total_crates": 3, "source": "test", "generated": "2026-01-01T00:00:00Z", "method": "test" },
            "summary": {
                "layers": { "Foundation": 3 },
                "clusters": [{ "name": "core", "count": 2, "members": ["a", "b"] }],
                "capability_paths": []
            },
            "crates": [
                { "name": "a", "layer": "Foundation", "layer_depth": 0, "domain_cluster": "core", "internal_dep_count": 0, "dependent_count": 0, "dependencies": [], "dependents": [] },
                { "name": "b", "layer": "Foundation", "layer_depth": 0, "domain_cluster": "core", "internal_dep_count": 0, "dependent_count": 0, "dependencies": [], "dependents": [] },
                { "name": "orphan", "layer": "Foundation", "layer_depth": 0, "domain_cluster": "", "internal_dep_count": 0, "dependent_count": 0, "dependencies": [], "dependents": [] }
            ]
        })
        .to_string();

        let result = bootstrap::from_topology_json(&json).unwrap();
        assert_eq!(result.unassigned.len(), 1);
        assert_eq!(result.unassigned.first().unwrap(), "orphan");
        assert_eq!(result.stats.assigned_crates, 2);
    }

    #[test]
    fn bootstrap_rejects_bad_json() {
        let result = bootstrap::from_topology_json("not json at all");
        assert!(result.is_err());
    }

    // ========================================================================
    // Bootstrap — Error Display
    // ========================================================================

    #[test]
    fn bootstrap_error_display() {
        let e = bootstrap::BootstrapError::JsonParse("unexpected token".to_owned());
        assert!(format!("{e}").contains("unexpected token"));

        let e = bootstrap::BootstrapError::Io("file not found".to_owned());
        assert!(format!("{e}").contains("file not found"));
    }

    // ========================================================================
    // Bootstrap — resolve_layer
    // ========================================================================

    #[test]
    fn bootstrap_resolve_layer() {
        assert_eq!(
            bootstrap::resolve_layer("Foundation"),
            Some(Layer::Foundation)
        );
        assert_eq!(bootstrap::resolve_layer("Domain"), Some(Layer::Domain));
        assert_eq!(
            bootstrap::resolve_layer("Orchestration"),
            Some(Layer::Orchestration)
        );
        assert_eq!(bootstrap::resolve_layer("Service"), Some(Layer::Service));
        assert_eq!(bootstrap::resolve_layer("Unknown"), None);
    }

    // ========================================================================
    // Scanner — Unit Tests (no filesystem)
    // ========================================================================

    #[test]
    fn scanner_rejects_nonexistent_dir() {
        use crate::bridge::scanner;
        use std::path::Path;

        let result = scanner::scan_workspace(Path::new("/tmp/nonexistent-workspace-dir-12345"));
        assert!(result.is_err());
    }

    #[test]
    fn scanner_error_display() {
        use crate::bridge::scanner::ScanError;

        let e = ScanError::NoCratesDir("/some/path".to_owned());
        assert!(format!("{e}").contains("/some/path"));

        let e = ScanError::Io("permission denied".to_owned());
        assert!(format!("{e}").contains("permission denied"));
    }

    // ========================================================================
    // Scanner — Integration (scans actual workspace)
    // ========================================================================

    #[test]
    fn scanner_scans_real_workspace() {
        use crate::bridge::scanner;
        use std::path::Path;

        let workspace_root = Path::new("/home/matthew/Projects/nexcore");
        if !workspace_root.join("crates").is_dir() {
            // Skip if not in the expected environment
            return;
        }

        let scan = scanner::scan_workspace(workspace_root).unwrap();
        // The workspace has many crates
        assert!(
            scan.crates.len() > 100,
            "expected >100 crates, got {}",
            scan.crates.len()
        );
        // There should be internal dependency edges
        assert!(!scan.edges.is_empty(), "expected internal dependency edges");
        // Note: stem-math ↔ nexcore-lex-primitiva is a dev-dependency cycle,
        // not a production [dependencies] cycle. The scanner correctly only
        // reads [dependencies], so this cycle is not detected — by design.
        // Verify the scanner found at least some edges from stem-math
        let stem_math_edges: Vec<_> = scan
            .edges
            .iter()
            .filter(|(from, _)| from == "stem-math")
            .collect();
        assert!(
            !stem_math_edges.is_empty(),
            "expected stem-math to have dependencies"
        );
    }

    // ========================================================================
    // Full Pipeline — Bootstrap → Round-Trip (real workspace-topology.json)
    // ========================================================================

    #[test]
    fn full_pipeline_bootstrap_real_topology() {
        use std::path::Path;

        let topo_path = Path::new("/home/matthew/.claude/knowledge/workspace-topology.json");
        if !topo_path.is_file() {
            // Skip if file not available
            return;
        }

        let result = bootstrap::from_topology_file(topo_path).unwrap();

        // Coverage should be >= 90%
        assert!(
            result.stats.coverage_pct >= 90.0,
            "coverage {:.1}% < 90% threshold",
            result.stats.coverage_pct
        );

        // Bay should have holds
        assert!(result.bay.hold_count() > 10, "expected >10 holds");

        // Bay TOML should round-trip
        let reloaded = manifest::load_bay(&result.bay_toml).unwrap();
        assert_eq!(result.bay.hold_count(), reloaded.hold_count());
        assert_eq!(result.bay.crate_count(), reloaded.crate_count());

        // Every hold TOML should round-trip
        for (name, toml_str) in &result.hold_tomls {
            let hold = manifest::load_hold(toml_str)
                .unwrap_or_else(|e| panic!("hold '{name}' round-trip failed: {e}"));
            assert_eq!(hold.name(), name.as_str());
        }
    }
}
