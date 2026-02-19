// Copyright © 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! T1 primitive to target language mappings.
//!
//! ## Tier: T2-C (μ + σ + κ)
//!
//! Each T1 primitive maps to constructs in target languages.

use lex_primitiva::LexPrimitiva;
use serde::{Deserialize, Serialize};

/// Target language construct representing a T1 primitive.
///
/// ## Tier: T2-P (ς + S)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TargetConstruct {
    /// The construct syntax (e.g., "Vec", "fn", "struct")
    pub syntax: String,
    /// Category of construct
    pub category: ConstructCategory,
    /// Notes about the mapping
    pub notes: Option<String>,
}

impl TargetConstruct {
    /// Create a new target construct
    #[must_use]
    pub fn new(syntax: impl Into<String>, category: ConstructCategory) -> Self {
        Self {
            syntax: syntax.into(),
            category,
            notes: None,
        }
    }

    /// Add notes to the construct
    #[must_use]
    pub fn with_notes(mut self, notes: impl Into<String>) -> Self {
        self.notes = Some(notes.into());
        self
    }
}

/// Category of target construct.
///
/// ## Tier: T2-P (Σ)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConstructCategory {
    /// Type construct (Vec, list, Array)
    Type,
    /// Keyword (fn, def, function)
    Keyword,
    /// Operator (==, ===, ->)
    Operator,
    /// Literal form
    Literal,
    /// Control flow
    Control,
}

/// Complete primitive mapping table for a target language.
///
/// ## Tier: T2-C (σ + μ)
#[derive(Debug, Clone)]
pub struct PrimitiveMapping {
    mappings: Vec<(LexPrimitiva, TargetConstruct)>,
}

impl PrimitiveMapping {
    /// Create a new empty mapping
    #[must_use]
    pub fn new() -> Self {
        Self {
            mappings: Vec::new(),
        }
    }

    /// Add a mapping
    pub fn add(&mut self, prim: LexPrimitiva, construct: TargetConstruct) {
        self.mappings.push((prim, construct));
    }

    /// Get construct for a primitive
    #[must_use]
    pub fn get(&self, prim: LexPrimitiva) -> Option<&TargetConstruct> {
        self.mappings
            .iter()
            .find(|(p, _)| *p == prim)
            .map(|(_, c)| c)
    }

    /// Iterate over all mappings
    pub fn iter(&self) -> impl Iterator<Item = &(LexPrimitiva, TargetConstruct)> {
        self.mappings.iter()
    }

    /// Create Rust primitive mappings
    #[must_use]
    pub fn rust() -> Self {
        let mut m = Self::new();
        m.add(
            LexPrimitiva::Sequence,
            TargetConstruct::new("Vec", ConstructCategory::Type),
        );
        m.add(
            LexPrimitiva::Mapping,
            TargetConstruct::new("fn", ConstructCategory::Keyword),
        );
        m.add(
            LexPrimitiva::State,
            TargetConstruct::new("struct", ConstructCategory::Keyword),
        );
        m.add(
            LexPrimitiva::Recursion,
            TargetConstruct::new("Box<Self>", ConstructCategory::Type),
        );
        m.add(
            LexPrimitiva::Void,
            TargetConstruct::new("()", ConstructCategory::Type),
        );
        m.add(
            LexPrimitiva::Boundary,
            TargetConstruct::new("Result", ConstructCategory::Type),
        );
        m.add(
            LexPrimitiva::Quantity,
            TargetConstruct::new("i64", ConstructCategory::Type),
        );
        m.add(
            LexPrimitiva::Causality,
            TargetConstruct::new("->", ConstructCategory::Operator),
        );
        m.add(
            LexPrimitiva::Comparison,
            TargetConstruct::new("==", ConstructCategory::Operator),
        );
        m.add(
            LexPrimitiva::Sum,
            TargetConstruct::new("enum", ConstructCategory::Keyword),
        );
        m.add(
            LexPrimitiva::Location,
            TargetConstruct::new("&", ConstructCategory::Operator),
        );
        m.add(
            LexPrimitiva::Existence,
            TargetConstruct::new("Option", ConstructCategory::Type),
        );
        m.add(
            LexPrimitiva::Persistence,
            TargetConstruct::new("'static", ConstructCategory::Keyword),
        );
        m.add(
            LexPrimitiva::Irreversibility,
            TargetConstruct::new("Drop", ConstructCategory::Keyword),
        );
        m.add(
            LexPrimitiva::Frequency,
            TargetConstruct::new("Iterator", ConstructCategory::Type),
        );
        m.add(
            LexPrimitiva::Product,
            TargetConstruct::new("(A, B)", ConstructCategory::Type),
        );
        m
    }

