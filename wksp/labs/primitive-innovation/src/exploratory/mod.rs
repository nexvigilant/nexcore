// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! # P10-P12: Exploratory Innovation Types
//!
//! Advanced cross-domain bridges requiring higher primitive counts.
//!
//! | Type | Tier | Bridge | Dominant | Confidence |
//! |------|------|--------|----------|------------|
//! | `QuantumStateSpace` | T3 | quantum × stos | ς State | 0.78 |
//! | `CloudResourceGraph` | T2-C | aggregate × cloud | Σ Sum | 0.83 |
//! | `SchemaGuidedSplitter` | T2-C | transcriptase × dtree | κ Comparison | 0.82 |

mod cloud_resource;
mod quantum_state;
mod schema_splitter;

pub use cloud_resource::CloudResourceGraph;
pub use quantum_state::QuantumStateSpace;
pub use schema_splitter::SchemaGuidedSplitter;
