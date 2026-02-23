// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! # Primitive Foundation
//!
//! Re-exports the T1 Lex Primitiva primitives most relevant to `nexcore-shell`.
//!
//! ## Dominant Primitives for the Shell
//!
//! | Component | Dominant Primitive | Rationale |
//! |-----------|--------------------|-----------|
//! | Shell     | μ (Mapping)        | Input mapping + boot sequence |
//! | Layout    | ∂ (Boundary)       | Bounded screen regions |
//! | App       | ∃ (Existence)      | App registration and lifecycle |
//! | Lock      | ∂ (Boundary)       | Security boundary + state |
//! | Notifications | σ (Sequence)   | Priority-ordered message queue |
//! | Launcher  | Σ (Sum)            | Collection of launchable apps |
//! | AI Partner | μ (Mapping)       | Intent → action mapping |

pub use nexcore_lex_primitiva::grounding::GroundsTo;
pub use nexcore_lex_primitiva::primitiva::LexPrimitiva;
pub use nexcore_lex_primitiva::primitiva::PrimitiveComposition;
pub use nexcore_lex_primitiva::tier::Tier;
