//! Flywheel bridge — emits primitive tier-shift events into the flywheel bus.
//!
//! ## T1 Grounding
//!
//! | Function | Primitives |
//! |---|---|
//! | `emit_tier_shift` | κ (comparison: old vs new tier) + ∂ (boundary: tier threshold) |
//! | `emit_extraction_complete` | N (quantity: primitive count) + σ (sequence: scan cycle) |
//! | `consume_primitive_events` | μ (mapping: event→action) + → (causality: react to drift) |

use nexcore_flywheel::{EventKind, FlywheelBus, FlywheelEvent, node::FlywheelTier};

/// Emit a tier-shift event when a primitive's classification changes.
///
/// Uses `ThresholdDrift` with parameter `"tier:{name}"` and delta encoding
/// the tier distance (e.g., T3→T2 = +1.0, T2→T1 = +1.0, T1→T2 = -1.0).
///
/// # Arguments
///
/// * `bus`   — shared flywheel bus
/// * `name`  — primitive name (e.g. `"∂"`, `"κ"`)
/// * `delta` — tier shift direction (+1.0 = promotion, -1.0 = demotion)
pub fn emit_tier_shift(bus: &FlywheelBus, name: &str, delta: f64) -> FlywheelEvent {
    let kind = EventKind::ThresholdDrift {
        parameter: format!("tier:{name}"),
        delta,
    };
    bus.emit(FlywheelEvent::broadcast(FlywheelTier::Staging, kind))
}

/// Emit an extraction-complete event after a scan cycle finishes.
///
/// Reports the number of primitives extracted and their tier distribution
/// via the event payload.
///
/// # Arguments
///
/// * `bus`       — shared flywheel bus
/// * `t1_count`  — T1 (universal) primitives found
/// * `t2_count`  — T2 (cross-domain) primitives found
/// * `t3_count`  — T3 (domain-specific) primitives found
pub fn emit_extraction_complete(
    bus: &FlywheelBus,
    t1_count: u64,
    t2_count: u64,
    t3_count: u64,
) -> FlywheelEvent {
    let total = t1_count + t2_count + t3_count;
    let kind = EventKind::InsightAccumulated {
        pattern_count: total,
    };
    let event =
        FlywheelEvent::broadcast(FlywheelTier::Staging, kind).with_payload(serde_json::json!({
            "source": "primitive-scanner",
            "t1": t1_count,
            "t2": t2_count,
            "t3": t3_count,
        }));
    bus.emit(event)
}

/// Consume pending flywheel events relevant to the primitives node.
///
/// Drains `ThresholdDrift` events from the staging tier so the scanner
/// can react to drift signals from other nodes (e.g., fidelity drift
/// from relay chains that may invalidate tier classifications).
///
/// Returns the consumed events for the caller to process.
pub fn consume_primitive_events(bus: &FlywheelBus) -> Vec<FlywheelEvent> {
    let events = bus.consume(FlywheelTier::Staging);
    events
        .into_iter()
        .filter(|e| matches!(&e.kind, EventKind::ThresholdDrift { parameter, .. } if parameter.starts_with("tier:") || parameter.starts_with("fidelity:")))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_emit_tier_shift() {
        let bus = FlywheelBus::new();
        let event = emit_tier_shift(&bus, "∂", 1.0);

        match &event.kind {
            EventKind::ThresholdDrift { parameter, delta } => {
                assert_eq!(parameter, "tier:∂");
                assert!((delta - 1.0).abs() < f64::EPSILON);
            }
            other => panic!("unexpected event kind: {other:?}"),
        }

        let consumed = bus.consume(FlywheelTier::Staging);
        assert_eq!(consumed.len(), 1, "expected 1 event in Staging tier");
    }

    #[test]
    fn test_emit_extraction_complete() {
        let bus = FlywheelBus::new();
        let event = emit_extraction_complete(&bus, 9, 4, 3);

        match &event.kind {
            EventKind::InsightAccumulated { pattern_count } => {
                assert_eq!(*pattern_count, 16);
            }
            other => panic!("unexpected event kind: {other:?}"),
        }

        let payload = event.payload.as_ref().unwrap();
        assert_eq!(payload["source"], "primitive-scanner");
        assert_eq!(payload["t1"], 9);

        let consumed = bus.consume(FlywheelTier::Staging);
        assert_eq!(consumed.len(), 1);
    }

    #[test]
    fn test_consume_filters_relevant_events() {
        let bus = FlywheelBus::new();

        // Emit a tier shift (relevant)
        emit_tier_shift(&bus, "κ", -1.0);

        // Emit a fidelity drift (relevant)
        bus.emit_fidelity_drift("test-chain", 0.85, -0.05);

        // Emit a trust update (irrelevant to primitives)
        let trust = EventKind::TrustUpdate {
            score: 0.9,
            level: "high".to_owned(),
        };
        bus.emit(FlywheelEvent::broadcast(FlywheelTier::Staging, trust));

        let consumed = consume_primitive_events(&bus);
        assert_eq!(
            consumed.len(),
            2,
            "should consume tier + fidelity, not trust"
        );
    }

    #[test]
    fn test_tier_shift_targets_staging() {
        let bus = FlywheelBus::new();
        emit_tier_shift(&bus, "ς", 1.0);

        let events = bus.consume(FlywheelTier::Staging);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].source_node, FlywheelTier::Staging);
        assert!(
            events[0].target_node.is_none(),
            "broadcast event must have no specific target"
        );
    }
}
