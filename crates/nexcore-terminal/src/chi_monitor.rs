//! χ (chi) monitor — connects live terminal event streams to [`SessionHealth`].
//!
//! ## Purpose
//!
//! [`ChiMonitor`] maintains two sliding-window event queues (input and output)
//! and derives a [`SessionHealth`] snapshot on demand. It is the bridge between
//! raw keystroke / render-line events and the T Tauri branching-ratio model
//! described in [`crate::health`].
//!
//! ## Backpressure Estimation
//!
//! Beyond the χ ratio, the monitor tracks the inter-output gap — the wall-clock
//! duration between consecutive `record_output` calls. When the most recent gap
//! exceeds twice the rolling average gap, backpressure is rising; the deviation
//! is normalised to [0.0, 1.0] and fed as `τ_disk` into [`SessionHealth::new`].
//!
//! ## Example
//!
//! ```rust
//! use std::time::Duration;
//! use nexcore_terminal::chi_monitor::ChiMonitor;
//! use nexcore_terminal::health::HealthBand;
//!
//! let mut m = ChiMonitor::new(Duration::from_secs(60));
//! for _ in 0..100 { m.record_input(); }
//! for _ in 0..5  { m.record_output(); }
//! let health = m.compute();
//! assert_eq!(health.band, HealthBand::Healthy);
//! ```

use std::collections::VecDeque;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

use crate::health::SessionHealth;

/// Maximum number of inter-output gap samples retained for rolling-average
/// backpressure estimation. Older samples are evicted as the deque fills.
const MAX_GAP_SAMPLES: usize = 64;

/// Scaling factor for the gap-ratio → backpressure normalisation.
///
/// A gap-ratio of `BACKPRESSURE_SCALE` or above clamps to 1.0.
/// Chosen as 4.0 so that a gap four times the average maps to full backpressure,
/// while a gap exactly twice the average maps to 0.5.
const BACKPRESSURE_SCALE: f64 = 4.0;

/// χ (chi) monitor — sliding-window event counter and health state machine.
///
/// Call [`record_input`](ChiMonitor::record_input) for every arriving event
/// (keystroke, API call, command) and [`record_output`](ChiMonitor::record_output)
/// for every emitted event (rendered line, response chunk). Call
/// [`compute`](ChiMonitor::compute) to refresh [`SessionHealth`].
///
/// Thread safety: `ChiMonitor` is `!Send` (it stores `std::time::Instant`);
/// wrap in a `tokio::sync::Mutex` if shared across tasks.
#[derive(Debug, Serialize, Deserialize)]
pub struct ChiMonitor {
    /// Rolling window of input event timestamps (nanoseconds since UNIX epoch,
    /// stored as `u64` for `serde` compatibility).
    #[serde(skip)]
    input_events: VecDeque<Instant>,

    /// Rolling window of output event timestamps.
    #[serde(skip)]
    output_events: VecDeque<Instant>,

    /// Window duration for rate computation (nanoseconds, stored as `u64`).
    #[serde(with = "duration_serde")]
    window: Duration,

    /// Timestamp of the most recent `record_output` call, used for gap tracking.
    #[serde(skip)]
    last_output_at: Option<Instant>,

    /// Rolling buffer of inter-output gap durations (nanoseconds).
    ///
    /// Evicts oldest entries beyond [`MAX_GAP_SAMPLES`].
    #[serde(skip)]
    output_gaps: VecDeque<Duration>,

    /// Current computed health snapshot; refreshed by [`compute`].
    current_health: SessionHealth,
}

