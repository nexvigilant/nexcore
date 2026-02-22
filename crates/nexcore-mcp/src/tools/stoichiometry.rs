//! Stoichiometry MCP tools — encode/decode concepts as balanced primitive equations.
//!
//! Wraps the `nexcore-stoichiometry` crate's public API:
//! - Encode: concept + definition -> balanced primitive equation
//! - Decode: balanced equation -> Jeopardy-style "What is X?" answer
//! - Sisters: find concepts with overlapping primitive compositions
//! - Mass State: thermodynamic analysis (entropy, Gibbs, depleted/saturated)
//! - Dictionary: list/search the built-in PV term registry
//!
//! ## T1 Primitive Grounding
//! - Encoding: N(Quantity) + μ(Mapping) + ∂(Boundary)
//! - Decoding: ρ(Recursion) + κ(Comparison)
//! - Sisters: κ(Comparison) + ∃(Existence)
//! - Mass State: Σ(Sum) + ν(Frequency)

use crate::params::stoichiometry::{
    StoichiometryDecodeParams, StoichiometryDictionaryParams, StoichiometryEncodeParams,
    StoichiometryIsBalancedParams, StoichiometryIsIsomerParams, StoichiometryMassStateParams,
    StoichiometryProveParams, StoichiometrySistersParams,
};
use nexcore_lex_primitiva::primitiva::LexPrimitiva;
use nexcore_stoichiometry::balance::Balancer;
use nexcore_stoichiometry::prelude::*;
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use serde_json::json;

/// Encode a concept as a balanced primitive equation.
///
/// Creates a fresh codec with the built-in dictionary, encodes the concept,
/// and returns the balanced equation as JSON.
pub fn encode(params: StoichiometryEncodeParams) -> Result<CallToolResult, McpError> {
    let source = parse_source(&params.source);
    let mut codec = StoichiometricCodec::builtin();

    match codec.encode(&params.concept, &params.definition, source) {
        Ok(equation) => {
            let eq_json = serde_json::to_value(&equation).map_err(|e| {
                McpError::internal_error(format!("Serialization failed: {e}"), None)
            })?;
            Ok(CallToolResult::success(vec![Content::text(
                json!({
                    "success": true,
                    "concept": params.concept,
                    "definition": params.definition,
                    "equation": eq_json,
                    "balanced": equation.balance.is_balanced,
                    "delta": equation.balance.delta,
                    "reactant_count": equation.reactants.len(),
                    "display": format!("{equation}"),
                })
                .to_string(),
            )]))
        }
        Err(e) => {
            let content = vec![Content::text(
                json!({
                    "success": false,
                    "error": format!("{e}"),
                    "concept": params.concept,
                })
                .to_string(),
            )];
            Ok(CallToolResult::error(content))
        }
    }
}

/// Decode a balanced equation back to a Jeopardy-style answer.
///
/// Accepts a JSON-serialized `BalancedEquation` and returns the decoded concept.
pub fn decode(params: StoichiometryDecodeParams) -> Result<CallToolResult, McpError> {
    let equation: BalancedEquation = serde_json::from_str(&params.equation_json)
        .map_err(|e| McpError::invalid_params(format!("Invalid equation JSON: {e}"), None))?;

    let codec = StoichiometricCodec::builtin();

    match codec.decode(&equation) {
        Some(answer) => {
            let answer_json = serde_json::to_value(&answer).map_err(|e| {
                McpError::internal_error(format!("Serialization failed: {e}"), None)
            })?;
            Ok(CallToolResult::success(vec![Content::text(
                json!({
                    "success": true,
                    "answer": answer_json,
                    "display": format!("{answer}"),
                })
                .to_string(),
            )]))
        }
        None => {
            let content = vec![Content::text(
                json!({
                    "success": false,
                    "error": "No matching concept found for the given equation",
                })
                .to_string(),
            )];
            Ok(CallToolResult::error(content))
        }
    }
}

/// Find sister concepts with overlapping primitive compositions.
///
/// Looks up the concept in the built-in dictionary, then searches for terms
/// whose Jaccard similarity exceeds the threshold.
pub fn sisters(params: StoichiometrySistersParams) -> Result<CallToolResult, McpError> {
    let codec = StoichiometricCodec::builtin();

    let term = codec.dictionary().lookup(&params.concept).ok_or_else(|| {
        McpError::invalid_params(
            format!(
                "Concept '{}' not found in dictionary. Use stoichiometry_dictionary to list available terms.",
                params.concept
            ),
            None,
        )
    })?;

    let sisters = codec.find_sisters(&term.equation, params.threshold);

    let sisters_json: Vec<serde_json::Value> = sisters
        .iter()
        .map(|s| {
            serde_json::to_value(s)
                .unwrap_or_else(|_| json!({"name": s.name, "similarity": s.similarity}))
        })
        .collect();

    Ok(CallToolResult::success(vec![Content::text(
        json!({
            "success": true,
            "concept": params.concept,
            "threshold": params.threshold,
            "sister_count": sisters.len(),
            "sisters": sisters_json,
        })
        .to_string(),
    )]))
}

