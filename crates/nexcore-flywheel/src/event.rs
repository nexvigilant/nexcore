//! Flywheel event envelope and event kinds.

use crate::node::FlywheelTier;
use nexcore_chrono::DateTime;
use serde::{Deserialize, Serialize};

/// Typed event payloads carried on the flywheel bus.
///
/// Each variant represents a distinct signal that nodes emit and consume.
/// Tagged serialization (`"type": "snake_case"`) for JSON wire format.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum EventKind {
    /// A node completed one full processing cycle.
    CycleComplete {
        /// Monotonic iteration counter for this node.
        iteration: u64,
    },
    /// A homeostatic baseline shifted beyond normal drift.
    BaselineShift {
        /// Name of the metric whose baseline moved.
        metric: String,
        /// Previous baseline value.
        old: f64,
        /// New baseline value.
        new: f64,
    },
    /// A tuning threshold drifted from its calibrated value.
    ThresholdDrift {
        /// Parameter that drifted.
        parameter: String,
        /// Magnitude of drift from calibrated value.
        delta: f64,
    },
    /// A subsystem signals readiness for adaptation.
    AdaptationReady {
        /// Adaptation category (e.g. "sensitivity", "threshold").
        category: String,
    },
    /// Trust engine recorded a score update.
    TrustUpdate {
        /// New trust score (0.0–1.0).
        score: f64,
        /// Trust level label (e.g. "high", "degraded").
        level: String,
    },
    /// Skill maturation engine detected transfer-readiness.
    MaturationSignal {
        /// Skill identifier.
        skill: String,
        /// Cross-domain transfer score (0.0–1.0).
        transfer_score: f64,
    },
    /// Insight engine accumulated new patterns.
    InsightAccumulated {
        /// Number of patterns accumulated in this batch.
        pattern_count: u64,
    },
    /// Fractionation complete — crude separated into typed streams.
    FractionationComplete {
        /// Health-related signals extracted.
        health_count: u64,
        /// Threat signals extracted.
        threat_count: u64,
        /// Learning signals extracted.
        learning_count: u64,
        /// Noise signals stripped.
        noise_stripped: u64,
    },
    /// Yield report — refinery cycle metrics.
    YieldReport {
        /// Overall yield percentage (0.0–100.0).
        yield_pct: f64,
        /// Signal conversion percentage.
        conversion_pct: f64,
        /// Selectivity percentage (desired vs total products).
        selectivity_pct: f64,
        /// Ratio of recycled unconverted signals.
        recycle_ratio: f64,
        /// Ratio of lost/discarded signals.
        loss_ratio: f64,
    },
    /// Recycle stream — unconverted signals returning as next cycle's feedstock.
    RecycleStream {
        /// Number of signals recycled.
        signal_count: u64,
        /// Node that originated the unconverted signals.
        source_node: String,
    },
    /// A skill was promoted to a higher tier.
    SkillPromoted {
        /// Skill identifier.
        skill: String,
        /// Previous tier.
        old_tier: String,
        /// New tier.
        new_tier: String,
    },
    /// Novelty detector flagged a previously unseen pattern.
    NoveltyDetected {
        /// Source subsystem that detected the novelty.
        source: String,
        /// Novelty score (0.0–1.0, higher = more novel).
        novelty: f64,
        /// Human-readable summary.
        summary: String,
    },
    /// Relay chain fidelity dropped below acceptable threshold.
    RelayDegradation {
        /// Chain identifier.
        chain: String,
        /// Total composed fidelity (F_total = Π f_i).
        f_total: f64,
        /// Minimum single-relay fidelity in the chain.
        f_min: f64,
    },
    /// Untyped event for extensibility.
    Custom {
        /// Event label.
        label: String,
        /// Arbitrary JSON payload.
        data: serde_json::Value,
    },
}

/// A single event on the flywheel bus, carrying a typed payload between nodes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlywheelEvent {
    /// Node tier that emitted this event.
    pub source_node: FlywheelTier,
    /// Target tier, or `None` for broadcast to all nodes.
    pub target_node: Option<FlywheelTier>,
    /// Typed event payload.
    pub kind: EventKind,
    /// When the event was created.
    pub timestamp: DateTime,
    /// Optional untyped metadata attached to the event.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub payload: Option<serde_json::Value>,
}

impl FlywheelEvent {
    /// Create a new event from source to optional target.
    pub fn new(source: FlywheelTier, target: Option<FlywheelTier>, kind: EventKind) -> Self {
        Self {
            source_node: source,
            target_node: target,
            kind,
            timestamp: DateTime::now(),
            payload: None,
        }
    }

    /// Create a broadcast event (targets all nodes).
    pub fn broadcast(source: FlywheelTier, kind: EventKind) -> Self {
        Self::new(source, None, kind)
    }

    /// Create an event directed at a specific node tier.
    pub fn targeted(source: FlywheelTier, target: FlywheelTier, kind: EventKind) -> Self {
        Self::new(source, Some(target), kind)
    }

    /// Returns the event with an attached JSON payload.
    #[must_use]
    pub fn with_payload(mut self, payload: serde_json::Value) -> Self {
        self.payload = Some(payload);
        self
    }

