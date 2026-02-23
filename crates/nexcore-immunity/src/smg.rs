// Copyright (c) 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! SMG degradation complex — bridges UPF verdicts to degradation actions.
//!
//! ## Biology Analog
//!
//! In biology, the SMG (Suppressor with Morphogenetic effect on Genitalia)
//! proteins execute the degradation pathway after NMD surveillance detects
//! a premature termination codon:
//!
//! - **SMG5/7**: Recruit decapping enzymes and the 5'->3' exonuclease Xrn1
//! - **SMG6**: Endonucleolytic cleavage near the PTC
//!
//! Both pathways converge on mRNA destruction, but SMG6 also feeds back
//! to the spliceosome to prevent re-splicing of the same defective transcript.
//!
//! ## Purpose
//!
//! Converts UPF complex verdicts into concrete degradation actions that the
//! orchestration layer can execute. The SMG complex is a pure function:
//! verdict in, actions out. It does NOT emit signals directly — the caller
//! maps actions to cytokine emissions.
//!
//! ## Primitive Grounding: ->(Causality) + void(Void) + mu(Mapping)

use crate::co_translational::{UpfAnomaly, UpfChannel, UpfVerdict};
use serde::{Deserialize, Serialize};

/// An action to be executed by the orchestration layer.
///
/// The SMG complex produces actions; the caller executes them.
/// This separation keeps immunity orthogonal to the cytokine bus.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SmgAction {
    /// Abort the pipeline execution (SMG5 analog).
    AbortPipeline {
        /// Why the pipeline should be aborted.
        reason: String,
        /// Which UPF channels contributed to this decision.
        contributing_channels: Vec<UpfChannel>,
    },
    /// Flag the source artifact for review (SMG6 analog).
    /// Prevents the defective task spec from being reused.
    FlagSource {
        /// Identifier of the source to flag (e.g., task hash).
        source_id: String,
        /// Why the source is being flagged.
        reason: String,
    },
    /// Request an adaptive update to spliceosome templates (SMG7 analog).
    /// Feeds failure patterns back to improve future expectations.
    AdaptiveUpdate {
        /// Which task category needs template refinement.
        category: String,
        /// Structured update details.
        details: serde_json::Value,
    },
}

/// Configuration for the SMG degradation complex.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmgConfig {
    /// Minimum severity to trigger source flagging (default: 0.7).
    pub flag_source_severity: f32,
    /// Whether to produce adaptive updates on every degradation (default: true).
    pub adaptive_feedback_enabled: bool,
    /// Task hash/ID for the current pipeline (set by caller).
    pub current_task_id: Option<String>,
}

impl Default for SmgConfig {
    fn default() -> Self {
        Self {
            flag_source_severity: 0.7,
            adaptive_feedback_enabled: true,
            current_task_id: None,
        }
    }
}

/// The SMG degradation complex — converts UPF verdicts to executable actions.
///
/// Pure function: verdict in, actions out. No side effects.
///
/// ## Design Principles
///
/// 1. **No direct signal emission** — produces SmgAction data, caller emits
/// 2. **Graduated response** — severity determines which actions fire
/// 3. **Adaptive feedback** — degradation events feed back to spliceosome
#[derive(Debug, Clone)]
pub struct SmgComplex {
    config: SmgConfig,
}

impl Default for SmgComplex {
    fn default() -> Self {
        Self::new()
    }
}

impl SmgComplex {
    /// Create with default configuration.
    #[must_use]
    pub fn new() -> Self {
        Self {
            config: SmgConfig::default(),
        }
    }

    /// Create with custom configuration.
    #[must_use]
    pub fn with_config(config: SmgConfig) -> Self {
        Self { config }
    }

    /// Set the current task ID for source flagging.
    pub fn set_task_id(&mut self, task_id: impl Into<String>) {
        self.config.current_task_id = Some(task_id.into());
    }

    /// Process a UPF verdict into degradation actions.
    ///
    /// - `Continue` -> no actions
    /// - `Stall` -> no actions (stalls are handled by the pipeline, not SMG)
    /// - `Degrade` -> abort + optional flag source + optional adaptive update
    #[must_use]
    pub fn process_verdict(&self, verdict: &UpfVerdict) -> Vec<SmgAction> {
        match verdict {
            UpfVerdict::Continue | UpfVerdict::Stall { .. } => Vec::new(),
            UpfVerdict::Degrade { anomalies } => self.build_degradation_actions(anomalies),
        }
    }

