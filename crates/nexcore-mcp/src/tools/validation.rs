//! Universal validation tools: L1-L5 validation, domain detection
//!
//! Cross-domain validation using the L1-L5 validation stack.

use std::path::Path;

use crate::params::{ValidationCheckParams, ValidationDomainsParams, ValidationRunParams};
use crate::tooling::attach_forensic_meta;
use nexcore_vigilance::validation::{
    DomainRegistry, UniversalValidator, ValidationLevel, engine::ValidationConfig,
};
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use serde_json::json;

/// Run full L1-L5 validation
pub fn run(params: ValidationRunParams) -> Result<CallToolResult, McpError> {
    let target_path = Path::new(&params.target);

    // Get domain (auto-detect or explicit)
    let domain = params
        .domain
        .as_deref()
        .unwrap_or_else(|| DomainRegistry::detect_domain(target_path));

    // Get max level
    let max_level = params
        .max_level
        .as_deref()
        .and_then(parse_level)
        .unwrap_or(ValidationLevel::L5Impact);

    // Create validator
    let domain_validator = DomainRegistry::get_validator(domain);
    let validator = UniversalValidator::new(domain_validator);

    // Configure
    let config = ValidationConfig::default()
        .with_max_level(max_level)
        .with_fail_fast(params.fail_fast.unwrap_or(true))
        .silent(); // Don't emit to stderr in MCP

    // Run validation
    let result = validator.validate_with_config(&params.target, config);

    // Build response
    let levels_json: serde_json::Map<String, serde_json::Value> = result
        .levels
        .iter()
        .map(|(level, lr)| {
            (
                format!("L{}", level.number()),
                json!({
                    "name": level.name(),
                    "status": format!("{:?}", lr.status),
                    "checks_passed": lr.checks_passed(),
                    "checks_total": lr.checks_total(),
                    "duration_ms": lr.duration_ms,
                    "checks": lr.checks.iter().map(|c| {
                        json!({
                            "name": c.name,
                            "status": format!("{:?}", c.status),
                            "message": c.message,
                            "severity": format!("{:?}", c.severity),
                        })
                    }).collect::<Vec<_>>(),
                }),
            )
        })
        .collect();

    let response = json!({
        "target": result.target,
        "domain": result.domain,
        "overall_status": format!("{:?}", result.overall_status),
        "highest_passed": result.highest_passed.map(|l| format!("L{}", l.number())),
        "score": result.score,
        "duration_ms": result.duration_ms,
        "levels": levels_json,
        "timestamp": result.timestamp,
    });

    let passed = matches!(
        result.overall_status,
        nexcore_vigilance::validation::ValidationStatus::Green
    );
    let mut res = CallToolResult::success(vec![Content::text(response.to_string())]);
    attach_forensic_meta(&mut res, result.score / 100.0, Some(passed), "validation");
    Ok(res)
}

/// Quick check (L1-L2 only)
pub fn check(params: ValidationCheckParams) -> Result<CallToolResult, McpError> {
    run(ValidationRunParams {
        target: params.target,
        domain: params.domain,
        max_level: Some("L2".to_string()),
        fail_fast: Some(true),
    })
}

/// List available validation domains
pub fn domains(_params: ValidationDomainsParams) -> Result<CallToolResult, McpError> {
    let domains: Vec<_> = DomainRegistry::list_domains()
        .iter()
        .map(|d| {
            json!({
                "name": d.name,
                "description": d.description,
                "patterns": d.patterns,
                "priority": d.priority,
            })
        })
        .collect();

    let response = json!({
        "domains": domains,
        "count": domains.len(),
        "levels": [
            {"level": "L1", "name": "Coherence", "question": "Is this internally consistent?"},
            {"level": "L2", "name": "Structural", "question": "Is this built correctly per spec?"},
            {"level": "L3", "name": "Functional", "question": "Does this produce correct outputs?"},
            {"level": "L4", "name": "Operational", "question": "Does this work reliably?"},
            {"level": "L5", "name": "Impact", "question": "Does this achieve outcomes?"},
        ],
    });

    Ok(CallToolResult::success(vec![Content::text(
        response.to_string(),
    )]))
}

fn parse_level(s: &str) -> Option<ValidationLevel> {
    match s.to_uppercase().as_str() {
        "L1" | "L1_COHERENCE" => Some(ValidationLevel::L1Coherence),
        "L2" | "L2_STRUCTURAL" => Some(ValidationLevel::L2Structural),
        "L3" | "L3_FUNCTIONAL" => Some(ValidationLevel::L3Functional),
        "L4" | "L4_OPERATIONAL" => Some(ValidationLevel::L4Operational),
        "L5" | "L5_IMPACT" => Some(ValidationLevel::L5Impact),
        _ => None,
    }
}

// ============================================================================
// Test Classification Tools
// ============================================================================

use crate::params::ValidationClassifyTestsParams;
use nexcore_vigilance::validation::{TestCategory, classify_tests};

/// Classify tests in a Rust source path
pub fn classify_tests_tool(
    params: ValidationClassifyTestsParams,
) -> Result<CallToolResult, McpError> {
    let path = Path::new(&params.path);

    if !path.exists() {
        let json = json!({
            "error": format!("Path does not exist: {}", params.path),
        });
        return Ok(CallToolResult::success(vec![Content::text(
            json.to_string(),
        )]));
    }

    match classify_tests(path) {
        Ok(classification) => {
            let tests_json: Vec<_> = classification
                .tests
                .iter()
                .map(|t| {
                    json!({
                        "name": t.name,
                        "file": t.file,
                        "category": t.category.name(),
                        "confidence": t.confidence,
                        "patterns": t.patterns,
                    })
                })
                .collect();

            let distribution: Vec<_> = TestCategory::all()
                .iter()
                .map(|cat| {
                    let count = classification.counts.get(*cat);
                    let pct = if classification.total_tests > 0 {
                        (count as f64 / classification.total_tests as f64) * 100.0
                    } else {
                        0.0
                    };
                    json!({
                        "category": cat.name(),
                        "description": cat.description(),
                        "count": count,
                        "percentage": format!("{:.1}%", pct),
                    })
                })
                .collect();

            let json = json!({
                "path": classification.path,
                "total_tests": classification.total_tests,
                "distribution": distribution,
                "coverage": {
                    "covered_categories": classification.coverage.covered_categories,
                    "total_categories": classification.coverage.total_categories,
                    "category_coverage": format!("{:.1}%", classification.coverage.category_coverage),
                    "missing_categories": classification.coverage.missing_categories.iter()
                        .map(|c| c.name()).collect::<Vec<_>>(),
                    "recommendations": classification.coverage.recommendations,
                },
                "tests": tests_json,
            });
            Ok(CallToolResult::success(vec![Content::text(
                json.to_string(),
            )]))
        }
        Err(e) => {
            let json = json!({
                "error": format!("{e}"),
            });
            Ok(CallToolResult::success(vec![Content::text(
                json.to_string(),
            )]))
        }
    }
}
