//! # Watcher Engine — ν (Frequency) Layer
//!
//! The always-on event loop that polls WatchSources and forwards events.
//! Uses the sentinel `select_once()` decomposition pattern.
//!
//! ## Tier: T2-C (ν + σ)

use crate::vigilance::error::{VigilError, VigilResult};
use crate::vigilance::event::WatchEvent;
use std::time::Duration;
use tokio::sync::mpsc;

/// A source of watch events.
///
/// Tier: T1 (ν) — the fundamental frequency observation primitive.
pub trait WatchSource: Send + Sync {
    /// Human-readable name of this source.
    fn name(&self) -> &str;

    /// How often this source should be polled.
    fn frequency(&self) -> Duration;

    /// Poll for new events. Returns empty vec if nothing observed.
    fn poll(&mut self) -> VigilResult<Vec<WatchEvent>>;
}

/// The Watcher engine — always-on event loop.
///
/// Tier: T2-C (ν + σ), dominant ν
pub struct Watcher {
    sources: Vec<Box<dyn WatchSource>>,
    event_tx: mpsc::Sender<WatchEvent>,
    event_counter: u64,
}

impl Watcher {
    /// Create a new watcher with the given event channel sender.
    pub fn new(event_tx: mpsc::Sender<WatchEvent>) -> Self {
        Self {
            sources: Vec::new(),
            event_tx,
            event_counter: 0,
        }
    }

    /// Register a watch source.
    pub fn add_source(&mut self, source: Box<dyn WatchSource>) {
        tracing::info!(source = source.name(), "watcher_source_registered");
        self.sources.push(source);
    }

    /// Number of registered sources.
    pub fn source_count(&self) -> usize {
        self.sources.len()
    }

    /// Poll all sources once and send events through the channel.
    ///
    /// Returns the number of events collected.
    /// This is the sentinel `select_once()` pattern — one poll cycle.
    pub async fn poll_once(&mut self) -> VigilResult<usize> {
        let mut total = 0;

        for source in &mut self.sources {
            match source.poll() {
                Ok(events) => {
                    for event in events {
                        if self.event_tx.send(event).await.is_err() {
                            return Err(VigilError::ChannelClosed(
                                "event channel closed".to_string(),
                            ));
                        }
                        total += 1;
                        self.event_counter += 1;
                    }
                }
                Err(e) => {
                    tracing::warn!(
                        source = source.name(),
                        error = %e,
                        "watcher_source_poll_error"
                    );
                    // Continue polling other sources — don't let one failure stop all
                }
            }
        }

        Ok(total)
    }

    /// Total events produced since creation.
    pub fn total_events(&self) -> u64 {
        self.event_counter
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vigilance::event::{EventKind, EventSeverity};

    /// A test source that produces N events per poll.
    struct TestSource {
        name: String,
        events_per_poll: usize,
        poll_count: u64,
    }

    impl TestSource {
        fn new(name: &str, events_per_poll: usize) -> Self {
            Self {
                name: name.to_string(),
                events_per_poll,
                poll_count: 0,
            }
        }
    }

    impl WatchSource for TestSource {
        fn name(&self) -> &str {
            &self.name
        }

        fn frequency(&self) -> Duration {
            Duration::from_millis(100)
        }

        fn poll(&mut self) -> VigilResult<Vec<WatchEvent>> {
            self.poll_count += 1;
            let events: Vec<WatchEvent> = (0..self.events_per_poll)
                .map(|i| {
                    WatchEvent::new(
                        self.poll_count * 100 + i as u64,
                        &self.name,
                        EventKind::Timer,
                        EventSeverity::Info,
                        serde_json::json!({"poll": self.poll_count, "index": i}),
                    )
                })
                .collect();
            Ok(events)
        }
    }

    /// A source that always fails.
    struct FailingSource;

    impl WatchSource for FailingSource {
        fn name(&self) -> &str {
            "failing"
        }

        fn frequency(&self) -> Duration {
            Duration::from_secs(1)
        }

        fn poll(&mut self) -> VigilResult<Vec<WatchEvent>> {
            Err(VigilError::Watcher {
                source_name: "failing".to_string(),
                message: "always fails".to_string(),
            })
        }
    }

    #[tokio::test]
    async fn watcher_polls_sources() {
        let (tx, mut rx) = mpsc::channel(100);
        let mut watcher = Watcher::new(tx);
        watcher.add_source(Box::new(TestSource::new("test-a", 2)));

        let count = watcher.poll_once().await;
        assert!(count.is_ok());
        assert_eq!(count.unwrap_or(0), 2);

        let evt = rx.try_recv();
        assert!(evt.is_ok());
    }

    #[tokio::test]
    async fn watcher_multiple_sources() {
        let (tx, mut rx) = mpsc::channel(100);
        let mut watcher = Watcher::new(tx);
        watcher.add_source(Box::new(TestSource::new("alpha", 1)));
        watcher.add_source(Box::new(TestSource::new("beta", 3)));

        let count = watcher.poll_once().await;
        assert_eq!(count.unwrap_or(0), 4);
        assert_eq!(watcher.source_count(), 2);

        // Drain all events
        let mut received = 0;
        while rx.try_recv().is_ok() {
            received += 1;
        }
        assert_eq!(received, 4);
    }

    #[tokio::test]
    async fn watcher_handles_failing_source() {
        let (tx, _rx) = mpsc::channel(100);
        let mut watcher = Watcher::new(tx);
        watcher.add_source(Box::new(FailingSource));
        watcher.add_source(Box::new(TestSource::new("healthy", 1)));

        // Should still succeed — failing source is skipped
        let count = watcher.poll_once().await;
        assert_eq!(count.unwrap_or(0), 1);
    }

    #[tokio::test]
    async fn watcher_tracks_total_events() {
        let (tx, _rx) = mpsc::channel(100);
        let mut watcher = Watcher::new(tx);
        watcher.add_source(Box::new(TestSource::new("counter", 3)));

        let _ = watcher.poll_once().await;
        assert_eq!(watcher.total_events(), 3);

        let _ = watcher.poll_once().await;
        assert_eq!(watcher.total_events(), 6);
    }
}
