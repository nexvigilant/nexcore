//! # Boundary Gate — ∂ (Boundary) Layer
//!
//! Evaluates events against boundary specifications. When a threshold is
//! crossed, generates a BoundaryViolation for the consequence pipeline.
//!
//! ## Tier: T2-C (∂ + κ + ν)

use crate::vigilance::event::{EventKind, EventSeverity, WatchEvent, now_millis};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

/// Specification of a boundary condition.
///
/// Tier: T2-P (∂)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundarySpec {
    /// Human-readable name
    pub name: String,
    /// Only check events from this source (None = all sources)
    pub source_filter: Option<String>,
    /// Only check events of this kind (None = all kinds)
    pub kind_filter: Option<EventKind>,
    /// The threshold check to apply
    pub threshold: ThresholdCheck,
    /// Minimum time between violations (prevents flood)
    pub cooldown: Duration,
}

/// Threshold conditions that can trigger a boundary violation.
///
/// Tier: T2-P (κ)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ThresholdCheck {
    /// Fires when count of matching events exceeds `count` within `window`
    CountExceeds { count: u64, window: Duration },
    /// Fires when event severity is at least this level
    SeverityAtLeast(EventSeverity),
    /// Fires when payload matches a pattern at the given JSON path
    PayloadMatch { json_path: String, pattern: String },
    /// Always fires (every matching event is a violation)
    Always,
}

/// Record of a boundary being crossed.
///
/// Tier: T2-C (∂ + κ + ν)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundaryViolation {
    /// Which boundary was violated
    pub boundary: String,
    /// The triggering event
    pub event: WatchEvent,
    /// How many times this boundary has been violated
    pub violation_count: u64,
    /// First time this boundary was violated (millis)
    pub first_seen: u64,
    /// Most recent violation (millis)
    pub last_seen: u64,
}

/// Internal tracking state for a single boundary.
#[derive(Debug)]
struct BoundaryState {
    /// Timestamps of recent matching events (for sliding window)
    window: Vec<u64>,
    /// Last time a violation was emitted
    last_violation: u64,
    /// Total violations for this boundary
    violation_count: u64,
    /// First violation timestamp
    first_violation: Option<u64>,
}

impl BoundaryState {
    fn new() -> Self {
        Self {
            window: Vec::new(),
            last_violation: 0,
            violation_count: 0,
            first_violation: None,
        }
    }
}

/// The Boundary Gate — evaluates events against specs.
///
/// Tier: T2-C (∂ + κ + ν), dominant ∂
pub struct BoundaryGate {
    specs: Vec<BoundarySpec>,
    state: HashMap<String, BoundaryState>,
}

impl BoundaryGate {
    /// Create a new empty boundary gate.
    pub fn new() -> Self {
        Self {
            specs: Vec::new(),
            state: HashMap::new(),
        }
    }

    /// Add a boundary specification.
    pub fn add_spec(&mut self, spec: BoundarySpec) {
        let name = spec.name.clone();
        self.specs.push(spec);
        self.state.entry(name).or_insert_with(BoundaryState::new);
    }

    /// Number of boundary specs.
    pub fn spec_count(&self) -> usize {
        self.specs.len()
    }

    /// Get all boundary spec names.
    pub fn spec_names(&self) -> Vec<&str> {
        self.specs.iter().map(|s| s.name.as_str()).collect()
    }

    /// Evaluate an event against all boundaries.
    ///
    /// Returns violations for each boundary that was crossed.
    pub fn evaluate(&mut self, event: &WatchEvent) -> Vec<BoundaryViolation> {
        let now = now_millis();
        let mut violations = Vec::new();

        for spec in &self.specs {
            // Check source filter
            if let Some(ref filter) = spec.source_filter {
                if event.source != *filter {
                    continue;
                }
            }

            // Check kind filter
            if let Some(ref filter) = spec.kind_filter {
                if event.kind != *filter {
                    continue;
                }
            }

            let state = self
                .state
                .entry(spec.name.clone())
                .or_insert_with(BoundaryState::new);

            // Check cooldown
            if now.saturating_sub(state.last_violation) < spec.cooldown.as_millis() as u64 {
                continue;
            }

            // Evaluate threshold
            let violated = match &spec.threshold {
                ThresholdCheck::CountExceeds { count, window } => {
                    let window_ms = window.as_millis() as u64;
                    state.window.push(now);
                    // Prune events outside window
                    state
                        .window
                        .retain(|ts| now.saturating_sub(*ts) <= window_ms);
                    state.window.len() as u64 > *count
                }
                ThresholdCheck::SeverityAtLeast(min_severity) => event.severity >= *min_severity,
                ThresholdCheck::PayloadMatch { json_path, pattern } => {
                    // Simple top-level JSON path lookup
                    event
                        .payload
                        .get(json_path)
                        .and_then(|v| v.as_str())
                        .map(|s| s.contains(pattern.as_str()))
                        .unwrap_or(false)
                }
                ThresholdCheck::Always => true,
            };

            if violated {
                state.violation_count += 1;
                state.last_violation = now;
                if state.first_violation.is_none() {
                    state.first_violation = Some(now);
                }

                violations.push(BoundaryViolation {
                    boundary: spec.name.clone(),
                    event: event.clone(),
                    violation_count: state.violation_count,
                    first_seen: state.first_violation.unwrap_or(now),
                    last_seen: now,
                });
            }
        }

        violations
    }
}

