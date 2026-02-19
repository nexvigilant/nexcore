//! Michaelis-Menten Equation → Capacity Efficiency
//!
//! Chemistry: v = Vmax × [S] / (Km + [S])
//! Capability: Throughput = Max_capacity × Demand / (Half_saturation + Demand)
//!
//! Models diminishing returns as capacity approaches saturation.

use super::types::NormalizedScore;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Saturation point (Km equivalent)
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct SaturationPoint(f64);

impl SaturationPoint {
    #[must_use]
    pub fn new(value: f64) -> Self {
        Self(value.max(0.01)) // Prevent division by zero
    }

    #[must_use]
    pub const fn value(&self) -> f64 {
        self.0
    }
}

/// Capacity efficiency via Michaelis-Menten kinetics
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct CapacityEfficiency {
    /// Current throughput
    pub throughput: f64,
    /// Maximum theoretical capacity
    pub max_capacity: f64,
    /// Current demand/load
    pub demand: f64,
    /// Half-saturation point
    pub half_saturation: SaturationPoint,
    /// Utilization ratio (demand / max_capacity)
    pub utilization: f64,
    /// Efficiency normalized [0, 1]
    pub normalized: NormalizedScore,
    /// Operating zone
    pub zone: OperatingZone,
}

impl CapacityEfficiency {
    /// Calculate capacity efficiency using Michaelis-Menten model
    ///
    /// # Arguments
    /// * `max_capacity` - Maximum theoretical throughput (Vmax)
    /// * `demand` - Current workload/requests ([S])
    /// * `half_saturation` - Demand at 50% capacity (Km)
    #[must_use]
    pub fn calculate(max_capacity: f64, demand: f64, half_saturation: f64) -> Self {
        let max_capacity = max_capacity.max(1.0);
        let demand = demand.max(0.0);
        let half_saturation = SaturationPoint::new(half_saturation);

        // Michaelis-Menten: v = Vmax × [S] / (Km + [S])
        let throughput = max_capacity * demand / (half_saturation.value() + demand);
        let utilization = demand / max_capacity;

        // Efficiency = how well we're converting demand to throughput
        // At low utilization: efficiency ≈ 1.0 (linear)
        // At high utilization: efficiency drops (diminishing returns)
        let theoretical_linear = (demand / max_capacity).min(1.0) * max_capacity;
        let efficiency = if theoretical_linear > 0.01 {
            throughput / theoretical_linear
        } else {
            1.0
        };

        let normalized = NormalizedScore::new(efficiency);
        let zone = OperatingZone::from_utilization(utilization);

        Self {
            throughput,
            max_capacity,
            demand,
            half_saturation,
            utilization,
            normalized,
            zone,
        }
    }

    /// Marginal efficiency (derivative) - how much more output per unit input
    #[must_use]
    pub fn marginal_efficiency(&self) -> f64 {
        // dv/d[S] = Vmax × Km / (Km + [S])²
        let km = self.half_saturation.value();
        let s = self.demand;
        self.max_capacity * km / (km + s).powi(2)
    }

    /// Headroom remaining before saturation
    #[must_use]
    pub fn headroom(&self) -> f64 {
        (self.max_capacity - self.throughput).max(0.0)
    }

    /// Recommendation based on operating zone
    #[must_use]
    pub fn recommendation(&self) -> &'static str {
        match self.zone {
            OperatingZone::Underutilized => "Add more work to increase efficiency",
            OperatingZone::Optimal => "Maintain current load",
            OperatingZone::Saturating => "Monitor closely, prepare to scale",
            OperatingZone::Saturated => "Scale capacity immediately",
        }
    }
}

impl fmt::Display for CapacityEfficiency {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Capacity: {:.0}/{:.0} ({:.0}% util, {} zone)",
            self.throughput,
            self.max_capacity,
            self.utilization * 100.0,
            self.zone
        )
    }
}

/// Operating zone based on utilization
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OperatingZone {
    /// < 50% utilization - linear scaling available
    Underutilized,
    /// 50-80% utilization - optimal efficiency
    Optimal,
    /// 80-95% utilization - diminishing returns
    Saturating,
    /// > 95% utilization - at capacity
    Saturated,
}

impl OperatingZone {
    #[must_use]
    pub fn from_utilization(util: f64) -> Self {
        match util {
            u if u < 0.50 => Self::Underutilized,
            u if u < 0.80 => Self::Optimal,
            u if u < 0.95 => Self::Saturating,
            _ => Self::Saturated,
        }
    }
}

impl fmt::Display for OperatingZone {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Underutilized => write!(f, "Underutilized"),
            Self::Optimal => write!(f, "Optimal"),
            Self::Saturating => write!(f, "Saturating"),
            Self::Saturated => write!(f, "Saturated"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_low_utilization() {
        let eff = CapacityEfficiency::calculate(100.0, 30.0, 50.0);
        assert_eq!(eff.zone, OperatingZone::Underutilized);
        assert!(eff.marginal_efficiency() > 0.5);
    }

    #[test]
    fn test_optimal_utilization() {
        let eff = CapacityEfficiency::calculate(100.0, 70.0, 50.0);
        assert_eq!(eff.zone, OperatingZone::Optimal);
    }

    #[test]
    fn test_saturated() {
        let eff = CapacityEfficiency::calculate(100.0, 200.0, 50.0);
        assert_eq!(eff.zone, OperatingZone::Saturated);
        assert!(eff.marginal_efficiency() < 0.2);
    }

    #[test]
    fn test_half_saturation_meaning() {
        // At demand = Km, throughput should be Vmax/2
        let eff = CapacityEfficiency::calculate(100.0, 50.0, 50.0);
        assert!((eff.throughput - 50.0).abs() < 0.1);
    }
}
