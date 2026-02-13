//! Heart — central pump with dual circulation.
//!
//! ## Biological Model
//!
//! The heart collects data (venous return), enriches it (oxygenation),
//! and distributes it (arterial output). Two circuits:
//! - **Pulmonary**: validation/refresh (like type checking before use)
//! - **Systemic**: delivery to consumers
//!
//! ## Frank-Starling Law
//!
//! `stroke_volume = f(venous_return)` — throughput auto-scales with input.
//! More input → more output, up to a ceiling. This prevents backpressure
//! buildup without explicit scaling configuration.
//!
//! ## T1 Grounding: → (Causality) dominant
//!
//! The heart IS a causal engine: input causes processing causes output.
//! pump() = collect → enrich → distribute (sequential causality chain).

use serde::{Deserialize, Serialize};

use crate::types::{Circuit, FlowRate, HeartRate, Pressure, Resistance, Volume};

/// Heart configuration — the pump parameters.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeartConfig {
    /// Maximum stroke volume (items per beat)
    pub max_stroke_volume: f64,
    /// Frank-Starling compliance (how much SV increases with preload)
    pub compliance: f64,
    /// Resting heart rate
    pub resting_rate: HeartRate,
    /// Maximum heart rate under load
    pub max_rate: HeartRate,
}

impl Default for HeartConfig {
    fn default() -> Self {
        Self {
            max_stroke_volume: 120.0, // ~120 mL
            compliance: 0.7,          // 70% responsiveness to preload
            resting_rate: HeartRate::new(72.0),
            max_rate: HeartRate::new(180.0),
        }
    }
}

/// The heart — central pump.
///
/// # Tier: T3 (→ · σ · ν · ∂ · ς · N)
///
/// Causality-dominant: the heart IS the causal engine that drives
/// all circulation. Every pump cycle causes data movement.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Heart {
    /// Configuration
    pub config: HeartConfig,
    /// Current heart rate
    pub rate: HeartRate,
    /// Current stroke volume
    pub stroke_volume: f64,
    /// Beat counter
    pub beat_count: u64,
    /// Current circuit being pumped
    pub active_circuit: Circuit,
}

impl Default for Heart {
    fn default() -> Self {
        Self {
            config: HeartConfig::default(),
            rate: HeartRate::default(),
            stroke_volume: 70.0, // ~70 mL resting
            beat_count: 0,
            active_circuit: Circuit::Systemic,
        }
    }
}

impl Heart {
    /// Create a new heart with default configuration.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a heart with custom configuration.
    pub fn with_config(config: HeartConfig) -> Self {
        Self {
            rate: config.resting_rate,
            config,
            ..Self::default()
        }
    }

    /// **Frank-Starling Law**: compute stroke volume from venous return (preload).
    ///
    /// `SV = min(max_sv, preload × compliance)`
    ///
    /// More venous return → more stretch → stronger contraction → more output.
    /// Capped at max_stroke_volume (the heart has finite capacity).
    pub fn frank_starling(&self, venous_return: f64) -> f64 {
        let sv = venous_return * self.config.compliance;
        if sv > self.config.max_stroke_volume {
            self.config.max_stroke_volume
        } else if sv < 0.0 {
            0.0
        } else {
            sv
        }
    }

    /// **Cardiac Output**: CO = stroke_volume × heart_rate.
    ///
    /// Maps to: total_throughput = items_per_cycle × cycles_per_second
    pub fn cardiac_output(&self) -> FlowRate {
        FlowRate::new(self.stroke_volume * self.rate.value() / 60.0)
    }

    /// **Blood Pressure**: BP = CO × peripheral_resistance.
    ///
    /// Maps to: system_load = throughput × downstream_latency
    pub fn blood_pressure(&self, peripheral_resistance: Resistance) -> Pressure {
        let co = self.cardiac_output();
        Pressure::new(co.value() * peripheral_resistance.value())
    }

    /// Execute one pump cycle.
    ///
    /// Pipeline: receive_preload → frank_starling → eject
    ///
    /// Returns the volume ejected.
    pub fn pump(&mut self, venous_return: f64) -> PumpResult {
        // Frank-Starling: auto-scale stroke volume
        self.stroke_volume = self.frank_starling(venous_return);

        // Increment beat counter
        self.beat_count += 1;

        // Alternate circuits (simplified dual circulation)
        self.active_circuit = match self.active_circuit {
            Circuit::Pulmonary => Circuit::Systemic,
            Circuit::Systemic => Circuit::Pulmonary,
            Circuit::Portal => Circuit::Systemic,
        };

        PumpResult {
            stroke_volume: self.stroke_volume,
            cardiac_output: self.cardiac_output(),
            circuit: self.active_circuit,
            beat_number: self.beat_count,
        }
    }

    /// Adjust heart rate based on demand.
    ///
    /// Negative feedback: rate increases with demand, decreases when demand drops.
    /// Set point: resting_rate. Termination: max_rate ceiling.
    pub fn regulate_rate(&mut self, demand: f64) {
        let resting = self.config.resting_rate.value();
        let max = self.config.max_rate.value();
        let target = resting + (max - resting) * demand.clamp(0.0, 1.0);
        self.rate = HeartRate::new(target);
    }
}

/// Result of a single pump cycle.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PumpResult {
    /// Volume ejected this beat
    pub stroke_volume: f64,
    /// Current cardiac output rate
    pub cardiac_output: FlowRate,
    /// Which circuit received the output
    pub circuit: Circuit,
    /// Cumulative beat count
    pub beat_number: u64,
}

/// Vital signs snapshot from the heart.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CardiacVitals {
    /// Current heart rate
    pub heart_rate: HeartRate,
    /// Current stroke volume
    pub stroke_volume: f64,
    /// Current cardiac output
    pub cardiac_output: FlowRate,
    /// Current blood pressure
    pub blood_pressure: Pressure,
    /// Total beats since start
    pub beat_count: u64,
    /// Active circuit
    pub circuit: Circuit,
}

impl Heart {
    /// Capture current vital signs.
    pub fn vitals(&self, resistance: Resistance) -> CardiacVitals {
        CardiacVitals {
            heart_rate: self.rate,
            stroke_volume: self.stroke_volume,
            cardiac_output: self.cardiac_output(),
            blood_pressure: self.blood_pressure(resistance),
            beat_count: self.beat_count,
            circuit: self.active_circuit,
        }
    }
}
