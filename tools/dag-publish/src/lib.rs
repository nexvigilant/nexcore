use std::collections::{HashMap, HashSet, VecDeque};
use std::fs;
use std::path::Path;

use anyhow::{Context, Result, bail};

/// Build a dependency graph from crate directories.
///
/// Scans each subdirectory of `crates_dir` for a Cargo.toml, extracts the
/// package name, and identifies internal dependencies. A dependency is
/// considered internal if any of the following holds:
///
/// 1. It has `registry = "<registry_name>"` (already converted to standalone).
/// 2. It has `path = "../..."` (direct path dep to a sibling crate).
/// 3. It has `workspace = true` and the workspace-level entry has a `path`
///    field (workspace-inherited internal dep).
///
/// `[dev-dependencies]` are excluded because they are not needed at publish time.
///
/// Returns a list of `(package_name, vec_of_internal_dep_package_names)`.
pub fn build_dag(crates_dir: &Path, registry_name: &str) -> Result<Vec<(String, Vec<String>)>> {
    // Step 1: Read the workspace root Cargo.toml to understand workspace deps.
    // The workspace root is two levels up from crates_dir (crates_dir = <root>/crates).
    let workspace_root = crates_dir
        .parent()
        .context("crates_dir has no parent directory")?;
    let workspace_toml_path = workspace_root.join("Cargo.toml");

    let workspace_internal_deps = if workspace_toml_path.exists() {
        parse_workspace_internal_deps(&workspace_toml_path)?
    } else {
        HashMap::new()
    };

    // Step 2: Scan all crate directories and collect their package names.
    let crate_dirs = collect_crate_dirs(crates_dir)?;

    // Build a mapping from directory name to package name, and collect all known
    // package names so we can validate dependency references.
    let mut dir_to_package: HashMap<String, String> = HashMap::new();
    let mut all_package_names: HashSet<String> = HashSet::new();

    for dir in &crate_dirs {
        let cargo_path = crates_dir.join(dir).join("Cargo.toml");
        if !cargo_path.exists() {
            continue;
        }
        let contents = fs::read_to_string(&cargo_path)
            .with_context(|| format!("Failed to read {}", cargo_path.display()))?;
        let doc: toml::Value = contents
            .parse()
            .with_context(|| format!("Failed to parse {}", cargo_path.display()))?;

        let package_name = extract_package_name(&doc, dir);
        all_package_names.insert(package_name.clone());
        dir_to_package.insert(dir.clone(), package_name);
    }

    // Step 3: For each crate, extract internal dependencies.
    let mut result: Vec<(String, Vec<String>)> = Vec::new();

    for dir in &crate_dirs {
        let cargo_path = crates_dir.join(dir).join("Cargo.toml");
        if !cargo_path.exists() {
            continue;
        }
        let package_name = match dir_to_package.get(dir) {
            Some(name) => name.clone(),
            None => continue,
        };

        let contents = fs::read_to_string(&cargo_path)?;
        let doc: toml::Value = contents.parse()?;

        let internal_deps = extract_internal_deps(
            &doc,
            registry_name,
            &workspace_internal_deps,
            &all_package_names,
            &dir_to_package,
        );

        // Warn about deps that reference unknown crates (might be already published).
        for dep in &internal_deps {
            if !all_package_names.contains(dep) {
                eprintln!(
                    "warning: {package_name} depends on '{dep}' which is not found in {}, \
                     assuming it is already published",
                    crates_dir.display()
                );
            }
        }

        // Only keep deps that are in our set of known crates.
        let known_deps: Vec<String> = internal_deps
            .into_iter()
            .filter(|d| all_package_names.contains(d))
            .collect();

        result.push((package_name, known_deps));
    }

    Ok(result)
}

