//! # Batch Telescope Processor
//!
//! Runs the Zeta Telescope pipeline across multiple zero sets in parallel
//! using Rayon, collecting aggregate statistics and comparative analysis.
//!
//! ## Architecture
//!
//! ```text
//! Vec<LmfdbZeroSet> ──→ rayon::par_iter ──→ run_telescope per set
//!                                          ──→ BatchReport (aggregate)
//! ```

use rayon::prelude::*;
use serde::{Deserialize, Serialize};

use crate::error::ZetaError;
use crate::lmfdb::LmfdbZeroSet;
use crate::pipeline::{TelescopeConfig, TelescopeReport, run_telescope};
use crate::zeros::ZetaZero;

// ── Configuration ────────────────────────────────────────────────────────────

/// Configuration for batch telescope processing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchConfig {
    /// Telescope configuration applied to each zero set.
    pub telescope: TelescopeConfig,
    /// Minimum zeros required per set (sets below this are skipped).
    pub min_zeros: usize,
    /// Whether to collect individual reports (memory-intensive for large batches).
    pub collect_reports: bool,
}

impl Default for BatchConfig {
    fn default() -> Self {
        Self {
            telescope: TelescopeConfig::default(),
            min_zeros: 20,
            collect_reports: true,
        }
    }
}

// ── Report Types ─────────────────────────────────────────────────────────────

/// Individual entry in a batch run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchEntry {
    /// Label identifying this zero set.
    pub label: String,
    /// Number of zeros in the set.
    pub n_zeros: usize,
    /// RH confidence from the telescope.
    pub rh_confidence: f64,
    /// Whether the telescope succeeded.
    pub success: bool,
    /// Error message if failed.
    pub error: Option<String>,
    /// Full telescope report (if `collect_reports` is true).
    #[serde(skip)]
    pub report: Option<TelescopeReport>,
}

/// Aggregate statistics from a batch run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchStatistics {
    /// Number of zero sets processed.
    pub total: usize,
    /// Number that succeeded.
    pub succeeded: usize,
    /// Number that failed.
    pub failed: usize,
    /// Number skipped (below min_zeros threshold).
    pub skipped: usize,
    /// Mean RH confidence across successful runs.
    pub mean_confidence: f64,
    /// Minimum RH confidence across successful runs.
    pub min_confidence: f64,
    /// Maximum RH confidence across successful runs.
    pub max_confidence: f64,
    /// Standard deviation of RH confidence.
    pub std_confidence: f64,
}

/// Complete batch processing report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchReport {
    /// Per-entry results.
    pub entries: Vec<BatchEntry>,
    /// Aggregate statistics.
    pub statistics: BatchStatistics,
}

// ── Public API ───────────────────────────────────────────────────────────────

/// Run the telescope on multiple LMFDB zero sets in parallel.
///
/// Uses Rayon for data-parallel execution. Each zero set is processed
/// independently. Results are collected and aggregated.
///
/// # Errors
///
/// Returns [`ZetaError::InvalidParameter`] if no zero sets are provided.
pub fn run_telescope_batch(
    zero_sets: &[LmfdbZeroSet],
    config: &BatchConfig,
) -> Result<BatchReport, ZetaError> {
    if zero_sets.is_empty() {
        return Err(ZetaError::InvalidParameter(
            "no zero sets provided for batch processing".to_string(),
        ));
    }

    let entries: Vec<BatchEntry> = zero_sets
        .par_iter()
        .map(|zs| process_single(zs, config))
        .collect();

    let statistics = compute_statistics(&entries);

    Ok(BatchReport {
        entries,
        statistics,
    })
}