    /// Build the full set of degradation actions from anomalies.
    fn build_degradation_actions(&self, anomalies: &[UpfAnomaly]) -> Vec<SmgAction> {
        let mut actions = Vec::new();

        // SMG5: Always abort on degradation
        let contributing_channels: Vec<UpfChannel> =
            anomalies.iter().map(|a| a.channel).collect();
        let reasons: Vec<String> = anomalies.iter().map(|a| a.description.clone()).collect();
        let reason = reasons.join("; ");

        actions.push(SmgAction::AbortPipeline {
            reason,
            contributing_channels,
        });

        // SMG6: Flag source if any anomaly exceeds severity threshold
        let max_severity = anomalies
            .iter()
            .map(|a| a.severity)
            .fold(0.0f32, f32::max);

        if max_severity >= self.config.flag_source_severity {
            let source_id = self
                .config
                .current_task_id
                .clone()
                .unwrap_or_else(|| "unknown".to_string());

            actions.push(SmgAction::FlagSource {
                source_id,
                reason: format!(
                    "Severity {max_severity:.2} exceeds flag threshold {:.2}",
                    self.config.flag_source_severity
                ),
            });
        }

        // SMG7: Adaptive feedback
        if self.config.adaptive_feedback_enabled {
            let channel_summary: Vec<serde_json::Value> = anomalies
                .iter()
                .map(|a| {
                    serde_json::json!({
                        "channel": format!("{}", a.channel),
                        "severity": a.severity,
                        "description": a.description,
                    })
                })
                .collect();

            actions.push(SmgAction::AdaptiveUpdate {
                category: "unknown".to_string(), // Caller should override with actual category
                details: serde_json::json!({
                    "anomalies": channel_summary,
                    "max_severity": max_severity,
                    "action": "refine_ejc_templates",
                }),
            });
        }

        actions
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_anomaly(channel: UpfChannel, severity: f32) -> UpfAnomaly {
        UpfAnomaly {
            channel,
            description: format!("{channel} anomaly"),
            severity,
        }
    }

    #[test]
    fn test_continue_produces_no_actions() {
        let smg = SmgComplex::new();
        let actions = smg.process_verdict(&UpfVerdict::Continue);
        assert!(actions.is_empty());
    }

    #[test]
    fn test_stall_produces_no_actions() {
        let smg = SmgComplex::new();
        let verdict = UpfVerdict::Stall {
            anomalies: vec![make_anomaly(UpfChannel::Upf1, 0.5)],
        };
        let actions = smg.process_verdict(&verdict);
        assert!(actions.is_empty());
    }

    #[test]
    fn test_degrade_produces_abort() {
        let smg = SmgComplex::new();
        let verdict = UpfVerdict::Degrade {
            anomalies: vec![make_anomaly(UpfChannel::Upf1, 0.8)],
        };
        let actions = smg.process_verdict(&verdict);
        assert!(actions.iter().any(|a| matches!(a, SmgAction::AbortPipeline { .. })));
    }

    #[test]
    fn test_high_severity_flags_source() {
        let mut smg = SmgComplex::new();
        smg.set_task_id("task-123");
        let verdict = UpfVerdict::Degrade {
            anomalies: vec![make_anomaly(UpfChannel::Upf2, 0.9)],
        };
        let actions = smg.process_verdict(&verdict);
        let flag = actions.iter().find(|a| matches!(a, SmgAction::FlagSource { .. }));
        assert!(flag.is_some());
        if let Some(SmgAction::FlagSource { source_id, .. }) = flag {
            assert_eq!(source_id, "task-123");
        }
    }

    #[test]
    fn test_low_severity_no_flag() {
        let smg = SmgComplex::new();
        let verdict = UpfVerdict::Degrade {
            anomalies: vec![make_anomaly(UpfChannel::Upf3, 0.3)], // Below 0.7 threshold
        };
        let actions = smg.process_verdict(&verdict);
        assert!(!actions.iter().any(|a| matches!(a, SmgAction::FlagSource { .. })));
    }

    #[test]
    fn test_adaptive_update_produced() {
        let smg = SmgComplex::new();
        let verdict = UpfVerdict::Degrade {
            anomalies: vec![make_anomaly(UpfChannel::Upf1, 0.5)],
        };
        let actions = smg.process_verdict(&verdict);
        assert!(actions.iter().any(|a| matches!(a, SmgAction::AdaptiveUpdate { .. })));
    }

    #[test]
    fn test_adaptive_disabled() {
        let config = SmgConfig {
            adaptive_feedback_enabled: false,
            ..SmgConfig::default()
        };
        let smg = SmgComplex::with_config(config);
        let verdict = UpfVerdict::Degrade {
            anomalies: vec![make_anomaly(UpfChannel::Upf1, 0.5)],
        };
        let actions = smg.process_verdict(&verdict);
        assert!(!actions.iter().any(|a| matches!(a, SmgAction::AdaptiveUpdate { .. })));
    }

    #[test]
    fn test_multiple_anomalies_in_abort_reason() {
        let smg = SmgComplex::new();
        let verdict = UpfVerdict::Degrade {
            anomalies: vec![
                make_anomaly(UpfChannel::Upf1, 0.6),
                make_anomaly(UpfChannel::Upf2, 0.7),
                make_anomaly(UpfChannel::Upf3, 0.8),
            ],
        };
        let actions = smg.process_verdict(&verdict);
        if let Some(SmgAction::AbortPipeline { contributing_channels, .. }) = actions.first() {
            assert_eq!(contributing_channels.len(), 3);
        } else {
            panic!("Expected AbortPipeline as first action");
        }
    }

    #[test]
    fn test_full_degradation_pipeline() {
        let mut smg = SmgComplex::new();
        smg.set_task_id("spec-456");

        let verdict = UpfVerdict::Degrade {
            anomalies: vec![
                make_anomaly(UpfChannel::Upf1, 0.9),
                make_anomaly(UpfChannel::Upf2, 0.8),
            ],
        };

        let actions = smg.process_verdict(&verdict);

        // Should have all 3 action types: abort + flag + adaptive
        assert_eq!(actions.len(), 3);
        assert!(matches!(&actions[0], SmgAction::AbortPipeline { .. }));
        assert!(matches!(&actions[1], SmgAction::FlagSource { .. }));
        assert!(matches!(&actions[2], SmgAction::AdaptiveUpdate { .. }));
    }
}
