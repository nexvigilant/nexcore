//! NexCore Declension — Latin-inspired architectural primitives
//!
//! Latin partitions its lexicon into declension classes, assigns grammatical
//! cases, inflects verb families, drops inferrable pronouns, and enforces
//! multi-dimensional agreement. We transfer these patterns to software:
//!
//! - **∂ Declension classes** — 5 architectural layers with dependency rules
//! - **ς Component cases** — 7 grammatical roles for crates
//! - **μ Tool inflection** — family-based tool grouping (fewer tools, more modes)
//! - **∅ Pro-drop** — contextual parameter elision
//! - **× Agreement** — multi-dimensional compatibility checking
//!
//! ## Primitive Grounding
//! Dominant primitives per module:
//! - `declension`: ∂ Boundary (layer partitioning)
//! - `case`: ς State (role assignment)
//! - `inflection`: μ Mapping (tool family compression)
//! - `prodrop`: ∅ Void (parameter elision)
//! - `agreement`: × Product (cross-dimensional checks)

#![forbid(unsafe_code)]
#![cfg_attr(
    not(test),
    deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)
)]

pub mod agreement;
pub mod case;
pub mod declension;
pub mod grounding;
pub mod inflection;
pub mod prodrop;

// Re-exports for convenience
pub use agreement::{AgreementDimension, AgreementResult, DimensionCheck};
pub use case::{CasedComponent, ComponentCase};
pub use declension::{Declension, DeclinedCrate};
pub use inflection::{Inflection, InflectionAnalysis, ToolFamily};
pub use prodrop::{ProDropAnalysis, ProDropContext};
