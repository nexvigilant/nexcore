// Copyright © 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! Prima Universal Code Generator
//!
//! ## Tier: T2-C (μ + σ + → + Σ)
//!
//! Generates target language code from Prima AST with transfer confidence tracking.
//!
//! ```text
//! Prima AST → Backend.emit() → Target Code
//!     ↓
//! T1 Primitives tracked through generation
//! ```

#![forbid(unsafe_code)]
#![cfg_attr(
    not(test),
    deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)
)]

pub mod backends;
mod emitter;
mod error;
mod primitives;

pub use emitter::{CodeEmitter, EmitContext, EmitResult};
pub use error::CodegenError;
pub use primitives::{PrimitiveMapping, TargetConstruct};

use lex_primitiva::LexPrimitiva;
use prima::prelude::{Expr, Program, Stmt};

/// Code generation backend trait.
///
/// ## Tier: T2-P (μ + →)
///
/// Each backend implements emission for a target language.
pub trait Backend {
    /// Target language name (e.g., "Rust", "Python", "TypeScript")
    fn name(&self) -> &'static str;

    /// File extension for generated code
    fn extension(&self) -> &'static str;

    /// Map a T1 primitive to target language construct
    fn map_primitive(&self, prim: LexPrimitiva) -> TargetConstruct;

    /// Emit a complete program
    fn emit_program(
        &self,
        program: &Program,
        ctx: &mut EmitContext,
    ) -> Result<String, CodegenError>;

    /// Emit a single statement
    fn emit_stmt(&self, stmt: &Stmt, ctx: &mut EmitContext) -> Result<String, CodegenError>;

    /// Emit an expression
    fn emit_expr(&self, expr: &Expr, ctx: &mut EmitContext) -> Result<String, CodegenError>;
}

/// Supported target languages.
///
/// ## Tier: T2-P (Σ)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TargetLanguage {
    Rust,
    Python,
    TypeScript,
    Go,
    C,
}

impl TargetLanguage {
    /// Get all supported languages
    #[must_use]
    pub fn all() -> &'static [Self] {
        &[
            Self::Rust,
            Self::Python,
            Self::TypeScript,
            Self::Go,
            Self::C,
        ]
    }

    /// Get language name
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Rust => "Rust",
            Self::Python => "Python",
            Self::TypeScript => "TypeScript",
            Self::Go => "Go",
            Self::C => "C",
        }
    }

    /// Get file extension
    #[must_use]
    pub const fn extension(self) -> &'static str {
        match self {
            Self::Rust => "rs",
            Self::Python => "py",
            Self::TypeScript => "ts",
            Self::Go => "go",
            Self::C => "c",
        }
    }

    /// Transfer confidence for this target
    ///
    /// Based on how well T1 primitives map to the target.
    #[must_use]
    pub const fn transfer_confidence(self) -> f64 {
        match self {
            Self::Rust => 0.95,       // Excellent primitive mapping
            Self::TypeScript => 0.85, // Good, some type system gaps
            Self::Go => 0.80,         // Good, lacks generics nuance
            Self::Python => 0.75,     // Dynamic typing reduces confidence
            Self::C => 0.70,          // Manual memory, no sum types
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_target_language_all() {
        assert_eq!(TargetLanguage::all().len(), 5);
    }

    #[test]
    fn test_target_language_names() {
        assert_eq!(TargetLanguage::Rust.name(), "Rust");
        assert_eq!(TargetLanguage::Python.name(), "Python");
        assert_eq!(TargetLanguage::TypeScript.name(), "TypeScript");
    }

    #[test]
    fn test_target_language_extensions() {
        assert_eq!(TargetLanguage::Rust.extension(), "rs");
        assert_eq!(TargetLanguage::Python.extension(), "py");
        assert_eq!(TargetLanguage::TypeScript.extension(), "ts");
    }

    #[test]
    fn test_transfer_confidence_ordering() {
        // Rust should have highest confidence
        assert!(
            TargetLanguage::Rust.transfer_confidence()
                > TargetLanguage::Python.transfer_confidence()
        );
        // All confidences should be in valid range
        for lang in TargetLanguage::all() {
            let conf = lang.transfer_confidence();
            assert!(
                conf >= 0.0 && conf <= 1.0,
                "{:?} has invalid confidence",
                lang
            );
        }
    }
}
