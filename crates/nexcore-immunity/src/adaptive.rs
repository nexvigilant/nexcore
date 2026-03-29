// Copyright (c) 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! NMD adaptive learning — accumulates degradation events and computes
//! threshold adjustments for spliceosome template refinement.
//!
//! ## Biology Analog
//!
//! After SMG-mediated degradation, the cell feeds failure patterns back
//! to improve splicing accuracy for future transcripts. Similarly,
//! `NmdAdaptiveEngine` accumulates abort/flag events and produces
//! calibration recommendations for UPF thresholds.
//!
//! ## Design
//!
//! Pure data transformation — no direct brain mutation. Produces
//! `NmdLearningEvent` structs that the orchestration layer (friday/brain)
//! maps to belief updates and trust recordings.
//!
//! ## Primitive Grounding: ρ(Recursion) + ν(Frequency) + μ(Mapping)

use crate::co_translational::UpfChannel;
use crate::smg::SmgAction;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A single degradation event recorded for adaptive learning.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DegradationEvent {
    /// Task category that was degraded.
    pub category: String,
    /// UPF channels that triggered the degradation.
    pub channels: Vec<UpfChannel>,
    /// Peak severity among anomalies.
    pub max_severity: f32,
    /// Unix timestamp (seconds) when the degradation occurred.
    pub timestamp_secs: u64,
}

/// Accumulated statistics per task category.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CategoryStats {
    /// Total pipeline runs observed (successes + degradations).
    pub total_runs: u32,
    /// Number of runs that resulted in degradation.
    pub degradation_count: u32,
    /// Channel display name -> hit count.
    pub channel_hits: HashMap<String, u32>,
    /// Running average severity of degradation events.
    pub avg_severity: f32,
    /// Maximum severity seen.
    pub max_severity: f32,
}

/// A threshold adjustment recommendation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThresholdAdjustment {
    /// Task category this applies to.
    pub category: String,
    /// UPF config parameter name to adjust.
    pub parameter: String,
    /// Current threshold value.
    pub current_value: f32,
    /// Recommended new value.
    pub recommended_value: f32,
    /// Human-readable reason for the adjustment.
    pub reason: String,
    /// Confidence in this recommendation (0.0–1.0).
    pub confidence: f32,
}

/// Structured learning event for the orchestration layer.
///
/// The brain/friday layer converts these into actual belief updates,
/// pattern promotions, and trust recordings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NmdLearningEvent {
    /// Record a degradation as evidence on an NMD belief.
    RecordDegradation {
        /// Task category involved.
        category: String,
        /// Descriptive proposition for the belief.
        proposition: String,
        /// Evidence weight (negative = degradation occurred).
        evidence_weight: f64,
    },
    /// Recommend spliceosome threshold adjustments.
    AdjustThresholds {
        /// Recommended parameter changes.
        adjustments: Vec<ThresholdAdjustment>,
    },
    /// Record a trust event for the NMD surveillance domain.
    RecordTrustEvent {
        /// Trust domain (always "nmd_surveillance").
        domain: String,
        /// Whether surveillance succeeded (caught real issue) or failed.
        success: bool,
    },
}

/// Configuration for the adaptive learning engine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdaptiveConfig {
    /// Minimum degradation events before suggesting threshold adjustments.
    pub min_events_for_adjustment: u32,
    /// Degradation rate threshold to trigger adjustment (default: 0.3 = 30%).
    pub adjustment_trigger_rate: f32,
    /// Maximum threshold change per adjustment step (default: 0.1).
    pub max_threshold_delta: f32,
}

impl Default for AdaptiveConfig {
    fn default() -> Self {
        Self {
            min_events_for_adjustment: 5,
            adjustment_trigger_rate: 0.3,
            max_threshold_delta: 0.1,
        }
    }
}

