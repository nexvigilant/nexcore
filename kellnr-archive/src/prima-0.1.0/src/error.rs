// Copyright © 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! # Prima Error Types
//!
//! Error types mathematically grounded in the ∂ (Boundary) primitive.
//!
//! ## Mathematical Foundation
//!
//! Errors represent boundaries in computation:
//! - **∂ (Boundary)**: Limits and constraints that halt execution
//! - **Σ (Sum)**: Errors are one variant of many possible outcomes
//! - **∅ (Void)**: Errors represent absence of expected value
//!
//! ## Grounding Path
//!
//! PrimaError → ∂ (Boundary) → κ (Comparison) → N (Quantity) → {0, 1}
//!
//! ## Tier: T2-P (∂ + Σ)

use crate::token::Span;
use thiserror::Error;

/// Prima error type.
///
/// Each variant represents a specific boundary violation in the computation.
///
/// ## Grounding
/// - Composition: ∂ (Boundary) + Σ (Sum) + λ (Location)
/// - Tier: T2-P
#[derive(Debug, Clone, Error)]
pub enum PrimaError {
    /// Lexer boundary: invalid character sequence.
    /// Grounding: σ (Sequence) violated at ∂ (Boundary)
    #[error("∂[lexer] at {span}: {message}")]
    Lexer { span: Span, message: String },

    /// Parser boundary: syntax violation.
    /// Grounding: μ (Mapping) from tokens to AST failed
    #[error("∂[parser] at {span}: {message}")]
    Parser { span: Span, message: String },

    /// Analyzer boundary: semantic validation failed.
    /// Grounding: κ (Comparison) + Σ (Sum) for type checking
    #[error("∂[analyze] at {span}: {message}")]
    Analyzer { span: Span, message: String },

    /// Type boundary: composition mismatch.
    /// Grounding: Tier classification violation
    #[error("∂[type] at {span}: {message}")]
    Type { span: Span, message: String },

    /// Runtime boundary: evaluation halted.
    /// Grounding: → (Causality) chain broken
    #[error("∂[runtime]: {message}")]
    Runtime { message: String },

    /// Undefined reference: λ (Location) points to ∅ (Void).
    /// Grounding: λ → ∅
    #[error("∂[undefined]: λ({name}) → ∅")]
    Undefined { name: String },

    /// Division by zero: N/0 is undefined.
    /// Grounding: N ÷ 0 → ∂ (Boundary)
    #[error("∂[arithmetic]: N ÷ 0 undefined")]
    DivisionByZero,

    /// Integer overflow: N exceeds i64 bounds.
    /// Grounding: N → ∂ (Boundary exceeded)
    #[error("∂[arithmetic]: integer overflow")]
    Overflow,

    /// Recursion depth exceeded: ρ stack too deep.
    /// Grounding: ρ (Recursion) → ∂ (Boundary)
    #[error("∂[recursion]: max depth {depth} exceeded")]
    RecursionLimit { depth: usize },

    /// Step limit exceeded: → chain too long.
    /// Grounding: → (Causality) → ∂ (Boundary)
    #[error("∂[steps]: max steps {limit} exceeded")]
    StepLimit { limit: u64 },

    /// Eval disabled: code injection blocked.
    /// Grounding: → (Causality) + ∂ (Boundary)
    #[error("∂[eval]: eval() is disabled — enable with Interpreter::with_eval(true)")]
    EvalDisabled,

    /// Grounding violation: primitive composition invalid.
    /// Grounding: Composition fails to reach {0, 1}
    #[error("∂[grounding]: {message}")]
    Grounding { message: String },

    /// IO boundary: external system interaction failed.
    /// Grounding: π (Persistence) + ∂ (Boundary)
    #[error("∂[io]: {0}")]
    Io(String),
}

impl From<std::io::Error> for PrimaError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e.to_string())
    }
}

impl PrimaError {
    /// Create a lexer error.
    pub fn lexer(span: Span, message: impl Into<String>) -> Self {
        Self::Lexer {
            span,
            message: message.into(),
        }
    }

