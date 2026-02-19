//! # Arithmetic Operations
//!
//! Power-of-two tests, logarithms, integer square root, binary GCD.
//!
//! ## Tier: T2-C (N Quantity + ∂ Boundary + ς State)
//! ## Dominant: N (Quantity) — arithmetic measurements derived from bit state

use crate::error::WordError;
use crate::properties::{Alignment, BitCount};
use crate::width::{Word, WordWidth};

/// Arithmetic operations on binary words.
///
/// Blanket-implemented for all `Word<W>`.
pub trait ArithmeticOps: Sized {
    /// Returns true if exactly one bit is set (value is a power of two).
    fn is_power_of_two(&self) -> bool;

    /// Returns the next power of two ≥ self, or None on overflow.
    fn next_power_of_two(&self) -> Option<Self>;

    /// Floor log base 2 (position of highest set bit). Returns error for zero.
    fn log2(&self) -> Result<BitCount, WordError>;

    /// Integer square root via Newton's method. Returns error for zero.
    fn isqrt(&self) -> Result<Self, WordError>;

    /// Binary GCD (Stein's algorithm). Returns error if both are zero.
    fn binary_gcd(&self, other: &Self) -> Result<Self, WordError>;

    /// Align value up to the next multiple of `alignment` (must be power of two).
    fn align_up(&self, alignment: u32) -> Result<Self, WordError>;

    /// Check alignment: returns `Aligned(trailing_zeros)` if aligned to power-of-two.
    fn check_alignment(&self) -> Alignment;
}

impl<W: WordWidth> ArithmeticOps for Word<W> {
    #[inline]
    fn is_power_of_two(&self) -> bool {
        !self.0.is_zero() && self.0.bitand(self.0.wrapping_sub(W::one())).is_zero()
    }

    fn next_power_of_two(&self) -> Option<Self> {
        if self.0.is_zero() {
            return Some(Word(W::one()));
        }
        if self.is_power_of_two() {
            return Some(*self);
        }
        let bits_needed = W::BITS - self.0.leading_zeros_raw();
        if bits_needed >= W::BITS {
            return None; // overflow
        }
        Some(Word(W::one().shl(bits_needed)))
    }

    fn log2(&self) -> Result<BitCount, WordError> {
        if self.0.is_zero() {
            return Err(WordError::ZeroValue);
        }
        Ok(BitCount(W::BITS - 1 - self.0.leading_zeros_raw()))
    }

    fn isqrt(&self) -> Result<Self, WordError> {
        if self.0.is_zero() {
            return Err(WordError::ZeroValue);
        }
        // Newton's method on u64, then truncate back
        let n = self.0.to_u64();
        if n == 1 {
            return Ok(Word(W::one()));
        }
        let mut x = 1u64 << ((64 - n.leading_zeros() + 1) / 2);
        loop {
            let next = (x + n / x) / 2;
            if next >= x {
                break;
            }
            x = next;
        }
        // Verify: x*x <= n && (x+1)*(x+1) > n
        Ok(Word(W::from_u64(x)))
    }

    fn binary_gcd(&self, other: &Self) -> Result<Self, WordError> {
        if self.0.is_zero() && other.0.is_zero() {
            return Err(WordError::ZeroValue);
        }
        // Stein's algorithm on u64
        let mut a = self.0.to_u64();
        let mut b = other.0.to_u64();

        if a == 0 {
            return Ok(Word(W::from_u64(b)));
        }
        if b == 0 {
            return Ok(Word(W::from_u64(a)));
        }

        let shift = (a | b).trailing_zeros();
        a >>= a.trailing_zeros();

        loop {
            b >>= b.trailing_zeros();
            if a > b {
                std::mem::swap(&mut a, &mut b);
            }
            b -= a;
            if b == 0 {
                break;
            }
        }

        Ok(Word(W::from_u64(a << shift)))
    }

    fn align_up(&self, alignment: u32) -> Result<Self, WordError> {
        if alignment == 0 || (alignment & (alignment - 1)) != 0 {
            return Err(WordError::InvalidAlignment {
                alignment: alignment as u64,
            });
        }
        let align = W::from_u64(alignment as u64);
        let mask = align.wrapping_sub(W::one());
        // (self + mask) & !mask
        let sum_raw = self.0.to_u64().wrapping_add(mask.to_u64());
        let result = W::from_u64(sum_raw).bitand(mask.bitnot());
        Ok(Word(result))
    }

