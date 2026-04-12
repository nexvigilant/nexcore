//! Thermal zone management — monitoring and cooling routing.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ThermalZone {
    BatteryPack,
    MotorController,
    PowerElectronics,
    PilotInterface,
    Ambient,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CoolingMethod {
    Passive,
    ActiveElevated,
    AggressiveCooling,
    Preheat,
    EmergencyDump,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ThermalAlert {
    Nominal,
    Info,
    Warning,
    Critical,
    Emergency,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThermalAction {
    pub zone: ThermalZone,
    pub cooling: CoolingMethod,
    pub derate_pct: u8,
    pub alert: ThermalAlert,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZoneReading {
    pub zone: ThermalZone,
    pub temperature_c: f64,
    pub coolant_available: bool,
}

impl ZoneReading {
    pub fn route(&self) -> ThermalAction {
        if self.temperature_c > 80.0 {
            return ThermalAction {
                zone: self.zone,
                cooling: CoolingMethod::EmergencyDump,
                derate_pct: 100,
                alert: ThermalAlert::Emergency,
            };
        }
        if self.temperature_c > 60.0 {
            let derate = match self.zone {
                ThermalZone::BatteryPack => 80,
                ThermalZone::PilotInterface => 50,
                _ => 60,
            };
            return ThermalAction {
                zone: self.zone,
                cooling: CoolingMethod::AggressiveCooling,
                derate_pct: derate,
                alert: ThermalAlert::Critical,
            };
        }
        if self.temperature_c > 40.0 {
            let (cooling, derate) = if self.coolant_available {
                (CoolingMethod::ActiveElevated, 0)
            } else {
                (CoolingMethod::Passive, 30)
            };
            return ThermalAction {
                zone: self.zone,
                cooling,
                derate_pct: derate,
                alert: ThermalAlert::Warning,
            };
        }
        if self.temperature_c < 0.0 {
            return ThermalAction {
                zone: self.zone,
                cooling: CoolingMethod::Preheat,
                derate_pct: 20,
                alert: ThermalAlert::Info,
            };
        }
        ThermalAction {
            zone: self.zone,
            cooling: CoolingMethod::Passive,
            derate_pct: 0,
            alert: ThermalAlert::Nominal,
        }
    }
}

/// Cooling loop — Q = mass_flow × Cp × ΔT.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoolingLoop {
    pub inlet_temp_c: f64,
    pub outlet_temp_c: f64,
    pub flow_rate_lpm: f64,
    pub pump_active: bool,
}

impl CoolingLoop {
    pub fn heat_removed_w(&self) -> f64 {
        if !self.pump_active {
            return 0.0;
        }
        let mass_flow = self.flow_rate_lpm * 1.05 / 60.0;
        mass_flow * 3400.0 * (self.outlet_temp_c - self.inlet_temp_c)
    }
}

/// Heat sink — ΔT = R_th × P.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeatSink {
    pub thermal_resistance_c_per_w: f64,
    pub surface_area_cm2: f64,
}

impl HeatSink {
    pub fn temp_rise_c(&self, power_w: f64) -> f64 {
        self.thermal_resistance_c_per_w * power_w
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_emergency() {
        let r = ZoneReading {
            zone: ThermalZone::MotorController,
            temperature_c: 85.0,
            coolant_available: true,
        };
        assert_eq!(r.route().alert, ThermalAlert::Emergency);
    }

    #[test]
    fn test_battery_critical() {
        let r = ZoneReading {
            zone: ThermalZone::BatteryPack,
            temperature_c: 65.0,
            coolant_available: true,
        };
        assert_eq!(r.route().derate_pct, 80);
    }

    #[test]
    fn test_pilot_critical() {
        let r = ZoneReading {
            zone: ThermalZone::PilotInterface,
            temperature_c: 62.0,
            coolant_available: true,
        };
        assert_eq!(r.route().derate_pct, 50);
    }

    #[test]
    fn test_warm_with_coolant() {
        let r = ZoneReading {
            zone: ThermalZone::MotorController,
            temperature_c: 50.0,
            coolant_available: true,
        };
        assert_eq!(r.route().derate_pct, 0);
    }

    #[test]
    fn test_warm_no_coolant() {
        let r = ZoneReading {
            zone: ThermalZone::MotorController,
            temperature_c: 50.0,
            coolant_available: false,
        };
        assert_eq!(r.route().derate_pct, 30);
    }

    #[test]
    fn test_cold() {
        let r = ZoneReading {
            zone: ThermalZone::BatteryPack,
            temperature_c: -10.0,
            coolant_available: false,
        };
        assert_eq!(r.route().cooling, CoolingMethod::Preheat);
    }

    #[test]
    fn test_nominal() {
        let r = ZoneReading {
            zone: ThermalZone::Ambient,
            temperature_c: 25.0,
            coolant_available: false,
        };
        assert_eq!(r.route().alert, ThermalAlert::Nominal);
    }

    #[test]
    fn test_cooling_loop() {
        let cl = CoolingLoop {
            inlet_temp_c: 25.0,
            outlet_temp_c: 35.0,
            flow_rate_lpm: 2.0,
            pump_active: true,
        };
        assert!(cl.heat_removed_w() > 1100.0);
    }

    #[test]
    fn test_cooling_pump_off() {
        let cl = CoolingLoop {
            inlet_temp_c: 25.0,
            outlet_temp_c: 35.0,
            flow_rate_lpm: 2.0,
            pump_active: false,
        };
        assert!(cl.heat_removed_w().abs() < f64::EPSILON);
    }

    #[test]
    fn test_heat_sink() {
        let hs = HeatSink {
            thermal_resistance_c_per_w: 0.5,
            surface_area_cm2: 100.0,
        };
        assert!((hs.temp_rise_c(50.0) - 25.0).abs() < f64::EPSILON);
    }
}
