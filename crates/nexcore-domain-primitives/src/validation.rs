//! Corpus coverage validation — every KSB decomposes to T1 primitives.
//!
//! ## T1 Grounding
//!
//! - `∃` (Existence) — validate that every KSB has a primitive decomposition
//! - `Σ` (Sum) — aggregate coverage across the full corpus
//! - `∂` (Boundary) — enforce 100% coverage threshold

use serde::{Deserialize, Serialize};

// ── Types ─────────────────────────────────────────────────────────────────────

/// Result of decomposing a single KSB to primitives.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KsbDecomposition {
    /// KSB identifier.
    pub ksb_id: String,
    /// Human-readable name.
    pub ksb_name: String,
    /// Primitives found during decomposition.
    pub primitives_found: Vec<String>,
    /// `true` if at least one primitive maps to this KSB.
    pub is_covered: bool,
}

/// Result of the full corpus coverage analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageResult {
    /// Total KSBs in the corpus.
    pub total_ksbs: usize,
    /// KSBs that decompose to at least one primitive.
    pub covered: usize,
    /// KSBs with no primitive decomposition.
    pub uncovered: usize,
    /// Coverage percentage (0.0–100.0).
    pub coverage_pct: f64,
    /// Target percentage (100.0 for full coverage).
    pub target_pct: f64,
    /// `true` if coverage_pct >= target_pct.
    pub passes: bool,
    /// Names of uncovered KSBs (for remediation).
    pub uncovered_ksbs: Vec<String>,
}

// ── Core function ─────────────────────────────────────────────────────────────

/// Validate that all KSBs in the given corpus decompose to T1 primitives.
///
/// Each KSB is a tuple of `(id, name, primitives)`. A KSB is considered
/// covered if its primitive list is non-empty. Target: 100% coverage
/// (raised from previous 95% threshold).
///
/// # Example
///
/// ```rust
/// use nexcore_domain_primitives::validation::validate_corpus_coverage;
///
/// let ksbs = vec![
///     ("K001".into(), "Signal Detection".into(), vec!["N".into(), "→".into()]),
///     ("K002".into(), "Causality Assessment".into(), vec!["→".into()]),
///     ("K003".into(), "Orphan KSB".into(), vec![]),
/// ];
/// let result = validate_corpus_coverage(&ksbs);
/// assert_eq!(result.covered, 2);
/// assert_eq!(result.uncovered, 1);
/// assert!(!result.passes);
/// ```
pub fn validate_corpus_coverage(
    ksbs: &[(String, String, Vec<String>)],
) -> CoverageResult {
    let total = ksbs.len();
    let mut covered = 0usize;
    let mut uncovered_ksbs = Vec::new();

    for (id, name, primitives) in ksbs {
        if primitives.is_empty() {
            uncovered_ksbs.push(format!("{id}: {name}"));
        } else {
            covered += 1;
        }
    }

    let coverage_pct = if total > 0 {
        covered as f64 / total as f64 * 100.0
    } else {
        100.0
    };
    let target_pct = 100.0;

    CoverageResult {
        total_ksbs: total,
        covered,
        uncovered: total - covered,
        coverage_pct,
        target_pct,
        passes: (coverage_pct - target_pct).abs() < f64::EPSILON || coverage_pct >= target_pct,
        uncovered_ksbs,
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_corpus_passes() {
        let result = validate_corpus_coverage(&[]);
        assert!(result.passes);
        assert!((result.coverage_pct - 100.0).abs() < f64::EPSILON);
    }

    #[test]
    fn full_coverage_passes() {
        let ksbs = vec![
            ("K1".into(), "Detection".into(), vec!["N".into()]),
            ("K2".into(), "Causality".into(), vec!["→".into()]),
        ];
        let result = validate_corpus_coverage(&ksbs);
        assert!(result.passes);
        assert_eq!(result.covered, 2);
        assert_eq!(result.uncovered, 0);
    }

    #[test]
    fn partial_coverage_fails() {
        let ksbs = vec![
            ("K1".into(), "Good".into(), vec!["N".into()]),
            ("K2".into(), "Bad".into(), vec![]),
        ];
        let result = validate_corpus_coverage(&ksbs);
        assert!(!result.passes);
        assert!((result.coverage_pct - 50.0).abs() < f64::EPSILON);
        assert_eq!(result.uncovered_ksbs.len(), 1);
    }

    #[test]
    fn uncovered_ksbs_listed() {
        let ksbs = vec![
            ("K1".into(), "Orphan A".into(), vec![]),
            ("K2".into(), "Orphan B".into(), vec![]),
            ("K3".into(), "Covered".into(), vec!["∃".into()]),
        ];
        let result = validate_corpus_coverage(&ksbs);
        assert_eq!(result.uncovered_ksbs.len(), 2);
        assert!(result.uncovered_ksbs[0].contains("Orphan A"));
    }
}
