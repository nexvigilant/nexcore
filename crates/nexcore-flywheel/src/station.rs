//! Station event bridge — maps NexVigilant Station events to FlywheelEvents.
//!
//! The station broadcasts `StationEvent` via SSE after every tool call.
//! This module provides the mapping function that converts station events
//! into `FlywheelEvent::Custom` for the bus.
//!
//! ## Bus Topology (2026-03-06)
//!
//! ```text
//! Station (ferroforge)          nexcore-flywheel           Consumers
//! ┌──────────────────┐   SSE   ┌──────────────────┐       ┌──────────┐
//! │ tool_call         │───A→B──│ station_event_to_ │──D→E──│ 3 bridges│
//! │ → broadcast(event)│        │ flywheel() → emit │       │ (F)      │
//! └──────────────────┘        └──────────────────┘       └────┬─────┘
//!         ↑                                                    │
//!         └──── G: flywheel_react() → LoopGuard → Actions ←──┘
//! ```
//!
//! Forward path A→F verified end-to-end. Return path G closed (2026-03-06):
//! - Link 6: `LoopGuard` — ∂ boundary (max_depth + cooldown rate limiter)
//! - Link 5: `flywheel_event_to_action()` — μ mapping (EventKind → StationAction)
//! - Link 7: `flywheel_react()` — → causality (consume → rules → guard → actions)
//!
//! ## T1 Primitive Grounding
//!
//! | Concept | Primitive | Symbol |
//! |---------|-----------|--------|
//! | Event mapping | Mapping | μ |
//! | Cross-boundary bridge | Boundary | ∂ |
//! | Domain extraction | Location | λ |
//! | Loop guard | Boundary + Frequency | ∂ + ν |
//! | Decision rules | Mapping + Causality | μ + → |
//! | Runtime wiring | Causality + Sequence | → + σ |

use crate::bridge::FlywheelBus;
use crate::event::{EventKind, FlywheelEvent};
use crate::node::FlywheelTier;
use serde::{Deserialize, Serialize};
use std::time::Instant;

/// A station event as received from the NexVigilant Station SSE stream.
///
/// This mirrors `protocol::StationEvent` in ferroforge but is defined
/// independently to avoid cross-workspace dependencies.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StationEvent {
    /// Domain that handled the call (e.g., "api.fda.gov")
    pub domain: String,
    /// MCP tool name (e.g., "api_fda_gov_search_adverse_events")
    pub tool: String,
    /// Outcome: "ok", "error", "stub", "no_handler"
    pub status: String,
    /// Wall-clock duration in milliseconds
    pub duration_ms: u64,
    /// ISO 8601 timestamp
    pub timestamp: String,
}

/// The JSON-RPC notification envelope as sent by the station.
#[derive(Debug, Clone, Deserialize)]
pub struct StationEventNotification {
    /// Always "2.0"
    #[allow(dead_code)]
    pub jsonrpc: String,
    /// Always "notifications/station_event"
    #[allow(dead_code)]
    pub method: String,
    /// The station event payload
    pub params: StationEvent,
}

/// Map a station event to a flywheel event.
///
/// Station events enter the flywheel as `EventKind::Custom` broadcasts
/// from `FlywheelTier::Live` (the station is live infrastructure).
pub fn station_event_to_flywheel(event: StationEvent) -> FlywheelEvent {
    let payload = serde_json::to_value(&event).unwrap_or_default();
    FlywheelEvent::broadcast(
        FlywheelTier::Live,
        EventKind::Custom {
            label: "station_tool_call".into(),
            data: serde_json::json!({
                "domain": event.domain,
                "tool": event.tool,
                "status": event.status,
                "duration_ms": event.duration_ms,
            }),
        },
    )
    .with_payload(payload)
}

// ── Link 6: LoopGuard (∂ — boundary before capability) ────────────────────

/// Prevents runaway feedback loops by enforcing depth and rate limits.
///
/// The guard sits at G (return path) and ensures that flywheel reactions
/// cannot cascade indefinitely. Without it, a single event could trigger
/// an infinite loop of station calls.
///
/// ## T1 Grounding: ∂ (Boundary) + ν (Frequency) + ς (State)
pub struct LoopGuard {
    /// Maximum cascading reactions per consume cycle.
    pub max_depth: u8,
    /// Minimum milliseconds between actions.
    pub cooldown_ms: u64,
    /// Tracks when the last action was dispatched.
    last_action: Option<Instant>,
    /// Actions dispatched in the current cycle.
    depth: u8,
}

impl LoopGuard {
    /// Create a guard with the given limits.
    pub fn new(max_depth: u8, cooldown_ms: u64) -> Self {
        Self {
            max_depth,
            cooldown_ms,
            last_action: None,
            depth: 0,
        }
    }

