//! # Lex Primitiva Grounding for nexcore-pvdsl
//!
//! GroundsTo implementations for all nexcore-pvdsl public types.
//! PVDSL is a domain-specific language for pharmacovigilance workflows,
//! providing lexer, parser, AST, bytecode compiler, VM, and transpiler.
//!
//! ## Type Grounding Table
//!
//! | Type | Primitives | Dominant | Tier | Rationale |
//! |------|-----------|----------|------|-----------|
//! | TokenType | Σ | Σ | T1 | Sum enum of token classifications |
//! | Token | ς × λ | ς | T2-P | Stateful token with position context |
//! | Lexer | ς σ | ς | T2-P | Stateful scanner with sequential advance |
//! | Parser | ς σ ρ | ς | T2-P | Stateful recursive descent parser |
//! | Statement | Σ ρ σ | Σ | T2-P | Recursive sum of statement variants |
//! | Expression | Σ ρ | Σ | T2-P | Recursive sum of expression variants |
//! | Program | σ ς × | σ | T2-P | Ordered sequence of statements with metadata |
//! | ProgramMetadata | ς | ς | T1 | Simple metadata state |
//! | OpCode | Σ | Σ | T1 | Sum enum of bytecode operation codes |
//! | CompiledProgram | σ ς × | σ | T2-P | Ordered instructions with constant/name pools |
//! | BytecodeGenerator | ς σ μ | ς | T2-P | Stateful AST-to-bytecode transformer |
//! | RuntimeValue | Σ ρ | Σ | T2-P | Recursive sum of runtime value types |
//! | VirtualMachine | ς σ μ | ς | T2-P | Stateful stack machine executing bytecode |
//! | PvdslEngine | ς σ μ | ς | T2-P | Stateful high-level execution engine |
//! | PvdslError | Σ | Σ | T1 | Sum enum of error variants |
//! | RuleCondition | ∂ κ N | ∂ | T2-P | Boundary condition with threshold comparison |
//! | RegulatoryRule | ∂ → σ ς | ∂ | T2-C | Boundary rule with causal action sequence |
//! | GvpTranspiler | μ σ ∂ ς | μ | T2-C | Mapping from guidelines to PVDSL with state |
//! | LevenshteinResult | N κ | N | T2-P | Numeric distance with similarity comparison |

use nexcore_lex_primitiva::grounding::GroundsTo;
use nexcore_lex_primitiva::primitiva::{LexPrimitiva, PrimitiveComposition};
use nexcore_lex_primitiva::state_mode::StateMode;

use crate::ast::{Expression, Program, ProgramMetadata, Statement};
use crate::bytecode::{BytecodeGenerator, CompiledProgram, OpCode};
use crate::engine::PvdslEngine;
use crate::error::PvdslError;
use crate::lexer::{Lexer, Token, TokenType};
use crate::parser::Parser;
use crate::runtime::RuntimeValue;
use crate::text::LevenshteinResult;
use crate::transpiler::{GvpTranspiler, RegulatoryRule, RuleCondition};
use crate::vm::VirtualMachine;

// ============================================================================
// T1 Universal (1 unique primitive)
// ============================================================================

/// TokenType: Sum enum of all possible token classifications.
/// Tier: T1Universal. Dominant: Σ Sum.
/// WHY: Pure one-of-N exhaustive classification with no additional structure.
impl GroundsTo for TokenType {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum, // Σ -- one-of-N token category
        ])
        .with_dominant(LexPrimitiva::Sum, 1.0)
    }
}

/// OpCode: Sum enum of bytecode instruction types.
/// Tier: T1Universal. Dominant: Σ Sum.
/// WHY: Pure one-of-N instruction variant (LoadConst, BinaryAdd, Jump, etc).
impl GroundsTo for OpCode {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum, // Σ -- one-of-N instruction variant
        ])
        .with_dominant(LexPrimitiva::Sum, 1.0)
    }
}

/// ProgramMetadata: Simple boolean metadata about a program.
/// Tier: T1Universal. Dominant: ς State.
/// WHY: Single boolean flag -- minimal state with no other structure.
impl GroundsTo for ProgramMetadata {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::State, // ς -- simple metadata state
        ])
        .with_dominant(LexPrimitiva::State, 1.0)
        .with_state_mode(StateMode::Modal)
    }

    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Modal)
    }
}

