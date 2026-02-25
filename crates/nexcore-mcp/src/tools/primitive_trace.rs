//! Primitive Trace — simulate how a word/concept interacts across the full stack.
//!
//! Takes a concept + its T1 primitives, then traces every grounded type
//! that uses those primitives, grouped by layer. Computes molecular weight,
//! interaction density, and architectural reach.
//!
//! 1 tool: primitive_trace.
//!
//! Tier: T2-C (μ Mapping + κ Comparison + Σ Sum + σ Sequence)

use std::collections::{BTreeSet, HashMap, HashSet};

use nexcore_lex_primitiva::molecular_weight::{AtomicMass, MolecularFormula};
use nexcore_lex_primitiva::primitiva::LexPrimitiva;
use nexcore_lex_primitiva::{GroundingTier, PrimitiveComposition};
use serde_json::json;

use crate::params::PrimitiveTraceParams;
use crate::tools::lex_primitiva::{KNOWN_TYPES, find_primitive, get_composition_direct};
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content, ErrorCode};

/// Trace how a concept's primitives manifest across the codebase.
pub fn primitive_trace(params: PrimitiveTraceParams) -> Result<CallToolResult, McpError> {
    // Parse primitives
    let mut primitives = Vec::new();
    for name in &params.primitives {
        match find_primitive(name) {
            Some(p) => primitives.push(p),
            None => {
                return Err(McpError::new(
                    ErrorCode(400),
                    format!(
                        "Unknown primitive '{}'. Use name (e.g. 'state') or symbol (e.g. 'ς')",
                        name
                    ),
                    None,
                ));
            }
        }
    }

    if primitives.is_empty() {
        return Err(McpError::new(
            ErrorCode(400),
            "primitives array must not be empty",
            None,
        ));
    }

    let concept = params.concept.as_deref().unwrap_or("unnamed");
    let prim_set: BTreeSet<LexPrimitiva> = primitives.iter().copied().collect();

    // --- Molecular Weight ---
    let formula = MolecularFormula::new(concept).with_all(&primitives);
    let weight = formula.weight();

    // --- Per-primitive traces ---
    // For each primitive, find ALL types that include it
    let mut per_primitive: Vec<serde_json::Value> = Vec::new();
    let mut all_matching_types: HashSet<&str> = HashSet::new();

    for &prim in &primitives {
        let mass = AtomicMass::of(prim);
        let mut types_using: Vec<(&str, String)> = Vec::new();

        for &type_name in KNOWN_TYPES {
            if let Some(comp) = get_composition_direct(type_name) {
                if comp.unique().contains(&prim) {
                    let tier = GroundingTier::classify(&comp);
                    let is_dominant = comp.dominant == Some(prim);
                    types_using.push((type_name, format!("{}", tier)));
                    all_matching_types.insert(type_name);
                }
            }
        }

        per_primitive.push(json!({
            "primitive": prim.name(),
            "symbol": prim.symbol(),
            "atomic_mass_bits": round3(mass.bits()),
            "frequency": mass.frequency(),
            "types_count": types_using.len(),
            "types": types_using.iter().take(15).map(|(name, tier)| {
                json!({ "type": name, "tier": tier })
            }).collect::<Vec<_>>(),
            "truncated": types_using.len() > 15,
        }));
    }

    // --- Cross-primitive analysis ---
    // Types that contain ALL the concept's primitives (exact structural matches)
    let mut exact_matches: Vec<serde_json::Value> = Vec::new();
    let mut partial_matches: Vec<serde_json::Value> = Vec::new();

    for &type_name in KNOWN_TYPES {
        if let Some(comp) = get_composition_direct(type_name) {
            let type_prims = comp.unique();
            if prim_set.is_subset(&type_prims) {
                // Type contains ALL our primitives
                let tier = GroundingTier::classify(&comp);
                let extra: Vec<_> = type_prims.difference(&prim_set).map(|p| p.name()).collect();
                exact_matches.push(json!({
                    "type": type_name,
                    "tier": format!("{}", tier),
                    "dominant": comp.dominant.map(|d| d.name()),
                    "extra_primitives": extra,
                    "total_primitives": type_prims.len(),
                }));
            } else {
                // Check overlap
                let overlap: HashSet<_> = prim_set.intersection(&type_prims).copied().collect();
                if overlap.len() >= 2 && overlap.len() < prim_set.len() {
                    let missing: Vec<_> =
                        prim_set.difference(&type_prims).map(|p| p.name()).collect();
                    partial_matches.push(json!({
                        "type": type_name,
                        "overlap_count": overlap.len(),
                        "overlap": overlap.iter().map(|p| p.name()).collect::<Vec<_>>(),
                        "missing": missing,
                    }));
                }
            }
        }
    }

    // Sort partial matches by overlap count descending
    partial_matches.sort_by(|a, b| {
        let ca = a.get("overlap_count").and_then(|v| v.as_u64()).unwrap_or(0);
        let cb = b.get("overlap_count").and_then(|v| v.as_u64()).unwrap_or(0);
        cb.cmp(&ca)
    });

    // --- Architectural layer grouping ---
    let layer_map = classify_layers(&exact_matches);

    // --- Primitive interaction density ---
    // How many types share pairs of our primitives?
    let mut pair_density: Vec<serde_json::Value> = Vec::new();
    let prim_vec: Vec<_> = prim_set.iter().copied().collect();
    for i in 0..prim_vec.len() {
        for j in (i + 1)..prim_vec.len() {
            let p1 = prim_vec[i];
            let p2 = prim_vec[j];
            let mut count = 0;
            for &type_name in KNOWN_TYPES {
                if let Some(comp) = get_composition_direct(type_name) {
                    let s = comp.unique();
                    if s.contains(&p1) && s.contains(&p2) {
                        count += 1;
                    }
                }
            }
            pair_density.push(json!({
                "pair": [p1.name(), p2.name()],
                "symbols": format!("{}+{}", p1.symbol(), p2.symbol()),
                "co_occurrence_count": count,
            }));
        }
    }

    // Sort by co-occurrence descending
    pair_density.sort_by(|a, b| {
        let ca = a
            .get("co_occurrence_count")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        let cb = b
            .get("co_occurrence_count")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        cb.cmp(&ca)
    });

    let result = json!({
        "concept": concept,
        "formula": formula.formula_string(),
        "molecular_weight": {
            "daltons": round3(weight.daltons()),
            "transfer_class": format!("{}", weight.transfer_class()),
            "predicted_transfer": round3(weight.predicted_transfer()),
            "primitive_count": weight.primitive_count(),
        },
        "per_primitive_trace": per_primitive,
        "structural_matches": {
            "exact_count": exact_matches.len(),
            "exact": exact_matches.iter().take(20).cloned().collect::<Vec<_>>(),
            "partial_count": partial_matches.len(),
            "partial_top10": partial_matches.iter().take(10).cloned().collect::<Vec<_>>(),
        },
        "pair_interaction_density": pair_density,
        "architectural_reach": {
            "total_types_touched": all_matching_types.len(),
            "total_known_types": KNOWN_TYPES.len(),
            "reach_percent": round3(all_matching_types.len() as f64 / KNOWN_TYPES.len() as f64 * 100.0),
            "layer_distribution": layer_map,
        },
    });

    Ok(CallToolResult::success(vec![Content::text(
        result.to_string(),
    )]))
}