/// Run the telescope on multiple raw zero vectors in parallel.
///
/// Convenience wrapper when zero sets aren't from LMFDB.
pub fn run_telescope_batch_raw(
    zero_sets: &[(&str, &[ZetaZero])],
    config: &BatchConfig,
) -> Result<BatchReport, ZetaError> {
    if zero_sets.is_empty() {
        return Err(ZetaError::InvalidParameter(
            "no zero sets provided for batch processing".to_string(),
        ));
    }

    let entries: Vec<BatchEntry> = zero_sets
        .par_iter()
        .map(|&(label, zeros)| process_raw(label, zeros, config))
        .collect();

    let statistics = compute_statistics(&entries);

    Ok(BatchReport {
        entries,
        statistics,
    })
}

// ── Internal ─────────────────────────────────────────────────────────────────

fn process_single(zs: &LmfdbZeroSet, config: &BatchConfig) -> BatchEntry {
    let label = zs.l_function.label.clone();
    let n_zeros = zs.zeros.len();

    if n_zeros < config.min_zeros {
        return BatchEntry {
            label,
            n_zeros,
            rh_confidence: 0.0,
            success: false,
            error: Some(format!(
                "skipped: {n_zeros} < {} min_zeros",
                config.min_zeros
            )),
            report: None,
        };
    }

    match run_telescope(&zs.zeros, &config.telescope) {
        Ok(report) => {
            let confidence = report.overall_rh_confidence;
            BatchEntry {
                label,
                n_zeros,
                rh_confidence: confidence,
                success: true,
                error: None,
                report: if config.collect_reports {
                    Some(report)
                } else {
                    None
                },
            }
        }
        Err(e) => BatchEntry {
            label,
            n_zeros,
            rh_confidence: 0.0,
            success: false,
            error: Some(e.to_string()),
            report: None,
        },
    }
}

fn process_raw(label: &str, zeros: &[ZetaZero], config: &BatchConfig) -> BatchEntry {
    let n_zeros = zeros.len();

    if n_zeros < config.min_zeros {
        return BatchEntry {
            label: label.to_string(),
            n_zeros,
            rh_confidence: 0.0,
            success: false,
            error: Some(format!(
                "skipped: {n_zeros} < {} min_zeros",
                config.min_zeros
            )),
            report: None,
        };
    }

    match run_telescope(zeros, &config.telescope) {
        Ok(report) => {
            let confidence = report.overall_rh_confidence;
            BatchEntry {
                label: label.to_string(),
                n_zeros,
                rh_confidence: confidence,
                success: true,
                error: None,
                report: if config.collect_reports {
                    Some(report)
                } else {
                    None
                },
            }
        }
        Err(e) => BatchEntry {
            label: label.to_string(),
            n_zeros,
            rh_confidence: 0.0,
            success: false,
            error: Some(e.to_string()),
            report: None,
        },
    }
}

