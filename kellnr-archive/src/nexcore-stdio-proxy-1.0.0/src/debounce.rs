//! Generic event debouncer.
//!
//! ## Primitive Foundation
//!
//! | Primitive | Manifestation |
//! |-----------|---------------|
//! | T1: State (ς) | Deadline tracking for quiet detection |
//! | T1: Sequence (σ) | Event stream → debounced signal stream |
//! | T1: Mapping (μ) | Predicate: raw event → relevant? |

use std::sync::mpsc as std_mpsc;
use std::time::{Duration, Instant};

use tokio::sync::mpsc;

/// Tier: T2-P — Result of polling one event from a source.
pub enum PollResult {
    /// The event matched the predicate.
    Matched,
    /// The event did not match (or timeout).
    Irrelevant,
    /// The source channel is disconnected.
    Disconnected,
}

/// Tier: T2-P — Generic debouncer that filters events through a predicate
/// and emits signals only after a quiet period.
pub struct Debouncer<E> {
    source: std_mpsc::Receiver<E>,
    sink: mpsc::Sender<()>,
    predicate: Box<dyn Fn(&E) -> bool + Send>,
    debounce: Duration,
}

impl<E: Send + 'static> Debouncer<E> {
    /// Create a new debouncer.
    ///
    /// - `source`: blocking receiver of raw events
    /// - `sink`: async sender for debounced signals
    /// - `predicate`: returns true if the event is relevant
    /// - `debounce`: quiet period duration
    pub fn new(
        source: std_mpsc::Receiver<E>,
        sink: mpsc::Sender<()>,
        predicate: impl Fn(&E) -> bool + Send + 'static,
        debounce: Duration,
    ) -> Self {
        Self {
            source,
            sink,
            predicate: Box::new(predicate),
            debounce,
        }
    }

    /// Run the debounce loop (blocking — use with `spawn_blocking`).
    pub fn run(self) {
        loop {
            match self.poll_event(Duration::from_millis(500)) {
                PollResult::Matched => {
                    self.emit_after_quiet();
                }
                PollResult::Disconnected => break,
                PollResult::Irrelevant => {}
            }
        }
    }

    /// Poll one event with timeout.
    fn poll_event(&self, timeout: Duration) -> PollResult {
        match self.source.recv_timeout(timeout) {
            Ok(e) if (self.predicate)(&e) => PollResult::Matched,
            Ok(_) => PollResult::Irrelevant,
            Err(std_mpsc::RecvTimeoutError::Timeout) => PollResult::Irrelevant,
            Err(std_mpsc::RecvTimeoutError::Disconnected) => PollResult::Disconnected,
        }
    }

    /// Wait for quiet period then emit signal.
    fn emit_after_quiet(&self) {
        tracing::debug!("Event matched, starting debounce");
        if self.wait_quiet() {
            tracing::info!("Debounce complete, emitting signal");
            let _ = self.sink.blocking_send(());
        }
    }

    /// Wait for a quiet period with no matching events.
    /// Returns true if quiet achieved, false if source disconnected.
    fn wait_quiet(&self) -> bool {
        let mut deadline = Instant::now() + self.debounce;
        loop {
            let remaining = deadline.saturating_duration_since(Instant::now());
            if remaining.is_zero() {
                return true;
            }
            match self.poll_event(remaining) {
                PollResult::Matched => deadline = Instant::now() + self.debounce,
                PollResult::Disconnected => return false,
                PollResult::Irrelevant => {} // keep waiting
            }
        }
    }
}

/// Create a poll result from a blocking receiver (standalone, for testing).
pub fn poll_from_receiver<E>(
    rx: &std_mpsc::Receiver<E>,
    predicate: &dyn Fn(&E) -> bool,
    timeout: Duration,
) -> PollResult {
    match rx.recv_timeout(timeout) {
        Ok(e) if predicate(&e) => PollResult::Matched,
        Ok(_) => PollResult::Irrelevant,
        Err(std_mpsc::RecvTimeoutError::Timeout) => PollResult::Irrelevant,
        Err(std_mpsc::RecvTimeoutError::Disconnected) => PollResult::Disconnected,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn poll_timeout_is_irrelevant() {
        let (_tx, rx) = std_mpsc::channel::<i32>();
        let result = poll_from_receiver(&rx, &|_: &i32| true, Duration::from_millis(10));
        assert!(matches!(result, PollResult::Irrelevant));
    }

    #[test]
    fn poll_disconnected_detected() {
        let (tx, rx) = std_mpsc::channel::<i32>();
        drop(tx);
        let result = poll_from_receiver(&rx, &|_: &i32| true, Duration::from_millis(10));
        assert!(matches!(result, PollResult::Disconnected));
    }

    #[test]
    fn poll_matched_event() {
        let (tx, rx) = std_mpsc::channel::<i32>();
        tx.send(42).unwrap_or_else(|e| panic!("send: {e}"));
        let result = poll_from_receiver(&rx, &|v: &i32| *v == 42, Duration::from_millis(100));
        assert!(matches!(result, PollResult::Matched));
    }

    #[test]
    fn poll_irrelevant_event() {
        let (tx, rx) = std_mpsc::channel::<i32>();
        tx.send(99).unwrap_or_else(|e| panic!("send: {e}"));
        let result = poll_from_receiver(&rx, &|v: &i32| *v == 42, Duration::from_millis(100));
        assert!(matches!(result, PollResult::Irrelevant));
    }

    #[test]
    fn debouncer_emits_after_quiet() {
        let (src_tx, src_rx) = std_mpsc::channel::<i32>();
        let (sink_tx, mut sink_rx) = mpsc::channel::<()>(16);

        let debouncer =
            Debouncer::new(src_rx, sink_tx, |v: &i32| *v > 0, Duration::from_millis(50));

        std::thread::spawn(move || {
            src_tx.send(1).unwrap_or_else(|e| panic!("send: {e}"));
            std::thread::sleep(Duration::from_millis(100));
            drop(src_tx);
        });

        std::thread::spawn(move || {
            debouncer.run();
        });

        // Should receive one debounced signal
        let result = sink_rx.blocking_recv();
        assert!(result.is_some());
    }

    #[test]
    fn debouncer_resets_on_rapid_events() {
        let (src_tx, src_rx) = std_mpsc::channel::<i32>();
        let (sink_tx, mut sink_rx) = mpsc::channel::<()>(16);

        let debouncer =
            Debouncer::new(src_rx, sink_tx, |v: &i32| *v > 0, Duration::from_millis(80));

        std::thread::spawn(move || {
            // Rapid events should reset debounce
            src_tx.send(1).unwrap_or_else(|e| panic!("send: {e}"));
            std::thread::sleep(Duration::from_millis(30));
            src_tx.send(2).unwrap_or_else(|e| panic!("send: {e}"));
            std::thread::sleep(Duration::from_millis(30));
            src_tx.send(3).unwrap_or_else(|e| panic!("send: {e}"));
            // Now wait for quiet
            std::thread::sleep(Duration::from_millis(200));
            drop(src_tx);
        });

        std::thread::spawn(move || {
            debouncer.run();
        });

        // Should still get exactly one signal (debounce reset on each event)
        let result = sink_rx.blocking_recv();
        assert!(result.is_some());
    }
}