/// PvdslError: Sum enum of error variants.
/// Tier: T1Universal. Dominant: Σ Sum.
/// WHY: One-of-N error classification (ParseError, TypeError, etc).
impl GroundsTo for PvdslError {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum, // Σ -- one-of-N error variant
        ])
        .with_dominant(LexPrimitiva::Sum, 1.0)
    }
}

// ============================================================================
// T2-P (2-3 unique primitives)
// ============================================================================

/// Token: Lexer token with type, value, and position.
/// Tier: T2Primitive. Dominant: ς State.
/// WHY: Encapsulates token state (type + value) with positional context.
impl GroundsTo for Token {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::State,    // ς -- token type + value state
            LexPrimitiva::Product,  // × -- (type, value, line, col) product
            LexPrimitiva::Location, // λ -- source position (line, column)
        ])
        .with_dominant(LexPrimitiva::State, 0.70)
        .with_state_mode(StateMode::Modal)
    }

    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Modal)
    }
}

/// Lexer: Stateful scanner advancing through source characters.
/// Tier: T2Primitive. Dominant: ς State.
/// WHY: Mutable state (pos, line, column) with sequential character scanning.
impl GroundsTo for Lexer {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::State,    // ς -- mutable scanner state
            LexPrimitiva::Sequence, // σ -- sequential character scanning
        ])
        .with_dominant(LexPrimitiva::State, 0.80)
        .with_state_mode(StateMode::Mutable)
    }

    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Mutable)
    }
}

/// Parser: Recursive descent parser converting tokens to AST.
/// Tier: T2Primitive. Dominant: ς State.
/// WHY: Mutable state (current position) with recursive descent and sequential consumption.
impl GroundsTo for Parser {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::State,     // ς -- parser state (current token)
            LexPrimitiva::Sequence,  // σ -- sequential token consumption
            LexPrimitiva::Recursion, // ρ -- recursive descent into sub-expressions
        ])
        .with_dominant(LexPrimitiva::State, 0.70)
        .with_state_mode(StateMode::Mutable)
    }

    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Mutable)
    }
}

/// Statement: Recursive sum of statement node variants.
/// Tier: T2Primitive. Dominant: Σ Sum.
/// WHY: AST node sum type with recursive body references (FunctionDef, IfStatement).
impl GroundsTo for Statement {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum,       // Σ -- one-of-7 statement variant
            LexPrimitiva::Recursion, // ρ -- recursive body (Vec<Statement>)
            LexPrimitiva::Sequence,  // σ -- ordered body statements
        ])
        .with_dominant(LexPrimitiva::Sum, 0.70)
    }
}

/// Expression: Recursive sum of expression node variants.
/// Tier: T2Primitive. Dominant: Σ Sum.
/// WHY: AST node sum type with recursive Box<Expression> references.
impl GroundsTo for Expression {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum,       // Σ -- one-of-7 expression variant
            LexPrimitiva::Recursion, // ρ -- recursive Box<Expression> in BinaryExpression
        ])
        .with_dominant(LexPrimitiva::Sum, 0.75)
    }
}

/// Program: Ordered sequence of top-level statements with metadata.
/// Tier: T2Primitive. Dominant: σ Sequence.
/// WHY: Ordered list of statements forming a complete program.
impl GroundsTo for Program {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sequence, // σ -- ordered statement list
            LexPrimitiva::State,    // ς -- metadata state
            LexPrimitiva::Product,  // × -- (statements, metadata) product
        ])
        .with_dominant(LexPrimitiva::Sequence, 0.75)
        .with_state_mode(StateMode::Accumulated)
    }

    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Accumulated)
    }
}

/// CompiledProgram: Compiled bytecode with constant and name pools.
/// Tier: T2Primitive. Dominant: σ Sequence.
/// WHY: Ordered instruction sequence with associated lookup pools.
impl GroundsTo for CompiledProgram {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sequence, // σ -- ordered instruction sequence
            LexPrimitiva::State,    // ς -- constant/name pool state
            LexPrimitiva::Product,  // × -- (instructions, constants, names) product
        ])
        .with_dominant(LexPrimitiva::Sequence, 0.70)
        .with_state_mode(StateMode::Accumulated)
    }

    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Accumulated)
    }
}

/// BytecodeGenerator: Stateful AST-to-bytecode compiler.
/// Tier: T2Primitive. Dominant: ς State.
/// WHY: Accumulates instructions/constants/names while traversing AST.
impl GroundsTo for BytecodeGenerator {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::State,    // ς -- mutable compilation state
            LexPrimitiva::Sequence, // σ -- sequential instruction emission
            LexPrimitiva::Mapping,  // μ -- AST -> bytecode transformation
        ])
        .with_dominant(LexPrimitiva::State, 0.70)
        .with_state_mode(StateMode::Mutable)
    }

    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Mutable)
    }
}

