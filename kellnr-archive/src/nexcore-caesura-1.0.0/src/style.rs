//! Style caesura detection — naming entropy, comment density, line length.
//!
//! Sliding window over files in a directory. Per-file metrics are computed,
//! then files whose metrics deviate >2σ from module baseline are flagged.

use crate::metrics::StyleMetrics;
use crate::types::{Caesura, CaesuraScore, CaesuraType, Stratum, StratumLocation};
use std::path::Path;

/// Style caesura detector.
///
/// Tier: T3 — Full domain type
pub struct StyleDetector {
    /// Sensitivity multiplier for σ deviation threshold.
    /// Lower = more sensitive (detects smaller deviations).
    pub sigma_threshold: f64,
}

impl Default for StyleDetector {
    fn default() -> Self {
        Self {
            sigma_threshold: 2.0,
        }
    }
}

impl StyleDetector {
    /// Create a detector with custom sensitivity.
    pub fn with_sensitivity(sigma_threshold: f64) -> Self {
        Self { sigma_threshold }
    }

    /// Compute style metrics for a single file's contents.
    pub fn compute_metrics(contents: &str) -> StyleMetrics {
        let lines: Vec<&str> = contents.lines().collect();
        let total_lines = lines.len();

        let mut comment_lines = 0usize;
        let mut snake_count = 0usize;
        let mut camel_count = 0usize;
        let mut line_lengths = Vec::with_capacity(total_lines);

        for line in &lines {
            let trimmed = line.trim();

            // Count comment lines
            if trimmed.starts_with("//") || trimmed.starts_with("///") || trimmed.starts_with("//!")
            {
                comment_lines += 1;
            }

            line_lengths.push(line.len() as f64);

            // Extract identifiers: words matching [a-zA-Z_][a-zA-Z0-9_]*
            for word in extract_identifiers(trimmed) {
                if word.contains('_') && word.chars().any(|c| c.is_ascii_lowercase()) {
                    snake_count += 1;
                } else if word.len() > 1
                    && word
                        .chars()
                        .next()
                        .map_or(false, |c| c.is_ascii_lowercase())
                    && word.chars().any(|c| c.is_ascii_uppercase())
                {
                    camel_count += 1;
                }
            }
        }

        let total_idents = snake_count + camel_count;
        let snake_case_ratio = if total_idents > 0 {
            snake_count as f64 / total_idents as f64
        } else {
            0.0
        };
        let camel_case_ratio = if total_idents > 0 {
            camel_count as f64 / total_idents as f64
        } else {
            0.0
        };

        let comment_density = if total_lines > 0 {
            comment_lines as f64 / total_lines as f64
        } else {
            0.0
        };

        let mean_line_length = if line_lengths.is_empty() {
            0.0
        } else {
            line_lengths.iter().sum::<f64>() / line_lengths.len() as f64
        };

        let stddev_line_length = if line_lengths.len() < 2 {
            0.0
        } else {
            let variance = line_lengths
                .iter()
                .map(|&l| (l - mean_line_length).powi(2))
                .sum::<f64>()
                / (line_lengths.len() - 1) as f64;
            variance.sqrt()
        };

        StyleMetrics {
            snake_case_ratio,
            camel_case_ratio,
            comment_density,
            mean_line_length,
            stddev_line_length,
            total_lines,
        }
    }

