// Copyright © 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! # Prima (πρίμα) — Primitive-First Programming Language
//!
//! A language where every construct grounds to the 15 Lex Primitiva.
//!
//! ## Root Constants
//!
//! All computation grounds to: **0** (absence) and **1** (existence)
//!
//! ## The 15 Lex Primitiva
//!
//! σ μ ς ρ ∅ ∂ ν ∃ π → κ N λ ∝ Σ
//!
//! ## File Extension
//!
//! `.true` (code that compiles is true) or `.prima` (fallback)
//!
//! ## Tier System
//!
//! | Tier | Primitives | Transfer |
//! |------|------------|----------|
//! | T1 | 1 | 1.0 |
//! | T2-P | 2-3 | 0.9 |
//! | T2-C | 4-5 | 0.7 |
//! | T3 | 6+ | 0.4 |

#![forbid(unsafe_code)]
#![cfg_attr(not(test), deny(clippy::unwrap_used))]
#![cfg_attr(not(test), deny(clippy::expect_used))]
#![allow(
    missing_docs,
    clippy::allow_attributes_without_reason,
    reason = "Prima language surface is evolving rapidly; full docs are intentionally staged"
)]

pub mod analyze;
pub mod ast;
pub mod builtins;
pub mod bytecode;
pub mod codegen;
pub mod composition_track;
pub mod compress;
pub mod constants;
pub mod dev;
pub mod effects;
pub mod error;
pub mod exhaustive;
pub mod grounding;
pub mod infer;
pub mod interpret;
pub mod ir;
pub mod lexer;
pub mod module;
pub mod molecular;
pub mod nottrue;
pub mod optimize;
pub mod parser;
pub mod repl;
pub mod reverse;
pub mod stdlib;
pub mod token;
pub mod types;
pub mod value;
pub mod visual_repl;
pub mod vm;
pub mod vocabulary;

/// Prelude for convenient imports.
pub mod prelude {
    pub use crate::analyze::{AnalysisContext, Analyzer, InferredType};
    pub use crate::ast::{Block, Expr, Literal, Program, Stmt};
    pub use crate::bytecode::{
        BytecodeCompiler, BytecodeModule, Chunk, CompiledFunction, OpCode, disassemble,
    };
    pub use crate::codegen::Compiler;
    pub use crate::composition_track::{
        CompositionContext, CompositionEntry, CompositionStats, CompositionTracker, Derivation,
        tracking_composition,
    };
    pub use crate::compress::{
        Element, Lexicon, MolecularFormula, TokenMetrics, mapping, option_type, result_type, seq_n,
        sum_type,
    };
    pub use crate::constants::{
        ConstantTracker, GroundingTrace, PrimitiveGrounding, RootConstant, bool_to_constant,
        format_grounding_report, int_to_constant,
    };
    pub use crate::effects::{
        Effect, EffectContext, EffectSet, EffectSig, EffectViolation, builtin_effect,
        has_io_builtin, is_pure_builtin,
    };
    pub use crate::error::{PrimaError, PrimaResult};
    pub use crate::exhaustive::{
        BoolSpace, ExhaustivenessChecker, ExhaustivenessResult, ExhaustivenessWarning, IntSpace,
        MissingPattern, PatternSpace, RedundantArm, StringSpace, exhaustive_composition,
        is_exhaustive,
    };
    pub use crate::grounding::{
        GroundingResult, GroundingStep, GroundingVerifier, GroundingViolation, format_trace,
        verify_grounding,
    };
    pub use crate::infer::{
        Constraint, InferType, Substitution, TypeInferencer, TypeVar, infer_composition, unify,
    };
    pub use crate::interpret::Interpreter;
    pub use crate::ir::{
        BasicBlock, BlockId, Instruction, IrBuilder, IrConst, IrFunction, IrModule, Reg,
        Terminator, format_function, format_module,
    };
    pub use crate::lexer::Lexer;
    pub use crate::module::{
        Import, Module, ModuleItem, ModulePath, ModuleResolver, Visibility, build_module,
        module_composition,
    };
    pub use crate::optimize::{
        CommonSubexpressionElimination, ConstantFolding, CopyPropagation, DeadCodeElimination,
        OptLevel, OptimizationPass, Optimizer, UnreachableBlockElimination,
    };
    pub use crate::parser::Parser;
    pub use crate::stdlib::{Stdlib, StdlibFn, StdlibKind, execute as stdlib_execute};
    pub use crate::token::{FILE_EXTENSION, FILE_EXTENSION_FALLBACK, Span, Token, TokenKind};
    pub use crate::types::{PrimaType, TypeEnv};
    pub use crate::value::{Value, ValueData};
    pub use crate::vm::VM;
}

