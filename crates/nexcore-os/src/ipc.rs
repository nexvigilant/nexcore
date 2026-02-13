// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! Synchronous IPC bus using Cytokine typed events.
//!
//! ## Primitive Grounding
//!
//! - → Causality: Event emission causes handler activation
//! - μ Mapping: Typed signal matching (family → handlers)
//! - σ Sequence: FIFO event queue ordering
//! - N Quantity: Event queue depth tracking

use nexcore_cytokine::{Cytokine, CytokineFamily, Scope, ThreatLevel};
use std::collections::VecDeque;

/// Synchronous event bus for inter-service communication.
///
/// Tier: T3 (→ + μ + σ + N)
///
/// Uses `Cytokine` types from the biological signaling crate but operates
/// synchronously (no async runtime required). Events are queued and drained
/// during the OS tick loop.
pub struct EventBus {
    /// FIFO event queue.
    queue: VecDeque<Cytokine>,
    /// Total events emitted since boot.
    total_emitted: u64,
    /// Maximum queue depth (back-pressure).
    max_depth: usize,
    /// Source identifier for this bus.
    source: String,
}

impl EventBus {
    /// Create a new event bus.
    pub fn new(source: impl Into<String>) -> Self {
        Self {
            queue: VecDeque::new(),
            total_emitted: 0,
            max_depth: 1024,
            source: source.into(),
        }
    }

    /// Emit a cytokine event to the bus.
    ///
    /// Returns false if the queue is at capacity (back-pressure).
    pub fn emit(&mut self, signal: Cytokine) -> bool {
        if self.queue.len() >= self.max_depth {
            return false;
        }
        self.queue.push_back(signal);
        self.total_emitted += 1;
        true
    }

    /// Emit a service state change event.
    pub fn emit_service_event(&mut self, service_name: &str, from: &str, to: &str) {
        let signal = Cytokine::new(CytokineFamily::Il6, "service_state_change")
            .with_source(&self.source)
            .with_target(service_name)
            .with_scope(Scope::Systemic)
            .with_severity(ThreatLevel::Low)
            .with_payload(serde_json::json!({
                "service": service_name,
                "from": from,
                "to": to,
            }));
        self.emit(signal);
    }

    /// Emit a boot phase event.
    pub fn emit_boot_event(&mut self, phase: &str) {
        let signal = Cytokine::new(CytokineFamily::Csf, "boot_phase")
            .with_source(&self.source)
            .with_scope(Scope::Systemic)
            .with_severity(ThreatLevel::Low)
            .with_payload(serde_json::json!({ "phase": phase }));
        self.emit(signal);
    }

    /// Emit a shutdown event.
    pub fn emit_shutdown_event(&mut self) {
        let signal = Cytokine::new(CytokineFamily::TnfAlpha, "shutdown")
            .with_source(&self.source)
            .with_scope(Scope::Systemic)
            .with_severity(ThreatLevel::High);
        self.emit(signal);
    }

    /// Drain all pending events.
    pub fn drain(&mut self) -> Vec<Cytokine> {
        self.queue.drain(..).collect()
    }

    /// Peek at the next event without removing it.
    pub fn peek(&self) -> Option<&Cytokine> {
        self.queue.front()
    }

    /// Number of pending events.
    pub fn pending(&self) -> usize {
        self.queue.len()
    }

    /// Total events emitted since boot.
    pub fn total_emitted(&self) -> u64 {
        self.total_emitted
    }

    /// Whether the queue is empty.
    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new("nexcore-os")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn event_bus_creation() {
        let bus = EventBus::new("test");
        assert!(bus.is_empty());
        assert_eq!(bus.total_emitted(), 0);
    }

    #[test]
    fn emit_and_drain() {
        let mut bus = EventBus::new("test");
        let signal =
            Cytokine::new(CytokineFamily::Il1, "test_event").with_severity(ThreatLevel::Low);

        assert!(bus.emit(signal));
        assert_eq!(bus.pending(), 1);
        assert_eq!(bus.total_emitted(), 1);

        let events = bus.drain();
        assert_eq!(events.len(), 1);
        assert!(bus.is_empty());
        assert_eq!(bus.total_emitted(), 1); // Counter persists
    }

    #[test]
    fn service_state_event() {
        let mut bus = EventBus::new("test-os");
        bus.emit_service_event("guardian", "Registered", "Starting");

        assert_eq!(bus.pending(), 1);
        let events = bus.drain();
        assert_eq!(events[0].name, "service_state_change");
    }

    #[test]
    fn boot_event() {
        let mut bus = EventBus::new("test-os");
        bus.emit_boot_event("PalInit");

        assert_eq!(bus.pending(), 1);
        let events = bus.drain();
        assert_eq!(events[0].name, "boot_phase");
    }

    #[test]
    fn back_pressure() {
        let mut bus = EventBus::new("test");
        // Fill to capacity
        for i in 0..1024 {
            let signal = Cytokine::new(CytokineFamily::Il1, format!("event_{i}"));
            assert!(bus.emit(signal));
        }

        // Should reject at capacity
        let overflow = Cytokine::new(CytokineFamily::Il1, "overflow");
        assert!(!bus.emit(overflow));
        assert_eq!(bus.pending(), 1024);
    }

    #[test]
    fn peek_without_consuming() {
        let mut bus = EventBus::new("test");
        let signal = Cytokine::new(CytokineFamily::Il6, "peek_test");
        bus.emit(signal);

        assert!(bus.peek().is_some());
        assert_eq!(bus.pending(), 1); // Still there
    }

    #[test]
    fn shutdown_event() {
        let mut bus = EventBus::new("test-os");
        bus.emit_shutdown_event();

        let events = bus.drain();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].name, "shutdown");
    }
}
