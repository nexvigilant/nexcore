//! # FRIDAY Bridge Source — μ (Mapping) Layer
//!
//! Converts FRIDAY orchestrator events into vigilance WatchEvents,
//! bridging the two systems via ChannelSource pattern.
//!
//! ## Mapping
//! - FRIDAY `Priority` → vigilance `EventSeverity`
//! - FRIDAY `event_type` → vigilance `EventKind`
//! - FRIDAY `source` → vigilance `source`
//!
//! ## Tier: T2-C (μ + ν + →)

use crate::models::{Event as FridayEvent, Urgency as FridayPriority};
use crate::vigilance::event::{EventId, EventKind, EventSeverity, WatchEvent};

/// Map FRIDAY Priority to vigilance EventSeverity.
///
/// Tier: T1 (μ)
pub fn map_priority(priority: FridayPriority) -> EventSeverity {
    match priority {
        FridayPriority::Critical => EventSeverity::Critical,
        FridayPriority::High => EventSeverity::High,
        FridayPriority::Normal => EventSeverity::Medium,
        FridayPriority::Low => EventSeverity::Low,
    }
}

/// Map FRIDAY event_type string to vigilance EventKind.
///
/// Tier: T1 (μ)
pub fn map_event_type(event_type: &str) -> EventKind {
    match event_type {
        t if t.contains("timer") || t.contains("heartbeat") || t.contains("tick") => {
            EventKind::Timer
        }
        t if t.contains("file") || t.contains("watch") || t.contains("fs") => EventKind::FileChange,
        t if t.contains("signal") || t.contains("alert") || t.contains("alarm") => {
            EventKind::Signal
        }
        t if t.contains("channel") || t.contains("message") || t.contains("queue") => {
            EventKind::Channel
        }
        other => EventKind::Custom(other.to_string()),
    }
}

/// Convert a FRIDAY Event into a vigilance WatchEvent.
///
/// Tier: T2-C (μ + ν + →)
pub fn friday_to_watch_event(friday_event: &FridayEvent, sequence: u64) -> WatchEvent {
    WatchEvent {
        id: EventId(sequence),
        source: friday_event.source.clone(),
        kind: map_event_type(&friday_event.event_type),
        severity: map_priority(friday_event.priority),
        payload: friday_event.payload.clone(),
        timestamp: friday_event.timestamp.timestamp_millis() as u64,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Urgency;

    #[test]
    fn priority_mapping() {
        assert_eq!(map_priority(Urgency::Critical), EventSeverity::Critical);
        assert_eq!(map_priority(Urgency::High), EventSeverity::High);
        assert_eq!(map_priority(Urgency::Normal), EventSeverity::Medium);
        assert_eq!(map_priority(Urgency::Low), EventSeverity::Low);
    }

    #[test]
    fn event_type_mapping() {
        assert_eq!(map_event_type("heartbeat_tick"), EventKind::Timer);
        assert_eq!(map_event_type("file_changed"), EventKind::FileChange);
        assert_eq!(map_event_type("signal_detected"), EventKind::Signal);
        assert_eq!(map_event_type("channel_message"), EventKind::Channel);
        assert_eq!(
            map_event_type("user_spoke"),
            EventKind::Custom("user_spoke".to_string())
        );
    }

    #[test]
    fn friday_event_conversion() {
        let friday = FridayEvent {
            source: "voice".to_string(),
            event_type: "signal_alert".to_string(),
            priority: Urgency::High,
            payload: serde_json::json!({"text": "wake word detected"}),
            ..Default::default()
        };

        let watch = friday_to_watch_event(&friday, 42);
        assert_eq!(watch.id, EventId(42));
        assert_eq!(watch.source, "voice");
        assert_eq!(watch.kind, EventKind::Signal);
        assert_eq!(watch.severity, EventSeverity::High);
    }

    #[test]
    fn custom_event_type_preserved() {
        let friday = FridayEvent {
            event_type: "llm_response_received".to_string(),
            ..Default::default()
        };

        let watch = friday_to_watch_event(&friday, 0);
        match &watch.kind {
            EventKind::Custom(s) => assert_eq!(s, "llm_response_received"),
            other => panic!("Expected Custom, got {other:?}"),
        }
    }
}
