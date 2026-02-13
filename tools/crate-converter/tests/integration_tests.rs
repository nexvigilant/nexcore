use crate_converter::{convert_cargo_toml, InternalDepResolver};
use pretty_assertions::assert_eq;
use std::collections::HashMap;

/// A test resolver that returns pre-configured versions for internal crates.
struct TestResolver {
    versions: HashMap<String, String>,
}

impl TestResolver {
    fn new(entries: &[(&str, &str)]) -> Self {
        Self {
            versions: entries
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
        }
    }
}

impl InternalDepResolver for TestResolver {
    fn resolve_version(&self, crate_name: &str) -> anyhow::Result<String> {
        self.versions
            .get(crate_name)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("Unknown crate: {crate_name}"))
    }
}

// ---------------------------------------------------------------------------
// Workspace TOML used by most tests
// ---------------------------------------------------------------------------
const WORKSPACE_TOML: &str = r#"
[workspace]
members = ["crates/*"]
resolver = "2"

[workspace.package]
version = "1.0.0"
edition = "2024"
rust-version = "1.85"
authors = ["NexVigilant Team <team@nexvigilant.com>"]
description = "NexVigilant Core"
license = "LicenseRef-Proprietary"
repository = "https://github.com/nexvigilant/nexcore"

[workspace.dependencies]
serde = { version = "1.0.204", features = ["derive"] }
serde_json = "1.0.120"
chrono = { version = "0.4.38", features = ["serde"] }
tokio = { version = "1.38.0", features = ["full"] }
thiserror = "1.0.61"
rayon = "1.10.0"
phf = { version = "0.11.2", features = ["macros"] }
once_cell = "1.19.0"
tempfile = "3.10.1"
clap = { version = "4.5.11", features = ["derive", "cargo", "env"] }
proptest = "1.6.0"
tower = { version = "0.4.13", features = ["full"] }
http-body-util = "0.1.2"
axum = { version = "0.8.0", features = ["macros"] }
bytes = "1.6.1"
anyhow = "1.0.86"
tracing = "0.1.40"
uuid = { version = "1.10.0", features = ["v4", "serde"] }
ordered-float = { version = "4", features = ["serde"] }
typenum = "1.17.0"

# Internal deps
nexcore-tov = { path = "crates/nexcore-tov" }
nexcore-constants = { path = "crates/nexcore-constants" }
nexcore-lex-primitiva = { path = "crates/nexcore-lex-primitiva" }
nexcore-signal-types = { path = "crates/nexcore-signal-types" }
nexcore-id = { path = "crates/nexcore-id", features = ["serde"] }
signal = { path = "crates/signal" }
stem-math = { path = "crates/stem-math" }
stem-core = { path = "crates/stem-core" }
lex-primitiva = { path = "crates/nexcore-lex-primitiva", package = "nexcore-lex-primitiva" }
nexcore-transcriptase = { path = "crates/nexcore-transcriptase" }
antitransformer = { path = "crates/antitransformer" }

[workspace.lints.rust]
async_fn_in_trait = "allow"
dead_code = "allow"
unused_imports = "allow"
unused_variables = "allow"
unused_mut = "allow"
unused_assignments = "allow"
unused_must_use = "allow"

[workspace.lints.clippy]
all = "allow"
pedantic = "allow"
nursery = "allow"
cargo = "allow"
"#;

fn test_resolver() -> TestResolver {
    TestResolver::new(&[
        ("nexcore-tov", "0.2.0"),
        ("nexcore-constants", "0.1.0"),
        ("nexcore-lex-primitiva", "0.1.0"),
        ("nexcore-signal-types", "0.3.0"),
        ("nexcore-id", "0.1.0"),
        ("signal", "1.0.0"),
        ("stem-math", "0.1.0"),
        ("stem-core", "0.1.0"),
        ("nexcore-transcriptase", "0.1.0"),
        ("antitransformer", "0.1.0"),
    ])
}

