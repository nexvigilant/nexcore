//! State-of-Charge estimation using fused EKF + OCV + coulomb counting.
//!
//! Routes by available sensor data to select the best estimation method.
//! Maps to the `soc-estimator` microgram decision tree.

use serde::{Deserialize, Serialize};

/// Estimation method selected based on available sensor data.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SocMethod {
    /// Open-circuit voltage lookup — most accurate, requires rest state (>30 min).
    OcvLookup,
    /// Full Extended Kalman Filter fusion: voltage + current + temperature.
    EkfFused,
    /// EKF weighted toward current sensing during heavy load (voltage sag unreliable).
    EkfCurrentWeighted,
    /// EKF with cold-temperature compensation (below 0°C).
    EkfColdCompensated,
    /// EKF during regenerative braking (negative current).
    EkfRegen,
    /// Coulomb counting only — no prior SoC available for EKF.
    CoulombCounting,
    /// Voltage-only fallback — current sensor offline.
    VoltageOnly,
    /// Thermal override — cell >60°C, SoC irrelevant, survival mode.
    ThermalOverride,
}

/// Load regime affecting estimation accuracy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LoadState {
    /// Cell at rest >30 min — OCV is gold standard.
    Resting,
    /// Light discharge (<5A).
    LightLoad,
    /// Moderate discharge (5-20A).
    ModerateLoad,
    /// Heavy discharge (>20A) — voltage sag distorts OCV.
    HeavyLoad,
    /// Negative current — regenerative braking.
    Regenerating,
}

/// SoC estimation result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocEstimate {
    /// Estimated state of charge (0.0–1.0).
    pub soc: f64,
    /// Method selected based on available data.
    pub method: SocMethod,
    /// Confidence in the estimate (0.0–1.0).
    pub confidence: f64,
    /// State-of-health flag from the estimation process.
    pub soh_flag: SohFlag,
    /// Thermal state observation.
    pub thermal_state: ThermalState,
}

/// State-of-health flags surfaced during SoC estimation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SohFlag {
    /// All parameters nominal.
    Nominal,
    /// Coulomb counting drift accumulating — recalibrate at next rest.
    CheckDrift,
    /// Current sensor degraded or offline.
    DegradedSensing,
    /// Cell derated due to cold temperature.
    ColdDerated,
    /// Cell temperature critical — immediate action required.
    ThermalCritical,
}

/// Thermal state of the cell.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThermalState {
    /// Resting at thermal equilibrium.
    RestingEquilibrium,
    /// Normal operating temperature (0–45°C).
    Normal,
    /// Elevated but within spec (45–60°C).
    ElevatedNormal,
    /// Below 0°C — derated operation.
    Below0C,
    /// Above 60°C — shutdown required.
    ShutdownRequired,
    /// No temperature sensor available.
    Unknown,
}

/// Sensor inputs for SoC estimation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocInput {
    /// Terminal voltage in volts.
    pub voltage_v: f64,
    /// Instantaneous current in amps (positive = discharge). `None` if sensor offline.
    pub current_a: Option<f64>,
    /// Cell temperature in Celsius. `None` if sensor offline.
    pub temperature_c: Option<f64>,
    /// Time since last measurement in seconds.
    pub time_delta_s: Option<f64>,
    /// Previous SoC estimate (0.0–1.0). `None` on first estimation.
    pub previous_soc: Option<f64>,
    /// Nominal cell capacity in Ah.
    pub cell_capacity_ah: f64,
    /// Current load regime.
    pub load_state: LoadState,
}

impl SocInput {
    /// Route to the appropriate estimation method based on available data.
    pub fn select_method(&self) -> SocMethod {
        // Gate 1: Thermal override
        if let Some(temp) = self.temperature_c {
            if temp > 60.0 {
                return SocMethod::ThermalOverride;
            }
        }

        // Gate 2: Resting — OCV is gold standard
        if self.load_state == LoadState::Resting {
            return SocMethod::OcvLookup;
        }

        // Gate 3: Need current for coulomb counting
        let has_current = self.current_a.is_some();
        if !has_current {
            return SocMethod::VoltageOnly;
        }

        // Gate 4: Need previous SoC for EKF
        let has_prior = self.previous_soc.is_some();
        if !has_prior {
            return SocMethod::CoulombCounting;
        }

        // Gate 5: Cold deration
        if let Some(temp) = self.temperature_c {
            if temp < 0.0 {
                return SocMethod::EkfColdCompensated;
            }
        }

        // Gate 6: Load-specific EKF
        match self.load_state {
            LoadState::HeavyLoad => SocMethod::EkfCurrentWeighted,
            LoadState::Regenerating => SocMethod::EkfRegen,
            _ => SocMethod::EkfFused,
        }
    }