/// Classify types into architectural layers based on naming conventions.
fn classify_layers(types: &[serde_json::Value]) -> serde_json::Value {
    let mut layers: HashMap<&str, Vec<&str>> = HashMap::new();

    for t in types {
        let name = t.get("type").and_then(|v| v.as_str()).unwrap_or("");
        let layer = if name.contains("Guardian") || name.contains("Threat") || name.contains("Risk")
        {
            "guardian"
        } else if name.contains("Trust") || name.contains("Evidence") {
            "trust"
        } else if name.contains("Signal") || name.contains("Pv") || name.contains("Icsr") {
            "pv_domain"
        } else if name.contains("State") || name.contains("Lifecycle") || name.contains("Fsm") {
            "state_machines"
        } else if name.starts_with('(') || name.starts_with("Vec") || name.starts_with("Option") {
            "foundation_types"
        } else if name.contains("Skill") || name.contains("Brain") || name.contains("Session") {
            "orchestration"
        } else {
            "other"
        };
        layers.entry(layer).or_default().push(name);
    }

    let mut result = json!({});
    for (layer, types) in &layers {
        result[layer] = json!({
            "count": types.len(),
            "types": types,
        });
    }
    result
}

fn round3(v: f64) -> f64 {
    (v * 1000.0).round() / 1000.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trace_state_returns_types() {
        let params = PrimitiveTraceParams {
            concept: Some("Guard".to_string()),
            primitives: vec!["state".to_string(), "boundary".to_string()],
        };
        let result = primitive_trace(params);
        assert!(result.is_ok());
    }

    #[test]
    fn trace_single_primitive() {
        let params = PrimitiveTraceParams {
            concept: Some("Counter".to_string()),
            primitives: vec!["quantity".to_string()],
        };
        let result = primitive_trace(params);
        assert!(result.is_ok());
    }

    #[test]
    fn rejects_unknown_primitive() {
        let params = PrimitiveTraceParams {
            concept: Some("Bad".to_string()),
            primitives: vec!["nonexistent".to_string()],
        };
        assert!(primitive_trace(params).is_err());
    }

    #[test]
    fn accepts_symbols() {
        let params = PrimitiveTraceParams {
            concept: None,
            primitives: vec!["ς".to_string(), "∂".to_string(), "→".to_string()],
        };
        assert!(primitive_trace(params).is_ok());
    }
}