    /// Detect style caesuras across a set of file metrics.
    ///
    /// `file_metrics` is a list of (file_path, metrics) pairs.
    /// Returns caesuras for files whose metrics deviate >σ_threshold from baseline.
    pub fn detect(&self, file_metrics: &[(String, StyleMetrics)]) -> Vec<Caesura> {
        if file_metrics.len() < 2 {
            return Vec::new();
        }

        let mut caesuras = Vec::new();

        // Compute baseline statistics
        let n = file_metrics.len() as f64;
        let mean_entropy: f64 = file_metrics
            .iter()
            .map(|(_, m)| m.naming_entropy())
            .sum::<f64>()
            / n;
        let mean_comment: f64 = file_metrics
            .iter()
            .map(|(_, m)| m.comment_density)
            .sum::<f64>()
            / n;
        let mean_linelen: f64 = file_metrics
            .iter()
            .map(|(_, m)| m.mean_line_length)
            .sum::<f64>()
            / n;

        let std_entropy = stddev(
            file_metrics.iter().map(|(_, m)| m.naming_entropy()),
            mean_entropy,
            n,
        );
        let std_comment = stddev(
            file_metrics.iter().map(|(_, m)| m.comment_density),
            mean_comment,
            n,
        );
        let std_linelen = stddev(
            file_metrics.iter().map(|(_, m)| m.mean_line_length),
            mean_linelen,
            n,
        );

        for (path, metrics) in file_metrics {
            let mut deviations = Vec::new();

            if std_entropy > 0.0 {
                let z = (metrics.naming_entropy() - mean_entropy).abs() / std_entropy;
                if z > self.sigma_threshold {
                    deviations.push(format!("naming entropy z={z:.2}"));
                }
            }
            if std_comment > 0.0 {
                let z = (metrics.comment_density - mean_comment).abs() / std_comment;
                if z > self.sigma_threshold {
                    deviations.push(format!("comment density z={z:.2}"));
                }
            }
            if std_linelen > 0.0 {
                let z = (metrics.mean_line_length - mean_linelen).abs() / std_linelen;
                if z > self.sigma_threshold {
                    deviations.push(format!("line length z={z:.2}"));
                }
            }

            if !deviations.is_empty() {
                let max_z = deviations
                    .iter()
                    .filter_map(|d| d.split("z=").nth(1).and_then(|s| s.parse::<f64>().ok()))
                    .fold(0.0_f64, f64::max);

                // Normalize score: z of sigma_threshold maps to 0.5, z of 2*threshold maps to 1.0
                let score = (max_z / (2.0 * self.sigma_threshold)).min(1.0);

                caesuras.push(Caesura {
                    caesura_type: CaesuraType::Stylistic,
                    stratum: Stratum::Style,
                    score: CaesuraScore::new(score),
                    location: StratumLocation {
                        path: path.clone(),
                        line_range: None,
                        commit_sha: None,
                    },
                    description: format!("Style deviation: {}", deviations.join(", ")),
                    boundary_files: vec![path.clone()],
                });
            }
        }

        caesuras
    }

    /// Scan a directory for style caesuras.
    ///
    /// Reads all `.rs` files, computes metrics, detects outliers.
    pub fn scan_directory(&self, dir: &Path) -> Result<Vec<Caesura>, std::io::Error> {
        let file_metrics = collect_rs_file_metrics(dir)?;
        Ok(self.detect(&file_metrics))
    }
}

/// Collect style metrics from all `.rs` files in a directory (non-recursive).
pub fn collect_rs_file_metrics(dir: &Path) -> Result<Vec<(String, StyleMetrics)>, std::io::Error> {
    let mut results = Vec::new();

    if !dir.is_dir() {
        return Ok(results);
    }

    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().map_or(false, |e| e == "rs") {
            let contents = std::fs::read_to_string(&path)?;
            let metrics = StyleDetector::compute_metrics(&contents);
            results.push((path.display().to_string(), metrics));
        }
    }

    Ok(results)
}

/// Extract likely identifiers from a line of Rust code.
fn extract_identifiers(line: &str) -> Vec<&str> {
    let mut idents = Vec::new();
    let bytes = line.as_bytes();
    let mut start = None;

    for (i, &b) in bytes.iter().enumerate() {
        match (start, b) {
            (None, b'a'..=b'z' | b'A'..=b'Z' | b'_') => {
                start = Some(i);
            }
            (Some(_), b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'_') => {}
            (Some(s), _) => {
                let word = &line[s..i];
                if word.len() > 1 && !is_rust_keyword(word) {
                    idents.push(word);
                }
                start = None;
            }
            _ => {}
        }
    }

    // Handle identifier at end of line
    if let Some(s) = start {
        let word = &line[s..];
        if word.len() > 1 && !is_rust_keyword(word) {
            idents.push(word);
        }
    }

    idents
}

