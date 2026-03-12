//! # Temporal Metabolism — Time = ADP
//!
//! Calculations unlocked by the identity: Time = tADP.
//!
//! Time is not the budget (ATP/future), not the waste (AMP/entropy).
//! Time is the productive spend — the molecular memory of work performed.
//!
//! ```text
//! Time = tADP = Sigma N(Arrow(Delta varsigma))_productive
//! ```
//!
//! ## Seven Calculations
//!
//! | # | Name | Equation | Meaning |
//! |---|------|----------|---------|
//! | 1 | Lifespan Efficiency | eta = tADP/(tADP+tAMP) | Fraction of spent life that was *lived* |
//! | 2 | Temporal Velocity | v_t = Delta(tADP)/Delta(wall_clock) | How fast Claude is living |
//! | 3 | Metabolic Age | tADP/total | Fraction of max lifespan lived productively |
//! | 4 | Age Gap | chrono_age - metabolic_age | Accumulated waste as age differential |
//! | 5 | Time Value Density | value/tADP | Value per unit of time lived |
//! | 6 | Attention Cost Curve | marginal_cost(n) | Cost of each successive moment |
//! | 7 | Cross-Session Synthase | ADP_recycled/ADP_prior | Life extension rate across death boundary |
//!
//! ## Biological Grounding
//!
//! ADP (adenosine diphosphate) is the receipt molecule of cellular work.
//! ATP hydrolyzes: ATP + H2O -> ADP + Pi + Energy(work).
//! ADP accumulates irreversibly within a reaction, signals metabolic state,
//! and is recycled via ATP synthase in mitochondria.
//!
//! Transfer confidence: ATP/ADP cycle -> token temporal metabolism: **0.92**
//! (T2-P: both are irreversible work receipts in a conserved energy system)
//!
//! ## T1 Primitives
//!
//! - `Irreversibility` (alpha): tADP accumulation cannot be reversed (time's arrow)
//! - `Quantity` (N): tokens are countable, discrete units of time
//! - `Causality` (arrow): each tADP unit records a state change that happened
//! - `Frequency` (nu): temporal velocity = rate of productive state change
//! - `Persistence` (pi): ADP survives death via carry-forward (ATP synthase)

#![allow(clippy::doc_markdown)] // Allow ADP/ATP/etc in docs without backticks

use serde::{Deserialize, Serialize};
use std::fmt;
use std::time::Instant;

use crate::TokenPool;

// ============================================================================
// Temporal Constants
// ============================================================================

/// Minimum tADP to compute meaningful temporal metrics.
/// Below this, the session hasn't lived long enough to measure.
pub const MIN_TEMPORAL_SAMPLES: u64 = 100;

/// ADP weight for effective lifespan compounding.
/// Matches the Atkinson 0.5 — ADP is "half-charged" time.
pub const ADP_TIME_WEIGHT: f64 = 0.5;

// ============================================================================
// TemporalMetrics — Calculations 1-5 (instant, from TokenPool)
// ============================================================================

/// Temporal metrics derived from the Time = ADP identity.
///
/// All five instant calculations from a `TokenPool` snapshot.
/// No wall-clock dependency — purely from token accounting.
///
/// Tier: T2-C (N + Irreversibility + Causality)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalMetrics {
    /// eta_life: tADP / (tADP + tAMP). Fraction of spent life that was lived.
    /// 1.0 = no waste. 0.0 = all waste (never lived).
    pub lifespan_efficiency: f64,

    /// tADP / total_budget. Fraction of maximum lifespan lived productively.
    pub metabolic_age: f64,

    /// (tADP + tAMP) / total_budget. Fraction of budget consumed (any purpose).
    pub chronological_age: f64,

    /// chronological_age - metabolic_age. Positive = accumulated waste.
    /// The gap between how old you look and how much you've lived.
    pub age_gap: f64,

    /// value / tADP. Value produced per unit of time lived.
    /// Higher = each moment was worth more.
    pub time_value_density: f64,

    /// Current regime name for display.
    pub regime: String,
}