    /// Default production guard: max 3 reactions, 500ms cooldown.
    pub fn production() -> Self {
        Self::new(3, 500)
    }

    /// Check whether an action is allowed. Returns `true` and increments
    /// depth if the guard permits it, `false` if the limit is hit.
    pub fn allow(&mut self) -> bool {
        if self.depth >= self.max_depth {
            return false;
        }
        if let Some(last) = self.last_action
            && last.elapsed().as_millis() < u128::from(self.cooldown_ms)
        {
            return false;
        }
        self.depth += 1;
        self.last_action = Some(Instant::now());
        true
    }

    /// Reset the depth counter (call at the start of each consume cycle).
    pub fn reset_cycle(&mut self) {
        self.depth = 0;
    }
}

// ── Link 5: Decision Rules (μ — mapping events to actions) ───────────────

/// An action the flywheel wants to dispatch back to the station.
///
/// This is the content of return path G — the void we're filling.
/// Each action names a station tool and its arguments.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StationAction {
    /// MCP tool name to call on the station.
    pub tool: String,
    /// JSON arguments for the tool call.
    pub arguments: serde_json::Value,
    /// Human-readable reason for the action.
    pub reason: String,
}

/// Map a flywheel event to an optional station action.
///
/// This is the decision engine at G. It watches for specific EventKind
/// patterns and produces StationActions when thresholds are exceeded.
///
/// Current rules:
/// - `NoveltyDetected` with novelty > 0.7 → investigate via literature search
/// - `ThresholdDrift` with |delta| > 0.1 → re-query the drifting parameter's domain
/// - `Custom { label: "station_tool_call" }` with status "error" → retry the tool
pub fn flywheel_event_to_action(event: &FlywheelEvent) -> Option<StationAction> {
    match &event.kind {
        EventKind::NoveltyDetected {
            source,
            novelty,
            summary,
        } if *novelty > 0.7 => Some(StationAction {
            tool: "pubmed_ncbi_nlm_nih_gov_search_articles".into(),
            arguments: serde_json::json!({
                "query": summary,
                "max_results": 5,
            }),
            reason: format!(
                "Novelty {novelty:.2} from {source} exceeds 0.7 threshold — investigating"
            ),
        }),
        EventKind::ThresholdDrift { parameter, delta } if delta.abs() > 0.1 => {
            Some(StationAction {
                tool: "api_fda_gov_search_adverse_events".into(),
                arguments: serde_json::json!({
                    "search": parameter,
                    "limit": 10,
                }),
                reason: format!(
                    "Threshold drift {delta:+.3} on {parameter} exceeds ±0.1 — re-querying FAERS"
                ),
            })
        }
        EventKind::Custom { label, data }
            if label == "station_tool_call"
                && data.get("status").and_then(|s| s.as_str()) == Some("error") =>
        {
            let tool = data
                .get("tool")
                .and_then(|t| t.as_str())
                .unwrap_or("unknown")
                .to_string();
            Some(StationAction {
                tool: tool.clone(),
                arguments: serde_json::json!({}),
                reason: format!("Retrying failed station tool call: {tool}"),
            })
        }
        _ => None,
    }
}

// ── Link 7: Runtime Wiring (→ — causality, closing the loop) ─────────────

