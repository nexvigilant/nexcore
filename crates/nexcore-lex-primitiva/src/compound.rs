//! # Compound Growth Tracking
//!
//! Time-series tracking of primitive basis velocity using the formula:
//!
//! **V(t) = B(t) x n(t) x r(t)**
//!
//! - B = basis size (total primitives)
//! - n = transfer efficiency (weighted avg tier confidence)
//! - r = reuse rate (reused / total_needed)

use crate::tier::Tier;
use serde::{Deserialize, Serialize};

// ============================================================================
// BasisSnapshot — a single point-in-time primitive basis measurement
// ============================================================================

/// A point-in-time snapshot of the primitive basis.
///
/// Tier: T2-C (sigma + varsigma + N -- Sequence + State + Quantity)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BasisSnapshot {
    /// Session identifier (e.g. "session-001")
    pub session: String,
    /// T1 primitives in the basis
    pub t1_count: u32,
    /// T2-P primitives in the basis
    pub t2_p_count: u32,
    /// T2-C primitives in the basis
    pub t2_c_count: u32,
    /// T3 primitives in the basis
    pub t3_count: u32,
    /// Primitives reused from the existing basis
    pub reused: u32,
    /// Total primitives needed for the session
    pub total_needed: u32,
}

impl BasisSnapshot {
    /// Total number of primitives across all tiers.
    #[must_use]
    pub fn basis_size(&self) -> u32 {
        self.t1_count + self.t2_p_count + self.t2_c_count + self.t3_count
    }

    /// Reuse rate: fraction of needed primitives that were reused.
    ///
    /// Returns 0.0 when `total_needed` is zero.
    #[must_use]
    pub fn reuse_rate(&self) -> f64 {
        if self.total_needed == 0 {
            return 0.0;
        }
        self.reused as f64 / self.total_needed as f64
    }

    /// Weighted transfer efficiency across all tiers.
    ///
    /// Each tier's count is weighted by its `transfer_multiplier()`.
    /// Returns 0.0 when the basis is empty.
    #[must_use]
    pub fn transfer_efficiency(&self) -> f64 {
        let counts = [
            (self.t1_count, Tier::T1Universal),
            (self.t2_p_count, Tier::T2Primitive),
            (self.t2_c_count, Tier::T2Composite),
            (self.t3_count, Tier::T3DomainSpecific),
        ];

        let total: u32 = counts.iter().map(|(c, _)| c).sum();
        if total == 0 {
            return 0.0;
        }

        let weighted_sum: f64 = counts
            .iter()
            .map(|(c, t)| *c as f64 * t.transfer_multiplier())
            .sum();

        weighted_sum / total as f64
    }

    /// Compound velocity: V = B x eta x r.
    #[must_use]
    pub fn velocity(&self) -> f64 {
        self.basis_size() as f64 * self.transfer_efficiency() * self.reuse_rate()
    }

    /// Raw component triple: (basis_size, transfer_efficiency, reuse_rate).
    #[must_use]
    pub fn component_triple(&self) -> (f64, f64, f64) {
        (
            self.basis_size() as f64,
            self.transfer_efficiency(),
            self.reuse_rate(),
        )
    }
}

// ============================================================================
// CompoundTracker — time-series accumulator
// ============================================================================

/// Time-series compound growth tracker.
///
/// Tier: T3 (Domain-specific compound growth tracker)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CompoundTracker {
    snapshots: Vec<BasisSnapshot>,
}

impl CompoundTracker {
    /// Create an empty tracker.
    #[must_use]
    pub fn new() -> Self {
        Self {
            snapshots: Vec::new(),
        }
    }

    /// Record a new basis snapshot. Returns `&mut Self` for chaining.
    pub fn record(&mut self, snapshot: BasisSnapshot) -> &mut Self {
        self.snapshots.push(snapshot);
        self
    }

    /// Number of recorded snapshots.
    #[must_use]
    pub fn len(&self) -> usize {
        self.snapshots.len()
    }