    /// Create Python primitive mappings
    #[must_use]
    pub fn python() -> Self {
        let mut m = Self::new();
        m.add(
            LexPrimitiva::Sequence,
            TargetConstruct::new("list", ConstructCategory::Type),
        );
        m.add(
            LexPrimitiva::Mapping,
            TargetConstruct::new("def", ConstructCategory::Keyword),
        );
        m.add(
            LexPrimitiva::State,
            TargetConstruct::new("class", ConstructCategory::Keyword),
        );
        m.add(
            LexPrimitiva::Recursion,
            TargetConstruct::new("self", ConstructCategory::Keyword),
        );
        m.add(
            LexPrimitiva::Void,
            TargetConstruct::new("None", ConstructCategory::Literal),
        );
        m.add(
            LexPrimitiva::Boundary,
            TargetConstruct::new("try/except", ConstructCategory::Control),
        );
        m.add(
            LexPrimitiva::Quantity,
            TargetConstruct::new("int", ConstructCategory::Type),
        );
        m.add(
            LexPrimitiva::Causality,
            TargetConstruct::new("->", ConstructCategory::Operator),
        );
        m.add(
            LexPrimitiva::Comparison,
            TargetConstruct::new("==", ConstructCategory::Operator),
        );
        m.add(
            LexPrimitiva::Sum,
            TargetConstruct::new("Union", ConstructCategory::Type),
        );
        m.add(
            LexPrimitiva::Location,
            TargetConstruct::new("id()", ConstructCategory::Keyword),
        );
        m.add(
            LexPrimitiva::Existence,
            TargetConstruct::new("Optional", ConstructCategory::Type),
        );
        m.add(
            LexPrimitiva::Persistence,
            TargetConstruct::new("global", ConstructCategory::Keyword),
        );
        m.add(
            LexPrimitiva::Irreversibility,
            TargetConstruct::new("del", ConstructCategory::Keyword),
        );
        m.add(
            LexPrimitiva::Frequency,
            TargetConstruct::new("iter()", ConstructCategory::Keyword),
        );
        m.add(
            LexPrimitiva::Product,
            TargetConstruct::new("tuple", ConstructCategory::Type),
        );
        m
    }

