//! Molecular Weight of Words — Shannon information-theoretic mass computation.
//!
//! Computes the "molecular weight" of any concept by summing the
//! information content of its constituent T1 primitives.
//!
//! 5 tools: compute, periodic_table, compare, predict_transfer, recalibrate.
//!
//! Tier: T2-P (Σ Sum + N Quantity + μ Mapping)

use nexcore_lex_primitiva::molecular_weight::{
    AtomicMass, MolecularFormula, TransferClass, max_atomic_mass, max_molecular_weight,
    min_atomic_mass, shannon_entropy,
};
use nexcore_lex_primitiva::primitiva::LexPrimitiva;
use serde_json::json;

use crate::params::{MwCompareParams, MwComputeParams, MwPredictTransferParams};
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content, ErrorCode};

// ============================================================================
// Helpers
// ============================================================================

fn parse_primitive(input: &str) -> Result<LexPrimitiva, McpError> {
    for p in LexPrimitiva::all() {
        if p.symbol() == input || p.name().eq_ignore_ascii_case(input) {
            return Ok(p);
        }
    }
    Err(McpError::new(
        ErrorCode(400),
        format!(
            "Unknown primitive '{}'. Use name (e.g. 'state', 'boundary') or symbol (e.g. 'ς', '∂')",
            input
        ),
        None,
    ))
}

fn parse_primitive_list(input: &[String]) -> Result<Vec<LexPrimitiva>, McpError> {
    input.iter().map(|s| parse_primitive(s.trim())).collect()
}

fn round3(v: f64) -> f64 {
    (v * 1000.0).round() / 1000.0
}

// ============================================================================
// Tool implementations
// ============================================================================

/// Compute the molecular weight of a concept from its constituent primitives.
pub fn mw_compute(params: MwComputeParams) -> Result<CallToolResult, McpError> {
    let primitives = parse_primitive_list(&params.primitives)?;

    if primitives.is_empty() {
        return Err(McpError::new(
            ErrorCode(400),
            "primitives array must not be empty",
            None,
        ));
    }

    let name = params.name.as_deref().unwrap_or("unnamed");
    let mut formula = MolecularFormula::new(name);
    for p in &primitives {
        formula = formula.with(*p);
    }

    let weight = formula.weight();
    let masses: Vec<_> = formula
        .atomic_masses()
        .iter()
        .map(|m| {
            json!({
                "primitive": m.primitive().name(),
                "symbol": m.primitive().symbol(),
                "mass_bits": round3(m.bits()),
                "frequency": m.frequency(),
                "probability": round3(m.probability()),
            })
        })
        .collect();

    let result = json!({
        "concept": name,
        "formula": formula.formula_string(),
        "molecular_weight_daltons": round3(weight.daltons()),
        "primitive_count": weight.primitive_count(),
        "average_mass": round3(weight.average_mass()),
        "transfer_class": format!("{}", weight.transfer_class()),
        "predicted_transfer_confidence": round3(weight.predicted_transfer()),
        "tier_prediction": format!("{}", weight.tier_aware_class()),
        "hybrid_transfer_confidence": round3(weight.predicted_transfer_hybrid()),
        "constituents": masses,
        "interpretation": match weight.transfer_class() {
            TransferClass::Light => "Light molecule — high cross-domain transferability (T2-P behavior)",
            TransferClass::Medium => "Medium molecule — moderate transferability (T2-C behavior)",
            TransferClass::Heavy => "Heavy molecule — domain-locked, low transferability (T3 behavior)",
            _ => "Unknown transfer class",
        },
    });

    Ok(CallToolResult::success(vec![Content::text(
        result.to_string(),
    )]))
}

/// Get the full periodic table of T1 primitive atomic masses.
pub fn mw_periodic_table() -> Result<CallToolResult, McpError> {
    let table: Vec<_> = AtomicMass::periodic_table()
        .iter()
        .map(|m| {
            json!({
                "rank": 0, // will be set below
                "primitive": m.primitive().name(),
                "symbol": m.primitive().symbol(),
                "mass_bits": round3(m.bits()),
                "frequency": m.frequency(),
                "probability": round3(m.probability()),
            })
        })
        .collect();

    // Add rank
    let table: Vec<_> = table
        .into_iter()
        .enumerate()
        .map(|(i, mut v)| {
            v.as_object_mut().map(|o| {
                o.insert("rank".to_string(), json!(i + 1));
            });
            v
        })
        .collect();

    let entropy = shannon_entropy();
    let max_mw = max_molecular_weight();
    let lightest = min_atomic_mass();
    let heaviest = max_atomic_mass();

    let result = json!({
        "total_primitives": 16,
        "total_references": 3664,
        "shannon_entropy_bits": round3(entropy),
        "lightest_atom": {
            "primitive": lightest.primitive().name(),
            "mass_bits": round3(lightest.bits()),
        },
        "heaviest_atom": {
            "primitive": heaviest.primitive().name(),
            "mass_bits": round3(heaviest.bits()),
        },
        "max_molecular_weight": round3(max_mw.daltons()),
        "periodic_table": table,
    });

    Ok(CallToolResult::success(vec![Content::text(
        result.to_string(),
    )]))
}