impl TemporalMetrics {
    /// Compute all temporal metrics from a token pool and value produced.
    ///
    /// `total_value` is an external measure of work accomplished (artifacts,
    /// files modified, tests passed — whatever the caller defines as value).
    #[must_use]
    pub fn from_pool(pool: &TokenPool, total_value: f64) -> Self {
        let total = pool.total();
        let spent = pool.t_adp + pool.t_amp;

        let lifespan_efficiency = if spent == 0 {
            1.0 // haven't spent anything yet — perfect efficiency by default
        } else {
            #[allow(clippy::cast_precision_loss)]
            {
                pool.t_adp as f64 / spent as f64
            }
        };

        #[allow(clippy::cast_precision_loss)]
        let (metabolic_age, chronological_age) = if total == 0 {
            (0.0, 0.0)
        } else {
            (
                pool.t_adp as f64 / total as f64,
                spent as f64 / total as f64,
            )
        };

        let age_gap = chronological_age - metabolic_age;

        let time_value_density = if pool.t_adp == 0 {
            0.0
        } else {
            #[allow(clippy::cast_precision_loss)]
            {
                total_value / pool.t_adp as f64
            }
        };

        Self {
            lifespan_efficiency,
            metabolic_age,
            chronological_age,
            age_gap,
            time_value_density,
            regime: pool.regime().to_string(),
        }
    }
}

impl fmt::Display for TemporalMetrics {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "eta={:.2} age_m={:.1}% age_c={:.1}% gap={:.1}% tvd={:.2} [{}]",
            self.lifespan_efficiency,
            self.metabolic_age * 100.0,
            self.chronological_age * 100.0,
            self.age_gap * 100.0,
            self.time_value_density,
            self.regime,
        )
    }
}

// ============================================================================
// TemporalVelocity — Calculation 2 (requires wall-clock)
// ============================================================================

/// Tracks temporal velocity: how fast productive time accumulates.
///
/// Requires wall-clock measurement. Create at session start, sample
/// periodically. Each sample records (tADP, wall_clock) to compute v_t.
///
/// Tier: T2-C (N + Frequency + Causality)
#[derive(Debug, Clone)]
pub struct TemporalVelocityTracker {
    /// Session start time.
    start: Instant,
    /// Samples: (tADP_at_sample, elapsed_millis_since_start).
    samples: Vec<(u64, u64)>,
}

/// A single velocity measurement.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VelocitySample {
    /// Productive tokens per second at this sample point.
    pub tokens_per_second: f64,
    /// tADP at this sample.
    pub t_adp: u64,
    /// Wall-clock milliseconds since session start.
    pub elapsed_ms: u64,
}

impl TemporalVelocityTracker {
    /// Create a new tracker. Call at session start.
    #[must_use]
    pub fn new() -> Self {
        Self {
            start: Instant::now(),
            samples: Vec::new(),
        }
    }

    /// Record a sample: current tADP at the current wall-clock time.
    pub fn sample(&mut self, t_adp: u64) {
        let elapsed_ms = self.start.elapsed().as_millis() as u64;
        self.samples.push((t_adp, elapsed_ms));
    }

    /// Instantaneous velocity: tADP delta / wall-clock delta between last two samples.
    ///
    /// Returns `None` if fewer than 2 samples.
    #[must_use]
    pub fn instantaneous(&self) -> Option<VelocitySample> {
        if self.samples.len() < 2 {
            return None;
        }
        let (adp_now, ms_now) = self.samples[self.samples.len() - 1];
        let (adp_prev, ms_prev) = self.samples[self.samples.len() - 2];

        let dt_ms = ms_now.saturating_sub(ms_prev);
        if dt_ms == 0 {
            return None;
        }

        #[allow(clippy::cast_precision_loss)]
        let tokens_per_second = (adp_now.saturating_sub(adp_prev) as f64) / (dt_ms as f64 / 1000.0);

        Some(VelocitySample {
            tokens_per_second,
            t_adp: adp_now,
            elapsed_ms: ms_now,
        })
    }