/// The NMD adaptive learning engine.
///
/// Accumulates degradation events per task category and produces
/// `NmdLearningEvent`s for the orchestration layer. Stateful but
/// side-effect-free: call `process_adaptive_action` to feed events,
/// read `category_stats` and `degradation_rate` for introspection.
#[derive(Debug, Clone)]
pub struct NmdAdaptiveEngine {
    config: AdaptiveConfig,
    events: Vec<DegradationEvent>,
    category_stats: HashMap<String, CategoryStats>,
}

impl Default for NmdAdaptiveEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl NmdAdaptiveEngine {
    /// Create with default configuration.
    #[must_use]
    pub fn new() -> Self {
        Self {
            config: AdaptiveConfig::default(),
            events: Vec::new(),
            category_stats: HashMap::new(),
        }
    }

    /// Create with custom configuration.
    #[must_use]
    pub fn with_config(config: AdaptiveConfig) -> Self {
        Self {
            config,
            events: Vec::new(),
            category_stats: HashMap::new(),
        }
    }

    /// Record a successful pipeline completion (no degradation).
    pub fn record_success(&mut self, category: &str) {
        let stats = self.category_stats.entry(category.to_string()).or_default();
        stats.total_runs += 1;
    }

    /// Process an `SmgAction::AdaptiveUpdate` into learning events.
    ///
    /// Non-AdaptiveUpdate actions are ignored (returns empty vec).
    #[must_use]
    pub fn process_adaptive_action(&mut self, action: &SmgAction) -> Vec<NmdLearningEvent> {
        let SmgAction::AdaptiveUpdate { category, details } = action else {
            return Vec::new();
        };

        let max_severity = details
            .get("max_severity")
            .and_then(serde_json::Value::as_f64)
            .map_or(0.0, |v| v as f32);

        let channels = extract_channels(details);

        // Record the event
        let event = DegradationEvent {
            category: category.clone(),
            channels: channels.clone(),
            max_severity,
            timestamp_secs: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
        };
        self.events.push(event);

        // Update category stats
        let stats = self.category_stats.entry(category.clone()).or_default();
        stats.total_runs += 1;
        stats.degradation_count += 1;

        if max_severity > stats.max_severity {
            stats.max_severity = max_severity;
        }

        // Recompute average severity from all events in this category
        let (sum, count) = self
            .events
            .iter()
            .filter(|e| e.category == *category)
            .fold((0.0f32, 0u32), |(s, c), e| (s + e.max_severity, c + 1));
        #[allow(clippy::cast_precision_loss)] // count is small enough for f32
        {
            stats.avg_severity = if count > 0 { sum / count as f32 } else { 0.0 };
        }

        // Track channel hit counts
        for ch in &channels {
            let ch_name = format!("{ch}");
            *stats.channel_hits.entry(ch_name).or_insert(0) += 1;
        }

        // Generate learning events
        let mut learning_events = Vec::new();

        // 1. Record degradation as belief evidence
        learning_events.push(NmdLearningEvent::RecordDegradation {
            category: category.clone(),
            proposition: format!(
                "Task category '{}' degradation rate: {:.1}%",
                category,
                self.degradation_rate(category) * 100.0
            ),
            evidence_weight: -f64::from(max_severity),
        });

        // 2. Record trust event (degradation = the system caught something)
        learning_events.push(NmdLearningEvent::RecordTrustEvent {
            domain: "nmd_surveillance".to_string(),
            success: false,
        });

        // 3. Check if threshold adjustment is warranted
        if let Some(adjustments) = self.compute_adjustments(category) {
            learning_events.push(NmdLearningEvent::AdjustThresholds { adjustments });
        }

        learning_events
    }

    /// Get the degradation rate for a category (0.0–1.0).
    #[must_use]
    pub fn degradation_rate(&self, category: &str) -> f32 {
        self.category_stats.get(category).map_or(0.0, |s| {
            if s.total_runs == 0 {
                0.0
            } else {
                #[allow(clippy::cast_precision_loss)]
                // u32 values within f32 range for rate calculations
                {
                    s.degradation_count as f32 / s.total_runs as f32
                }
            }
        })
    }

    /// Get stats for a category.
    #[must_use]
    pub fn category_stats(&self, category: &str) -> Option<&CategoryStats> {
        self.category_stats.get(category)
    }

