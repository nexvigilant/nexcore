//! JSON persistence and drift detection for measurement history.
//!
//! Storage: `~/nexcore/output/measure_history.json`
//!
//! ## Primitive Foundation
//!
//! | Primitive | Manifestation |
//! |-----------|---------------|
//! | T1: Sequence (σ) | Time-series of measurements |
//! | T1: State (ς) | Persistent snapshot storage |
//! | T1: Comparison (κ) | Welch t-test for drift |

use crate::error::{MeasureError, MeasureResult};
use crate::stats;
use crate::types::*;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Default history file path.
pub fn default_history_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    PathBuf::from(home)
        .join("nexcore")
        .join("output")
        .join("measure_history.json")
}

/// A single history record (workspace snapshot).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryRecord {
    /// When the snapshot was taken.
    pub timestamp: MeasureTimestamp,
    /// Total LOC at this point.
    pub total_loc: usize,
    /// Total tests.
    pub total_tests: usize,
    /// Number of crates.
    pub crate_count: usize,
    /// Mean health score.
    pub mean_health: f64,
    /// Graph density.
    pub graph_density: f64,
    /// Max dependency depth.
    pub max_depth: usize,
}

/// Full history container.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MeasureHistory {
    pub records: Vec<HistoryRecord>,
}

impl MeasureHistory {
    /// Load from JSON file.
    pub fn load(path: &Path) -> MeasureResult<Self> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let content = std::fs::read_to_string(path).map_err(MeasureError::Io)?;
        let history: Self = serde_json::from_str(&content)?;
        Ok(history)
    }

    /// Save to JSON file.
    pub fn save(&self, path: &Path) -> MeasureResult<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(MeasureError::Io)?;
        }
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json).map_err(MeasureError::Io)?;
        Ok(())
    }

    /// Append a workspace measurement as a new record.
    pub fn record(&mut self, wm: &WorkspaceMeasurement, mean_health: f64) {
        self.records.push(HistoryRecord {
            timestamp: wm.timestamp,
            total_loc: wm.total_loc,
            total_tests: wm.total_tests,
            crate_count: wm.crate_count,
            mean_health,
            graph_density: wm.graph_density.value(),
            max_depth: wm.max_depth,
        });
    }

    /// Get last N records.
    pub fn last_n(&self, n: usize) -> &[HistoryRecord] {
        let start = self.records.len().saturating_sub(n);
        &self.records[start..]
    }
}

/// Detect drift in metrics between two time windows.
///
/// Splits last `window * 2` records into before/after halves and runs Welch t-tests.
pub fn detect_drift(history: &MeasureHistory, window: usize) -> MeasureResult<Vec<DriftResult>> {
    let total = window * 2;
    if history.records.len() < total {
        return Err(MeasureError::InsufficientData {
            need: total,
            got: history.records.len(),
            context: "drift detection needs window*2 records".into(),
        });
    }

    let records = history.last_n(total);
    let (before, after) = records.split_at(window);

    let mut results = Vec::new();
    results.push(drift_test(
        "health_score",
        extract_metric(before, |r| r.mean_health),
        extract_metric(after, |r| r.mean_health),
    )?);
    results.push(drift_test(
        "total_loc",
        extract_metric(before, |r| r.total_loc as f64),
        extract_metric(after, |r| r.total_loc as f64),
    )?);
    results.push(drift_test(
        "total_tests",
        extract_metric(before, |r| r.total_tests as f64),
        extract_metric(after, |r| r.total_tests as f64),
    )?);
    results.push(drift_test(
        "graph_density",
        extract_metric(before, |r| r.graph_density),
        extract_metric(after, |r| r.graph_density),
    )?);

    Ok(results)
}

/// Run a single Welch t-test for drift.
fn drift_test(metric: &str, before: Vec<f64>, after: Vec<f64>) -> MeasureResult<DriftResult> {
    let result = stats::welch_t_test(&before, &after)?;
    let significant = result.p_value < 0.05;
    let direction = determine_direction(&before, &after, significant);

    Ok(DriftResult {
        metric: metric.to_string(),
        t_statistic: result.t_statistic,
        dof: result.dof,
        p_value: result.p_value,
        significant,
        direction,
    })
}

/// Determine drift direction from means.
fn determine_direction(before: &[f64], after: &[f64], significant: bool) -> DriftDirection {
    if !significant {
        return DriftDirection::Stable;
    }
    let mean_before = before.iter().sum::<f64>() / before.len() as f64;
    let mean_after = after.iter().sum::<f64>() / after.len() as f64;
    if mean_after > mean_before {
        DriftDirection::Improving
    } else {
        DriftDirection::Degrading
    }
}

/// Extract a metric field from records.
fn extract_metric(records: &[HistoryRecord], f: impl Fn(&HistoryRecord) -> f64) -> Vec<f64> {
    records.iter().map(|r| f(r)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_record(loc: usize, tests: usize, health: f64) -> HistoryRecord {
        HistoryRecord {
            timestamp: MeasureTimestamp::now(),
            total_loc: loc,
            total_tests: tests,
            crate_count: 10,
            mean_health: health,
            graph_density: 0.05,
            max_depth: 5,
        }
    }

    #[test]
    fn history_save_load_roundtrip() {
        let dir = std::env::temp_dir().join("nexcore_measure_test");
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("test_history.json");

        let mut h = MeasureHistory::default();
        h.records.push(make_record(1000, 50, 7.5));
        h.records.push(make_record(1200, 60, 7.8));
        h.save(&path).unwrap_or_else(|_| ());

        let loaded = MeasureHistory::load(&path).unwrap_or_else(|_| MeasureHistory::default());
        assert_eq!(loaded.records.len(), 2);
        assert!((loaded.records[0].mean_health - 7.5).abs() < f64::EPSILON);

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn last_n_slices_correctly() {
        let mut h = MeasureHistory::default();
        for i in 0..10 {
            h.records.push(make_record(i * 100, i * 10, i as f64));
        }
        let last3 = h.last_n(3);
        assert_eq!(last3.len(), 3);
        assert!((last3[0].mean_health - 7.0).abs() < f64::EPSILON);
    }

    #[test]
    fn drift_insufficient_data() {
        let h = MeasureHistory::default();
        assert!(detect_drift(&h, 5).is_err());
    }

    #[test]
    fn drift_detects_step_change() {
        let mut h = MeasureHistory::default();
        // Before window: health ~5.0
        for _ in 0..5 {
            h.records.push(make_record(1000, 50, 5.0));
        }
        // After window: health ~8.0
        for _ in 0..5 {
            h.records.push(make_record(1000, 50, 8.0));
        }
        let drifts = detect_drift(&h, 5).unwrap_or_default();
        let health_drift = drifts.iter().find(|d| d.metric == "health_score");
        assert!(health_drift.is_some());
        let hd = health_drift.unwrap_or_else(|| &drifts[0]);
        assert!(hd.significant, "step change should be significant");
        assert_eq!(hd.direction, DriftDirection::Improving);
    }

    #[test]
    fn drift_stable_when_no_change() {
        let mut h = MeasureHistory::default();
        for _ in 0..10 {
            h.records.push(make_record(1000, 50, 7.0));
        }
        let drifts = detect_drift(&h, 5).unwrap_or_default();
        for d in &drifts {
            assert_eq!(d.direction, DriftDirection::Stable);
        }
    }

    #[test]
    fn load_nonexistent_returns_empty() {
        let h = MeasureHistory::load(Path::new("/nonexistent/path.json"))
            .unwrap_or_else(|_| MeasureHistory::default());
        assert!(h.records.is_empty());
    }
}