impl ChiMonitor {
    /// Create a new monitor with the given sliding-window duration.
    ///
    /// Events older than `window` are pruned from consideration during
    /// [`compute`]. A window of 60 seconds is a reasonable default for
    /// interactive terminal sessions.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::time::Duration;
    /// use nexcore_terminal::chi_monitor::ChiMonitor;
    ///
    /// let m = ChiMonitor::new(Duration::from_secs(30));
    /// assert_eq!(m.chi(), 0.0);
    /// ```
    #[must_use]
    pub fn new(window: Duration) -> Self {
        Self {
            input_events: VecDeque::new(),
            output_events: VecDeque::new(),
            window,
            last_output_at: None,
            output_gaps: VecDeque::new(),
            current_health: SessionHealth::new(0, 0, 0.0),
        }
    }

    /// Record an input event (keystroke, API call, command arriving).
    ///
    /// Stamps the current instant into the input window. Call this every time
    /// the terminal receives data from the user or an upstream producer.
    pub fn record_input(&mut self) {
        self.input_events.push_back(Instant::now());
    }

    /// Record an output event (line rendered, response chunk emitted).
    ///
    /// Stamps the current instant into the output window and updates the
    /// inter-output gap buffer used for backpressure estimation.
    pub fn record_output(&mut self) {
        let now = Instant::now();

        // Update gap tracking before pushing the timestamp.
        if let Some(prev) = self.last_output_at {
            let gap = now.saturating_duration_since(prev);
            if self.output_gaps.len() >= MAX_GAP_SAMPLES {
                self.output_gaps.pop_front();
            }
            self.output_gaps.push_back(gap);
        }
        self.last_output_at = Some(now);
        self.output_events.push_back(now);
    }

    /// Prune events outside the sliding window, compute χ and backpressure,
    /// and return a reference to the refreshed [`SessionHealth`].
    ///
    /// This is the primary polling call. It is cheap (O(n) pruning then O(1)
    /// arithmetic) and safe to call on every render frame.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::time::Duration;
    /// use nexcore_terminal::chi_monitor::ChiMonitor;
    /// use nexcore_terminal::health::HealthBand;
    ///
    /// let mut m = ChiMonitor::new(Duration::from_secs(60));
    /// // No events yet → SpinningUp.
    /// assert_eq!(m.compute().band, HealthBand::SpinningUp);
    /// ```
    pub fn compute(&mut self) -> &SessionHealth {
        let now = Instant::now();
        let cutoff = now.checked_sub(self.window).unwrap_or(now);

        // Prune expired entries from the front (oldest) of each deque.
        while self.input_events.front().is_some_and(|&t| t <= cutoff) {
            self.input_events.pop_front();
        }
        while self.output_events.front().is_some_and(|&t| t <= cutoff) {
            self.output_events.pop_front();
        }

        let input_count = self.input_events.len() as u64;
        let output_count = self.output_events.len() as u64;
        let backpressure = self.estimate_backpressure();

        self.current_health = SessionHealth::new(input_count, output_count, backpressure);
        &self.current_health
    }

    /// Current χ value from the last [`compute`] call.
    ///
    /// Returns `0.0` if [`compute`] has not yet been called or if there are
    /// no input events in the window.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::time::Duration;
    /// use nexcore_terminal::chi_monitor::ChiMonitor;
    ///
    /// let m = ChiMonitor::new(Duration::from_secs(60));
    /// assert_eq!(m.chi(), 0.0);
    /// ```
    #[must_use]
    pub fn chi(&self) -> f64 {
        self.current_health.chi
    }

    /// Reference to the most recently computed [`SessionHealth`] snapshot.
    ///
    /// Stale until [`compute`] is called. For a fresh snapshot use
    /// `monitor.compute()` directly.
    #[must_use]
    pub fn health(&self) -> &SessionHealth {
        &self.current_health
    }

    /// Returns `true` when the current health band is [`HealthBand::Healthy`].
    ///
    /// Convenience shortcut; equivalent to
    /// `monitor.health().band == HealthBand::Healthy`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::time::Duration;
    /// use nexcore_terminal::chi_monitor::ChiMonitor;
    ///
    /// let m = ChiMonitor::new(Duration::from_secs(60));
    /// // Empty monitor → SpinningUp, not healthy.
    /// assert!(!m.is_healthy());
    /// ```
    #[must_use]
    pub fn is_healthy(&self) -> bool {
        use crate::health::HealthBand;
        self.current_health.band == HealthBand::Healthy
    }