    fn check_alignment(&self) -> Alignment {
        if self.0.is_zero() {
            // Zero is aligned to any power of two; report max alignment
            return Alignment::Aligned(W::BITS);
        }
        let tz = self.0.trailing_zeros_raw();
        if tz > 0 {
            Alignment::Aligned(tz)
        } else {
            Alignment::Unaligned
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::width::{Word8, Word16, Word32, Word64};

    // ── is_power_of_two ───────────────────────────────────────────────

    #[test]
    fn is_power_of_two_true() {
        assert!(Word8::new(1).is_power_of_two());
        assert!(Word8::new(2).is_power_of_two());
        assert!(Word8::new(4).is_power_of_two());
        assert!(Word8::new(128).is_power_of_two());
        assert!(Word32::new(1 << 20).is_power_of_two());
    }

    #[test]
    fn is_power_of_two_false() {
        assert!(!Word8::new(0).is_power_of_two());
        assert!(!Word8::new(3).is_power_of_two());
        assert!(!Word8::new(5).is_power_of_two());
        assert!(!Word8::new(255).is_power_of_two());
    }

    // ── next_power_of_two ─────────────────────────────────────────────

    #[test]
    fn next_power_of_two_exact() {
        let result = Word8::new(4).next_power_of_two();
        assert!(result.is_some());
        assert_eq!(result.map(|w| w.raw()), Some(4));
    }

    #[test]
    fn next_power_of_two_round_up() {
        let result = Word8::new(5).next_power_of_two();
        assert!(result.is_some());
        assert_eq!(result.map(|w| w.raw()), Some(8));
    }

    #[test]
    fn next_power_of_two_zero() {
        let result = Word8::new(0).next_power_of_two();
        assert!(result.is_some());
        assert_eq!(result.map(|w| w.raw()), Some(1));
    }

    #[test]
    fn next_power_of_two_overflow() {
        // 129 for u8 would need 256 which overflows
        assert!(Word8::new(129).next_power_of_two().is_none());
    }

    // ── log2 ──────────────────────────────────────────────────────────

    #[test]
    fn log2_powers() {
        assert_eq!(Word8::new(1).log2().map(|b| b.value()), Ok(0));
        assert_eq!(Word8::new(2).log2().map(|b| b.value()), Ok(1));
        assert_eq!(Word8::new(4).log2().map(|b| b.value()), Ok(2));
        assert_eq!(Word8::new(128).log2().map(|b| b.value()), Ok(7));
    }

    #[test]
    fn log2_non_powers() {
        assert_eq!(Word8::new(3).log2().map(|b| b.value()), Ok(1));
        assert_eq!(Word8::new(255).log2().map(|b| b.value()), Ok(7));
    }

    #[test]
    fn log2_zero() {
        assert!(Word8::new(0).log2().is_err());
    }

    #[test]
    fn log2_large() {
        assert_eq!(Word64::new(1u64 << 63).log2().map(|b| b.value()), Ok(63));
    }

    // ── isqrt ─────────────────────────────────────────────────────────

    #[test]
    fn isqrt_perfect_squares() {
        assert_eq!(Word32::new(1).isqrt().map(|w| w.raw()), Ok(1));
        assert_eq!(Word32::new(4).isqrt().map(|w| w.raw()), Ok(2));
        assert_eq!(Word32::new(9).isqrt().map(|w| w.raw()), Ok(3));
        assert_eq!(Word32::new(16).isqrt().map(|w| w.raw()), Ok(4));
        assert_eq!(Word32::new(100).isqrt().map(|w| w.raw()), Ok(10));
    }

    #[test]
    fn isqrt_non_perfect() {
        // floor(sqrt(10)) = 3
        assert_eq!(Word32::new(10).isqrt().map(|w| w.raw()), Ok(3));
        // floor(sqrt(255)) = 15
        assert_eq!(Word32::new(255).isqrt().map(|w| w.raw()), Ok(15));
    }

    #[test]
    fn isqrt_zero() {
        assert!(Word32::new(0).isqrt().is_err());
    }

    #[test]
    fn isqrt_large() {
        // floor(sqrt(2^32 - 1)) = 65535
        assert_eq!(Word64::new(0xFFFF_FFFF).isqrt().map(|w| w.raw()), Ok(65535));
    }

    // ── binary_gcd ────────────────────────────────────────────────────

    #[test]
    fn gcd_basic() {
        assert_eq!(
            Word32::new(12).binary_gcd(&Word32::new(8)).map(|w| w.raw()),
            Ok(4)
        );
        assert_eq!(
            Word32::new(54)
                .binary_gcd(&Word32::new(24))
                .map(|w| w.raw()),
            Ok(6)
        );
    }

    #[test]
    fn gcd_one_zero() {
        assert_eq!(
            Word32::new(0).binary_gcd(&Word32::new(5)).map(|w| w.raw()),
            Ok(5)
        );
        assert_eq!(
            Word32::new(7).binary_gcd(&Word32::new(0)).map(|w| w.raw()),
            Ok(7)
        );
    }

    #[test]
    fn gcd_both_zero() {
        assert!(Word32::new(0).binary_gcd(&Word32::new(0)).is_err());
    }

    #[test]
    fn gcd_coprime() {
        assert_eq!(
            Word32::new(7).binary_gcd(&Word32::new(13)).map(|w| w.raw()),
            Ok(1)
        );
    }

    #[test]
    fn gcd_equal() {
        assert_eq!(
            Word32::new(42)
                .binary_gcd(&Word32::new(42))
                .map(|w| w.raw()),
            Ok(42)
        );
    }

    #[test]
    fn gcd_commutative() {
        let a = Word32::new(48);
        let b = Word32::new(18);
        assert_eq!(a.binary_gcd(&b), b.binary_gcd(&a));
    }

    // ── align_up ──────────────────────────────────────────────────────

    #[test]
    fn align_up_already_aligned() {
        assert_eq!(Word32::new(16).align_up(16).map(|w| w.raw()), Ok(16));
    }

    #[test]
    fn align_up_round_up() {
        assert_eq!(Word32::new(17).align_up(16).map(|w| w.raw()), Ok(32));
        assert_eq!(Word32::new(1).align_up(4).map(|w| w.raw()), Ok(4));
    }

    #[test]
    fn align_up_invalid_alignment() {
        assert!(Word32::new(10).align_up(3).is_err());
        assert!(Word32::new(10).align_up(0).is_err());
    }

    // ── check_alignment ───────────────────────────────────────────────

    #[test]
    fn check_alignment_zero() {
        assert!(Word8::new(0).check_alignment().is_aligned());
    }

    #[test]
    fn check_alignment_odd() {
        assert_eq!(Word8::new(1).check_alignment(), Alignment::Unaligned);
        assert_eq!(Word8::new(3).check_alignment(), Alignment::Unaligned);
    }

    #[test]
    fn check_alignment_powers() {
        assert_eq!(Word8::new(2).check_alignment(), Alignment::Aligned(1));
        assert_eq!(Word8::new(4).check_alignment(), Alignment::Aligned(2));
        assert_eq!(Word8::new(8).check_alignment(), Alignment::Aligned(3));
        assert_eq!(Word32::new(256).check_alignment(), Alignment::Aligned(8));
    }

    // ── cross-width ───────────────────────────────────────────────────

    #[test]
    fn arithmetic_word8() {
        assert!(Word8::new(64).is_power_of_two());
        assert_eq!(Word8::new(64).log2().map(|b| b.value()), Ok(6));
    }

    #[test]
    fn arithmetic_word16() {
        assert_eq!(Word16::new(1024).log2().map(|b| b.value()), Ok(10));
        assert_eq!(
            Word16::new(100)
                .binary_gcd(&Word16::new(75))
                .map(|w| w.raw()),
            Ok(25)
        );
    }

    #[test]
    fn arithmetic_word64() {
        let big = Word64::new(1u64 << 40);
        assert!(big.is_power_of_two());
        assert_eq!(big.log2().map(|b| b.value()), Ok(40));
    }
}