    /// Returns `true` if this event targets the given tier (or is broadcast).
    pub fn targets(&self, tier: FlywheelTier) -> bool {
        match self.target_node {
            Some(t) => t == tier,
            None => true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::node::FlywheelTier;

    #[test]
    fn new_event_sets_source_and_kind() {
        let e = FlywheelEvent::new(
            FlywheelTier::Live,
            Some(FlywheelTier::Staging),
            EventKind::CycleComplete { iteration: 42 },
        );
        assert_eq!(e.source_node, FlywheelTier::Live);
        assert_eq!(e.target_node, Some(FlywheelTier::Staging));
        assert!(e.payload.is_none());
    }

    #[test]
    fn broadcast_has_no_target() {
        let e = FlywheelEvent::broadcast(
            FlywheelTier::Staging,
            EventKind::InsightAccumulated { pattern_count: 5 },
        );
        assert!(e.target_node.is_none());
    }

    #[test]
    fn targeted_sets_target() {
        let e = FlywheelEvent::targeted(
            FlywheelTier::Live,
            FlywheelTier::Draft,
            EventKind::CycleComplete { iteration: 1 },
        );
        assert_eq!(e.target_node, Some(FlywheelTier::Draft));
    }

    #[test]
    fn with_payload_attaches_json() {
        let e = FlywheelEvent::broadcast(
            FlywheelTier::Live,
            EventKind::CycleComplete { iteration: 1 },
        )
        .with_payload(serde_json::json!({"key": "value"}));
        assert!(e.payload.is_some());
    }

    #[test]
    fn targets_matching_tier() {
        let e = FlywheelEvent::targeted(
            FlywheelTier::Live,
            FlywheelTier::Staging,
            EventKind::CycleComplete { iteration: 1 },
        );
        assert!(e.targets(FlywheelTier::Staging));
        assert!(!e.targets(FlywheelTier::Draft));
    }

    #[test]
    fn broadcast_targets_all() {
        let e = FlywheelEvent::broadcast(
            FlywheelTier::Live,
            EventKind::CycleComplete { iteration: 1 },
        );
        assert!(e.targets(FlywheelTier::Live));
        assert!(e.targets(FlywheelTier::Staging));
        assert!(e.targets(FlywheelTier::Draft));
    }

    #[test]
    fn cycle_complete_roundtrip() {
        let kind = EventKind::CycleComplete { iteration: 99 };
        let json = serde_json::to_string(&kind).expect("ser");
        let back: EventKind = serde_json::from_str(&json).expect("de");
        matches!(back, EventKind::CycleComplete { iteration: 99 });
    }

    #[test]
    fn baseline_shift_roundtrip() {
        let kind = EventKind::BaselineShift {
            metric: "prr".into(),
            old: 5.0,
            new: 7.0,
        };
        let json = serde_json::to_string(&kind).expect("ser");
        assert!(json.contains("baseline_shift"));
    }

    #[test]
    fn threshold_drift_roundtrip() {
        let kind = EventKind::ThresholdDrift {
            parameter: "alpha".into(),
            delta: 0.05,
        };
        let json = serde_json::to_string(&kind).expect("ser");
        let back: EventKind = serde_json::from_str(&json).expect("de");
        if let EventKind::ThresholdDrift { parameter, .. } = back {
            assert_eq!(parameter, "alpha");
        } else {
            panic!("wrong");
        }
    }

    #[test]
    fn trust_update_serializes() {
        let kind = EventKind::TrustUpdate {
            score: 0.95,
            level: "high".into(),
        };
        let json = serde_json::to_string(&kind).expect("ser");
        assert!(json.contains("trust_update"));
    }

    #[test]
    fn fractionation_complete_serializes() {
        let kind = EventKind::FractionationComplete {
            health_count: 10,
            threat_count: 2,
            learning_count: 5,
            noise_stripped: 100,
        };
        let json = serde_json::to_string(&kind).expect("ser");
        assert!(json.contains("fractionation_complete"));
    }

    #[test]
    fn yield_report_serializes() {
        let kind = EventKind::YieldReport {
            yield_pct: 85.0,
            conversion_pct: 70.0,
            selectivity_pct: 90.0,
            recycle_ratio: 0.1,
            loss_ratio: 0.05,
        };
        let json = serde_json::to_string(&kind).expect("ser");
        assert!(json.contains("yield_report"));
    }

    #[test]
    fn custom_event_roundtrip() {
        let kind = EventKind::Custom {
            label: "test".into(),
            data: serde_json::json!({"n": [1,2,3]}),
        };
        let json = serde_json::to_string(&kind).expect("ser");
        let back: EventKind = serde_json::from_str(&json).expect("de");
        if let EventKind::Custom { label, data } = back {
            assert_eq!(label, "test");
            assert_eq!(data["n"][1], 2);
        } else {
            panic!("wrong");
        }
    }

    #[test]
    fn full_event_roundtrip() {
        let e = FlywheelEvent::targeted(
            FlywheelTier::Staging,
            FlywheelTier::Live,
            EventKind::AdaptationReady {
                category: "threshold".into(),
            },
        );
        let json = serde_json::to_string(&e).expect("ser");
        let back: FlywheelEvent = serde_json::from_str(&json).expect("de");
        assert_eq!(back.source_node, FlywheelTier::Staging);
        assert_eq!(back.target_node, Some(FlywheelTier::Live));
    }
}