/// Compute thermodynamic mass state for a concept.
///
/// Returns entropy, Gibbs free energy, equilibrium status, and
/// depleted/saturated primitives for the concept's equation.
pub fn mass_state(params: StoichiometryMassStateParams) -> Result<CallToolResult, McpError> {
    let codec = StoichiometricCodec::builtin();

    let term = codec.dictionary().lookup(&params.concept).ok_or_else(|| {
        McpError::invalid_params(
            format!(
                "Concept '{}' not found in dictionary. Use stoichiometry_dictionary to list available terms.",
                params.concept
            ),
            None,
        )
    })?;

    let state = MassState::from_equation(&term.equation);

    let depleted_names: Vec<String> = state.depleted().iter().map(|p| format!("{p:?}")).collect();
    let saturated_names: Vec<String> = state.saturated().iter().map(|p| format!("{p:?}")).collect();

    Ok(CallToolResult::success(vec![Content::text(
        json!({
            "success": true,
            "concept": params.concept,
            "total_mass": state.total_mass(),
            "entropy": state.entropy(),
            "max_entropy": MassState::max_entropy(),
            "entropy_ratio": if MassState::max_entropy() > 0.0 {
                state.entropy() / MassState::max_entropy()
            } else {
                0.0
            },
            "gibbs_free_energy": state.gibbs_free_energy(),
            "is_equilibrium": state.is_equilibrium(),
            "depleted_primitives": depleted_names,
            "depleted_count": state.depleted().len(),
            "saturated_primitives": saturated_names,
            "saturated_count": state.saturated().len(),
        })
        .to_string(),
    )]))
}

/// List or search the built-in dictionary.
///
/// Action "list" returns all terms. Action "search" filters by primitive name.
pub fn dictionary(params: StoichiometryDictionaryParams) -> Result<CallToolResult, McpError> {
    let codec = StoichiometricCodec::builtin();
    let dict = codec.dictionary();

    match params.action.as_str() {
        "list" => {
            let terms: Vec<serde_json::Value> = dict
                .all_terms()
                .iter()
                .map(|t| {
                    json!({
                        "name": t.name,
                        "definition": t.definition,
                        "source": format!("{}", t.source),
                        "balanced": t.equation.balance.is_balanced,
                        "display": format!("{}", t.equation),
                    })
                })
                .collect();

            Ok(CallToolResult::success(vec![Content::text(
                json!({
                    "success": true,
                    "action": "list",
                    "term_count": terms.len(),
                    "terms": terms,
                })
                .to_string(),
            )]))
        }
        "search" => {
            let filter = params.filter_primitive.as_deref().unwrap_or("");
            if filter.is_empty() {
                return Err(McpError::invalid_params(
                    "filter_primitive is required when action is 'search'".to_string(),
                    None,
                ));
            }

            // Parse the primitive name
            let target = parse_primitive(filter).ok_or_else(|| {
                McpError::invalid_params(
                    format!(
                        "Unknown primitive '{}'. Valid names: Causality, Quantity, Existence, Comparison, State, Mapping, Sequence, Recursion, Void, Boundary, Frequency, Location, Persistence, Irreversibility, Sum",
                        filter
                    ),
                    None,
                )
            })?;

            let terms: Vec<serde_json::Value> = dict
                .all_terms()
                .iter()
                .filter(|t| t.equation.concept.formula.primitives().contains(&target))
                .map(|t| {
                    json!({
                        "name": t.name,
                        "definition": t.definition,
                        "source": format!("{}", t.source),
                        "primitive_count": t.equation.concept.formula.primitives().iter()
                            .filter(|p| **p == target).count(),
                        "display": format!("{}", t.equation),
                    })
                })
                .collect();

            Ok(CallToolResult::success(vec![Content::text(
                json!({
                    "success": true,
                    "action": "search",
                    "filter_primitive": filter,
                    "match_count": terms.len(),
                    "terms": terms,
                })
                .to_string(),
            )]))
        }
        other => Err(McpError::invalid_params(
            format!("Unknown action '{}'. Use 'list' or 'search'.", other),
            None,
        )),
    }
}

// ============================================================================
// Helpers
// ============================================================================

/// Parse a source string into a `DefinitionSource`.
///
/// Accepts formats like "ICH E2A", "CIOMS 1", "FDA Guidance X", "WHO Drug Y",
/// "MedDRA Z", or falls back to `Custom`.
fn parse_source(source: &str) -> DefinitionSource {
    let lower = source.to_lowercase();
    if lower.starts_with("ich") {
        DefinitionSource::IchGuideline(source.to_string())
    } else if lower.starts_with("cioms") {
        DefinitionSource::CiomsReport(source.to_string())
    } else if lower.starts_with("fda") {
        DefinitionSource::FdaGuidance(source.to_string())
    } else if lower.starts_with("who") {
        DefinitionSource::WhoDrug(source.to_string())
    } else if lower.starts_with("meddra") {
        DefinitionSource::MedDRA(source.to_string())
    } else {
        DefinitionSource::Custom(source.to_string())
    }
}