// ---------------------------------------------------------------------------
// Pattern A: full workspace inheritance
// ---------------------------------------------------------------------------
#[test]
fn pattern_a_full_workspace_inheritance() {
    let crate_toml = r#"[package]
name = "nexcore-pv-core"
version.workspace = true
edition.workspace = true
authors.workspace = true
license.workspace = true
repository.workspace = true
rust-version.workspace = true
description = "Pharmacovigilance signal detection core"
keywords = ["pharmacovigilance", "signal-detection"]
categories = ["science", "algorithms"]

[dependencies]
serde = { workspace = true, features = ["derive"] }
serde_json = { workspace = true }
chrono = { workspace = true }
rand = "0.9"
rayon = { workspace = true }
thiserror = { workspace = true }
nexcore-tov = { workspace = true }
nexcore-constants = { workspace = true }
nexcore-lex-primitiva = { workspace = true }
nexcore-signal-types = { workspace = true }
signal = { workspace = true }
phf = { workspace = true }
once_cell = { workspace = true }

[dev-dependencies]
tempfile = { workspace = true }

[lints]
workspace = true
"#;

    let result =
        convert_cargo_toml(crate_toml, WORKSPACE_TOML, "nexcore", &test_resolver()).unwrap();
    let doc = result.parse::<toml_edit::DocumentMut>().unwrap();

    // Package fields inlined
    assert_eq!(doc["package"]["version"].as_str(), Some("1.0.0"));
    assert_eq!(doc["package"]["edition"].as_str(), Some("2024"));
    assert_eq!(doc["package"]["rust-version"].as_str(), Some("1.85"));
    assert_eq!(
        doc["package"]["license"].as_str(),
        Some("LicenseRef-Proprietary")
    );
    assert_eq!(
        doc["package"]["repository"].as_str(),
        Some("https://github.com/nexvigilant/nexcore")
    );
    // Description preserved (not from workspace)
    assert_eq!(
        doc["package"]["description"].as_str(),
        Some("Pharmacovigilance signal detection core")
    );

    // External dep: serde should have version from workspace, features merged
    let serde = &doc["dependencies"]["serde"];
    assert_eq!(serde["version"].as_str(), Some("1.0.204"));
    let serde_features: Vec<&str> = serde["features"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap())
        .collect();
    assert!(serde_features.contains(&"derive"));

    // External dep: serde_json should be simple version string
    let serde_json = &doc["dependencies"]["serde_json"];
    assert_eq!(serde_json["version"].as_str(), Some("1.0.120"));

    // External dep: chrono should have features from workspace
    let chrono = &doc["dependencies"]["chrono"];
    assert_eq!(chrono["version"].as_str(), Some("0.4.38"));

    // Non-workspace dep: rand should be untouched
    let rand_dep = &doc["dependencies"]["rand"];
    assert_eq!(rand_dep.as_str(), Some("0.9"));

    // Internal dep: nexcore-tov should get version from resolver + registry
    let tov = &doc["dependencies"]["nexcore-tov"];
    assert_eq!(tov["version"].as_str(), Some("0.2.0"));
    assert_eq!(tov["registry"].as_str(), Some("nexcore"));
    // Must NOT have path or workspace keys
    assert!(tov.get("path").is_none());
    assert!(tov.get("workspace").is_none());

    // Internal dep: signal
    let sig = &doc["dependencies"]["signal"];
    assert_eq!(sig["version"].as_str(), Some("1.0.0"));
    assert_eq!(sig["registry"].as_str(), Some("nexcore"));

    // Dev-dep: tempfile (external)
    let tempfile = &doc["dev-dependencies"]["tempfile"];
    assert_eq!(tempfile["version"].as_str(), Some("3.10.1"));

    // Lints: should have concrete lint config
    assert!(doc["lints"].get("workspace").is_none());
    assert_eq!(
        doc["lints"]["rust"]["async_fn_in_trait"].as_str(),
        Some("allow")
    );
    assert_eq!(doc["lints"]["rust"]["dead_code"].as_str(), Some("allow"));
    assert_eq!(doc["lints"]["clippy"]["all"].as_str(), Some("allow"));
    assert_eq!(
        doc["lints"]["clippy"]["pedantic"].as_str(),
        Some("allow")
    );
}

