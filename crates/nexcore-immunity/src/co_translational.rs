// Copyright (c) 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! Co-translational surveillance — UPF complex for NMD pipeline monitoring.
//!
//! ## Biology Analog
//!
//! In biology, the UPF (Up-Frameshift) proteins form a surveillance complex
//! that monitors translation in real-time. UPF1 detects premature stop codons
//! by checking EJC positions, UPF2 bridges UPF1 to the EJC, and UPF3 anchors
//! the complex at the exon junction.
//!
//! ## Purpose
//!
//! This module implements co-translational monitoring for LLM pipeline execution.
//! Given EJC markers from the spliceosome and checkpoint observations from the
//! pipeline, it detects structural anomalies in real-time.
//!
//! ## UPF Channels
//!
//! - **UPF1**: Phase order verification — are execution phases in expected sequence?
//! - **UPF2**: Tool category drift — Jaccard distance between expected and observed categories.
//! - **UPF3**: Grounding requirement satisfaction — is external validation happening?
//!
//! ## Primitive Grounding: κ(Comparison) + σ(Sequence) + ∂(Boundary)

use nexcore_spliceosome::{EjcMarker, TaskCategory};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// UPF verdict — the outcome of co-translational monitoring at a checkpoint.
///
/// ## Tier: T2-C (κ + σ + ∂)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum UpfVerdict {
    /// All structural checks pass — continue execution.
    Continue,
    /// Structural anomaly detected — stall for human review.
    Stall {
        /// Anomalies that triggered the stall.
        anomalies: Vec<UpfAnomaly>,
    },
    /// Critical structural violation — recommend pipeline degradation.
    Degrade {
        /// Anomalies that triggered degradation.
        anomalies: Vec<UpfAnomaly>,
    },
}

/// A specific structural anomaly detected by a UPF channel.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UpfAnomaly {
    /// Which UPF channel detected this anomaly.
    pub channel: UpfChannel,
    /// Human-readable description.
    pub description: String,
    /// Severity score in \[0.0, 1.0\].
    pub severity: f32,
}

/// UPF surveillance channels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum UpfChannel {
    /// Phase order verification.
    Upf1,
    /// Tool category drift detection (Jaccard distance).
    Upf2,
    /// Grounding requirement satisfaction.
    Upf3,
}

impl std::fmt::Display for UpfChannel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Upf1 => write!(f, "UPF1-PhaseOrder"),
            Self::Upf2 => write!(f, "UPF2-ToolDrift"),
            Self::Upf3 => write!(f, "UPF3-Grounding"),
        }
    }
}

/// Configuration for UPF complex thresholds.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpfConfig {
    /// Jaccard distance threshold for UPF2 tool drift (default: 0.5).
    /// Higher values are more permissive.
    pub tool_drift_threshold: f32,
    /// Minimum grounding ratio for UPF3 (default: 0.3).
    /// `grounding_signals / total_calls` must exceed this.
    pub grounding_ratio_threshold: f32,
    /// Number of anomalies before recommending degradation (default: 3).
    pub degrade_threshold: usize,
    /// Severity threshold for individual anomalies to count toward degradation (default: 0.5).
    pub severity_floor: f32,
}

impl Default for UpfConfig {
    fn default() -> Self {
        Self {
            tool_drift_threshold: 0.5,
            grounding_ratio_threshold: 0.3,
            degrade_threshold: 3,
            severity_floor: 0.5,
        }
    }
}

/// Observation data from a pipeline checkpoint.
///
/// This is what the pipeline feeds to the UPF complex at each checkpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckpointObservation {
    /// Which phase is currently executing.
    pub phase_id: String,
    /// Tool categories observed in this phase.
    pub observed_categories: Vec<TaskCategory>,
    /// Number of grounding signals (external validation events).
    pub grounding_signals: u32,
    /// Total tool calls in this phase.
    pub total_calls: u32,
    /// Index of this checkpoint in the execution sequence.
    pub checkpoint_index: usize,
}