    /// Whether the tracker has no snapshots.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.snapshots.is_empty()
    }

    /// Most recent snapshot, if any.
    #[must_use]
    pub fn latest(&self) -> Option<&BasisSnapshot> {
        self.snapshots.last()
    }

    /// Current velocity (from the latest snapshot), or 0.0 if empty.
    #[must_use]
    pub fn current_velocity(&self) -> f64 {
        self.latest().map_or(0.0, BasisSnapshot::velocity)
    }

    /// Velocity at each recorded snapshot.
    #[must_use]
    pub fn velocity_history(&self) -> Vec<f64> {
        self.snapshots.iter().map(BasisSnapshot::velocity).collect()
    }

    /// Pairwise growth rates: V(t) / V(t-1) for consecutive snapshots.
    ///
    /// Returns empty vec if fewer than 2 snapshots. If V(t-1) is zero,
    /// that rate is recorded as 0.0 (no growth from zero).
    #[must_use]
    pub fn growth_rates(&self) -> Vec<f64> {
        let velocities = self.velocity_history();
        if velocities.len() < 2 {
            return Vec::new();
        }

        velocities
            .windows(2)
            .map(|w| if w[0] == 0.0 { 0.0 } else { w[1] / w[0] })
            .collect()
    }

    /// Geometric mean of growth rates.
    ///
    /// Returns 1.0 if no growth rates exist (fewer than 2 snapshots).
    /// Filters out non-positive rates before computing.
    #[must_use]
    pub fn avg_growth_rate(&self) -> f64 {
        let rates = self.growth_rates();
        let positive: Vec<f64> = rates.into_iter().filter(|r| *r > 0.0).collect();
        if positive.is_empty() {
            return 1.0;
        }

        let log_sum: f64 = positive.iter().map(|r| r.ln()).sum();
        (log_sum / positive.len() as f64).exp()
    }

    /// Project the effect of adding `count` primitives to a given tier.
    ///
    /// Clones the latest snapshot, increments the specified tier count,
    /// and computes the velocity delta.
    ///
    /// Returns `None` if the tracker is empty.
    #[must_use]
    pub fn project_addition(&self, tier: Tier, count: u32) -> Option<ProjectionResult> {
        let latest = self.latest()?;
        let current_velocity = latest.velocity();

        let mut projected = latest.clone();
        match tier {
            Tier::T1Universal => projected.t1_count += count,
            Tier::T2Primitive => projected.t2_p_count += count,
            Tier::T2Composite => projected.t2_c_count += count,
            Tier::T3DomainSpecific => projected.t3_count += count,
        }

        let projected_velocity = projected.velocity();
        let efficiency_delta = projected.transfer_efficiency() - latest.transfer_efficiency();

        Some(ProjectionResult {
            tier: tier.code().to_string(),
            count,
            current_velocity,
            projected_velocity,
            velocity_gain: projected_velocity - current_velocity,
            efficiency_delta,
        })
    }

    /// Borrow the snapshot slice.
    #[must_use]
    pub fn snapshots(&self) -> &[BasisSnapshot] {
        &self.snapshots
    }
}

// ============================================================================
// ProjectionResult — what-if analysis output
// ============================================================================

