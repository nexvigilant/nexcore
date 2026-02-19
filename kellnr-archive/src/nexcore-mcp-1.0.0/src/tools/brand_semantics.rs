//! Brand Semantics tools: primitive extraction and decomposition for brand names
//!
//! Exposes the NexVigilant brand decomposition methodology via MCP.

use crate::params::{BrandDecompositionGetParams, BrandPrimitiveTestParams};
use nexcore_vigilance::vocab::brand_semantics::{
    PrimitiveTest, PrimitiveVerdict, SemanticTier, TestResult, nexvigilant_decomposition,
};
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use serde_json::json;

/// Get the pre-computed NexVigilant brand decomposition
pub fn brand_decomposition_nexvigilant() -> Result<CallToolResult, McpError> {
    let decomp = nexvigilant_decomposition();

    let json = json!({
        "brand_name": decomp.brand_name,
        "extraction_date": decomp.extraction_date,
        "source_mode": decomp.source_mode,
        "source_warning": decomp.source_warning,
        "semantic_field": decomp.semantic_field,
        "roots": decomp.roots.iter().map(|r| json!({
            "root": r.root,
            "original_form": r.original_form,
            "meaning": r.meaning,
            "language": format!("{}", r.language),
        })).collect::<Vec<_>>(),
        "primitives": {
            "t1_universal": decomp.primitives.t1_universal.iter().map(|p| json!({
                "name": p.name,
                "definition": p.definition,
                "grounds_to": p.grounds_to,
            })).collect::<Vec<_>>(),
            "t2_primitives": decomp.primitives.t2_primitives.iter().map(|p| json!({
                "name": p.name,
                "domains_present": p.domains_present,
            })).collect::<Vec<_>>(),
            "t2_composites": decomp.primitives.t2_composites.iter().map(|p| json!({
                "name": p.name,
                "components": p.components,
                "note": p.note,
            })).collect::<Vec<_>>(),
            "t3_domain_specific": decomp.primitives.t3_domain_specific.iter().map(|p| json!({
                "name": p.name,
                "components": p.components,
            })).collect::<Vec<_>>(),
        },
        "primitive_tests": decomp.primitive_tests.iter().map(|t| json!({
            "term": t.term,
            "definition": t.definition,
            "test1_no_domain_deps": format!("{}", t.test1_no_domain_deps),
            "domain_terms_found": t.domain_terms_found,
            "test2_external_grounding": format!("{}", t.test2_external_grounding),
            "external_grounding": t.external_grounding,
            "test3_not_synonym": format!("{}", t.test3_not_synonym),
            "synonym_analysis": t.synonym_analysis,
            "verdict": format!("{:?}", t.verdict),
            "tier": t.tier.label(),
            "notes": t.notes,
        })).collect::<Vec<_>>(),
        "transfer_mappings": decomp.transfer_mappings.iter().map(|m| json!({
            "source": m.source,
            "target": m.target,
            "target_domain": m.target_domain,
            "confidence": {
                "overall": m.confidence.overall,
                "structural": m.confidence.structural,
                "functional": m.confidence.functional,
                "contextual": m.confidence.contextual,
                "limiting_factor": m.confidence.limiting_factor(),
            },
            "caveat": m.caveat,
        })).collect::<Vec<_>>(),
        "semantic_formula": decomp.semantic_formula,
        "interpretation": decomp.interpretation,
        "brand_insight": decomp.brand_insight,
        "validation": {
            "terms_analyzed": decomp.terms_analyzed,
            "primitives_identified": decomp.primitives_identified,
            "composites_identified": decomp.composites_identified,
        },
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&json).unwrap_or_default(),
    )]))
}

/// Get a brand decomposition by name (currently only "nexvigilant" is pre-computed)
pub fn brand_decomposition_get(
    params: BrandDecompositionGetParams,
) -> Result<CallToolResult, McpError> {
    let name = params.name.to_lowercase();

    if name == "nexvigilant" || name == "nex-vigilant" || name == "nex_vigilant" {
        brand_decomposition_nexvigilant()
    } else {
        let json = json!({
            "error": format!("Brand '{}' not found in pre-computed decompositions", params.name),
            "available": ["nexvigilant"],
            "hint": "Use brand_primitive_test to test individual terms, or request a new brand decomposition"
        });
        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&json).unwrap_or_default(),
        )]))
    }
}

