//! # Event — Observable occurrences in a system
//!
//! An event is the atomic unit of observation. Events form sequences (σ),
//! and sequences reveal causal patterns (→).
//!
//! ## Primitive Grounding
//!
//! | Type | Tier | Primitives |
//! |------|------|------------|
//! | `Event` | T2-P | ∃ + ν + λ (existence at a frequency in a location) |
//! | `EventKind` | T1 | Σ (sum type — one of N categories) |
//! | `EventSequence` | T2-C | σ + ∃ + ν (ordered sequence of existences) |

use nexcore_chrono::DateTime;
use serde::{Deserialize, Serialize};

/// A categorized event kind. Events are strings so any domain can use Oracle.
pub type EventKind = String;

/// An observed event with metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    /// The event category (e.g., "tool:Read", "build:fail", "test:pass").
    pub kind: EventKind,
    /// When the event was observed.
    pub timestamp: DateTime,
    /// Optional context (e.g., file path, error message).
    pub context: Option<String>,
}

impl Event {
    /// Create a new event at the current time.
    pub fn now(kind: impl Into<String>) -> Self {
        Self {
            kind: kind.into(),
            timestamp: DateTime::now(),
            context: None,
        }
    }

    /// Create a new event with context.
    pub fn with_context(kind: impl Into<String>, context: impl Into<String>) -> Self {
        Self {
            kind: kind.into(),
            timestamp: DateTime::now(),
            context: Some(context.into()),
        }
    }

    /// Create an event at a specific time.
    pub fn at(kind: impl Into<String>, timestamp: DateTime) -> Self {
        Self {
            kind: kind.into(),
            timestamp,
            context: None,
        }
    }
}

/// A time-ordered sequence of events.
///
/// Grounding: σ(Sequence) + ∃(Existence) + ν(Frequency)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EventSequence {
    events: Vec<Event>,
}

impl EventSequence {
    /// Create an empty sequence.
    pub fn new() -> Self {
        Self { events: Vec::new() }
    }

    /// Push an event onto the sequence.
    pub fn push(&mut self, event: Event) {
        self.events.push(event);
    }

    /// Push a simple event by kind name.
    pub fn push_kind(&mut self, kind: impl Into<String>) {
        self.events.push(Event::now(kind));
    }

    /// Number of events in the sequence.
    pub fn len(&self) -> usize {
        self.events.len()
    }

    /// Whether the sequence is empty.
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    /// Get the event kinds as a slice-like iterator.
    pub fn kinds(&self) -> impl Iterator<Item = &str> {
        self.events.iter().map(|e| e.kind.as_str())
    }

    /// Get all events.
    pub fn events(&self) -> &[Event] {
        &self.events
    }

    /// Extract all unique event kinds.
    pub fn unique_kinds(&self) -> Vec<&str> {
        let mut seen = std::collections::HashSet::new();
        self.events
            .iter()
            .filter_map(|e| {
                if seen.insert(e.kind.as_str()) {
                    Some(e.kind.as_str())
                } else {
                    None
                }
            })
            .collect()
    }

    /// Extract consecutive bigrams (pairs) from the sequence.
    pub fn bigrams(&self) -> Vec<(&str, &str)> {
        self.events
            .windows(2)
            .map(|w| (w[0].kind.as_str(), w[1].kind.as_str()))
            .collect()
    }

    /// Extract consecutive trigrams from the sequence.
    pub fn trigrams(&self) -> Vec<(&str, &str, &str)> {
        self.events
            .windows(3)
            .map(|w| (w[0].kind.as_str(), w[1].kind.as_str(), w[2].kind.as_str()))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn event_creation() {
        let e = Event::now("test:pass");
        assert_eq!(e.kind, "test:pass");
        assert!(e.context.is_none());
    }

    #[test]
    fn event_with_context() {
        let e = Event::with_context("build:fail", "missing dependency");
        assert_eq!(e.kind, "build:fail");
        assert_eq!(e.context.as_deref(), Some("missing dependency"));
    }

    #[test]
    fn sequence_push_and_len() {
        let mut seq = EventSequence::new();
        assert!(seq.is_empty());
        seq.push_kind("a");
        seq.push_kind("b");
        seq.push_kind("c");
        assert_eq!(seq.len(), 3);
    }

    #[test]
    fn sequence_bigrams() {
        let mut seq = EventSequence::new();
        seq.push_kind("a");
        seq.push_kind("b");
        seq.push_kind("c");
        seq.push_kind("a");
        let bigrams = seq.bigrams();
        assert_eq!(bigrams, vec![("a", "b"), ("b", "c"), ("c", "a")]);
    }

    #[test]
    fn sequence_trigrams() {
        let mut seq = EventSequence::new();
        for k in &["x", "y", "z", "w"] {
            seq.push_kind(*k);
        }
        let trigrams = seq.trigrams();
        assert_eq!(trigrams.len(), 2);
        assert_eq!(trigrams[0], ("x", "y", "z"));
        assert_eq!(trigrams[1], ("y", "z", "w"));
    }

    #[test]
    fn sequence_unique_kinds() {
        let mut seq = EventSequence::new();
        seq.push_kind("a");
        seq.push_kind("b");
        seq.push_kind("a");
        seq.push_kind("c");
        let unique = seq.unique_kinds();
        assert_eq!(unique.len(), 3);
    }

    #[test]
    fn empty_sequence_bigrams() {
        let seq = EventSequence::new();
        assert!(seq.bigrams().is_empty());
    }

    #[test]
    fn single_event_bigrams() {
        let mut seq = EventSequence::new();
        seq.push_kind("lone");
        assert!(seq.bigrams().is_empty());
    }

    #[test]
    fn serde_roundtrip() {
        let mut seq = EventSequence::new();
        seq.push_kind("test");
        let json = serde_json::to_string(&seq).unwrap_or_default();
        assert!(json.contains("test"));
    }
}
