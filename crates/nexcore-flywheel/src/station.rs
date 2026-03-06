//! Station event bridge — maps NexVigilant Station events to FlywheelEvents.
//!
//! The station broadcasts `StationEvent` via SSE after every tool call.
//! This module provides the mapping function that converts station events
//! into `FlywheelEvent::Custom` for the bus.
//!
//! ## T1 Primitive Grounding
//!
//! | Concept | Primitive | Symbol |
//! |---------|-----------|--------|
//! | Event mapping | Mapping | μ |
//! | Cross-boundary bridge | Boundary | ∂ |
//! | Domain extraction | Location | λ |

use crate::event::{EventKind, FlywheelEvent};
use crate::node::FlywheelTier;
use serde::{Deserialize, Serialize};

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
        use crate::bridge::FlywheelBus;

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
}
