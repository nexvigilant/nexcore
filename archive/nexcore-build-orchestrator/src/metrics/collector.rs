//! Metrics collector — per-stage timing and statistics.
//!
//! ## Primitive Foundation
//! | Primitive | Manifestation |
//! |-----------|---------------|
//! | T1: N (Quantity) | Timing values, counts |
//! | T1: σ (Sequence) | Ordered measurements |
//! | T1: μ (Mapping) | Stage → timing map |

use crate::pipeline::state::PipelineRunState;
use crate::types::{BuildDuration, StageId};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Timing data for a single stage.
///
/// Tier: T2-P (N + μ, dominant N)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StageTiming {
    pub stage_id: StageId,
    pub duration: BuildDuration,
    pub success: bool,
}

/// Collect timing data from a completed pipeline run.
#[must_use]
pub fn collect_timings(state: &PipelineRunState) -> Vec<StageTiming> {
    state
        .stages
        .iter()
        .filter_map(|s| {
            s.duration.map(|d| StageTiming {
                stage_id: s.stage_id.clone(),
                duration: d,
                success: s.status.is_success(),
            })
        })
        .collect()
}

/// Aggregate timing statistics across multiple runs.
///
/// Tier: T2-C (N + μ + σ + κ, dominant N)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimingAggregates {
    /// Stage name → list of durations (ms).
    pub stage_durations: HashMap<String, Vec<u64>>,
    /// Total pipeline durations (ms).
    pub pipeline_durations: Vec<u64>,
}

impl TimingAggregates {
    /// Create empty aggregates.
    #[must_use]
    pub fn new() -> Self {
        Self {
            stage_durations: HashMap::new(),
            pipeline_durations: Vec::new(),
        }
    }

    /// Add data from a pipeline run.
    pub fn add_run(&mut self, state: &PipelineRunState) {
        if let Some(total) = &state.total_duration {
            self.pipeline_durations.push(total.millis);
        }
        for timing in collect_timings(state) {
            self.stage_durations
                .entry(timing.stage_id.0)
                .or_default()
                .push(timing.duration.millis);
        }
    }

    /// Get average pipeline duration in ms.
    #[must_use]
    pub fn avg_pipeline_ms(&self) -> Option<u64> {
        if self.pipeline_durations.is_empty() {
            return None;
        }
        let sum: u64 = self.pipeline_durations.iter().sum();
        Some(sum / self.pipeline_durations.len() as u64)
    }

    /// Get average duration for a stage in ms.
    #[must_use]
    pub fn avg_stage_ms(&self, stage: &str) -> Option<u64> {
        let durations = self.stage_durations.get(stage)?;
        if durations.is_empty() {
            return None;
        }
        let sum: u64 = durations.iter().sum();
        Some(sum / durations.len() as u64)
    }
}

impl Default for TimingAggregates {
    fn default() -> Self {
        Self::new()
    }
}