    /// Get all category stats.
    #[must_use]
    pub fn all_stats(&self) -> &HashMap<String, CategoryStats> {
        &self.category_stats
    }

    /// Total degradation events recorded.
    #[must_use]
    pub fn total_events(&self) -> usize {
        self.events.len()
    }

    /// Compute threshold adjustments if degradation rate exceeds trigger.
    fn compute_adjustments(&self, category: &str) -> Option<Vec<ThresholdAdjustment>> {
        let stats = self.category_stats.get(category)?;

        // Need minimum events before adjusting
        if stats.total_runs < self.config.min_events_for_adjustment {
            return None;
        }

        let deg_rate = self.degradation_rate(category);
        if deg_rate < self.config.adjustment_trigger_rate {
            return None;
        }

        let mut adjustments = Vec::new();

        // Find the dominant failing channel
        if let Some((dominant_channel, &count)) = stats.channel_hits.iter().max_by_key(|&(_, c)| *c)
        {
            #[allow(clippy::cast_precision_loss)]
            // u32 values within f32 range for rate calculations
            let channel_rate = count as f32 / stats.degradation_count as f32;

            // If one channel accounts for >50% of failures, recommend tightening
            if channel_rate > 0.5 {
                let (param, current, direction) = if dominant_channel.starts_with("UPF1") {
                    // Phase order violations — no numeric threshold to adjust,
                    // but we can flag for stricter enforcement
                    ("phase_order_strictness", 1.0f32, 0.0f32)
                } else if dominant_channel.starts_with("UPF2") {
                    // Tool drift — lower threshold = stricter
                    ("tool_drift_threshold", 0.5f32, -1.0f32)
                } else if dominant_channel.starts_with("UPF3") {
                    // Grounding — higher threshold = stricter
                    ("grounding_ratio_threshold", 0.3f32, 1.0f32)
                } else {
                    return None;
                };

                // Skip UPF1 since it has no numeric threshold
                if direction != 0.0 {
                    let delta = self.config.max_threshold_delta * direction;
                    let recommended = current + delta;

                    adjustments.push(ThresholdAdjustment {
                        category: category.to_string(),
                        parameter: param.to_string(),
                        current_value: current,
                        recommended_value: recommended,
                        reason: format!(
                            "{dominant_channel} accounts for {:.0}% of degradations in '{}' \
                             (rate: {:.1}%)",
                            channel_rate * 100.0,
                            category,
                            deg_rate * 100.0,
                        ),
                        #[allow(clippy::cast_precision_loss)] // u32 within f32 range
                        confidence: channel_rate * (1.0 - 1.0 / stats.total_runs as f32),
                    });
                }
            }
        }

        if adjustments.is_empty() {
            None
        } else {
            Some(adjustments)
        }
    }
}

