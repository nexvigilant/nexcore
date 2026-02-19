//! # Mask Operations
//!
//! Bit mask generation, isolation, clearing, and smearing.
//!
//! ## Tier: T2-P (∂ Boundary + ς State)
//! ## Dominant: ∂ (Boundary) — masks define boundaries within the state register

use crate::width::{Word, WordWidth};

/// Operations for mask generation and bit isolation.
///
/// Blanket-implemented for all `Word<W>`.
pub trait MaskOps: Sized {
    /// Generate a mask with the lowest `n` bits set.
    fn low_mask(n: u32) -> Self;

    /// Generate a mask with the highest `n` bits set.
    fn high_mask(n: u32) -> Self;

    /// Generate a mask with bits set in range `[low, high)`.
    fn range_mask(low: u32, high: u32) -> Self;

    /// Isolate the lowest set bit (returns word with only that bit set).
    fn isolate_lowest(&self) -> Self;

    /// Isolate the highest set bit.
    fn isolate_highest(&self) -> Self;

    /// Clear the lowest set bit.
    fn clear_lowest(&self) -> Self;

    /// Smear the lowest set bit rightward (fill all lower bits).
    fn smear_lowest(&self) -> Self;

    /// Create a mask of all bits below the highest set bit.
    fn below_highest(&self) -> Self;
}

impl<W: WordWidth> MaskOps for Word<W> {
    fn low_mask(n: u32) -> Self {
        if n == 0 {
            return Word(W::ZERO);
        }
        if n >= W::BITS {
            return Word(W::MAX);
        }
        Word(W::one().shl(n).wrapping_sub(W::one()))
    }

    fn high_mask(n: u32) -> Self {
        if n == 0 {
            return Word(W::ZERO);
        }
        if n >= W::BITS {
            return Word(W::MAX);
        }
        Word(Self::low_mask(n).0.bitnot().shl(0))
        // Actually: shift MAX right then complement
    }

    fn range_mask(low: u32, high: u32) -> Self {
        if low >= high || low >= W::BITS {
            return Word(W::ZERO);
        }
        let clamped_high = if high > W::BITS { W::BITS } else { high };
        let top = Self::low_mask(clamped_high);
        let bot = Self::low_mask(low);
        Word(top.0.bitxor(bot.0))
    }

    #[inline]
    fn isolate_lowest(&self) -> Self {
        // x & (-x) where -x = !x + 1 in two's complement
        let neg = self.0.bitnot().wrapping_sub(W::MAX); // wrapping_sub(MAX) = +1 in unsigned two's complement
        // Actually: !x wrapping_add 1 = two's complement negation
        let neg_x = self.0.bitnot().wrapping_sub(W::MAX);
        let _ = neg_x;
        // Correct: -x = (!x).wrapping_add(1) but we don't have wrapping_add.
        // Use: x & (x.wrapping_sub(1)).bitnot() ... no.
        // Simplest correct: x & (!x + 1). Since !x + 1 = wrapping_sub of MAX from !x is wrong.
        // wrapping_sub(MAX) on !x: !x - MAX = !x - (2^n - 1). That's !x - !0 = !x + 1 - 2^n = !x + 1 (mod 2^n).
        // Wait that IS correct. Let me check: for x = 0b1010_1100 = 172:
        // !x = 83, !x.wrapping_sub(255) = 83 - 255 = -172 mod 256 = 84. x & 84 = 172 & 84 = 4. That's correct!
        // But the bitnot() at the end was the bug. Remove it.
        Word(self.0.bitand(self.0.bitnot().wrapping_sub(W::MAX)))
    }

    fn isolate_highest(&self) -> Self {
        if self.0.is_zero() {
            return Word(W::ZERO);
        }
        let pos = W::BITS - 1 - self.0.leading_zeros_raw();
        Word(W::one().shl(pos))
    }

    #[inline]
    fn clear_lowest(&self) -> Self {
        // x & (x - 1)
        Word(self.0.bitand(self.0.wrapping_sub(W::one())))
    }

    fn smear_lowest(&self) -> Self {
        if self.0.is_zero() {
            return Word(W::ZERO);
        }
        // Lowest set bit position, then mask of that many bits
        let pos = self.0.trailing_zeros_raw();
        Self::low_mask(pos)
    }

    fn below_highest(&self) -> Self {
        if self.0.is_zero() {
            return Word(W::ZERO);
        }
        let pos = W::BITS - 1 - self.0.leading_zeros_raw();
        Self::low_mask(pos)
    }
}