    /// Create TypeScript primitive mappings
    #[must_use]
    pub fn typescript() -> Self {
        let mut m = Self::new();
        m.add(
            LexPrimitiva::Sequence,
            TargetConstruct::new("Array", ConstructCategory::Type),
        );
        m.add(
            LexPrimitiva::Mapping,
            TargetConstruct::new("function", ConstructCategory::Keyword),
        );
        m.add(
            LexPrimitiva::State,
            TargetConstruct::new("interface", ConstructCategory::Keyword),
        );
        m.add(
            LexPrimitiva::Recursion,
            TargetConstruct::new("this", ConstructCategory::Keyword),
        );
        m.add(
            LexPrimitiva::Void,
            TargetConstruct::new("void", ConstructCategory::Type),
        );
        m.add(
            LexPrimitiva::Boundary,
            TargetConstruct::new("Result<T,E>", ConstructCategory::Type),
        );
        m.add(
            LexPrimitiva::Quantity,
            TargetConstruct::new("number", ConstructCategory::Type),
        );
        m.add(
            LexPrimitiva::Causality,
            TargetConstruct::new("=>", ConstructCategory::Operator),
        );
        m.add(
            LexPrimitiva::Comparison,
            TargetConstruct::new("===", ConstructCategory::Operator),
        );
        m.add(
            LexPrimitiva::Sum,
            TargetConstruct::new("|", ConstructCategory::Operator),
        );
        m.add(
            LexPrimitiva::Location,
            TargetConstruct::new("&", ConstructCategory::Operator),
        );
        m.add(
            LexPrimitiva::Existence,
            TargetConstruct::new("T | undefined", ConstructCategory::Type),
        );
        m.add(
            LexPrimitiva::Persistence,
            TargetConstruct::new("const", ConstructCategory::Keyword),
        );
        m.add(
            LexPrimitiva::Irreversibility,
            TargetConstruct::new("readonly", ConstructCategory::Keyword),
        );
        m.add(
            LexPrimitiva::Frequency,
            TargetConstruct::new("Iterator", ConstructCategory::Type),
        );
        m.add(
            LexPrimitiva::Product,
            TargetConstruct::new("[A, B]", ConstructCategory::Type),
        );
        m
    }

    /// Create Go primitive mappings
    #[must_use]
    pub fn go() -> Self {
        let mut m = Self::new();
        m.add(
            LexPrimitiva::Sequence,
            TargetConstruct::new("[]", ConstructCategory::Type),
        );
        m.add(
            LexPrimitiva::Mapping,
            TargetConstruct::new("func", ConstructCategory::Keyword),
        );
        m.add(
            LexPrimitiva::State,
            TargetConstruct::new("struct", ConstructCategory::Keyword),
        );
        m.add(
            LexPrimitiva::Recursion,
            TargetConstruct::new("*Self", ConstructCategory::Type),
        );
        m.add(
            LexPrimitiva::Void,
            TargetConstruct::new("struct{}", ConstructCategory::Type),
        );
        m.add(
            LexPrimitiva::Boundary,
            TargetConstruct::new("error", ConstructCategory::Type),
        );
        m.add(
            LexPrimitiva::Quantity,
            TargetConstruct::new("int64", ConstructCategory::Type),
        );
        m.add(
            LexPrimitiva::Causality,
            TargetConstruct::new("return", ConstructCategory::Keyword),
        );
        m.add(
            LexPrimitiva::Comparison,
            TargetConstruct::new("==", ConstructCategory::Operator),
        );
        m.add(
            LexPrimitiva::Sum,
            TargetConstruct::new("interface{}", ConstructCategory::Type),
        );
        m.add(
            LexPrimitiva::Location,
            TargetConstruct::new("&", ConstructCategory::Operator),
        );
        m.add(
            LexPrimitiva::Existence,
            TargetConstruct::new("*T", ConstructCategory::Type),
        );
        m.add(
            LexPrimitiva::Persistence,
            TargetConstruct::new("const", ConstructCategory::Keyword),
        );
        m.add(
            LexPrimitiva::Irreversibility,
            TargetConstruct::new("Close()", ConstructCategory::Keyword),
        );
        m.add(
            LexPrimitiva::Frequency,
            TargetConstruct::new("range", ConstructCategory::Keyword),
        );
        m.add(
            LexPrimitiva::Product,
            TargetConstruct::new("struct{A,B}", ConstructCategory::Type),
        );
        m
    }

