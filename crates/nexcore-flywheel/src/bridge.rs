//! Flywheel bridge: emit, consume, snapshot.

use crate::event::FlywheelEvent;
use crate::node::FlywheelTier;
use nexcore_chrono::DateTime;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

/// Thread-safe event bus for flywheel inter-tier communication.
#[derive(Debug, Clone)]
pub struct FlywheelBus {
    buffer: Arc<Mutex<Vec<FlywheelEvent>>>,
}

impl Default for FlywheelBus {
    fn default() -> Self {
        Self::new()
    }
}

impl FlywheelBus {
    /// Creates a new empty event bus.
    pub fn new() -> Self {
        Self {
            buffer: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Publishes an event to the bus and returns it.
    pub fn emit(&self, event: FlywheelEvent) -> FlywheelEvent {
        if let Ok(mut buf) = self.buffer.lock() {
            buf.push(event.clone());
        }
        event
    }

    /// Drains and returns all pending events that target the given tier.
    pub fn consume(&self, tier: FlywheelTier) -> Vec<FlywheelEvent> {
        let Ok(mut buf) = self.buffer.lock() else {
            return Vec::new();
        };
        let mut consumed = Vec::new();
        let mut remaining = Vec::new();
        for event in buf.drain(..) {
            if event.targets(tier) {
                consumed.push(event);
            } else {
                remaining.push(event);
            }
        }
        *buf = remaining;
        consumed
    }

    /// Returns the number of events currently pending in the bus.
    pub fn pending_count(&self) -> usize {
        self.buffer.lock().map(|buf| buf.len()).unwrap_or(0)
    }

    /// Creates a point-in-time snapshot of the bus state.
    pub fn snapshot(&self) -> FlywheelSnapshot {
        let events = self
            .buffer
            .lock()
            .map(|buf| buf.clone())
            .unwrap_or_default();
        FlywheelSnapshot {
            timestamp: DateTime::now(),
            pending_events: events.len(),
            events,
        }
    }

    /// Removes all pending events from the bus.
    pub fn clear(&self) {
        if let Ok(mut buf) = self.buffer.lock() {
            buf.clear();
        }
    }

    /// Emit a fidelity drift event when a relay chain's total fidelity shifts.
    ///
    /// Called when a `RelayChain` recalculates and the total fidelity product
    /// changes by more than the configured drift threshold.
    pub fn emit_fidelity_drift(&self, chain: &str, f_total: f64, delta: f64) -> FlywheelEvent {
        use crate::event::EventKind;
        let kind = EventKind::ThresholdDrift {
            parameter: format!("fidelity:{chain}"),
            delta,
        };
        let event = FlywheelEvent::broadcast(FlywheelTier::Staging, kind)
            .with_payload(serde_json::json!({ "f_total": f_total }));
        self.emit(event)
    }

    /// Emit a relay degradation event when F_total drops below F_min.
    ///
    /// Called when relay verification detects total fidelity has fallen
    /// below the safety-critical threshold (default 0.80).
    pub fn emit_relay_degradation(&self, chain: &str, f_total: f64, f_min: f64) -> FlywheelEvent {
        use crate::event::EventKind;
        let kind = EventKind::RelayDegradation {
            chain: chain.to_owned(),
            f_total,
            f_min,
        };
        self.emit(FlywheelEvent::broadcast(FlywheelTier::Staging, kind))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::EventKind;

    #[test]
    fn new_bus_is_empty() {
        let bus = FlywheelBus::new();
        assert_eq!(bus.pending_count(), 0);
    }
    #[test]
    fn default_bus_is_empty() {
        let bus = FlywheelBus::default();
        assert_eq!(bus.pending_count(), 0);
    }
    #[test]
    fn emit_increments_pending() {
        let bus = FlywheelBus::new();
        bus.emit(FlywheelEvent::broadcast(
            FlywheelTier::Live,
            EventKind::CycleComplete { iteration: 1 },
        ));
        assert_eq!(bus.pending_count(), 1);
    }
    #[test]
    fn consume_drains_matching() {
        let bus = FlywheelBus::new();
        bus.emit(FlywheelEvent::targeted(
            FlywheelTier::Live,
            FlywheelTier::Staging,
            EventKind::CycleComplete { iteration: 1 },
        ));
        bus.emit(FlywheelEvent::targeted(
            FlywheelTier::Live,
            FlywheelTier::Draft,
            EventKind::CycleComplete { iteration: 2 },
        ));
        let consumed = bus.consume(FlywheelTier::Staging);
        assert_eq!(consumed.len(), 1);
        assert_eq!(bus.pending_count(), 1);
    }
    #[test]
    fn consume_broadcast_delivers_to_all() {
        let bus = FlywheelBus::new();
        bus.emit(FlywheelEvent::broadcast(
            FlywheelTier::Live,
            EventKind::CycleComplete { iteration: 1 },
        ));
        let consumed = bus.consume(FlywheelTier::Draft);
        assert_eq!(consumed.len(), 1);
    }
    #[test]
    fn clear_empties_bus() {
        let bus = FlywheelBus::new();
        bus.emit(FlywheelEvent::broadcast(
            FlywheelTier::Live,
            EventKind::CycleComplete { iteration: 1 },
        ));
        bus.emit(FlywheelEvent::broadcast(
            FlywheelTier::Live,
            EventKind::CycleComplete { iteration: 2 },
        ));
        bus.clear();
        assert_eq!(bus.pending_count(), 0);
    }
    #[test]
    fn snapshot_captures_state() {
        let bus = FlywheelBus::new();
        bus.emit(FlywheelEvent::broadcast(
            FlywheelTier::Live,
            EventKind::CycleComplete { iteration: 1 },
        ));
        let snap = bus.snapshot();
        assert_eq!(snap.pending_events, 1);
        assert_eq!(snap.events.len(), 1);
    }
    #[test]
    fn emit_fidelity_drift_creates_threshold_drift() {
        let bus = FlywheelBus::new();
        let e = bus.emit_fidelity_drift("test-chain", 0.85, 0.02);
        if let EventKind::ThresholdDrift { parameter, delta } = &e.kind {
            assert!(parameter.contains("test-chain"));
            assert!((delta - 0.02).abs() < f64::EPSILON);
        } else {
            panic!("wrong kind");
        }
    }
    #[test]
    fn emit_relay_degradation_creates_event() {
        let bus = FlywheelBus::new();
        let e = bus.emit_relay_degradation("chain-x", 0.75, 0.6);
        if let EventKind::RelayDegradation { chain, f_total, .. } = &e.kind {
            assert_eq!(chain, "chain-x");
            assert!((f_total - 0.75).abs() < f64::EPSILON);
        } else {
            panic!("wrong kind");
        }
    }
    #[test]
    fn bus_is_clone_safe() {
        let bus = FlywheelBus::new();
        let bus2 = bus.clone();
        bus.emit(FlywheelEvent::broadcast(
            FlywheelTier::Live,
            EventKind::CycleComplete { iteration: 1 },
        ));
        assert_eq!(bus2.pending_count(), 1); // shared Arc
    }
    #[test]
    fn snapshot_serializes() {
        let bus = FlywheelBus::new();
        bus.emit(FlywheelEvent::broadcast(
            FlywheelTier::Live,
            EventKind::CycleComplete { iteration: 1 },
        ));
        let snap = bus.snapshot();
        let json = serde_json::to_string(&snap).expect("ser");
        let back: FlywheelSnapshot = serde_json::from_str(&json).expect("de");
        assert_eq!(back.pending_events, 1);
    }
}

/// A point-in-time capture of the event bus state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlywheelSnapshot {
    /// The time at which the snapshot was taken.
    pub timestamp: DateTime,
    /// The number of events pending at snapshot time.
    pub pending_events: usize,
    /// The events pending at snapshot time.
    pub events: Vec<FlywheelEvent>,
}
