//! Thermal runaway detection — 3-stage NMC model.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum RunawayStage {
    Normal,
    PrecursorFastRise,
    PropagationWarning,
    Stage1Onset,
    Stage1WithGas,
    Stage1InternalShort,
    Stage2Acceleration,
    Stage3Propagation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum PropagationRisk {
    None,
    Low,
    Moderate,
    High,
    Certain,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RunawayAction {
    Monitor,
    CoolAndDerate,
    DisconnectModule,
    PyroDisconnect,
    EmergencyJettison,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunawayAssessment {
    pub stage: RunawayStage,
    pub time_to_next_s: Option<f64>,
    pub propagation_risk: PropagationRisk,
    pub action: RunawayAction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CellSensors {
    pub cell_temp_c: f64,
    pub rise_rate_c_per_min: Option<f64>,
    pub gas_detected: bool,
    pub voltage_dropping: bool,
    pub adjacent_cell_temp_c: Option<f64>,
}

impl CellSensors {
    pub fn assess(&self) -> RunawayAssessment {
        if self.cell_temp_c > 200.0 {
            return RunawayAssessment {
                stage: RunawayStage::Stage3Propagation,
                time_to_next_s: Some(0.0),
                propagation_risk: PropagationRisk::Certain,
                action: RunawayAction::EmergencyJettison,
            };
        }
        if self.cell_temp_c > 130.0 {
            return RunawayAssessment {
                stage: RunawayStage::Stage2Acceleration,
                time_to_next_s: Some(60.0),
                propagation_risk: PropagationRisk::High,
                action: RunawayAction::PyroDisconnect,
            };
        }
        if self.cell_temp_c > 80.0 {
            if self.gas_detected {
                return RunawayAssessment {
                    stage: RunawayStage::Stage1WithGas,
                    time_to_next_s: Some(300.0),
                    propagation_risk: PropagationRisk::Moderate,
                    action: RunawayAction::DisconnectModule,
                };
            }
            if self.voltage_dropping {
                return RunawayAssessment {
                    stage: RunawayStage::Stage1InternalShort,
                    time_to_next_s: Some(120.0),
                    propagation_risk: PropagationRisk::High,
                    action: RunawayAction::DisconnectModule,
                };
            }
            return RunawayAssessment {
                stage: RunawayStage::Stage1Onset,
                time_to_next_s: Some(900.0),
                propagation_risk: PropagationRisk::Low,
                action: RunawayAction::CoolAndDerate,
            };
        }
        if let Some(rate) = self.rise_rate_c_per_min {
            if rate > 5.0 {
                return RunawayAssessment {
                    stage: RunawayStage::PrecursorFastRise,
                    time_to_next_s: Some(((80.0 - self.cell_temp_c) / rate * 60.0).max(0.0)),
                    propagation_risk: PropagationRisk::Low,
                    action: RunawayAction::CoolAndDerate,
                };
            }
        }
        if let Some(adj) = self.adjacent_cell_temp_c {
            if adj > 60.0 {
                return RunawayAssessment {
                    stage: RunawayStage::PropagationWarning,
                    time_to_next_s: None,
                    propagation_risk: PropagationRisk::Moderate,
                    action: RunawayAction::CoolAndDerate,
                };
            }
        }
        RunawayAssessment {
            stage: RunawayStage::Normal,
            time_to_next_s: None,
            propagation_risk: PropagationRisk::None,
            action: RunawayAction::Monitor,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn s(temp: f64) -> CellSensors {
        CellSensors {
            cell_temp_c: temp,
            rise_rate_c_per_min: None,
            gas_detected: false,
            voltage_dropping: false,
            adjacent_cell_temp_c: None,
        }
    }

    #[test]
    fn test_stage3() {
        assert_eq!(s(220.0).assess().stage, RunawayStage::Stage3Propagation);
    }
    #[test]
    fn test_stage2() {
        assert_eq!(s(140.0).assess().stage, RunawayStage::Stage2Acceleration);
    }
    #[test]
    fn test_stage1_gas() {
        let mut c = s(90.0);
        c.gas_detected = true;
        assert_eq!(c.assess().stage, RunawayStage::Stage1WithGas);
    }
    #[test]
    fn test_stage1_short() {
        let mut c = s(85.0);
        c.voltage_dropping = true;
        assert_eq!(c.assess().stage, RunawayStage::Stage1InternalShort);
    }
    #[test]
    fn test_stage1_onset() {
        assert_eq!(s(85.0).assess().stage, RunawayStage::Stage1Onset);
    }
    #[test]
    fn test_precursor() {
        let mut c = s(55.0);
        c.rise_rate_c_per_min = Some(8.0);
        assert_eq!(c.assess().stage, RunawayStage::PrecursorFastRise);
    }
    #[test]
    fn test_propagation() {
        let mut c = s(35.0);
        c.adjacent_cell_temp_c = Some(65.0);
        assert_eq!(c.assess().stage, RunawayStage::PropagationWarning);
    }
    #[test]
    fn test_normal() {
        assert_eq!(s(30.0).assess().stage, RunawayStage::Normal);
    }
}
