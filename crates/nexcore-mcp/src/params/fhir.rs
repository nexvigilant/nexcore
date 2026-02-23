//! FHIR (HL7 R4) MCP tool parameters.
//!
//! Typed parameter structs for FHIR resource parsing, validation,
//! and AdverseEvent → SignalInput conversion.

use schemars::JsonSchema;
use serde::Deserialize;

/// Parse a FHIR AdverseEvent JSON and convert to a SignalInput for PV analysis.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct FhirAdverseEventToSignalParams {
    /// FHIR AdverseEvent resource as JSON string.
    pub adverse_event_json: String,
}

/// Batch convert multiple FHIR AdverseEvent JSONs to SignalInputs.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct FhirBatchToSignalsParams {
    /// Array of FHIR AdverseEvent resources as JSON strings.
    pub adverse_events_json: Vec<String>,
}

/// Parse a FHIR Bundle JSON and extract resource summaries.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct FhirParseBundleParams {
    /// FHIR Bundle resource as JSON string.
    pub bundle_json: String,
}

/// Validate a FHIR resource JSON structure (check required fields, resource_type).
#[derive(Debug, Deserialize, JsonSchema)]
pub struct FhirValidateResourceParams {
    /// FHIR resource as JSON string.
    pub resource_json: String,
    /// Expected resource type (e.g. "AdverseEvent", "Patient", "Medication").
    pub expected_type: Option<String>,
}