/// Extract UPF channels from `AdaptiveUpdate` details JSON.
///
/// Expects the format produced by `SmgComplex::build_degradation_actions`:
/// ```json
/// { "anomalies": [{ "channel": "UPF2-ToolDrift", ... }] }
/// ```
fn extract_channels(details: &serde_json::Value) -> Vec<UpfChannel> {
    details
        .get("anomalies")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|a| {
                    a.get("channel").and_then(|c| c.as_str()).and_then(|s| {
                        if s.starts_with("UPF1") {
                            Some(UpfChannel::Upf1)
                        } else if s.starts_with("UPF2") {
                            Some(UpfChannel::Upf2)
                        } else if s.starts_with("UPF3") {
                            Some(UpfChannel::Upf3)
                        } else {
                            None
                        }
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
    use super::*;

    fn make_adaptive_action(category: &str, severity: f32, channel: &str) -> SmgAction {
        SmgAction::AdaptiveUpdate {
            category: category.to_string(),
            details: serde_json::json!({
                "anomalies": [{
                    "channel": channel,
                    "severity": severity,
                    "description": "test anomaly",
                }],
                "max_severity": severity,
                "action": "refine_ejc_templates",
            }),
        }
    }

    #[test]
    fn test_process_non_adaptive_action_ignored() {
        let mut engine = NmdAdaptiveEngine::new();
        let action = SmgAction::AbortPipeline {
            reason: "test".to_string(),
            contributing_channels: vec![],
        };
        let events = engine.process_adaptive_action(&action);
        assert!(events.is_empty());
    }

    #[test]
    fn test_process_single_degradation() {
        let mut engine = NmdAdaptiveEngine::new();
        let action = make_adaptive_action("Explore", 0.8, "UPF2-ToolDrift");

        let events = engine.process_adaptive_action(&action);

        // Should produce at least RecordDegradation + RecordTrustEvent
        assert!(events.len() >= 2);
        assert!(
            events
                .iter()
                .any(|e| matches!(e, NmdLearningEvent::RecordDegradation { .. }))
        );
        assert!(
            events
                .iter()
                .any(|e| matches!(e, NmdLearningEvent::RecordTrustEvent { .. }))
        );

        assert_eq!(engine.total_events(), 1);
        assert!((engine.degradation_rate("Explore") - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_record_success_lowers_rate() {
        let mut engine = NmdAdaptiveEngine::new();

        // 1 degradation
        engine.process_adaptive_action(&make_adaptive_action("Mutate", 0.6, "UPF3-Grounding"));

        // 4 successes
        for _ in 0..4 {
            engine.record_success("Mutate");
        }

        // Rate = 1/5 = 0.2
        assert!((engine.degradation_rate("Mutate") - 0.2).abs() < f32::EPSILON);
    }

    #[test]
    fn test_category_stats_tracking() {
        let mut engine = NmdAdaptiveEngine::new();

        engine.process_adaptive_action(&make_adaptive_action("Explore", 0.5, "UPF2-ToolDrift"));
        engine.process_adaptive_action(&make_adaptive_action("Explore", 0.9, "UPF1-PhaseOrder"));

        let stats = engine.category_stats("Explore");
        assert!(stats.is_some());

        let default_stats = CategoryStats::default();
        let stats = stats.unwrap_or(&default_stats);
        assert_eq!(stats.degradation_count, 2);
        assert!((stats.max_severity - 0.9).abs() < f32::EPSILON);
    }

    #[test]
    fn test_threshold_adjustment_below_min_events() {
        let mut engine = NmdAdaptiveEngine::new();

        // Only 1 event — below min_events_for_adjustment (5)
        engine.process_adaptive_action(&make_adaptive_action("Compute", 0.8, "UPF2-ToolDrift"));

        let events =
            engine.process_adaptive_action(&make_adaptive_action("Compute", 0.7, "UPF2-ToolDrift"));

        // Should NOT produce AdjustThresholds (only 2 events, need 5)
        assert!(
            !events
                .iter()
                .any(|e| matches!(e, NmdLearningEvent::AdjustThresholds { .. }))
        );
    }

    #[test]
    fn test_threshold_adjustment_triggered() {
        let config = AdaptiveConfig {
            min_events_for_adjustment: 3,
            adjustment_trigger_rate: 0.3,
            max_threshold_delta: 0.1,
        };
        let mut engine = NmdAdaptiveEngine::with_config(config);

        // 3 degradations, all UPF2 → rate = 100% > 30%, and UPF2 dominance = 100%
        engine.process_adaptive_action(&make_adaptive_action("Explore", 0.8, "UPF2-ToolDrift"));
        engine.process_adaptive_action(&make_adaptive_action("Explore", 0.7, "UPF2-ToolDrift"));
        let events =
            engine.process_adaptive_action(&make_adaptive_action("Explore", 0.9, "UPF2-ToolDrift"));

        // Should produce AdjustThresholds
        let adjust = events
            .iter()
            .find(|e| matches!(e, NmdLearningEvent::AdjustThresholds { .. }));
        assert!(adjust.is_some());

        if let Some(NmdLearningEvent::AdjustThresholds { adjustments }) = adjust {
            assert_eq!(adjustments.len(), 1);
            assert_eq!(adjustments[0].parameter, "tool_drift_threshold");
            // Current 0.5, direction -1.0, delta 0.1 → recommended = 0.4
            assert!((adjustments[0].recommended_value - 0.4).abs() < f32::EPSILON);
        }
    }

    #[test]
    fn test_no_adjustment_when_rate_below_trigger() {
        let config = AdaptiveConfig {
            min_events_for_adjustment: 3,
            adjustment_trigger_rate: 0.5,
            max_threshold_delta: 0.1,
        };
        let mut engine = NmdAdaptiveEngine::with_config(config);

        // 1 degradation + 4 successes = rate 20% < 50% trigger
        engine.process_adaptive_action(&make_adaptive_action("Mutate", 0.8, "UPF2-ToolDrift"));
        for _ in 0..4 {
            engine.record_success("Mutate");
        }

        // Add 2 more degradations (total 3 degradations, 7 runs = 42% < 50%)
        engine.process_adaptive_action(&make_adaptive_action("Mutate", 0.7, "UPF2-ToolDrift"));
        let events =
            engine.process_adaptive_action(&make_adaptive_action("Mutate", 0.6, "UPF2-ToolDrift"));

        assert!(
            !events
                .iter()
                .any(|e| matches!(e, NmdLearningEvent::AdjustThresholds { .. }))
        );
    }

    #[test]
    fn test_channel_extraction() {
        let details = serde_json::json!({
            "anomalies": [
                { "channel": "UPF1-PhaseOrder", "severity": 0.5 },
                { "channel": "UPF3-Grounding", "severity": 0.7 },
            ],
            "max_severity": 0.7,
        });

        let channels = extract_channels(&details);
        assert_eq!(channels.len(), 2);
        assert!(channels.contains(&UpfChannel::Upf1));
        assert!(channels.contains(&UpfChannel::Upf3));
    }

    #[test]
    fn test_channel_extraction_empty() {
        let details = serde_json::json!({ "max_severity": 0.5 });
        let channels = extract_channels(&details);
        assert!(channels.is_empty());
    }

    #[test]
    fn test_multiple_categories_independent() {
        let mut engine = NmdAdaptiveEngine::new();

        engine.process_adaptive_action(&make_adaptive_action("Explore", 0.8, "UPF2-ToolDrift"));
        engine.process_adaptive_action(&make_adaptive_action("Mutate", 0.6, "UPF3-Grounding"));

        assert!((engine.degradation_rate("Explore") - 1.0).abs() < f32::EPSILON);
        assert!((engine.degradation_rate("Mutate") - 1.0).abs() < f32::EPSILON);
        assert!((engine.degradation_rate("Compute") - 0.0).abs() < f32::EPSILON);

        let all = engine.all_stats();
        assert_eq!(all.len(), 2);
    }

    #[test]
    fn test_upf3_adjustment_direction() {
        let config = AdaptiveConfig {
            min_events_for_adjustment: 3,
            adjustment_trigger_rate: 0.3,
            max_threshold_delta: 0.1,
        };
        let mut engine = NmdAdaptiveEngine::with_config(config);

        // 3 UPF3 degradations
        for _ in 0..2 {
            engine.process_adaptive_action(&make_adaptive_action("Verify", 0.7, "UPF3-Grounding"));
        }
        let events =
            engine.process_adaptive_action(&make_adaptive_action("Verify", 0.8, "UPF3-Grounding"));

        if let Some(NmdLearningEvent::AdjustThresholds { adjustments }) = events
            .iter()
            .find(|e| matches!(e, NmdLearningEvent::AdjustThresholds { .. }))
        {
            assert_eq!(adjustments[0].parameter, "grounding_ratio_threshold");
            // Direction is +1 for UPF3 (higher = stricter): 0.3 + 0.1 = 0.4
            assert!((adjustments[0].recommended_value - 0.4).abs() < f32::EPSILON);
        } else {
            panic!("Expected AdjustThresholds event");
        }
    }
}