/// Result of projecting a tier addition.
///
/// Tier: T2-C (varsigma + N + proportional -- State + Quantity + Proportion)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectionResult {
    /// Tier code (e.g. "T1", "T2-P")
    pub tier: String,
    /// Number of primitives added
    pub count: u32,
    /// Velocity before the addition
    pub current_velocity: f64,
    /// Velocity after the addition
    pub projected_velocity: f64,
    /// Absolute velocity gain
    pub velocity_gain: f64,
    /// Change in transfer efficiency
    pub efficiency_delta: f64,
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_snapshot(session: &str) -> BasisSnapshot {
        BasisSnapshot {
            session: session.to_string(),
            t1_count: 15,
            t2_p_count: 2,
            t2_c_count: 13,
            t3_count: 0,
            reused: 28,
            total_needed: 30,
        }
    }

    fn growing_snapshot(session: &str, t2_p: u32, reused: u32, total: u32) -> BasisSnapshot {
        BasisSnapshot {
            session: session.to_string(),
            t1_count: 15,
            t2_p_count: t2_p,
            t2_c_count: 13,
            t3_count: 0,
            reused,
            total_needed: total,
        }
    }

    #[test]
    fn test_empty_snapshot() {
        let snap = BasisSnapshot {
            session: "empty".to_string(),
            t1_count: 0,
            t2_p_count: 0,
            t2_c_count: 0,
            t3_count: 0,
            reused: 0,
            total_needed: 0,
        };
        assert_eq!(snap.basis_size(), 0);
        assert!((snap.velocity() - 0.0).abs() < f64::EPSILON);
        assert!((snap.reuse_rate() - 0.0).abs() < f64::EPSILON);
        assert!((snap.transfer_efficiency() - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_basis_size() {
        let snap = sample_snapshot("s1");
        assert_eq!(snap.basis_size(), 30); // 15 + 2 + 13 + 0
    }

    #[test]
    fn test_transfer_efficiency() {
        // Pure T1: efficiency = 1.0
        let pure_t1 = BasisSnapshot {
            session: "t1".to_string(),
            t1_count: 10,
            t2_p_count: 0,
            t2_c_count: 0,
            t3_count: 0,
            reused: 10,
            total_needed: 10,
        };
        assert!((pure_t1.transfer_efficiency() - 1.0).abs() < f64::EPSILON);

        // Pure T3: efficiency = 0.4
        let pure_t3 = BasisSnapshot {
            session: "t3".to_string(),
            t1_count: 0,
            t2_p_count: 0,
            t2_c_count: 0,
            t3_count: 10,
            reused: 10,
            total_needed: 10,
        };
        assert!((pure_t3.transfer_efficiency() - 0.4).abs() < f64::EPSILON);

        // Mixed: (15*1.0 + 2*0.9 + 13*0.7) / 30 = (15 + 1.8 + 9.1) / 30 = 25.9/30
        let mixed = sample_snapshot("mix");
        let expected = (15.0 + 1.8 + 9.1) / 30.0;
        assert!((mixed.transfer_efficiency() - expected).abs() < 1e-10);
    }

    #[test]
    fn test_reuse_rate() {
        let snap = BasisSnapshot {
            session: "r".to_string(),
            t1_count: 5,
            t2_p_count: 0,
            t2_c_count: 0,
            t3_count: 0,
            reused: 5,
            total_needed: 10,
        };
        assert!((snap.reuse_rate() - 0.5).abs() < f64::EPSILON);

        // Zero total_needed
        let zero = BasisSnapshot {
            session: "z".to_string(),
            t1_count: 5,
            t2_p_count: 0,
            t2_c_count: 0,
            t3_count: 0,
            reused: 0,
            total_needed: 0,
        };
        assert!((zero.reuse_rate() - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_velocity_formula() {
        // V = B * eta * r = 30 * (25.9/30) * (28/30)
        let snap = sample_snapshot("v");
        let expected = 30.0 * snap.transfer_efficiency() * snap.reuse_rate();
        assert!((snap.velocity() - expected).abs() < 1e-10);
    }

    #[test]
    fn test_component_triple() {
        let snap = sample_snapshot("ct");
        let (b, eta, r) = snap.component_triple();
        assert!((b - 30.0).abs() < f64::EPSILON);
        assert!((eta - snap.transfer_efficiency()).abs() < f64::EPSILON);
        assert!((r - snap.reuse_rate()).abs() < f64::EPSILON);
    }

    #[test]
    fn test_tracker_record_chain() {
        let mut tracker = CompoundTracker::new();
        tracker
            .record(sample_snapshot("s1"))
            .record(sample_snapshot("s2"));
        assert_eq!(tracker.len(), 2);
        assert!(!tracker.is_empty());
    }

    #[test]
    fn test_velocity_history() {
        let mut tracker = CompoundTracker::new();
        tracker
            .record(growing_snapshot("s1", 2, 28, 30))
            .record(growing_snapshot("s2", 4, 30, 32))
            .record(growing_snapshot("s3", 6, 33, 35));
        let history = tracker.velocity_history();
        assert_eq!(history.len(), 3);
        // Each velocity should be positive with these inputs
        for v in &history {
            assert!(*v > 0.0);
        }
    }

    #[test]
    fn test_growth_rates() {
        let mut tracker = CompoundTracker::new();
        tracker
            .record(growing_snapshot("s1", 2, 28, 30))
            .record(growing_snapshot("s2", 4, 30, 32))
            .record(growing_snapshot("s3", 6, 33, 35));
        let rates = tracker.growth_rates();
        assert_eq!(rates.len(), 2);
        // Growth rates should be positive since velocity is increasing
        for r in &rates {
            assert!(*r > 0.0);
        }
    }

    #[test]
    fn test_avg_growth_geometric() {
        let mut tracker = CompoundTracker::new();
        tracker
            .record(growing_snapshot("s1", 2, 28, 30))
            .record(growing_snapshot("s2", 4, 30, 32))
            .record(growing_snapshot("s3", 6, 33, 35));
        let avg = tracker.avg_growth_rate();
        // Geometric mean of two positive rates should be > 0
        assert!(avg > 0.0);

        // Empty tracker returns 1.0
        let empty = CompoundTracker::new();
        assert!((empty.avg_growth_rate() - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_project_addition() {
        let mut tracker = CompoundTracker::new();
        tracker.record(sample_snapshot("s1"));

        let result = tracker.project_addition(Tier::T2Primitive, 3);
        assert!(result.is_some());
        let proj = result.unwrap_or_else(|| unreachable!());
        assert_eq!(proj.tier, "T2-P");
        assert_eq!(proj.count, 3);
        // Adding T2-P should increase velocity (higher efficiency than T2-C/T3)
        assert!(proj.velocity_gain > 0.0);
        assert!(proj.projected_velocity > proj.current_velocity);

        // Empty tracker returns None
        let empty = CompoundTracker::new();
        assert!(empty.project_addition(Tier::T1Universal, 1).is_none());
    }
}
