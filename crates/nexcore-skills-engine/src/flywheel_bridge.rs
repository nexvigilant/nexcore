//! Flywheel bridge — emits skill maturation events into the flywheel bus.
//!
//! ## T1 Grounding
//!
//! | Function | Primitives |
//! |---|---|
//! | `emit_maturation_signal` | ς (state: skill tier) + N (quantity: transfer_score) + → (causality) |
//! | `emit_skill_promoted` | ∂ (boundary: tier gate) + μ (mapping: old_tier → new_tier) |

use nexcore_flywheel::{EventKind, FlywheelBus, FlywheelEvent, node::FlywheelTier};

/// Emit a maturation signal when a skill's transfer score crosses a tier threshold.
///
/// Called when the skill maturation engine detects that a skill has accumulated
/// enough demonstrations to warrant tier advancement consideration.
///
/// # Arguments
///
/// * `bus`            — shared flywheel bus
/// * `skill`         — skill name or path
/// * `transfer_score` — computed maturation score in `[0.0, 1.0]`
pub fn emit_maturation_signal(
    bus: &FlywheelBus,
    skill: &str,
    transfer_score: f64,
) -> FlywheelEvent {
    let kind = EventKind::MaturationSignal {
        skill: skill.to_owned(),
        transfer_score,
    };
    bus.emit(FlywheelEvent::broadcast(FlywheelTier::Staging, kind))
}

/// Emit a promotion event when a skill advances tiers (e.g. Draft → Staging → Live).
///
/// Called after a skill passes all promotion gates and its tier is updated.
///
/// # Arguments
///
/// * `bus`      — shared flywheel bus
/// * `skill`    — skill name or path
/// * `old_tier` — tier before promotion (e.g. `"draft"`)
/// * `new_tier` — tier after promotion (e.g. `"staging"`)
pub fn emit_skill_promoted(
    bus: &FlywheelBus,
    skill: &str,
    old_tier: &str,
    new_tier: &str,
) -> FlywheelEvent {
    let kind = EventKind::SkillPromoted {
        skill: skill.to_owned(),
        old_tier: old_tier.to_owned(),
        new_tier: new_tier.to_owned(),
    };
    bus.emit(FlywheelEvent::broadcast(FlywheelTier::Staging, kind))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_emit_maturation_signal() {
        let bus = FlywheelBus::new();
        let event = emit_maturation_signal(&bus, "forge", 0.78);

        match &event.kind {
            EventKind::MaturationSignal {
                skill,
                transfer_score,
            } => {
                assert_eq!(skill, "forge");
                assert!(
                    (*transfer_score - 0.78).abs() < f64::EPSILON,
                    "score mismatch: expected 0.78, got {transfer_score}"
                );
            }
            other => panic!("unexpected event kind: {other:?}"),
        }

        let consumed = bus.consume(FlywheelTier::Staging);
        assert_eq!(consumed.len(), 1, "expected 1 event in Staging tier");
    }

    #[test]
    fn test_emit_skill_promoted() {
        let bus = FlywheelBus::new();
        let event = emit_skill_promoted(&bus, "forge", "draft", "staging");

        match &event.kind {
            EventKind::SkillPromoted {
                skill,
                old_tier,
                new_tier,
            } => {
                assert_eq!(skill, "forge");
                assert_eq!(old_tier, "draft");
                assert_eq!(new_tier, "staging");
            }
            other => panic!("unexpected event kind: {other:?}"),
        }

        let consumed = bus.consume(FlywheelTier::Staging);
        assert_eq!(consumed.len(), 1, "expected 1 event in Staging tier");
    }

    #[test]
    fn test_maturation_event_targets_staging() {
        let bus = FlywheelBus::new();
        emit_maturation_signal(&bus, "rust-dev", 0.92);

        let events = bus.consume(FlywheelTier::Staging);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].source_node, FlywheelTier::Staging);
        assert!(
            events[0].target_node.is_none(),
            "broadcast event must have no specific target"
        );
    }
}