// ---------------------------------------------------------------------------
// Pattern B: workspace deps only (package fields already concrete)
// ---------------------------------------------------------------------------
#[test]
fn pattern_b_workspace_deps_only() {
    let crate_toml = r#"[package]
name = "nexcore-lex-primitiva"
version = "0.1.0"
edition = "2024"
rust-version = "1.85"
description = "The 16 irreducible Lex Primitiva symbols"
license = "MIT"
repository = "https://github.com/nexvigilant/nexcore"

[lib]
path = "src/lib.rs"

[[bin]]
name = "lex-primitiva"
path = "src/bin/main.rs"

[dependencies]
serde = { workspace = true, features = ["derive"] }
serde_json = { workspace = true }
clap = { workspace = true, features = ["derive"] }

[dev-dependencies]
stem-math = { workspace = true }
proptest = { workspace = true }

[lints]
workspace = true
"#;

    let result =
        convert_cargo_toml(crate_toml, WORKSPACE_TOML, "nexcore", &test_resolver()).unwrap();
    let doc = result.parse::<toml_edit::DocumentMut>().unwrap();

    // Package fields unchanged
    assert_eq!(doc["package"]["version"].as_str(), Some("0.1.0"));
    assert_eq!(doc["package"]["edition"].as_str(), Some("2024"));
    assert_eq!(doc["package"]["license"].as_str(), Some("MIT"));

    // lib and bin sections preserved
    assert_eq!(doc["lib"]["path"].as_str(), Some("src/lib.rs"));
    let bins = doc["bin"].as_array_of_tables().unwrap();
    let first_bin = bins.iter().next().unwrap();
    assert_eq!(first_bin["name"].as_str(), Some("lex-primitiva"));

    // External dep: serde features from both workspace and crate-level merged
    let serde = &doc["dependencies"]["serde"];
    assert_eq!(serde["version"].as_str(), Some("1.0.204"));
    let serde_features: Vec<&str> = serde["features"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap())
        .collect();
    assert!(serde_features.contains(&"derive"));

    // External dep: clap features merged from both workspace and crate
    let clap = &doc["dependencies"]["clap"];
    assert_eq!(clap["version"].as_str(), Some("4.5.11"));
    let clap_features: Vec<&str> = clap["features"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap())
        .collect();
    // Workspace has ["derive", "cargo", "env"], crate adds ["derive"] — should be union
    assert!(clap_features.contains(&"derive"));
    assert!(clap_features.contains(&"cargo"));
    assert!(clap_features.contains(&"env"));

    // Internal dev-dep: stem-math
    let stem_math = &doc["dev-dependencies"]["stem-math"];
    assert_eq!(stem_math["version"].as_str(), Some("0.1.0"));
    assert_eq!(stem_math["registry"].as_str(), Some("nexcore"));

    // External dev-dep: proptest
    let proptest = &doc["dev-dependencies"]["proptest"];
    assert_eq!(proptest["version"].as_str(), Some("1.6.0"));
}

// ---------------------------------------------------------------------------
// Pattern C: already standalone — no changes
// ---------------------------------------------------------------------------
#[test]
fn pattern_c_already_standalone() {
    let crate_toml = r#"[package]
name = "nexcore-id"
version = "0.1.0"
edition = "2024"
rust-version = "1.85"
description = "Zero-dependency UUID implementation"

[dependencies]
serde = { version = "1.0", optional = true, default-features = false, features = ["derive"] }

[features]
default = ["std"]
std = []
serde = ["dep:serde"]

[dev-dependencies]
serde_json = "1.0"

[lints.rust]
unsafe_code = "deny"

[lints.clippy]
all = "warn"
"#;

    let result =
        convert_cargo_toml(crate_toml, WORKSPACE_TOML, "nexcore", &test_resolver()).unwrap();

    // Should be unchanged
    assert_eq!(result.trim(), crate_toml.trim());
}

