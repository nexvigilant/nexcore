//! # Temporal Primitives
//!
//! Cross-domain temporal patterns extracted from nexcore-sentinel.
//! Each primitive is grounded to T1 universals (Duration, Timestamp, Count)
//! and transfer-tested across ≥5 domains.
//!
//! ## Contents
//!
//! | Type | Tier | Confidence | Domains |
//! |------|------|------------|---------|
//! | [`SlidingWindow`] | T2-P | 0.91 | Rate limiting, PV signals, networking, finance |
//! | [`Ttl`] | T2-P | 0.91 | Caching, DNS, sessions, bans, leases |
//! | [`ExpiryRecord`] | T2-C | 0.88 | Cache entries, tokens, certificates, subscriptions |
//! | [`ThresholdCounter`] | T2-C | 0.90 | Circuit breakers, rate limiters, intrusion detection |

use std::fmt;
use std::hash::Hash;
use std::time::Duration;

use nexcore_chrono::DateTime;
use serde::{Deserialize, Serialize};

// ============================================================================
// T2-P: SlidingWindow
// ============================================================================

/// Tier: T2-P — A bounded temporal window that moves with "now".
///
/// Grounds to: `Duration` (T1) + recency semantics.
///
/// ## Primitive Test
///
/// - **No domain-internal deps**: uses only time, duration, event, discard
/// - **External grounding**: moving average, TCP sliding window, rolling volatility
/// - **Not a synonym**: distinct from "duration" (static) and "interval" (fixed)
///
/// ## Transfer Confidence: 0.91
///
/// | Dimension | Score | Notes |
/// |-----------|-------|-------|
/// | Structural | 0.95 | Duration newtype, universally representable |
/// | Functional | 0.90 | Prune-to-window operation identical across domains |
/// | Contextual | 0.85 | Every event-counting domain uses this |
///
/// ## Cross-Domain Applications
///
/// - **Rate limiting**: N requests per window
/// - **Signal detection (PV)**: adverse events within observation window
/// - **Financial analytics**: rolling N-day volatility
/// - **Network monitoring**: packet counts in window
/// - **Circuit breakers**: failure rate in window
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SlidingWindow(Duration);

impl SlidingWindow {
    /// Create a new sliding window with the given duration.
    #[inline]
    #[must_use]
    pub const fn new(duration: Duration) -> Self {
        Self(duration)
    }

    /// Create from seconds.
    #[inline]
    #[must_use]
    pub const fn from_secs(secs: u64) -> Self {
        Self(Duration::from_secs(secs))
    }

    /// Get the window duration.
    #[inline]
    #[must_use]
    pub const fn duration(&self) -> Duration {
        self.0
    }

    /// Compute the cutoff timestamp: anything before this is outside the window.
    #[must_use]
    pub fn cutoff(&self, now: DateTime) -> DateTime {
        now - nexcore_chrono::Duration::from_std(self.0)
    }

    /// Check if a timestamp is within the window relative to `now`.
    #[must_use]
    pub fn contains(&self, timestamp: DateTime, now: DateTime) -> bool {
        timestamp >= self.cutoff(now)
    }

    /// Prune a sorted vec of timestamps, retaining only those within the window.
    pub fn prune(&self, timestamps: &mut Vec<DateTime>, now: DateTime) {
        let cutoff = self.cutoff(now);
        timestamps.retain(|&t| t >= cutoff);
    }
}

impl fmt::Display for SlidingWindow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SlidingWindow({}s)", self.0.as_secs())
    }
}

// ============================================================================
// T2-P: TTL (Time-To-Live)
// ============================================================================

/// Tier: T2-P — Time-To-Live: duration until automatic expiry.
///
/// Grounds to: `Duration` (T1) + expiry semantics.
///
/// ## Primitive Test
///
/// - **No domain-internal deps**: uses only duration, expiry
/// - **External grounding**: cache TTL, DNS TTL, session timeout, lease duration
/// - **Not a synonym**: carries "expiry" semantics beyond bare `Duration`
///
/// ## Transfer Confidence: 0.91
///
/// | Dimension | Score | Notes |
/// |-----------|-------|-------|
/// | Structural | 0.95 | Duration newtype, universally representable |
/// | Functional | 0.90 | Expiry check: `now >= start + ttl` |
/// | Contextual | 0.85 | TTL is universally understood |
///
/// ## Cross-Domain Applications
///
/// - **Caching**: cache entry time-to-live
/// - **DNS**: record TTL
/// - **Security**: ban/lockout duration
/// - **Sessions**: session timeout
/// - **Distributed systems**: lease duration
/// - **Pharmacy**: drug shelf life / expiry
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Ttl(Duration);

impl Ttl {
    /// Create a new TTL.
    #[inline]
    #[must_use]
    pub const fn new(duration: Duration) -> Self {
        Self(duration)
    }