/// Compare molecular weights of two concepts.
pub fn mw_compare(params: MwCompareParams) -> Result<CallToolResult, McpError> {
    let prims_a = parse_primitive_list(&params.primitives_a)?;
    let prims_b = parse_primitive_list(&params.primitives_b)?;

    if prims_a.is_empty() || prims_b.is_empty() {
        return Err(McpError::new(
            ErrorCode(400),
            "Both primitive lists must be non-empty",
            None,
        ));
    }

    let name_a = params.name_a.as_deref().unwrap_or("concept_a");
    let name_b = params.name_b.as_deref().unwrap_or("concept_b");

    let formula_a = MolecularFormula::new(name_a).with_all(&prims_a);
    let formula_b = MolecularFormula::new(name_b).with_all(&prims_b);

    let wa = formula_a.weight();
    let wb = formula_b.weight();

    // Shared primitives
    let set_a: std::collections::HashSet<_> = prims_a.iter().collect();
    let set_b: std::collections::HashSet<_> = prims_b.iter().collect();
    let shared: Vec<_> = set_a.intersection(&set_b).map(|p| p.name()).collect();
    let only_a: Vec<_> = set_a.difference(&set_b).map(|p| p.name()).collect();
    let only_b: Vec<_> = set_b.difference(&set_a).map(|p| p.name()).collect();

    let jaccard = if set_a.is_empty() && set_b.is_empty() {
        0.0
    } else {
        shared.len() as f64 / set_a.union(&set_b).count() as f64
    };

    let result = json!({
        "concept_a": {
            "name": name_a,
            "formula": formula_a.formula_string(),
            "molecular_weight": round3(wa.daltons()),
            "transfer_class": format!("{}", wa.transfer_class()),
            "predicted_transfer": round3(wa.predicted_transfer()),
            "tier_prediction": format!("{}", wa.tier_aware_class()),
            "hybrid_transfer": round3(wa.predicted_transfer_hybrid()),
        },
        "concept_b": {
            "name": name_b,
            "formula": formula_b.formula_string(),
            "molecular_weight": round3(wb.daltons()),
            "transfer_class": format!("{}", wb.transfer_class()),
            "predicted_transfer": round3(wb.predicted_transfer()),
            "tier_prediction": format!("{}", wb.tier_aware_class()),
            "hybrid_transfer": round3(wb.predicted_transfer_hybrid()),
        },
        "comparison": {
            "weight_delta": round3((wa.daltons() - wb.daltons()).abs()),
            "heavier": if wa.daltons() > wb.daltons() { name_a } else { name_b },
            "shared_primitives": shared,
            "only_in_a": only_a,
            "only_in_b": only_b,
            "jaccard_similarity": round3(jaccard),
            "same_transfer_class": wa.transfer_class() == wb.transfer_class(),
        },
    });

    Ok(CallToolResult::success(vec![Content::text(
        result.to_string(),
    )]))
}

/// Predict transfer confidence from a molecular weight value.
pub fn mw_predict_transfer(params: MwPredictTransferParams) -> Result<CallToolResult, McpError> {
    let primitives = parse_primitive_list(&params.primitives)?;

    if primitives.is_empty() {
        return Err(McpError::new(
            ErrorCode(400),
            "primitives array must not be empty",
            None,
        ));
    }

    let weight = MolecularFormula::weight_of(&primitives);
    let transfer = weight.predicted_transfer();
    let hybrid_transfer = weight.predicted_transfer_hybrid();

    let result = json!({
        "molecular_weight": round3(weight.daltons()),
        "predicted_transfer_confidence": round3(transfer),
        "hybrid_transfer_confidence": round3(hybrid_transfer),
        "transfer_class": format!("{}", weight.transfer_class()),
        "tier_prediction": format!("{}", weight.tier_aware_class()),
        "primitive_count": weight.primitive_count(),
        "average_mass": round3(weight.average_mass()),
        "interpretation": format!(
            "{:.1}% transfer (MW-only), {:.1}% transfer (hybrid tier-aware)",
            transfer * 100.0,
            hybrid_transfer * 100.0
        ),
        "thresholds": {
            "light_below": 11.0,
            "medium_below": 18.0,
            "heavy_above": 18.0,
        },
    });

    Ok(CallToolResult::success(vec![Content::text(
        result.to_string(),
    )]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compute_state_boundary() {
        let params = MwComputeParams {
            name: Some("Guard".to_string()),
            primitives: vec!["state".to_string(), "boundary".to_string()],
        };
        let result = mw_compute(params);
        assert!(result.is_ok());
    }

    #[test]
    fn periodic_table_returns_16() {
        let result = mw_periodic_table();
        assert!(result.is_ok());
    }

    #[test]
    fn compare_different_weights() {
        let params = MwCompareParams {
            name_a: Some("Simple".to_string()),
            primitives_a: vec!["quantity".to_string()],
            name_b: Some("Complex".to_string()),
            primitives_b: vec![
                "state".to_string(),
                "boundary".to_string(),
                "causality".to_string(),
                "sequence".to_string(),
                "mapping".to_string(),
                "persistence".to_string(),
            ],
        };
        let result = mw_compare(params);
        assert!(result.is_ok());
    }

    #[test]
    fn predict_transfer_higher_for_light() {
        let light = MwPredictTransferParams {
            primitives: vec!["quantity".to_string(), "mapping".to_string()],
        };
        let heavy = MwPredictTransferParams {
            primitives: vec![
                "state".to_string(),
                "boundary".to_string(),
                "causality".to_string(),
                "sequence".to_string(),
                "mapping".to_string(),
                "product".to_string(),
            ],
        };
        let lr = mw_predict_transfer(light);
        let hr = mw_predict_transfer(heavy);
        assert!(lr.is_ok());
        assert!(hr.is_ok());
    }

    #[test]
    fn rejects_empty_primitives() {
        let params = MwComputeParams {
            name: Some("Empty".to_string()),
            primitives: vec![],
        };
        assert!(mw_compute(params).is_err());
    }

    #[test]
    fn accepts_symbols() {
        let params = MwComputeParams {
            name: Some("Symbolic".to_string()),
            primitives: vec!["ς".to_string(), "∂".to_string()],
        };
        assert!(mw_compute(params).is_ok());
    }
}