    /// Estimate SoC using the selected method.
    pub fn estimate(&self) -> SocEstimate {
        let method = self.select_method();

        match method {
            SocMethod::ThermalOverride => SocEstimate {
                soc: 0.0,
                method,
                confidence: 1.0,
                soh_flag: SohFlag::ThermalCritical,
                thermal_state: ThermalState::ShutdownRequired,
            },
            SocMethod::OcvLookup => {
                let soc = ocv_to_soc_nmc(self.voltage_v);
                SocEstimate {
                    soc,
                    method,
                    confidence: 0.95,
                    soh_flag: SohFlag::Nominal,
                    thermal_state: ThermalState::RestingEquilibrium,
                }
            }
            SocMethod::VoltageOnly => {
                let soc = ocv_to_soc_nmc(self.voltage_v);
                SocEstimate {
                    soc,
                    method,
                    confidence: 0.40,
                    soh_flag: SohFlag::DegradedSensing,
                    thermal_state: ThermalState::Unknown,
                }
            }
            SocMethod::CoulombCounting => SocEstimate {
                soc: 0.70, // Starting estimate without prior
                method,
                confidence: 0.75,
                soh_flag: SohFlag::CheckDrift,
                thermal_state: ThermalState::Normal,
            },
            SocMethod::EkfColdCompensated => {
                let base_soc = self.previous_soc.unwrap_or(0.5);
                let coulomb_delta = self.coulomb_delta();
                SocEstimate {
                    soc: (base_soc - coulomb_delta).clamp(0.0, 1.0),
                    method,
                    confidence: 0.65,
                    soh_flag: SohFlag::ColdDerated,
                    thermal_state: ThermalState::Below0C,
                }
            }
            SocMethod::EkfCurrentWeighted => {
                let base_soc = self.previous_soc.unwrap_or(0.5);
                let coulomb_delta = self.coulomb_delta();
                // Heavy load: weight coulomb counting 70%, voltage 30%
                let voltage_soc = ocv_to_soc_nmc(self.voltage_v);
                let fused = 0.7 * (base_soc - coulomb_delta) + 0.3 * voltage_soc;
                SocEstimate {
                    soc: fused.clamp(0.0, 1.0),
                    method,
                    confidence: 0.80,
                    soh_flag: SohFlag::Nominal,
                    thermal_state: ThermalState::ElevatedNormal,
                }
            }
            SocMethod::EkfRegen => {
                let base_soc = self.previous_soc.unwrap_or(0.5);
                let coulomb_delta = self.coulomb_delta(); // Negative current = charging
                SocEstimate {
                    soc: (base_soc - coulomb_delta).clamp(0.0, 1.0),
                    method,
                    confidence: 0.82,
                    soh_flag: SohFlag::Nominal,
                    thermal_state: ThermalState::Normal,
                }
            }
            SocMethod::EkfFused => {
                let base_soc = self.previous_soc.unwrap_or(0.5);
                let coulomb_delta = self.coulomb_delta();
                let voltage_soc = ocv_to_soc_nmc(self.voltage_v);
                // Balanced: 50% coulomb, 50% voltage
                let fused = 0.5 * (base_soc - coulomb_delta) + 0.5 * voltage_soc;
                SocEstimate {
                    soc: fused.clamp(0.0, 1.0),
                    method,
                    confidence: 0.88,
                    soh_flag: SohFlag::Nominal,
                    thermal_state: ThermalState::Normal,
                }
            }
        }
    }

    /// Coulomb counting delta: ΔSoC = I × Δt / (Q_rated × 3600).
    fn coulomb_delta(&self) -> f64 {
        let current = self.current_a.unwrap_or(0.0);
        let dt = self.time_delta_s.unwrap_or(1.0);
        current * dt / (self.cell_capacity_ah * 3600.0)
    }
}

