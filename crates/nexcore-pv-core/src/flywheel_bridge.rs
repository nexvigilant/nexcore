// Copyright (c) 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! Flywheel bridge — PV signal detection emits domain events into the flywheel bus.
//!
//! ## T1 Grounding
//!
//! | Function | Primitives |
//! |---|---|
//! | `emit_signal_detected` | N (quantity: score) + κ (comparison: threshold) + → (causality) |
//! | `emit_causality_assessed` | → (causality) + ς (state: verdict) + σ (sequence: chain) |
//! | `emit_safety_margin_shift` | ∂ (boundary: margin) + μ (mapping: old → new) |

use nexcore_flywheel::{EventKind, FlywheelBus, FlywheelEvent, node::FlywheelTier};

/// Emit a signal detection event when a disproportionality score crosses threshold.
///
/// Called after PRR, ROR, IC, or EBGM computation produces a signal above the
/// configured threshold. The `algorithm` identifies which method fired
/// (e.g. `"PRR"`, `"EBGM"`), `drug` and `event` name the pair, and `score`
/// is the computed disproportionality value.
pub fn emit_signal_detected(
    bus: &FlywheelBus,
    algorithm: &str,
    drug: &str,
    event: &str,
    score: f64,
) {
    bus.emit(FlywheelEvent::broadcast(
        FlywheelTier::Live,
        EventKind::Custom {
            label: "pv_signal_detected".to_owned(),
            data: serde_json::json!({
                "algorithm": algorithm,
                "drug": drug,
                "event": event,
                "score": score,
            }),
        },
    ));
}

/// Emit a causality assessment event when Naranjo or WHO-UMC scoring completes.
///
/// Called after a causality algorithm produces a verdict for a drug-event pair.
/// The `method` identifies the framework (e.g. `"naranjo"`, `"who_umc"`),
/// `verdict` is the classification (e.g. `"probable"`, `"possible"`),
/// and `score` is the numeric result (Naranjo points or equivalent).
pub fn emit_causality_assessed(
    bus: &FlywheelBus,
    method: &str,
    drug: &str,
    event: &str,
    verdict: &str,
    score: f64,
) {
    bus.emit(FlywheelEvent::broadcast(
        FlywheelTier::Live,
        EventKind::Custom {
            label: "pv_causality_assessed".to_owned(),
            data: serde_json::json!({
                "method": method,
                "drug": drug,
                "event": event,
                "verdict": verdict,
                "score": score,
            }),
        },
    ));
}

/// Emit a safety margin shift event when d(s) changes for a monitored drug.
///
/// Called when the ToV §9.2 formal safety margin recalculates and the distance
/// shifts by `delta` (positive = safer, negative = more dangerous).
pub fn emit_safety_margin_shift(bus: &FlywheelBus, drug: &str, old: f64, new: f64) {
    bus.emit(FlywheelEvent::broadcast(
        FlywheelTier::Live,
        EventKind::Custom {
            label: "pv_safety_margin_shift".to_owned(),
            data: serde_json::json!({
                "drug": drug,
                "old_margin": old,
                "new_margin": new,
                "delta": new - old,
            }),
        },
    ));
}

/// Consume pending flywheel events relevant to the PV signal node.
///
/// Drains `Custom` events with PV-domain labels (`pv_signal_detected`,
/// `pv_causality_assessed`, `pv_safety_margin_shift`) plus `BaselineShift`
/// and `ThresholdDrift` events that may affect signal thresholds.
/// PV signal detection reacts to homeostasis baseline shifts (recalibrating
/// thresholds) and its own prior signals (temporal trend analysis).
pub fn consume_pv_events(bus: &FlywheelBus) -> Vec<FlywheelEvent> {
    let events = bus.consume(FlywheelTier::Live);
    events
        .into_iter()
        .filter(|e| match &e.kind {
            EventKind::Custom { label, .. } => label.starts_with("pv_"),
            EventKind::BaselineShift { .. } | EventKind::ThresholdDrift { .. } => true,
            _ => false,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_consume_pv_events_filters() {
        let bus = FlywheelBus::new();

        // Relevant: PV signal
        emit_signal_detected(&bus, "PRR", "metformin", "lactic_acidosis", 3.1);
        // Relevant: baseline shift from homeostasis
        bus.emit(FlywheelEvent::broadcast(
            FlywheelTier::Live,
            EventKind::BaselineShift {
                metric: "threat_level".to_owned(),
                old: 0.5,
                new: 0.7,
            },
        ));
        // Irrelevant: skill maturation
        bus.emit(FlywheelEvent::broadcast(
            FlywheelTier::Live,
            EventKind::MaturationSignal {
                skill: "forge".to_owned(),
                transfer_score: 0.8,
            },
        ));

        let consumed = consume_pv_events(&bus);
        assert_eq!(
            consumed.len(),
            2,
            "should consume pv_signal + baseline, not maturation"
        );
    }

    #[test]
    fn test_consume_includes_own_signals() {
        let bus = FlywheelBus::new();
        emit_causality_assessed(&bus, "naranjo", "warfarin", "bleeding", "probable", 7.0);
        emit_safety_margin_shift(&bus, "warfarin", 0.85, 0.72);

        let consumed = consume_pv_events(&bus);
        assert_eq!(consumed.len(), 2, "PV node consumes its own domain events");
    }

    #[test]
    fn test_emit_signal_detected() {
        let bus = FlywheelBus::new();
        emit_signal_detected(&bus, "PRR", "rofecoxib", "myocardial_infarction", 4.2);

        let events = bus.consume(FlywheelTier::Staging);
        assert_eq!(events.len(), 1, "expected exactly one event");
        match &events[0].kind {
            EventKind::Custom { label, data } => {
                assert_eq!(label, "pv_signal_detected");
                assert_eq!(data["algorithm"], "PRR");
                assert_eq!(data["drug"], "rofecoxib");
                assert_eq!(data["event"], "myocardial_infarction");
                assert!((data["score"].as_f64().unwrap_or(0.0) - 4.2).abs() < f64::EPSILON);
            }
            other => panic!("unexpected event kind: {other:?}"),
        }
    }

    #[test]
    fn test_emit_causality_assessed() {
        let bus = FlywheelBus::new();
        emit_causality_assessed(&bus, "naranjo", "ibuprofen", "gi_bleed", "probable", 7.0);

        let events = bus.consume(FlywheelTier::Draft);
        assert_eq!(events.len(), 1, "expected exactly one event");
        match &events[0].kind {
            EventKind::Custom { label, data } => {
                assert_eq!(label, "pv_causality_assessed");
                assert_eq!(data["verdict"], "probable");
                assert!((data["score"].as_f64().unwrap_or(0.0) - 7.0).abs() < f64::EPSILON);
            }
            other => panic!("unexpected event kind: {other:?}"),
        }
    }

    #[test]
    fn test_emit_safety_margin_shift() {
        let bus = FlywheelBus::new();
        emit_safety_margin_shift(&bus, "warfarin", 0.85, 0.72);

        let events = bus.consume(FlywheelTier::Live);
        assert_eq!(events.len(), 1, "expected exactly one event");
        match &events[0].kind {
            EventKind::Custom { label, data } => {
                assert_eq!(label, "pv_safety_margin_shift");
                assert_eq!(data["drug"], "warfarin");
                let delta = data["delta"].as_f64().unwrap_or(0.0);
                assert!((delta - (-0.13)).abs() < 1e-10);
            }
            other => panic!("unexpected event kind: {other:?}"),
        }
    }
}