impl Default for BoundaryGate {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vigilance::event::EventId;

    fn make_event(severity: EventSeverity, source: &str) -> WatchEvent {
        WatchEvent {
            id: EventId(1),
            source: source.to_string(),
            kind: EventKind::Signal,
            severity,
            payload: serde_json::json!({}),
            timestamp: now_millis(),
        }
    }

    #[test]
    fn severity_threshold_fires() {
        let mut gate = BoundaryGate::new();
        gate.add_spec(BoundarySpec {
            name: "high-alert".to_string(),
            source_filter: None,
            kind_filter: None,
            threshold: ThresholdCheck::SeverityAtLeast(EventSeverity::High),
            cooldown: Duration::from_millis(0),
        });

        let event = make_event(EventSeverity::Critical, "test");
        let violations = gate.evaluate(&event);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].boundary, "high-alert");
    }

    #[test]
    fn severity_threshold_does_not_fire_below() {
        let mut gate = BoundaryGate::new();
        gate.add_spec(BoundarySpec {
            name: "high-alert".to_string(),
            source_filter: None,
            kind_filter: None,
            threshold: ThresholdCheck::SeverityAtLeast(EventSeverity::High),
            cooldown: Duration::from_millis(0),
        });

        let event = make_event(EventSeverity::Low, "test");
        let violations = gate.evaluate(&event);
        assert!(violations.is_empty());
    }

    #[test]
    fn always_threshold_fires() {
        let mut gate = BoundaryGate::new();
        gate.add_spec(BoundarySpec {
            name: "catch-all".to_string(),
            source_filter: None,
            kind_filter: None,
            threshold: ThresholdCheck::Always,
            cooldown: Duration::from_millis(0),
        });

        let event = make_event(EventSeverity::Info, "test");
        let violations = gate.evaluate(&event);
        assert_eq!(violations.len(), 1);
    }

    #[test]
    fn source_filter_works() {
        let mut gate = BoundaryGate::new();
        gate.add_spec(BoundarySpec {
            name: "guardian-only".to_string(),
            source_filter: Some("guardian".to_string()),
            kind_filter: None,
            threshold: ThresholdCheck::Always,
            cooldown: Duration::from_millis(0),
        });

        // Wrong source — should not fire
        let event = make_event(EventSeverity::Info, "timer");
        assert!(gate.evaluate(&event).is_empty());

        // Right source — should fire
        let event = make_event(EventSeverity::Info, "guardian");
        assert_eq!(gate.evaluate(&event).len(), 1);
    }

    #[test]
    fn payload_match_works() {
        let mut gate = BoundaryGate::new();
        gate.add_spec(BoundarySpec {
            name: "error-match".to_string(),
            source_filter: None,
            kind_filter: None,
            threshold: ThresholdCheck::PayloadMatch {
                json_path: "status".to_string(),
                pattern: "error".to_string(),
            },
            cooldown: Duration::from_millis(0),
        });

        let mut event = make_event(EventSeverity::Info, "test");
        event.payload = serde_json::json!({"status": "error: timeout"});
        assert_eq!(gate.evaluate(&event).len(), 1);

        event.payload = serde_json::json!({"status": "ok"});
        assert!(gate.evaluate(&event).is_empty());
    }

    #[test]
    fn violation_count_tracks() {
        let mut gate = BoundaryGate::new();
        gate.add_spec(BoundarySpec {
            name: "counter".to_string(),
            source_filter: None,
            kind_filter: None,
            threshold: ThresholdCheck::Always,
            cooldown: Duration::from_millis(0),
        });

        let event = make_event(EventSeverity::Info, "test");
        let _ = gate.evaluate(&event);
        let v = gate.evaluate(&event);
        assert_eq!(v[0].violation_count, 2);
    }

    #[test]
    fn spec_management() {
        let mut gate = BoundaryGate::new();
        assert_eq!(gate.spec_count(), 0);

        gate.add_spec(BoundarySpec {
            name: "a".to_string(),
            source_filter: None,
            kind_filter: None,
            threshold: ThresholdCheck::Always,
            cooldown: Duration::from_millis(0),
        });
        gate.add_spec(BoundarySpec {
            name: "b".to_string(),
            source_filter: None,
            kind_filter: None,
            threshold: ThresholdCheck::Always,
            cooldown: Duration::from_millis(0),
        });

        assert_eq!(gate.spec_count(), 2);
        let names = gate.spec_names();
        assert!(names.contains(&"a"));
        assert!(names.contains(&"b"));
    }
}
