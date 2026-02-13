//! # Position Operations
//!
//! Bit scanning and positional queries.
//!
//! ## Tier: T2-P (σ Sequence + λ Location)
//! ## Dominant: σ (Sequence) — positions are ordinal within the bit sequence

use crate::properties::{BitCount, BitPosition};
use crate::width::{Word, WordWidth};

/// Operations that locate specific bits within a word.
///
/// Blanket-implemented for all `Word<W>`.
pub trait PositionOps {
    /// Count of leading zeros (from MSB).
    fn leading_zeros(&self) -> BitCount;

    /// Count of trailing zeros (from LSB).
    fn trailing_zeros(&self) -> BitCount;

    /// Count of leading ones (from MSB).
    fn leading_ones(&self) -> BitCount;

    /// Count of trailing ones (from LSB).
    fn trailing_ones(&self) -> BitCount;

    /// Position of the highest set bit (MSB-side), or None if zero.
    fn highest_set_bit(&self) -> Option<BitPosition>;

    /// Position of the lowest set bit (LSB-side), or None if zero.
    fn lowest_set_bit(&self) -> Option<BitPosition>;

    /// Effective width: position of highest set bit + 1, or 0 if zero.
    fn effective_width(&self) -> BitCount;
}

impl<W: WordWidth> PositionOps for Word<W> {
    #[inline]
    fn leading_zeros(&self) -> BitCount {
        BitCount(self.0.leading_zeros_raw())
    }

    #[inline]
    fn trailing_zeros(&self) -> BitCount {
        BitCount(if self.0.is_zero() {
            W::BITS
        } else {
            self.0.trailing_zeros_raw()
        })
    }

    #[inline]
    fn leading_ones(&self) -> BitCount {
        BitCount(self.0.bitnot().leading_zeros_raw())
    }

    #[inline]
    fn trailing_ones(&self) -> BitCount {
        BitCount(self.0.bitnot().trailing_zeros_raw())
    }

    #[inline]
    fn highest_set_bit(&self) -> Option<BitPosition> {
        if self.0.is_zero() {
            None
        } else {
            Some(BitPosition(W::BITS - 1 - self.0.leading_zeros_raw()))
        }
    }

    #[inline]
    fn lowest_set_bit(&self) -> Option<BitPosition> {
        if self.0.is_zero() {
            None
        } else {
            Some(BitPosition(self.0.trailing_zeros_raw()))
        }
    }

