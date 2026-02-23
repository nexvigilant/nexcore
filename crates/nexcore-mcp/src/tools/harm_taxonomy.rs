//! Harm classification taxonomy MCP tools (ToV §9).
//!
//! 8 types (A-H from §9 + I from GVR), conservation law mapping,
//! axiom connections, manifestation levels, and classification engine.

use rmcp::ErrorData as McpError;
use rmcp::model::CallToolResult;
use serde_json::json;

use crate::params::harm_taxonomy::{
    HarmAxiomCatalogParams, HarmAxiomParams, HarmCatalogParams, HarmClassifyParams,
    HarmCombinationsParams, HarmDefinitionParams, HarmExhaustivenessParams,
    HarmManifestationDeriveParams,
};

// ── Helpers ──────────────────────────────────────────────────────────────

fn ok_json(value: serde_json::Value) -> Result<CallToolResult, McpError> {
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string_pretty(&value).unwrap_or_else(|_| format!("{value}")),
    )]))
}

fn err_result(msg: &str) -> Result<CallToolResult, McpError> {
    Ok(CallToolResult::error(vec![rmcp::model::Content::text(
        msg.to_string(),
    )]))
}

fn parse_harm_type(s: &str) -> Option<nexcore_harm_taxonomy::HarmTypeId> {
    use nexcore_harm_taxonomy::HarmTypeId;
    match s.to_uppercase().trim() {
        "A" => Some(HarmTypeId::A),
        "B" => Some(HarmTypeId::B),
        "C" => Some(HarmTypeId::C),
        "D" => Some(HarmTypeId::D),
        "E" => Some(HarmTypeId::E),
        "F" => Some(HarmTypeId::F),
        "G" => Some(HarmTypeId::G),
        "H" => Some(HarmTypeId::H),
        "I" => Some(HarmTypeId::I),
        _ => None,
    }
}

fn parse_multiplicity(s: &str) -> Option<nexcore_harm_taxonomy::PerturbationMultiplicity> {
    use nexcore_harm_taxonomy::PerturbationMultiplicity;
    match s.to_lowercase().trim() {
        "single" => Some(PerturbationMultiplicity::Single),
        "multiple" => Some(PerturbationMultiplicity::Multiple),
        _ => None,
    }
}

fn parse_temporal(s: &str) -> Option<nexcore_harm_taxonomy::TemporalProfile> {
    use nexcore_harm_taxonomy::TemporalProfile;
    match s.to_lowercase().trim() {
        "acute" => Some(TemporalProfile::Acute),
        "chronic" => Some(TemporalProfile::Chronic),
        "any" => Some(TemporalProfile::Any),
        _ => None,
    }
}

fn parse_determinism(s: &str) -> Option<nexcore_harm_taxonomy::ResponseDeterminism> {
    use nexcore_harm_taxonomy::ResponseDeterminism;
    match s.to_lowercase().trim() {
        "deterministic" => Some(ResponseDeterminism::Deterministic),
        "stochastic" => Some(ResponseDeterminism::Stochastic),
        _ => None,
    }
}

// ── Tools ────────────────────────────────────────────────────────────────

/// Classify a harm event from its three binary characteristics.
pub fn harm_classify(p: HarmClassifyParams) -> Result<CallToolResult, McpError> {
    let multiplicity = match parse_multiplicity(&p.multiplicity) {
        Some(m) => m,
        None => return err_result("multiplicity must be 'single' or 'multiple'"),
    };
    let temporal = match parse_temporal(&p.temporal) {
        Some(t) => t,
        None => return err_result("temporal must be 'acute', 'chronic', or 'any'"),
    };
    let determinism = match parse_determinism(&p.determinism) {
        Some(d) => d,
        None => return err_result("determinism must be 'deterministic' or 'stochastic'"),
    };

    let result = nexcore_harm_taxonomy::classify_harm_event(multiplicity, temporal, determinism);
    ok_json(serde_json::to_value(&result).unwrap_or_default())
}

/// Get the full definition for a harm type (A-I).
pub fn harm_definition(p: HarmDefinitionParams) -> Result<CallToolResult, McpError> {
    let harm_type = match parse_harm_type(&p.harm_type) {
        Some(h) => h,
        None => return err_result("harm_type must be A, B, C, D, E, F, G, H, or I"),
    };

    let def = harm_type.definition();
    ok_json(serde_json::to_value(&def).unwrap_or_default())
}

/// List all harm type definitions (A-I catalog).
pub fn harm_catalog(_p: HarmCatalogParams) -> Result<CallToolResult, McpError> {
    let catalog = nexcore_harm_taxonomy::HarmTypeDefinition::catalog();
    ok_json(serde_json::to_value(&catalog).unwrap_or_default())
}

/// Verify Theorem 9.0.1 (exhaustiveness of harm types A-H).
pub fn harm_exhaustiveness(_p: HarmExhaustivenessParams) -> Result<CallToolResult, McpError> {
    let result = nexcore_harm_taxonomy::verify_exhaustiveness();
    ok_json(serde_json::to_value(&result).unwrap_or_default())
}

/// Get the harm-axiom connection for a specific type.
pub fn harm_axiom_connection(p: HarmAxiomParams) -> Result<CallToolResult, McpError> {
    let harm_type = match parse_harm_type(&p.harm_type) {
        Some(h) => h,
        None => return err_result("harm_type must be A, B, C, D, E, F, G, H, or I"),
    };

    let connection = nexcore_harm_taxonomy::HarmAxiomConnection::for_type(harm_type);
    ok_json(serde_json::to_value(&connection).unwrap_or_default())
}

/// List all harm-axiom connections.
pub fn harm_axiom_catalog(_p: HarmAxiomCatalogParams) -> Result<CallToolResult, McpError> {
    let catalog = nexcore_harm_taxonomy::HarmAxiomConnection::catalog();
    ok_json(serde_json::to_value(&catalog).unwrap_or_default())
}

/// List common harm type combinations (non-exclusivity).
pub fn harm_combinations(_p: HarmCombinationsParams) -> Result<CallToolResult, McpError> {
    let combos = nexcore_harm_taxonomy::HarmTypeCombination::common_combinations();
    ok_json(serde_json::to_value(&combos).unwrap_or_default())
}

/// Derive manifestation level from propagation probabilities.
pub fn harm_manifestation_derive(
    p: HarmManifestationDeriveParams,
) -> Result<CallToolResult, McpError> {
    let harm_type = match parse_harm_type(&p.harm_type) {
        Some(h) => h,
        None => return err_result("harm_type must be A, B, C, D, E, F, G, H, or I"),
    };

    let derivation = nexcore_harm_taxonomy::ManifestationDerivation::derive(
        harm_type,
        &p.propagation_probs,
        p.detection_threshold,
        p.p_thresh,
    );
    ok_json(serde_json::to_value(&derivation).unwrap_or_default())
}
