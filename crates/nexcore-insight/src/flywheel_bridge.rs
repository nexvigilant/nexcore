//! Flywheel bridge — emits insight accumulation events into the flywheel bus.
//!
//! ## T1 Grounding
//!
//! | Function | Primitives |
//! |---|---|
//! | `emit_insight_accumulated` | N (quantity: pattern_count) + ρ (recursion: learning loop) |
//! | `emit_novelty_detected` | ∅ (void: gap found) + κ (comparison: novelty threshold) |

use nexcore_flywheel::{EventKind, FlywheelBus, FlywheelEvent, node::FlywheelTier};

/// Emit an insight accumulation event when the engine ingests patterns.
///
/// Called after the insight engine processes one or more observations and
/// accumulates new patterns. The `pattern_count` reflects the total
/// accumulated patterns after ingestion.
///
/// # Arguments
///
/// * `bus`           — shared flywheel bus
/// * `pattern_count` — total accumulated patterns after this ingestion cycle
pub fn emit_insight_accumulated(bus: &FlywheelBus, pattern_count: u64) -> FlywheelEvent {
    let kind = EventKind::InsightAccumulated { pattern_count };
    bus.emit(FlywheelEvent::broadcast(FlywheelTier::Staging, kind))
}

/// Emit a novelty detection event when the engine identifies a novel pattern.
///
/// Called when an observation's novelty score exceeds the configured threshold,
/// indicating a pattern not seen in prior ingestion cycles.
///
/// # Arguments
///
/// * `bus`       — shared flywheel bus
/// * `source`    — adapter that produced the observation (e.g. `"guardian"`, `"faers"`)
/// * `novelty`   — novelty score in `[0.0, 1.0]`
/// * `summary`   — short description of the novel pattern
pub fn emit_novelty_detected(
    bus: &FlywheelBus,
    source: &str,
    novelty: f64,
    summary: &str,
) -> FlywheelEvent {
    let kind = EventKind::NoveltyDetected {
        source: source.to_owned(),
        novelty,
        summary: summary.to_owned(),
    };
    bus.emit(FlywheelEvent::broadcast(FlywheelTier::Staging, kind))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_emit_insight_accumulated() {
        let bus = FlywheelBus::new();
        let event = emit_insight_accumulated(&bus, 42);

        match &event.kind {
            EventKind::InsightAccumulated { pattern_count } => {
                assert_eq!(*pattern_count, 42);
            }
            other => panic!("unexpected event kind: {other:?}"),
        }

        let consumed = bus.consume(FlywheelTier::Staging);
        assert_eq!(consumed.len(), 1, "expected 1 event in Staging tier");
    }

    #[test]
    fn test_emit_novelty_detected() {
        let bus = FlywheelBus::new();
        let event = emit_novelty_detected(&bus, "guardian", 0.91, "new threat pattern");

        match &event.kind {
            EventKind::NoveltyDetected {
                source,
                novelty,
                summary,
            } => {
                assert_eq!(source, "guardian");
                assert!((*novelty - 0.91).abs() < f64::EPSILON);
                assert_eq!(summary, "new threat pattern");
            }
            other => panic!("unexpected event kind: {other:?}"),
        }

        let consumed = bus.consume(FlywheelTier::Staging);
        assert_eq!(consumed.len(), 1, "expected 1 event in Staging tier");
    }

    #[test]
    fn test_insight_event_targets_staging() {
        let bus = FlywheelBus::new();
        emit_insight_accumulated(&bus, 10);

        let events = bus.consume(FlywheelTier::Staging);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].source_node, FlywheelTier::Staging);
        assert!(
            events[0].target_node.is_none(),
            "broadcast event must have no specific target"
        );
    }
}