    /// Create a parser error.
    pub fn parser(span: Span, message: impl Into<String>) -> Self {
        Self::Parser {
            span,
            message: message.into(),
        }
    }

    /// Create an analyzer error.
    pub fn analyzer(span: Span, message: impl Into<String>) -> Self {
        Self::Analyzer {
            span,
            message: message.into(),
        }
    }

    /// Create a type error.
    pub fn type_error(span: Span, message: impl Into<String>) -> Self {
        Self::Type {
            span,
            message: message.into(),
        }
    }

    /// Create a runtime error.
    pub fn runtime(message: impl Into<String>) -> Self {
        Self::Runtime {
            message: message.into(),
        }
    }

    /// Create an undefined error.
    pub fn undefined(name: impl Into<String>) -> Self {
        Self::Undefined { name: name.into() }
    }

    /// Create a grounding error.
    pub fn grounding(message: impl Into<String>) -> Self {
        Self::Grounding {
            message: message.into(),
        }
    }

    /// Get the dominant primitive for this error type.
    #[must_use]
    pub const fn dominant_primitive(&self) -> &'static str {
        match self {
            Self::Lexer { .. } => "σ",          // Sequence violated
            Self::Parser { .. } => "μ",         // Mapping failed
            Self::Analyzer { .. } => "κ",       // Comparison/verification failed
            Self::Type { .. } => "κ",           // Comparison failed
            Self::Runtime { .. } => "→",        // Causality broken
            Self::Undefined { .. } => "λ",      // Location invalid
            Self::DivisionByZero => "N",        // Quantity undefined
            Self::Overflow => "N",              // Quantity overflow
            Self::RecursionLimit { .. } => "ρ", // Recursion exceeded
            Self::StepLimit { .. } => "→",      // Causality chain too long
            Self::EvalDisabled => "∂",          // Boundary enforced
            Self::Grounding { .. } => "∂",      // Boundary reached
            Self::Io(_) => "π",                 // Persistence failed
        }
    }
}

/// Result type for Prima operations.
///
/// Grounding: Σ(T, ∂) — Sum of success value or boundary error
pub type PrimaResult<T> = Result<T, PrimaError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display_uses_boundary_symbol() {
        let err = PrimaError::lexer(Span::new(0, 1, 0), "invalid");
        let s = format!("{}", err);
        assert!(s.contains("∂")); // Boundary symbol
    }

    #[test]
    fn test_undefined_shows_location_and_void() {
        let err = PrimaError::undefined("x");
        let s = format!("{}", err);
        assert!(s.contains("λ")); // Location
        assert!(s.contains("∅")); // Void
    }

    #[test]
    fn test_division_by_zero_shows_quantity() {
        let err = PrimaError::DivisionByZero;
        let s = format!("{}", err);
        assert!(s.contains("N")); // Quantity
        assert!(s.contains("÷")); // Division
        assert!(s.contains("0")); // Zero constant
    }

    #[test]
    fn test_dominant_primitives() {
        assert_eq!(
            PrimaError::lexer(Span::default(), "").dominant_primitive(),
            "σ"
        );
        assert_eq!(
            PrimaError::parser(Span::default(), "").dominant_primitive(),
            "μ"
        );
        assert_eq!(
            PrimaError::type_error(Span::default(), "").dominant_primitive(),
            "κ"
        );
        assert_eq!(PrimaError::runtime("").dominant_primitive(), "→");
        assert_eq!(PrimaError::undefined("x").dominant_primitive(), "λ");
        assert_eq!(PrimaError::DivisionByZero.dominant_primitive(), "N");
        assert_eq!(PrimaError::grounding("").dominant_primitive(), "∂");
    }

    #[test]
    fn test_grounding_error() {
        let err = PrimaError::grounding("composition does not reach {0, 1}");
        let s = format!("{}", err);
        assert!(s.contains("grounding"));
    }
}
