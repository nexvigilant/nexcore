// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! # Primitive Foundation
//!
//! Re-exports the T1 Lex Primitiva primitives most relevant to `nexcore-compositor`.
//!
//! ## Dominant Primitives for the Compositor
//!
//! | Component | Dominant Primitive | Rationale |
//! |-----------|--------------------|-----------|
//! | Surface   | ∂ (Boundary)       | Bounded pixel region on screen |
//! | Z-order   | σ (Sequence)       | Ordered front-to-back surface stacking |
//! | Focus     | ∃ (Existence)      | Active surface state (exists + focused) |
//! | Compositor| Σ (Sum)            | Sum of all surfaces mapped to display |
//! | Tiling    | μ (Mapping)        | Window list → tile position assignment |
//! | Rendering | μ (Mapping)        | Surface framebuffers → display output |
//! | Mode      | κ (Comparison)     | Form factor drives mode selection |

pub use nexcore_lex_primitiva::grounding::GroundsTo;
pub use nexcore_lex_primitiva::primitiva::LexPrimitiva;
pub use nexcore_lex_primitiva::primitiva::PrimitiveComposition;
pub use nexcore_lex_primitiva::tier::Tier;
