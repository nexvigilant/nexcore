//! Homeostasis System Parameters
//! Tier: T3 (Domain-specific MCP tool parameters)
//!
//! Self-regulation, setpoints, Hill-curve response, storm detection.

use rmcp::schemars::{self, JsonSchema};
use rmcp::serde::Deserialize;

/// Sense the current system state by providing metric readings.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct HomeostasisSenseParams {
    /// Metric name being sensed (e.g., "error_rate", "latency_ms", "queue_depth")
    pub metric_name: String,
    /// Current observed value
    pub value: f64,
    /// Optional sensor type: "external_threat", "internal_damage", "self_measurement", "environmental"
    #[serde(default)]
    pub sensor_type: Option<String>,
    /// Whether this reading is anomalous (above expected range)
    #[serde(default)]
    pub is_anomalous: Option<bool>,
}

/// Analyze setpoint deviation for a named metric against its baseline.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct HomeostasisSetpointParams {
    /// Metric name (e.g., "error_rate", "cpu_usage", "response_time_ms")
    pub metric_name: String,
    /// Current observed value
    pub current_value: f64,
    /// Known healthy setpoint (baseline) for this metric
    pub setpoint: f64,
    /// Absolute maximum tolerable value
    #[serde(default)]
    pub absolute_maximum: Option<f64>,
}

/// Compute the homeostatic error signal (deviation from setpoint).
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct HomeostasisErrorParams {
    /// Current threat level (sum of threat signal strengths)
    pub threat_level: f64,
    /// Current response level (system's response magnitude)
    pub response_level: f64,
    /// Current damage level from internal/external sources
    #[serde(default)]
    pub damage_level: Option<f64>,
    /// Current dampening level (anti-inflammatory signals)
    #[serde(default)]
    pub dampening_level: Option<f64>,
}

/// Generate a Hill-curve bounded response for a given signal strength.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct HomeostasisRespondParams {
    /// Signal strength to respond to
    pub signal_strength: f64,
    /// Maximum response ceiling (Rmax)
    pub max_response: f64,
    /// Signal at half-maximal response (K_half). Defaults to max_response * 0.5
    #[serde(default)]
    pub k_half: Option<f64>,
    /// Hill coefficient controlling steepness (1=gradual, 4=switch-like). Default 2.0
    #[serde(default)]
    pub hill_coefficient: Option<f64>,
}

/// Evaluate current system state for storm (cytokine storm) signatures.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct HomeostasisStormDetectParams {
    /// Current threat level
    pub threat_level: f64,
    /// Current response level
    pub response_level: f64,
    /// Current damage level
    #[serde(default)]
    pub damage_level: Option<f64>,
    /// Historical threat values (oldest first) for trend analysis
    #[serde(default)]
    pub threat_history: Option<Vec<f64>>,
    /// Historical response values (oldest first) for trend analysis
    #[serde(default)]
    pub response_history: Option<Vec<f64>>,
}

/// Recall incident memory patterns (what past storms/incidents looked like).
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct HomeostasisMemoryRecallParams {
    /// Pattern to search for (e.g., "retry_storm", "cascade_failure", "rate_limit")
    #[serde(default)]
    pub pattern: Option<String>,
    /// Maximum number of incidents to return
    #[serde(default)]
    pub limit: Option<usize>,
}
