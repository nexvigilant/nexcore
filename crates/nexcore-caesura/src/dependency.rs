//! Dependency caesura detection — Cargo.toml diff analysis.
//!
//! Parses Cargo.toml files and detects dependency pattern shifts:
//! clusters of new deps, semver major bumps, mixed workspace/path/git sources.

use crate::metrics::DepMetrics;
use crate::types::{Caesura, CaesuraScore, CaesuraType, Stratum, StratumLocation};
use std::path::Path;

/// Dependency caesura detector.
///
/// Tier: T3 — Full domain type
pub struct DepDetector {
    /// Number of new deps in a cluster to trigger a caesura.
    pub cluster_threshold: usize,
    /// Ratio of non-workspace deps to total that triggers concern.
    pub non_workspace_ratio_threshold: f64,
}

impl Default for DepDetector {
    fn default() -> Self {
        Self {
            cluster_threshold: 5,
            non_workspace_ratio_threshold: 0.5,
        }
    }
}

impl DepDetector {
    /// Parse dependency metrics from Cargo.toml content.
    pub fn compute_metrics(cargo_toml: &str) -> DepMetrics {
        let mut dep_count = 0usize;
        let mut workspace_deps = 0usize;
        let mut versioned_deps = 0usize;
        let mut path_deps = 0usize;
        let mut git_deps = 0usize;

        let mut in_deps = false;

        for line in cargo_toml.lines() {
            let trimmed = line.trim();

            // Track section headers
            if trimmed.starts_with('[') {
                in_deps = trimmed == "[dependencies]"
                    || trimmed == "[dev-dependencies]"
                    || trimmed == "[build-dependencies]"
                    || trimmed.starts_with("[dependencies.")
                    || trimmed.starts_with("[dev-dependencies.")
                    || trimmed.starts_with("[build-dependencies.");
                continue;
            }

            if !in_deps {
                continue;
            }

            // Skip empty lines and comments
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            // Each non-empty line in a deps section is a dependency
            if trimmed.contains('=') {
                dep_count += 1;

                if trimmed.contains("workspace = true") || trimmed.contains("workspace=true") {
                    workspace_deps += 1;
                }
                if trimmed.contains("version") && !trimmed.contains("workspace") {
                    versioned_deps += 1;
                }
                if trimmed.contains("path") {
                    path_deps += 1;
                }
                if trimmed.contains("git") {
                    git_deps += 1;
                }
            }
        }

        DepMetrics {
            dep_count,
            workspace_deps,
            versioned_deps,
            path_deps,
            git_deps,
        }
    }

    /// Detect dependency caesuras from metrics of two Cargo.toml versions,
    /// or from a single Cargo.toml where the dep composition is unusual.
    pub fn detect_single(&self, path: &str, metrics: &DepMetrics) -> Vec<Caesura> {
        let mut caesuras = Vec::new();

        if metrics.dep_count == 0 {
            return caesuras;
        }

        // Check for high non-workspace ratio (mixed sourcing)
        let non_ws = metrics.dep_count.saturating_sub(metrics.workspace_deps);
        let ratio = non_ws as f64 / metrics.dep_count as f64;

        if ratio > self.non_workspace_ratio_threshold && non_ws >= self.cluster_threshold {
            let score = (ratio * 0.8).min(1.0);
            caesuras.push(Caesura {
                caesura_type: CaesuraType::Dependency,
                stratum: Stratum::Dependency,
                score: CaesuraScore::new(score),
                location: StratumLocation {
                    path: path.to_string(),
                    line_range: None,
                    commit_sha: None,
                },
                description: format!(
                    "High non-workspace dep ratio: {non_ws}/{} ({ratio:.0}%) — suggests dep cluster from different era",
                    metrics.dep_count
                ),
                boundary_files: vec![path.to_string()],
            });
        }

        // Check for git deps (unusual in workspace context)
        if metrics.git_deps > 0 {
            let score = (metrics.git_deps as f64 * 0.2).min(0.8);
            caesuras.push(Caesura {
                caesura_type: CaesuraType::Dependency,
                stratum: Stratum::Dependency,
                score: CaesuraScore::new(score),
                location: StratumLocation {
                    path: path.to_string(),
                    line_range: None,
                    commit_sha: None,
                },
                description: format!(
                    "{} git dependencies detected — possible pre-publication or forked deps",
                    metrics.git_deps
                ),
                boundary_files: vec![path.to_string()],
            });
        }

        caesuras
    }