// ---------------------------------------------------------------------------
// Edge case: path deps without workspace = true
// ---------------------------------------------------------------------------
#[test]
fn path_deps_without_workspace_converted_to_registry() {
    let crate_toml = r#"[package]
name = "nexcore-synth"
version = "0.1.0"
edition = "2024"
description = "Autonomous primitive synthesis engine"

[dependencies]
nexcore-lex-primitiva = { path = "../nexcore-lex-primitiva" }
nexcore-transcriptase = { path = "../nexcore-transcriptase" }
antitransformer = { path = "../antitransformer" }
nexcore-id = { path = "../nexcore-id" }
serde = { workspace = true }
serde_json = { workspace = true }
thiserror = { workspace = true }
tracing = { workspace = true }
chrono = { workspace = true }
anyhow = { workspace = true }
"#;

    let result =
        convert_cargo_toml(crate_toml, WORKSPACE_TOML, "nexcore", &test_resolver()).unwrap();
    let doc = result.parse::<toml_edit::DocumentMut>().unwrap();

    // Path deps converted to registry deps
    let lex = &doc["dependencies"]["nexcore-lex-primitiva"];
    assert_eq!(lex["version"].as_str(), Some("0.1.0"));
    assert_eq!(lex["registry"].as_str(), Some("nexcore"));
    assert!(lex.get("path").is_none());

    let trans = &doc["dependencies"]["nexcore-transcriptase"];
    assert_eq!(trans["version"].as_str(), Some("0.1.0"));
    assert_eq!(trans["registry"].as_str(), Some("nexcore"));

    let anti = &doc["dependencies"]["antitransformer"];
    assert_eq!(anti["version"].as_str(), Some("0.1.0"));
    assert_eq!(anti["registry"].as_str(), Some("nexcore"));

    let id = &doc["dependencies"]["nexcore-id"];
    assert_eq!(id["version"].as_str(), Some("0.1.0"));
    assert_eq!(id["registry"].as_str(), Some("nexcore"));

    // Workspace deps still resolved
    assert_eq!(
        doc["dependencies"]["serde"]["version"].as_str(),
        Some("1.0.204")
    );
}

// ---------------------------------------------------------------------------
// Edge case: path dep with version already specified
// ---------------------------------------------------------------------------
#[test]
fn path_dep_with_version_keeps_version() {
    let crate_toml = r#"[package]
name = "some-crate"
version = "0.1.0"
edition = "2024"

[dependencies]
stem-core = { path = "../stem-core", version = "0.1.0" }
"#;

    let result =
        convert_cargo_toml(crate_toml, WORKSPACE_TOML, "nexcore", &test_resolver()).unwrap();
    let doc = result.parse::<toml_edit::DocumentMut>().unwrap();

    let stem = &doc["dependencies"]["stem-core"];
    assert_eq!(stem["version"].as_str(), Some("0.1.0"));
    assert_eq!(stem["registry"].as_str(), Some("nexcore"));
    assert!(stem.get("path").is_none());
}

// ---------------------------------------------------------------------------
// Edge case: internal dep with features from workspace definition
// ---------------------------------------------------------------------------
#[test]
fn internal_dep_with_workspace_features() {
    // nexcore-id has features = ["serde"] in workspace.dependencies
    let crate_toml = r#"[package]
name = "some-crate"
version = "0.1.0"
edition = "2024"

[dependencies]
nexcore-id = { workspace = true }
"#;

    let result =
        convert_cargo_toml(crate_toml, WORKSPACE_TOML, "nexcore", &test_resolver()).unwrap();
    let doc = result.parse::<toml_edit::DocumentMut>().unwrap();

    let id = &doc["dependencies"]["nexcore-id"];
    assert_eq!(id["version"].as_str(), Some("0.1.0"));
    assert_eq!(id["registry"].as_str(), Some("nexcore"));
    let features: Vec<&str> = id["features"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap())
        .collect();
    assert!(features.contains(&"serde"));
}

// ---------------------------------------------------------------------------
// Edge case: workspace dep with optional = true
// ---------------------------------------------------------------------------
#[test]
fn workspace_dep_with_optional() {
    let crate_toml = r#"[package]
name = "some-crate"
version = "0.1.0"
edition = "2024"

[dependencies]
serde = { workspace = true, optional = true }
"#;

    let result =
        convert_cargo_toml(crate_toml, WORKSPACE_TOML, "nexcore", &test_resolver()).unwrap();
    let doc = result.parse::<toml_edit::DocumentMut>().unwrap();

    let serde = &doc["dependencies"]["serde"];
    assert_eq!(serde["version"].as_str(), Some("1.0.204"));
    assert_eq!(serde["optional"].as_bool(), Some(true));
    // Should have workspace features
    let features: Vec<&str> = serde["features"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap())
        .collect();
    assert!(features.contains(&"derive"));
}