/// Topological sort using Kahn's algorithm.
///
/// Returns crate names in publish order (dependencies first).
/// Returns an error if a cycle is detected, including the crates involved.
pub fn topological_sort(crates: &[(String, Vec<String>)]) -> Result<Vec<String>> {
    // Build adjacency list and in-degree map.
    let mut in_degree: HashMap<&str, usize> = HashMap::new();
    let mut dependents: HashMap<&str, Vec<&str>> = HashMap::new();

    // Initialize all crates with in-degree 0.
    for (name, _) in crates {
        in_degree.entry(name.as_str()).or_insert(0);
        dependents.entry(name.as_str()).or_default();
    }

    // Build the set of known crate names for filtering.
    let known: HashSet<&str> = crates.iter().map(|(name, _)| name.as_str()).collect();

    // Populate edges: for each dep, the dependent crate must wait.
    for (name, deps) in crates {
        for dep in deps {
            if !known.contains(dep.as_str()) {
                // Skip unknown deps (already published externally).
                continue;
            }
            dependents.entry(dep.as_str()).or_default().push(name.as_str());
            *in_degree.entry(name.as_str()).or_insert(0) += 1;
        }
    }

    // Start with all crates that have zero in-degree.
    let mut queue: VecDeque<&str> = VecDeque::new();
    for (name, degree) in &in_degree {
        if *degree == 0 {
            queue.push_back(name);
        }
    }

    // Sort the initial queue for deterministic output.
    let mut sorted_queue: Vec<&str> = queue.into_iter().collect();
    sorted_queue.sort();
    let mut queue: VecDeque<&str> = sorted_queue.into_iter().collect();

    let mut order: Vec<String> = Vec::new();

    while let Some(current) = queue.pop_front() {
        order.push(current.to_string());

        // Collect and sort dependents for deterministic output.
        let mut next_candidates: Vec<&str> = Vec::new();
        if let Some(deps) = dependents.get(current) {
            for &dependent in deps {
                let degree = in_degree.get_mut(dependent).expect("in_degree missing");
                *degree -= 1;
                if *degree == 0 {
                    next_candidates.push(dependent);
                }
            }
        }
        next_candidates.sort();
        for candidate in next_candidates {
            queue.push_back(candidate);
        }
    }

    // If we didn't process all crates, there is a cycle.
    if order.len() != crates.len() {
        let in_cycle: Vec<String> = in_degree
            .iter()
            .filter(|(_, degree)| **degree > 0)
            .map(|(name, _)| name.to_string())
            .collect();
        let mut cycle_sorted = in_cycle;
        cycle_sorted.sort();
        bail!(
            "Circular dependency detected among: {}",
            cycle_sorted.join(", ")
        );
    }

    Ok(order)
}

/// Group crates into parallelizable phases.
///
/// Each phase contains crates whose dependencies are all satisfied by
/// earlier phases. Phase 0 contains crates with no internal dependencies.
pub fn group_into_phases(crates: &[(String, Vec<String>)]) -> Result<Vec<Vec<String>>> {
    let known: HashSet<&str> = crates.iter().map(|(name, _)| name.as_str()).collect();

    // Build in-degree map and adjacency list (same as topological_sort).
    let mut in_degree: HashMap<&str, usize> = HashMap::new();
    let mut dependents: HashMap<&str, Vec<&str>> = HashMap::new();

    for (name, _) in crates {
        in_degree.entry(name.as_str()).or_insert(0);
        dependents.entry(name.as_str()).or_default();
    }

    for (name, deps) in crates {
        for dep in deps {
            if !known.contains(dep.as_str()) {
                continue;
            }
            dependents.entry(dep.as_str()).or_default().push(name.as_str());
            *in_degree.entry(name.as_str()).or_insert(0) += 1;
        }
    }

    let mut phases: Vec<Vec<String>> = Vec::new();
    let mut remaining = in_degree.len();

    // BFS by levels: each level is one phase.
    loop {
        let mut phase: Vec<String> = in_degree
            .iter()
            .filter(|(_, degree)| **degree == 0)
            .map(|(name, _)| name.to_string())
            .collect();

        if phase.is_empty() {
            break;
        }

        phase.sort();

        // Remove these crates from the graph and decrement in-degrees.
        for name in &phase {
            if let Some(deps) = dependents.get(name.as_str()) {
                for &dependent in deps {
                    if let Some(degree) = in_degree.get_mut(dependent) {
                        *degree -= 1;
                    }
                }
            }
        }

        for name in &phase {
            in_degree.remove(name.as_str());
        }

        remaining -= phase.len();
        phases.push(phase);
    }

    if remaining > 0 {
        let in_cycle: Vec<String> = in_degree
            .keys()
            .map(|name| name.to_string())
            .collect();
        let mut cycle_sorted = in_cycle;
        cycle_sorted.sort();
        bail!(
            "Circular dependency detected among: {}",
            cycle_sorted.join(", ")
        );
    }

    Ok(phases)
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Collect sorted subdirectory names under `crates_dir`.
fn collect_crate_dirs(crates_dir: &Path) -> Result<Vec<String>> {
    let mut dirs: Vec<String> = Vec::new();
    let entries = fs::read_dir(crates_dir)
        .with_context(|| format!("Failed to read directory: {}", crates_dir.display()))?;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() && path.join("Cargo.toml").exists() {
            if let Some(name) = entry.file_name().to_str() {
                dirs.push(name.to_string());
            }
        }
    }

    dirs.sort();
    Ok(dirs)
}