    /// Compare two Cargo.toml snapshots and detect dependency caesuras.
    pub fn detect_diff(&self, path: &str, before: &DepMetrics, after: &DepMetrics) -> Vec<Caesura> {
        let mut caesuras = Vec::new();

        let added = after.dep_count.saturating_sub(before.dep_count);
        let removed = before.dep_count.saturating_sub(after.dep_count);

        // Large cluster of new deps
        if added >= self.cluster_threshold {
            let score = (added as f64 / 10.0).min(1.0);
            caesuras.push(Caesura {
                caesura_type: CaesuraType::Dependency,
                stratum: Stratum::Dependency,
                score: CaesuraScore::new(score),
                location: StratumLocation {
                    path: path.to_string(),
                    line_range: None,
                    commit_sha: None,
                },
                description: format!("{added} new dependencies added in cluster"),
                boundary_files: vec![path.to_string()],
            });
        }

        // Large cluster of removed deps
        if removed >= self.cluster_threshold {
            let score = (removed as f64 / 10.0).min(1.0);
            caesuras.push(Caesura {
                caesura_type: CaesuraType::Dependency,
                stratum: Stratum::Dependency,
                score: CaesuraScore::new(score),
                location: StratumLocation {
                    path: path.to_string(),
                    line_range: None,
                    commit_sha: None,
                },
                description: format!("{removed} dependencies removed in cluster"),
                boundary_files: vec![path.to_string()],
            });
        }

        caesuras
    }

    /// Scan a directory tree for Cargo.toml files and detect dependency caesuras.
    pub fn scan_directory(&self, dir: &Path) -> Result<Vec<Caesura>, std::io::Error> {
        let mut caesuras = Vec::new();
        scan_cargo_tomls(dir, &mut |path| {
            let contents = std::fs::read_to_string(path)?;
            let metrics = Self::compute_metrics(&contents);
            let path_str = path.display().to_string();
            caesuras.extend(self.detect_single(&path_str, &metrics));
            Ok(())
        })?;
        Ok(caesuras)
    }
}

/// Walk directory tree and call `f` for each Cargo.toml found.
fn scan_cargo_tomls(
    dir: &Path,
    f: &mut dyn FnMut(&Path) -> Result<(), std::io::Error>,
) -> Result<(), std::io::Error> {
    if !dir.is_dir() {
        return Ok(());
    }

    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            // Skip target directories
            if path.file_name().map_or(false, |n| n == "target") {
                continue;
            }
            scan_cargo_tomls(&path, f)?;
        } else if path.file_name().map_or(false, |n| n == "Cargo.toml") {
            f(&path)?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_metrics_workspace() {
        let toml = r#"
[dependencies]
serde = { workspace = true }
tokio = { workspace = true }
anyhow = { workspace = true }
"#;
        let m = DepDetector::compute_metrics(toml);
        assert_eq!(m.dep_count, 3);
        assert_eq!(m.workspace_deps, 3);
        assert_eq!(m.versioned_deps, 0);
    }

    #[test]
    fn test_compute_metrics_mixed() {
        let toml = r#"
[dependencies]
serde = { workspace = true }
custom-lib = { path = "../custom-lib" }
external = { version = "1.0", features = ["full"] }
forked = { git = "https://github.com/example/forked" }
"#;
        let m = DepDetector::compute_metrics(toml);
        assert_eq!(m.dep_count, 4);
        assert_eq!(m.workspace_deps, 1);
        assert_eq!(m.path_deps, 1);
        assert_eq!(m.git_deps, 1);
        assert_eq!(m.versioned_deps, 1);
    }

    #[test]
    fn test_detect_single_no_caesura() {
        let metrics = DepMetrics {
            dep_count: 5,
            workspace_deps: 5,
            versioned_deps: 0,
            path_deps: 0,
            git_deps: 0,
        };
        let detector = DepDetector::default();
        let caesuras = detector.detect_single("Cargo.toml", &metrics);
        assert!(caesuras.is_empty());
    }

    #[test]
    fn test_detect_single_high_non_workspace() {
        let metrics = DepMetrics {
            dep_count: 10,
            workspace_deps: 2,
            versioned_deps: 6,
            path_deps: 2,
            git_deps: 0,
        };
        let detector = DepDetector::default();
        let caesuras = detector.detect_single("Cargo.toml", &metrics);
        assert!(!caesuras.is_empty());
    }

    #[test]
    fn test_detect_diff_cluster_added() {
        let before = DepMetrics {
            dep_count: 3,
            workspace_deps: 3,
            versioned_deps: 0,
            path_deps: 0,
            git_deps: 0,
        };
        let after = DepMetrics {
            dep_count: 10,
            workspace_deps: 5,
            versioned_deps: 5,
            path_deps: 0,
            git_deps: 0,
        };
        let detector = DepDetector::default();
        let caesuras = detector.detect_diff("Cargo.toml", &before, &after);
        assert!(!caesuras.is_empty());
    }

    #[test]
    fn test_detect_git_deps() {
        let metrics = DepMetrics {
            dep_count: 5,
            workspace_deps: 3,
            versioned_deps: 0,
            path_deps: 0,
            git_deps: 2,
        };
        let detector = DepDetector::default();
        let caesuras = detector.detect_single("Cargo.toml", &metrics);
        assert!(caesuras.iter().any(|c| c.description.contains("git")));
    }

    #[test]
    fn test_empty_toml() {
        let m = DepDetector::compute_metrics("");
        assert_eq!(m.dep_count, 0);
    }
}