// ---------------------------------------------------------------------------
// Edge case: dotted key syntax for workspace deps (e.g., nexcore-constants.workspace = true)
// ---------------------------------------------------------------------------
#[test]
fn dotted_key_workspace_dep() {
    let crate_toml = r#"[package]
name = "nexcore-tov"
version.workspace = true
edition.workspace = true
authors.workspace = true
license.workspace = true
repository.workspace = true
description = "Theory of Vigilance"

[dependencies]
serde = { workspace = true, features = ["derive"] }
thiserror = "2.0"
nexcore-constants.workspace = true
nexcore-lex-primitiva.workspace = true
typenum.workspace = true

[lints]
workspace = true
"#;

    let result =
        convert_cargo_toml(crate_toml, WORKSPACE_TOML, "nexcore", &test_resolver()).unwrap();
    let doc = result.parse::<toml_edit::DocumentMut>().unwrap();

    // Internal dep with dotted syntax should be converted
    let constants = &doc["dependencies"]["nexcore-constants"];
    assert_eq!(constants["version"].as_str(), Some("0.1.0"));
    assert_eq!(constants["registry"].as_str(), Some("nexcore"));

    // External dep with dotted syntax
    let typenum = &doc["dependencies"]["typenum"];
    assert_eq!(typenum["version"].as_str(), Some("1.17.0"));

    // Non-workspace dep preserved
    let thiserror = &doc["dependencies"]["thiserror"];
    assert_eq!(thiserror.as_str(), Some("2.0"));
}

// ---------------------------------------------------------------------------
// Edge case: workspace dep with default-features = false
// ---------------------------------------------------------------------------
#[test]
fn workspace_dep_default_features_false() {
    let crate_toml = r#"[package]
name = "some-crate"
version = "0.1.0"
edition = "2024"

[dependencies]
tokio = { workspace = true, default-features = false, features = ["rt"] }
"#;

    let result =
        convert_cargo_toml(crate_toml, WORKSPACE_TOML, "nexcore", &test_resolver()).unwrap();
    let doc = result.parse::<toml_edit::DocumentMut>().unwrap();

    let tokio = &doc["dependencies"]["tokio"];
    assert_eq!(tokio["version"].as_str(), Some("1.38.0"));
    assert_eq!(tokio["default-features"].as_bool(), Some(false));
    // When default-features = false is set at crate level, only crate-level features should be used
    let features: Vec<&str> = tokio["features"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap())
        .collect();
    assert!(features.contains(&"rt"));
}

// ---------------------------------------------------------------------------
// Edge case: build-dependencies with workspace = true
// ---------------------------------------------------------------------------
#[test]
fn build_dependencies_converted() {
    let crate_toml = r#"[package]
name = "some-crate"
version = "0.1.0"
edition = "2024"

[dependencies]
serde = { workspace = true }

[build-dependencies]
serde = { workspace = true }

[lints]
workspace = true
"#;

    let result =
        convert_cargo_toml(crate_toml, WORKSPACE_TOML, "nexcore", &test_resolver()).unwrap();
    let doc = result.parse::<toml_edit::DocumentMut>().unwrap();

    // Both regular and build deps should be resolved
    assert_eq!(
        doc["dependencies"]["serde"]["version"].as_str(),
        Some("1.0.204")
    );
    assert_eq!(
        doc["build-dependencies"]["serde"]["version"].as_str(),
        Some("1.0.204")
    );
}

// ---------------------------------------------------------------------------
// Edge case: workspace dep with package rename (lex-primitiva -> nexcore-lex-primitiva)
// ---------------------------------------------------------------------------
#[test]
fn workspace_dep_with_package_rename() {
    let crate_toml = r#"[package]
name = "some-crate"
version = "0.1.0"
edition = "2024"

[dependencies]
lex-primitiva = { workspace = true }
"#;

    let result =
        convert_cargo_toml(crate_toml, WORKSPACE_TOML, "nexcore", &test_resolver()).unwrap();
    let doc = result.parse::<toml_edit::DocumentMut>().unwrap();

    let lex = &doc["dependencies"]["lex-primitiva"];
    // The workspace def has package = "nexcore-lex-primitiva" and path = "crates/nexcore-lex-primitiva"
    // So it's internal; the resolver should look up "nexcore-lex-primitiva" (the package name)
    assert_eq!(lex["version"].as_str(), Some("0.1.0"));
    assert_eq!(lex["registry"].as_str(), Some("nexcore"));
    assert_eq!(lex["package"].as_str(), Some("nexcore-lex-primitiva"));
}