pub use analyze::Analyzer;
pub use error::{PrimaError, PrimaResult};
pub use interpret::Interpreter;
pub use lexer::Lexer;
pub use nottrue::FILE_EXTENSION_NOT;
pub use parser::Parser;
pub use token::{FILE_EXTENSION, FILE_EXTENSION_FALLBACK};
pub use value::Value;

/// Evaluate Prima source code (tree-walking interpreter).
pub fn eval(source: &str) -> PrimaResult<Value> {
    let tokens = Lexer::new(source).tokenize()?;
    let program = Parser::new(tokens).parse()?;
    Interpreter::new().eval_program(&program)
}

/// Compile and run Prima source code (bytecode VM).
///
/// This is the faster execution path using the bytecode compiler and VM.
pub fn compile_and_run(source: &str) -> PrimaResult<Value> {
    let tokens = Lexer::new(source).tokenize()?;
    let program = Parser::new(tokens).parse()?;
    let module = codegen::Compiler::new().compile(&program)?;
    vm::VM::new().run(&module)
}

/// Parse Prima source code to AST.
pub fn parse(source: &str) -> PrimaResult<ast::Program> {
    let tokens = Lexer::new(source).tokenize()?;
    Parser::new(tokens).parse()
}

/// Tokenize Prima source code.
pub fn tokenize(source: &str) -> PrimaResult<Vec<token::Token>> {
    Lexer::new(source).tokenize()
}

/// Analyze Prima source code for type safety.
///
/// Returns the analysis context with inferred types and any errors/warnings.
pub fn analyze_program(source: &str) -> PrimaResult<analyze::AnalysisContext> {
    let tokens = Lexer::new(source).tokenize()?;
    let program = Parser::new(tokens).parse()?;
    let mut analyzer = Analyzer::new();
    analyzer.analyze(&program)?;
    Ok(analyzer.into_context())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_root_constants() {
        assert!(eval("0").unwrap().is_zero());
        assert!(eval("1").unwrap().is_one());
    }

    #[test]
    fn test_eval() {
        assert_eq!(eval("1 + 2").unwrap(), Value::int(3));
    }

    #[test]
    fn test_function() {
        let r = eval("fn f(x: N) → N { x * 2 }\nf(21)").unwrap();
        assert_eq!(r, Value::int(42));
    }

    #[test]
    fn test_file_extension() {
        assert_eq!(FILE_EXTENSION, "true");
        assert_eq!(FILE_EXTENSION_FALLBACK, "prima");
    }

    // ─────────────────────────────────────────────────────────────────────────
    // compile_and_run Integration Tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_compile_and_run_compiles() {
        // Just verify compilation succeeds
        let result = compile_and_run("42");
        assert!(result.is_ok());
    }

    #[test]
    fn test_compile_and_run_function_def() {
        // Function definition compiles
        let result = compile_and_run("fn answer() → N { 42 }");
        assert!(result.is_ok());
    }

    #[test]
    fn test_compile_and_run_sequence() {
        let result = compile_and_run("σ[1, 2, 3]");
        assert!(result.is_ok());
    }

    #[test]
    fn test_compile_and_run_if_expr() {
        let result = compile_and_run("if true { 1 } else { 2 }");
        println!("RESULT: {:?}", result);
        assert!(result.is_ok(), "Expected Ok, got: {:?}", result);
    }

    #[test]
    fn test_compile_and_run_for_loop() {
        let result = compile_and_run("for i in σ[1, 2, 3] { i }");
        println!("FOR LOOP RESULT: {:?}", result);
        assert!(result.is_ok(), "Expected Ok, got: {:?}", result);
    }
}