    #[inline]
    fn effective_width(&self) -> BitCount {
        if self.0.is_zero() {
            BitCount(0)
        } else {
            BitCount(W::BITS - self.0.leading_zeros_raw())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::width::{Word8, Word16, Word32, Word64};

    // ── leading_zeros ─────────────────────────────────────────────────

    #[test]
    fn leading_zeros_all_zero() {
        assert_eq!(Word8::new(0).leading_zeros().value(), 8);
        assert_eq!(Word16::new(0).leading_zeros().value(), 16);
        assert_eq!(Word32::new(0).leading_zeros().value(), 32);
        assert_eq!(Word64::new(0).leading_zeros().value(), 64);
    }

    #[test]
    fn leading_zeros_msb_set() {
        assert_eq!(Word8::new(0x80).leading_zeros().value(), 0);
        assert_eq!(Word16::new(0x8000).leading_zeros().value(), 0);
        assert_eq!(Word32::new(0x8000_0000).leading_zeros().value(), 0);
    }

    #[test]
    fn leading_zeros_one() {
        assert_eq!(Word8::new(1).leading_zeros().value(), 7);
        assert_eq!(Word32::new(1).leading_zeros().value(), 31);
    }

    // ── trailing_zeros ────────────────────────────────────────────────

    #[test]
    fn trailing_zeros_all_zero() {
        assert_eq!(Word8::new(0).trailing_zeros().value(), 8);
        assert_eq!(Word64::new(0).trailing_zeros().value(), 64);
    }

    #[test]
    fn trailing_zeros_lsb_set() {
        assert_eq!(Word8::new(1).trailing_zeros().value(), 0);
        assert_eq!(Word32::new(1).trailing_zeros().value(), 0);
    }

    #[test]
    fn trailing_zeros_power_of_two() {
        assert_eq!(Word8::new(0b1000).trailing_zeros().value(), 3);
        assert_eq!(Word32::new(0x100).trailing_zeros().value(), 8);
    }

    // ── leading/trailing ones ─────────────────────────────────────────

    #[test]
    fn leading_ones() {
        assert_eq!(Word8::new(0xFF).leading_ones().value(), 8);
        assert_eq!(Word8::new(0xF0).leading_ones().value(), 4);
        assert_eq!(Word8::new(0x00).leading_ones().value(), 0);
    }

    #[test]
    fn trailing_ones() {
        assert_eq!(Word8::new(0xFF).trailing_ones().value(), 8);
        assert_eq!(Word8::new(0x0F).trailing_ones().value(), 4);
        assert_eq!(Word8::new(0x00).trailing_ones().value(), 0);
    }

    // ── highest/lowest set bit ────────────────────────────────────────

    #[test]
    fn highest_set_bit_zero() {
        assert!(Word8::new(0).highest_set_bit().is_none());
        assert!(Word64::new(0).highest_set_bit().is_none());
    }

    #[test]
    fn highest_set_bit_values() {
        assert_eq!(Word8::new(1).highest_set_bit().map(|p| p.value()), Some(0));
        assert_eq!(
            Word8::new(0x80).highest_set_bit().map(|p| p.value()),
            Some(7)
        );
        assert_eq!(
            Word8::new(0b0010_0000).highest_set_bit().map(|p| p.value()),
            Some(5)
        );
    }

    #[test]
    fn lowest_set_bit_zero() {
        assert!(Word8::new(0).lowest_set_bit().is_none());
    }

    #[test]
    fn lowest_set_bit_values() {
        assert_eq!(Word8::new(1).lowest_set_bit().map(|p| p.value()), Some(0));
        assert_eq!(
            Word8::new(0b1000).lowest_set_bit().map(|p| p.value()),
            Some(3)
        );
        assert_eq!(
            Word8::new(0x80).lowest_set_bit().map(|p| p.value()),
            Some(7)
        );
    }

    // ── effective_width ───────────────────────────────────────────────

    #[test]
    fn effective_width_zero() {
        assert_eq!(Word8::new(0).effective_width().value(), 0);
        assert_eq!(Word64::new(0).effective_width().value(), 0);
    }

    #[test]
    fn effective_width_values() {
        assert_eq!(Word8::new(1).effective_width().value(), 1);
        assert_eq!(Word8::new(0b11).effective_width().value(), 2);
        assert_eq!(Word8::new(0xFF).effective_width().value(), 8);
        assert_eq!(Word32::new(0x100).effective_width().value(), 9);
    }

    // ── cross-width consistency ───────────────────────────────────────

    #[test]
    fn position_consistency_16() {
        let w = Word16::new(0x0100); // bit 8 set
        assert_eq!(w.highest_set_bit().map(|p| p.value()), Some(8));
        assert_eq!(w.lowest_set_bit().map(|p| p.value()), Some(8));
        assert_eq!(w.effective_width().value(), 9);
        assert_eq!(w.leading_zeros().value(), 7);
        assert_eq!(w.trailing_zeros().value(), 8);
    }

    #[test]
    fn position_consistency_64() {
        let w = Word64::new(1u64 << 63);
        assert_eq!(w.highest_set_bit().map(|p| p.value()), Some(63));
        assert_eq!(w.leading_zeros().value(), 0);
        assert_eq!(w.trailing_zeros().value(), 63);
        assert_eq!(w.effective_width().value(), 64);
    }
}
