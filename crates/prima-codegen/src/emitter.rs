// Copyright © 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! Code emission infrastructure.
//!
//! ## Tier: T2-C (μ + ς + σ + →)

use crate::{Backend, CodegenError, TargetLanguage};
use lex_primitiva::LexPrimitiva;
use prima::prelude::Program;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Emission context tracking state during code generation.
///
/// ## Tier: T2-C (ς + σ)
#[derive(Debug, Clone)]
pub struct EmitContext {
    /// Current indentation level
    pub indent: usize,
    /// Indent string (e.g., "    " or "\t")
    pub indent_str: String,
    /// T1 primitives encountered during emission
    pub primitives_used: HashSet<LexPrimitiva>,
    /// Variables in scope
    pub scope: Vec<String>,
    /// Accumulated warnings
    pub warnings: Vec<String>,
}

impl EmitContext {
    /// Create new emission context
    #[must_use]
    pub fn new() -> Self {
        Self {
            indent: 0,
            indent_str: "    ".to_string(),
            primitives_used: HashSet::new(),
            scope: Vec::new(),
            warnings: Vec::new(),
        }
    }

    /// Get current indentation string
    #[must_use]
    pub fn indentation(&self) -> String {
        self.indent_str.repeat(self.indent)
    }

    /// Increase indentation
    pub fn indent(&mut self) {
        self.indent += 1;
    }

    /// Decrease indentation
    pub fn dedent(&mut self) {
        if self.indent > 0 {
            self.indent -= 1;
        }
    }

    /// Record a primitive used
    pub fn record_primitive(&mut self, prim: LexPrimitiva) {
        self.primitives_used.insert(prim);
    }

    /// Add a variable to scope
    pub fn add_to_scope(&mut self, name: impl Into<String>) {
        self.scope.push(name.into());
    }

    /// Check if variable is in scope
    #[must_use]
    pub fn in_scope(&self, name: &str) -> bool {
        self.scope.contains(&name.to_string())
    }

    /// Add a warning
    pub fn warn(&mut self, msg: impl Into<String>) {
        self.warnings.push(msg.into());
    }

    /// Calculate transfer confidence based on primitives used
    #[must_use]
    pub fn transfer_confidence(&self, target: TargetLanguage) -> f64 {
        if self.primitives_used.is_empty() {
            return 1.0;
        }

        // Base confidence from target language
        let base = target.transfer_confidence();

        // Adjust based on primitive complexity
        let t1_count = self.primitives_used.len();
        let complexity_factor = if t1_count <= 3 {
            1.0 // Simple: T2-P
        } else if t1_count <= 5 {
            0.9 // Medium: T2-C
        } else {
            0.8 // Complex: T3
        };

        base * complexity_factor
    }
}

impl Default for EmitContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of code emission.
///
/// ## Tier: T2-P (ς + N)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmitResult {
    /// Generated code
    pub code: String,
    /// Target language
    pub language: String,
    /// Transfer confidence (0.0 - 1.0)
    pub confidence: f64,
    /// T1 primitives used
    pub primitives: Vec<String>,
    /// Any warnings generated
    pub warnings: Vec<String>,
}

/// Code emitter orchestrating backend generation.
///
/// ## Tier: T2-C (μ + → + σ)
pub struct CodeEmitter<B: Backend> {
    backend: B,
}

impl<B: Backend> CodeEmitter<B> {
    /// Create new emitter with backend
    #[must_use]
    pub fn new(backend: B) -> Self {
        Self { backend }
    }

    /// Get the backend
    #[must_use]
    pub fn backend(&self) -> &B {
        &self.backend
    }

    /// Emit code for a program
    pub fn emit(&self, program: &Program) -> Result<EmitResult, CodegenError> {
        let mut ctx = EmitContext::new();
        let code = self.backend.emit_program(program, &mut ctx)?;

        // Determine target language
        let language = self.backend.name().to_string();
        let target = match language.as_str() {
            "Rust" => TargetLanguage::Rust,
            "Python" => TargetLanguage::Python,
            "TypeScript" => TargetLanguage::TypeScript,
            "Go" => TargetLanguage::Go,
            "C" => TargetLanguage::C,
            _ => TargetLanguage::Rust, // Default
        };

        Ok(EmitResult {
            code,
            language,
            confidence: ctx.transfer_confidence(target),
            primitives: ctx
                .primitives_used
                .iter()
                .map(|p| p.symbol().to_string())
                .collect(),
            warnings: ctx.warnings,
        })
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_new() {
        let ctx = EmitContext::new();
        assert_eq!(ctx.indent, 0);
        assert!(ctx.primitives_used.is_empty());
        assert!(ctx.scope.is_empty());
    }

    #[test]
    fn test_context_indentation() {
        let mut ctx = EmitContext::new();
        assert_eq!(ctx.indentation(), "");

        ctx.indent();
        assert_eq!(ctx.indentation(), "    ");

        ctx.indent();
        assert_eq!(ctx.indentation(), "        ");

        ctx.dedent();
        assert_eq!(ctx.indentation(), "    ");
    }

    #[test]
    fn test_context_scope() {
        let mut ctx = EmitContext::new();
        assert!(!ctx.in_scope("x"));

        ctx.add_to_scope("x");
        assert!(ctx.in_scope("x"));
        assert!(!ctx.in_scope("y"));
    }

    #[test]
    fn test_context_primitives() {
        let mut ctx = EmitContext::new();
        ctx.record_primitive(LexPrimitiva::Sequence);
        ctx.record_primitive(LexPrimitiva::Mapping);
        ctx.record_primitive(LexPrimitiva::Sequence); // Duplicate

        assert_eq!(ctx.primitives_used.len(), 2);
    }

    #[test]
    fn test_transfer_confidence_simple() {
        let mut ctx = EmitContext::new();
        ctx.record_primitive(LexPrimitiva::Quantity);

        let conf = ctx.transfer_confidence(TargetLanguage::Rust);
        assert!(conf > 0.9); // High confidence for Rust with simple primitives
    }

    #[test]
    fn test_transfer_confidence_complex() {
        let mut ctx = EmitContext::new();
        for prim in LexPrimitiva::all().iter().take(8) {
            ctx.record_primitive(*prim);
        }

        let conf = ctx.transfer_confidence(TargetLanguage::Python);
        // Python base (0.75) × complexity factor (0.8) = 0.6
        assert!(conf < 0.7);
    }

    #[test]
    fn test_emit_result_serialization() {
        let result = EmitResult {
            code: "fn main() {}".to_string(),
            language: "Rust".to_string(),
            confidence: 0.95,
            primitives: vec!["μ".to_string(), "→".to_string()],
            warnings: vec![],
        };

        let json = serde_json::to_string(&result);
        assert!(json.is_ok());
    }
}