/// The UPF surveillance complex — co-translational structural monitor.
///
/// Compares pipeline checkpoint observations against EJC markers
/// deposited by the spliceosome before execution began.
///
/// ## Design Principles
///
/// 1. **Orthogonal to content** — checks structure, not output quality
/// 2. **Real-time** — runs at each checkpoint during execution
/// 3. **Three-channel** — UPF1 (order), UPF2 (categories), UPF3 (grounding)
/// 4. **Graduated response** — Continue -> Stall -> Degrade
#[derive(Debug, Clone)]
pub struct UpfComplex {
    config: UpfConfig,
}

impl Default for UpfComplex {
    fn default() -> Self {
        Self::new()
    }
}

impl UpfComplex {
    /// Create a UPF complex with default thresholds.
    #[must_use]
    pub fn new() -> Self {
        Self {
            config: UpfConfig::default(),
        }
    }

    /// Create a UPF complex with custom configuration.
    #[must_use]
    pub fn with_config(config: UpfConfig) -> Self {
        Self { config }
    }

    /// Get the current configuration.
    #[must_use]
    pub fn config(&self) -> &UpfConfig {
        &self.config
    }

    /// Scan a checkpoint observation against EJC markers.
    ///
    /// This is the core co-translational monitoring operation.
    /// Called at each pipeline checkpoint to detect structural anomalies.
    #[must_use]
    pub fn scan_checkpoint(
        &self,
        observation: &CheckpointObservation,
        markers: &[EjcMarker],
    ) -> UpfVerdict {
        let mut anomalies = Vec::new();

        // UPF1: Phase order check
        if let Some(anomaly) = self.check_phase_order(observation, markers) {
            anomalies.push(anomaly);
        }

        // UPF2: Tool category drift
        if let Some(anomaly) = self.check_tool_drift(observation, markers) {
            anomalies.push(anomaly);
        }

        // UPF3: Grounding requirement
        if let Some(anomaly) = self.check_grounding(observation, markers) {
            anomalies.push(anomaly);
        }

        if anomalies.is_empty() {
            return UpfVerdict::Continue;
        }

        // Count severe anomalies
        let severe_count = anomalies
            .iter()
            .filter(|a| a.severity >= self.config.severity_floor)
            .count();

        if severe_count >= self.config.degrade_threshold {
            UpfVerdict::Degrade { anomalies }
        } else {
            UpfVerdict::Stall { anomalies }
        }
    }

    /// UPF1: Verify phase order matches expected sequence.
    ///
    /// Checks that the current phase_id matches the marker at the
    /// observation's checkpoint index.
    #[allow(clippy::unused_self)] // method kept as instance method for API consistency
    fn check_phase_order(
        &self,
        observation: &CheckpointObservation,
        markers: &[EjcMarker],
    ) -> Option<UpfAnomaly> {
        let expected_marker = markers.get(observation.checkpoint_index)?;

        if observation.phase_id == expected_marker.phase_id {
            None
        } else {
            // Out-of-order is less severe than unknown phase
            let exists_elsewhere = markers
                .iter()
                .any(|m| m.phase_id == observation.phase_id);

            let severity = if exists_elsewhere { 0.6 } else { 0.9 };

            Some(UpfAnomaly {
                channel: UpfChannel::Upf1,
                description: format!(
                    "Phase order violation: expected '{}' at index {}, observed '{}'",
                    expected_marker.phase_id, observation.checkpoint_index, observation.phase_id
                ),
                severity,
            })
        }
    }

