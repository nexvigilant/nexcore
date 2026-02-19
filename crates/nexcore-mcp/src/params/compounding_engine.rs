//! Params for compounding pipeline metrics tools.

use serde::Deserialize;

/// Measure compounding velocity across learning dimensions.
#[derive(Debug, Deserialize)]
pub struct CompoundingVelocityParams {
    /// Time window in hours (default: 24)
    pub window_hours: Option<u64>,
}

/// Check learning loop health (CE→RO→AC→AE phases).
#[derive(Debug, Deserialize)]
pub struct CompoundingLoopHealthParams {}

/// Get compounding metrics summary.
#[derive(Debug, Deserialize)]
pub struct CompoundingMetricsParams {
    /// Include historical trend (default: false)
    pub include_trend: Option<bool>,
}