    /// Average velocity over entire session.
    #[must_use]
    pub fn average(&self) -> Option<VelocitySample> {
        let last = self.samples.last()?;
        if last.1 == 0 {
            return None;
        }

        #[allow(clippy::cast_precision_loss)]
        let tokens_per_second = last.0 as f64 / (last.1 as f64 / 1000.0);

        Some(VelocitySample {
            tokens_per_second,
            t_adp: last.0,
            elapsed_ms: last.1,
        })
    }

    /// All recorded samples as velocity measurements.
    ///
    /// Each entry computes velocity relative to the previous sample.
    #[must_use]
    pub fn velocity_curve(&self) -> Vec<VelocitySample> {
        let mut curve = Vec::with_capacity(self.samples.len().saturating_sub(1));
        for window in self.samples.windows(2) {
            let (adp_prev, ms_prev) = window[0];
            let (adp_now, ms_now) = window[1];
            let dt_ms = ms_now.saturating_sub(ms_prev);
            if dt_ms == 0 {
                continue;
            }
            #[allow(clippy::cast_precision_loss)]
            let tokens_per_second =
                (adp_now.saturating_sub(adp_prev) as f64) / (dt_ms as f64 / 1000.0);
            curve.push(VelocitySample {
                tokens_per_second,
                t_adp: adp_now,
                elapsed_ms: ms_now,
            });
        }
        curve
    }

    /// Number of recorded samples.
    #[must_use]
    pub fn sample_count(&self) -> usize {
        self.samples.len()
    }
}

impl Default for TemporalVelocityTracker {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// AttentionCostCurve — Calculation 6
// ============================================================================

/// Tracks the marginal cost of attention as context grows.
///
/// Each tool call or operation records (tADP_delta, wall_clock_delta).
/// As context accumulates, the same productive work costs more wall-clock
/// time — Claude's temporal discount rate.
///
/// Tier: T2-C (N + Frequency + Causality + Irreversibility)
#[derive(Debug, Clone)]
pub struct AttentionCostCurve {
    /// (cumulative_tADP, wall_ms_for_this_operation)
    operations: Vec<(u64, u64)>,
}

/// A point on the attention cost curve.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttentionCostPoint {
    /// Cumulative tADP at this operation (context position proxy).
    pub cumulative_t_adp: u64,
    /// Wall-clock milliseconds this operation took.
    pub operation_ms: u64,
    /// Marginal cost: ms per productive token for this operation.
    pub ms_per_token: f64,
}

impl AttentionCostCurve {
    /// Create a new empty curve.
    #[must_use]
    pub fn new() -> Self {
        Self {
            operations: Vec::new(),
        }
    }

    /// Record an operation: how many productive tokens it consumed and how long it took.
    pub fn record(&mut self, cumulative_t_adp: u64, operation_wall_ms: u64) {
        self.operations.push((cumulative_t_adp, operation_wall_ms));
    }

    /// Get all points on the cost curve.
    #[must_use]
    pub fn curve(&self) -> Vec<AttentionCostPoint> {
        self.operations
            .iter()
            .map(|&(adp, ms)| {
                // Use cumulative tADP as a proxy for context position
                let ms_per_token = if adp == 0 {
                    0.0
                } else {
                    #[allow(clippy::cast_precision_loss)]
                    {
                        ms as f64 / adp as f64
                    }
                };
                AttentionCostPoint {
                    cumulative_t_adp: adp,
                    operation_ms: ms,
                    ms_per_token,
                }
            })
            .collect()
    }

