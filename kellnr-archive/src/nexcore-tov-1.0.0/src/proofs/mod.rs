//! # Logic Proofs Library
//!
//! Curry-Howard correspondence for automated theorem verification through the type system.
//! If a Rust function type-checks, the corresponding logical theorem is proven valid.

#![allow(dead_code)]
#![allow(unused_variables)]

pub mod attenuation;
pub mod codex_compliance;
pub mod inference_rules;
pub mod logic_prelude;
pub mod proof_patterns;
pub mod theorems;
pub mod type_level;

#[cfg(feature = "kani")]
pub mod kani_proofs;

/// Prelude module for commonly used proof types.
pub mod prelude {
    pub use crate::proofs::codex_compliance::{
        CompareAbsent, Confident, ConfidentProof, GroundsTo, HasTier, IsPrimitive, Tier, Versioned,
    };
    pub use crate::proofs::inference_rules::*;
    pub use crate::proofs::logic_prelude::*;
    pub use crate::proofs::proof_patterns::*;
}