/// Consume events from the bus, apply decision rules through the guard,
/// and collect resulting station actions.
///
/// This is the runtime function that closes return path G:
/// ```text
/// FlywheelBus → consume → flywheel_event_to_action → LoopGuard → Vec<StationAction>
/// ```
///
/// The caller (e.g., `flywheel_react` MCP tool) is responsible for
/// dispatching the returned actions to the station.
pub fn flywheel_react(
    bus: &FlywheelBus,
    tier: FlywheelTier,
    guard: &mut LoopGuard,
) -> Vec<StationAction> {
    guard.reset_cycle();
    let events = bus.consume(tier);
    events
        .iter()
        .filter_map(|event| {
            let action = flywheel_event_to_action(event)?;
            if guard.allow() { Some(action) } else { None }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_station_event_to_flywheel() {
        let event = StationEvent {
            domain: "api.fda.gov".into(),
            tool: "api_fda_gov_search_adverse_events".into(),
            status: "ok".into(),
            duration_ms: 42,
            timestamp: "2026-03-06T22:52:33Z".into(),
        };

        let flywheel = station_event_to_flywheel(event);
        assert_eq!(flywheel.source_node, FlywheelTier::Live);
        assert!(flywheel.target_node.is_none()); // broadcast
        assert!(flywheel.payload.is_some());

        // Verify the Custom kind has the right label
        match &flywheel.kind {
            EventKind::Custom { label, data } => {
                assert_eq!(label, "station_tool_call");
                assert_eq!(data["domain"], "api.fda.gov");
                assert_eq!(data["duration_ms"], 42);
            }
            other => panic!("expected Custom, got {other:?}"),
        }
    }

    #[test]
    fn test_station_event_roundtrip_serialization() {
        let event = StationEvent {
            domain: "pubmed.ncbi.nlm.nih.gov".into(),
            tool: "pubmed_ncbi_nlm_nih_gov_search_articles".into(),
            status: "error".into(),
            duration_ms: 1500,
            timestamp: "2026-03-06T23:00:00Z".into(),
        };

        let json = serde_json::to_string(&event).expect("serialize");
        let parsed: StationEvent = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(parsed.domain, "pubmed.ncbi.nlm.nih.gov");
        assert_eq!(parsed.status, "error");
    }

    #[test]
    fn test_notification_envelope_deserialize() {
        let raw = r#"{
            "jsonrpc": "2.0",
            "method": "notifications/station_event",
            "params": {
                "domain": "api.fda.gov",
                "tool": "api_fda_gov_search_adverse_events",
                "status": "ok",
                "duration_ms": 6,
                "timestamp": "2026-03-06T22:52:33Z"
            }
        }"#;

        let notification: StationEventNotification =
            serde_json::from_str(raw).expect("deserialize notification");
        assert_eq!(notification.params.domain, "api.fda.gov");
        assert_eq!(notification.params.duration_ms, 6);

        // Convert to flywheel event
        let flywheel = station_event_to_flywheel(notification.params);
        assert!(flywheel.payload.is_some());
    }

    #[test]
    fn test_station_event_emits_on_bus() {
        let bus = FlywheelBus::new();
        let event = StationEvent {
            domain: "dailymed.nlm.nih.gov".into(),
            tool: "dailymed_nlm_nih_gov_search_drugs".into(),
            status: "ok".into(),
            duration_ms: 200,
            timestamp: "2026-03-06T23:05:00Z".into(),
        };

        let flywheel = station_event_to_flywheel(event);
        bus.emit(flywheel);

        // Broadcast reaches all tiers
        let snapshot = bus.snapshot();
        assert_eq!(snapshot.pending_events, 1);

        // Consume from any tier (broadcast)
        let events = bus.consume(FlywheelTier::Staging);
        assert_eq!(events.len(), 1);
        match &events[0].kind {
            EventKind::Custom { label, .. } => assert_eq!(label, "station_tool_call"),
            other => panic!("expected Custom, got {other:?}"),
        }
    }

    // ── Link 6: LoopGuard tests ──────────────────────────────────────────

    #[test]
    fn test_loop_guard_respects_max_depth() {
        let mut guard = LoopGuard::new(2, 0); // no cooldown
        assert!(guard.allow());
        assert!(guard.allow());
        assert!(!guard.allow()); // depth 2 exceeded
    }

    #[test]
    fn test_loop_guard_reset_cycle() {
        let mut guard = LoopGuard::new(1, 0);
        assert!(guard.allow());
        assert!(!guard.allow());
        guard.reset_cycle();
        assert!(guard.allow()); // depth reset
    }

    #[test]
    fn test_loop_guard_cooldown() {
        let mut guard = LoopGuard::new(10, 60_000); // 60s cooldown
        assert!(guard.allow()); // first is always allowed
        assert!(!guard.allow()); // too soon
    }

    #[test]
    fn test_loop_guard_production_defaults() {
        let guard = LoopGuard::production();
        assert_eq!(guard.max_depth, 3);
        assert_eq!(guard.cooldown_ms, 500);
    }

    // ── Link 5: Decision Rules tests ─────────────────────────────────────

    #[test]
    fn test_novelty_above_threshold_triggers_action() {
        let event = FlywheelEvent::broadcast(
            FlywheelTier::Live,
            EventKind::NoveltyDetected {
                source: "immunity".into(),
                novelty: 0.85,
                summary: "new adverse reaction pattern".into(),
            },
        );
        let action = flywheel_event_to_action(&event);
        assert!(action.is_some());
        let a = action.unwrap();
        assert_eq!(a.tool, "pubmed_ncbi_nlm_nih_gov_search_articles");
        assert!(a.reason.contains("0.85"));
    }

    #[test]
    fn test_novelty_below_threshold_no_action() {
        let event = FlywheelEvent::broadcast(
            FlywheelTier::Live,
            EventKind::NoveltyDetected {
                source: "immunity".into(),
                novelty: 0.3,
                summary: "minor fluctuation".into(),
            },
        );
        assert!(flywheel_event_to_action(&event).is_none());
    }

    #[test]
    fn test_threshold_drift_triggers_faers_query() {
        let event = FlywheelEvent::broadcast(
            FlywheelTier::Live,
            EventKind::ThresholdDrift {
                parameter: "prr_sensitivity".into(),
                delta: -0.25,
            },
        );
        let action = flywheel_event_to_action(&event);
        assert!(action.is_some());
        let a = action.unwrap();
        assert_eq!(a.tool, "api_fda_gov_search_adverse_events");
        assert!(a.reason.contains("prr_sensitivity"));
    }

    #[test]
    fn test_small_drift_no_action() {
        let event = FlywheelEvent::broadcast(
            FlywheelTier::Live,
            EventKind::ThresholdDrift {
                parameter: "minor".into(),
                delta: 0.05,
            },
        );
        assert!(flywheel_event_to_action(&event).is_none());
    }

    #[test]
    fn test_station_error_triggers_retry() {
        let event = FlywheelEvent::broadcast(
            FlywheelTier::Live,
            EventKind::Custom {
                label: "station_tool_call".into(),
                data: serde_json::json!({
                    "tool": "dailymed_nlm_nih_gov_search_drugs",
                    "status": "error",
                    "domain": "dailymed.nlm.nih.gov",
                }),
            },
        );
        let action = flywheel_event_to_action(&event);
        assert!(action.is_some());
        let a = action.unwrap();
        assert_eq!(a.tool, "dailymed_nlm_nih_gov_search_drugs");
        assert!(a.reason.contains("Retrying"));
    }

    #[test]
    fn test_station_ok_no_retry() {
        let event = FlywheelEvent::broadcast(
            FlywheelTier::Live,
            EventKind::Custom {
                label: "station_tool_call".into(),
                data: serde_json::json!({
                    "tool": "some_tool",
                    "status": "ok",
                }),
            },
        );
        assert!(flywheel_event_to_action(&event).is_none());
    }

    #[test]
    fn test_unrelated_event_no_action() {
        let event = FlywheelEvent::broadcast(
            FlywheelTier::Live,
            EventKind::CycleComplete { iteration: 42 },
        );
        assert!(flywheel_event_to_action(&event).is_none());
    }

    // ── Link 7: flywheel_react integration tests ─────────────────────────

    #[test]
    fn test_flywheel_react_closes_loop() {
        let bus = FlywheelBus::new();
        let mut guard = LoopGuard::new(5, 0);

        // Emit a novelty event that should trigger an action
        bus.emit(FlywheelEvent::broadcast(
            FlywheelTier::Live,
            EventKind::NoveltyDetected {
                source: "signal".into(),
                novelty: 0.9,
                summary: "metformin lactic acidosis cluster".into(),
            },
        ));

        let actions = flywheel_react(&bus, FlywheelTier::Staging, &mut guard);
        assert_eq!(actions.len(), 1);
        assert_eq!(actions[0].tool, "pubmed_ncbi_nlm_nih_gov_search_articles");
    }

    #[test]
    fn test_flywheel_react_guard_limits_actions() {
        let bus = FlywheelBus::new();
        let mut guard = LoopGuard::new(1, 0); // only 1 action per cycle

        // Emit 3 actionable events
        for i in 0..3 {
            bus.emit(FlywheelEvent::broadcast(
                FlywheelTier::Live,
                EventKind::ThresholdDrift {
                    parameter: format!("param_{i}"),
                    delta: 0.5,
                },
            ));
        }

        let actions = flywheel_react(&bus, FlywheelTier::Staging, &mut guard);
        assert_eq!(actions.len(), 1); // guard capped at 1
    }

    #[test]
    fn test_flywheel_react_empty_bus() {
        let bus = FlywheelBus::new();
        let mut guard = LoopGuard::production();
        let actions = flywheel_react(&bus, FlywheelTier::Live, &mut guard);
        assert!(actions.is_empty());
    }

    #[test]
    fn test_flywheel_react_mixed_events() {
        let bus = FlywheelBus::new();
        let mut guard = LoopGuard::new(10, 0);

        // 1 actionable + 1 non-actionable
        bus.emit(FlywheelEvent::broadcast(
            FlywheelTier::Live,
            EventKind::CycleComplete { iteration: 1 },
        ));
        bus.emit(FlywheelEvent::broadcast(
            FlywheelTier::Live,
            EventKind::NoveltyDetected {
                source: "test".into(),
                novelty: 0.8,
                summary: "interesting finding".into(),
            },
        ));

        let actions = flywheel_react(&bus, FlywheelTier::Staging, &mut guard);
        assert_eq!(actions.len(), 1); // only the novelty event
    }
}