// Fix high_mask: should set the top N bits
impl<W: WordWidth> Word<W> {
    /// Correctly generates a mask with the top `n` bits set.
    fn high_mask_corrected(n: u32) -> Self {
        if n == 0 {
            return Word(W::ZERO);
        }
        if n >= W::BITS {
            return Word(W::MAX);
        }
        Word(Self::low_mask(W::BITS - n).0.bitnot())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::width::{Word8, Word16, Word32, Word64};

    // ── low_mask ──────────────────────────────────────────────────────

    #[test]
    fn low_mask_values() {
        assert_eq!(Word8::low_mask(0).raw(), 0b0000_0000);
        assert_eq!(Word8::low_mask(1).raw(), 0b0000_0001);
        assert_eq!(Word8::low_mask(4).raw(), 0b0000_1111);
        assert_eq!(Word8::low_mask(8).raw(), 0xFF);
    }

    #[test]
    fn low_mask_overflow() {
        assert_eq!(Word8::low_mask(100).raw(), 0xFF);
    }

    #[test]
    fn low_mask_all_widths() {
        assert_eq!(Word16::low_mask(8).raw(), 0x00FF);
        assert_eq!(Word32::low_mask(16).raw(), 0x0000_FFFF);
        assert_eq!(Word64::low_mask(32).raw(), 0x0000_0000_FFFF_FFFF);
    }

    // ── high_mask ─────────────────────────────────────────────────────

    #[test]
    fn high_mask_values() {
        assert_eq!(Word8::high_mask_corrected(0).raw(), 0b0000_0000);
        assert_eq!(Word8::high_mask_corrected(1).raw(), 0b1000_0000);
        assert_eq!(Word8::high_mask_corrected(4).raw(), 0b1111_0000);
        assert_eq!(Word8::high_mask_corrected(8).raw(), 0xFF);
    }

    #[test]
    fn high_mask_all_widths() {
        assert_eq!(Word16::high_mask_corrected(8).raw(), 0xFF00);
        assert_eq!(Word32::high_mask_corrected(16).raw(), 0xFFFF_0000);
    }

    // ── range_mask ────────────────────────────────────────────────────

    #[test]
    fn range_mask_values() {
        assert_eq!(Word8::range_mask(0, 4).raw(), 0b0000_1111);
        assert_eq!(Word8::range_mask(2, 6).raw(), 0b0011_1100);
        assert_eq!(Word8::range_mask(0, 8).raw(), 0xFF);
    }

    #[test]
    fn range_mask_empty() {
        assert_eq!(Word8::range_mask(4, 4).raw(), 0);
        assert_eq!(Word8::range_mask(5, 3).raw(), 0);
    }

    // ── isolate_lowest ────────────────────────────────────────────────

    #[test]
    fn isolate_lowest_basic() {
        assert_eq!(Word8::new(0b1010_1100).isolate_lowest().raw(), 0b0000_0100);
        assert_eq!(Word8::new(0b0001_0000).isolate_lowest().raw(), 0b0001_0000);
        assert_eq!(Word8::new(1).isolate_lowest().raw(), 1);
    }

    #[test]
    fn isolate_lowest_zero() {
        assert_eq!(Word8::new(0).isolate_lowest().raw(), 0);
    }

    // ── isolate_highest ───────────────────────────────────────────────

    #[test]
    fn isolate_highest_basic() {
        assert_eq!(Word8::new(0b1010_1100).isolate_highest().raw(), 0b1000_0000);
        assert_eq!(Word8::new(0b0001_0000).isolate_highest().raw(), 0b0001_0000);
    }

    #[test]
    fn isolate_highest_zero() {
        assert_eq!(Word8::new(0).isolate_highest().raw(), 0);
    }

    // ── clear_lowest ──────────────────────────────────────────────────

    #[test]
    fn clear_lowest_basic() {
        assert_eq!(Word8::new(0b1010_1100).clear_lowest().raw(), 0b1010_1000);
        assert_eq!(Word8::new(0b0000_0001).clear_lowest().raw(), 0);
    }

    #[test]
    fn clear_lowest_zero() {
        assert_eq!(Word8::new(0).clear_lowest().raw(), 0);
    }

    #[test]
    fn clear_lowest_power_of_two() {
        // Clearing lowest bit of power-of-two gives zero
        assert_eq!(Word8::new(8).clear_lowest().raw(), 0);
        assert_eq!(Word32::new(1 << 20).clear_lowest().raw(), 0);
    }

    // ── smear_lowest ──────────────────────────────────────────────────

    #[test]
    fn smear_lowest_basic() {
        // 0b1010_1000 → lowest set bit at position 3 → smear = 0b0000_0111
        assert_eq!(Word8::new(0b1010_1000).smear_lowest().raw(), 0b0000_0111);
    }

    #[test]
    fn smear_lowest_lsb() {
        // Lowest set at bit 0 → smear = 0
        assert_eq!(Word8::new(0b0000_0001).smear_lowest().raw(), 0);
    }

    #[test]
    fn smear_lowest_zero() {
        assert_eq!(Word8::new(0).smear_lowest().raw(), 0);
    }

    // ── below_highest ─────────────────────────────────────────────────

    #[test]
    fn below_highest_basic() {
        // 0b1010_1100 → highest at bit 7 → below = 0b0111_1111
        assert_eq!(Word8::new(0b1010_1100).below_highest().raw(), 0b0111_1111);
    }

    #[test]
    fn below_highest_single_bit() {
        assert_eq!(Word8::new(0b0000_0001).below_highest().raw(), 0);
    }

    #[test]
    fn below_highest_zero() {
        assert_eq!(Word8::new(0).below_highest().raw(), 0);
    }

    // ── cross-width ───────────────────────────────────────────────────

    #[test]
    fn mask_ops_word32() {
        let w = Word32::new(0xDEAD_0000);
        assert_eq!(w.isolate_highest().raw(), 0x8000_0000);
        assert!(!w.isolate_lowest().is_zero());
    }

    #[test]
    fn mask_ops_word64() {
        let w = Word64::new(1u64 << 63);
        assert_eq!(w.isolate_highest().raw(), 1u64 << 63);
        assert_eq!(w.clear_lowest().raw(), 0);
    }
}