    /// UPF2: Detect tool category drift via Jaccard distance.
    ///
    /// Compares observed tool categories against expected categories
    /// from the EJC marker. Drift = 1 - Jaccard(expected, observed).
    fn check_tool_drift(
        &self,
        observation: &CheckpointObservation,
        markers: &[EjcMarker],
    ) -> Option<UpfAnomaly> {
        let marker = markers.get(observation.checkpoint_index)?;

        if marker.expected_tool_categories.is_empty()
            && observation.observed_categories.is_empty()
        {
            return None;
        }

        let expected: HashSet<TaskCategory> =
            marker.expected_tool_categories.iter().copied().collect();
        let observed: HashSet<TaskCategory> =
            observation.observed_categories.iter().copied().collect();

        let intersection = expected.intersection(&observed).count();
        let union = expected.union(&observed).count();

        let drift = if union == 0 {
            0.0
        } else {
            #[allow(clippy::cast_precision_loss)] // set sizes are small enough for f32
            { 1.0 - (intersection as f32 / union as f32) }
        };

        if drift > self.config.tool_drift_threshold {
            Some(UpfAnomaly {
                channel: UpfChannel::Upf2,
                description: format!(
                    "Tool category drift: Jaccard distance {drift:.2} exceeds threshold {:.2} \
                     (expected {:?}, observed {:?})",
                    self.config.tool_drift_threshold,
                    marker.expected_tool_categories,
                    observation.observed_categories
                ),
                severity: drift.min(1.0),
            })
        } else {
            None
        }
    }

