//! Ghost privacy MCP tool parameters.
//!
//! Typed parameter structs for privacy enforcement, PII detection,
//! anonymization boundary checking, and data scrubbing.

use schemars::JsonSchema;
use serde::Deserialize;

/// Check anonymization boundary violations for given metrics.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GhostBoundaryCheckParams {
    /// Ghost mode: "Off", "Standard", "Strict", or "Maximum".
    pub mode: String,
    /// Observed re-identification risk [0.0, 1.0].
    pub risk: f64,
    /// Observed k-anonymity value.
    pub k: u32,
    /// Observed l-diversity value.
    pub l: u32,
}

/// Get properties for a ghost mode (k-anonymity target, reversal allowed, etc).
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GhostModeInfoParams {
    /// Ghost mode: "Off", "Standard", "Strict", or "Maximum".
    pub mode: String,
}

/// Get the effective policy for a data category under a given mode.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GhostCategoryPolicyParams {
    /// Ghost mode: "Off", "Standard", "Strict", or "Maximum".
    pub mode: String,
    /// Data category: "BasicIdentity", "HealthData", "FinancialData",
    /// "BiometricData", "BehavioralData", "LocationData",
    /// "DeviceData", "CommunicationData".
    pub category: String,
}

/// Scan fields for PII leak patterns.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GhostScanPiiParams {
    /// Ghost mode: "Off", "Standard", "Strict", or "Maximum".
    pub mode: String,
    /// Map of field names to field values to scan.
    pub fields: std::collections::HashMap<String, String>,
}

/// Scrub PII from a set of fields according to ghost mode policy.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GhostScrubFieldsParams {
    /// Ghost mode: "Off", "Standard", "Strict", or "Maximum".
    pub mode: String,
    /// Map of field names to field values to scrub.
    pub fields: std::collections::HashMap<String, String>,
}