    /// Create C primitive mappings
    #[must_use]
    pub fn c() -> Self {
        let mut m = Self::new();
        m.add(
            LexPrimitiva::Sequence,
            TargetConstruct::new("*", ConstructCategory::Type),
        );
        m.add(
            LexPrimitiva::Mapping,
            TargetConstruct::new("function", ConstructCategory::Keyword),
        );
        m.add(
            LexPrimitiva::State,
            TargetConstruct::new("struct", ConstructCategory::Keyword),
        );
        m.add(
            LexPrimitiva::Recursion,
            TargetConstruct::new("*self", ConstructCategory::Type),
        );
        m.add(
            LexPrimitiva::Void,
            TargetConstruct::new("void", ConstructCategory::Type),
        );
        m.add(
            LexPrimitiva::Boundary,
            TargetConstruct::new("int", ConstructCategory::Type)
                .with_notes("C uses return codes for errors"),
        );
        m.add(
            LexPrimitiva::Quantity,
            TargetConstruct::new("int64_t", ConstructCategory::Type),
        );
        m.add(
            LexPrimitiva::Causality,
            TargetConstruct::new("return", ConstructCategory::Keyword),
        );
        m.add(
            LexPrimitiva::Comparison,
            TargetConstruct::new("==", ConstructCategory::Operator),
        );
        m.add(
            LexPrimitiva::Sum,
            TargetConstruct::new("union", ConstructCategory::Keyword)
                .with_notes("Tagged union with enum discriminant"),
        );
        m.add(
            LexPrimitiva::Location,
            TargetConstruct::new("&", ConstructCategory::Operator),
        );
        m.add(
            LexPrimitiva::Existence,
            TargetConstruct::new("*", ConstructCategory::Type).with_notes("NULL for absence"),
        );
        m.add(
            LexPrimitiva::Persistence,
            TargetConstruct::new("static", ConstructCategory::Keyword),
        );
        m.add(
            LexPrimitiva::Irreversibility,
            TargetConstruct::new("free()", ConstructCategory::Keyword),
        );
        m.add(
            LexPrimitiva::Frequency,
            TargetConstruct::new("for", ConstructCategory::Keyword),
        );
        m.add(
            LexPrimitiva::Product,
            TargetConstruct::new("struct", ConstructCategory::Keyword)
                .with_notes("C structs are product types"),
        );
        m
    }
}

impl Default for PrimitiveMapping {
    fn default() -> Self {
        Self::new()
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rust_mapping_complete() {
        let m = PrimitiveMapping::rust();
        // Should have mapping for all 16 primitives
        for prim in LexPrimitiva::all() {
            assert!(m.get(prim).is_some(), "Missing Rust mapping for {:?}", prim);
        }
    }

    #[test]
    fn test_python_mapping_complete() {
        let m = PrimitiveMapping::python();
        for prim in LexPrimitiva::all() {
            assert!(
                m.get(prim).is_some(),
                "Missing Python mapping for {:?}",
                prim
            );
        }
    }

    #[test]
    fn test_typescript_mapping_complete() {
        let m = PrimitiveMapping::typescript();
        for prim in LexPrimitiva::all() {
            assert!(
                m.get(prim).is_some(),
                "Missing TypeScript mapping for {:?}",
                prim
            );
        }
    }

    #[test]
    fn test_go_mapping_complete() {
        let m = PrimitiveMapping::go();
        for prim in LexPrimitiva::all() {
            assert!(m.get(prim).is_some(), "Missing Go mapping for {:?}", prim);
        }
    }

    #[test]
    fn test_c_mapping_complete() {
        let m = PrimitiveMapping::c();
        for prim in LexPrimitiva::all() {
            assert!(m.get(prim).is_some(), "Missing C mapping for {:?}", prim);
        }
    }

    #[test]
    fn test_rust_sequence_is_vec() {
        let m = PrimitiveMapping::rust();
        let seq = m.get(LexPrimitiva::Sequence);
        assert!(seq.is_some());
        if let Some(construct) = seq {
            assert_eq!(construct.syntax, "Vec");
        }
    }

    #[test]
    fn test_python_sequence_is_list() {
        let m = PrimitiveMapping::python();
        let seq = m.get(LexPrimitiva::Sequence);
        assert!(seq.is_some());
        if let Some(construct) = seq {
            assert_eq!(construct.syntax, "list");
        }
    }

    #[test]
    fn test_construct_with_notes() {
        let c = TargetConstruct::new("Vec", ConstructCategory::Type).with_notes("Growable array");
        assert_eq!(c.notes, Some("Growable array".to_string()));
    }
}
