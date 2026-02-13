//! High-level language for nexcore-dna.
//!
//! Compiles expression-oriented source text to DNA programs.
//! Designed for ≤1.0 token ratio (AI-efficient syntax).
//!
//! ## Pipeline
//!
//! ```text
//! source → lexer → parser → optimizer → codegen → Program → VM → output
//! ```
//!
//! ## Quick Start
//!
//! ```rust
//! use nexcore_dna::lang::compiler;
//!
//! let result = compiler::eval("2 + 3 * 4").unwrap();
//! assert_eq!(result.output, vec![14]);
//! ```
//!
//! Tier: T3 (→ + μ + σ + ∂ + ρ + ς)

pub mod ast;
pub mod codegen;
pub mod compiler;
pub mod diagnostic;
pub mod json;
pub mod lexer;
pub mod optimizer;
pub mod parser;
pub mod repl;
pub mod templates;
