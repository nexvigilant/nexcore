// Copyright © 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! # Academy Forge
//!
//! Extract structured knowledge from nexcore Rust source into a universal
//! Intermediate Representation (IR) that feeds both Observatory (3D spatial
//! rendering) and Academy (experiential learning pathways).
//!
//! ## MCP Tools
//!
//! | Tool | Input | Output |
//! |------|-------|--------|
//! | `forge_extract` | `crate_name, domain?` | `CrateAnalysis` JSON |
//! | `forge_validate` | `content` (JSON) | `ValidationReport` JSON |
//! | `forge_schema` | (none) | StaticPathway JSON Schema |
//! | `forge_compile` | `input_path, output_dir` | TypeScript files |
//!
//! ## Workflow
//!
//! 1. `forge_extract(crate="nexcore-tov", domain="vigilance")` → IR
//! 2. Claude writes academy content using IR + educational theory
//! 3. `forge_validate(content)` → pass/fail + specific errors
//! 4. Fix errors, re-validate until clean
//! 5. `forge_compile(input, output_dir)` → Studio TypeScript files

#![forbid(unsafe_code)]
#![cfg_attr(
    not(test),
    deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)
)]

pub mod atomize;
pub mod compile;
pub mod domain;
pub mod error;
pub mod extract;
pub mod graph;
pub mod guidance;
pub mod ir;
pub mod scaffold;
pub mod validate;

// Re-exports
pub use atomize::atomize;
pub use compile::{CompileParams, CompileResult, compile as compile_pathway};
pub use error::{ForgeError, ForgeResult};
pub use extract::extract_crate;
pub use graph::build_graph;
pub use guidance::{GuidanceInput, GuidanceScaffoldParams, generate as guidance_scaffold};
pub use ir::{
    AloEdge, AloEdgeType, AloType, AtomicLearningObject, AtomizedPathway, BloomLevel,
    CrateAnalysis, DomainAnalysis, GraphTopology, LearningGraph,
};
pub use scaffold::{ScaffoldParams, generate as scaffold};
pub use validate::{ValidationReport, validate};
