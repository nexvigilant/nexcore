//! Params for cardiovascular system (data transport, pressure, flow) tools.

use serde::Deserialize;

/// Compute blood pressure (data throughput pressure) from cardiac output and resistance.
#[derive(Debug, Deserialize)]
pub struct CardioBloodPressureParams {
    /// Cardiac output: tools called per minute
    pub cardiac_output: f64,
    /// Peripheral resistance: average latency factor
    pub peripheral_resistance: f64,
}

/// Assess blood health (data quality across transport).
#[derive(Debug, Deserialize)]
pub struct CardioBloodHealthParams {
    /// Red cell count (data carriers — active MCP tools)
    pub red_cells: u64,
    /// White cell count (defense — hooks/validators)
    pub white_cells: u64,
    /// Platelet count (repair mechanisms — error handlers)
    pub platelets: u64,
}

/// Diagnose cardiovascular pathology from symptoms.
#[derive(Debug, Deserialize)]
pub struct CardioDiagnoseParams {
    /// Observed symptoms (e.g., "high_latency", "data_loss", "backpressure")
    pub symptoms: Vec<String>,
}

/// Get cardiac vitals overview.
#[derive(Debug, Deserialize)]
pub struct CardioVitalsParams {}
