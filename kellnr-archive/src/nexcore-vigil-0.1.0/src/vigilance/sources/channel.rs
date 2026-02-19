//! # Channel Source — ν (Frequency) + σ (Sequence)
//!
//! A WatchSource backed by a tokio mpsc receiver. Bridges events from
//! other subsystems (Guardian, signal pipeline, etc.) into the watcher.

use crate::vigilance::error::VigilResult;
use crate::vigilance::event::WatchEvent;
use crate::vigilance::watcher::WatchSource;
use std::time::Duration;
use tokio::sync::mpsc;

/// A WatchSource that drains events from a tokio mpsc channel.
///
/// Tier: T2-P (ν + σ)
pub struct ChannelSource {
    name: String,
    rx: mpsc::Receiver<WatchEvent>,
    frequency: Duration,
}

impl ChannelSource {
    /// Create a new channel source.
    ///
    /// Returns the source and the sender half for injecting events.
    pub fn new(
        name: impl Into<String>,
        buffer: usize,
        frequency: Duration,
    ) -> (Self, mpsc::Sender<WatchEvent>) {
        let (tx, rx) = mpsc::channel(buffer);
        (
            Self {
                name: name.into(),
                rx,
                frequency,
            },
            tx,
        )
    }

    /// Create from an existing receiver.
    pub fn from_receiver(
        name: impl Into<String>,
        rx: mpsc::Receiver<WatchEvent>,
        frequency: Duration,
    ) -> Self {
        Self {
            name: name.into(),
            rx,
            frequency,
        }
    }
}

impl WatchSource for ChannelSource {
    fn name(&self) -> &str {
        &self.name
    }

    fn frequency(&self) -> Duration {
        self.frequency
    }

    fn poll(&mut self) -> VigilResult<Vec<WatchEvent>> {
        let mut events = Vec::new();
        // Drain all currently buffered events (non-blocking)
        while let Ok(event) = self.rx.try_recv() {
            events.push(event);
        }
        Ok(events)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vigilance::event::{EventKind, EventSeverity};

    #[tokio::test]
    async fn channel_source_drains_events() {
        let (mut source, tx) = ChannelSource::new("bridge", 100, Duration::from_millis(100));

        // Inject events
        for i in 0..3 {
            let event = WatchEvent::new(
                i,
                "external",
                EventKind::Channel,
                EventSeverity::Medium,
                serde_json::json!({"index": i}),
            );
            let _ = tx.send(event).await;
        }

        let events = source.poll().unwrap_or_default();
        assert_eq!(events.len(), 3);
    }

    #[tokio::test]
    async fn channel_source_empty_when_no_events() {
        let (mut source, _tx) = ChannelSource::new("empty", 100, Duration::from_millis(100));
        let events = source.poll().unwrap_or_default();
        assert!(events.is_empty());
    }

    #[tokio::test]
    async fn channel_source_works_after_sender_drop() {
        let (mut source, tx) = ChannelSource::new("drop-test", 100, Duration::from_millis(100));

        let event = WatchEvent::new(
            1,
            "pre-drop",
            EventKind::Channel,
            EventSeverity::Info,
            serde_json::json!({}),
        );
        let _ = tx.send(event).await;
        drop(tx);

        // Should still drain the buffered event
        let events = source.poll().unwrap_or_default();
        assert_eq!(events.len(), 1);
    }
}
