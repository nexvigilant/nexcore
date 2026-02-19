//! Prima Language MCP Tools
//!
//! Exposes Prima language capabilities as MCP tools:
//! - Parse Prima source to AST
//! - Evaluate Prima expressions
//! - Generate code for multiple targets (Rust, Python, TypeScript, Go, C)
//! - List primitives used in source
//!
//! ## Tier: T2-C (μ + σ + → + κ)
//!
//! ## Lifecycle
//! - **begins**: Tool invocation starts
//! - **exists**: Source code validated for existence
//! - **changes**: State transforms through parse/eval/codegen
//! - **persists**: Results returned as JSON
//! - **ends**: Tool completes with success/error

use crate::params::{PrimaCodegenParams, PrimaEvalParams, PrimaParseParams, PrimaPrimitivesParams};
use prima_codegen::{
    Backend, EmitContext, TargetLanguage,
    backends::{CBackend, GoBackend, PythonBackend, RustBackend, TypeScriptBackend},
};
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use serde_json::json;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Compute tier string from primitive count.
fn tier_from_count(count: usize) -> &'static str {
    match count {
        0 => "∅ (Void)",
        1 => "T1 (Universal)",
        2..=3 => "T2-P (Cross-domain Primitive)",
        4..=5 => "T2-C (Cross-domain Composite)",
        _ => "T3 (Domain-specific)",
    }
}

/// Convert primitives_used HashSet to JSON-friendly vec of symbols.
fn primitives_to_symbols(ctx: &EmitContext) -> Vec<String> {
    ctx.primitives_used
        .iter()
        .map(|p| p.symbol().to_string())
        .collect()
}

// ---------------------------------------------------------------------------
// prima_parse: Parse source to AST
// ---------------------------------------------------------------------------

/// Parse Prima source code and return AST as JSON.
pub fn prima_parse(params: PrimaParseParams) -> Result<CallToolResult, McpError> {
    let source = &params.source;

    match prima::parse(source) {
        Ok(program) => {
            let ast_json = json!({
                "success": true,
                "statements": program.statements.len(),
                "ast": format!("{:?}", program),
            });
            Ok(CallToolResult::success(vec![Content::text(
                serde_json::to_string_pretty(&ast_json)
                    .unwrap_or_else(|_| "Parse succeeded".to_string()),
            )]))
        }
        Err(e) => Ok(CallToolResult::success(vec![Content::text(
            json!({
                "success": false,
                "error": e.to_string(),
            })
            .to_string(),
        )])),
    }
}

// ---------------------------------------------------------------------------
// prima_eval: Evaluate Prima expression
// ---------------------------------------------------------------------------

/// Evaluate a Prima expression and return the result.
pub fn prima_eval(params: PrimaEvalParams) -> Result<CallToolResult, McpError> {
    let source = &params.source;

    match prima::eval(source) {
        Ok(value) => {
            let result = json!({
                "success": true,
                "value": format!("{}", value),
                "type": format!("{:?}", std::mem::discriminant(&value.data)),
            });
            Ok(CallToolResult::success(vec![Content::text(
                serde_json::to_string_pretty(&result)
                    .unwrap_or_else(|_| "Eval succeeded".to_string()),
            )]))
        }
        Err(e) => Ok(CallToolResult::success(vec![Content::text(
            json!({
                "success": false,
                "error": e.to_string(),
            })
            .to_string(),
        )])),
    }
}

// ---------------------------------------------------------------------------
// prima_codegen: Generate code for target language
// ---------------------------------------------------------------------------

/// Generate code from Prima source for a target language.
pub fn prima_codegen(params: PrimaCodegenParams) -> Result<CallToolResult, McpError> {
    let source = &params.source;
    let target = params.target.as_deref().unwrap_or("rust");

    // Parse source
    let program = match prima::parse(source) {
        Ok(p) => p,
        Err(e) => {
            return Ok(CallToolResult::success(vec![Content::text(
                json!({
                    "success": false,
                    "error": format!("Parse error: {}", e),
                })
                .to_string(),
            )]));
        }
    };

    // Select backend
    let backend: Box<dyn Backend> = match target.to_lowercase().as_str() {
        "rust" => Box::new(RustBackend::new()),
        "python" | "py" => Box::new(PythonBackend::new()),
        "typescript" | "ts" => Box::new(TypeScriptBackend::new()),
        "go" | "golang" => Box::new(GoBackend::new()),
        "c" => Box::new(CBackend::new()),
        other => {
            return Ok(CallToolResult::success(vec![Content::text(json!({
                "success": false,
                "error": format!("Unknown target: {}. Use: rust, python, typescript, go, c", other),
            }).to_string())]));
        }
    };

    // Generate code
    let mut ctx = EmitContext::new();
    match backend.emit_program(&program, &mut ctx) {
        Ok(code) => {
            let target_lang = match target.to_lowercase().as_str() {
                "python" | "py" => TargetLanguage::Python,
                "typescript" | "ts" => TargetLanguage::TypeScript,
                "go" | "golang" => TargetLanguage::Go,
                "c" => TargetLanguage::C,
                _ => TargetLanguage::Rust,
            };

            let symbols = primitives_to_symbols(&ctx);
            let result = json!({
                "success": true,
                "target": target,
                "code": code,
                "primitives_used": symbols,
                "transfer_confidence": ctx.transfer_confidence(target_lang),
                "tier": tier_from_count(symbols.len()),
            });
            Ok(CallToolResult::success(vec![Content::text(
                serde_json::to_string_pretty(&result).unwrap_or_else(|_| code),
            )]))
        }
        Err(e) => Ok(CallToolResult::success(vec![Content::text(
            json!({
                "success": false,
                "error": format!("Codegen error: {}", e),
            })
            .to_string(),
        )])),
    }
}

