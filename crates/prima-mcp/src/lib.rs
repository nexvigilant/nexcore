// Copyright © 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! # Prima-to-MCP Compiler
//!
//! Compiles Prima functions into MCP tool definitions.
//!
//! ## Philosophy
//!
//! Write a Prima function, get an MCP tool. Every function is automatically:
//! - Documented with its primitive grounding
//! - Schema-validated via JSON Schema
//! - Ready for Claude Code integration
//!
//! ## Tier: T2-C (μ + σ + κ + →)
//!
//! ## Example
//!
//! ```prima
//! // confidence.true
//! μ confidence(tier: N) → N {
//!     Σ tier { 1 → 100, 2 → 90, 3 → 70, _ → 40 }
//! }
//! ```
//!
//! Compiles to:
//!
//! ```json
//! {
//!   "name": "prima_confidence",
//!   "inputSchema": {
//!     "type": "object",
//!     "properties": { "tier": { "type": "integer" } },
//!     "required": ["tier"]
//!   },
//!   "grounding": ["μ", "→", "N", "Σ"]
//! }
//! ```

#![forbid(unsafe_code)]
#![cfg_attr(
    not(test),
    deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)
)]

pub mod academy;
pub mod extract;
pub mod schema;

pub use academy::{academy_tools, execute_academy_tool};
pub use extract::{FunctionSig, Param, PrimaType, extract_functions};
pub use schema::{function_to_mcp_schema, generate_catalog, type_to_schema};

/// Compile a Prima source file to MCP tool definitions.
///
/// ## Tier: T2-C (σ + μ + →)
#[must_use]
pub fn compile(source: &str, prefix: &str) -> serde_json::Value {
    let functions = extract_functions(source);
    generate_catalog(&functions, prefix)
}

/// Compile multiple Prima files into a unified catalog.
///
/// ## Tier: T2-C (σ + μ + →)
#[must_use]
pub fn compile_files(sources: &[(String, String)], prefix: &str) -> serde_json::Value {
    let mut all_functions = Vec::new();

    for (_filename, source) in sources {
        all_functions.extend(extract_functions(source));
    }

    generate_catalog(&all_functions, prefix)
}

// ═══════════════════════════════════════════════════════════════════════════
// INTEGRATION TESTS
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compile_simple() {
        let source = r#"
// Calculate confidence from tier
μ confidence(tier: N) → N {
    Σ tier { 1 → 100, 2 → 90, _ → 40 }
}
"#;
        let catalog = compile(source, "prima");
        assert_eq!(catalog["count"], 1);

        let tools = catalog["tools"].as_array();
        assert!(tools.is_some());
        let empty_tools: Vec<serde_json::Value> = vec![];
        let tools = tools.unwrap_or(&empty_tools);
        assert_eq!(tools.len(), 1);

        let tool = &tools[0];
        assert_eq!(tool["name"], "prima_confidence");
    }

    #[test]
    fn test_compile_multiple_functions() {
        let source = r#"
μ add(a: N, b: N) → N { a + b }
μ mul(a: N, b: N) → N { a * b }
μ neg(x: N) → N { 0 - x }
"#;
        let catalog = compile(source, "math");
        assert_eq!(catalog["count"], 3);
    }

    #[test]
    fn test_grounding_in_output() {
        let source = "μ process(data: σ[N]) → N { data |> Ω(0, |a,b| a+b) }";
        let catalog = compile(source, "test");

        let empty_tools: Vec<serde_json::Value> = vec![];
        let tools = catalog["tools"].as_array().unwrap_or(&empty_tools);
        let tool = &tools[0];
        let grounding = tool["grounding"].as_array();

        assert!(grounding.is_some());
        let empty_grounding: Vec<serde_json::Value> = vec![];
        let grounding = grounding.unwrap_or(&empty_grounding);
        assert!(grounding.iter().any(|g| g == "σ"));
        assert!(grounding.iter().any(|g| g == "N"));
        assert!(grounding.iter().any(|g| g == "μ"));
    }

    #[test]
    fn test_compile_real_prima_example() {
        let source = r#"
// Skill validator using PRIMA methodology
// Classifies skills by tier and computes confidence
μ classify_tier(primitive_count: N) → N {
    ∂ primitive_count κ= 1 { 1 }
    else { ∂ primitive_count κ< 4 { 2 }
    else { ∂ primitive_count κ< 6 { 3 }
    else { 4 } } }
}

// Transfer confidence based on tier
μ transfer_confidence(tier_code: N) → N {
    Σ tier_code {
        1 → 100,
        2 → 90,
        3 → 70,
        _ → 40
    }
}
"#;
        let catalog = compile(source, "prima_skill");
        assert_eq!(catalog["count"], 2);

        let empty_tools2: Vec<serde_json::Value> = vec![];
        let tools = catalog["tools"].as_array().unwrap_or(&empty_tools2);
        let names: Vec<&str> = tools.iter().filter_map(|t| t["name"].as_str()).collect();

        assert!(names.contains(&"prima_skill_classify_tier"));
        assert!(names.contains(&"prima_skill_transfer_confidence"));
    }
}
