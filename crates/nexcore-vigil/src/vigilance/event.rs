//! # Watch Events — ν (Frequency) Layer
//!
//! Events flowing through the vigilance system. Every event is a discrete
//! observation from a WatchSource.
//!
//! ## Tier: T2-P (ν + σ)
//! Events are frequency-observations ordered in time.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Unique event identifier.
/// Tier: T1 (N)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EventId(pub u64);

impl fmt::Display for EventId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "evt-{}", self.0)
    }
}

/// Classification of what kind of event was observed.
/// Tier: T2-P (Σ)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EventKind {
    /// Periodic timer tick
    Timer,
    /// File system change detected
    FileChange,
    /// Signal from another subsystem (Guardian, PV, etc.)
    Signal,
    /// Message received via channel bridge
    Channel,
    /// User-defined event type
    Custom(String),
}

impl fmt::Display for EventKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Timer => write!(f, "timer"),
            Self::FileChange => write!(f, "file_change"),
            Self::Signal => write!(f, "signal"),
            Self::Channel => write!(f, "channel"),
            Self::Custom(s) => write!(f, "custom:{s}"),
        }
    }
}

/// Severity of an observed event.
/// Tier: T2-P (κ)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum EventSeverity {
    Info = 0,
    Low = 1,
    Medium = 2,
    High = 3,
    Critical = 4,
}

impl fmt::Display for EventSeverity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Info => write!(f, "info"),
            Self::Low => write!(f, "low"),
            Self::Medium => write!(f, "medium"),
            Self::High => write!(f, "high"),
            Self::Critical => write!(f, "critical"),
        }
    }
}

/// A discrete observation from a WatchSource.
///
/// Tier: T2-P (ν + σ)
/// Dominant: ν (Frequency) — events are frequency observations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatchEvent {
    /// Unique event identifier
    pub id: EventId,
    /// Name of the source that produced this event
    pub source: String,
    /// What kind of event this is
    pub kind: EventKind,
    /// Severity classification
    pub severity: EventSeverity,
    /// Arbitrary structured payload
    pub payload: serde_json::Value,
    /// Unix timestamp in milliseconds
    pub timestamp: u64,
}

impl WatchEvent {
    /// Create a new WatchEvent with the given parameters.
    pub fn new(
        id: u64,
        source: impl Into<String>,
        kind: EventKind,
        severity: EventSeverity,
        payload: serde_json::Value,
    ) -> Self {
        Self {
            id: EventId(id),
            source: source.into(),
            kind,
            severity,
            payload,
            timestamp: now_millis(),
        }
    }
}

/// Current time in milliseconds since Unix epoch.
pub(crate) fn now_millis() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn event_id_display() {
        let id = EventId(42);
        assert_eq!(format!("{id}"), "evt-42");
    }

    #[test]
    fn event_severity_ordering() {
        assert!(EventSeverity::Info < EventSeverity::Low);
        assert!(EventSeverity::Low < EventSeverity::Medium);
        assert!(EventSeverity::Medium < EventSeverity::High);
        assert!(EventSeverity::High < EventSeverity::Critical);
    }

    #[test]
    fn event_kind_display() {
        assert_eq!(format!("{}", EventKind::Timer), "timer");
        assert_eq!(format!("{}", EventKind::FileChange), "file_change");
        assert_eq!(
            format!("{}", EventKind::Custom("test".to_string())),
            "custom:test"
        );
    }

    #[test]
    fn watch_event_new() {
        let evt = WatchEvent::new(
            1,
            "test-source",
            EventKind::Timer,
            EventSeverity::Info,
            serde_json::json!({"tick": 1}),
        );
        assert_eq!(evt.id, EventId(1));
        assert_eq!(evt.source, "test-source");
        assert!(evt.timestamp > 0);
    }

    #[test]
    fn event_serialization_roundtrip() {
        let evt = WatchEvent::new(
            99,
            "roundtrip",
            EventKind::Signal,
            EventSeverity::High,
            serde_json::json!({"signal": "pv_alert"}),
        );
        let json = serde_json::to_string(&evt);
        assert!(json.is_ok());
        let parsed: Result<WatchEvent, _> = serde_json::from_str(&json.unwrap_or_default());
        assert!(parsed.is_ok());
    }
}
