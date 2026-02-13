//! # Rotation Operations
//!
//! Circular bit rotation with modular semantics.
//!
//! ## Tier: T2-P (ς State + σ Sequence)
//! ## Dominant: ς (State) — rotation is state transformation preserving population

use crate::width::{Word, WordWidth};

/// Operations for circular bit rotation.
///
/// Blanket-implemented for all `Word<W>`.
pub trait RotationOps: Sized {
    /// Rotate left by `n` bits (circular, modular).
    fn rotate_left(&self, n: u32) -> Self;

    /// Rotate right by `n` bits (circular, modular).
    fn rotate_right(&self, n: u32) -> Self;

    /// Barrel shift: rotate left if `n > 0`, right if `n < 0`.
    fn barrel_shift(&self, n: i32) -> Self;
}

impl<W: WordWidth> RotationOps for Word<W> {
    #[inline]
    fn rotate_left(&self, n: u32) -> Self {
        Word(self.0.rotate_left_raw(n % W::BITS))
    }

    #[inline]
    fn rotate_right(&self, n: u32) -> Self {
        Word(self.0.rotate_right_raw(n % W::BITS))
    }

    #[inline]
    fn barrel_shift(&self, n: i32) -> Self {
        if n >= 0 {
            self.rotate_left(n as u32)
        } else {
            self.rotate_right(n.unsigned_abs())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::width::{Word8, Word16, Word32, Word64};

    // ── rotate_left ───────────────────────────────────────────────────

    #[test]
    fn rotate_left_basic() {
        assert_eq!(Word8::new(0b1000_0001).rotate_left(1).raw(), 0b0000_0011);
    }

    #[test]
    fn rotate_left_full_width() {
        let w = Word8::new(0xAB);
        assert_eq!(w.rotate_left(8).raw(), 0xAB); // Full rotation = identity
    }

    #[test]
    fn rotate_left_zero() {
        let w = Word32::new(0xDEAD_BEEF);
        assert_eq!(w.rotate_left(0).raw(), 0xDEAD_BEEF);
    }

    #[test]
    fn rotate_left_preserves_popcount() {
        use crate::population::PopulationOps;
        let w = Word32::new(0xDEAD_BEEF);
        let rotated = w.rotate_left(13);
        assert_eq!(w.popcount(), rotated.popcount());
    }

    // ── rotate_right ──────────────────────────────────────────────────

    #[test]
    fn rotate_right_basic() {
        assert_eq!(Word8::new(0b0000_0011).rotate_right(1).raw(), 0b1000_0001);
    }

    #[test]
    fn rotate_right_full_width() {
        let w = Word16::new(0xABCD);
        assert_eq!(w.rotate_right(16).raw(), 0xABCD);
    }

    // ── rotate inverse ────────────────────────────────────────────────

    #[test]
    fn rotate_left_right_inverse() {
        let w = Word32::new(0xDEAD_BEEF);
        for n in 0..32 {
            assert_eq!(w.rotate_left(n).rotate_right(n).raw(), w.raw());
        }
    }

    #[test]
    fn rotate_right_left_inverse() {
        let w = Word64::new(0xCAFE_BABE_DEAD_BEEF);
        for n in 0..64 {
            assert_eq!(w.rotate_right(n).rotate_left(n).raw(), w.raw());
        }
    }

    // ── barrel_shift ──────────────────────────────────────────────────

    #[test]
    fn barrel_shift_positive() {
        let w = Word8::new(0b0000_0001);
        assert_eq!(w.barrel_shift(1).raw(), 0b0000_0010);
    }

    #[test]
    fn barrel_shift_negative() {
        let w = Word8::new(0b0000_0010);
        assert_eq!(w.barrel_shift(-1).raw(), 0b0000_0001);
    }

    #[test]
    fn barrel_shift_zero() {
        let w = Word32::new(0xABCD);
        assert_eq!(w.barrel_shift(0).raw(), 0xABCD);
    }

    #[test]
    fn barrel_shift_inverse() {
        let w = Word16::new(0x1234);
        assert_eq!(w.barrel_shift(5).barrel_shift(-5).raw(), w.raw());
    }

    // ── modular overflow ──────────────────────────────────────────────

    #[test]
    fn rotate_modular_overflow() {
        let w = Word8::new(0b1010_0101);
        // Rotating by 9 == rotating by 1 for 8-bit word
        assert_eq!(w.rotate_left(9).raw(), w.rotate_left(1).raw());
        assert_eq!(w.rotate_right(9).raw(), w.rotate_right(1).raw());
    }

    #[test]
    fn rotate_modular_large_n() {
        let w = Word32::new(0xDEAD);
        assert_eq!(w.rotate_left(32 * 100 + 5).raw(), w.rotate_left(5).raw());
    }
}
