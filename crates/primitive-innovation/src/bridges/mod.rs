// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! # P3-P5: Cross-Domain Bridges
//!
//! **Problem**: 6 high-confidence bridge types identified, zero built.
//! These connect currently-independent crate pairs through shared T1 primitives.
//!
//! ## Bridge Types
//!
//! | Bridge | Pair | Confidence | Impact |
//! |--------|------|------------|--------|
//! | `NeuroendocrineCoordinator` | cytokine + hormones | 0.89 | HIGH |
//! | `EnergeticTransition` | energy + state-os | 0.87 | HIGH |
//! | `SchemaImmuneSystem` | immunity + ribosome | 0.85 | HIGH |

mod energetic_transition;
mod neuroendocrine;
mod schema_immune;

pub use energetic_transition::EnergeticTransition;
pub use neuroendocrine::NeuroendocrineCoordinator;
pub use schema_immune::SchemaImmuneSystem;
