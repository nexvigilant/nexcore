//! Flywheel bridge — emits trust state changes into the nexcore-flywheel bus.
//!
//! Trust score transitions are causally significant events (→) in the
//! NexCore ecosystem. This module maps trust engine output (μ) to
//! `FlywheelEvent` payloads and emits them from the Live tier (∂).

use nexcore_flywheel::{EventKind, FlywheelBus, FlywheelEvent, node::FlywheelTier};

/// Emit a `TrustUpdate` event from the `Live` tier into `bus`.
///
/// Constructs a broadcast `FlywheelEvent` carrying the trust `score` and
/// human-readable `level` string, pushes it onto the bus, and returns the
/// emitted event so callers can inspect or forward it.
///
/// # Arguments
///
/// * `bus`   — shared flywheel bus (Arc-backed, clone-safe)
/// * `score` — Bayesian trust score in `[0.0, 1.0]`
/// * `level` — human-readable label (e.g., `"high"`, `"low"`, `"guarded"`)
///
/// # Returns
///
/// The `FlywheelEvent` that was pushed onto the bus.
pub fn emit_trust_update(bus: &FlywheelBus, score: f64, level: &str) -> FlywheelEvent {
    let kind = EventKind::TrustUpdate {
        score,
        level: level.to_string(),
    };
    bus.emit(FlywheelEvent::broadcast(FlywheelTier::Live, kind))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Emit a trust update and verify the consumed event carries the
    /// correct score and level from the Live tier.
    #[test]
    fn test_emit_trust_update() {
        let bus = FlywheelBus::new();
        let event = emit_trust_update(&bus, 0.82, "high");

        // Event returned from emit must match what we put in.
        match &event.kind {
            EventKind::TrustUpdate { score, level } => {
                assert!(
                    (*score - 0.82).abs() < f64::EPSILON,
                    "score mismatch: expected 0.82, got {score}"
                );
                assert_eq!(level, "high");
            }
            other => panic!("unexpected event kind: {other:?}"),
        }

        // The bus must have buffered exactly one event; consuming from
        // the Live tier (broadcast) yields it.
        let consumed = bus.consume(FlywheelTier::Live);
        assert_eq!(consumed.len(), 1, "expected 1 event in Live tier");
        match &consumed[0].kind {
            EventKind::TrustUpdate { score, level } => {
                assert!((*score - 0.82).abs() < f64::EPSILON);
                assert_eq!(level, "high");
            }
            other => panic!("unexpected consumed event kind: {other:?}"),
        }
    }

    /// Verify that the emitted event originates from the Live tier.
    #[test]
    fn test_trust_event_targets_live() {
        let bus = FlywheelBus::new();
        emit_trust_update(&bus, 0.55, "guarded");

        // Broadcast events target all tiers; Live must receive the event.
        let live_events = bus.consume(FlywheelTier::Live);
        assert_eq!(
            live_events.len(),
            1,
            "Live tier must receive broadcast trust event"
        );
        assert_eq!(
            live_events[0].source_node,
            FlywheelTier::Live,
            "event source must be FlywheelTier::Live"
        );
        // target_node is None for broadcasts — the event is tier-agnostic.
        assert!(
            live_events[0].target_node.is_none(),
            "broadcast event must have no specific target node"
        );
    }
}
