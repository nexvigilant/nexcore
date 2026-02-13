//! Filesystem scanner: parse Cargo.toml, count LOC and #[test] functions.
//!
//! ## Primitive Foundation
//!
//! | Primitive | Manifestation |
//! |-----------|---------------|
//! | T1: Sequence (σ) | Iterate files, accumulate counts |
//! | T1: Mapping (μ) | path → LOC/test counts |

use crate::entropy;
use crate::error::{MeasureError, MeasureResult};
use crate::graph;
use crate::types::*;
use std::path::Path;
use walkdir::WalkDir;

/// Collect a single crate's measurement.
pub fn measure_crate(workspace_root: &Path, crate_name: &str) -> MeasureResult<CrateMeasurement> {
    let crate_dir = workspace_root.join("crates").join(crate_name);
    if !crate_dir.exists() {
        return Err(MeasureError::CrateNotFound {
            name: crate_name.into(),
        });
    }
    let src_dir = crate_dir.join("src");
    let (module_locs, test_count, ops, tokens) = scan_source_dir(&src_dir)?;
    let total_loc: usize = module_locs.iter().sum();
    let module_count = module_locs.len();

    let h = entropy::shannon_entropy(&module_locs)?;
    let r = entropy::redundancy(&module_locs)?;
    let kloc = (total_loc as f64) / 1000.0;
    let td = if kloc > f64::EPSILON {
        (test_count as f64) / kloc
    } else {
        0.0
    };

    let cdi_val = if tokens > 0 {
        (ops as f64) / (tokens as f64)
    } else {
        0.0
    };

    // Graph info (fan-in/out) from workspace dep graph
    let dep_graph = graph::build_dep_graph(workspace_root)?;
    let (fi, fo) = crate_fan_in_out(&dep_graph, crate_name);

    Ok(CrateMeasurement {
        crate_id: CrateId::new(crate_name),
        timestamp: MeasureTimestamp::now(),
        loc: total_loc,
        test_count,
        module_count,
        module_loc_distribution: module_locs,
        entropy: h,
        redundancy: r,
        test_density: TestDensity::new(td),
        fan_in: fi,
        fan_out: fo,
        coupling_ratio: dep_graph
            .index
            .get(crate_name)
            .map(|&idx| dep_graph.coupling_ratio(idx))
            .unwrap_or_else(|| CouplingRatio::new(0.0)),
        cdi: CodeDensityIndex::new(cdi_val),
    })
}

/// Collect workspace-level measurement.
pub fn measure_workspace(workspace_root: &Path) -> MeasureResult<WorkspaceMeasurement> {
    let dep_graph = graph::build_dep_graph(workspace_root)?;
    let analysis = dep_graph.analyze();
    let crate_names: Vec<String> = dep_graph.names.clone();

    let mut crates = Vec::new();
    for name in &crate_names {
        match measure_crate_with_graph(workspace_root, name, &dep_graph) {
            Ok(cm) => crates.push(cm),
            Err(_) => {
                continue;
            } // Skip crates that fail (e.g. no src/)
        }
    }

    let total_loc: usize = crates.iter().map(|c| c.loc).sum();
    let total_tests: usize = crates.iter().map(|c| c.test_count).sum();

    Ok(WorkspaceMeasurement {
        timestamp: MeasureTimestamp::now(),
        crate_count: crates.len(),
        total_loc,
        total_tests,
        graph_density: analysis.density,
        max_depth: analysis.max_depth,
        scc_count: analysis.cycle_count,
        crates,
    })
}

/// Measure a crate using a pre-built dep graph (avoid rebuilding per crate).
fn measure_crate_with_graph(
    workspace_root: &Path,
    crate_name: &str,
    dep_graph: &graph::DepGraph,
) -> MeasureResult<CrateMeasurement> {
    let crate_dir = workspace_root.join("crates").join(crate_name);
    let src_dir = crate_dir.join("src");
    if !src_dir.exists() {
        return Err(MeasureError::CrateNotFound {
            name: crate_name.into(),
        });
    }

    let (module_locs, test_count, ops, tokens) = scan_source_dir(&src_dir)?;
    let total_loc: usize = module_locs.iter().sum();
    let h = entropy::shannon_entropy(&module_locs)?;
    let r = entropy::redundancy(&module_locs)?;
    let kloc = (total_loc as f64) / 1000.0;
    let td = if kloc > f64::EPSILON {
        (test_count as f64) / kloc
    } else {
        0.0
    };

    let cdi_val = if tokens > 0 {
        (ops as f64) / (tokens as f64)
    } else {
        0.0
    };

    let (fi, fo) = crate_fan_in_out(dep_graph, crate_name);

    Ok(CrateMeasurement {
        crate_id: CrateId::new(crate_name),
        timestamp: MeasureTimestamp::now(),
        loc: total_loc,
        test_count,
        module_count: module_locs.len(),
        module_loc_distribution: module_locs,
        entropy: h,
        redundancy: r,
        test_density: TestDensity::new(td),
        fan_in: fi,
        fan_out: fo,
        coupling_ratio: dep_graph
            .index
            .get(crate_name)
            .map(|&idx| dep_graph.coupling_ratio(idx))
            .unwrap_or_else(|| CouplingRatio::new(0.0)),
        cdi: CodeDensityIndex::new(cdi_val),
    })
}

