// Copyright © 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! # Codegen MCP Tools
//!
//! Provides code generation tools for the MCP server.
//!
//! ## Tier: T2-C (μ + σ + → + π)
//!
//! ## Tools
//! - `prima_codegen` → Generate target language code from Prima source
//! - `prima_targets` → List available code generation targets
//! - `prima_primitives` → List primitives used in source

use prima_codegen::{
    Backend, EmitContext, TargetLanguage,
    backends::{CBackend, GoBackend, PythonBackend, RustBackend, TypeScriptBackend},
};
use serde::Serialize;
use serde_json::{Value as JsonValue, json};

/// Codegen tool result.
#[derive(Debug, Serialize)]
pub struct CodegenResult {
    /// Generated code.
    pub code: String,
    /// Target language.
    pub target: String,
    /// Transfer confidence score.
    pub transfer_confidence: f64,
    /// Primitives used.
    pub primitives: Vec<String>,
}

/// Codegen error.
#[derive(Debug)]
pub enum CodegenError {
    ParseError(String),
    EmitError(String),
    UnknownTarget(String),
}

impl std::fmt::Display for CodegenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CodegenError::ParseError(e) => write!(f, "Parse error: {}", e),
            CodegenError::EmitError(e) => write!(f, "Emit error: {}", e),
            CodegenError::UnknownTarget(t) => write!(f, "Unknown target: {}", t),
        }
    }
}

/// Generate code from Prima source.
///
/// ## Parameters
/// - `source`: Prima source code
/// - `target`: Target language (rust, python, typescript, go, c)
///
/// ## Returns
/// Generated code with metadata.
pub fn generate(source: &str, target: &str) -> Result<CodegenResult, CodegenError> {
    // Parse Prima source
    let program = prima::parse(source).map_err(|e| CodegenError::ParseError(format!("{}", e)))?;

    // Get target language
    let lang = match target.to_lowercase().as_str() {
        "rust" | "rs" => TargetLanguage::Rust,
        "python" | "py" => TargetLanguage::Python,
        "typescript" | "ts" => TargetLanguage::TypeScript,
        "go" | "golang" => TargetLanguage::Go,
        "c" => TargetLanguage::C,
        other => return Err(CodegenError::UnknownTarget(other.to_string())),
    };

    // Generate code
    let mut ctx = EmitContext::new();
    let code = match lang {
        TargetLanguage::Rust => RustBackend::new().emit_program(&program, &mut ctx),
        TargetLanguage::Python => PythonBackend::new().emit_program(&program, &mut ctx),
        TargetLanguage::TypeScript => TypeScriptBackend::new().emit_program(&program, &mut ctx),
        TargetLanguage::Go => GoBackend::new().emit_program(&program, &mut ctx),
        TargetLanguage::C => CBackend::new().emit_program(&program, &mut ctx),
    }
    .map_err(|e| CodegenError::EmitError(format!("{}", e)))?;

    // Collect primitives
    let primitives: Vec<String> = ctx
        .primitives_used
        .iter()
        .map(|p| p.symbol().to_string())
        .collect();

    Ok(CodegenResult {
        code,
        target: format!("{:?}", lang).to_lowercase(),
        transfer_confidence: ctx.transfer_confidence(lang),
        primitives,
    })
}

/// List available code generation targets.
///
/// ## Returns
/// Array of target info objects.
pub fn list_targets() -> JsonValue {
    json!([
        {
            "name": "rust",
            "extension": "rs",
            "transfer_confidence": 0.95,
            "description": "Rust - Systems programming with ownership semantics"
        },
        {
            "name": "python",
            "extension": "py",
            "transfer_confidence": 0.75,
            "description": "Python - Dynamic scripting with duck typing"
        },
        {
            "name": "typescript",
            "extension": "ts",
            "transfer_confidence": 0.85,
            "description": "TypeScript - JavaScript with static types"
        },
        {
            "name": "go",
            "extension": "go",
            "transfer_confidence": 0.80,
            "description": "Go - Concurrent systems with goroutines"
        },
        {
            "name": "c",
            "extension": "c",
            "transfer_confidence": 0.70,
            "description": "C - Low-level systems programming"
        }
    ])
}

/// List primitives used in Prima source.
///
/// ## Parameters
/// - `source`: Prima source code
///
/// ## Returns
/// Array of primitive symbols with descriptions.
pub fn list_primitives(source: &str) -> Result<JsonValue, CodegenError> {
    // Parse to extract primitives
    let program = prima::parse(source).map_err(|e| CodegenError::ParseError(format!("{}", e)))?;

    // Generate with any backend to populate ctx.primitives_used
    let mut ctx = EmitContext::new();
    let _ = RustBackend::new().emit_program(&program, &mut ctx);

    // Build primitive info
    let primitives: Vec<JsonValue> = ctx
        .primitives_used
        .iter()
        .map(|p| {
            json!({
                "symbol": p.symbol(),
                "name": format!("{:?}", p).to_lowercase(),
                "tier": "T1"
            })
        })
        .collect();

    Ok(json!({
        "count": primitives.len(),
        "primitives": primitives
    }))
}

