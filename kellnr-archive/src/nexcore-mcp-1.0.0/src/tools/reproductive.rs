//! # Reproductive System MCP Tools
//!
//! Tools for biological optimization: Genetic Guarding, Tissue Specialization,
//! and Mitotic Repair.

use crate::params::{
    ReproductiveGuardMutationParams, ReproductiveSpecializeAgentParams,
    ReproductiveStartMitosisParams,
};
use nexcore_reproductive::genetics::{GeneticGuard, GenomeRequirement};
use nexcore_reproductive::mitosis::{FailingCell, MitoticRepair};
use nexcore_reproductive::phenotypes::{SomaticSpecialization, TissuePhenotype};
use nexcore_lex_primitiva::LexPrimitiva;
use rmcp::model::CallToolResult;
use serde_json::json;

/// Checks if a proposed code change (mutation) is architectural lethal.
pub fn guard_mutation(params: ReproductiveGuardMutationParams) -> Result<CallToolResult, rmcp::ErrorData> {
    let mut primitives = Vec::new();
    for s in params.primitives {
        match s.as_str() {
            "Sequence" | "σ" => primitives.push(LexPrimitiva::Sequence),
            "Mapping" | "μ" => primitives.push(LexPrimitiva::Mapping),
            "State" | "ς" => primitives.push(LexPrimitiva::State),
            "Recursion" | "ρ" => primitives.push(LexPrimitiva::Recursion),
            "Void" | "∅" => primitives.push(LexPrimitiva::Void),
            "Boundary" | "∂" => primitives.push(LexPrimitiva::Boundary),
            "Frequency" | "ν" => primitives.push(LexPrimitiva::Frequency),
            "Existence" | "∃" => primitives.push(LexPrimitiva::Existence),
            "Persistence" | "π" => primitives.push(LexPrimitiva::Persistence),
            "Causality" | "→" => primitives.push(LexPrimitiva::Causality),
            "Comparison" | "κ" => primitives.push(LexPrimitiva::Comparison),
            "Quantity" | "N" => primitives.push(LexPrimitiva::Quantity),
                                    "Location" | "λ" => primitives.push(LexPrimitiva::Location),
                                    "Irreversibility" | "∝" => primitives.push(LexPrimitiva::Irreversibility),
                                    "Sum" | "Σ" => primitives.push(LexPrimitiva::Sum),            _ => {}
        }
    }

    let guard = GeneticGuard::new(GenomeRequirement::default());
    let lethal = guard.is_mutation_lethal(&primitives, params.uses_unsafe);

    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        json!({
            "is_lethal": lethal,
            "verdict": if lethal { "Lethal Mutation: Architectural drift detected." } else { "Viable Mutation: Bedrock preserved." }
        }).to_string()
    )]))
}

/// Gets parameters for a specialized subagent tissue phenotype.
pub fn specialize_agent(params: ReproductiveSpecializeAgentParams) -> Result<CallToolResult, rmcp::ErrorData> {
    let phenotype = match params.phenotype.as_str() {
        "Nervous" => TissuePhenotype::Nervous,
        "Immune" => TissuePhenotype::Immune,
        "Muscular" => TissuePhenotype::Muscular,
        "Germ" => TissuePhenotype::Germ,
        _ => return Err(rmcp::ErrorData::invalid_params(format!("Unknown phenotype: {}", params.phenotype), None)),
    };

    let spec = SomaticSpecialization::for_phenotype(phenotype);
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string_pretty(&spec).unwrap_or_default()
    )]))
}

/// Initializes a mitotic repair cycle for a failing crate.
pub fn start_mitosis(params: ReproductiveStartMitosisParams) -> Result<CallToolResult, rmcp::ErrorData> {
    let cell = FailingCell {
        name: params.crate_name,
        error_type: params.error_type,
        severity: params.severity,
    };

    let repair = MitoticRepair::new(cell);
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string_pretty(&repair).unwrap_or_default()
    )]))
}