    /// Create from seconds.
    #[inline]
    #[must_use]
    pub const fn from_secs(secs: u64) -> Self {
        Self(Duration::from_secs(secs))
    }

    /// Get the inner duration.
    #[inline]
    #[must_use]
    pub const fn duration(&self) -> Duration {
        self.0
    }

    /// Compute the expiry timestamp given a start time.
    #[must_use]
    pub fn expires_at(&self, start: DateTime) -> DateTime {
        start + nexcore_chrono::Duration::from_std(self.0)
    }

    /// Check if something started at `start` has expired by `now`.
    #[must_use]
    pub fn is_expired(&self, start: DateTime, now: DateTime) -> bool {
        now >= self.expires_at(start)
    }

    /// Zero TTL (immediately expired).
    pub const ZERO: Self = Self(Duration::ZERO);

    /// 1-hour TTL.
    pub const ONE_HOUR: Self = Self(Duration::from_secs(3600));

    /// 24-hour TTL.
    pub const ONE_DAY: Self = Self(Duration::from_secs(86400));
}

impl fmt::Display for Ttl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "TTL({}s)", self.0.as_secs())
    }
}

// ============================================================================
// T2-C: ExpiryRecord<T>
// ============================================================================

/// Tier: T2-C — An entity bound to a validity window `[created_at, expires_at]`.
///
/// Components: `T` (entity) + `DateTime` (start) + `DateTime` (expiry).
///
/// ## Primitive Test
///
/// - **No domain-internal deps**: entity, time range, expiry predicate
/// - **External grounding**: cache entries, certificates, session tokens, bans
/// - **Not a synonym for interval**: Interval has no entity binding
///
/// ## Transfer Confidence: 0.88
///
/// | Dimension | Score | Notes |
/// |-----------|-------|-------|
/// | Structural | 0.90 | Generic over entity type T |
/// | Functional | 0.90 | `is_expired(now)` check is universal |
/// | Contextual | 0.80 | Every system with temporal validity uses this |
///
/// ## Cross-Domain Applications
///
/// - **Cache management**: entry with expiry
/// - **Certificate management**: validity windows
/// - **Session management**: token lifetime
/// - **Security**: ban records
/// - **Subscription billing**: active period tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpiryRecord<T> {
    /// The entity this record is about.
    pub entity: T,
    /// When the record was created / enforcement began.
    pub created_at: DateTime,
    /// When the record expires / enforcement ends.
    pub expires_at: DateTime,
}

impl<T> ExpiryRecord<T> {
    /// Create a new expiry record from entity, start time, and TTL.
    #[must_use]
    pub fn new(entity: T, created_at: DateTime, ttl: Ttl) -> Self {
        Self {
            entity,
            created_at,
            expires_at: ttl.expires_at(created_at),
        }
    }

    /// Create with an explicit expiry timestamp.
    #[must_use]
    pub fn with_expiry(entity: T, created_at: DateTime, expires_at: DateTime) -> Self {
        Self {
            entity,
            created_at,
            expires_at,
        }
    }

    /// Check if the record has expired relative to `now`.
    #[must_use]
    pub fn is_expired(&self, now: DateTime) -> bool {
        now >= self.expires_at
    }

    /// Check if the record is still active at `now`.
    #[must_use]
    pub fn is_active(&self, now: DateTime) -> bool {
        !self.is_expired(now)
    }

    /// Remaining duration until expiry, or zero if already expired.
    #[must_use]
    pub fn remaining(&self, now: DateTime) -> Duration {
        if self.is_expired(now) {
            Duration::ZERO
        } else {
            (self.expires_at - now)
                .to_std()
                .unwrap_or(std::time::Duration::ZERO)
        }
    }

    /// Map the entity to a different type while preserving temporal state.
    #[must_use]
    pub fn map<U, F: FnOnce(T) -> U>(self, f: F) -> ExpiryRecord<U> {
        ExpiryRecord {
            entity: f(self.entity),
            created_at: self.created_at,
            expires_at: self.expires_at,
        }
    }
}

impl<T: PartialEq> PartialEq for ExpiryRecord<T> {
    fn eq(&self, other: &Self) -> bool {
        self.entity == other.entity
            && self.created_at == other.created_at
            && self.expires_at == other.expires_at
    }
}

impl<T: Eq> Eq for ExpiryRecord<T> {}

impl<T: fmt::Display> fmt::Display for ExpiryRecord<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ExpiryRecord({}, expires: {})",
            self.entity,
            self.expires_at
                .format("%Y-%m-%d %H:%M:%S")
                .unwrap_or_default()
        )
    }
}

// ============================================================================
// T2-C: ThresholdCounter
// ============================================================================