/// Run a primitive test on a term
pub fn brand_primitive_test(params: BrandPrimitiveTestParams) -> Result<CallToolResult, McpError> {
    let mut test = PrimitiveTest::new(&params.term, &params.definition);

    // Test 1: No Domain-Internal Dependencies
    let domain_terms: Vec<String> = params.domain_terms_in_definition.unwrap_or_else(Vec::new);

    let test1_result = if domain_terms.is_empty() {
        TestResult::Pass
    } else {
        TestResult::Fail
    };
    test = test.with_test1(test1_result, domain_terms.clone());

    // Test 2: Grounds to External Concepts
    let external: Vec<String> = params.external_grounding.unwrap_or_else(Vec::new);

    let test2_result = if external.is_empty() {
        TestResult::Fail
    } else {
        TestResult::Pass
    };
    test = test.with_test2(test2_result, external.clone());

    // Test 3: Not Merely a Synonym
    let synonym_analysis = params
        .synonym_analysis
        .unwrap_or_else(|| "Not analyzed".to_string());
    let test3_result = if params.is_synonym.unwrap_or(false) {
        TestResult::Fail
    } else {
        TestResult::Pass
    };
    test = test.with_test3(test3_result, &synonym_analysis);

    // Compute verdict
    let verdict = test.compute_verdict();

    // Determine tier based on domain coverage
    let tier = if verdict == PrimitiveVerdict::Composite {
        if params.domain_count.unwrap_or(1) > 1 {
            SemanticTier::T2Composite
        } else {
            SemanticTier::T3DomainSpecific
        }
    } else if params.domain_count.unwrap_or(1) >= 10 {
        SemanticTier::T1Universal
    } else if params.domain_count.unwrap_or(1) > 1 {
        SemanticTier::T2Primitive
    } else {
        SemanticTier::T3DomainSpecific
    };

    test = test.with_verdict(verdict, tier);

    // Build response
    let json = json!({
        "term": test.term,
        "definition": test.definition,
        "tests": {
            "test1_no_domain_deps": {
                "result": format!("{}", test.test1_no_domain_deps),
                "domain_terms_found": domain_terms,
                "explanation": if domain_terms.is_empty() {
                    "No domain-specific terms in definition"
                } else {
                    "Definition contains domain-specific terms"
                }
            },
            "test2_external_grounding": {
                "result": format!("{}", test.test2_external_grounding),
                "external_concepts": external,
                "explanation": if external.is_empty() {
                    "No external grounding provided"
                } else {
                    "Term grounds to external concepts"
                }
            },
            "test3_not_synonym": {
                "result": format!("{}", test.test3_not_synonym),
                "analysis": synonym_analysis,
                "explanation": if params.is_synonym.unwrap_or(false) {
                    "Term is merely a synonym of another"
                } else {
                    "Term carries distinct semantic load"
                }
            }
        },
        "verdict": format!("{:?}", verdict),
        "tier": tier.label(),
        "tier_name": tier.name(),
        "is_primitive": tier.is_primitive(),
        "rationale": match verdict {
            PrimitiveVerdict::Primitive => "All tests pass - term is irreducible",
            PrimitiveVerdict::Composite => "Test 1 failed - term decomposes further",
            PrimitiveVerdict::Undetermined => "Borderline results - requires judgment",
        }
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&json).unwrap_or_default(),
    )]))
}

/// List available semantic tiers with descriptions
pub fn brand_semantic_tiers() -> Result<CallToolResult, McpError> {
    let tiers = vec![
        json!({
            "tier": "T1",
            "name": "T1 Universal",
            "description": "Irreducible symbols (σ, μ, ς, ρ, ∅, ∂, f, ∃, π, →, κ, N, λ, ∝) across ALL domains",
            "examples": ["Sequence (σ)", "Causality (→)", "Quantity (N)"],
            "transfer": "Direct mapping; free translation"
        }),
        json!({
            "tier": "T2-P",
            "name": "T2-P Cross-Domain Primitive",
            "description": "Atomic atoms built from T1 that transfer across 2+ domains",
            "examples": ["Validation", "Threshold", "Timing"],
            "transfer": "Find domain instantiation"
        }),
        json!({
            "tier": "T2-C",
            "name": "T2-C Cross-Domain Composite",
            "description": "Architectural patterns built from T1/T2-P components",
            "examples": ["Pipeline", "Loop", "Session"],
            "transfer": "Map component primitives"
        }),
        json!({
            "tier": "T3",
            "name": "T3 Domain-Specific",
            "description": "Complex constructs unique to specific applications",
            "examples": ["ICSR", "MedDRA", "adverse event"],
            "transfer": "Find equivalent in target domain"
        }),
    ];

    let json = json!({
        "tiers": tiers,
        "classification_logic": {
            "T1": "domains == ALL AND irreducible",
            "T2-P": "domains > 1 AND irreducible within domain",
            "T2-C": "domains > 1 AND has internal dependencies",
            "T3": "domains == 1"
        },
        "primitive_test": {
            "test1": "No domain-internal dependencies",
            "test2": "Grounds to T1 symbols",
            "test3": "Not merely a synonym"
        }
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&json).unwrap_or_default(),
    )]))
}
