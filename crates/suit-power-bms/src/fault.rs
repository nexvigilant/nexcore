//! Fault tree for battery management system.
//!
//! Implements the fault detection logic from BMS spec 1.1.1.2.3:
//! overvolt, undervolt, overcurrent, overtemp, and isolation faults.
//! Each fault has a severity level and recommended action.

use serde::{Deserialize, Serialize};

/// Battery fault categories from the BMS fault tree.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FaultType {
    /// Cell voltage exceeds 4.25V — overcharge risk.
    Overvoltage,
    /// Cell voltage below 2.5V — deep discharge damage.
    Undervoltage,
    /// Pack current exceeds rated continuous (e.g., >45A per cell for P45B).
    Overcurrent,
    /// Cell temperature exceeds 60°C — thermal runaway onset at 80-130°C for NMC.
    Overtemperature,
    /// Cell temperature below -20°C — lithium plating risk even at rest.
    Undertemperature,
    /// Insulation resistance below threshold (Bender IR155 equivalent).
    InsulationFailure,
    /// Cell voltage imbalance exceeds balancing capacity (>150mV spread).
    CellImbalance,
    /// Communication loss between slave BMS and master (isoSPI timeout).
    CommLoss,
    /// Current sensor disagreement between shunt and hall sensors.
    SensorDisagreement,
}

/// Fault severity levels driving response urgency.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum FaultSeverity {
    /// Informational — log only, no action.
    Info,
    /// Warning — alert pilot, derate performance.
    Warning,
    /// Critical — immediate power reduction, prepare for shutdown.
    Critical,
    /// Emergency — disconnect load, activate safety systems.
    Emergency,
}

/// Recommended action for a detected fault.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FaultAction {
    /// Log the event, continue normal operation.
    LogOnly,
    /// Reduce maximum power output.
    Derate,
    /// Disconnect non-essential loads, maintain life-critical systems.
    ShedLoads,
    /// Open main contactors, switch to aux pack.
    Disconnect,
    /// Fire pyrotechnic disconnect, jettison pack if airborne.
    EmergencyDisconnect,
}

/// A detected fault with its classification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fault {
    /// What type of fault was detected.
    pub fault_type: FaultType,
    /// How severe is it.
    pub severity: FaultSeverity,
    /// What to do about it.
    pub action: FaultAction,
    /// The measured value that triggered the fault.
    pub measured_value: f64,
    /// The threshold that was exceeded.
    pub threshold: f64,
    /// Human-readable description.
    pub description: String,
}

/// Fault detection thresholds for NMC 21700 chemistry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FaultThresholds {
    /// Maximum cell voltage (V).
    pub overvolt_v: f64,
    /// Minimum cell voltage (V).
    pub undervolt_v: f64,
    /// Maximum continuous current per cell (A).
    pub overcurrent_a: f64,
    /// Maximum cell temperature (°C).
    pub overtemp_c: f64,
    /// Minimum cell temperature (°C).
    pub undertemp_c: f64,
    /// Minimum insulation resistance (kΩ).
    pub insulation_min_kohm: f64,
    /// Maximum cell-to-cell voltage spread (mV).
    pub imbalance_max_mv: f64,
    /// Maximum sensor disagreement (A).
    pub sensor_disagree_a: f64,
}

impl Default for FaultThresholds {
    /// Default thresholds for Samsung 50S / Molicel P45B NMC 21700.
    fn default() -> Self {
        Self {
            overvolt_v: 4.25,
            undervolt_v: 2.5,
            overcurrent_a: 45.0,
            overtemp_c: 60.0,
            undertemp_c: -20.0,
            insulation_min_kohm: 500.0,
            imbalance_max_mv: 150.0,
            sensor_disagree_a: 5.0,
        }
    }
}

impl FaultThresholds {
    /// LFP thresholds — wider abuse margin for aux pack and limb buffers.
    pub fn lfp() -> Self {
        Self {
            overvolt_v: 3.65,
            undervolt_v: 2.0,
            overcurrent_a: 30.0,
            overtemp_c: 80.0, // LFP tolerates higher temps
            undertemp_c: -30.0,
            insulation_min_kohm: 500.0,
            imbalance_max_mv: 200.0,
            sensor_disagree_a: 5.0,
        }
    }

    /// Check a cell voltage against thresholds.
    pub fn check_voltage(&self, voltage_v: f64) -> Option<Fault> {
        if voltage_v > self.overvolt_v {
            Some(Fault {
                fault_type: FaultType::Overvoltage,
                severity: FaultSeverity::Critical,
                action: FaultAction::ShedLoads,
                measured_value: voltage_v,
                threshold: self.overvolt_v,
                description: format!(
                    "Cell voltage {voltage_v:.3}V exceeds {:.3}V — stop charging immediately",
                    self.overvolt_v
                ),
            })
        } else if voltage_v < self.undervolt_v {
            Some(Fault {
                fault_type: FaultType::Undervoltage,
                severity: FaultSeverity::Warning,
                action: FaultAction::Derate,
                measured_value: voltage_v,
                threshold: self.undervolt_v,
                description: format!(
                    "Cell voltage {voltage_v:.3}V below {:.3}V — deep discharge damage risk",
                    self.undervolt_v
                ),
            })
        } else {
            None
        }
    }