// ---------------------------------------------------------------------------
// Edge case: features merging — crate features + workspace features deduplicated
// ---------------------------------------------------------------------------
#[test]
fn features_are_deduplicated() {
    let crate_toml = r#"[package]
name = "some-crate"
version = "0.1.0"
edition = "2024"

[dependencies]
serde = { workspace = true, features = ["derive", "alloc"] }
"#;

    let result =
        convert_cargo_toml(crate_toml, WORKSPACE_TOML, "nexcore", &test_resolver()).unwrap();
    let doc = result.parse::<toml_edit::DocumentMut>().unwrap();

    let serde = &doc["dependencies"]["serde"];
    let features: Vec<&str> = serde["features"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap())
        .collect();
    // Should contain derive (from both) and alloc (from crate), no duplicates
    assert_eq!(
        features.iter().filter(|&&f| f == "derive").count(),
        1,
        "derive should appear exactly once"
    );
    assert!(features.contains(&"alloc"));
}

// ---------------------------------------------------------------------------
// Edge case: workspace dep is a simple string version (e.g., serde_json = "1.0.120")
// ---------------------------------------------------------------------------
#[test]
fn workspace_dep_simple_string_version() {
    let crate_toml = r#"[package]
name = "some-crate"
version = "0.1.0"
edition = "2024"

[dependencies]
serde_json = { workspace = true }
"#;

    let result =
        convert_cargo_toml(crate_toml, WORKSPACE_TOML, "nexcore", &test_resolver()).unwrap();
    let doc = result.parse::<toml_edit::DocumentMut>().unwrap();

    // serde_json in workspace is just "1.0.120" (string), should become version = "1.0.120"
    let sj = &doc["dependencies"]["serde_json"];
    assert_eq!(sj["version"].as_str(), Some("1.0.120"));
}

// ---------------------------------------------------------------------------
// Edge case: authors field is an array, should be properly inlined
// ---------------------------------------------------------------------------
#[test]
fn authors_array_inlined() {
    let crate_toml = r#"[package]
name = "some-crate"
version.workspace = true
edition.workspace = true
authors.workspace = true
description = "Test"
"#;

    let result =
        convert_cargo_toml(crate_toml, WORKSPACE_TOML, "nexcore", &test_resolver()).unwrap();
    let doc = result.parse::<toml_edit::DocumentMut>().unwrap();

    let authors = doc["package"]["authors"].as_array().unwrap();
    assert_eq!(
        authors.iter().next().unwrap().as_str(),
        Some("NexVigilant Team <team@nexvigilant.com>")
    );
}

// ---------------------------------------------------------------------------
// Edge case: target-specific dependencies
// ---------------------------------------------------------------------------
#[test]
fn target_specific_deps_converted() {
    let crate_toml = r#"[package]
name = "some-crate"
version = "0.1.0"
edition = "2024"

[target.'cfg(unix)'.dependencies]
signal = { workspace = true }

[target.'cfg(unix)'.dev-dependencies]
tempfile = { workspace = true }
"#;

    let result =
        convert_cargo_toml(crate_toml, WORKSPACE_TOML, "nexcore", &test_resolver()).unwrap();
    let doc = result.parse::<toml_edit::DocumentMut>().unwrap();

    let signal = &doc["target"]["cfg(unix)"]["dependencies"]["signal"];
    assert_eq!(signal["version"].as_str(), Some("1.0.0"));
    assert_eq!(signal["registry"].as_str(), Some("nexcore"));

    let tempfile = &doc["target"]["cfg(unix)"]["dev-dependencies"]["tempfile"];
    assert_eq!(tempfile["version"].as_str(), Some("3.10.1"));
}