/// Get MCP tool definitions for codegen tools.
pub fn codegen_tools() -> Vec<JsonValue> {
    vec![
        json!({
            "name": "prima_codegen",
            "description": "Generate target language code from Prima source. Returns generated code with transfer confidence.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "source": {
                        "type": "string",
                        "description": "Prima source code to compile"
                    },
                    "target": {
                        "type": "string",
                        "enum": ["rust", "python", "typescript", "go", "c"],
                        "description": "Target language for code generation"
                    }
                },
                "required": ["source", "target"]
            }
        }),
        json!({
            "name": "prima_targets",
            "description": "List available code generation targets with transfer confidence scores.",
            "inputSchema": {
                "type": "object",
                "properties": {},
                "required": []
            }
        }),
        json!({
            "name": "prima_primitives",
            "description": "List T1 primitives used in Prima source code.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "source": {
                        "type": "string",
                        "description": "Prima source code to analyze"
                    }
                },
                "required": ["source"]
            }
        }),
    ]
}

// ═══════════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_rust() {
        let source = "μ add(a: N, b: N) → N { a + b }";
        let result = generate(source, "rust");
        assert!(result.is_ok());
        let r = result.ok().unwrap_or_else(|| CodegenResult {
            code: String::new(),
            target: String::new(),
            transfer_confidence: 0.0,
            primitives: vec![],
        });
        assert!(r.code.contains("fn add"));
        assert!(r.transfer_confidence > 0.9);
    }

    #[test]
    fn test_generate_python() {
        let source = "μ add(a: N, b: N) → N { a + b }";
        let result = generate(source, "python");
        assert!(result.is_ok());
        let r = result.ok().unwrap_or_else(|| CodegenResult {
            code: String::new(),
            target: String::new(),
            transfer_confidence: 0.0,
            primitives: vec![],
        });
        assert!(r.code.contains("def add"));
    }

    #[test]
    fn test_generate_typescript() {
        let source = "μ add(a: N, b: N) → N { a + b }";
        let result = generate(source, "typescript");
        assert!(result.is_ok());
        let r = result.ok().unwrap_or_else(|| CodegenResult {
            code: String::new(),
            target: String::new(),
            transfer_confidence: 0.0,
            primitives: vec![],
        });
        assert!(r.code.contains("function add"));
    }

    #[test]
    fn test_generate_go() {
        let source = "μ add(a: N, b: N) → N { a + b }";
        let result = generate(source, "go");
        assert!(result.is_ok());
        let r = result.ok().unwrap_or_else(|| CodegenResult {
            code: String::new(),
            target: String::new(),
            transfer_confidence: 0.0,
            primitives: vec![],
        });
        assert!(r.code.contains("func add"));
    }

    #[test]
    fn test_generate_c() {
        let source = "μ add(a: N, b: N) → N { a + b }";
        let result = generate(source, "c");
        assert!(result.is_ok());
        let r = result.ok().unwrap_or_else(|| CodegenResult {
            code: String::new(),
            target: String::new(),
            transfer_confidence: 0.0,
            primitives: vec![],
        });
        assert!(r.code.contains("int64_t add"));
    }

    #[test]
    fn test_generate_unknown_target() {
        let source = "μ add(a: N, b: N) → N { a + b }";
        let result = generate(source, "cobol");
        assert!(result.is_err());
    }

    #[test]
    fn test_list_targets() {
        let targets = list_targets();
        assert!(targets.is_array());
        let arr = targets.as_array();
        assert!(arr.is_some());
        assert_eq!(arr.map(|a| a.len()).unwrap_or(0), 5);
    }

    #[test]
    fn test_list_primitives() {
        let source = "μ add(a: N, b: N) → N { a + b }";
        let result = list_primitives(source);
        assert!(result.is_ok());
    }

    #[test]
    fn test_codegen_tools_schema() {
        let tools = codegen_tools();
        assert_eq!(tools.len(), 3);

        // Verify prima_codegen tool
        let codegen_tool = &tools[0];
        assert_eq!(codegen_tool["name"], "prima_codegen");
        assert!(codegen_tool["inputSchema"]["properties"]["source"].is_object());
        assert!(codegen_tool["inputSchema"]["properties"]["target"].is_object());
    }
}
