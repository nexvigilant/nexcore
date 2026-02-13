//! CEP (Cognitive Evolution Pipeline) tools
//!
//! 8-stage knowledge discovery pipeline with feedback loops.
//! Patent: NV-2026-001

use crate::params::{
    CepExecuteStageParams, CepValidateExtractionParams, DomainTranslateParams,
    PrimitiveExtractParams,
};
use nexcore_vigilance::cep::{ExtractionValidation, StageId, ValidationThresholds};
use nexcore_vigilance::domain_discovery::PrimitiveTier;
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use serde_json::json;

/// Execute a single CEP stage
pub fn execute_stage(params: CepExecuteStageParams) -> Result<CallToolResult, McpError> {
    let stage = match params.stage.to_uppercase().as_str() {
        "SEE" | "1" => StageId::See,
        "SPEAK" | "2" => StageId::Speak,
        "DECOMPOSE" | "3" => StageId::Decompose,
        "COMPOSE" | "4" => StageId::Compose,
        "TRANSLATE" | "5" => StageId::Translate,
        "VALIDATE" | "6" => StageId::Validate,
        "DEPLOY" | "7" => StageId::Deploy,
        "IMPROVE" | "8" => StageId::Improve,
        _ => {
            return Ok(CallToolResult::success(vec![Content::text(
                json!({
                    "error": "Invalid stage. Use SEE, SPEAK, DECOMPOSE, COMPOSE, TRANSLATE, VALIDATE, DEPLOY, or IMPROVE (or 1-8)"
                }).to_string(),
            )]));
        }
    };

    let json = json!({
        "stage": stage.name(),
        "stage_number": stage.number(),
        "domain": params.domain,
        "status": "ready",
        "description": match stage {
            StageId::See => "Observe phenomena without prejudice",
            StageId::Speak => "Articulate into structured vocabulary with preliminary tier assignment",
            StageId::Decompose => "Extract T1/T2/T3 primitives via DAG topological analysis",
            StageId::Compose => "Synthesize novel structures from primitives",
            StageId::Translate => "Bidirectional cross-domain mapping with confidence scores",
            StageId::Validate => "Verify coverage ≥0.95, minimality ≥0.90, independence ≥0.90",
            StageId::Deploy => "Generate operational artifacts and monitoring",
            StageId::Improve => "Aggregate feedback signals for next cycle",
        },
        "next_stage": stage.next().map(|s| s.name()),
        "note": "Use cep_pipeline_run for full pipeline execution"
    });

    Ok(CallToolResult::success(vec![Content::text(
        json.to_string(),
    )]))
}

/// Get CEP pipeline stages information
pub fn pipeline_stages() -> Result<CallToolResult, McpError> {
    let stages = [
        StageId::See,
        StageId::Speak,
        StageId::Decompose,
        StageId::Compose,
        StageId::Translate,
        StageId::Validate,
        StageId::Deploy,
        StageId::Improve,
    ];

    let stage_info: Vec<_> = stages
        .iter()
        .map(|s| {
            json!({
                "number": s.number(),
                "name": s.name(),
                "next": s.next().map(|n| n.name()),
            })
        })
        .collect();

    let json = json!({
        "pipeline": "Cognitive Evolution Pipeline (CEP)",
        "patent": "NV-2026-001",
        "stages": stage_info,
        "feedback_loop": "Stage 8 (IMPROVE) feeds back to Stage 1 (SEE)",
        "thresholds": {
            "coverage": "≥ 0.95",
            "minimality": "≥ 0.90",
            "independence": "≥ 0.90"
        }
    });

    Ok(CallToolResult::success(vec![Content::text(
        json.to_string(),
    )]))
}

/// Validate primitive extraction quality
pub fn validate_extraction(
    params: CepValidateExtractionParams,
) -> Result<CallToolResult, McpError> {
    let thresholds = ValidationThresholds {
        coverage: params.coverage_threshold.unwrap_or(0.95),
        minimality: params.minimality_threshold.unwrap_or(0.90),
        independence: params.independence_threshold.unwrap_or(0.90),
    };

    let validation = ExtractionValidation::with_thresholds(
        params.coverage,
        params.minimality,
        params.independence,
        thresholds,
    );

    let (weakest_name, weakest_value) = validation.weakest_metric();

    let json = json!({
        "is_valid": validation.is_valid,
        "metrics": {
            "coverage": {
                "value": validation.coverage.0,
                "threshold": thresholds.coverage,
                "passed": validation.coverage.0 >= thresholds.coverage
            },
            "minimality": {
                "value": validation.minimality.0,
                "threshold": thresholds.minimality,
                "passed": validation.minimality.0 >= thresholds.minimality
            },
            "independence": {
                "value": validation.independence.0,
                "threshold": thresholds.independence,
                "passed": validation.independence.0 >= thresholds.independence
            }
        },
        "weakest_metric": {
            "name": weakest_name,
            "value": weakest_value
        },
        "patent_ref": "NV-2026-002 (thresholds defined in §5.4)"
    });

    Ok(CallToolResult::success(vec![Content::text(
        json.to_string(),
    )]))
}

