//! # Timer Source — ν (Frequency)
//!
//! Produces a WatchEvent at a configurable interval.
//! Useful for periodic health checks, heartbeats, and scheduled scans.

use crate::vigilance::error::VigilResult;
use crate::vigilance::event::{EventKind, EventSeverity, WatchEvent};
use crate::vigilance::watcher::WatchSource;
use std::time::Duration;

/// A WatchSource that produces events at a fixed interval.
///
/// Tier: T2-P (ν + N)
pub struct TimerSource {
    name: String,
    interval: Duration,
    tick_count: u64,
}

impl TimerSource {
    /// Create a new timer source.
    pub fn new(name: impl Into<String>, interval: Duration) -> Self {
        Self {
            name: name.into(),
            interval,
            tick_count: 0,
        }
    }
}

impl WatchSource for TimerSource {
    fn name(&self) -> &str {
        &self.name
    }

    fn frequency(&self) -> Duration {
        self.interval
    }

    fn poll(&mut self) -> VigilResult<Vec<WatchEvent>> {
        self.tick_count += 1;
        Ok(vec![WatchEvent::new(
            self.tick_count,
            &self.name,
            EventKind::Timer,
            EventSeverity::Info,
            serde_json::json!({
                "tick": self.tick_count,
                "interval_ms": self.interval.as_millis(),
            }),
        )])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timer_source_produces_events() {
        let mut source = TimerSource::new("heartbeat", Duration::from_secs(1));

        let events = source.poll();
        assert!(events.is_ok());
        let events = events.unwrap_or_default();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].kind, EventKind::Timer);
        assert_eq!(events[0].source, "heartbeat");
    }

    #[test]
    fn timer_source_increments_ticks() {
        let mut source = TimerSource::new("counter", Duration::from_millis(100));

        let _ = source.poll();
        let _ = source.poll();
        let events = source.poll().unwrap_or_default();

        // Third tick should have tick_count=3
        let tick: u64 = events[0]
            .payload
            .get("tick")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        assert_eq!(tick, 3);
    }
}
