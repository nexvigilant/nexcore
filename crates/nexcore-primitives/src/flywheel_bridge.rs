//! Flywheel bridge — emits fidelity and relay events into the flywheel bus.
//!
//! ## T1 Grounding
//!
//! | Function | Primitives |
//! |---|---|
//! | `emit_fidelity_drift` | ∂ (boundary: fidelity threshold) + N (quantity: delta) |
//! | `emit_relay_degradation` | → (causality: hop chain) + κ (comparison: F_total vs F_min) |

use nexcore_flywheel::{EventKind, FlywheelBus, FlywheelEvent, node::FlywheelTier};

/// Emit a fidelity drift event when a relay chain's total fidelity shifts.
///
/// Called when a `RelayChain` recalculates and the total fidelity product
/// changes by more than the configured drift threshold. Positive delta
/// means improvement; negative means degradation.
///
/// # Arguments
///
/// * `bus`       — shared flywheel bus
/// * `chain`     — relay chain identifier
/// * `f_total`   — current total fidelity (product of hop fidelities)
/// * `delta`     — change since last measurement
pub fn emit_fidelity_drift(
    bus: &FlywheelBus,
    chain: &str,
    f_total: f64,
    delta: f64,
) -> FlywheelEvent {
    let kind = EventKind::ThresholdDrift {
        parameter: format!("fidelity:{chain}"),
        delta,
    };
    let event = FlywheelEvent::broadcast(FlywheelTier::Staging, kind)
        .with_payload(serde_json::json!({ "f_total": f_total }));
    bus.emit(event)
}

/// Emit a relay degradation event when F_total drops below F_min.
///
/// Called when a relay verification detects that the total fidelity of a
/// pipeline has fallen below the safety-critical threshold (default 0.80).
///
/// # Arguments
///
/// * `bus`     — shared flywheel bus
/// * `chain`   — relay chain identifier
/// * `f_total` — current total fidelity
/// * `f_min`   — minimum acceptable fidelity threshold
pub fn emit_relay_degradation(
    bus: &FlywheelBus,
    chain: &str,
    f_total: f64,
    f_min: f64,
) -> FlywheelEvent {
    let kind = EventKind::RelayDegradation {
        chain: chain.to_owned(),
        f_total,
        f_min,
    };
    bus.emit(FlywheelEvent::broadcast(FlywheelTier::Staging, kind))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_emit_fidelity_drift() {
        let bus = FlywheelBus::new();
        let event = emit_fidelity_drift(&bus, "signal-pipeline", 0.88, -0.03);

        match &event.kind {
            EventKind::ThresholdDrift { parameter, delta } => {
                assert_eq!(parameter, "fidelity:signal-pipeline");
                assert!((*delta - (-0.03)).abs() < f64::EPSILON);
            }
            other => panic!("unexpected event kind: {other:?}"),
        }

        assert!(event.payload.is_some(), "payload must carry f_total");
        let f_total = event.payload.as_ref().and_then(|p| p["f_total"].as_f64());
        assert!((f_total.unwrap_or(0.0) - 0.88).abs() < f64::EPSILON);

        let consumed = bus.consume(FlywheelTier::Staging);
        assert_eq!(consumed.len(), 1);
    }

    #[test]
    fn test_emit_relay_degradation() {
        let bus = FlywheelBus::new();
        let event = emit_relay_degradation(&bus, "pv-pipeline", 0.72, 0.80);

        match &event.kind {
            EventKind::RelayDegradation {
                chain,
                f_total,
                f_min,
            } => {
                assert_eq!(chain, "pv-pipeline");
                assert!((*f_total - 0.72).abs() < f64::EPSILON);
                assert!((*f_min - 0.80).abs() < f64::EPSILON);
            }
            other => panic!("unexpected event kind: {other:?}"),
        }

        let consumed = bus.consume(FlywheelTier::Staging);
        assert_eq!(consumed.len(), 1, "expected 1 event in Staging tier");
    }

    #[test]
    fn test_fidelity_event_targets_staging() {
        let bus = FlywheelBus::new();
        emit_fidelity_drift(&bus, "test-chain", 0.95, 0.01);

        let events = bus.consume(FlywheelTier::Staging);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].source_node, FlywheelTier::Staging);
        assert!(events[0].target_node.is_none());
    }
}
