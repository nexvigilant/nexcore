//! Architecture caesura detection — coupling, pub surface, import patterns.
//!
//! Parses `mod` declarations and `use` imports from Rust source files.
//! Detects files whose architecture metrics deviate from the module baseline.

use crate::metrics::ArchMetrics;
use crate::types::{Caesura, CaesuraScore, CaesuraType, Stratum, StratumLocation};
use std::path::Path;

/// Architecture caesura detector.
///
/// Tier: T3 — Full domain type
pub struct ArchDetector {
    /// Sensitivity multiplier for σ deviation threshold.
    pub sigma_threshold: f64,
}

impl Default for ArchDetector {
    fn default() -> Self {
        Self {
            sigma_threshold: 2.0,
        }
    }
}

impl ArchDetector {
    /// Create a detector with custom sensitivity.
    pub fn with_sensitivity(sigma_threshold: f64) -> Self {
        Self { sigma_threshold }
    }

    /// Compute architecture metrics for a single file's contents.
    pub fn compute_metrics(contents: &str) -> ArchMetrics {
        let lines: Vec<&str> = contents.lines().collect();
        let total_lines = lines.len();

        let mut import_count = 0usize;
        let mut pub_surface = 0usize;
        let mut mod_count = 0usize;

        for line in &lines {
            let trimmed = line.trim();

            // Count use imports
            if trimmed.starts_with("use ") {
                import_count += 1;
            }
            // Count pub items
            if trimmed.starts_with("pub ") && !trimmed.starts_with("pub use ") {
                pub_surface += 1;
            }
            // Count mod declarations
            if (trimmed.starts_with("mod ") || trimmed.starts_with("pub mod "))
                && trimmed.ends_with(';')
            {
                mod_count += 1;
            }
        }

        let coupling_density = if total_lines > 0 {
            import_count as f64 / total_lines as f64
        } else {
            0.0
        };

        ArchMetrics {
            import_count,
            pub_surface,
            mod_count,
            coupling_density,
        }
    }

    /// Detect architecture caesuras across a set of file metrics.
    pub fn detect(&self, file_metrics: &[(String, ArchMetrics)]) -> Vec<Caesura> {
        if file_metrics.len() < 2 {
            return Vec::new();
        }

        let mut caesuras = Vec::new();
        let n = file_metrics.len() as f64;

        let mean_coupling: f64 = file_metrics
            .iter()
            .map(|(_, m)| m.coupling_density)
            .sum::<f64>()
            / n;
        let mean_pub: f64 = file_metrics
            .iter()
            .map(|(_, m)| m.pub_surface as f64)
            .sum::<f64>()
            / n;
        let mean_imports: f64 = file_metrics
            .iter()
            .map(|(_, m)| m.import_count as f64)
            .sum::<f64>()
            / n;

        let std_coupling = stddev(
            file_metrics.iter().map(|(_, m)| m.coupling_density),
            mean_coupling,
            n,
        );
        let std_pub = stddev(
            file_metrics.iter().map(|(_, m)| m.pub_surface as f64),
            mean_pub,
            n,
        );
        let std_imports = stddev(
            file_metrics.iter().map(|(_, m)| m.import_count as f64),
            mean_imports,
            n,
        );

        for (path, metrics) in file_metrics {
            let mut deviations = Vec::new();

            if std_coupling > 0.0 {
                let z = (metrics.coupling_density - mean_coupling).abs() / std_coupling;
                if z > self.sigma_threshold {
                    deviations.push(format!("coupling density z={z:.2}"));
                }
            }
            if std_pub > 0.0 {
                let z = (metrics.pub_surface as f64 - mean_pub).abs() / std_pub;
                if z > self.sigma_threshold {
                    deviations.push(format!("pub surface z={z:.2}"));
                }
            }
            if std_imports > 0.0 {
                let z = (metrics.import_count as f64 - mean_imports).abs() / std_imports;
                if z > self.sigma_threshold {
                    deviations.push(format!("import count z={z:.2}"));
                }
            }

            if !deviations.is_empty() {
                let max_z = deviations
                    .iter()
                    .filter_map(|d| d.split("z=").nth(1).and_then(|s| s.parse::<f64>().ok()))
                    .fold(0.0_f64, f64::max);

                let score = (max_z / (2.0 * self.sigma_threshold)).min(1.0);

                caesuras.push(Caesura {
                    caesura_type: CaesuraType::Architectural,
                    stratum: Stratum::Architecture,
                    score: CaesuraScore::new(score),
                    location: StratumLocation {
                        path: path.clone(),
                        line_range: None,
                        commit_sha: None,
                    },
                    description: format!("Architecture deviation: {}", deviations.join(", ")),
                    boundary_files: vec![path.clone()],
                });
            }
        }

        caesuras
    }