/// Parse the workspace Cargo.toml and return a map of dependency-key to the
/// actual package name for all internal deps (those with a `path` field).
///
/// For example, `lex-primitiva = { path = "crates/nexcore-lex-primitiva", package = "nexcore-lex-primitiva" }`
/// yields `"lex-primitiva" -> "nexcore-lex-primitiva"`.
///
/// If there is no `package` field, the key itself is the package name:
/// `nexcore-id = { path = "crates/nexcore-id" }` yields `"nexcore-id" -> "nexcore-id"`.
fn parse_workspace_internal_deps(
    workspace_toml_path: &Path,
) -> Result<HashMap<String, String>> {
    let contents = fs::read_to_string(workspace_toml_path)
        .with_context(|| format!("Failed to read {}", workspace_toml_path.display()))?;
    let doc: toml::Value = contents
        .parse()
        .with_context(|| format!("Failed to parse {}", workspace_toml_path.display()))?;

    let mut internal_deps: HashMap<String, String> = HashMap::new();

    let ws_deps = match doc
        .get("workspace")
        .and_then(|w| w.get("dependencies"))
        .and_then(|d| d.as_table())
    {
        Some(t) => t,
        None => return Ok(internal_deps),
    };

    for (key, value) in ws_deps {
        let table = match value.as_table() {
            Some(t) => t,
            None => continue, // Simple string version = external dep
        };

        // Internal deps have a `path` field.
        if table.get("path").is_none() {
            continue;
        }

        // The actual package name is either the `package` field or the key itself.
        let package_name = table
            .get("package")
            .and_then(|v| v.as_str())
            .unwrap_or(key);

        internal_deps.insert(key.clone(), package_name.to_string());
    }

    Ok(internal_deps)
}

/// Extract the package name from a parsed Cargo.toml.
/// Falls back to the directory name if no [package].name is found.
fn extract_package_name(doc: &toml::Value, dir_name: &str) -> String {
    doc.get("package")
        .and_then(|p| p.get("name"))
        .and_then(|n| n.as_str())
        .unwrap_or(dir_name)
        .to_string()
}

/// Extract the list of internal dependency package names from a crate's Cargo.toml.
///
/// Checks `[dependencies]` and `[build-dependencies]` only (NOT `[dev-dependencies]`).
///
/// A dependency is internal if:
/// 1. It has `registry = "<registry_name>"` (already converted).
/// 2. It has `path = "../..."` (direct path dep).
/// 3. It has `workspace = true` and the workspace-level entry is in `workspace_internal_deps`.
fn extract_internal_deps(
    doc: &toml::Value,
    registry_name: &str,
    workspace_internal_deps: &HashMap<String, String>,
    all_package_names: &HashSet<String>,
    dir_to_package: &HashMap<String, String>,
) -> Vec<String> {
    let mut deps: Vec<String> = Vec::new();

    // Process [dependencies] and [build-dependencies] only.
    for section in &["dependencies", "build-dependencies"] {
        let section_table = match doc.get(section).and_then(|s| s.as_table()) {
            Some(t) => t,
            None => continue,
        };

        for (key, value) in section_table {
            if let Some(dep_name) = resolve_internal_dep(
                key,
                value,
                registry_name,
                workspace_internal_deps,
                all_package_names,
                dir_to_package,
            ) {
                if !deps.contains(&dep_name) {
                    deps.push(dep_name);
                }
            }
        }
    }

    // Process target-specific dependencies (but not dev-dependencies).
    if let Some(target) = doc.get("target").and_then(|t| t.as_table()) {
        for (_target_triple, target_value) in target {
            for section in &["dependencies", "build-dependencies"] {
                let section_table = match target_value.get(section).and_then(|s| s.as_table()) {
                    Some(t) => t,
                    None => continue,
                };

                for (key, value) in section_table {
                    if let Some(dep_name) = resolve_internal_dep(
                        key,
                        value,
                        registry_name,
                        workspace_internal_deps,
                        all_package_names,
                        dir_to_package,
                    ) {
                        if !deps.contains(&dep_name) {
                            deps.push(dep_name);
                        }
                    }
                }
            }
        }
    }

    deps.sort();
    deps
}

