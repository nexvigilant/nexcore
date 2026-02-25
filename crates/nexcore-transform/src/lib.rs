//! # NexVigilant Core — transform
//!
//! Deterministic text transformation engine for cross-domain rewriting.
//!
//! ## Overview
//!
//! This crate provides the computational backbone for transforming text
//! from one domain to another while preserving structural fidelity.
//! The pipeline:
//!
//! 1. **Segment** — split source text into indexed paragraphs
//! 2. **Annotate** — identify domain concepts per paragraph
//! 3. **Map** — build a concept mapping table (source → target)
//! 4. **Compile** — generate per-paragraph rewrite instructions
//! 5. **Score** — measure fidelity of the transformation output
//!
//! ## Primitive Foundation
//!
//! | Type | Tier | Dominant |
//! |------|------|----------|
//! | `Paragraph` | T2-P | σ (Sequence) |
//! | `SourceText` | T2-C | σ (Sequence) |
//! | `DomainProfile` | T2-C | μ (Mapping) |
//! | `ConceptAnnotation` | T2-C | μ (Mapping) |
//! | `ConceptMapping` | T2-C | μ (Mapping) |
//! | `MappingTable` | T2-C | σ (Sequence) |
//! | `TransformationPlan` | T3 | σ (Sequence) |
//! | `FidelityReport` | T2-C | κ (Comparison) |
//!
//! ## Example
//!
//! ```rust
//! use nexcore_transform::prelude::*;
//!
//! let registry = DomainProfileRegistry::new();
//! let pv = registry.get("pharmacovigilance").unwrap();
//! let plan = compile_plan(
//!     "Federalist No. 1",
//!     "The citizen must exercise vigilance against danger.",
//!     "political-philosophy",
//!     pv,
//! );
//! assert!(!plan.instructions.is_empty());
//! ```

#![forbid(unsafe_code)]
#![cfg_attr(not(test), deny(clippy::unwrap_used))]
#![cfg_attr(not(test), deny(clippy::expect_used))]
#![cfg_attr(not(test), deny(clippy::panic))]

pub mod annotation;
pub mod fidelity;
pub mod grounding;
pub mod ledger;
pub mod mapping;
pub mod plan;
pub mod profile;
pub mod render;
pub mod segment;

/// Prelude for convenient imports.
pub mod prelude {
    pub use crate::annotation::{ConceptAnnotation, ConceptOccurrence, annotate};
    pub use crate::fidelity::{FidelityReport, score_fidelity};
    pub use crate::ledger::{LedgerEntry, LedgerSummary, TransferLedger, build_ledger};
    pub use crate::mapping::{ConceptMapping, MappingMethod, MappingTable, build_mapping_table};
    pub use crate::plan::{ParagraphInstruction, TransformationPlan, compile_plan};
    pub use crate::profile::{
        ConceptBridge, DomainProfile, DomainProfileRegistry, RhetoricalRole, bridge_for,
    };
    pub use crate::render::{render_fidelity, render_ledger, render_plan};
    pub use crate::segment::{Paragraph, SourceText, segment};
}

// Re-export key types at crate root for convenience.
pub use plan::compile_plan;
pub use profile::DomainProfileRegistry;
pub use segment::segment;
