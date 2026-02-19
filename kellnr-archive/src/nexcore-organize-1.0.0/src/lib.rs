//! # NexVigilant Core — organize
//!
//! **ORGANIZE** — 8-step file organization pipeline grounded to T1 primitives.
//!
//! Each step maps to a distinct T1 primitive from the Lex Primitiva:
//!
//! | Step | Letter | Primitive | Symbol | Module | Purpose |
//! |------|--------|-----------|--------|--------|---------|
//! | 1 | **O**bserve | Existence | ∃ | [`observe`] | Recursive inventory with metadata |
//! | 2 | **R**ank | Comparison | κ | [`rank`] | Score/prioritize by recency, size, relevance |
//! | 3 | **G**roup | Mapping | μ | [`group`] | Cluster into categories by rules |
//! | 4 | **A**ssign | Causality | → | [`assign`] | Map groups to actions (Move/Archive/Delete/Keep/Review) |
//! | 5 | **N**ame | Boundary | ∂ | [`name`] | Naming conventions, conflict detection |
//! | 6 | **I**ntegrate | Sum | Σ | [`integrate`] | Execute plan (or dry-run report) |
//! | 7 | **Z**ero-out | Void | ∅ | [`zero_out`] | Remove empties, detect duplicates |
//! | 8 | **E**nforce | State | ς | [`enforce`] | State snapshot for drift detection |
//!
//! ## Tier: T3 (8 unique primitives: ∃ κ μ → ∂ Σ ∅ ς)
//!
//! ## Design Principles
//!
//! - **Forward-only data flow** — each step takes the previous step's output
//! - **Dry-run by default** — no mutations without explicit opt-in
//! - **No async** — pure synchronous I/O for simplicity
//! - **Content hashing** — SHA-256 for duplicate detection
//!
//! ## Example
//!
//! ```no_run
//! use nexcore_organize::config::OrganizeConfig;
//! use nexcore_organize::pipeline::OrganizePipeline;
//! use nexcore_organize::report;
//!
//! let config = OrganizeConfig::default_for("/tmp/messy");
//! let pipeline = OrganizePipeline::new(config);
//! let result = pipeline.run().unwrap();
//! println!("{}", report::markdown_report(&result));
//! ```

#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

pub mod assign;
pub mod config;
pub mod enforce;
pub mod error;
pub mod grounding;
pub mod group;
pub mod integrate;
pub mod name;
pub mod observe;
pub mod pipeline;
pub mod rank;
pub mod report;
pub mod zero_out;

/// Prelude for convenient imports.
pub mod prelude {
    pub use crate::config::{FileOp, OrganizeConfig};
    pub use crate::error::{OrganizeError, OrganizeResult};
    pub use crate::pipeline::{OrganizePipeline, OrganizeResult2};
    pub use crate::report::{format_bytes, json_report, markdown_report};
}