/// RuntimeValue: Recursive sum type of runtime values.
/// Tier: T2Primitive. Dominant: Σ Sum.
/// WHY: One-of-6 value variant, List contains Vec<RuntimeValue> (recursive).
impl GroundsTo for RuntimeValue {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum,       // Σ -- one-of-6 value variant
            LexPrimitiva::Recursion, // ρ -- List(Vec<RuntimeValue>) self-reference
        ])
        .with_dominant(LexPrimitiva::Sum, 0.80)
    }
}

/// VirtualMachine: Stack-based bytecode interpreter.
/// Tier: T2Primitive. Dominant: ς State.
/// WHY: Mutable state (stack, variables, natives) executing sequential bytecode.
impl GroundsTo for VirtualMachine {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::State,    // ς -- stack + variable state
            LexPrimitiva::Sequence, // σ -- sequential instruction execution
            LexPrimitiva::Mapping,  // μ -- native function dispatch mapping
        ])
        .with_dominant(LexPrimitiva::State, 0.70)
        .with_state_mode(StateMode::Mutable)
    }

    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Mutable)
    }
}

/// PvdslEngine: High-level scripting engine (compile + execute).
/// Tier: T2Primitive. Dominant: ς State.
/// WHY: Wraps VirtualMachine state, provides sequential compile-then-execute.
impl GroundsTo for PvdslEngine {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::State,    // ς -- wraps VM state
            LexPrimitiva::Sequence, // σ -- compile -> execute pipeline
            LexPrimitiva::Mapping,  // μ -- source -> result transformation
        ])
        .with_dominant(LexPrimitiva::State, 0.70)
        .with_state_mode(StateMode::Mutable)
    }

    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Mutable)
    }
}

/// RuleCondition: Single condition within a regulatory rule.
/// Tier: T2Primitive. Dominant: ∂ Boundary.
/// WHY: Defines a threshold boundary (metric > threshold) for safety gating.
impl GroundsTo for RuleCondition {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary,   // ∂ -- threshold boundary
            LexPrimitiva::Comparison, // κ -- comparison operator
            LexPrimitiva::Quantity,   // N -- numeric threshold value
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.75)
    }
}

/// LevenshteinResult: Edit distance with similarity ratio.
/// Tier: T2Primitive. Dominant: N Quantity.
/// WHY: Numeric distance measurement with similarity comparison.
impl GroundsTo for LevenshteinResult {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity,   // N -- edit distance and similarity
            LexPrimitiva::Comparison, // κ -- similarity comparison
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.80)
    }
}

// ============================================================================
// T2-C (4-5 unique primitives)
// ============================================================================

/// RegulatoryRule: Complex rule with conditions, action, and source.
/// Tier: T2Composite. Dominant: ∂ Boundary.
/// WHY: Safety boundary rule: conditions define thresholds, action enforces boundary,
/// triggered sequentially. Regulatory compliance is boundary enforcement.
impl GroundsTo for RegulatoryRule {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary,  // ∂ -- regulatory boundary enforcement
            LexPrimitiva::Causality, // → -- rule triggers action
            LexPrimitiva::Sequence,  // σ -- conditions evaluated in order
            LexPrimitiva::State,     // ς -- rule metadata state
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.65)
        .with_state_mode(StateMode::Modal)
    }

    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Modal)
    }
}