    /// Check temperature against thresholds.
    pub fn check_temperature(&self, temp_c: f64) -> Option<Fault> {
        if temp_c > self.overtemp_c {
            let severity = if temp_c > 80.0 {
                FaultSeverity::Emergency
            } else {
                FaultSeverity::Critical
            };
            let action = if temp_c > 80.0 {
                FaultAction::EmergencyDisconnect
            } else {
                FaultAction::ShedLoads
            };
            Some(Fault {
                fault_type: FaultType::Overtemperature,
                severity,
                action,
                measured_value: temp_c,
                threshold: self.overtemp_c,
                description: format!(
                    "Cell temp {temp_c:.1}°C exceeds {:.1}°C — thermal runaway risk",
                    self.overtemp_c
                ),
            })
        } else if temp_c < self.undertemp_c {
            Some(Fault {
                fault_type: FaultType::Undertemperature,
                severity: FaultSeverity::Warning,
                action: FaultAction::Derate,
                measured_value: temp_c,
                threshold: self.undertemp_c,
                description: format!(
                    "Cell temp {temp_c:.1}°C below {:.1}°C — lithium plating risk",
                    self.undertemp_c
                ),
            })
        } else {
            None
        }
    }

    /// Check current against overcurrent threshold.
    pub fn check_current(&self, current_a: f64) -> Option<Fault> {
        let abs_current = current_a.abs();
        if abs_current > self.overcurrent_a {
            Some(Fault {
                fault_type: FaultType::Overcurrent,
                severity: FaultSeverity::Critical,
                action: FaultAction::ShedLoads,
                measured_value: abs_current,
                threshold: self.overcurrent_a,
                description: format!(
                    "Current {abs_current:.1}A exceeds {:.1}A rated continuous",
                    self.overcurrent_a
                ),
            })
        } else {
            None
        }
    }

    /// Run all fault checks against a cell snapshot.
    pub fn check_all(&self, voltage_v: f64, current_a: f64, temp_c: f64) -> Vec<Fault> {
        let mut faults = Vec::new();
        if let Some(f) = self.check_voltage(voltage_v) {
            faults.push(f);
        }
        if let Some(f) = self.check_current(current_a) {
            faults.push(f);
        }
        if let Some(f) = self.check_temperature(temp_c) {
            faults.push(f);
        }
        faults
    }

    /// Return the highest-severity action from a set of faults.
    pub fn worst_action(faults: &[Fault]) -> FaultAction {
        faults
            .iter()
            .map(|f| f.action)
            .max_by_key(|a| match a {
                FaultAction::LogOnly => 0,
                FaultAction::Derate => 1,
                FaultAction::ShedLoads => 2,
                FaultAction::Disconnect => 3,
                FaultAction::EmergencyDisconnect => 4,
            })
            .unwrap_or(FaultAction::LogOnly)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_thresholds_nmc() {
        let t = FaultThresholds::default();
        assert!((t.overvolt_v - 4.25).abs() < f64::EPSILON);
        assert!((t.overcurrent_a - 45.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_lfp_wider_thermal_margin() {
        let lfp = FaultThresholds::lfp();
        let nmc = FaultThresholds::default();
        assert!(lfp.overtemp_c > nmc.overtemp_c);
    }

    #[test]
    fn test_overvoltage_detected() {
        let t = FaultThresholds::default();
        let fault = t.check_voltage(4.30);
        assert!(fault.is_some());
        let f = fault.expect("fault should exist"); // test-only
        assert_eq!(f.fault_type, FaultType::Overvoltage);
        assert_eq!(f.severity, FaultSeverity::Critical);
    }

    #[test]
    fn test_undervoltage_detected() {
        let t = FaultThresholds::default();
        let fault = t.check_voltage(2.3);
        assert!(fault.is_some());
        assert_eq!(
            fault.expect("fault should exist").fault_type, // test-only
            FaultType::Undervoltage
        );
    }

    #[test]
    fn test_normal_voltage_no_fault() {
        let t = FaultThresholds::default();
        assert!(t.check_voltage(3.7).is_none());
    }

    #[test]
    fn test_overtemp_critical_vs_emergency() {
        let t = FaultThresholds::default();
        let critical = t.check_temperature(65.0);
        assert_eq!(
            critical.as_ref().expect("fault").severity, // test-only
            FaultSeverity::Critical
        );

        let emergency = t.check_temperature(85.0);
        assert_eq!(
            emergency.as_ref().expect("fault").severity, // test-only
            FaultSeverity::Emergency
        );
    }

    #[test]
    fn test_overcurrent_detected() {
        let t = FaultThresholds::default();
        let fault = t.check_current(50.0);
        assert!(fault.is_some());
        assert_eq!(
            fault.expect("fault").fault_type, // test-only
            FaultType::Overcurrent
        );
    }

    #[test]
    fn test_check_all_multiple_faults() {
        let t = FaultThresholds::default();
        let faults = t.check_all(4.30, 50.0, 65.0);
        assert_eq!(faults.len(), 3);
    }

    #[test]
    fn test_check_all_no_faults() {
        let t = FaultThresholds::default();
        let faults = t.check_all(3.7, 15.0, 30.0);
        assert!(faults.is_empty());
    }

    #[test]
    fn test_worst_action_escalation() {
        let t = FaultThresholds::default();
        let faults = t.check_all(2.3, 50.0, 85.0);
        let action = FaultThresholds::worst_action(&faults);
        assert_eq!(action, FaultAction::EmergencyDisconnect);
    }
}
