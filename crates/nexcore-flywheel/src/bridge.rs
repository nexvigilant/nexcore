//! Flywheel bridge: emit, consume, snapshot.

use crate::event::FlywheelEvent;
use crate::node::FlywheelTier;
use nexcore_chrono::DateTime;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

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
    pub fn new() -> Self {
        Self {
            buffer: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn emit(&self, event: FlywheelEvent) -> FlywheelEvent {
        if let Ok(mut buf) = self.buffer.lock() {
            buf.push(event.clone());
        }
        event
    }

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

    pub fn pending_count(&self) -> usize {
        self.buffer.lock().map(|buf| buf.len()).unwrap_or(0)
    }

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlywheelSnapshot {
    pub timestamp: DateTime,
    pub pending_events: usize,
    pub events: Vec<FlywheelEvent>,
}
