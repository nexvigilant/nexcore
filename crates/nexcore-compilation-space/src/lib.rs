// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! # The Full Compilation Space
//!
//! A 7-axis framework for modeling ALL code transformations as movements
//! through a multidimensional space.
//!
//! ## The Insight
//!
//! Compilation isn't a line — it's a 7-dimensional space:
//!
//! | Axis | Dimension | T1 Primitive | Example |
//! |------|-----------|--------------|---------|
//! | **Abstraction** | Vertical | σ Sequence | Source → Token → AST → IR → Binary |
//! | **Language** | Lateral | μ Mapping | Rust → C → WASM → JavaScript |
//! | **Time** | Temporal | ν Frequency | v1 → diff → patch → v2 |
//! | **Evaluation** | Resolution | ∂ Boundary | Symbolic → Partial → Concrete |
//! | **Reflection** | Self-reference | ρ Recursion | Code → Code-about-code |
//! | **Projection** | Dimensionality | Σ→σ Collapse | Graph → Tree → Linear → Scalar |
//! | **Branching** | Superposition | Σ Sum | cfg!, features, conditional compilation |
//!
//! Every tool a programmer uses — lexer, parser, transpiler, linter,
//! AI assistant, debugger, profiler — is a **transform** that moves
//! artifacts between points in this space.
//!
//! ## Core Types
//!
//! - [`CompilationPoint`] — a position in the 7D space
//! - [`Transform`] — a movement between two points
//! - [`TransformChain`] — a sequence of transforms (a pipeline)
//! - [`Axis`] — the 7 dimensions
//!
//! ## Catalog
//!
//! The [`catalog`] module provides factory functions for well-known transforms:
//! `lex()`, `parse()`, `transpile()`, `intent_compile()`, `refactor()`, etc.
//!
//! ## Example
//!
//! ```rust
//! use nexcore_compilation_space::catalog;
//! use nexcore_compilation_space::axis::LanguageId;
//!
//! // Build a standard Rust compilation pipeline
//! let chain = catalog::compile_chain(LanguageId::rust());
//! assert_eq!(chain.len(), 4); // lex → parse → lower → codegen
//! assert!(chain.validate().is_empty()); // All steps connect
//!
//! // AI-first: intent → Prima → PVDSL → execution
//! let ai_chain = catalog::nexcore_ai_pipeline();
//! assert_eq!(ai_chain.len(), 3);
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![cfg_attr(
    not(test),
    deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)
)]

pub mod axis;
pub mod catalog;
pub mod point;
pub mod spatial_bridge;
pub mod transform;

// Re-export core types at crate root for convenience.
pub use axis::{
    AbstractionLevel, Axis, BranchConfig, Dimensionality, Direction, EvalState, LanguageId,
    ReflectionDepth, TemporalCoord,
};
pub use point::CompilationPoint;
pub use transform::{ChainError, Transform, TransformChain};
