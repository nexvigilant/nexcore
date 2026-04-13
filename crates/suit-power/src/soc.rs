use serde::{Deserialize, Serialize};

/// State-of-Charge estimate for a battery pack.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocEstimate {
    /// Percentage SOC (0.0 to 100.0)
    pub soc: f32,
    /// Estimated remaining capacity (Ah)
    pub capacity: f32,
    /// Health of the battery pack (0.0 to 1.0)
    pub health: f32,
}

/// Estimator engine for managing battery health.
pub struct SocEstimator {
    /// Internal filter state for EKF or Coulomb counter.
    pub filter_state: f32,
}

impl SocEstimator {
    /// Update the SOC estimation with new current and voltage measurements.
    pub fn update(&mut self, _voltage: f32, _current: f32, _temp: f32) -> SocEstimate {
        // TODO: Implement EKF for state-of-charge tracking.
        SocEstimate {
            soc: 100.0,
            capacity: 50.0,
            health: 1.0,
        }
    }
}