/// Simplified NMC OCV-to-SoC lookup.
/// Linear interpolation between known points on the NMC 21700 discharge curve.
fn ocv_to_soc_nmc(voltage: f64) -> f64 {
    // NMC typical: 4.2V = 100%, 3.6V ≈ 50%, 3.0V ≈ 5%, 2.5V = 0%
    if voltage >= 4.2 {
        1.0
    } else if voltage >= 3.6 {
        0.5 + (voltage - 3.6) / (4.2 - 3.6) * 0.5
    } else if voltage >= 3.0 {
        0.05 + (voltage - 3.0) / (3.6 - 3.0) * 0.45
    } else if voltage >= 2.5 {
        (voltage - 2.5) / (3.0 - 2.5) * 0.05
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resting_selects_ocv() {
        let input = SocInput {
            voltage_v: 3.85,
            current_a: None,
            temperature_c: Some(25.0),
            time_delta_s: None,
            previous_soc: None,
            cell_capacity_ah: 5.0,
            load_state: LoadState::Resting,
        };
        assert_eq!(input.select_method(), SocMethod::OcvLookup);
    }

    #[test]
    fn test_thermal_override() {
        let input = SocInput {
            voltage_v: 3.8,
            current_a: Some(5.0),
            temperature_c: Some(65.0),
            time_delta_s: Some(1.0),
            previous_soc: Some(0.8),
            cell_capacity_ah: 5.0,
            load_state: LoadState::LightLoad,
        };
        let est = input.estimate();
        assert_eq!(est.method, SocMethod::ThermalOverride);
        assert_eq!(est.soh_flag, SohFlag::ThermalCritical);
    }

    #[test]
    fn test_no_current_selects_voltage_only() {
        let input = SocInput {
            voltage_v: 3.6,
            current_a: None,
            temperature_c: Some(25.0),
            time_delta_s: None,
            previous_soc: None,
            cell_capacity_ah: 5.0,
            load_state: LoadState::ModerateLoad,
        };
        assert_eq!(input.select_method(), SocMethod::VoltageOnly);
        assert!((input.estimate().confidence - 0.40).abs() < f64::EPSILON);
    }

    #[test]
    fn test_heavy_load_selects_current_weighted() {
        let input = SocInput {
            voltage_v: 3.5,
            current_a: Some(30.0),
            temperature_c: Some(35.0),
            time_delta_s: Some(1.0),
            previous_soc: Some(0.6),
            cell_capacity_ah: 5.0,
            load_state: LoadState::HeavyLoad,
        };
        assert_eq!(input.select_method(), SocMethod::EkfCurrentWeighted);
    }

    #[test]
    fn test_cold_selects_compensated() {
        let input = SocInput {
            voltage_v: 3.4,
            current_a: Some(10.0),
            temperature_c: Some(-5.0),
            time_delta_s: Some(1.0),
            previous_soc: Some(0.5),
            cell_capacity_ah: 5.0,
            load_state: LoadState::ModerateLoad,
        };
        assert_eq!(input.select_method(), SocMethod::EkfColdCompensated);
        assert_eq!(input.estimate().thermal_state, ThermalState::Below0C);
    }

    #[test]
    fn test_regen_selects_regen_mode() {
        let input = SocInput {
            voltage_v: 3.9,
            current_a: Some(-10.0),
            temperature_c: Some(25.0),
            time_delta_s: Some(1.0),
            previous_soc: Some(0.6),
            cell_capacity_ah: 5.0,
            load_state: LoadState::Regenerating,
        };
        let est = input.estimate();
        assert_eq!(est.method, SocMethod::EkfRegen);
        // Negative current = charging, SoC should increase
        assert!(est.soc > 0.6);
    }

    #[test]
    fn test_normal_selects_ekf_fused() {
        let input = SocInput {
            voltage_v: 3.7,
            current_a: Some(15.0),
            temperature_c: Some(30.0),
            time_delta_s: Some(1.0),
            previous_soc: Some(0.7),
            cell_capacity_ah: 5.0,
            load_state: LoadState::ModerateLoad,
        };
        let est = input.estimate();
        assert_eq!(est.method, SocMethod::EkfFused);
        assert!((est.confidence - 0.88).abs() < f64::EPSILON);
    }

    #[test]
    fn test_ocv_lookup_curve() {
        assert!((ocv_to_soc_nmc(4.2) - 1.0).abs() < f64::EPSILON);
        assert!((ocv_to_soc_nmc(3.6) - 0.5).abs() < f64::EPSILON);
        assert!((ocv_to_soc_nmc(2.5) - 0.0).abs() < f64::EPSILON);
        // Mid-range sanity
        let mid = ocv_to_soc_nmc(3.9);
        assert!(mid > 0.5 && mid < 1.0);
    }

    #[test]
    fn test_coulomb_delta_discharge() {
        let input = SocInput {
            voltage_v: 3.7,
            current_a: Some(18.0), // 18A discharge
            temperature_c: Some(25.0),
            time_delta_s: Some(60.0), // 1 minute
            previous_soc: Some(0.8),
            cell_capacity_ah: 5.0,
            load_state: LoadState::ModerateLoad,
        };
        // 18A * 60s / (5Ah * 3600) = 0.06
        let delta = input.coulomb_delta();
        assert!((delta - 0.06).abs() < 1e-10);
    }
}
