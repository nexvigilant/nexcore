//! Motor fault detection.
//!
//! Maps to `motor-fault-detector` microgram.

use serde::{Deserialize, Serialize};

/// Motor fault types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MotorFault {
    None,
    WindingOvertempCritical,
    WindingOvertempWarning,
    OvercurrentCritical,
    OvercurrentWarning,
    EncoderLoss,
    Stall,
    PhaseImbalance,
}

/// Fault severity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum MotorFaultSeverity {
    Nominal,
    Warning,
    Critical,
    Emergency,
}

/// Motor sensor snapshot.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MotorSensors {
    pub phase_current_a: f64,
    pub rated_current_a: f64,
    pub winding_temp_c: f64,
    pub rpm: f64,
    pub encoder_valid: bool,
    pub phase_balance_pct: f64,
}

/// Motor fault assessment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MotorFaultAssessment {
    pub fault: MotorFault,
    pub severity: MotorFaultSeverity,
}

impl MotorSensors {
    /// Detect faults from sensor readings.
    pub fn detect_fault(&self) -> MotorFaultAssessment {
        // Winding overtemp — most urgent
        if self.winding_temp_c > 150.0 {
            return MotorFaultAssessment {
                fault: MotorFault::WindingOvertempCritical,
                severity: MotorFaultSeverity::Emergency,
            };
        }
        if self.winding_temp_c > 120.0 {
            return MotorFaultAssessment {
                fault: MotorFault::WindingOvertempWarning,
                severity: MotorFaultSeverity::Warning,
            };
        }

        // Overcurrent
        if self.phase_current_a > self.rated_current_a * 2.0 {
            return MotorFaultAssessment {
                fault: MotorFault::OvercurrentCritical,
                severity: MotorFaultSeverity::Critical,
            };
        }
        if self.phase_current_a > self.rated_current_a * 1.5 {
            return MotorFaultAssessment {
                fault: MotorFault::OvercurrentWarning,
                severity: MotorFaultSeverity::Warning,
            };
        }

        // Encoder loss
        if !self.encoder_valid {
            return MotorFaultAssessment {
                fault: MotorFault::EncoderLoss,
                severity: MotorFaultSeverity::Critical,
            };
        }

        // Stall
        if self.rpm < 10.0 && self.phase_current_a > self.rated_current_a * 0.75 {
            return MotorFaultAssessment {
                fault: MotorFault::Stall,
                severity: MotorFaultSeverity::Critical,
            };
        }

        // Phase imbalance
        if self.phase_balance_pct < 85.0 {
            return MotorFaultAssessment {
                fault: MotorFault::PhaseImbalance,
                severity: MotorFaultSeverity::Warning,
            };
        }

        MotorFaultAssessment {
            fault: MotorFault::None,
            severity: MotorFaultSeverity::Nominal,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn nominal() -> MotorSensors {
        MotorSensors {
            phase_current_a: 15.0,
            rated_current_a: 20.0,
            winding_temp_c: 60.0,
            rpm: 500.0,
            encoder_valid: true,
            phase_balance_pct: 95.0,
        }
    }

    #[test]
    fn test_nominal() {
        assert_eq!(nominal().detect_fault().fault, MotorFault::None);
    }

    #[test]
    fn test_winding_emergency() {
        let mut s = nominal();
        s.winding_temp_c = 160.0;
        assert_eq!(s.detect_fault().severity, MotorFaultSeverity::Emergency);
    }

    #[test]
    fn test_overcurrent_critical() {
        let mut s = nominal();
        s.phase_current_a = 45.0;
        assert_eq!(s.detect_fault().fault, MotorFault::OvercurrentCritical);
    }

    #[test]
    fn test_encoder_loss() {
        let mut s = nominal();
        s.encoder_valid = false;
        assert_eq!(s.detect_fault().fault, MotorFault::EncoderLoss);
    }

    #[test]
    fn test_stall() {
        let mut s = nominal();
        s.rpm = 2.0;
        s.phase_current_a = 18.0;
        assert_eq!(s.detect_fault().fault, MotorFault::Stall);
    }

    #[test]
    fn test_phase_imbalance() {
        let mut s = nominal();
        s.phase_balance_pct = 75.0;
        assert_eq!(s.detect_fault().fault, MotorFault::PhaseImbalance);
    }
}
