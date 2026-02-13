//! # Primitive Extractor Skill
//!
//! Extract irreducible conceptual primitives from any domain.

#![forbid(unsafe_code)]

mod extractor;
pub mod grounding;
mod types;

pub use extractor::PrimitiveExtractor;
pub use types::{Primitive, PrimitiveTier};
