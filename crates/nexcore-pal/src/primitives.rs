// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! # Primitive Foundation
//!
//! Re-exports the T1 Lex Primitiva primitives most relevant to `nexcore-pal`.
//!
//! ## Dominant Primitives for PAL
//!
//! | Subsystem | Dominant Primitive | Rationale |
//! |-----------|-------------------|-----------|
//! | Display   | μ (Mapping)       | Maps pixel data to physical display |
//! | Input     | σ (Sequence)      | Ordered stream of input events |
//! | Storage   | π (Persistence)   | Persistent key-value data store |
//! | Power     | ς (State)         | Battery/charging state machine |
//! | Haptics   | ν (Frequency)     | Timed vibration pulses |
//! | Platform  | Σ (Sum)           | Composition of all subsystems |
//!
//! This module is only available when the `grounding` feature is enabled.

#[cfg(feature = "grounding")]
pub use nexcore_lex_primitiva::grounding::GroundsTo;
#[cfg(feature = "grounding")]
pub use nexcore_lex_primitiva::primitiva::LexPrimitiva;
#[cfg(feature = "grounding")]
pub use nexcore_lex_primitiva::primitiva::PrimitiveComposition;
#[cfg(feature = "grounding")]
pub use nexcore_lex_primitiva::tier::Tier;
