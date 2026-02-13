//! # GroundsTo implementations for nexcore-prima types
//!
//! Connects Prima language types to the Lex Primitiva type system.
//!
//! ## Domain Signature
//!
//! - **ρ (Recursion)**: AST is recursive (expressions contain expressions)
//! - **σ (Sequence)**: programs are statement sequences, token streams
//! - **Σ (Sum)**: most AST nodes are enum variants

use nexcore_lex_primitiva::grounding::GroundsTo;
use nexcore_lex_primitiva::primitiva::{LexPrimitiva, PrimitiveComposition};

use crate::ast::{Block, Expr, Literal, Program, Stmt};
use crate::error::PrimaError;
use crate::interpret::Interpreter;
use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::token::Span;
use crate::types::PrimaType;
use crate::value::Value;

// ---------------------------------------------------------------------------
// T1/T2-P: Token types
// ---------------------------------------------------------------------------

/// Span: T2-P (λ + N), dominant λ
///
/// Source location span. Location-dominant: addressing source positions.
impl GroundsTo for Span {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Location, // λ -- source position
            LexPrimitiva::Quantity, // N -- byte offsets, line number
        ])
        .with_dominant(LexPrimitiva::Location, 0.90)
    }
}

// ---------------------------------------------------------------------------
// T2-P/T2-C: AST types
// ---------------------------------------------------------------------------

/// Literal: T2-P (N + Σ), dominant N
///
/// Literal values: integers, floats, booleans, strings, unit.
impl GroundsTo for Literal {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity, // N -- numeric values
            LexPrimitiva::Sum,      // Σ -- variant enumeration
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.85)
    }
}

/// Expr: T2-C (ρ + Σ + → + N), dominant ρ
///
/// Expressions: recursive by nature (binary ops contain sub-expressions).
impl GroundsTo for Expr {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Recursion, // ρ -- nested sub-expressions
            LexPrimitiva::Sum,       // Σ -- variant enumeration
            LexPrimitiva::Causality, // → -- evaluation produces value
            LexPrimitiva::Quantity,  // N -- literal values
        ])
        .with_dominant(LexPrimitiva::Recursion, 0.85)
    }
}

/// Stmt: T2-C (ς + Σ + → + ρ), dominant ς
///
/// Statements modify state: let bindings, fn defs, type defs.
impl GroundsTo for Stmt {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::State,     // ς -- state modification
            LexPrimitiva::Sum,       // Σ -- variant enumeration
            LexPrimitiva::Causality, // → -- execution effect
            LexPrimitiva::Recursion, // ρ -- contains Expr
        ])
        .with_dominant(LexPrimitiva::State, 0.80)
    }
}

/// Block: T2-P (σ + ρ), dominant σ
///
/// A block of statements. Sequence-dominant: ordered execution.
impl GroundsTo for Block {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sequence,  // σ -- statement ordering
            LexPrimitiva::Recursion, // ρ -- blocks nest in fn/if
        ])
        .with_dominant(LexPrimitiva::Sequence, 0.85)
    }
}

/// Program: T2-P (σ + ρ), dominant σ
///
/// Top-level program: sequence of statements.
impl GroundsTo for Program {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sequence,  // σ -- statement sequence
            LexPrimitiva::Recursion, // ρ -- recursive structure
        ])
        .with_dominant(LexPrimitiva::Sequence, 0.90)
    }
}

// ---------------------------------------------------------------------------
// T2-P: Type system
// ---------------------------------------------------------------------------

/// PrimaType: T2-P (Σ + κ), dominant Σ
///
/// Prima type annotation. Sum-dominant: one of many type variants.
impl GroundsTo for PrimaType {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum,        // Σ -- type variant enumeration
            LexPrimitiva::Comparison, // κ -- type checking
        ])
        .with_dominant(LexPrimitiva::Sum, 0.85)
    }
}

/// Value: T2-P (Σ + N), dominant Σ
///
/// Runtime value: Int, Float, Bool, String, Unit, List, Function.
impl GroundsTo for Value {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum,      // Σ -- variant enumeration
            LexPrimitiva::Quantity, // N -- numeric payload
        ])
        .with_dominant(LexPrimitiva::Sum, 0.85)
    }
}

// ---------------------------------------------------------------------------
// T3: Pipeline types
// ---------------------------------------------------------------------------

