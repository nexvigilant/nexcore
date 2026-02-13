// Copyright © 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! # Function Extractor
//!
//! Extracts Prima function signatures for MCP tool generation.
//!
//! ## Tier: T2-C (σ + μ + κ)
//!
//! ## Example
//!
//! ```prima
//! μ confidence(tier: N) → N { ... }
//! ```
//!
//! Extracts to:
//! ```json
//! {
//!   "name": "confidence",
//!   "params": [{"name": "tier", "type": "N"}],
//!   "return_type": "N"
//! }
//! ```

use serde::{Deserialize, Serialize};

/// A Prima type for MCP schema generation.
///
/// ## Tier: T2-P (Σ + κ)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PrimaType {
    /// N - Quantity (integer)
    Int,
    /// Float
    Float,
    /// String
    String,
    /// Bool
    Bool,
    /// σ - Sequence
    Seq(Option<Box<PrimaType>>),
    /// ∅ - Void
    Void,
    /// Unknown type
    Unknown(std::string::String),
}

impl PrimaType {
    /// Parse a Prima type string.
    ///
    /// ## Examples
    /// - "N" → Int
    /// - "String" → String
    /// - "σ" → Seq(None)
    /// - "σ[N]" → Seq(Some(Int))
    #[must_use]
    pub fn parse(s: &str) -> Self {
        let s = s.trim();
        match s {
            "N" => Self::Int,
            "Float" => Self::Float,
            "String" => Self::String,
            "Bool" => Self::Bool,
            "∅" | "Void" => Self::Void,
            "σ" => Self::Seq(None),
            _ if s.starts_with("σ[") && s.ends_with(']') => {
                let inner = &s[3..s.len() - 1]; // Skip "σ[" and "]"
                let inner_type = Self::parse(inner);
                Self::Seq(Some(Box::new(inner_type)))
            }
            _ => Self::Unknown(s.to_string()),
        }
    }

    /// Get the Lex Primitiva grounding symbols.
    #[must_use]
    pub fn grounding(&self) -> Vec<&'static str> {
        match self {
            Self::Int => vec!["N"],
            Self::Float => vec!["N", "∝"],
            Self::String => vec!["σ", "N"],
            Self::Bool => vec!["Σ"],
            Self::Seq(_) => vec!["σ"],
            Self::Void => vec!["∅"],
            Self::Unknown(_) => vec!["?"],
        }
    }
}

/// A function parameter.
///
/// ## Tier: T2-P (λ + κ)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Param {
    /// Parameter name.
    pub name: std::string::String,
    /// Parameter type.
    pub ty: PrimaType,
}

/// An extracted Prima function signature.
///
/// ## Tier: T2-C (μ + σ + κ)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FunctionSig {
    /// Function name.
    pub name: std::string::String,
    /// Parameters.
    pub params: Vec<Param>,
    /// Return type.
    pub return_type: PrimaType,
    /// Documentation (from comments).
    pub doc: Option<std::string::String>,
}

impl FunctionSig {
    /// Get all primitive groundings used in this function.
    #[must_use]
    pub fn all_groundings(&self) -> Vec<&'static str> {
        let mut groundings = vec!["μ", "→"]; // All functions have mapping + causality

        for param in &self.params {
            groundings.extend(param.ty.grounding());
        }
        groundings.extend(self.return_type.grounding());

        // Deduplicate
        groundings.sort();
        groundings.dedup();
        groundings
    }
}

/// Extract function signatures from Prima source code.
///
/// ## Tier: T2-C (σ + μ + κ + ∂)
///
/// Looks for patterns like:
/// ```prima
/// μ name(param: Type, ...) → ReturnType { ... }
/// ```
#[must_use]
pub fn extract_functions(source: &str) -> Vec<FunctionSig> {
    let mut functions = Vec::new();
    let mut current_doc: Option<std::string::String> = None;

    for line in source.lines() {
        let trimmed = line.trim();

        // Collect doc comments
        if trimmed.starts_with("//") {
            let comment = trimmed.trim_start_matches('/').trim();
            if let Some(ref mut doc) = current_doc {
                doc.push('\n');
                doc.push_str(comment);
            } else {
                current_doc = Some(comment.to_string());
            }
            continue;
        }

        // Look for function definition: μ name(...) → Type
        if trimmed.starts_with("μ ") || trimmed.starts_with("fn ") {
            if let Some(sig) = parse_function_line(trimmed, current_doc.take()) {
                functions.push(sig);
            }
        } else if !trimmed.is_empty() {
            // Non-empty, non-comment line clears doc
            current_doc = None;
        }
    }

    functions
}

/// Parse a single function definition line.
fn parse_function_line(line: &str, doc: Option<std::string::String>) -> Option<FunctionSig> {
    // Skip μ or fn prefix
    let rest = line
        .trim_start_matches("μ ")
        .trim_start_matches("fn ")
        .trim();

    // Find function name (ends at '(')
    let paren_pos = rest.find('(')?;
    let name = rest[..paren_pos].trim().to_string();

    // Find params (between '(' and ')')
    let close_paren = rest.find(')')?;
    let params_str = &rest[paren_pos + 1..close_paren];
    let params = parse_params(params_str);

    // Find return type (after →)
    let return_type = if let Some(arrow_pos) = rest.find('→') {
        let after_arrow = &rest[arrow_pos + 3..]; // → is 3 bytes
        let type_end = after_arrow.find('{').unwrap_or(after_arrow.len());
        PrimaType::parse(after_arrow[..type_end].trim())
    } else {
        PrimaType::Void
    };

    Some(FunctionSig {
        name,
        params,
        return_type,
        doc,
    })
}

