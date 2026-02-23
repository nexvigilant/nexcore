//! # nexcore-viz: STEM Visualization Engine
//!
//! SVG-generating visualization tools for the STEM primitive system, plus
//! molecular structure types, file parsers, graph theory algorithms, and
//! a Validated DAG (VDAG) engine for drug-class AE signal overlay.
//!
//! ## Infrastructure
//!
//! | Module | Purpose |
//! |--------|---------|
//! | `svg` | Foundation: shapes, text, arcs, arrows, palette |
//! | `metrics` | Text measurement approximation (char-width tables) |
//! | `scale` | Data-to-pixel scales (linear, log, ordinal) |
//! | `theme` | Theming system (dark, light, high-contrast, print) |
//! | `axis` | Axis rendering with ticks, labels, grid lines, legends |
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
//! ## Molecular & Chemistry
//!
//! | Module | Purpose |
//! |--------|---------|
//! | `molecular` | Atom/Bond types, element data, VdW radii, CPK colors |
//! | `parsers` | SDF/MOL V2000 and PDB file parsers |
//! | `surface` | Marching cubes for molecular surface mesh generation |
//!
//! ## Graph Theory
//!
//! | Module | Purpose |
//! |--------|---------|
//! | `spectral` | Adjacency/Laplacian matrices, power iteration, Fiedler value |
//! | `community` | Louvain community detection algorithm |
//! | `centrality` | Degree, betweenness, closeness, eigenvector centrality |
//!
//! ## VDAG — Validated DAG
//!
//! | Module | Purpose |
//! |--------|---------|
//! | `vdag` | Drug-class taxonomy with AE signal overlay (PRR/ROR/IC/EBGM) |
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
//! metrics.rs       Text measurement (char-width approximation)
//! scale.rs         Data-to-pixel scales (linear, log, ordinal)
//! theme.rs         Theming (dark, light, high-contrast, print)
//! svg.rs           Foundation: shapes, text, arcs, arrows, palette
//!   |
//!   +-- axis.rs          Axis rendering (ticks, labels, grids, legends)
//!   +-- taxonomy.rs      STEM sunburst (domain->trait->grounding)
//!   +-- composition.rs   Type decomposition (center-node -> T1 primitives)
//!   +-- science_loop.rs  Method cycles (Science, Chemistry, Math)
//!   +-- confidence.rs    Confidence waterfall (proof chain)
//!   +-- bounds.rs        Bounded value number line
//!   +-- dag.rs           Topological DAG layout
//!   +-- molecular.rs     Atom/Bond/Molecule types + render helpers
//!   +-- parsers/         SDF/MOL + PDB file parsers
//!   +-- surface.rs       Marching cubes mesh generation
//!   +-- spectral.rs      Spectral graph analysis
//!   +-- community.rs     Louvain community detection
//!   +-- centrality.rs    Graph centrality measures
//!   +-- vdag.rs          Validated DAG with AE signal overlay
//! ```

#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

// Infrastructure (leaf modules first)
pub mod math;
pub mod metrics;
pub mod scale;
pub mod svg;
pub mod theme;

// Infrastructure (depends on metrics + scale + svg + theme)
pub mod axis;

// Diagram modules
pub mod bounds;
pub mod composition;
pub mod confidence;
pub mod dag;
pub mod science_loop;
pub mod taxonomy;

// Molecular & chemistry
pub mod molecular;
pub mod parsers;
pub mod surface;

// Graph theory
pub mod centrality;
pub mod community;
pub mod spectral;

// VDAG — Validated DAG with AE signal overlay
pub mod vdag;

// Phase 4 "Nervous System" — AE signal overlay computation
pub mod ae_overlay;

// Phase 2: Biologics, interactions, higher-dimensional math
pub mod antibody;
pub mod interaction;
pub mod projection;
pub mod protein;
pub mod topology;

// Phase 3: Physics engine, GPU compute, LOD, particle systems
pub mod dynamics;
pub mod force_field;
pub mod gpu_layout;
pub mod hzb;
pub mod hypergraph;
pub mod instancing;
pub mod lod;
pub mod minimizer;
pub mod particle;
pub mod picking;
pub mod sdf_text;
pub mod transfer_lut;

// Phase 4: Nervous system — distance geometry coordinate generation
pub mod coord_gen;

// Phase 4: Nervous system — bipartite drug-AE network layout
pub mod bipartite;

// Phase 5: Eyes — Calabi-Yau manifold renderer for string theory visualization
pub mod manifold;

// Phase 5: Eyes — string vibration mode engine (standing waves, harmonics, superposition)
pub mod string_modes;

// Phase 5: Eyes — WebGPU rendering pipeline (pipeline state, WGSL codegen, TAA, GTAO, SSS)
pub mod clustered_lighting;
pub mod renderer;
pub mod volumetrics;

// Phase 5: Eyes — molecular orbital computation (STO-3G basis, electron density grids)
pub mod orbital;

// Re-export key types for convenience
pub use axis::{Axis, LegendEntry, Orientation, TickFormat};
pub use bounds::{BoundedValue, render_bounds};
pub use composition::{PrimitiveNode, TypeComposition, render_composition};
pub use confidence::{Claim, render_confidence_chain};
pub use dag::{DagEdge, DagNode, render_dag};
pub use metrics::{TextExtent, TextMetrics, measure_text, measure_text_bold};
pub use scale::{LinearScale, LogScale, OrdinalScale, Scale};
pub use science_loop::{LoopStep, render_science_loop};
pub use taxonomy::{TraitEntry, render_taxonomy};
pub use theme::Theme;