/// Parse a primitive name string into a `LexPrimitiva`.
fn parse_primitive(name: &str) -> Option<LexPrimitiva> {
    match name.to_lowercase().as_str() {
        "causality" | "cause" | "→" => Some(LexPrimitiva::Causality),
        "quantity" | "n" => Some(LexPrimitiva::Quantity),
        "existence" | "∃" => Some(LexPrimitiva::Existence),
        "comparison" | "κ" => Some(LexPrimitiva::Comparison),
        "state" | "ς" => Some(LexPrimitiva::State),
        "mapping" | "μ" => Some(LexPrimitiva::Mapping),
        "sequence" | "σ" => Some(LexPrimitiva::Sequence),
        "recursion" | "ρ" => Some(LexPrimitiva::Recursion),
        "void" | "∅" => Some(LexPrimitiva::Void),
        "boundary" | "∂" => Some(LexPrimitiva::Boundary),
        "frequency" | "ν" => Some(LexPrimitiva::Frequency),
        "location" | "λ" => Some(LexPrimitiva::Location),
        "persistence" | "π" => Some(LexPrimitiva::Persistence),
        "irreversibility" | "∝" => Some(LexPrimitiva::Irreversibility),
        "sum" | "σ_sum" | "Σ" => Some(LexPrimitiva::Sum),
        "product" | "×" => Some(LexPrimitiva::Product),
        _ => None,
    }
}

// ============================================================================
// Balance & Isomer tools
// ============================================================================

/// Check if a balanced equation satisfies primitive conservation.
///
/// Deserializes the equation JSON, runs `Balancer::is_balanced`, and returns
/// the boolean result plus the per-primitive deficit array.
pub fn is_balanced(params: StoichiometryIsBalancedParams) -> Result<CallToolResult, McpError> {
    let equation: BalancedEquation = serde_json::from_str(&params.equation_json)
        .map_err(|e| McpError::invalid_params(format!("Invalid equation JSON: {e}"), None))?;

    let balanced = Balancer::is_balanced(&equation);
    let deficit = Balancer::deficit(&equation);

    Ok(CallToolResult::success(vec![Content::text(
        json!({
            "success": true,
            "is_balanced": balanced,
            "deficit": deficit,
            "concept": equation.concept.name,
        })
        .to_string(),
    )]))
}

/// Generate a balance proof for an equation showing reactant/product mass conservation.
///
/// Returns the full `BalanceProof` including reactant mass, product mass, delta,
/// and per-primitive inventories.
pub fn prove(params: StoichiometryProveParams) -> Result<CallToolResult, McpError> {
    let equation: BalancedEquation = serde_json::from_str(&params.equation_json)
        .map_err(|e| McpError::invalid_params(format!("Invalid equation JSON: {e}"), None))?;

    let proof = Balancer::prove(&equation.reactants, &equation.concept);
    let proof_json = serde_json::to_value(&proof).map_err(|e| {
        McpError::internal_error(format!("Serialization failed: {e}"), None)
    })?;

    Ok(CallToolResult::success(vec![Content::text(
        json!({
            "success": true,
            "concept": equation.concept.name,
            "proof": proof_json,
            "is_balanced": proof.is_balanced,
            "delta": proof.delta,
            "reactant_mass": proof.reactant_mass,
            "product_mass": proof.product_mass,
        })
        .to_string(),
    )]))
}

/// Check if two equations are isomers (same primitive set, different dominant).
///
/// Also returns the Jaccard similarity between the two primitive compositions.
pub fn is_isomer(params: StoichiometryIsIsomerParams) -> Result<CallToolResult, McpError> {
    let eq_a: BalancedEquation = serde_json::from_str(&params.equation_a_json)
        .map_err(|e| McpError::invalid_params(format!("Invalid equation A JSON: {e}"), None))?;
    let eq_b: BalancedEquation = serde_json::from_str(&params.equation_b_json)
        .map_err(|e| McpError::invalid_params(format!("Invalid equation B JSON: {e}"), None))?;

    let isomer = nexcore_stoichiometry::sister::is_isomer(&eq_a, &eq_b);
    let similarity = nexcore_stoichiometry::sister::jaccard_similarity(
        eq_a.concept.formula.primitives(),
        eq_b.concept.formula.primitives(),
    );

    Ok(CallToolResult::success(vec![Content::text(
        json!({
            "success": true,
            "is_isomer": isomer,
            "concept_a": eq_a.concept.name,
            "concept_b": eq_b.concept.name,
            "jaccard_similarity": format!("{:.3}", similarity),
        })
        .to_string(),
    )]))
}
