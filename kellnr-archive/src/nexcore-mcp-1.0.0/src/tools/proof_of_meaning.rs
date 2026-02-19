//! Proof of Meaning MCP tools
//!
//! Chemistry-inspired semantic transformation engine for provable
//! regulatory terminology equivalence. Exposes distillation, chromatography,
//! titration, equivalence proofs, and registry statistics.

use rmcp::ErrorData as McpError;
use rmcp::model::CallToolResult;

use nexcore_proof_of_meaning::chromatography::Column;
use nexcore_proof_of_meaning::distillation::Distiller;
use nexcore_proof_of_meaning::element::ElementClass;
use nexcore_proof_of_meaning::pipeline::ProofPipeline;
use nexcore_proof_of_meaning::registry::AtomRegistry;

use crate::params::{
    PomChromatographParams, PomDistillParams, PomProveEquivalenceParams, PomRegistryStatsParams,
    PomTitrateParams,
};

/// Distill a regulatory expression into atoms ordered by volatility.
pub fn pom_distill(params: PomDistillParams) -> Result<CallToolResult, McpError> {
    let distiller = Distiller::new();
    let result = distiller.distill(&params.expression);

    let fractions: Vec<serde_json::Value> = result
        .fractions
        .iter()
        .map(|f| {
            serde_json::json!({
                "atom": f.atom.label,
                "class": format!("{:?}", f.atom.class),
                "volatility": f.atom.volatility.into_inner(),
                "separation_temperature": f.separation_temperature.into_inner(),
                "purity": f.purity.into_inner(),
                "fraction_number": f.fraction_number,
            })
        })
        .collect();

    let json = serde_json::json!({
        "input_expression": result.input_expression,
        "fractions": fractions,
        "fraction_count": result.fractions.len(),
        "residue_count": result.residue.len(),
        "mass_balance": {
            "input_mass": result.mass_balance.input_mass,
            "recovered_mass": result.mass_balance.recovered_mass,
            "loss_percent": result.mass_balance.loss_percent,
            "verdict": format!("{:?}", result.mass_balance.verdict),
        },
    });

    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string_pretty(&json).unwrap_or_default(),
    )]))
}

/// Classify atoms into hierarchy positions via chromatographic separation.
pub fn pom_chromatograph(params: PomChromatographParams) -> Result<CallToolResult, McpError> {
    let column = Column::pv_standard();
    let result = column.separate(&params.expression);

    let bands: Vec<serde_json::Value> = result
        .bands
        .iter()
        .map(|b| {
            let alts: Vec<serde_json::Value> = b
                .alternative_sites
                .iter()
                .map(|(class, aff)| {
                    serde_json::json!({
                        "class": format!("{class:?}"),
                        "affinity": aff.into_inner(),
                    })
                })
                .collect();
            serde_json::json!({
                "atom_label": b.atom_label,
                "bound_class": format!("{:?}", b.bound_class),
                "binding_affinity": b.binding_affinity.into_inner(),
                "bandwidth": b.bandwidth.into_inner(),
                "alternative_sites": alts,
            })
        })
        .collect();

    let resolutions: Vec<serde_json::Value> = result
        .resolution_scores
        .iter()
        .map(|r| {
            serde_json::json!({
                "band_a": r.band_a,
                "band_b": r.band_b,
                "resolution": r.resolution.into_inner(),
            })
        })
        .collect();

    let json = serde_json::json!({
        "input_expression": result.input_expression,
        "bands": bands,
        "band_count": result.bands.len(),
        "resolution_scores": resolutions,
        "quality": format!("{:?}", result.quality),
    });

    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string_pretty(&json).unwrap_or_default(),
    )]))
}

/// Titrate an expression against canonical standards to measure meaning coverage.
pub fn pom_titrate(params: PomTitrateParams) -> Result<CallToolResult, McpError> {
    let pipeline = ProofPipeline::pv_standard();
    let trail = pipeline.transform(&params.expression);

    let steps: Vec<serde_json::Value> = trail
        .steps
        .iter()
        .map(|s| {
            serde_json::json!({
                "step_number": s.step_number,
                "method": format!("{:?}", s.method),
                "input": s.input_description,
                "output": s.output_description,
                "verification": format!("{:?}", s.verification),
            })
        })
        .collect();

    let json = serde_json::json!({
        "input_expression": trail.input_expression,
        "steps": steps,
        "step_count": trail.steps.len(),
        "trail_valid": trail.trail_valid,
        "warnings": trail.warnings,
    });

    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string_pretty(&json).unwrap_or_default(),
    )]))
}

/// Prove semantic equivalence between two regulatory expressions.
pub fn pom_prove_equivalence(
    params: PomProveEquivalenceParams,
) -> Result<CallToolResult, McpError> {
    let pipeline = ProofPipeline::pv_standard();
    let proof = pipeline.prove_equivalence(&params.expression_a, &params.expression_b);

    let json = serde_json::json!({
        "expression_a": params.expression_a,
        "expression_b": params.expression_b,
        "proof_valid": proof.proof_valid,
        "equivalence": {
            "shared_atoms": proof.equivalence.shared_atoms,
            "unique_to_a": proof.equivalence.unique_to_a,
            "unique_to_b": proof.equivalence.unique_to_b,
            "equivalence_score": proof.equivalence.equivalence_score.into_inner(),
            "verdict": format!("{:?}", proof.equivalence.verdict),
        },
        "trail_a": {
            "valid": proof.trail_a.trail_valid,
            "steps": proof.trail_a.steps.len(),
            "warnings": proof.trail_a.warnings,
        },
        "trail_b": {
            "valid": proof.trail_b.trail_valid,
            "steps": proof.trail_b.steps.len(),
            "warnings": proof.trail_b.warnings,
        },
    });

    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string_pretty(&json).unwrap_or_default(),
    )]))
}

/// Get registry statistics — atom count by class and total.
pub fn pom_registry_stats(_params: PomRegistryStatsParams) -> Result<CallToolResult, McpError> {
    let mut registry = AtomRegistry::new();
    registry.seed_all();

    let classes = ElementClass::all();
    let by_class: Vec<serde_json::Value> = classes
        .iter()
        .map(|c| {
            serde_json::json!({
                "class": format!("{c:?}"),
                "count": registry.count_by_class(c),
            })
        })
        .collect();

    let json = serde_json::json!({
        "total_atoms": registry.count(),
        "class_count": classes.len(),
        "by_class": by_class,
    });

    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string_pretty(&json).unwrap_or_default(),
    )]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pom_distill() {
        let params = PomDistillParams {
            expression: "serious cardiac adverse event".to_string(),
        };
        let result = pom_distill(params);
        assert!(result.is_ok());
    }

    #[test]
    fn test_pom_chromatograph() {
        let params = PomChromatographParams {
            expression: "serious cardiac adverse event".to_string(),
        };
        let result = pom_chromatograph(params);
        assert!(result.is_ok());
    }

    #[test]
    fn test_pom_titrate() {
        let params = PomTitrateParams {
            expression: "cardiac adverse event".to_string(),
        };
        let result = pom_titrate(params);
        assert!(result.is_ok());
    }

    #[test]
    fn test_pom_prove_equivalence() {
        let params = PomProveEquivalenceParams {
            expression_a: "cardiac adverse event".to_string(),
            expression_b: "cardiac adverse reaction".to_string(),
        };
        let result = pom_prove_equivalence(params);
        assert!(result.is_ok());
    }

    #[test]
    fn test_pom_registry_stats() {
        let params = PomRegistryStatsParams {};
        let result = pom_registry_stats(params);
        assert!(result.is_ok());
    }
}