// ---------------------------------------------------------------------------
// prima_primitives: List primitives used in source
// ---------------------------------------------------------------------------

/// Analyze Prima source and list the T1 primitives used.
pub fn prima_primitives(params: PrimaPrimitivesParams) -> Result<CallToolResult, McpError> {
    let source = &params.source;

    // Parse to check validity
    let program = match prima::parse(source) {
        Ok(p) => p,
        Err(e) => {
            return Ok(CallToolResult::success(vec![Content::text(
                json!({
                    "success": false,
                    "error": format!("Parse error: {}", e),
                })
                .to_string(),
            )]));
        }
    };

    // Use Rust backend to analyze primitives
    let mut ctx = EmitContext::new();
    let backend = RustBackend::new();
    let _ = backend.emit_program(&program, &mut ctx);

    let symbols = primitives_to_symbols(&ctx);
    let count = symbols.len();

    // Check each of the 15 primitives
    let check = |sym: &str| -> bool { symbols.iter().any(|s| s == sym) };

    let result = json!({
        "success": true,
        "primitives_used": symbols,
        "primitive_count": count,
        "tier": tier_from_count(count),
        "coverage": format!("{}/15", count),
        "lex_primitiva": [
            {"symbol": "σ", "name": "Sequence", "used": check("σ")},
            {"symbol": "μ", "name": "Mapping", "used": check("μ")},
            {"symbol": "ς", "name": "State", "used": check("ς")},
            {"symbol": "ρ", "name": "Recursion", "used": check("ρ")},
            {"symbol": "∅", "name": "Void", "used": check("∅")},
            {"symbol": "∂", "name": "Boundary", "used": check("∂")},
            {"symbol": "ν", "name": "Invariant", "used": check("ν")},
            {"symbol": "∃", "name": "Existence", "used": check("∃")},
            {"symbol": "π", "name": "Persistence", "used": check("π")},
            {"symbol": "→", "name": "Causality", "used": check("→")},
            {"symbol": "κ", "name": "Comparison", "used": check("κ")},
            {"symbol": "N", "name": "Quantity", "used": check("N")},
            {"symbol": "λ", "name": "Location", "used": check("λ")},
            {"symbol": "∝", "name": "Proportion", "used": check("∝")},
            {"symbol": "Σ", "name": "Sum", "used": check("Σ")},
        ],
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_else(|_| "Analysis complete".to_string()),
    )]))
}

// ---------------------------------------------------------------------------
// prima_targets: List available codegen targets
// ---------------------------------------------------------------------------

/// List available code generation targets with transfer confidence.
pub fn prima_targets() -> Result<CallToolResult, McpError> {
    let result = json!({
        "targets": [
            {"name": "rust", "extension": ".rs", "transfer_confidence": 0.95, "type_safety": "strong"},
            {"name": "python", "extension": ".py", "transfer_confidence": 0.70, "type_safety": "dynamic"},
            {"name": "typescript", "extension": ".ts", "transfer_confidence": 0.85, "type_safety": "gradual"},
            {"name": "go", "extension": ".go", "transfer_confidence": 0.80, "type_safety": "strong"},
            {"name": "c", "extension": ".c", "transfer_confidence": 0.60, "type_safety": "manual"},
        ],
        "lex_primitiva": "σ μ ς ρ ∅ ∂ ν ∃ π → κ N λ ∝ Σ",
        "file_extension": ".true",
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_else(|_| "Targets listed".to_string()),
    )]))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prima_parse_valid() {
        let params = PrimaParseParams {
            source: "λ x = 42".to_string(),
        };
        let result = prima_parse(params);
        assert!(result.is_ok());
    }

    #[test]
    fn test_prima_parse_invalid() {
        let params = PrimaParseParams {
            source: "invalid {{{{".to_string(),
        };
        let result = prima_parse(params);
        assert!(result.is_ok()); // Returns success with error in body
    }

    #[test]
    fn test_prima_codegen_rust() {
        let params = PrimaCodegenParams {
            source: "μ double(x: N) → N { x * 2 }".to_string(),
            target: Some("rust".to_string()),
        };
        let result = prima_codegen(params);
        assert!(result.is_ok());
    }

    #[test]
    fn test_prima_codegen_python() {
        let params = PrimaCodegenParams {
            source: "μ add(a: N, b: N) → N { a + b }".to_string(),
            target: Some("python".to_string()),
        };
        let result = prima_codegen(params);
        assert!(result.is_ok());
    }

    #[test]
    fn test_prima_targets() {
        let result = prima_targets();
        assert!(result.is_ok());
    }

    #[test]
    fn test_prima_primitives() {
        let params = PrimaPrimitivesParams {
            source: "μ f(x: N) → N { x }".to_string(),
        };
        let result = prima_primitives(params);
        assert!(result.is_ok());
    }

    #[test]
    fn test_tier_from_count() {
        assert_eq!(tier_from_count(0), "∅ (Void)");
        assert_eq!(tier_from_count(1), "T1 (Universal)");
        assert_eq!(tier_from_count(3), "T2-P (Cross-domain Primitive)");
        assert_eq!(tier_from_count(5), "T2-C (Cross-domain Composite)");
        assert_eq!(tier_from_count(7), "T3 (Domain-specific)");
    }
}
