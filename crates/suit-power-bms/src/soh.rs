//! State-of-Health tracking — capacity fade and internal resistance growth.
//!
//! SoH degrades over cycle life. NMC 21700 targets 500+ cycles at 80% DoD
//! before reaching 80% capacity retention.

use serde::{Deserialize, Serialize};

/// State-of-Health estimate.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SohEstimate {
    /// Capacity retention (1.0 = new, 0.8 = end of life for most applications).
    pub capacity_retention: f64,
    /// Internal resistance growth factor (1.0 = new, >1.5 = degraded).
    pub resistance_growth: f64,
    /// Estimated remaining useful cycles.
    pub remaining_cycles: u32,
    /// Health grade.
    pub grade: SohGrade,
}

/// SoH grade classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SohGrade {
    /// >90% capacity — like new.
    Excellent,
    /// 80-90% capacity — normal aging.
    Good,
    /// 70-80% capacity — replace soon.
    Fair,
    /// <70% capacity — replace immediately.
    Poor,
}

/// SoH tracker that accumulates cycle data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SohTracker {
    /// Total full-equivalent cycles completed.
    pub total_cycles: u32,
    /// Rated cycle life at 80% DoD (from cell datasheet).
    pub rated_cycle_life: u32,
    /// Initial capacity in Ah.
    pub initial_capacity_ah: f64,
    /// Most recent measured capacity in Ah.
    pub measured_capacity_ah: f64,
    /// Initial internal resistance in mΩ.
    pub initial_resistance_mohm: f64,
    /// Most recent measured internal resistance in mΩ.
    pub measured_resistance_mohm: f64,
}

impl SohTracker {
    /// Create a new tracker for a fresh cell.
    pub fn new(capacity_ah: f64, resistance_mohm: f64, rated_cycles: u32) -> Self {
        Self {
            total_cycles: 0,
            rated_cycle_life: rated_cycles,
            initial_capacity_ah: capacity_ah,
            measured_capacity_ah: capacity_ah,
            initial_resistance_mohm: resistance_mohm,
            measured_resistance_mohm: resistance_mohm,
        }
    }

    /// NMC 21700 defaults (Samsung 50S / Molicel P45B class).
    pub fn nmc_21700() -> Self {
        Self::new(5.0, 25.0, 500)
    }

    /// LFP 18650 defaults (aux pack / limb buffers).
    pub fn lfp_18650() -> Self {
        Self::new(1.5, 40.0, 2000)
    }

    /// Record a completed charge cycle.
    pub fn record_cycle(&mut self, measured_capacity_ah: f64, measured_resistance_mohm: f64) {
        self.total_cycles += 1;
        self.measured_capacity_ah = measured_capacity_ah;
        self.measured_resistance_mohm = measured_resistance_mohm;
    }

    /// Compute current SoH estimate.
    pub fn estimate(&self) -> SohEstimate {
        let retention = if self.initial_capacity_ah > 0.0 {
            self.measured_capacity_ah / self.initial_capacity_ah
        } else {
            1.0
        };

        let r_growth = if self.initial_resistance_mohm > 0.0 {
            self.measured_resistance_mohm / self.initial_resistance_mohm
        } else {
            1.0
        };

        let remaining = self.rated_cycle_life.saturating_sub(self.total_cycles);

        let grade = if retention > 0.9 {
            SohGrade::Excellent
        } else if retention > 0.8 {
            SohGrade::Good
        } else if retention > 0.7 {
            SohGrade::Fair
        } else {
            SohGrade::Poor
        };

        SohEstimate {
            capacity_retention: retention,
            resistance_growth: r_growth,
            remaining_cycles: remaining,
            grade,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_cell_excellent() {
        let tracker = SohTracker::nmc_21700();
        let est = tracker.estimate();
        assert_eq!(est.grade, SohGrade::Excellent);
        assert!((est.capacity_retention - 1.0).abs() < f64::EPSILON);
        assert_eq!(est.remaining_cycles, 500);
    }

    #[test]
    fn test_aged_cell_good() {
        let mut tracker = SohTracker::nmc_21700();
        for _ in 0..300 {
            tracker.record_cycle(4.3, 32.0);
        }
        let est = tracker.estimate();
        assert_eq!(est.grade, SohGrade::Good);
        assert_eq!(est.remaining_cycles, 200);
    }

    #[test]
    fn test_worn_cell_fair() {
        let mut tracker = SohTracker::nmc_21700();
        tracker.record_cycle(3.6, 45.0); // 72% retention
        tracker.total_cycles = 450;
        let est = tracker.estimate();
        assert_eq!(est.grade, SohGrade::Fair);
    }

    #[test]
    fn test_dead_cell_poor() {
        let mut tracker = SohTracker::nmc_21700();
        tracker.record_cycle(3.0, 60.0); // 60% retention
        tracker.total_cycles = 600;
        let est = tracker.estimate();
        assert_eq!(est.grade, SohGrade::Poor);
        assert_eq!(est.remaining_cycles, 0);
    }

    #[test]
    fn test_lfp_longer_life() {
        let tracker = SohTracker::lfp_18650();
        assert_eq!(tracker.rated_cycle_life, 2000);
        assert_eq!(tracker.estimate().remaining_cycles, 2000);
    }

    #[test]
    fn test_resistance_growth_tracked() {
        let mut tracker = SohTracker::nmc_21700();
        tracker.record_cycle(4.5, 37.5); // 50% R growth
        let est = tracker.estimate();
        assert!((est.resistance_growth - 1.5).abs() < f64::EPSILON);
    }
}