/// Lexer: T3 (σ + μ + ∂ + λ + ρ + N), dominant σ
///
/// Tokenizer: scans source string into token sequence.
impl GroundsTo for Lexer<'_> {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sequence,  // σ -- character stream → token stream
            LexPrimitiva::Mapping,   // μ -- chars → tokens
            LexPrimitiva::Boundary,  // ∂ -- delimiter recognition
            LexPrimitiva::Location,  // λ -- position tracking
            LexPrimitiva::Recursion, // ρ -- nested string/comment handling
            LexPrimitiva::Quantity,  // N -- numeric literal parsing
        ])
        .with_dominant(LexPrimitiva::Sequence, 0.80)
    }
}

/// Parser: T3 (ρ + σ + Σ + ∂ + → + μ), dominant ρ
///
/// Recursive descent parser: token stream → AST.
impl GroundsTo for Parser {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Recursion, // ρ -- recursive descent
            LexPrimitiva::Sequence,  // σ -- token consumption
            LexPrimitiva::Sum,       // Σ -- AST variant production
            LexPrimitiva::Boundary,  // ∂ -- syntax constraints
            LexPrimitiva::Causality, // → -- parse → AST
            LexPrimitiva::Mapping,   // μ -- tokens → nodes
        ])
        .with_dominant(LexPrimitiva::Recursion, 0.85)
    }
}

/// Interpreter: T3 (→ + ρ + ς + μ + σ + ∂), dominant →
///
/// Tree-walking interpreter: AST → values.
impl GroundsTo for Interpreter {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Causality, // → -- evaluation causes effects
            LexPrimitiva::Recursion, // ρ -- recursive evaluation
            LexPrimitiva::State,     // ς -- environment/scope
            LexPrimitiva::Mapping,   // μ -- name → value binding
            LexPrimitiva::Sequence,  // σ -- statement execution order
            LexPrimitiva::Boundary,  // ∂ -- type/runtime errors
        ])
        .with_dominant(LexPrimitiva::Causality, 0.80)
    }
}

// ---------------------------------------------------------------------------
// Error types
// ---------------------------------------------------------------------------

/// PrimaError: T2-C (∂ + λ + Σ + ∅), dominant ∂
///
/// Prima language errors: lex, parse, type, runtime.
impl GroundsTo for PrimaError {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary, // ∂ -- syntax/type constraints
            LexPrimitiva::Location, // λ -- error span
            LexPrimitiva::Sum,      // Σ -- error variant
            LexPrimitiva::Void,     // ∅ -- undefined variable
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.85)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use nexcore_lex_primitiva::tier::Tier;

    #[test]
    fn span_is_location_dominant() {
        assert_eq!(
            <Span as GroundsTo>::dominant_primitive(),
            Some(LexPrimitiva::Location)
        );
        assert_eq!(<Span as GroundsTo>::tier(), Tier::T2Primitive);
    }

    #[test]
    fn expr_is_recursion_dominant() {
        assert_eq!(
            <Expr as GroundsTo>::dominant_primitive(),
            Some(LexPrimitiva::Recursion)
        );
        assert_eq!(<Expr as GroundsTo>::tier(), Tier::T2Composite);
    }

    #[test]
    fn program_is_sequence_dominant() {
        assert_eq!(
            <Program as GroundsTo>::dominant_primitive(),
            Some(LexPrimitiva::Sequence)
        );
    }

    #[test]
    fn parser_is_recursion_dominant() {
        assert_eq!(
            <Parser as GroundsTo>::dominant_primitive(),
            Some(LexPrimitiva::Recursion)
        );
        assert_eq!(<Parser as GroundsTo>::tier(), Tier::T3DomainSpecific);
    }

    #[test]
    fn interpreter_is_causality_dominant() {
        assert_eq!(
            <Interpreter as GroundsTo>::dominant_primitive(),
            Some(LexPrimitiva::Causality)
        );
    }

    #[test]
    fn value_is_sum_dominant() {
        assert_eq!(
            <Value as GroundsTo>::dominant_primitive(),
            Some(LexPrimitiva::Sum)
        );
    }

    #[test]
    fn prima_error_is_boundary_dominant() {
        assert_eq!(
            <PrimaError as GroundsTo>::dominant_primitive(),
            Some(LexPrimitiva::Boundary)
        );
    }
}
