//! Params for circulatory system (data transport) tools.

use serde::Deserialize;

/// Pump data through the circulatory system.
#[derive(Debug, Deserialize)]
pub struct CirculatoryPumpParams {
    /// Data payload to transport
    pub payload: String,
    /// Source origin
    pub source: String,
    /// Destination (digestive, nervous, storage, immune)
    #[serde(default)]
    pub destination: Option<String>,
}

/// Check blood pressure (throughput monitoring).
#[derive(Debug, Deserialize)]
pub struct CirculatoryPressureParams {
    /// Current queue depth
    pub queue_depth: u64,
    /// Capacity limit
    pub capacity: u64,
}

/// Get circulatory system health overview.
#[derive(Debug, Deserialize)]
pub struct CirculatoryHealthParams {}