/// GvpTranspiler: Transpiler converting regulatory guidelines to PVDSL scripts.
/// Tier: T2Composite. Dominant: μ Mapping.
/// WHY: Core function is transformation from guidelines to executable scripts.
/// Maintains state (accumulated rules) and processes sequentially.
impl GroundsTo for GvpTranspiler {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping,  // μ -- guideline -> PVDSL transformation
            LexPrimitiva::Sequence, // σ -- ordered rule processing
            LexPrimitiva::Boundary, // ∂ -- regulatory boundary context
            LexPrimitiva::State,    // ς -- accumulated rules state
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.65)
        .with_state_mode(StateMode::Mutable)
    }

    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Mutable)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nexcore_lex_primitiva::tier::Tier;

    #[test]
    fn token_type_is_t1() {
        assert_eq!(TokenType::tier(), Tier::T1Universal);
        assert_eq!(TokenType::dominant_primitive(), Some(LexPrimitiva::Sum));
    }

    #[test]
    fn opcode_is_t1() {
        assert_eq!(OpCode::tier(), Tier::T1Universal);
        assert_eq!(OpCode::dominant_primitive(), Some(LexPrimitiva::Sum));
    }

    #[test]
    fn program_metadata_is_t1() {
        assert_eq!(ProgramMetadata::tier(), Tier::T1Universal);
        assert_eq!(
            ProgramMetadata::dominant_primitive(),
            Some(LexPrimitiva::State)
        );
    }

    #[test]
    fn pvdsl_error_is_t1() {
        assert_eq!(PvdslError::tier(), Tier::T1Universal);
        assert_eq!(PvdslError::dominant_primitive(), Some(LexPrimitiva::Sum));
    }

    #[test]
    fn token_is_t2p() {
        assert_eq!(Token::tier(), Tier::T2Primitive);
        assert_eq!(Token::dominant_primitive(), Some(LexPrimitiva::State));
    }

    #[test]
    fn lexer_is_t2p() {
        assert_eq!(Lexer::tier(), Tier::T2Primitive);
        assert_eq!(Lexer::dominant_primitive(), Some(LexPrimitiva::State));
    }

    #[test]
    fn parser_is_t2p() {
        assert_eq!(Parser::tier(), Tier::T2Primitive);
        assert_eq!(Parser::dominant_primitive(), Some(LexPrimitiva::State));
    }

    #[test]
    fn statement_is_t2p() {
        assert_eq!(Statement::tier(), Tier::T2Primitive);
        assert_eq!(Statement::dominant_primitive(), Some(LexPrimitiva::Sum));
    }

    #[test]
    fn expression_is_t2p() {
        assert_eq!(Expression::tier(), Tier::T2Primitive);
        assert_eq!(Expression::dominant_primitive(), Some(LexPrimitiva::Sum));
    }

    #[test]
    fn program_is_t2p() {
        assert_eq!(Program::tier(), Tier::T2Primitive);
        assert_eq!(Program::dominant_primitive(), Some(LexPrimitiva::Sequence));
    }

    #[test]
    fn compiled_program_is_t2p() {
        assert_eq!(CompiledProgram::tier(), Tier::T2Primitive);
        assert_eq!(
            CompiledProgram::dominant_primitive(),
            Some(LexPrimitiva::Sequence)
        );
    }

    #[test]
    fn bytecode_generator_is_t2p() {
        assert_eq!(BytecodeGenerator::tier(), Tier::T2Primitive);
        assert_eq!(
            BytecodeGenerator::dominant_primitive(),
            Some(LexPrimitiva::State)
        );
    }

    #[test]
    fn runtime_value_is_t2p() {
        assert_eq!(RuntimeValue::tier(), Tier::T2Primitive);
        assert_eq!(RuntimeValue::dominant_primitive(), Some(LexPrimitiva::Sum));
    }

    #[test]
    fn virtual_machine_is_t2p() {
        assert_eq!(VirtualMachine::tier(), Tier::T2Primitive);
        assert_eq!(
            VirtualMachine::dominant_primitive(),
            Some(LexPrimitiva::State)
        );
    }

    #[test]
    fn pvdsl_engine_is_t2p() {
        assert_eq!(PvdslEngine::tier(), Tier::T2Primitive);
        assert_eq!(PvdslEngine::dominant_primitive(), Some(LexPrimitiva::State));
    }

    #[test]
    fn rule_condition_is_t2p_boundary() {
        assert_eq!(RuleCondition::tier(), Tier::T2Primitive);
        assert_eq!(
            RuleCondition::dominant_primitive(),
            Some(LexPrimitiva::Boundary)
        );
    }

    #[test]
    fn levenshtein_result_is_t2p_quantity() {
        assert_eq!(LevenshteinResult::tier(), Tier::T2Primitive);
        assert_eq!(
            LevenshteinResult::dominant_primitive(),
            Some(LexPrimitiva::Quantity)
        );
    }

    #[test]
    fn regulatory_rule_is_t2c_boundary() {
        assert_eq!(RegulatoryRule::tier(), Tier::T2Composite);
        assert_eq!(
            RegulatoryRule::dominant_primitive(),
            Some(LexPrimitiva::Boundary)
        );
    }

    #[test]
    fn gvp_transpiler_is_t2c_mapping() {
        assert_eq!(GvpTranspiler::tier(), Tier::T2Composite);
        assert_eq!(
            GvpTranspiler::dominant_primitive(),
            Some(LexPrimitiva::Mapping)
        );
    }
}
