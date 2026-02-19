//! Params for nervous system (signal routing, reflex arcs, myelination) tools.

use serde::Deserialize;

/// Analyze a reflex arc (trigger → response pattern).
#[derive(Debug, Deserialize)]
pub struct NervousReflexParams {
    /// The trigger pattern (e.g., hook event, tool name)
    pub trigger: String,
    /// Optional response to validate against
    pub expected_response: Option<String>,
}

/// Measure signal latency through a processing chain.
#[derive(Debug, Deserialize)]
pub struct NervousLatencyParams {
    /// Processing chain stages (e.g., ["hook", "dispatch", "tool", "response"])
    pub chain: Vec<String>,
    /// Optional per-stage latency measurements in ms
    pub latencies_ms: Option<Vec<f64>>,
}

/// Check myelination status (caching/optimization of hot paths).
#[derive(Debug, Deserialize)]
pub struct NervousMyelinationParams {
    /// Path or pattern to check myelination for
    pub path: Option<String>,
}

/// Get nervous system health overview.
#[derive(Debug, Deserialize)]
pub struct NervousHealthParams {}
