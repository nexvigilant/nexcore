// Copyright (c) 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! # nexcore-flywheel
//!
//! Three-node flywheel bridge with five autonomous loops.
//! Transport layer (bus, events, nodes) + loop logic (rim, momentum, friction, gyroscopic, elastic).
//!
//! ## Governing Equation: E = ½Iω²

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![cfg_attr(
    not(test),
    deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)
)]

pub mod bridge;
pub mod components;
pub mod equations;
pub mod event;
pub mod loops;
pub mod modules;
pub mod node;
pub mod refinery;
pub mod registry;
pub mod simulation;
pub mod station;
pub mod thresholds;
pub mod vdag;
pub mod vitals;

pub use bridge::{FlywheelBus, FlywheelSnapshot};
pub use components::{ComponentHealth, ComponentKind, FlywheelAnatomy};
pub use equations::{FlywheelPhysics, PhysicsInput};
pub use event::{EventKind, FlywheelEvent};
pub use loops::{CascadeInput, CascadeResult, SystemState, cascade};
pub use modules::Module;
pub use node::{FlywheelTier, NodeDescriptor, NodeStatus};
pub use refinery::{
    FractionationResult, ProductBreakdown, ProductType, RefineryMetrics, SignalFraction,
};
pub use registry::NodeRegistry;
pub use simulation::{FlywheelState, Scenario, SimulationConfig, Trajectory, TrajectorySummary};
pub use station::{StationEvent, StationEventNotification, station_event_to_flywheel};
pub use thresholds::FlywheelThresholds;
pub use vdag::{
    CascadeRecord, EvidenceQuality, FlywheelGoal, GradedCascadeResult, LearningInsight,
    LearningLoopType, LoopEvidence, RealityGradient, RealityRating, ThresholdAdjustment,
};
pub use vitals::FlywheelVitals;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_emit_consume_roundtrip() {
        let bus = FlywheelBus::new();
        bus.emit(FlywheelEvent::targeted(
            FlywheelTier::Live,
            FlywheelTier::Staging,
            EventKind::CycleComplete { iteration: 1 },
        ));
        assert_eq!(bus.consume(FlywheelTier::Staging).len(), 1);
        assert_eq!(bus.consume(FlywheelTier::Live).len(), 0);
        assert_eq!(bus.pending_count(), 0);
    }

    #[test]
    fn test_broadcast_reaches_all() {
        let bus = FlywheelBus::new();
        bus.emit(FlywheelEvent::broadcast(
            FlywheelTier::Live,
            EventKind::BaselineShift {
                metric: "threat_level".into(),
                old: 0.5,
                new: 0.3,
            },
        ));
        assert_eq!(bus.consume(FlywheelTier::Staging).len(), 1);
        assert_eq!(bus.consume(FlywheelTier::Draft).len(), 0);
    }

    #[test]
    fn test_snapshot_serializable() {
        let bus = FlywheelBus::new();
        bus.emit(FlywheelEvent::broadcast(
            FlywheelTier::Live,
            EventKind::ThresholdDrift {
                parameter: "upf_sensitivity".into(),
                delta: -0.02,
            },
        ));
        let snapshot = bus.snapshot();
        assert_eq!(snapshot.pending_events, 1);
        let json = serde_json::to_string(&snapshot);
        assert!(json.is_ok());
        let parsed: Result<FlywheelSnapshot, _> = serde_json::from_str(&json.unwrap_or_default());
        assert!(parsed.is_ok());
    }

    #[test]
    fn test_promote_node() {
        let mut registry = NodeRegistry::default_three_node();
        let node = registry.find("immunity");
        assert_eq!(node.map(|n| n.tier), Some(FlywheelTier::Live));
        let promoted = registry.promote("immunity");
        assert!(!promoted);
    }

    #[test]
    fn test_default_registry() {
        let registry = NodeRegistry::default_three_node();
        assert_eq!(registry.len(), 7);
        let (live, staging, draft) = registry.count_by_tier();
        assert_eq!(live, 4);
        assert_eq!(staging, 3);
        assert_eq!(draft, 0);
    }

    #[test]
    fn test_consume_empty_bus() {
        assert!(FlywheelBus::new().consume(FlywheelTier::Live).is_empty());
    }
    #[test]
    fn test_promote_nonexistent() {
        assert!(!NodeRegistry::default_three_node().promote("nope"));
    }

    #[test]
    fn test_staging_to_live() {
        let mut r = NodeRegistry::default_three_node();
        assert!(r.promote("skill-maturation"));
        assert_eq!(
            r.find("skill-maturation").map(|n| n.tier),
            Some(FlywheelTier::Live)
        );
    }

    #[test]
    fn test_double_promote_from_staging() {
        let mut r = NodeRegistry::default_three_node();
        // insight starts at Staging, first promote → Live
        assert!(r.promote("insight"));
        assert_eq!(r.find("insight").map(|n| n.tier), Some(FlywheelTier::Live));
        // second promote from Live → Live (no-op, returns false)
        assert!(!r.promote("insight"));
    }

    #[test]
    fn test_targeted_not_wrong_tier() {
        let bus = FlywheelBus::new();
        bus.emit(FlywheelEvent::targeted(
            FlywheelTier::Live,
            FlywheelTier::Draft,
            EventKind::CycleComplete { iteration: 42 },
        ));
        assert!(bus.consume(FlywheelTier::Live).is_empty());
        assert!(bus.consume(FlywheelTier::Staging).is_empty());
        assert_eq!(bus.consume(FlywheelTier::Draft).len(), 1);
    }

    #[test]
    fn test_multiple_selective() {
        let bus = FlywheelBus::new();
        bus.emit(FlywheelEvent::targeted(
            FlywheelTier::Live,
            FlywheelTier::Staging,
            EventKind::CycleComplete { iteration: 1 },
        ));
        bus.emit(FlywheelEvent::targeted(
            FlywheelTier::Live,
            FlywheelTier::Draft,
            EventKind::CycleComplete { iteration: 2 },
        ));
        bus.emit(FlywheelEvent::targeted(
            FlywheelTier::Staging,
            FlywheelTier::Live,
            EventKind::TrustUpdate {
                score: 0.85,
                level: "high".into(),
            },
        ));
        assert_eq!(bus.pending_count(), 3);
        assert_eq!(bus.consume(FlywheelTier::Staging).len(), 1);
        assert_eq!(bus.consume(FlywheelTier::Live).len(), 1);
        assert_eq!(bus.consume(FlywheelTier::Draft).len(), 1);
    }

    #[test]
    fn test_clear() {
        let bus = FlywheelBus::new();
        bus.emit(FlywheelEvent::broadcast(
            FlywheelTier::Live,
            EventKind::CycleComplete { iteration: 1 },
        ));
        bus.clear();
        assert_eq!(bus.pending_count(), 0);
    }
    #[test]
    fn test_clone_shares_state() {
        let b1 = FlywheelBus::new();
        let b2 = b1.clone();
        b1.emit(FlywheelEvent::broadcast(
            FlywheelTier::Live,
            EventKind::CycleComplete { iteration: 1 },
        ));
        assert_eq!(b2.consume(FlywheelTier::Staging).len(), 1);
    }
    #[test]
    fn test_not_empty() {
        assert!(!NodeRegistry::default_three_node().is_empty());
    }

    #[test]
    fn test_nodes_in_tier() {
        let r = NodeRegistry::default_three_node();
        assert_eq!(r.nodes_in_tier(FlywheelTier::Live).len(), 4);
        assert_eq!(r.nodes_in_tier(FlywheelTier::Staging).len(), 3);
        assert_eq!(r.nodes_in_tier(FlywheelTier::Draft).len(), 0);
    }

    #[test]
    fn test_event_with_payload() {
        let bus = FlywheelBus::new();
        let event = FlywheelEvent::broadcast(
            FlywheelTier::Live,
            EventKind::Custom {
                label: "test".into(),
                data: serde_json::json!({"key": "value"}),
            },
        )
        .with_payload(serde_json::json!({"extra": true}));
        assert!(event.payload.is_some());
        bus.emit(event);
        let events = bus.consume(FlywheelTier::Staging);
        assert!(events[0].payload.is_some());
    }

    #[test]
    fn test_all_event_kinds_serialize() {
        let kinds = vec![
            EventKind::CycleComplete { iteration: 1 },
            EventKind::BaselineShift {
                metric: "m".into(),
                old: 1.0,
                new: 2.0,
            },
            EventKind::ThresholdDrift {
                parameter: "p".into(),
                delta: 0.1,
            },
            EventKind::AdaptationReady {
                category: "c".into(),
            },
            EventKind::TrustUpdate {
                score: 0.9,
                level: "high".into(),
            },
            EventKind::MaturationSignal {
                skill: "s".into(),
                transfer_score: 0.8,
            },
            EventKind::InsightAccumulated { pattern_count: 42 },
            EventKind::FractionationComplete {
                health_count: 30,
                threat_count: 20,
                learning_count: 40,
                noise_stripped: 10,
            },
            EventKind::YieldReport {
                yield_pct: 0.70,
                conversion_pct: 0.90,
                selectivity_pct: 0.57,
                recycle_ratio: 0.22,
                loss_ratio: 0.10,
            },
            EventKind::RecycleStream {
                signal_count: 20,
                source_node: "immunity".into(),
            },
            EventKind::Custom {
                label: "l".into(),
                data: serde_json::json!(null),
            },
        ];
        for kind in kinds {
            let event = FlywheelEvent::broadcast(FlywheelTier::Live, kind);
            let json = serde_json::to_string(&event);
            assert!(json.is_ok());
            let parsed: Result<FlywheelEvent, _> = serde_json::from_str(&json.unwrap_or_default());
            assert!(parsed.is_ok());
        }
    }

    // ── Cascade integration tests ────────────────────────────────────────

    #[test]
    fn test_cascade_thriving() {
        use loops::*;
        let input = CascadeInput {
            rim: RimInput {
                tensile_strength: 200.0,
                centrifugal_force: 50.0,
            },
            momentum: MomentumInput {
                inertia: 100.0,
                omega: 5.0,
                friction_drain: 0.0,
            },
            friction: FrictionInput {
                manual_processes: 1.0,
                human_touchpoints: 1.0,
                velocity: 1.0,
                automation_coverage: 0.9,
            },
            gyroscopic: GyroscopicInput {
                momentum_l: 500.0,
                perturbation_torque: 10.0,
                critical_momentum: 50.0,
            },
            elastic: ElasticInput {
                stress: 20.0,
                yield_point: 100.0,
                fatigue_cycles: 50,
                fatigue_limit: 1000,
            },
        };
        let r = cascade(&input, &FlywheelThresholds::default());
        assert_eq!(r.system_state, SystemState::Thriving);
    }

    #[test]
    fn test_cascade_failed_fatigue() {
        use loops::*;
        let input = CascadeInput {
            rim: RimInput {
                tensile_strength: 200.0,
                centrifugal_force: 50.0,
            },
            momentum: MomentumInput {
                inertia: 100.0,
                omega: 5.0,
                friction_drain: 0.0,
            },
            friction: FrictionInput {
                manual_processes: 1.0,
                human_touchpoints: 1.0,
                velocity: 1.0,
                automation_coverage: 0.5,
            },
            gyroscopic: GyroscopicInput {
                momentum_l: 0.0,
                perturbation_torque: 10.0,
                critical_momentum: 50.0,
            },
            elastic: ElasticInput {
                stress: 50.0,
                yield_point: 100.0,
                fatigue_cycles: 2000,
                fatigue_limit: 1000,
            },
        };
        assert_eq!(
            cascade(&input, &FlywheelThresholds::default()).system_state,
            SystemState::Failed
        );
    }

    #[test]
    fn test_cascade_failed_disintegration() {
        use loops::*;
        let input = CascadeInput {
            rim: RimInput {
                tensile_strength: 10.0,
                centrifugal_force: 100.0,
            },
            momentum: MomentumInput {
                inertia: 100.0,
                omega: 5.0,
                friction_drain: 0.0,
            },
            friction: FrictionInput {
                manual_processes: 1.0,
                human_touchpoints: 1.0,
                velocity: 1.0,
                automation_coverage: 0.5,
            },
            gyroscopic: GyroscopicInput {
                momentum_l: 0.0,
                perturbation_torque: 10.0,
                critical_momentum: 50.0,
            },
            elastic: ElasticInput {
                stress: 20.0,
                yield_point: 100.0,
                fatigue_cycles: 50,
                fatigue_limit: 1000,
            },
        };
        assert_eq!(
            cascade(&input, &FlywheelThresholds::default()).system_state,
            SystemState::Failed
        );
    }

    #[test]
    fn test_cascade_friction_drains_momentum() {
        use loops::*;
        let input = CascadeInput {
            rim: RimInput {
                tensile_strength: 200.0,
                centrifugal_force: 50.0,
            },
            momentum: MomentumInput {
                inertia: 10.0,
                omega: 5.0,
                friction_drain: 0.0,
            },
            friction: FrictionInput {
                manual_processes: 100.0,
                human_touchpoints: 100.0,
                velocity: 1.0,
                automation_coverage: 0.0,
            },
            gyroscopic: GyroscopicInput {
                momentum_l: 0.0,
                perturbation_torque: 10.0,
                critical_momentum: 50.0,
            },
            elastic: ElasticInput {
                stress: 20.0,
                yield_point: 100.0,
                fatigue_cycles: 50,
                fatigue_limit: 1000,
            },
        };
        let r = cascade(&input, &FlywheelThresholds::default());
        assert_eq!(
            r.momentum.classification,
            loops::momentum::MomentumClassification::Stalled
        );
    }

    #[test]
    fn test_cascade_serializable() {
        use loops::*;
        let input = CascadeInput {
            rim: RimInput {
                tensile_strength: 100.0,
                centrifugal_force: 50.0,
            },
            momentum: MomentumInput {
                inertia: 100.0,
                omega: 5.0,
                friction_drain: 0.0,
            },
            friction: FrictionInput {
                manual_processes: 1.0,
                human_touchpoints: 1.0,
                velocity: 1.0,
                automation_coverage: 0.5,
            },
            gyroscopic: GyroscopicInput {
                momentum_l: 0.0,
                perturbation_torque: 10.0,
                critical_momentum: 50.0,
            },
            elastic: ElasticInput {
                stress: 20.0,
                yield_point: 100.0,
                fatigue_cycles: 50,
                fatigue_limit: 1000,
            },
        };
        let r = cascade(&input, &FlywheelThresholds::default());
        let json = serde_json::to_string(&r);
        assert!(json.is_ok());
        let parsed: Result<CascadeResult, _> = serde_json::from_str(&json.unwrap_or_default());
        assert!(parsed.is_ok());
    }
}
