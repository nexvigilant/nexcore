//! Orchestrator: combines all stratum detectors into a single scan.

use crate::architecture::ArchDetector;
use crate::dependency::DepDetector;
use crate::style::StyleDetector;
use crate::types::{Caesura, CaesuraSeverity, Stratum};
use std::path::Path;

/// Unified caesura detector that orchestrates all stratum detectors.
///
/// Tier: T3 — Orchestrator scanning a path for caesuras
pub struct CaesuraDetector {
    /// Style detector configuration.
    pub style: StyleDetector,
    /// Architecture detector configuration.
    pub arch: ArchDetector,
    /// Dependency detector configuration.
    pub dep: DepDetector,
    /// Which strata to scan (None = all).
    pub strata: Option<Vec<Stratum>>,
}

impl Default for CaesuraDetector {
    fn default() -> Self {
        Self {
            style: StyleDetector::default(),
            arch: ArchDetector::default(),
            dep: DepDetector::default(),
            strata: None,
        }
    }
}

impl CaesuraDetector {
    /// Create a detector with custom sensitivity applied to all detectors.
    pub fn with_sensitivity(sigma: f64) -> Self {
        Self {
            style: StyleDetector::with_sensitivity(sigma),
            arch: ArchDetector::with_sensitivity(sigma),
            dep: DepDetector::default(),
            strata: None,
        }
    }

    /// Restrict scanning to specific strata.
    pub fn with_strata(mut self, strata: Vec<Stratum>) -> Self {
        self.strata = Some(strata);
        self
    }

    /// Check if a stratum should be scanned.
    fn should_scan(&self, stratum: Stratum) -> bool {
        self.strata.as_ref().map_or(true, |s| s.contains(&stratum))
    }

    /// Scan a directory for caesuras across all enabled strata.
    pub fn scan(&self, dir: &Path) -> Result<Vec<Caesura>, std::io::Error> {
        let mut all_caesuras = Vec::new();

        // Style stratum: scan .rs files in dir
        if self.should_scan(Stratum::Style) {
            let style_caesuras = self.style.scan_directory(dir)?;
            all_caesuras.extend(style_caesuras);
        }

        // Architecture stratum: scan .rs files for import/pub patterns
        if self.should_scan(Stratum::Architecture) {
            let arch_caesuras = self.arch.scan_directory(dir)?;
            all_caesuras.extend(arch_caesuras);
        }

        // Dependency stratum: scan Cargo.toml files
        if self.should_scan(Stratum::Dependency) {
            let dep_caesuras = self.dep.scan_directory(dir)?;
            all_caesuras.extend(dep_caesuras);
        }

        // Sort by score descending (most severe first)
        all_caesuras.sort_by(|a, b| {
            b.score
                .value()
                .partial_cmp(&a.score.value())
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        Ok(all_caesuras)
    }

    /// Generate a markdown report from scan results.
    pub fn report(caesuras: &[Caesura]) -> String {
        if caesuras.is_empty() {
            return "No caesuras detected.".to_string();
        }

        let mut out = String::from("| Severity | Type | Score | Location | Description |\n");
        out.push_str("|----------|------|-------|----------|-------------|\n");

        for c in caesuras {
            out.push_str(&format!(
                "| {} | {} | {:.2} | {} | {} |\n",
                c.score.severity().label(),
                c.caesura_type.label(),
                c.score.value(),
                c.location.path,
                c.description,
            ));
        }

        // Summary line
        let severe = caesuras
            .iter()
            .filter(|c| c.score.severity() == CaesuraSeverity::Severe)
            .count();
        let moderate = caesuras
            .iter()
            .filter(|c| c.score.severity() == CaesuraSeverity::Moderate)
            .count();
        let mild = caesuras
            .iter()
            .filter(|c| c.score.severity() == CaesuraSeverity::Mild)
            .count();

        out.push_str(&format!(
            "\n**Total: {} caesuras** (severe={}, moderate={}, mild={})\n",
            caesuras.len(),
            severe,
            moderate,
            mild,
        ));

        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{CaesuraScore, CaesuraType, StratumLocation};

    #[test]
    fn test_report_empty() {
        let report = CaesuraDetector::report(&[]);
        assert_eq!(report, "No caesuras detected.");
    }

    #[test]
    fn test_report_with_caesuras() {
        let caesuras = vec![
            Caesura {
                caesura_type: CaesuraType::Stylistic,
                stratum: Stratum::Style,
                score: CaesuraScore::new(0.8),
                location: StratumLocation {
                    path: "src/foo.rs".to_string(),
                    line_range: None,
                    commit_sha: None,
                },
                description: "naming entropy z=3.5".to_string(),
                boundary_files: vec!["src/foo.rs".to_string()],
            },
            Caesura {
                caesura_type: CaesuraType::Dependency,
                stratum: Stratum::Dependency,
                score: CaesuraScore::new(0.4),
                location: StratumLocation {
                    path: "Cargo.toml".to_string(),
                    line_range: None,
                    commit_sha: None,
                },
                description: "5 git deps".to_string(),
                boundary_files: vec!["Cargo.toml".to_string()],
            },
        ];

        let report = CaesuraDetector::report(&caesuras);
        assert!(report.contains("severe"));
        assert!(report.contains("mild"));
        assert!(report.contains("Total: 2 caesuras"));
    }

    #[test]
    fn test_should_scan_all() {
        let detector = CaesuraDetector::default();
        assert!(detector.should_scan(Stratum::Style));
        assert!(detector.should_scan(Stratum::Architecture));
        assert!(detector.should_scan(Stratum::Dependency));
    }

    #[test]
    fn test_should_scan_filtered() {
        let detector = CaesuraDetector::default().with_strata(vec![Stratum::Style]);
        assert!(detector.should_scan(Stratum::Style));
        assert!(!detector.should_scan(Stratum::Architecture));
        assert!(!detector.should_scan(Stratum::Dependency));
    }

    #[test]
    fn test_default_sensitivity() {
        let detector = CaesuraDetector::default();
        assert!((detector.style.sigma_threshold - 2.0).abs() < f64::EPSILON);
        assert!((detector.arch.sigma_threshold - 2.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_custom_sensitivity() {
        let detector = CaesuraDetector::with_sensitivity(1.5);
        assert!((detector.style.sigma_threshold - 1.5).abs() < f64::EPSILON);
        assert!((detector.arch.sigma_threshold - 1.5).abs() < f64::EPSILON);
    }
}