/// Classify primitive into T1/T2/T3 tier
pub fn classify_primitive(domain_count: usize) -> Result<CallToolResult, McpError> {
    let (tier, confidence, description) = if domain_count >= 10 {
        (
            PrimitiveTier::T1Universal,
            1.0,
            "Universal ontological bedrock - appears in 10+ domains",
        )
    } else if domain_count >= 2 {
        let conf = 0.85 + (domain_count as f64 / 100.0);
        (
            PrimitiveTier::T2CrossDomain,
            conf.min(0.95),
            "Cross-domain transferable - appears in 2-9 domains",
        )
    } else {
        (
            PrimitiveTier::T3DomainSpecific,
            0.90,
            "Domain-specific - unique to this domain",
        )
    };

    let json = json!({
        "domain_count": domain_count,
        "tier": format!("{:?}", tier),
        "tier_name": tier.name(),
        "confidence": confidence,
        "description": description,
        "min_coverage_for_tier": tier.min_coverage(),
        "patent_ref": "NV-2026-002 §4 Tier Classification"
    });

    Ok(CallToolResult::success(vec![Content::text(
        json.to_string(),
    )]))
}

/// Extract primitives from a domain (stub for agent-based extraction)
pub fn extract_primitives(params: PrimitiveExtractParams) -> Result<CallToolResult, McpError> {
    // This is a stub - actual extraction requires LLM agent
    let json = json!({
        "domain": params.domain,
        "mode": params.mode.unwrap_or_else(|| "standard".to_string()),
        "status": "ready",
        "steps": [
            "1. Collect vocabulary from domain corpus",
            "2. Build dependency graph G = (V, E)",
            "3. Topological sort to find roots (in-degree = 0)",
            "4. Classify by domain coverage: T1 (≥10), T2 (2-9), T3 (1)",
            "5. Validate: coverage ≥0.95, minimality ≥0.90, independence ≥0.90"
        ],
        "note": "Use /primitive-extractor skill for full extraction with LLM assistance",
        "patent_ref": "NV-2026-002"
    });

    Ok(CallToolResult::success(vec![Content::text(
        json.to_string(),
    )]))
}

/// Translate concept between domains
pub fn domain_translate(params: DomainTranslateParams) -> Result<CallToolResult, McpError> {
    // Simplified translation logic - actual translation requires full mapping DB
    let json = json!({
        "source_domain": params.source_domain,
        "target_domain": params.target_domain,
        "concept": params.concept,
        "translation_rules": {
            "T1_Universal": "Identity mapping (confidence = 1.0)",
            "T2_CrossDomain": "Similarity-based matching (confidence 0.85-0.95)",
            "T3_DomainSpecific": "Novel synthesis via decomposition + recomposition (confidence 0.70-0.85)"
        },
        "status": "ready",
        "note": "Use /domain-translator skill for full bidirectional translation with confidence scores",
        "patent_ref": "NV-2026-001 Stage 5 (TRANSLATE)"
    });

    Ok(CallToolResult::success(vec![Content::text(
        json.to_string(),
    )]))
}

/// Get tier counts summary
pub fn tier_summary(t1: usize, t2: usize, t3: usize) -> Result<CallToolResult, McpError> {
    let total = t1 + t2 + t3;
    let json = json!({
        "tier_counts": {
            "T1_Universal": t1,
            "T2_CrossDomain": t2,
            "T3_DomainSpecific": t3,
            "total": total
        },
        "distribution": {
            "T1_percent": if total > 0 { (t1 as f64 / total as f64) * 100.0 } else { 0.0 },
            "T2_percent": if total > 0 { (t2 as f64 / total as f64) * 100.0 } else { 0.0 },
            "T3_percent": if total > 0 { (t3 as f64 / total as f64) * 100.0 } else { 0.0 },
        },
        "note": "Healthy extraction: T1 provides bedrock, T2 enables transfer, T3 captures domain specifics"
    });

    Ok(CallToolResult::success(vec![Content::text(
        json.to_string(),
    )]))
}
