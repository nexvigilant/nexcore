//! Integrity Assessment Parameters (AI Text Detection)
//!
//! Detection, KSB assessment, and domain calibration.

use rmcp::schemars::{self, JsonSchema};
use rmcp::serde::{Deserialize, Serialize};

/// Parameters for integrity_analyze.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct IntegrityAnalyzeParams {
    /// Text to analyze.
    pub text: String,
    /// Bloom taxonomy level (1-7).
    pub bloom_level: Option<u8>,
    /// PV domain ID.
    pub domain_id: Option<String>,
    /// Custom classification threshold.
    pub threshold: Option<f64>,
    /// Use strict threshold preset.
    #[serde(default)]
    pub strict_mode: bool,
}

/// Parameters for KSB response assessment.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct IntegrityAssessKsbParams {
    /// KSB response text to assess.
    pub text: String,
    /// Bloom taxonomy level (1-7).
    pub bloom_level: u8,
    /// PV domain ID.
    pub domain_id: Option<String>,
}

/// Parameters for domain calibration profile.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct IntegrityCalibrationParams {
    /// Domain ID to retrieve calibration for.
    pub domain_id: String,
}