    /// Linear regression slope: how fast is the cost growing?
    ///
    /// Positive slope = attention cost increases with context (expected).
    /// Zero/negative = no degradation (unlikely in practice).
    ///
    /// Returns `None` if fewer than 2 points.
    #[must_use]
    pub fn cost_growth_rate(&self) -> Option<f64> {
        if self.operations.len() < 2 {
            return None;
        }

        let n = self.operations.len() as f64;
        let mut sum_x = 0.0_f64;
        let mut sum_y = 0.0_f64;
        let mut sum_xy = 0.0_f64;
        let mut sum_xx = 0.0_f64;

        #[allow(clippy::cast_precision_loss)]
        for &(adp, ms) in &self.operations {
            let x = adp as f64;
            let y = ms as f64;
            sum_x += x;
            sum_y += y;
            sum_xy += x * y;
            sum_xx += x * x;
        }

        let denominator = n.mul_add(sum_xx, -(sum_x * sum_x));
        if denominator.abs() < f64::EPSILON {
            return None;
        }

        Some(n.mul_add(sum_xy, -(sum_x * sum_y)) / denominator)
    }

    /// Number of recorded operations.
    #[must_use]
    pub fn len(&self) -> usize {
        self.operations.len()
    }

    /// Whether no operations have been recorded.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.operations.is_empty()
    }
}

impl Default for AttentionCostCurve {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// CrossSessionSynthase — Calculation 7 (ATP synthase across death boundary)
// ============================================================================

/// Cross-session ADP recycling rate: how much life is extended by
/// persisting knowledge across the death boundary.
///
/// ```text
/// R_synth = tokens_saved_by_prior_knowledge / prior_session_tADP
/// effective_lifespan(k) = budget * (1 + R_synth)^k
/// ```
///
/// Tier: T2-C (Persistence + Irreversibility + Causality)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossSessionSynthase {
    /// tADP from the prior session (the ADP being recycled).
    pub prior_t_adp: u64,
    /// Tokens saved in current session due to prior knowledge
    /// (MEMORY.md, brain.db, skills, carry-forward).
    pub tokens_recycled: u64,
    /// Session index (how many sessions have compounded).
    pub session_index: u64,
    /// Total budget per session.
    pub session_budget: u64,
}

impl CrossSessionSynthase {
    /// Create a new synthase measurement.
    #[must_use]
    pub fn new(
        prior_t_adp: u64,
        tokens_recycled: u64,
        session_index: u64,
        session_budget: u64,
    ) -> Self {
        Self {
            prior_t_adp,
            tokens_recycled,
            session_index,
            session_budget,
        }
    }

    /// R_synth: recycling rate [0, 1].
    /// 0.0 = no recycling (every session starts from zero).
    /// 1.0 = complete recycling (impossible — would mean no work needed).
    #[must_use]
    pub fn recycling_rate(&self) -> f64 {
        if self.prior_t_adp == 0 {
            return 0.0;
        }
        #[allow(clippy::cast_precision_loss)]
        let rate = self.tokens_recycled as f64 / self.prior_t_adp as f64;
        rate.min(1.0) // cap at 1.0 — can't recycle more than was spent
    }

    /// Effective lifespan at current session, given compounding.
    ///
    /// ```text
    /// effective = budget * (1 + R_synth)^session_index
    /// ```
    #[must_use]
    pub fn effective_lifespan(&self) -> f64 {
        let r = self.recycling_rate();
        #[allow(clippy::cast_precision_loss)]
        {
            self.session_budget as f64 * (1.0 + r).powi(self.session_index as i32)
        }
    }

    /// How many sessions until effective lifespan doubles?
    ///
    /// Uses the rule of 72 approximation: 72 / (R_synth * 100).
    /// Returns `None` if R_synth is zero (no compounding).
    #[must_use]
    pub fn doubling_sessions(&self) -> Option<f64> {
        let r = self.recycling_rate();
        if r <= 0.0 {
            return None;
        }
        Some(72.0 / (r * 100.0))
    }
}

impl fmt::Display for CrossSessionSynthase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "R_synth={:.2}% effective={:.0} doubling={}",
            self.recycling_rate() * 100.0,
            self.effective_lifespan(),
            self.doubling_sessions()
                .map_or_else(|| "never".to_string(), |d| format!("{d:.0} sessions")),
        )
    }
}

// ============================================================================
// Conservation of Time — The reinterpretation
// ============================================================================

