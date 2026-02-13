//! Build summary — aggregate metrics for dashboard display.
//!
//! ## Primitive Foundation
//! | Primitive | Manifestation |
//! |-----------|---------------|
//! | T1: N (Quantity) | All numeric metrics |
//! | T1: Σ (Sum) | Aggregation |
//! | T1: κ (Comparison) | Success rate comparison |

use crate::pipeline::state::{PipelineRunState, RunStatus};
use crate::types::BuildDuration;
use serde::{Deserialize, Serialize};

/// High-level summary of build health.
///
/// Tier: T2-C (N + Σ + κ + σ, dominant N)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildSummary {
    /// Total number of runs.
    pub total_runs: usize,
    /// Successful runs.
    pub successful_runs: usize,
    /// Failed runs.
    pub failed_runs: usize,
    /// Success rate as percentage.
    pub success_rate: f64,
    /// Average total duration.
    pub avg_duration: Option<BuildDuration>,
    /// Fastest run.
    pub fastest_run: Option<BuildDuration>,
    /// Slowest run.
    pub slowest_run: Option<BuildDuration>,
    /// Most recent run status.
    pub last_status: Option<RunStatus>,
}

impl BuildSummary {
    /// Compute summary from a list of pipeline run states.
    #[must_use]
    pub fn from_runs(runs: &[PipelineRunState]) -> Self {
        let total_runs = runs.len();
        let successful_runs = runs
            .iter()
            .filter(|r| r.status == RunStatus::Completed)
            .count();
        let failed_runs = runs
            .iter()
            .filter(|r| r.status == RunStatus::Failed)
            .count();

        let success_rate = if total_runs > 0 {
            successful_runs as f64 / total_runs as f64 * 100.0
        } else {
            0.0
        };

        let durations: Vec<u64> = runs
            .iter()
            .filter_map(|r| r.total_duration.map(|d| d.millis))
            .collect();

        let avg_duration = if durations.is_empty() {
            None
        } else {
            let sum: u64 = durations.iter().sum();
            Some(BuildDuration {
                millis: sum / durations.len() as u64,
            })
        };

        let fastest_run = durations.iter().min().map(|&m| BuildDuration { millis: m });
        let slowest_run = durations.iter().max().map(|&m| BuildDuration { millis: m });

        let last_status = runs.first().map(|r| r.status);

        Self {
            total_runs,
            successful_runs,
            failed_runs,
            success_rate,
            avg_duration,
            fastest_run,
            slowest_run,
            last_status,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::StageId;

    fn make_run(status: RunStatus, duration_ms: u64) -> PipelineRunState {
        PipelineRunState {
            id: crate::types::PipelineId(format!("run-{duration_ms}")),
            definition_name: "test".into(),
            status,
            stages: vec![],
            started_at: chrono::Utc::now(),
            ended_at: Some(chrono::Utc::now()),
            total_duration: Some(BuildDuration {
                millis: duration_ms,
            }),
            source_hash: None,
            workspace_root: "/tmp".into(),
        }
    }

    #[test]
    fn summary_empty() {
        let summary = BuildSummary::from_runs(&[]);
        assert_eq!(summary.total_runs, 0);
        assert_eq!(summary.success_rate, 0.0);
        assert!(summary.avg_duration.is_none());
        assert!(summary.last_status.is_none());
    }

    #[test]
    fn summary_all_success() {
        let runs = vec![
            make_run(RunStatus::Completed, 1000),
            make_run(RunStatus::Completed, 3000),
        ];
        let summary = BuildSummary::from_runs(&runs);
        assert_eq!(summary.total_runs, 2);
        assert_eq!(summary.successful_runs, 2);
        assert_eq!(summary.failed_runs, 0);
        assert!((summary.success_rate - 100.0).abs() < f64::EPSILON);
        assert_eq!(summary.avg_duration.map(|d| d.millis), Some(2000));
        assert_eq!(summary.fastest_run.map(|d| d.millis), Some(1000));
        assert_eq!(summary.slowest_run.map(|d| d.millis), Some(3000));
    }

    #[test]
    fn summary_mixed() {
        let runs = vec![
            make_run(RunStatus::Completed, 2000),
            make_run(RunStatus::Failed, 1000),
        ];
        let summary = BuildSummary::from_runs(&runs);
        assert_eq!(summary.total_runs, 2);
        assert_eq!(summary.successful_runs, 1);
        assert_eq!(summary.failed_runs, 1);
        assert!((summary.success_rate - 50.0).abs() < f64::EPSILON);
    }

    #[test]
    fn summary_last_status() {
        let runs = vec![
            make_run(RunStatus::Failed, 500),
            make_run(RunStatus::Completed, 1000),
        ];
        let summary = BuildSummary::from_runs(&runs);
        assert_eq!(summary.last_status, Some(RunStatus::Failed));
    }
}
