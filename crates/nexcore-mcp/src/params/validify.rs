//! Params for Validify 8-gate crate validation.

use serde::Deserialize;

/// Run the full 8-gate Validify pipeline on a crate path.
#[derive(Debug, Deserialize)]
pub struct ValidifyRunParams {
    /// Path to the crate directory
    pub crate_path: String,
    /// Stop on first gate failure (default: true)
    pub fail_fast: Option<bool>,
    /// Skip specific gates by letter (e.g., ["F", "Y"])
    pub skip_gates: Option<Vec<String>>,
}

/// Run a single Validify gate.
#[derive(Debug, Deserialize)]
pub struct ValidifyGateParams {
    /// Path to the crate directory
    pub crate_path: String,
    /// Gate letter: V, A, L, I, D, I2, F, Y
    pub gate: String,
}

/// Get the Validify gate definitions.
#[derive(Debug, Deserialize)]
pub struct ValidifyGatesListParams {}