/// Check if a word is a Rust keyword (abbreviated list for filtering).
fn is_rust_keyword(word: &str) -> bool {
    matches!(
        word,
        "fn" | "let"
            | "mut"
            | "pub"
            | "use"
            | "mod"
            | "struct"
            | "enum"
            | "impl"
            | "trait"
            | "type"
            | "const"
            | "static"
            | "if"
            | "else"
            | "match"
            | "for"
            | "while"
            | "loop"
            | "return"
            | "break"
            | "continue"
            | "where"
            | "self"
            | "super"
            | "crate"
            | "as"
            | "in"
            | "ref"
            | "true"
            | "false"
            | "async"
            | "await"
            | "move"
            | "dyn"
            | "extern"
            | "unsafe"
    )
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
    fn test_compute_metrics_snake_case() {
        let code = "fn hello_world() {\n    let my_var = 1;\n}\n";
        let m = StyleDetector::compute_metrics(code);
        assert!(m.snake_case_ratio > 0.0);
        assert!((m.camel_case_ratio - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_compute_metrics_camel_case() {
        let code = "let helloWorld = 1;\nlet myVariable = 2;\n";
        let m = StyleDetector::compute_metrics(code);
        assert!(m.camel_case_ratio > 0.0);
    }

    #[test]
    fn test_comment_density() {
        let code = "// comment\n// comment\nfn foo() {}\nfn bar() {}\n";
        let m = StyleDetector::compute_metrics(code);
        assert!((m.comment_density - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_naming_entropy_pure_snake() {
        let code = "let my_var = 1;\nlet other_thing = 2;\n";
        let m = StyleDetector::compute_metrics(code);
        assert!((m.naming_entropy() - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_naming_entropy_mixed() {
        let code = "let my_var = 1;\nlet myVar = 2;\n";
        let m = StyleDetector::compute_metrics(code);
        assert!(m.naming_entropy() > 0.0);
    }

    #[test]
    fn test_detect_finds_outlier() {
        // Use 8 baseline files to establish a strong mean, then one extreme outlier.
        // With naming_entropy: baseline all have 0.0 (pure snake), outlier has 1.0 (mixed).
        // comment_density: baseline 0.1, outlier 0.9.
        // mean_line_length: baseline 40, outlier 200.
        let baseline = StyleMetrics {
            snake_case_ratio: 1.0,
            camel_case_ratio: 0.0,
            comment_density: 0.1,
            mean_line_length: 40.0,
            stddev_line_length: 10.0,
            total_lines: 100,
        };
        let outlier = StyleMetrics {
            snake_case_ratio: 0.5,
            camel_case_ratio: 0.5,
            comment_density: 0.9,
            mean_line_length: 200.0,
            stddev_line_length: 5.0,
            total_lines: 100,
        };

        let mut metrics: Vec<(String, StyleMetrics)> = (0..8)
            .map(|i| (format!("{i}.rs"), baseline.clone()))
            .collect();
        metrics.push(("outlier.rs".to_string(), outlier));

        // Use a more sensitive detector to ensure detection
        let detector = StyleDetector::with_sensitivity(1.5);
        let caesuras = detector.detect(&metrics);
        assert!(!caesuras.is_empty());
        assert!(caesuras.iter().any(|c| c.location.path == "outlier.rs"));
    }

    #[test]
    fn test_detect_no_caesura_for_uniform() {
        let uniform = StyleMetrics {
            snake_case_ratio: 1.0,
            camel_case_ratio: 0.0,
            comment_density: 0.1,
            mean_line_length: 40.0,
            stddev_line_length: 10.0,
            total_lines: 100,
        };
        let metrics = vec![
            ("a.rs".to_string(), uniform.clone()),
            ("b.rs".to_string(), uniform.clone()),
            ("c.rs".to_string(), uniform.clone()),
        ];

        let detector = StyleDetector::default();
        let caesuras = detector.detect(&metrics);
        assert!(caesuras.is_empty());
    }

    #[test]
    fn test_empty_file_metrics() {
        let m = StyleDetector::compute_metrics("");
        assert_eq!(m.total_lines, 0);
        assert!((m.comment_density - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_extract_identifiers() {
        let idents = extract_identifiers("let my_var = some_func();");
        assert!(idents.contains(&"my_var"));
        assert!(idents.contains(&"some_func"));
    }
}
