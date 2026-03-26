// Copyright © 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! Flywheel bridge for nexcore-immunity.
//!
//! Provides emission of adaptation signals from the immunity system onto the
//! nexcore-flywheel bus. Grounds T1 primitives:
//!
//! | Concept | Primitive | Symbol |
//! |---------|-----------|--------|
//! | Event creation | Existence | ∃ |
//! | Immunity → Flywheel | Causality | → |
//! | Category → EventKind | Mapping | μ |
//! | Emit then return | Sequence | σ |
//! | Function boundary | Boundary | ∂ |

use nexcore_flywheel::{EventKind, FlywheelBus, FlywheelEvent, node::FlywheelTier};

/// Emit an `AdaptationReady` event from the immunity system onto the flywheel bus.
///
/// The event is sourced from `FlywheelTier::Live` and broadcast to all tiers,
/// signalling that adaptive immunity has completed a learning cycle for the
/// given `category`.
///
/// Returns the emitted [`FlywheelEvent`] so callers can inspect it or chain
/// further operations.
///
/// # Example
///
/// ```rust,no_run
/// use nexcore_flywheel::FlywheelBus;
/// use nexcore_immunity::flywheel_bridge::emit_adaptation_ready;
///
/// let bus = FlywheelBus::new();
/// let event = emit_adaptation_ready(&bus, "unwrap-pattern");
/// ```
pub fn emit_adaptation_ready(bus: &FlywheelBus, category: &str) -> FlywheelEvent {
    // μ: map category string to the AdaptationReady variant
    let kind = EventKind::AdaptationReady {
        category: category.to_string(),
    };
    // ∃ + σ: create the broadcast event sourced from Live tier, then emit
    let event = FlywheelEvent::broadcast(FlywheelTier::Live, kind);
    bus.emit(event)
}

/// Consume pending flywheel events relevant to the immunity node.
///
/// Drains `AdaptationReady` events (self-feedback from prior adaptation cycles)
/// and `Custom` PV signal events that may trigger new antibody creation.
/// Immunity also reacts to `ThresholdDrift` as drift in control parameters
/// signals potential new threat vectors.
pub fn consume_immunity_events(bus: &FlywheelBus) -> Vec<FlywheelEvent> {
    let events = bus.consume(FlywheelTier::Live);
    events
        .into_iter()
        .filter(|e| match &e.kind {
            EventKind::AdaptationReady { .. } | EventKind::ThresholdDrift { .. } => true,
            EventKind::Custom { label, .. } => label.starts_with("pv_"),
            _ => false,
        })
        .collect()
}

// ═══════════════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use nexcore_flywheel::{EventKind, FlywheelBus, node::FlywheelTier};

    #[test]
    fn test_consume_immunity_events_filters() {
        let bus = FlywheelBus::new();

        // Relevant: adaptation feedback
        emit_adaptation_ready(&bus, "unwrap-pattern");
        // Relevant: PV signal (new threat vector)
        bus.emit(FlywheelEvent::broadcast(
            FlywheelTier::Live,
            EventKind::Custom {
                label: "pv_signal_detected".to_owned(),
                data: serde_json::json!({"drug": "test"}),
            },
        ));
        // Irrelevant: trust update
        bus.emit(FlywheelEvent::broadcast(
            FlywheelTier::Live,
            EventKind::TrustUpdate {
                score: 0.7,
                level: "medium".to_owned(),
            },
        ));

        let consumed = consume_immunity_events(&bus);
        assert_eq!(
            consumed.len(),
            2,
            "should consume adaptation + pv_signal, not trust"
        );
    }

    #[test]
    fn test_consume_includes_threshold_drift() {
        let bus = FlywheelBus::new();
        bus.emit(FlywheelEvent::broadcast(
            FlywheelTier::Live,
            EventKind::ThresholdDrift {
                parameter: "retry_ceiling".to_owned(),
                delta: -0.05,
            },
        ));

        let consumed = consume_immunity_events(&bus);
        assert_eq!(consumed.len(), 1, "drift signals feed immunity");
    }

    /// Emit an AdaptationReady event and consume it from the Live tier broadcast.
    /// Verifies the category round-trips correctly.
    #[test]
    fn test_emit_adaptation_ready() {
        let bus = FlywheelBus::new();
        let category = "unwrap-pattern";

        let event = emit_adaptation_ready(&bus, category);

        // Verify the returned event carries the right kind
        match &event.kind {
            EventKind::AdaptationReady { category: cat } => {
                assert_eq!(cat, category);
            }
            other => panic!("Expected AdaptationReady, got {:?}", other),
        }

        // Consume from Staging — broadcast events reach all tiers
        let consumed = bus.consume(FlywheelTier::Staging);
        assert_eq!(consumed.len(), 1, "Staging tier should receive one event");

        match &consumed[0].kind {
            EventKind::AdaptationReady { category: cat } => {
                assert_eq!(cat, category);
            }
            other => panic!(
                "Expected AdaptationReady in consumed event, got {:?}",
                other
            ),
        }
    }

    /// Emit and verify that a broadcast event is consumable and has no
    /// specific target (broadcast semantics: `target_node` is `None`,
    /// so `targets()` returns `true` for any tier).
    #[test]
    fn test_adaptation_event_broadcast() {
        let bus = FlywheelBus::new();
        let category = "expect-pattern";

        let event = emit_adaptation_ready(&bus, category);

        // Broadcast events have no target_node — they match any tier.
        assert!(
            event.target_node.is_none(),
            "broadcast event must have no specific target"
        );
        assert_eq!(event.source_node, FlywheelTier::Live);

        // Consuming from any tier drains the broadcast from the shared buffer.
        let consumed = bus.consume(FlywheelTier::Live);
        assert_eq!(consumed.len(), 1, "Live tier should consume the broadcast");

        match &consumed[0].kind {
            EventKind::AdaptationReady { category: cat } => {
                assert_eq!(cat, category);
            }
            other => panic!("Wrong event kind: {:?}", other),
        }

        // Buffer is now empty — subsequent consume returns nothing.
        assert_eq!(bus.pending_count(), 0, "buffer must be empty after consume");
    }
}