/// Parse parameter list.
fn parse_params(params_str: &str) -> Vec<Param> {
    if params_str.trim().is_empty() {
        return Vec::new();
    }

    params_str
        .split(',')
        .filter_map(|p| {
            let p = p.trim();
            let colon_pos = p.find(':')?;
            let name = p[..colon_pos].trim().to_string();
            let ty = PrimaType::parse(p[colon_pos + 1..].trim());
            Some(Param { name, ty })
        })
        .collect()
}

// ═══════════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    // ─────────────────────────────────────────────────────────────────────
    // Type parsing tests
    // ─────────────────────────────────────────────────────────────────────

    #[test]
    fn test_parse_simple_types() {
        assert_eq!(PrimaType::parse("N"), PrimaType::Int);
        assert_eq!(PrimaType::parse("String"), PrimaType::String);
        assert_eq!(PrimaType::parse("Bool"), PrimaType::Bool);
        assert_eq!(PrimaType::parse("∅"), PrimaType::Void);
    }

    #[test]
    fn test_parse_sequence_types() {
        assert_eq!(PrimaType::parse("σ"), PrimaType::Seq(None));
        assert_eq!(
            PrimaType::parse("σ[N]"),
            PrimaType::Seq(Some(Box::new(PrimaType::Int)))
        );
        assert_eq!(
            PrimaType::parse("σ[String]"),
            PrimaType::Seq(Some(Box::new(PrimaType::String)))
        );
    }

    #[test]
    fn test_type_grounding() {
        assert_eq!(PrimaType::Int.grounding(), vec!["N"]);
        assert_eq!(PrimaType::Seq(None).grounding(), vec!["σ"]);
        assert!(PrimaType::Bool.grounding().contains(&"Σ"));
    }

    // ─────────────────────────────────────────────────────────────────────
    // Function extraction tests
    // ─────────────────────────────────────────────────────────────────────

    #[test]
    fn test_extract_simple_function() {
        let source = r#"
μ add(a: N, b: N) → N {
    a + b
}
"#;
        let funcs = extract_functions(source);
        assert_eq!(funcs.len(), 1);

        let f = &funcs[0];
        assert_eq!(f.name, "add");
        assert_eq!(f.params.len(), 2);
        assert_eq!(f.params[0].name, "a");
        assert_eq!(f.params[0].ty, PrimaType::Int);
        assert_eq!(f.return_type, PrimaType::Int);
    }

    #[test]
    fn test_extract_with_doc() {
        let source = r#"
// Calculate transfer confidence
// Based on tier classification
μ confidence(tier: N) → N {
    Σ tier { 1 → 100, _ → 50 }
}
"#;
        let funcs = extract_functions(source);
        assert_eq!(funcs.len(), 1);

        let f = &funcs[0];
        assert!(f.doc.is_some());
        let empty_str = String::new();
        let doc = f.doc.as_ref().unwrap_or(&empty_str);
        assert!(doc.contains("transfer confidence"));
    }

    #[test]
    fn test_extract_sequence_param() {
        let source = "μ sum(nums: σ[N]) → N { nums |> Ω(0, |a,b| a+b) }";
        let funcs = extract_functions(source);
        assert_eq!(funcs.len(), 1);

        let f = &funcs[0];
        assert_eq!(
            f.params[0].ty,
            PrimaType::Seq(Some(Box::new(PrimaType::Int)))
        );
    }

    #[test]
    fn test_extract_void_return() {
        let source = "μ greet(name: String) → ∅ { println(name) }";
        let funcs = extract_functions(source);
        assert_eq!(funcs.len(), 1);
        assert_eq!(funcs[0].return_type, PrimaType::Void);
    }

    #[test]
    fn test_function_groundings() {
        let source = "μ process(data: σ[N]) → N { data |> Ω(0, |a,b| a+b) }";
        let funcs = extract_functions(source);
        let groundings = funcs[0].all_groundings();

        assert!(groundings.contains(&"μ"));
        assert!(groundings.contains(&"→"));
        assert!(groundings.contains(&"σ"));
        assert!(groundings.contains(&"N"));
    }

    #[test]
    fn test_extract_multiple_functions() {
        let source = r#"
μ foo(x: N) → N { x }
μ bar(s: String) → String { s }
"#;
        let funcs = extract_functions(source);
        assert_eq!(funcs.len(), 2);
        assert_eq!(funcs[0].name, "foo");
        assert_eq!(funcs[1].name, "bar");
    }

    #[test]
    fn test_extract_no_params() {
        let source = "μ answer() → N { 42 }";
        let funcs = extract_functions(source);
        assert_eq!(funcs.len(), 1);
        assert!(funcs[0].params.is_empty());
    }
}