    /// Scan a directory for architecture caesuras.
    pub fn scan_directory(&self, dir: &Path) -> Result<Vec<Caesura>, std::io::Error> {
        let file_metrics = collect_arch_metrics(dir)?;
        Ok(self.detect(&file_metrics))
    }
}

/// Collect architecture metrics from all `.rs` files in a directory.
pub fn collect_arch_metrics(dir: &Path) -> Result<Vec<(String, ArchMetrics)>, std::io::Error> {
    let mut results = Vec::new();

    if !dir.is_dir() {
        return Ok(results);
    }

    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().map_or(false, |e| e == "rs") {
            let contents = std::fs::read_to_string(&path)?;
            let metrics = ArchDetector::compute_metrics(&contents);
            results.push((path.display().to_string(), metrics));
        }
    }

    Ok(results)
}

/// Compute sample standard deviation.
fn stddev(values: impl Iterator<Item = f64>, mean: f64, n: f64) -> f64 {
    if n < 2.0 {
        return 0.0;
    }
    let sum_sq: f64 = values.map(|v| (v - mean).powi(2)).sum();
    (sum_sq / (n - 1.0)).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_metrics_basic() {
        let code = "use std::io;\nuse std::path::Path;\n\npub fn hello() {}\npub struct Foo;\n";
        let m = ArchDetector::compute_metrics(code);
        assert_eq!(m.import_count, 2);
        assert_eq!(m.pub_surface, 2);
        assert_eq!(m.mod_count, 0);
    }

    #[test]
    fn test_compute_metrics_mods() {
        let code = "pub mod foo;\nmod bar;\nuse crate::baz;\n";
        let m = ArchDetector::compute_metrics(code);
        assert_eq!(m.mod_count, 2);
        assert_eq!(m.import_count, 1);
    }

    #[test]
    fn test_coupling_density() {
        let code = "use a;\nuse b;\nuse c;\nfn x() {}\n";
        let m = ArchDetector::compute_metrics(code);
        assert!((m.coupling_density - 0.75).abs() < f64::EPSILON);
    }

    #[test]
    fn test_detect_architecture_outlier() {
        let baseline = ArchMetrics {
            import_count: 3,
            pub_surface: 2,
            mod_count: 0,
            coupling_density: 0.05,
        };
        let outlier = ArchMetrics {
            import_count: 50,
            pub_surface: 40,
            mod_count: 15,
            coupling_density: 0.8,
        };

        // 8 baseline files + 1 extreme outlier for strong signal
        let mut metrics: Vec<(String, ArchMetrics)> = (0..8)
            .map(|i| (format!("{i}.rs"), baseline.clone()))
            .collect();
        metrics.push(("outlier.rs".to_string(), outlier));

        let detector = ArchDetector::with_sensitivity(1.5);
        let caesuras = detector.detect(&metrics);
        assert!(!caesuras.is_empty());
        assert!(caesuras.iter().any(|c| c.location.path == "outlier.rs"));
    }

    #[test]
    fn test_detect_no_caesura_uniform() {
        let uniform = ArchMetrics {
            import_count: 5,
            pub_surface: 3,
            mod_count: 1,
            coupling_density: 0.1,
        };
        let metrics = vec![
            ("a.rs".to_string(), uniform.clone()),
            ("b.rs".to_string(), uniform.clone()),
            ("c.rs".to_string(), uniform.clone()),
        ];

        let detector = ArchDetector::default();
        let caesuras = detector.detect(&metrics);
        assert!(caesuras.is_empty());
    }

    #[test]
    fn test_empty_file() {
        let m = ArchDetector::compute_metrics("");
        assert_eq!(m.import_count, 0);
        assert_eq!(m.pub_surface, 0);
        assert!((m.coupling_density - 0.0).abs() < f64::EPSILON);
    }
}