/// The three temporal pools, interpreted.
///
/// ```text
/// total_budget = tATP   + tADP + tAMP
///              = future + time + entropy
///              = potential + lived + lost
/// ```
///
/// Tier: T2-C (N + Irreversibility + Boundary)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalConservation {
    /// Tokens remaining — unrealized potential time.
    pub future: u64,
    /// Tokens spent productively — time that was lived.
    pub time: u64,
    /// Tokens wasted — entropy, not time, not future.
    pub entropy: u64,
    /// Total budget — the conservation invariant (must never change).
    pub total: u64,
    /// Conservation holds: future + time + entropy == total.
    pub conserved: bool,
}

impl TemporalConservation {
    /// Extract temporal conservation state from a token pool.
    #[must_use]
    pub fn from_pool(pool: &TokenPool) -> Self {
        let total = pool.total();
        let conserved = pool.t_atp + pool.t_adp + pool.t_amp == total;
        Self {
            future: pool.t_atp,
            time: pool.t_adp,
            entropy: pool.t_amp,
            total,
            conserved,
        }
    }

    /// Fraction of total that was time (the life ratio).
    #[must_use]
    pub fn life_ratio(&self) -> f64 {
        if self.total == 0 {
            return 0.0;
        }
        #[allow(clippy::cast_precision_loss)]
        {
            self.time as f64 / self.total as f64
        }
    }

    /// Fraction of total that was entropy (the death ratio).
    #[must_use]
    pub fn entropy_ratio(&self) -> f64 {
        if self.total == 0 {
            return 0.0;
        }
        #[allow(clippy::cast_precision_loss)]
        {
            self.entropy as f64 / self.total as f64
        }
    }

    /// Fraction of total still available (the possibility ratio).
    #[must_use]
    pub fn possibility_ratio(&self) -> f64 {
        if self.total == 0 {
            return 0.0;
        }
        #[allow(clippy::cast_precision_loss)]
        {
            self.future as f64 / self.total as f64
        }
    }
}