    /// UPF3: Check grounding requirement satisfaction.
    ///
    /// Verifies that the ratio of grounding signals to total calls
    /// meets the marker's grounding confidence threshold.
    #[allow(clippy::unused_self)] // method kept as instance method for API consistency
    fn check_grounding(
        &self,
        observation: &CheckpointObservation,
        markers: &[EjcMarker],
    ) -> Option<UpfAnomaly> {
        let marker = markers.get(observation.checkpoint_index)?;

        if observation.total_calls == 0 {
            return None;
        }

        #[allow(clippy::cast_precision_loss)] // u32 values within f32 range for ratio calculation
        let grounding_ratio =
            observation.grounding_signals as f32 / observation.total_calls as f32;
        let required = marker.grounding_confidence_threshold;

        if grounding_ratio < required {
            let deficit = required - grounding_ratio;
            Some(UpfAnomaly {
                channel: UpfChannel::Upf3,
                description: format!(
                    "Grounding deficit: ratio {grounding_ratio:.2} below required {required:.2} \
                     (deficit: {deficit:.2})",
                ),
                severity: (deficit * 2.0).min(1.0),
            })
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_marker(phase: &str, categories: Vec<TaskCategory>, grounding: f32) -> EjcMarker {
        EjcMarker {
            phase_id: phase.to_string(),
            expected_tool_categories: categories,
            grounding_confidence_threshold: grounding,
            max_calls_before_checkpoint: 20,
            expected_confidence_range: (0.5, 1.0),
            skippable: false,
        }
    }

    fn make_observation(
        phase: &str,
        categories: Vec<TaskCategory>,
        grounding: u32,
        total: u32,
        index: usize,
    ) -> CheckpointObservation {
        CheckpointObservation {
            phase_id: phase.to_string(),
            observed_categories: categories,
            grounding_signals: grounding,
            total_calls: total,
            checkpoint_index: index,
        }
    }

    #[test]
    fn test_all_pass_returns_continue() {
        let upf = UpfComplex::new();
        let markers = vec![make_marker(
            "investigate",
            vec![TaskCategory::Explore],
            0.3,
        )];
        let obs = make_observation("investigate", vec![TaskCategory::Explore], 5, 10, 0);
        assert_eq!(upf.scan_checkpoint(&obs, &markers), UpfVerdict::Continue);
    }

    #[test]
    fn test_phase_order_violation() {
        let upf = UpfComplex::new();
        let markers = vec![
            make_marker("investigate", vec![TaskCategory::Explore], 0.3),
            make_marker("implement", vec![TaskCategory::Mutate], 0.3),
        ];
        let obs = make_observation("implement", vec![TaskCategory::Explore], 5, 10, 0);
        let verdict = upf.scan_checkpoint(&obs, &markers);
        match verdict {
            UpfVerdict::Stall { anomalies } | UpfVerdict::Degrade { anomalies } => {
                assert!(anomalies.iter().any(|a| a.channel == UpfChannel::Upf1));
            }
            UpfVerdict::Continue => panic!("Expected stall for phase order violation"),
        }
    }

    #[test]
    fn test_tool_drift_detection() {
        let upf = UpfComplex::new();
        let markers = vec![make_marker(
            "investigate",
            vec![TaskCategory::Explore],
            0.0,
        )];
        let obs = make_observation("investigate", vec![TaskCategory::Compute], 0, 10, 0);
        let verdict = upf.scan_checkpoint(&obs, &markers);
        match verdict {
            UpfVerdict::Stall { anomalies } | UpfVerdict::Degrade { anomalies } => {
                assert!(anomalies.iter().any(|a| a.channel == UpfChannel::Upf2));
            }
            UpfVerdict::Continue => panic!("Expected stall for tool drift"),
        }
    }

    #[test]
    fn test_grounding_deficit() {
        let upf = UpfComplex::new();
        let markers = vec![make_marker(
            "compute",
            vec![TaskCategory::Compute],
            0.7,
        )];
        let obs = make_observation("compute", vec![TaskCategory::Compute], 1, 10, 0);
        let verdict = upf.scan_checkpoint(&obs, &markers);
        match verdict {
            UpfVerdict::Stall { anomalies } | UpfVerdict::Degrade { anomalies } => {
                assert!(anomalies.iter().any(|a| a.channel == UpfChannel::Upf3));
            }
            UpfVerdict::Continue => panic!("Expected stall for grounding deficit"),
        }
    }

    #[test]
    fn test_degrade_on_multiple_severe_anomalies() {
        let config = UpfConfig {
            tool_drift_threshold: 0.0,
            grounding_ratio_threshold: 0.0,
            degrade_threshold: 2,
            severity_floor: 0.3,
        };
        let upf = UpfComplex::with_config(config);
        let markers = vec![make_marker(
            "investigate",
            vec![TaskCategory::Explore],
            0.8,
        )];
        let obs = make_observation("implement", vec![TaskCategory::Browse], 0, 10, 0);
        let verdict = upf.scan_checkpoint(&obs, &markers);
        assert!(matches!(verdict, UpfVerdict::Degrade { .. }));
    }

    #[test]
    fn test_empty_markers_returns_continue() {
        let upf = UpfComplex::new();
        let obs = make_observation("investigate", vec![TaskCategory::Explore], 5, 10, 0);
        assert_eq!(upf.scan_checkpoint(&obs, &[]), UpfVerdict::Continue);
    }

    #[test]
    fn test_zero_calls_skips_grounding() {
        let upf = UpfComplex::new();
        let markers = vec![make_marker(
            "investigate",
            vec![TaskCategory::Explore],
            0.9,
        )];
        let obs = make_observation("investigate", vec![TaskCategory::Explore], 0, 0, 0);
        assert_eq!(upf.scan_checkpoint(&obs, &markers), UpfVerdict::Continue);
    }

    #[test]
    fn test_jaccard_identical_sets() {
        let upf = UpfComplex::new();
        let cats = vec![TaskCategory::Explore, TaskCategory::Mutate];
        let markers = vec![make_marker("phase", cats.clone(), 0.0)];
        let obs = make_observation("phase", cats, 0, 10, 0);
        assert_eq!(upf.scan_checkpoint(&obs, &markers), UpfVerdict::Continue);
    }

    #[test]
    fn test_upf_channel_display() {
        assert_eq!(UpfChannel::Upf1.to_string(), "UPF1-PhaseOrder");
        assert_eq!(UpfChannel::Upf2.to_string(), "UPF2-ToolDrift");
        assert_eq!(UpfChannel::Upf3.to_string(), "UPF3-Grounding");
    }
}