/// Determine if a single dependency entry refers to an internal crate.
/// Returns `Some(package_name)` if internal, `None` if external.
fn resolve_internal_dep(
    key: &str,
    value: &toml::Value,
    registry_name: &str,
    workspace_internal_deps: &HashMap<String, String>,
    all_package_names: &HashSet<String>,
    dir_to_package: &HashMap<String, String>,
) -> Option<String> {
    let table = value.as_table()?;

    // Case 1: Already-converted dep with `registry = "nexcore"`.
    if let Some(reg) = table.get("registry").and_then(|v| v.as_str()) {
        if reg == registry_name {
            // The package name is either the `package` field or the key.
            let package_name = table
                .get("package")
                .and_then(|v| v.as_str())
                .unwrap_or(key);
            return Some(package_name.to_string());
        }
    }

    // Case 2: Direct path dep (`path = "../some-crate"`).
    if let Some(path_str) = table.get("path").and_then(|v| v.as_str()) {
        // The package name is either the `package` field, or we extract it from
        // the path, or we look it up by directory name.
        if let Some(pkg) = table.get("package").and_then(|v| v.as_str()) {
            return Some(pkg.to_string());
        }
        // Extract directory name from path (e.g., "../nexcore-tov" -> "nexcore-tov").
        let dir_name = Path::new(path_str)
            .file_name()
            .and_then(|f| f.to_str())
            .unwrap_or(key);
        // Look up the actual package name for this directory.
        if let Some(package_name) = dir_to_package.get(dir_name) {
            return Some(package_name.clone());
        }
        // Fallback: the directory name might be the package name itself.
        if all_package_names.contains(dir_name) {
            return Some(dir_name.to_string());
        }
        return Some(dir_name.to_string());
    }

    // Case 3: Workspace-inherited dep (`workspace = true`).
    let is_workspace = table
        .get("workspace")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    if is_workspace {
        // Check if this dependency key maps to an internal workspace dep.
        if let Some(package_name) = workspace_internal_deps.get(key) {
            return Some(package_name.clone());
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    // -----------------------------------------------------------------------
    // topological_sort tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_topo_sort_basic_chain() {
        // a -> b -> c (c must publish first)
        let crates = vec![
            ("a".to_string(), vec!["b".to_string()]),
            ("b".to_string(), vec!["c".to_string()]),
            ("c".to_string(), vec![]),
        ];

        let order = topological_sort(&crates).unwrap();
        assert_eq!(order, vec!["c", "b", "a"]);
    }

    #[test]
    fn test_topo_sort_diamond() {
        // d depends on b and c; b and c depend on a
        let crates = vec![
            ("a".to_string(), vec![]),
            ("b".to_string(), vec!["a".to_string()]),
            ("c".to_string(), vec!["a".to_string()]),
            ("d".to_string(), vec!["b".to_string(), "c".to_string()]),
        ];

        let order = topological_sort(&crates).unwrap();

        // a must come before b and c; b and c must come before d.
        let pos = |name: &str| order.iter().position(|n| n == name).unwrap();
        assert!(pos("a") < pos("b"));
        assert!(pos("a") < pos("c"));
        assert!(pos("b") < pos("d"));
        assert!(pos("c") < pos("d"));
    }

    #[test]
    fn test_topo_sort_independent_crates() {
        let crates = vec![
            ("alpha".to_string(), vec![]),
            ("beta".to_string(), vec![]),
            ("gamma".to_string(), vec![]),
        ];

        let order = topological_sort(&crates).unwrap();
        // All independent, should be sorted alphabetically (deterministic).
        assert_eq!(order, vec!["alpha", "beta", "gamma"]);
    }

    #[test]
    fn test_topo_sort_circular_dependency() {
        let crates = vec![
            ("a".to_string(), vec!["b".to_string()]),
            ("b".to_string(), vec!["c".to_string()]),
            ("c".to_string(), vec!["a".to_string()]),
        ];

        let err = topological_sort(&crates).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("Circular dependency"), "Got: {msg}");
        assert!(msg.contains("a"));
        assert!(msg.contains("b"));
        assert!(msg.contains("c"));
    }

    #[test]
    fn test_topo_sort_self_cycle() {
        let crates = vec![("a".to_string(), vec!["a".to_string()])];

        let err = topological_sort(&crates).unwrap_err();
        assert!(err.to_string().contains("Circular dependency"));
    }

    #[test]
    fn test_topo_sort_unknown_deps_ignored() {
        // "b" depends on "external" which is not in the crate list.
        let crates = vec![
            ("a".to_string(), vec!["b".to_string()]),
            ("b".to_string(), vec!["external".to_string()]),
        ];

        let order = topological_sort(&crates).unwrap();
        assert_eq!(order, vec!["b", "a"]);
    }

    #[test]
    fn test_topo_sort_single_crate() {
        let crates = vec![("solo".to_string(), vec![])];
        let order = topological_sort(&crates).unwrap();
        assert_eq!(order, vec!["solo"]);
    }

    #[test]
    fn test_topo_sort_empty() {
        let crates: Vec<(String, Vec<String>)> = vec![];
        let order = topological_sort(&crates).unwrap();
        assert!(order.is_empty());
    }

    // -----------------------------------------------------------------------
    // group_into_phases tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_phases_basic_chain() {
        let crates = vec![
            ("a".to_string(), vec!["b".to_string()]),
            ("b".to_string(), vec!["c".to_string()]),
            ("c".to_string(), vec![]),
        ];

        let phases = group_into_phases(&crates).unwrap();
        assert_eq!(phases.len(), 3);
        assert_eq!(phases[0], vec!["c"]);
        assert_eq!(phases[1], vec!["b"]);
        assert_eq!(phases[2], vec!["a"]);
    }

    #[test]
    fn test_phases_diamond() {
        let crates = vec![
            ("a".to_string(), vec![]),
            ("b".to_string(), vec!["a".to_string()]),
            ("c".to_string(), vec!["a".to_string()]),
            ("d".to_string(), vec!["b".to_string(), "c".to_string()]),
        ];

        let phases = group_into_phases(&crates).unwrap();
        assert_eq!(phases.len(), 3);
        assert_eq!(phases[0], vec!["a"]);
        assert_eq!(phases[1], vec!["b", "c"]); // b and c can be parallel
        assert_eq!(phases[2], vec!["d"]);
    }

    #[test]
    fn test_phases_all_independent() {
        let crates = vec![
            ("x".to_string(), vec![]),
            ("y".to_string(), vec![]),
            ("z".to_string(), vec![]),
        ];

        let phases = group_into_phases(&crates).unwrap();
        assert_eq!(phases.len(), 1);
        assert_eq!(phases[0], vec!["x", "y", "z"]);
    }

    #[test]
    fn test_phases_circular_dependency() {
        let crates = vec![
            ("a".to_string(), vec!["b".to_string()]),
            ("b".to_string(), vec!["a".to_string()]),
        ];

        let err = group_into_phases(&crates).unwrap_err();
        assert!(err.to_string().contains("Circular dependency"));
    }

    #[test]
    fn test_phases_complex_graph() {
        // Phase 0: e, f (no deps)
        // Phase 1: c (depends on e), d (depends on f)
        // Phase 2: b (depends on c, d)
        // Phase 3: a (depends on b)
        let crates = vec![
            ("a".to_string(), vec!["b".to_string()]),
            ("b".to_string(), vec!["c".to_string(), "d".to_string()]),
            ("c".to_string(), vec!["e".to_string()]),
            ("d".to_string(), vec!["f".to_string()]),
            ("e".to_string(), vec![]),
            ("f".to_string(), vec![]),
        ];

        let phases = group_into_phases(&crates).unwrap();
        assert_eq!(phases.len(), 4);
        assert_eq!(phases[0], vec!["e", "f"]);
        assert_eq!(phases[1], vec!["c", "d"]);
        assert_eq!(phases[2], vec!["b"]);
        assert_eq!(phases[3], vec!["a"]);
    }

    #[test]
    fn test_phases_empty() {
        let crates: Vec<(String, Vec<String>)> = vec![];
        let phases = group_into_phases(&crates).unwrap();
        assert!(phases.is_empty());
    }

    // -----------------------------------------------------------------------
    // build_dag tests (using temp directories)
    // -----------------------------------------------------------------------

    /// Helper to create a minimal crate directory with a Cargo.toml.
    fn create_crate(crates_dir: &Path, name: &str, cargo_toml: &str) {
        let crate_dir = crates_dir.join(name);
        fs::create_dir_all(&crate_dir).unwrap();
        fs::write(crate_dir.join("Cargo.toml"), cargo_toml).unwrap();
    }

    /// Helper to create a workspace root Cargo.toml.
    fn create_workspace_toml(root: &Path, content: &str) {
        fs::write(root.join("Cargo.toml"), content).unwrap();
    }

    #[test]
    fn test_build_dag_registry_deps() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let crates_dir = root.join("crates");
        fs::create_dir_all(&crates_dir).unwrap();

        create_workspace_toml(root, "[workspace]\nmembers = [\"crates/*\"]\n");

        create_crate(
            &crates_dir,
            "core",
            r#"
[package]
name = "core"

[dependencies]
"#,
        );

        create_crate(
            &crates_dir,
            "app",
            r#"
[package]
name = "app"

[dependencies]
core = { version = "1.0", registry = "nexcore" }
serde = "1.0"
"#,
        );

        let dag = build_dag(&crates_dir, "nexcore").unwrap();
        let app_entry = dag.iter().find(|(name, _)| name == "app").unwrap();
        assert_eq!(app_entry.1, vec!["core"]);

        let core_entry = dag.iter().find(|(name, _)| name == "core").unwrap();
        assert!(core_entry.1.is_empty());
    }

    #[test]
    fn test_build_dag_path_deps() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let crates_dir = root.join("crates");
        fs::create_dir_all(&crates_dir).unwrap();

        create_workspace_toml(root, "[workspace]\nmembers = [\"crates/*\"]\n");

        create_crate(
            &crates_dir,
            "utils",
            r#"
[package]
name = "utils"

[dependencies]
"#,
        );

        create_crate(
            &crates_dir,
            "service",
            r#"
[package]
name = "service"

[dependencies]
utils = { path = "../utils" }
reqwest = "0.12"
"#,
        );

        let dag = build_dag(&crates_dir, "nexcore").unwrap();
        let service_entry = dag.iter().find(|(name, _)| name == "service").unwrap();
        assert_eq!(service_entry.1, vec!["utils"]);
    }

    #[test]
    fn test_build_dag_workspace_deps() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let crates_dir = root.join("crates");
        fs::create_dir_all(&crates_dir).unwrap();

        create_workspace_toml(
            root,
            r#"
[workspace]
members = ["crates/*"]

[workspace.dependencies]
my-core = { path = "crates/my-core" }
serde = "1.0"
"#,
        );

        create_crate(
            &crates_dir,
            "my-core",
            r#"
[package]
name = "my-core"

[dependencies]
"#,
        );

        create_crate(
            &crates_dir,
            "my-app",
            r#"
[package]
name = "my-app"

[dependencies]
my-core = { workspace = true }
serde = { workspace = true }
"#,
        );

        let dag = build_dag(&crates_dir, "nexcore").unwrap();
        let app_entry = dag.iter().find(|(name, _)| name == "my-app").unwrap();
        assert_eq!(app_entry.1, vec!["my-core"]);
    }

    #[test]
    fn test_build_dag_excludes_dev_deps() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let crates_dir = root.join("crates");
        fs::create_dir_all(&crates_dir).unwrap();

        create_workspace_toml(root, "[workspace]\nmembers = [\"crates/*\"]\n");

        create_crate(
            &crates_dir,
            "base",
            r#"
[package]
name = "base"

[dependencies]
"#,
        );

        create_crate(
            &crates_dir,
            "tester",
            r#"
[package]
name = "tester"

[dependencies]

[dev-dependencies]
base = { path = "../base" }
"#,
        );

        let dag = build_dag(&crates_dir, "nexcore").unwrap();
        let tester_entry = dag.iter().find(|(name, _)| name == "tester").unwrap();
        // dev-deps should NOT appear in the DAG.
        assert!(tester_entry.1.is_empty());
    }

    #[test]
    fn test_build_dag_package_rename() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let crates_dir = root.join("crates");
        fs::create_dir_all(&crates_dir).unwrap();

        create_workspace_toml(
            root,
            r#"
[workspace]
members = ["crates/*"]

[workspace.dependencies]
my-alias = { path = "crates/actual-name", package = "actual-name" }
"#,
        );

        create_crate(
            &crates_dir,
            "actual-name",
            r#"
[package]
name = "actual-name"

[dependencies]
"#,
        );

        create_crate(
            &crates_dir,
            "consumer",
            r#"
[package]
name = "consumer"

[dependencies]
my-alias = { workspace = true }
"#,
        );

        let dag = build_dag(&crates_dir, "nexcore").unwrap();
        let consumer = dag.iter().find(|(name, _)| name == "consumer").unwrap();
        // Should use the actual package name, not the alias.
        assert_eq!(consumer.1, vec!["actual-name"]);
    }

    #[test]
    fn test_build_dag_mixed_dep_styles() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let crates_dir = root.join("crates");
        fs::create_dir_all(&crates_dir).unwrap();

        create_workspace_toml(
            root,
            r#"
[workspace]
members = ["crates/*"]

[workspace.dependencies]
lib-a = { path = "crates/lib-a" }
serde = "1.0"
"#,
        );

        create_crate(&crates_dir, "lib-a", "[package]\nname = \"lib-a\"\n");
        create_crate(&crates_dir, "lib-b", "[package]\nname = \"lib-b\"\n");
        create_crate(&crates_dir, "lib-c", "[package]\nname = \"lib-c\"\n");

        create_crate(
            &crates_dir,
            "app",
            r#"
[package]
name = "app"

[dependencies]
lib-a = { workspace = true }
lib-b = { path = "../lib-b" }
lib-c = { version = "1.0", registry = "nexcore" }
serde = { workspace = true }
"#,
        );

        let dag = build_dag(&crates_dir, "nexcore").unwrap();
        let app_entry = dag.iter().find(|(name, _)| name == "app").unwrap();
        let mut deps = app_entry.1.clone();
        deps.sort();
        assert_eq!(deps, vec!["lib-a", "lib-b", "lib-c"]);
    }

    #[test]
    fn test_build_dag_path_dep_with_package_rename() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let crates_dir = root.join("crates");
        fs::create_dir_all(&crates_dir).unwrap();

        create_workspace_toml(root, "[workspace]\nmembers = [\"crates/*\"]\n");

        create_crate(
            &crates_dir,
            "real-name",
            r#"
[package]
name = "real-name"

[dependencies]
"#,
        );

        create_crate(
            &crates_dir,
            "user",
            r#"
[package]
name = "user"

[dependencies]
alias = { path = "../real-name", package = "real-name" }
"#,
        );

        let dag = build_dag(&crates_dir, "nexcore").unwrap();
        let user_entry = dag.iter().find(|(name, _)| name == "user").unwrap();
        assert_eq!(user_entry.1, vec!["real-name"]);
    }

    #[test]
    fn test_build_dag_no_crates() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let crates_dir = root.join("crates");
        fs::create_dir_all(&crates_dir).unwrap();

        create_workspace_toml(root, "[workspace]\nmembers = [\"crates/*\"]\n");

        let dag = build_dag(&crates_dir, "nexcore").unwrap();
        assert!(dag.is_empty());
    }
}
