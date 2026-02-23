// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! Synchronous IPC bus using Cytokine typed events and structured ServiceCalls.
//!
//! ## Primitive Grounding
//!
//! - → Causality: Event emission causes handler activation
//! - μ Mapping: Typed signal matching (family → handlers)
//! - σ Sequence: FIFO event queue ordering
//! - N Quantity: Event queue depth tracking
//! - ∂ Boundary: Capability tokens and caller identity

pub mod call;
pub mod identity;

pub use call::{ServiceCall, ServiceRequest, ServiceResponse};
pub use identity::{CallerIdentity, CapabilityToken};

use nexcore_cytokine::{Cytokine, CytokineFamily, Scope, ThreatLevel};
use std::collections::{HashMap, VecDeque};

use crate::journal::{JournalEntry, Keywords, OsJournal, Severity, Subsystem};

/// Synchronous event bus for inter-service communication.
///
/// Tier: T3 (→ + μ + σ + N + ∂)
///
/// Handles fire-and-forget `Cytokine` events as well as structured
/// request/response `ServiceCall`s with caller identity and capability tokens.
pub struct EventBus {
    /// FIFO event queue (legacy fire-and-forget).
    queue: VecDeque<Cytokine>,
    /// Pending service calls waiting for a handler.
    pending_calls: VecDeque<ServiceCall>,
    /// Completed responses mapped by call ID.
    responses: HashMap<String, ServiceResponse>,
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
            pending_calls: VecDeque::new(),
            responses: HashMap::new(),
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

    /// Invoke a service call synchronously (queues the call).
    ///
    /// In a fully async/multi-process system this would block or return a Future.
    /// In our synchronous OS loop, we queue it. For testing/sync processing, we
    /// can directly return a simulated response if we handle it inline, but for now
    /// it enters the pending queue.
    pub fn call(&mut self, call: ServiceCall, journal: &mut OsJournal, tick: u64) -> bool {
        if self.pending_calls.len() >= self.max_depth {
            return false;
        }

        // Journal Integration: Record the IPC call
        journal.record(
            JournalEntry::new(
                Subsystem::Ipc,
                "call",
                Severity::Debug,
                format!(
                    "IPC call to {}::{}",
                    call.target_service, call.request.method
                ),
            )
            .with_keywords(Keywords::IPC)
            .with_str("caller", format!("{}", call.caller))
            .with_str("target", &call.target_service)
            .with_str("method", &call.request.method)
            .with_str("call_id", &call.id),
            tick,
        );

        self.pending_calls.push_back(call);
        self.total_emitted += 1;
        true
    }

    /// Fulfill a pending call with a response.
    pub fn resolve_call(
        &mut self,
        call_id: &str,
        response: ServiceResponse,
        journal: &mut OsJournal,
        tick: u64,
    ) {
        let severity = if response.success {
            Severity::Debug
        } else {
            Severity::Warning
        };
        journal.record(
            JournalEntry::new(
                Subsystem::Ipc,
                "response",
                severity,
                format!("IPC response for call {}", call_id),
            )
            .with_keywords(Keywords::IPC)
            .with_str("call_id", call_id)
            .with_bool("success", response.success),
            tick,
        );

        self.responses.insert(call_id.to_string(), response);
    }

    /// Retrieve a response for a given call ID (if ready).
    pub fn get_response(&mut self, call_id: &str) -> Option<ServiceResponse> {
        self.responses.remove(call_id)
    }

    /// Drain pending service calls.
    pub fn drain_calls(&mut self) -> Vec<ServiceCall> {
        self.pending_calls.drain(..).collect()
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

    /// Number of pending events (legacy cytokines).
    pub fn pending(&self) -> usize {
        self.queue.len()
    }

    /// Number of pending service calls.
    pub fn pending_calls(&self) -> usize {
        self.pending_calls.len()
    }

    /// Total events (and calls) emitted since boot.
    pub fn total_emitted(&self) -> u64 {
        self.total_emitted
    }

    /// Whether the queue is empty.
    pub fn is_empty(&self) -> bool {
        self.queue.is_empty() && self.pending_calls.is_empty()
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

    #[test]
    fn service_call_routing() {
        let mut bus = EventBus::new("test");
        let mut journal = OsJournal::with_config(1024, 256, Severity::Debug);

        let req = ServiceRequest::new("ping", serde_json::json!({}));
        let call = ServiceCall::new(CallerIdentity::System, "network", req)
            .with_token(CapabilityToken::new("super-secret-token"));

        let call_id = call.id.clone();

        // Dispatch call
        assert!(bus.call(call, &mut journal, 1));
        assert_eq!(bus.pending_calls(), 1);

        // Drain calls
        let mut calls = bus.drain_calls();
        assert_eq!(calls.len(), 1);
        let received_call = calls.remove(0);
        assert_eq!(received_call.target_service, "network");

        // Resolve response
        let resp = ServiceResponse::success(serde_json::json!({ "pong": true }));
        bus.resolve_call(&call_id, resp, &mut journal, 2);

        // Retrieve response
        let retrieved = bus.get_response(&call_id).unwrap();
        assert!(retrieved.success);

        // Journal verification
        assert_eq!(journal.total_recorded(), 2);
    }
}