/// Tier: T2-C — Accumulates events in a [`SlidingWindow`], triggers at threshold.
///
/// Components: [`SlidingWindow`] (T2-P) + `u64` count (T1) + threshold (T1).
///
/// ## Primitive Test
///
/// - **No domain-internal deps**: counter, event, window, threshold, trigger
/// - **External grounding**: circuit breakers, rate limiters, intrusion detection
/// - **Not a synonym**: compound of 3 primitives with trigger semantics
///
/// ## Transfer Confidence: 0.90
///
/// | Dimension | Score | Notes |
/// |-----------|-------|-------|
/// | Structural | 0.90 | Composition of existing primitives |
/// | Functional | 0.95 | count-in-window vs threshold is universal |
/// | Contextual | 0.80 | Naming varies: circuit breaker, rate limiter, etc. |
///
/// ## Cross-Domain Applications
///
/// - **Circuit breakers**: N failures in window → open circuit
/// - **Rate limiters**: N requests in window → reject
/// - **Intrusion detection**: N auth failures → ban
/// - **PV signal detection**: N adverse events → signal
/// - **Manufacturing**: N defects in window → halt line
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThresholdCounter {
    /// The sliding window for event retention.
    window: SlidingWindow,
    /// Maximum count before triggering.
    max_count: u64,
    /// Recorded event timestamps.
    events: Vec<DateTime>,
}

impl ThresholdCounter {
    /// Create a new threshold counter.
    #[must_use]
    pub fn new(window: SlidingWindow, max_count: u64) -> Self {
        Self {
            window,
            max_count,
            events: Vec::new(),
        }
    }

    /// Record an event at the given timestamp. Returns `true` if the
    /// threshold was reached (i.e., this event caused a trigger).
    pub fn record(&mut self, when: DateTime) -> bool {
        self.events.push(when);
        self.window.prune(&mut self.events, when);
        self.events.len() as u64 >= self.max_count
    }

    /// Current count of events within the window.
    #[must_use]
    pub fn count(&self, now: DateTime) -> u64 {
        self.events
            .iter()
            .filter(|&&t| self.window.contains(t, now))
            .count() as u64
    }

    /// Check if the threshold is currently exceeded.
    #[must_use]
    pub fn is_triggered(&self, now: DateTime) -> bool {
        self.count(now) >= self.max_count
    }

    /// Get the threshold value.
    #[must_use]
    pub fn threshold(&self) -> u64 {
        self.max_count
    }

    /// Get the window.
    #[must_use]
    pub fn window(&self) -> SlidingWindow {
        self.window
    }

    /// Clear all recorded events.
    pub fn reset(&mut self) {
        self.events.clear();
    }

    /// Prune expired events from memory.
    pub fn prune(&mut self, now: DateTime) {
        self.window.prune(&mut self.events, now);
    }
}

