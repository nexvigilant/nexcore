//! # NexVigilant Core — word
//!
//! Foundation binary word algorithms: typed newtypes (`Word8`–`Word64`) with
//! composable trait algebra grounded to State (ς).
//!
//! Raw `u8`/`u16`/`u32`/`u64` ground to Quantity (N) — they represent magnitudes.
//! When you manipulate individual bits, count populations, extract fields, or
//! rotate registers, you're performing **state transformations on state registers**.
//! The bit IS the irreducible unit of State (ς).
//!
//! ## Quick Start
//!
//! ```rust
//! use nexcore_word::prelude::*;
//!
//! let w = Word8::new(0b1010_1100);
//! assert_eq!(w.popcount().value(), 4);
//! assert_eq!(w.parity(), Parity::Even);
//! assert_eq!(w.leading_zeros().value(), 0);
//! assert_eq!(w.trailing_zeros().value(), 2);
//! ```
//!
//! ## Trait Algebra
//!
//! | Trait | Operations |
//! |-------|-----------|
//! | `PopulationOps` | popcount, zero_count, parity, hamming_distance |
//! | `PositionOps` | leading/trailing zeros/ones, highest/lowest set bit |
//! | `ManipulationOps` | set/clear/toggle/test bit, extract/insert field |
//! | `ArithmeticOps` | is_power_of_two, log2, isqrt, binary_gcd |
//! | `RotationOps` | rotate_left, rotate_right, barrel_shift |
//! | `MaskOps` | low/high/range masks, isolate, smear |
//!
//! `WordOps` = all six combined via blanket impl.

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![cfg_attr(
    not(test),
    deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)
)]

pub mod arithmetic;
pub mod error;
pub mod grounding;
pub mod manipulation;
pub mod mask;
pub mod population;
pub mod position;
pub mod properties;
pub mod rotation;
pub mod width;

use arithmetic::ArithmeticOps;
use manipulation::ManipulationOps;
use mask::MaskOps;
use population::PopulationOps;
use position::PositionOps;
use rotation::RotationOps;

/// Supertrait combining all six word operation traits.
///
/// Automatically implemented for any type that implements all sub-traits.
pub trait WordOps:
    PopulationOps + PositionOps + ManipulationOps + ArithmeticOps + RotationOps + MaskOps
{
}

/// Blanket implementation: anything implementing all six traits gets `WordOps`.
impl<T> WordOps for T where
    T: PopulationOps + PositionOps + ManipulationOps + ArithmeticOps + RotationOps + MaskOps
{
}

/// Convenience prelude for common imports.
pub mod prelude {
    pub use crate::WordOps;
    pub use crate::arithmetic::ArithmeticOps;
    pub use crate::error::WordError;
    pub use crate::manipulation::ManipulationOps;
    pub use crate::mask::MaskOps;
    pub use crate::population::PopulationOps;
    pub use crate::position::PositionOps;
    pub use crate::properties::{Alignment, BitCount, BitPosition, Parity};
    pub use crate::rotation::RotationOps;
    pub use crate::width::{Word, Word8, Word16, Word32, Word64, WordWidth};
}

#[cfg(test)]
mod tests {
    use super::prelude::*;

    #[test]
    fn word_ops_supertrait_compiles() {
        // Verify WordOps is auto-implemented for Word<W>
        fn assert_word_ops<T: WordOps>(_: &T) {}
        assert_word_ops(&Word8::new(0));
        assert_word_ops(&Word16::new(0));
        assert_word_ops(&Word32::new(0));
        assert_word_ops(&Word64::new(0));
    }

    #[test]
    fn prelude_integration() {
        let w = Word8::new(0b1010_1100);
        // PopulationOps
        assert_eq!(w.popcount().value(), 4);
        assert_eq!(w.parity(), Parity::Even);
        // PositionOps
        assert_eq!(w.leading_zeros().value(), 0);
        assert_eq!(w.trailing_zeros().value(), 2);
        // ManipulationOps
        let toggled = w.toggle_bit(0);
        assert!(toggled.is_ok());
        // ArithmeticOps
        assert!(!w.is_power_of_two());
        // RotationOps
        let rotated = w.rotate_left(4);
        assert_eq!(rotated.popcount(), w.popcount());
        // MaskOps
        let isolated = w.isolate_lowest();
        assert_eq!(isolated.raw(), 0b0000_0100);
    }

    #[test]
    fn doc_example_works() {
        let w = Word8::new(0b1010_1100);
        assert_eq!(w.popcount().value(), 4);
        assert_eq!(w.parity(), Parity::Even);
        assert_eq!(w.leading_zeros().value(), 0);
        assert_eq!(w.trailing_zeros().value(), 2);
    }
}