/// Scan a source directory, returning (LOC per module, total test count, semantic ops, total tokens).
fn scan_source_dir(src_dir: &Path) -> MeasureResult<(Vec<usize>, usize, usize, usize)> {
    let mut module_locs = Vec::new();
    let mut total_tests = 0usize;
    let mut total_ops = 0usize;
    let mut total_tokens = 0usize;

    for entry in WalkDir::new(src_dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if !is_rust_file(path) {
            continue;
        }
        let content = std::fs::read_to_string(path).map_err(MeasureError::Io)?;
        let loc = count_loc(&content);
        let tests = count_tests(&content);
        let (ops, tokens) = count_semantic_ops(&content);

        module_locs.push(loc);
        total_tests += tests;
        total_ops += ops;
        total_tokens += tokens;
    }
    Ok((module_locs, total_tests, total_ops, total_tokens))
}

/// Check if path is a .rs file (not in target/).
fn is_rust_file(path: &Path) -> bool {
    path.extension().and_then(|e| e.to_str()) == Some("rs")
        && !path.to_str().unwrap_or("").contains("/target/")
}

/// Count non-blank, non-comment lines.
fn count_loc(content: &str) -> usize {
    content
        .lines()
        .filter(|line| {
            let trimmed = line.trim();
            !trimmed.is_empty() && !trimmed.starts_with("//")
        })
        .count()
}

/// Count `#[test]` annotations.
fn count_tests(content: &str) -> usize {
    content
        .lines()
        .filter(|line| line.trim() == "#[test]")
        .count()
}

/// Count semantic operations and total tokens.
///
/// Semantic ops: struct, enum, type, impl, trait, fn, =>, where, ?
fn count_semantic_ops(content: &str) -> (usize, usize) {
    let tokens: Vec<&str> = content.split_whitespace().collect();
    let total = tokens.len();

    let ops = tokens
        .iter()
        .filter(|&&t| {
            matches!(
                t,
                "struct" | "enum" | "type" | "impl" | "trait" | "fn" | "=>" | "where" | "?"
            ) || t.ends_with('?') // handle trailing ?
        })
        .count();

    (ops, total)
}

/// Get fan-in and fan-out for a crate from the dep graph.
fn crate_fan_in_out(graph: &graph::DepGraph, name: &str) -> (usize, usize) {
    graph
        .index
        .get(name)
        .map(|&idx| (graph.fan_in(idx), graph.fan_out(idx)))
        .unwrap_or((0, 0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn count_loc_skips_blanks_and_comments() {
        let src = "fn main() {\n    // comment\n\n    println!(\"hi\");\n}\n";
        assert_eq!(count_loc(src), 3); // fn, println, }
    }

    #[test]
    fn count_tests_finds_annotations() {
        let src = "#[test]\nfn test_a() {}\n\n#[test]\nfn test_b() {}\n";
        assert_eq!(count_tests(src), 2);
    }

    #[test]
    fn count_tests_ignores_cfg_test() {
        let src = "#[cfg(test)]\nmod tests {\n    #[test]\n    fn it() {}\n}\n";
        assert_eq!(count_tests(src), 1);
    }

    #[test]
    fn is_rust_file_works() {
        assert!(is_rust_file(Path::new("/src/lib.rs")));
        assert!(!is_rust_file(Path::new("/src/lib.py")));
        assert!(!is_rust_file(Path::new("/target/debug/lib.rs")));
    }

    #[test]
    fn measure_workspace_integration() {
        let ws = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(|p| p.parent());
        if let Some(workspace_root) = ws {
            let result = measure_workspace(workspace_root);
            // May fail in isolation (no sibling crates), but should not panic
            if let Ok(wm) = result {
                assert!(wm.crate_count > 0);
                assert!(wm.total_loc > 0);
            }
        }
    }
}