fn compute_statistics(entries: &[BatchEntry]) -> BatchStatistics {
    let total = entries.len();
    let skipped = entries
        .iter()
        .filter(|e| {
            e.error
                .as_ref()
                .is_some_and(|msg| msg.starts_with("skipped"))
        })
        .count();
    let succeeded = entries.iter().filter(|e| e.success).count();
    let failed = total - succeeded - skipped;

    let confidences: Vec<f64> = entries
        .iter()
        .filter(|e| e.success)
        .map(|e| e.rh_confidence)
        .collect();

    let (mean, min, max, std) = if confidences.is_empty() {
        (0.0, 0.0, 0.0, 0.0)
    } else {
        let n = confidences.len() as f64;
        let mean = confidences.iter().sum::<f64>() / n;
        let min = confidences.iter().copied().fold(f64::INFINITY, f64::min);
        let max = confidences
            .iter()
            .copied()
            .fold(f64::NEG_INFINITY, f64::max);
        let variance = confidences.iter().map(|&c| (c - mean).powi(2)).sum::<f64>() / n;
        (mean, min, max, variance.sqrt())
    };

    BatchStatistics {
        total,
        succeeded,
        failed,
        skipped,
        mean_confidence: mean,
        min_confidence: min,
        max_confidence: max,
        std_confidence: std,
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lmfdb::{LmfdbLfunction, LmfdbZeroSet, embedded_riemann_zeros};
    use crate::zeros::find_zeros_bracket;

    fn make_zero_set(label: &str, zeros: Vec<ZetaZero>) -> LmfdbZeroSet {
        LmfdbZeroSet {
            l_function: LmfdbLfunction {
                label: label.to_string(),
                degree: 1,
                conductor: 1,
                description: format!("test {label}"),
            },
            zeros: zeros.clone(),
            parse_failures: 0,
            fidelity: 1.0,
        }
    }

    #[test]
    fn batch_on_embedded_zeros() {
        let zeros = embedded_riemann_zeros();
        let sets = vec![make_zero_set("riemann-30", zeros)];
        let config = BatchConfig::default();
        let report = run_telescope_batch(&sets, &config);
        assert!(report.is_ok());
        let r = report.unwrap_or_else(|_| unreachable!());
        assert_eq!(r.statistics.total, 1);
        assert_eq!(r.statistics.succeeded, 1);
        assert!(r.statistics.mean_confidence > 0.0);
    }

    #[test]
    fn batch_skips_small_sets() {
        let zeros = embedded_riemann_zeros();
        let small: Vec<ZetaZero> = zeros[..5].to_vec();
        let sets = vec![make_zero_set("small", small), make_zero_set("big", zeros)];
        let config = BatchConfig::default();
        let report = run_telescope_batch(&sets, &config);
        assert!(report.is_ok());
        let r = report.unwrap_or_else(|_| unreachable!());
        assert_eq!(r.statistics.total, 2);
        assert_eq!(r.statistics.succeeded, 1);
        assert_eq!(r.statistics.skipped, 1);
    }

    #[test]
    fn batch_raw_interface() {
        let zeros = embedded_riemann_zeros();
        let sets: Vec<(&str, &[ZetaZero])> = vec![("embedded", &zeros)];
        let config = BatchConfig::default();
        let report = run_telescope_batch_raw(&sets, &config);
        assert!(report.is_ok());
    }

    #[test]
    fn batch_multiple_computed_sets() {
        let z1 = find_zeros_bracket(10.0, 100.0, 0.1).unwrap_or_default();
        let z2 = find_zeros_bracket(100.0, 200.0, 0.1).unwrap_or_default();
        if z1.len() < 20 || z2.len() < 20 {
            return;
        }
        let sets = vec![
            make_zero_set("range-10-100", z1),
            make_zero_set("range-100-200", z2),
        ];
        let config = BatchConfig::default();
        let report = run_telescope_batch(&sets, &config);
        assert!(report.is_ok());
        let r = report.unwrap_or_else(|_| unreachable!());
        assert_eq!(r.statistics.succeeded, 2);
        assert!(r.statistics.mean_confidence > 0.0);
        eprintln!(
            "Batch: mean={:.4}, min={:.4}, max={:.4}",
            r.statistics.mean_confidence, r.statistics.min_confidence, r.statistics.max_confidence,
        );
    }

    #[test]
    fn empty_batch_is_error() {
        let config = BatchConfig::default();
        let result = run_telescope_batch(&[], &config);
        assert!(result.is_err());
    }

    #[test]
    fn statistics_computation() {
        let entries = vec![
            BatchEntry {
                label: "a".into(),
                n_zeros: 30,
                rh_confidence: 0.8,
                success: true,
                error: None,
                report: None,
            },
            BatchEntry {
                label: "b".into(),
                n_zeros: 30,
                rh_confidence: 0.6,
                success: true,
                error: None,
                report: None,
            },
        ];
        let stats = compute_statistics(&entries);
        assert_eq!(stats.succeeded, 2);
        assert!((stats.mean_confidence - 0.7).abs() < 1e-10);
        assert!((stats.min_confidence - 0.6).abs() < 1e-10);
        assert!((stats.max_confidence - 0.8).abs() < 1e-10);
    }
}
