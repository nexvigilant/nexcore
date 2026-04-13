use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct ComputeFlightStateParams {}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct ComputeTmrVoteParams {
    /// Sensor A value
    pub sensor_a: f64,
    /// Sensor B value
    pub sensor_b: f64,
    /// Sensor C value
    pub sensor_c: f64,
    /// Tolerance for agreement
    pub tolerance: Option<f64>,
}
