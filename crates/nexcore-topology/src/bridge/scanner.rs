//! Workspace Cargo.toml scanner for dependency edge discovery.
//!
//! Scans a Rust workspace directory, parsing each crate's `Cargo.toml` to
//! discover internal dependency edges. "Internal" means both the dependent
//! and the dependency exist within the workspace.
//!
//! ## Usage
//!
//! ```rust,no_run
//! use nexcore_topology::bridge::scanner::scan_workspace;
//! use std::path::Path;
//!
//! let result = scan_workspace(Path::new("/home/user/nexcore")).unwrap();
//! println!("Found {} crates with {} edges", result.crates.len(), result.edges.len());
//! ```

use serde::Deserialize;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::path::Path;

/// Result of scanning a workspace for internal dependencies.
#[derive(Debug, Clone)]
pub struct WorkspaceScan {
    /// All discovered crate names.
    pub crates: BTreeSet<String>,
    /// Internal dependency edges: (from_crate, to_crate).
    pub edges: Vec<(String, String)>,
    /// Detected dependency cycles (crate pairs that depend on each other).
    pub cycles: Vec<(String, String)>,
    /// Crates that failed to parse (name, error message).
    pub parse_errors: Vec<(String, String)>,
}

/// Error from workspace scanning.
#[derive(Debug, Clone)]
pub enum ScanError {
    /// The workspace root doesn't contain a `crates/` directory.
    NoCratesDir(String),
    /// IO error reading the workspace.
    Io(String),
}

impl std::fmt::Display for ScanError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoCratesDir(path) => write!(f, "no crates/ directory at {path}"),
            Self::Io(msg) => write!(f, "IO error: {msg}"),
        }
    }
}

impl std::error::Error for ScanError {}

// ============================================================================
// Minimal Cargo.toml Deserialization
// ============================================================================

/// Minimal Cargo.toml structure — only the fields we need.
#[derive(Debug, Deserialize)]
struct CargoToml {
    package: Option<PackageDef>,
    dependencies: Option<BTreeMap<String, toml::Value>>,
}

#[derive(Debug, Deserialize)]
struct PackageDef {
    name: Option<String>,
}

/// Extract the package name from a Cargo.toml parse result.
fn package_name(cargo: &CargoToml, dir_name: &str) -> String {
    cargo
        .package
        .as_ref()
        .and_then(|p| p.name.clone())
        .unwrap_or_else(|| dir_name.to_owned())
}

/// Extract dependency names from the `[dependencies]` table.
fn dep_names(cargo: &CargoToml) -> Vec<String> {
    match &cargo.dependencies {
        Some(deps) => deps.keys().cloned().collect(),
        None => Vec::new(),
    }
}

// ============================================================================
// Public API
// ============================================================================

/// Scan a workspace directory for internal dependency edges.
///
/// Expects the workspace to have a `crates/` subdirectory containing
/// crate directories, each with a `Cargo.toml`.
///
/// # Errors
/// Returns [`ScanError::NoCratesDir`] if `workspace_root/crates/` doesn't exist.
pub fn scan_workspace(workspace_root: &Path) -> Result<WorkspaceScan, ScanError> {
    let crates_dir = workspace_root.join("crates");
    if !crates_dir.is_dir() {
        return Err(ScanError::NoCratesDir(workspace_root.display().to_string()));
    }

    // Phase 1: Discover all crate names
    let mut crate_names: BTreeSet<String> = BTreeSet::new();
    let mut crate_deps: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut parse_errors: Vec<(String, String)> = Vec::new();

    let entries = std::fs::read_dir(&crates_dir)
        .map_err(|e| ScanError::Io(format!("reading crates/: {e}")))?;

    for entry in entries {
        let entry = match entry {
            Ok(e) => e,
            Err(e) => {
                parse_errors.push(("unknown".to_owned(), e.to_string()));
                continue;
            }
        };

        let dir_path = entry.path();
        if !dir_path.is_dir() {
            continue;
        }

        let cargo_path = dir_path.join("Cargo.toml");
        if !cargo_path.is_file() {
            continue;
        }

        let dir_name = entry.file_name().to_string_lossy().to_string();

        let content = match std::fs::read_to_string(&cargo_path) {
            Ok(c) => c,
            Err(e) => {
                parse_errors.push((dir_name, e.to_string()));
                continue;
            }
        };

        let cargo: CargoToml = match toml::from_str(&content) {
            Ok(c) => c,
            Err(e) => {
                parse_errors.push((dir_name, e.to_string()));
                continue;
            }
        };

        let name = package_name(&cargo, &dir_name);
        let deps = dep_names(&cargo);
        crate_names.insert(name.clone());
        crate_deps.insert(name, deps);
    }

    // Phase 2: Filter to internal edges only
    let mut edges: Vec<(String, String)> = Vec::new();
    for (from, deps) in &crate_deps {
        for dep in deps {
            if crate_names.contains(dep) {
                edges.push((from.clone(), dep.clone()));
            }
        }
    }

    // Phase 3: Detect cycles (mutual dependencies)
    let mut cycles: Vec<(String, String)> = Vec::new();
    let edge_set: BTreeSet<(&str, &str)> = edges
        .iter()
        .map(|(a, b)| (a.as_str(), b.as_str()))
        .collect();

    for (a, b) in &edge_set {
        if a < b && edge_set.contains(&(b, a)) {
            cycles.push(((*a).to_owned(), (*b).to_owned()));
        }
    }

    Ok(WorkspaceScan {
        crates: crate_names,
        edges,
        cycles,
        parse_errors,
    })
}
