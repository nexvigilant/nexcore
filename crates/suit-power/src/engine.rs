//! # Power Management Engine
//!
//! Centralized engine to orchestrate SOC estimation, load prioritization,
//! thermal derating, and fault recovery.

use crate::degradation::DegradationSequencer;
use crate::load::LoadPrioritizer;
use crate::load::LoadShedCommand;
use crate::mission::MissionForecast;
use crate::soc::SocEstimate;
use crate::soc::SocEstimator;

/// The central power management engine for the suit.
pub struct PowerEngine {
    /// State-of-Charge estimator.
    pub soc: SocEstimator,
    /// Load prioritization system.
    pub prioritizer: LoadPrioritizer,
    /// Failure degradation state machine.
    pub sequencer: DegradationSequencer,
}

impl PowerEngine {
    /// Create a new power management engine.
    pub fn new() -> Self {
        Self {
            soc: SocEstimator { filter_state: 0.0 },
            prioritizer: LoadPrioritizer {
                current_tier: crate::load::LoadTier::Comms,
            },
            sequencer: crate::degradation::DegradationSequencer {
                state: crate::degradation::PowerState::Nominal,
            },
        }
    }

    /// Primary power control loop (to be called every tick).
    pub fn update(
        &mut self,
        voltage: f32,
        current: f32,
        temp: f32,
        power_available: f32,
        _forecast: &MissionForecast,
    ) -> (SocEstimate, LoadShedCommand) {
        // 1. Update State of Charge
        let soc_est = self.soc.update(voltage, current, temp);

        // 2. Evaluate Load Shedding
        let load_command = self.prioritizer.evaluate_load(power_available);

        // 3. Check for Thermal/Safety Violations
        if temp > 80.0 {
            self.sequencer.transition(
                crate::degradation::PowerState::Caution,
                "High thermal stress",
            );
        }

        // Return system state
        (soc_est, load_command)
    }
}