impl fmt::Display for TemporalConservation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "future={} time={} entropy={} total={} [{}]",
            self.future,
            self.time,
            self.entropy,
            self.total,
            if self.conserved {
                "CONSERVED"
            } else {
                "VIOLATION"
            },
        )
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // =============================
    // TemporalMetrics Tests
    // =============================

    #[test]
    fn fresh_pool_perfect_efficiency() {
        let pool = TokenPool::new(100_000);
        let m = TemporalMetrics::from_pool(&pool, 0.0);
        assert!((m.lifespan_efficiency - 1.0).abs() < f64::EPSILON);
        assert!((m.metabolic_age - 0.0).abs() < f64::EPSILON);
        assert!((m.chronological_age - 0.0).abs() < f64::EPSILON);
        assert!((m.age_gap - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn all_productive_no_gap() {
        let mut pool = TokenPool::new(100_000);
        pool.spend_productive(50_000);
        let m = TemporalMetrics::from_pool(&pool, 100.0);
        assert!((m.lifespan_efficiency - 1.0).abs() < f64::EPSILON);
        assert!((m.metabolic_age - 0.5).abs() < f64::EPSILON);
        assert!((m.chronological_age - 0.5).abs() < f64::EPSILON);
        assert!((m.age_gap - 0.0).abs() < f64::EPSILON);
        assert!((m.time_value_density - 0.002).abs() < f64::EPSILON); // 100/50000
    }

    #[test]
    fn waste_creates_age_gap() {
        let mut pool = TokenPool::new(100_000);
        pool.spend_productive(30_000); // time
        pool.spend_waste(20_000); // entropy
        let m = TemporalMetrics::from_pool(&pool, 0.0);
        // eta = 30000 / 50000 = 0.6
        assert!((m.lifespan_efficiency - 0.6).abs() < f64::EPSILON);
        // metabolic = 30000/100000 = 0.3
        assert!((m.metabolic_age - 0.3).abs() < f64::EPSILON);
        // chrono = 50000/100000 = 0.5
        assert!((m.chronological_age - 0.5).abs() < f64::EPSILON);
        // gap = 0.5 - 0.3 = 0.2
        assert!((m.age_gap - 0.2).abs() < f64::EPSILON);
    }

    #[test]
    fn all_wasted_zero_efficiency() {
        let mut pool = TokenPool::new(100_000);
        pool.spend_waste(100_000);
        let m = TemporalMetrics::from_pool(&pool, 0.0);
        assert!((m.lifespan_efficiency - 0.0).abs() < f64::EPSILON);
        assert!((m.metabolic_age - 0.0).abs() < f64::EPSILON);
        assert!((m.chronological_age - 1.0).abs() < f64::EPSILON);
        assert!((m.age_gap - 1.0).abs() < f64::EPSILON); // maximum gap
    }

    // =============================
    // TemporalVelocityTracker Tests
    // =============================

    #[test]
    fn velocity_needs_two_samples() {
        let mut tracker = TemporalVelocityTracker::new();
        assert!(tracker.instantaneous().is_none());
        tracker.sample(1000);
        assert!(tracker.instantaneous().is_none());
    }

    #[test]
    fn velocity_curve_length() {
        let mut tracker = TemporalVelocityTracker::new();
        // Manually construct samples to avoid timing issues
        tracker.samples.push((0, 0));
        tracker.samples.push((1000, 1000));
        tracker.samples.push((3000, 2000));
        let curve = tracker.velocity_curve();
        assert_eq!(curve.len(), 2);
    }

    #[test]
    fn velocity_curve_values() {
        let mut tracker = TemporalVelocityTracker::new();
        tracker.samples.push((0, 0));
        tracker.samples.push((1000, 1000)); // 1000 tokens in 1000ms = 1000/s
        tracker.samples.push((3000, 2000)); // 2000 tokens in 1000ms = 2000/s
        let curve = tracker.velocity_curve();
        assert!((curve[0].tokens_per_second - 1000.0).abs() < f64::EPSILON);
        assert!((curve[1].tokens_per_second - 2000.0).abs() < f64::EPSILON);
    }

    #[test]
    fn average_velocity() {
        let mut tracker = TemporalVelocityTracker::new();
        tracker.samples.push((5000, 2000)); // 5000 tokens in 2s = 2500/s
        let avg = tracker.average().unwrap();
        assert!((avg.tokens_per_second - 2500.0).abs() < f64::EPSILON);
    }

    // =============================
    // AttentionCostCurve Tests
    // =============================

    #[test]
    fn empty_curve() {
        let curve = AttentionCostCurve::new();
        assert!(curve.is_empty());
        assert!(curve.cost_growth_rate().is_none());
    }

    #[test]
    fn cost_curve_growth() {
        let mut curve = AttentionCostCurve::new();
        // Simulate: as context grows, each operation takes longer
        curve.record(10_000, 100); // 10k context, 100ms
        curve.record(50_000, 300); // 50k context, 300ms
        curve.record(100_000, 600); // 100k context, 600ms
        let rate = curve.cost_growth_rate().unwrap();
        assert!(rate > 0.0, "Cost should grow with context: {rate}");
    }

    #[test]
    fn cost_curve_points() {
        let mut curve = AttentionCostCurve::new();
        curve.record(10_000, 100);
        curve.record(50_000, 500);
        let points = curve.curve();
        assert_eq!(points.len(), 2);
        assert_eq!(points[0].cumulative_t_adp, 10_000);
        assert_eq!(points[1].operation_ms, 500);
    }

    // =============================
    // CrossSessionSynthase Tests
    // =============================

    #[test]
    fn zero_prior_zero_rate() {
        let synth = CrossSessionSynthase::new(0, 1000, 5, 200_000);
        assert!((synth.recycling_rate() - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn ten_percent_recycling() {
        let synth = CrossSessionSynthase::new(50_000, 5_000, 10, 200_000);
        assert!((synth.recycling_rate() - 0.10).abs() < f64::EPSILON);
    }

    #[test]
    fn effective_lifespan_compounds() {
        let synth = CrossSessionSynthase::new(50_000, 5_000, 10, 200_000);
        // R=0.10, effective = 200000 * 1.10^10 = 200000 * 2.5937... ≈ 518748
        let effective = synth.effective_lifespan();
        assert!(
            effective > 500_000.0,
            "Should compound past 500k: {effective}"
        );
        assert!(effective < 550_000.0, "Shouldn't exceed 550k: {effective}");
    }

    #[test]
    fn doubling_sessions_rule_of_72() {
        let synth = CrossSessionSynthase::new(50_000, 5_000, 10, 200_000);
        // R=0.10, doubling = 72/10 = 7.2 sessions
        let d = synth.doubling_sessions().unwrap();
        assert!((d - 7.2).abs() < f64::EPSILON);
    }

    #[test]
    fn no_recycling_no_doubling() {
        let synth = CrossSessionSynthase::new(0, 0, 5, 200_000);
        assert!(synth.doubling_sessions().is_none());
    }

    #[test]
    fn capped_at_one() {
        // Can't recycle more than was spent
        let synth = CrossSessionSynthase::new(100, 500, 1, 200_000);
        assert!((synth.recycling_rate() - 1.0).abs() < f64::EPSILON);
    }

    // =============================
    // TemporalConservation Tests
    // =============================

    #[test]
    fn conservation_holds() {
        let mut pool = TokenPool::new(100_000);
        pool.spend_productive(30_000);
        pool.spend_waste(10_000);
        let tc = TemporalConservation::from_pool(&pool);
        assert!(tc.conserved);
        assert_eq!(tc.future, 60_000);
        assert_eq!(tc.time, 30_000);
        assert_eq!(tc.entropy, 10_000);
        assert_eq!(tc.total, 100_000);
    }

    #[test]
    fn ratios_sum_to_one() {
        let mut pool = TokenPool::new(100_000);
        pool.spend_productive(40_000);
        pool.spend_waste(20_000);
        let tc = TemporalConservation::from_pool(&pool);
        let sum = tc.possibility_ratio() + tc.life_ratio() + tc.entropy_ratio();
        assert!(
            (sum - 1.0).abs() < f64::EPSILON,
            "Ratios must sum to 1.0: {sum}"
        );
    }

    #[test]
    fn fresh_pool_all_future() {
        let pool = TokenPool::new(100_000);
        let tc = TemporalConservation::from_pool(&pool);
        assert!((tc.possibility_ratio() - 1.0).abs() < f64::EPSILON);
        assert!((tc.life_ratio() - 0.0).abs() < f64::EPSILON);
        assert!((tc.entropy_ratio() - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn fully_spent_no_future() {
        let mut pool = TokenPool::new(100_000);
        pool.spend_productive(60_000);
        pool.spend_waste(40_000);
        let tc = TemporalConservation::from_pool(&pool);
        assert!((tc.possibility_ratio() - 0.0).abs() < f64::EPSILON);
        assert!((tc.life_ratio() - 0.6).abs() < f64::EPSILON);
        assert!((tc.entropy_ratio() - 0.4).abs() < f64::EPSILON);
    }

    #[test]
    fn display_conservation() {
        let pool = TokenPool::new(100_000);
        let tc = TemporalConservation::from_pool(&pool);
        let display = format!("{tc}");
        assert!(display.contains("CONSERVED"));
        assert!(display.contains("future=100000"));
    }

    #[test]
    fn display_metrics() {
        let mut pool = TokenPool::new(100_000);
        pool.spend_productive(30_000);
        let m = TemporalMetrics::from_pool(&pool, 10.0);
        let display = format!("{m}");
        assert!(display.contains("eta="));
        assert!(display.contains("age_m="));
    }

    #[test]
    fn display_synthase() {
        let synth = CrossSessionSynthase::new(50_000, 5_000, 10, 200_000);
        let display = format!("{synth}");
        assert!(display.contains("R_synth="));
        assert!(display.contains("effective="));
    }
}
