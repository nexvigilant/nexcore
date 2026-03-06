//! Flywheel event envelope and event kinds.

use crate::node::FlywheelTier;
use nexcore_chrono::DateTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum EventKind {
    CycleComplete {
        iteration: u64,
    },
    BaselineShift {
        metric: String,
        old: f64,
        new: f64,
    },
    ThresholdDrift {
        parameter: String,
        delta: f64,
    },
    AdaptationReady {
        category: String,
    },
    TrustUpdate {
        score: f64,
        level: String,
    },
    MaturationSignal {
        skill: String,
        transfer_score: f64,
    },
    InsightAccumulated {
        pattern_count: u64,
    },
    /// Fractionation complete — crude separated into typed streams.
    FractionationComplete {
        health_count: u64,
        threat_count: u64,
        learning_count: u64,
        noise_stripped: u64,
    },
    /// Yield report — refinery cycle metrics.
    YieldReport {
        yield_pct: f64,
        conversion_pct: f64,
        selectivity_pct: f64,
        recycle_ratio: f64,
        loss_ratio: f64,
    },
    /// Recycle stream — unconverted signals returning as next cycle's feedstock.
    RecycleStream {
        signal_count: u64,
        source_node: String,
    },
    SkillPromoted {
        skill: String,
        old_tier: String,
        new_tier: String,
    },
    NoveltyDetected {
        source: String,
        novelty: f64,
        summary: String,
    },
    RelayDegradation {
        chain: String,
        f_total: f64,
        f_min: f64,
    },
    Custom {
        label: String,
        data: serde_json::Value,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlywheelEvent {
    pub source_node: FlywheelTier,
    pub target_node: Option<FlywheelTier>,
    pub kind: EventKind,
    pub timestamp: DateTime,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub payload: Option<serde_json::Value>,
}

impl FlywheelEvent {
    pub fn new(source: FlywheelTier, target: Option<FlywheelTier>, kind: EventKind) -> Self {
        Self {
            source_node: source,
            target_node: target,
            kind,
            timestamp: DateTime::now(),
            payload: None,
        }
    }

    pub fn broadcast(source: FlywheelTier, kind: EventKind) -> Self {
        Self::new(source, None, kind)
    }

    pub fn targeted(source: FlywheelTier, target: FlywheelTier, kind: EventKind) -> Self {
        Self::new(source, Some(target), kind)
    }

    #[must_use]
    pub fn with_payload(mut self, payload: serde_json::Value) -> Self {
        self.payload = Some(payload);
        self
    }

    pub fn targets(&self, tier: FlywheelTier) -> bool {
        match self.target_node {
            Some(t) => t == tier,
            None => true,
        }
    }
}
