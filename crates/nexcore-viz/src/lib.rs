//! # nexcore-viz: STEM Visualization Engine
//!
//! SVG-generating visualization tools for the STEM primitive system.
//!
//! ## Visualizations
//!
//! | Module | Diagram | Shows |
//! |--------|---------|-------|
//! | `taxonomy` | Radial sunburst | 32 STEM traits across 4 domains, T1 groundings |
//! | `composition` | Node-link | How any type decomposes to T1 Lex Primitiva |
//! | `science_loop` | Circular flow | 8-step scientific method with unfixable limits |
//! | `confidence` | Waterfall chart | Confidence propagation: conf(child) <= min(parents) |
//! | `bounds` | Number line | Bounded values, clamping, in/out-of-bounds |
//! | `dag` | Layered DAG | Topological ordering with parallel execution levels |
//!
//! ## Usage
//!
//! Each module provides a `render_*` function returning a self-contained SVG string.
//! These SVGs can be:
//! - Returned directly by MCP tools
//! - Served via REST API
//! - Embedded in HTML/Leptos components
//! - Saved as `.svg` files
//!
//! ## Architecture
//!
//! ```text
//! svg.rs        Foundation: shapes, text, arcs, arrows, colors
//!   |
//!   +-- taxonomy.rs       STEM sunburst (domain->trait->grounding)
//!   +-- composition.rs    Type decomposition (center-node -> T1 primitives)
//!   +-- science_loop.rs   Method cycles (Science, Chemistry, Math)
//!   +-- confidence.rs     Confidence waterfall (proof chain)
//!   +-- bounds.rs         Bounded value number line
//!   +-- dag.rs            Topological DAG layout
//! ```

#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

pub mod bounds;
pub mod composition;
pub mod confidence;
pub mod dag;
pub mod science_loop;
pub mod svg;
pub mod taxonomy;

// Re-export key types for convenience
pub use bounds::{render_bounds, BoundedValue};
pub use composition::{render_composition, PrimitiveNode, TypeComposition};
pub use confidence::{render_confidence_chain, Claim};
pub use dag::{render_dag, DagEdge, DagNode};
pub use science_loop::{render_science_loop, LoopStep};
pub use taxonomy::{render_taxonomy, TraitEntry};