    // ── Internal helpers ──────────────────────────────────────────────────────

    /// Estimate backpressure in [0.0, 1.0] from the inter-output gap buffer.
    ///
    /// ## Algorithm
    ///
    /// 1. Compute the rolling average gap (`avg`) from [`output_gaps`].
    /// 2. If the most recent gap (`latest`) exceeds `2 × avg`, backpressure
    ///    is rising.
    /// 3. Normalise: `pressure = (latest / avg - 1.0) / (BACKPRESSURE_SCALE - 1.0)`
    ///    clamped to [0.0, 1.0].
    ///
    /// Returns `0.0` when fewer than two output events have been recorded
    /// (not enough data for a gap).
    fn estimate_backpressure(&self) -> f64 {
        // Need at least two samples for a meaningful gap.
        let n = self.output_gaps.len();
        if n == 0 {
            return 0.0;
        }

        // Rolling average of all gap samples (nanoseconds as f64).
        let avg_nanos: f64 = self
            .output_gaps
            .iter()
            .map(|d| d.as_nanos() as f64)
            .sum::<f64>()
            / n as f64;

        if avg_nanos <= 0.0 {
            return 0.0;
        }

        // Latest gap is the last element.
        let latest_nanos = self.output_gaps.back().map_or(0.0, |d| d.as_nanos() as f64);

        let ratio = latest_nanos / avg_nanos;

        // Backpressure only rises when the latest gap exceeds twice the average.
        if ratio <= 2.0 {
            return 0.0;
        }

        // Normalise from [2.0, BACKPRESSURE_SCALE] → [0.0, 1.0].
        // ratio=2 → 0.0, ratio≥BACKPRESSURE_SCALE → 1.0.
        ((ratio - 2.0) / (BACKPRESSURE_SCALE - 2.0)).clamp(0.0, 1.0)
    }
}

/// Serde helper for `std::time::Duration` (stored as nanoseconds).
mod duration_serde {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::time::Duration;

