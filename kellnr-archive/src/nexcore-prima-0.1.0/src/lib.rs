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
//! `.σ` (sigma) or `.prima`
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
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![warn(missing_docs)]

pub mod ast;
pub mod builtins;
pub mod error;
pub mod grounding;
pub mod interpret;
pub mod lexer;
pub mod parser;
pub mod repl;
pub mod token;
pub mod types;
pub mod value;

/// Prelude for convenient imports.
pub mod prelude {
    pub use crate::ast::{Block, Expr, Literal, Program, Stmt};
    pub use crate::error::{PrimaError, PrimaResult};
    pub use crate::interpret::Interpreter;
    pub use crate::lexer::Lexer;
    pub use crate::parser::Parser;
    pub use crate::token::{FILE_EXTENSION, FILE_EXTENSION_ASCII, Span, Token, TokenKind};
    pub use crate::types::{PrimaType, TypeEnv};
    pub use crate::value::{Value, ValueData};
}

pub use error::{PrimaError, PrimaResult};
pub use interpret::Interpreter;
pub use lexer::Lexer;
pub use parser::Parser;
pub use token::{FILE_EXTENSION, FILE_EXTENSION_ASCII};
pub use value::Value;

/// Evaluate Prima source code.
pub fn eval(source: &str) -> PrimaResult<Value> {
    let tokens = Lexer::new(source).tokenize()?;
    let program = Parser::new(tokens).parse()?;
    Interpreter::new().eval_program(&program)
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
        assert_eq!(FILE_EXTENSION, "σ");
    }
}