impl fmt::Display for ThresholdCounter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ThresholdCounter({}/{}, {})",
            self.events.len(),
            self.max_count,
            self.window
        )
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use nexcore_chrono::Duration as ChronoDuration;

    fn now() -> DateTime {
        DateTime::now()
    }

    // ── SlidingWindow tests ──────────────────────────────────────

    #[test]
    fn sliding_window_from_secs() {
        let w = SlidingWindow::from_secs(600);
        assert_eq!(w.duration(), Duration::from_secs(600));
    }

    #[test]
    fn sliding_window_contains_recent() {
        let w = SlidingWindow::from_secs(60);
        let n = now();
        let recent = n - ChronoDuration::seconds(30);
        assert!(w.contains(recent, n));
    }

    #[test]
    fn sliding_window_excludes_old() {
        let w = SlidingWindow::from_secs(60);
        let n = now();
        let old = n - ChronoDuration::seconds(120);
        assert!(!w.contains(old, n));
    }

    #[test]
    fn sliding_window_prune() {
        let w = SlidingWindow::from_secs(60);
        let n = now();
        let mut timestamps = vec![
            n - ChronoDuration::seconds(120),
            n - ChronoDuration::seconds(30),
            n,
        ];
        w.prune(&mut timestamps, n);
        assert_eq!(timestamps.len(), 2); // 120s-old entry pruned
    }

    #[test]
    fn sliding_window_display() {
        let w = SlidingWindow::from_secs(300);
        assert_eq!(format!("{w}"), "SlidingWindow(300s)");
    }

    // ── TTL tests ────────────────────────────────────────────────

    #[test]
    fn ttl_from_secs() {
        let ttl = Ttl::from_secs(3600);
        assert_eq!(ttl.duration(), Duration::from_secs(3600));
    }

    #[test]
    fn ttl_not_expired() {
        let ttl = Ttl::from_secs(3600);
        let start = now();
        assert!(!ttl.is_expired(start, now()));
    }

    #[test]
    fn ttl_expired() {
        let ttl = Ttl::from_secs(60);
        let start = now() - ChronoDuration::seconds(120);
        assert!(ttl.is_expired(start, now()));
    }

    #[test]
    fn ttl_expires_at() {
        let ttl = Ttl::from_secs(100);
        let start = now();
        let expiry = ttl.expires_at(start);
        assert!(expiry > start);
    }

    #[test]
    fn ttl_constants() {
        assert_eq!(Ttl::ZERO.duration(), Duration::ZERO);
        assert_eq!(Ttl::ONE_HOUR.duration(), Duration::from_secs(3600));
        assert_eq!(Ttl::ONE_DAY.duration(), Duration::from_secs(86400));
    }

    #[test]
    fn ttl_display() {
        assert_eq!(format!("{}", Ttl::ONE_HOUR), "TTL(3600s)");
    }

    // ── ExpiryRecord tests ───────────────────────────────────────

    #[test]
    fn expiry_record_active() {
        let record = ExpiryRecord::new("10.0.0.1", now(), Ttl::ONE_HOUR);
        assert!(record.is_active(now()));
        assert!(!record.is_expired(now()));
    }

    #[test]
    fn expiry_record_expired() {
        let start = now() - ChronoDuration::seconds(7200);
        let record = ExpiryRecord::new("10.0.0.1", start, Ttl::ONE_HOUR);
        assert!(record.is_expired(now()));
        assert!(!record.is_active(now()));
    }

    #[test]
    fn expiry_record_remaining() {
        let record = ExpiryRecord::new("test", now(), Ttl::ONE_HOUR);
        let rem = record.remaining(now());
        // Should be close to 3600s (within a few seconds of test execution)
        assert!(rem.as_secs() > 3590);
    }

    #[test]
    fn expiry_record_remaining_when_expired() {
        let start = now() - ChronoDuration::seconds(7200);
        let record = ExpiryRecord::new("test", start, Ttl::ONE_HOUR);
        assert_eq!(record.remaining(now()), Duration::ZERO);
    }

    #[test]
    fn expiry_record_map() {
        let record = ExpiryRecord::new(42_u32, now(), Ttl::ONE_HOUR);
        let mapped = record.map(|n| n.to_string());
        assert_eq!(mapped.entity, "42");
    }

    #[test]
    fn expiry_record_equality() {
        let start = now();
        let a = ExpiryRecord::new("x", start, Ttl::ONE_HOUR);
        let b = ExpiryRecord::new("x", start, Ttl::ONE_HOUR);
        assert_eq!(a, b);
    }

    #[test]
    fn expiry_record_with_explicit_expiry() {
        let start = now();
        let end = start + ChronoDuration::seconds(999);
        let record = ExpiryRecord::with_expiry("entity", start, end);
        assert_eq!(record.expires_at, end);
    }

    // ── ThresholdCounter tests ───────────────────────────────────

    #[test]
    fn threshold_counter_below_threshold() {
        let window = SlidingWindow::from_secs(60);
        let mut counter = ThresholdCounter::new(window, 3);
        let n = now();
        assert!(!counter.record(n));
        assert!(!counter.record(n));
        assert!(!counter.is_triggered(n));
    }

    #[test]
    fn threshold_counter_triggers_at_threshold() {
        let window = SlidingWindow::from_secs(60);
        let mut counter = ThresholdCounter::new(window, 3);
        let n = now();
        assert!(!counter.record(n));
        assert!(!counter.record(n + ChronoDuration::seconds(1)));
        assert!(counter.record(n + ChronoDuration::seconds(2))); // 3rd event triggers
    }

    #[test]
    fn threshold_counter_old_events_pruned() {
        let window = SlidingWindow::from_secs(10);
        let mut counter = ThresholdCounter::new(window, 3);
        let n = now();

        // Record 2 events in the past (outside window)
        let old = n - ChronoDuration::seconds(30);
        counter.record(old);
        counter.record(old + ChronoDuration::seconds(1));

        // Record 1 event now — should NOT trigger (old events pruned)
        assert!(!counter.record(n));
        assert_eq!(counter.count(n), 1);
    }

    #[test]
    fn threshold_counter_reset() {
        let window = SlidingWindow::from_secs(60);
        let mut counter = ThresholdCounter::new(window, 3);
        counter.record(now());
        counter.record(now());
        counter.reset();
        assert_eq!(counter.count(now()), 0);
    }

    #[test]
    fn threshold_counter_accessors() {
        let window = SlidingWindow::from_secs(300);
        let counter = ThresholdCounter::new(window, 5);
        assert_eq!(counter.threshold(), 5);
        assert_eq!(counter.window(), SlidingWindow::from_secs(300));
    }

    #[test]
    fn threshold_counter_display() {
        let counter = ThresholdCounter::new(SlidingWindow::from_secs(60), 3);
        assert_eq!(
            format!("{counter}"),
            "ThresholdCounter(0/3, SlidingWindow(60s))"
        );
    }
}