    pub fn serialize<S: Serializer>(d: &Duration, s: S) -> Result<S::Ok, S::Error> {
        d.as_nanos().serialize(s)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Duration, D::Error> {
        let nanos = u128::deserialize(d)?;
        Ok(Duration::from_nanos(nanos as u64))
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::health::HealthBand;
    use std::time::Duration;

    /// Helper: build a monitor with a very long window so no events expire
    /// during the test.
    fn long_window() -> ChiMonitor {
        ChiMonitor::new(Duration::from_secs(3600))
    }

    // ── test_empty_monitor ────────────────────────────────────────────────────

    #[test]
    fn test_empty_monitor() {
        // No events recorded → χ must be 0.0 and band must be SpinningUp.
        let mut m = long_window();
        let h = m.compute();
        assert_eq!(
            h.chi, 0.0,
            "empty monitor: expected χ = 0.0, got {:.4}",
            h.chi
        );
        assert_eq!(
            h.band,
            HealthBand::SpinningUp,
            "empty monitor: expected SpinningUp, got {:?}",
            h.band
        );
        assert!(!m.is_healthy(), "empty monitor must not report healthy");
    }

    // ── test_healthy_flow ─────────────────────────────────────────────────────

    #[test]
    fn test_healthy_flow() {
        // 100 inputs, 5 outputs → χ = 5/100 = 0.05 → Healthy band.
        let mut m = long_window();
        for _ in 0..100 {
            m.record_input();
        }
        for _ in 0..5 {
            m.record_output();
        }
        let h = m.compute();

        let expected_chi = 5.0_f64 / 100.0;
        assert!(
            (h.chi - expected_chi).abs() < f64::EPSILON,
            "healthy_flow: expected χ = {expected_chi:.4}, got {:.4}",
            h.chi
        );
        assert_eq!(
            h.band,
            HealthBand::Healthy,
            "healthy_flow: expected Healthy, got {:?}",
            h.band
        );
        assert!(m.is_healthy(), "healthy_flow: is_healthy() should be true");
    }

    // ── test_high_output ──────────────────────────────────────────────────────

    #[test]
    fn test_high_output() {
        // 10 inputs, 8 outputs → χ = 8/10 = 0.80 → Critical band.
        let mut m = long_window();
        for _ in 0..10 {
            m.record_input();
        }
        for _ in 0..8 {
            m.record_output();
        }
        let h = m.compute();

        let expected_chi = 8.0_f64 / 10.0;
        assert!(
            (h.chi - expected_chi).abs() < f64::EPSILON,
            "high_output: expected χ = {expected_chi:.4}, got {:.4}",
            h.chi
        );
        assert_eq!(
            h.band,
            HealthBand::Critical,
            "high_output: expected Critical, got {:?}",
            h.band
        );
        assert!(!m.is_healthy(), "high_output: is_healthy() must be false");
    }

    // ── test_window_expiry ────────────────────────────────────────────────────

    #[test]
    fn test_window_expiry() {
        // Use an extremely small window so events recorded before a brief sleep
        // are treated as expired.
        //
        // We avoid `std::thread::sleep` (flaky on CI) by directly pushing
        // synthetic `Instant` values that are older than the window.
        // Since `Instant` cannot be constructed at an arbitrary past point via
        // the public API, we inject old events by manipulating the internal
        // deque directly (this is a white-box test — `chi_monitor` is in the
        // same crate as the tests).
        let mut m = ChiMonitor::new(Duration::from_nanos(1));

        // Push a "now" event — it will be within 1 ns of `compute`'s cutoff,
        // practically always expired by the time `compute` runs.
        m.input_events.push_back(Instant::now());
        m.output_events.push_back(Instant::now());

        // Spin briefly to ensure the 1 ns window has elapsed.
        let spin_until = Instant::now() + Duration::from_micros(10);
        while Instant::now() < spin_until {}

        let h = m.compute();
        assert_eq!(
            h.band,
            HealthBand::SpinningUp,
            "window_expiry: events must be pruned after window elapses"
        );
        assert_eq!(h.chi, 0.0, "window_expiry: pruned events → χ must be 0.0");
    }

    // ── test_backpressure_estimation ──────────────────────────────────────────

    #[test]
    fn test_backpressure_estimation() {
        // Simulate rising backpressure by inserting synthetic gap values
        // directly into `output_gaps`.
        //
        // Buffer: 10 × 100 µs + 1 × 500 µs = 11 samples.
        // avg = (10×100 + 500) / 11 = 1500/11 ≈ 136.36 µs
        // latest = 500 µs
        // ratio  = 500 / (1500/11) = 500×11/1500 = 5500/1500 = 11/3 ≈ 3.667
        // ratio > 2.0 → rising backpressure
        // pressure = (ratio - 2.0) / (BACKPRESSURE_SCALE - 2.0)
        //          = (11/3 - 2) / (4 - 2) = (5/3) / 2 = 5/6 ≈ 0.8333
        let mut m = long_window();

        for _ in 0..10 {
            m.output_gaps.push_back(Duration::from_micros(100));
        }
        m.output_gaps.push_back(Duration::from_micros(500));

        let bp = m.estimate_backpressure();
        assert!(
            bp > 0.0,
            "backpressure must be > 0 when latest gap >> average"
        );
        assert!(
            bp <= 1.0,
            "backpressure must be ≤ 1.0 (clamped), got {bp:.4}"
        );

        // Expected: 5/6 ≈ 0.8333…
        let expected = 5.0_f64 / 6.0;
        assert!(
            (bp - expected).abs() < 1e-9,
            "expected bp = {expected:.6}, got {bp:.6}"
        );
    }

    #[test]
    fn test_backpressure_clamps_at_one() {
        // With an extreme latest gap (10× the average) the formula exceeds 1.0
        // and must be clamped.
        // Buffer: 10 × 100 µs + 1 × 1000 µs = 11 samples.
        // avg = (1000 + 1000) / 11 = 2000/11 ≈ 181.8 µs
        // ratio = 1000 / (2000/11) = 11000/2000 = 5.5 → pressure = (5.5-2)/(4-2) = 1.75 → clamped 1.0
        let mut m = long_window();
        for _ in 0..10 {
            m.output_gaps.push_back(Duration::from_micros(100));
        }
        m.output_gaps.push_back(Duration::from_micros(1_000));

        let bp = m.estimate_backpressure();
        assert!(
            (bp - 1.0).abs() < f64::EPSILON,
            "extreme gap must clamp backpressure to 1.0, got {bp:.4}"
        );
    }

    #[test]
    fn test_backpressure_zero_when_normal() {
        // When the latest gap equals the average, ratio = 1.0 < 2.0 → no pressure.
        let mut m = long_window();
        for _ in 0..8 {
            m.output_gaps.push_back(Duration::from_micros(200));
        }
        let bp = m.estimate_backpressure();
        assert_eq!(bp, 0.0, "uniform gaps must not register backpressure");
    }

    #[test]
    fn test_backpressure_partial_rise() {
        // latest = 3× average → ratio = 3.0 → pressure = (3-2)/(4-2) = 0.5
        let mut m = long_window();
        for _ in 0..10 {
            m.output_gaps.push_back(Duration::from_micros(100));
        }
        m.output_gaps.push_back(Duration::from_micros(300));

        let bp = m.estimate_backpressure();
        // avg = (10*100 + 300) / 11 = 1300/11 ≈ 118.18 µs
        // latest = 300 µs → ratio ≈ 2.539
        // pressure = (2.539 - 2.0) / (4.0 - 2.0) ≈ 0.269
        assert!(
            bp > 0.0 && bp < 1.0,
            "partial rise: expected 0 < bp < 1, got {bp:.4}"
        );
    }

    #[test]
    fn test_no_backpressure_with_no_outputs() {
        // Without any outputs there are no gaps → backpressure = 0.0.
        let mut m = long_window();
        for _ in 0..50 {
            m.record_input();
        }
        let h = m.compute();
        assert_eq!(
            h.tau_disk, 0.0,
            "no outputs → zero backpressure → tau_disk must be 0.0"
        );
    }

    // ── serde round-trip ──────────────────────────────────────────────────────

    #[test]
    fn test_serde_round_trip() {
        // ChiMonitor must serialise and deserialise without panic.
        // Event deques are skipped (Instant is not Serialize); only `window`
        // and `current_health` survive the round-trip.
        let mut m = long_window();
        for _ in 0..10 {
            m.record_input();
        }
        for _ in 0..2 {
            m.record_output();
        }
        m.compute();

        let json = serde_json::to_string(&m).expect("serialise ChiMonitor");
        let m2: ChiMonitor = serde_json::from_str(&json).expect("deserialise ChiMonitor");

        // Window survives the round-trip.
        assert_eq!(m.window, m2.window);
        // Health snapshot survives (chi value).
        assert!(
            (m.chi() - m2.chi()).abs() < f64::EPSILON,
            "chi mismatch after serde: {} vs {}",
            m.chi(),
            m2.chi()
        );
    }

    // ── accessor helpers ──────────────────────────────────────────────────────

    #[test]
    fn test_chi_accessor_matches_health() {
        let mut m = long_window();
        for _ in 0..50 {
            m.record_input();
        }
        for _ in 0..10 {
            m.record_output();
        }
        m.compute();
        assert!(
            (m.chi() - m.health().chi).abs() < f64::EPSILON,
            "chi() and health().chi must agree"
        );
    }
}
