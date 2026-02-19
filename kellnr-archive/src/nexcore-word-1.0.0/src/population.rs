//! # Population Operations
//!
//! Bit population counting, parity, and Hamming distance.
//!
//! ## Tier: T2-P (ς State + N Quantity)
//! ## Dominant: N (Quantity) — counting set bits is a measurement operation

use crate::properties::{BitCount, Parity};
use crate::width::{Word, WordWidth};

/// Operations that count or classify bit populations.
///
/// Blanket-implemented for all `Word<W>`.
pub trait PopulationOps {
    /// Count of set (1) bits.
    fn popcount(&self) -> BitCount;

    /// Count of unset (0) bits.
    fn zero_count(&self) -> BitCount;

    /// Parity of the set bit population.
    fn parity(&self) -> Parity;

    /// Hamming distance (number of differing bit positions) to another word.
    fn hamming_distance(&self, other: &Self) -> BitCount;
}

impl<W: WordWidth> PopulationOps for Word<W> {
    #[inline]
    fn popcount(&self) -> BitCount {
        BitCount(self.0.count_ones_raw())
    }

    #[inline]
    fn zero_count(&self) -> BitCount {
        BitCount(W::BITS - self.0.count_ones_raw())
    }

    #[inline]
    fn parity(&self) -> Parity {
        Parity::from_popcount(self.0.count_ones_raw())
    }

    #[inline]
    fn hamming_distance(&self, other: &Self) -> BitCount {
        BitCount(self.0.bitxor(other.0).count_ones_raw())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::width::{Word8, Word16, Word32, Word64};

    // ── popcount ──────────────────────────────────────────────────────

    #[test]
    fn popcount_zero() {
        assert_eq!(Word8::new(0).popcount().value(), 0);
        assert_eq!(Word16::new(0).popcount().value(), 0);
        assert_eq!(Word32::new(0).popcount().value(), 0);
        assert_eq!(Word64::new(0).popcount().value(), 0);
    }

    #[test]
    fn popcount_max() {
        assert_eq!(Word8::new(0xFF).popcount().value(), 8);
        assert_eq!(Word16::new(0xFFFF).popcount().value(), 16);
        assert_eq!(Word32::new(0xFFFF_FFFF).popcount().value(), 32);
        assert_eq!(Word64::new(u64::MAX).popcount().value(), 64);
    }

    #[test]
    fn popcount_mixed() {
        assert_eq!(Word8::new(0b1010_1010).popcount().value(), 4);
        assert_eq!(Word8::new(0b0000_0001).popcount().value(), 1);
        assert_eq!(Word32::new(0xDEAD_BEEF).popcount().value(), 24);
    }

    // ── zero_count ────────────────────────────────────────────────────

    #[test]
    fn zero_count_complements_popcount() {
        let w = Word8::new(0b1010_1100);
        assert_eq!(w.popcount().value() + w.zero_count().value(), 8);

        let w32 = Word32::new(0xDEAD_BEEF);
        assert_eq!(w32.popcount().value() + w32.zero_count().value(), 32);
    }

    // ── parity ────────────────────────────────────────────────────────

    #[test]
    fn parity_even() {
        assert_eq!(Word8::new(0b0000_0000).parity(), Parity::Even);
        assert_eq!(Word8::new(0b1010_1010).parity(), Parity::Even);
        assert_eq!(Word8::new(0xFF).parity(), Parity::Even);
    }

    #[test]
    fn parity_odd() {
        assert_eq!(Word8::new(0b0000_0001).parity(), Parity::Odd);
        assert_eq!(Word8::new(0b1010_1011).parity(), Parity::Odd);
        assert_eq!(Word8::new(0b0111_1111).parity(), Parity::Odd);
    }

    #[test]
    fn parity_all_widths() {
        assert_eq!(Word16::new(1).parity(), Parity::Odd);
        assert_eq!(Word32::new(3).parity(), Parity::Even);
        assert_eq!(Word64::new(7).parity(), Parity::Odd);
    }

    // ── hamming_distance ──────────────────────────────────────────────

    #[test]
    fn hamming_identical() {
        assert_eq!(
            Word8::new(0xAB).hamming_distance(&Word8::new(0xAB)).value(),
            0
        );
    }

    #[test]
    fn hamming_single_bit() {
        assert_eq!(
            Word8::new(0b0000)
                .hamming_distance(&Word8::new(0b0001))
                .value(),
            1
        );
    }

    #[test]
    fn hamming_all_differ() {
        assert_eq!(
            Word8::new(0x00).hamming_distance(&Word8::new(0xFF)).value(),
            8
        );
    }

    #[test]
    fn hamming_symmetry() {
        let a = Word32::new(0xDEAD_BEEF);
        let b = Word32::new(0xCAFE_BABE);
        assert_eq!(a.hamming_distance(&b), b.hamming_distance(&a));
    }

    #[test]
    fn hamming_all_widths() {
        assert_eq!(
            Word16::new(0xFF00)
                .hamming_distance(&Word16::new(0x00FF))
                .value(),
            16
        );
        assert_eq!(
            Word64::new(0)
                .hamming_distance(&Word64::new(u64::MAX))
                .value(),
            64
        );
    }
}
