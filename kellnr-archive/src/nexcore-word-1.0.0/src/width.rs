//! # Word Width and Core Newtype
//!
//! Sealed `WordWidth` trait + `Word<W>` newtype that reinterprets unsigned integers
//! through the State (ς) lens.
//!
//! ## Tier: T1
//! ## Dominant: ς (State) — the bit IS the irreducible unit of state

use serde::{Deserialize, Serialize};
use std::fmt;

// ============================================================================
// Sealed trait pattern
// ============================================================================

mod sealed {
    pub trait Sealed {}
    impl Sealed for u8 {}
    impl Sealed for u16 {}
    impl Sealed for u32 {}
    impl Sealed for u64 {}
}

/// Trait for unsigned integer types that can serve as word storage.
///
/// Sealed to `u8`, `u16`, `u32`, `u64`. Provides the raw primitive operations
/// that the composable trait algebra builds upon.
pub trait WordWidth:
    sealed::Sealed + Copy + PartialEq + Eq + fmt::Debug + fmt::Binary + 'static
{
    /// The zero value for this width.
    const ZERO: Self;
    /// The all-ones value for this width.
    const MAX: Self;
    /// Number of bits in this word width.
    const BITS: u32;

    /// Count of set bits.
    fn count_ones_raw(self) -> u32;
    /// Count of leading zeros.
    fn leading_zeros_raw(self) -> u32;
    /// Count of trailing zeros.
    fn trailing_zeros_raw(self) -> u32;
    /// Rotate left by `n` bits.
    fn rotate_left_raw(self, n: u32) -> Self;
    /// Rotate right by `n` bits.
    fn rotate_right_raw(self, n: u32) -> Self;
    /// Reverse bit order.
    fn reverse_bits_raw(self) -> Self;
    /// Reverse byte order.
    fn swap_bytes_raw(self) -> Self;

    /// Bitwise AND.
    fn bitand(self, rhs: Self) -> Self;
    /// Bitwise OR.
    fn bitor(self, rhs: Self) -> Self;
    /// Bitwise XOR.
    fn bitxor(self, rhs: Self) -> Self;
    /// Bitwise NOT.
    fn bitnot(self) -> Self;
    /// Shift left.
    fn shl(self, n: u32) -> Self;
    /// Shift right.
    fn shr(self, n: u32) -> Self;

    /// Checked subtraction.
    fn checked_sub(self, rhs: Self) -> Option<Self>;
    /// Wrapping subtraction.
    fn wrapping_sub(self, rhs: Self) -> Self;

    /// Convert to u64 for cross-width operations.
    fn to_u64(self) -> u64;
    /// Convert from u64 (truncating).
    fn from_u64(v: u64) -> Self;

    /// Returns true if value is zero.
    fn is_zero(self) -> bool;

    /// Returns the value `1`.
    fn one() -> Self;
}

macro_rules! impl_word_width {
    ($ty:ty) => {
        impl WordWidth for $ty {
            const ZERO: Self = 0;
            const MAX: Self = <$ty>::MAX;
            const BITS: u32 = <$ty>::BITS;

            #[inline]
            fn count_ones_raw(self) -> u32 {
                self.count_ones()
            }
            #[inline]
            fn leading_zeros_raw(self) -> u32 {
                self.leading_zeros()
            }
            #[inline]
            fn trailing_zeros_raw(self) -> u32 {
                self.trailing_zeros()
            }
            #[inline]
            fn rotate_left_raw(self, n: u32) -> Self {
                <$ty>::rotate_left(self, n)
            }
            #[inline]
            fn rotate_right_raw(self, n: u32) -> Self {
                <$ty>::rotate_right(self, n)
            }
            #[inline]
            fn reverse_bits_raw(self) -> Self {
                <$ty>::reverse_bits(self)
            }
            #[inline]
            fn swap_bytes_raw(self) -> Self {
                <$ty>::swap_bytes(self)
            }
            #[inline]
            fn bitand(self, rhs: Self) -> Self {
                self & rhs
            }
            #[inline]
            fn bitor(self, rhs: Self) -> Self {
                self | rhs
            }
            #[inline]
            fn bitxor(self, rhs: Self) -> Self {
                self ^ rhs
            }
            #[inline]
            fn bitnot(self) -> Self {
                !self
            }
            #[inline]
            fn shl(self, n: u32) -> Self {
                if n >= Self::BITS {
                    Self::ZERO
                } else {
                    self << n
                }
            }
            #[inline]
            fn shr(self, n: u32) -> Self {
                if n >= Self::BITS {
                    Self::ZERO
                } else {
                    self >> n
                }
            }
            #[inline]
            fn checked_sub(self, rhs: Self) -> Option<Self> {
                <$ty>::checked_sub(self, rhs)
            }
            #[inline]
            fn wrapping_sub(self, rhs: Self) -> Self {
                <$ty>::wrapping_sub(self, rhs)
            }
            #[inline]
            fn to_u64(self) -> u64 {
                self as u64
            }
            #[inline]
            fn from_u64(v: u64) -> Self {
                v as $ty
            }
            #[inline]
            fn is_zero(self) -> bool {
                self == 0
            }
            #[inline]
            fn one() -> Self {
                1
            }
        }
    };
}

impl_word_width!(u8);
impl_word_width!(u16);
impl_word_width!(u32);
impl_word_width!(u64);

// ============================================================================
// Word<W> newtype
// ============================================================================

/// A typed binary word that reinterprets an unsigned integer as a state register.
///
/// Raw `u8`/`u16`/`u32`/`u64` ground to Quantity (N) — they represent magnitudes.
/// `Word<W>` reinterprets them through the State (ς) lens: each bit is an
/// irreducible unit of boolean state.
///
/// ## Tier: T1
/// ## Dominant: ς (State) — confidence 1.0
///
/// ## Example
///
/// ```
/// use nexcore_word::prelude::*;
///
/// let w = Word8::new(0b1010_1100);
/// assert_eq!(w.popcount().value(), 4);
/// ```
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Word<W: WordWidth>(pub W);

/// 8-bit word.
pub type Word8 = Word<u8>;
/// 16-bit word.
pub type Word16 = Word<u16>;
/// 32-bit word.
pub type Word32 = Word<u32>;
/// 64-bit word.
pub type Word64 = Word<u64>;

impl<W: WordWidth> Word<W> {
    /// Create a new word from a raw value.
    #[must_use]
    #[inline]
    pub const fn new(value: W) -> Self {
        Self(value)
    }

    /// Returns the inner raw value.
    #[must_use]
    #[inline]
    pub const fn raw(self) -> W {
        self.0
    }

    /// Returns the bit width of this word type.
    #[must_use]
    #[inline]
    pub const fn width() -> u32 {
        W::BITS
    }

    /// Returns true if all bits are zero.
    #[must_use]
    #[inline]
    pub fn is_zero(self) -> bool {
        self.0.is_zero()
    }

    /// Returns the zero word.
    #[must_use]
    #[inline]
    pub fn zero() -> Self {
        Self(W::ZERO)
    }

    /// Returns the all-ones word.
    #[must_use]
    #[inline]
    pub fn max_value() -> Self {
        Self(W::MAX)
    }
}

impl<W: WordWidth> From<W> for Word<W> {
    #[inline]
    fn from(v: W) -> Self {
        Self(v)
    }
}

impl<W: WordWidth> fmt::Debug for Word<W> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Word{}(0b{:0width$b})",
            W::BITS,
            self.0,
            width = W::BITS as usize
        )
    }
}

impl<W: WordWidth> fmt::Display for Word<W> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "0x{:X}", self.0.to_u64())
    }
}

impl<W: WordWidth> fmt::Binary for Word<W> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:0width$b}", self.0, width = W::BITS as usize)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn word8_creation() {
        let w = Word8::new(0xFF);
        assert_eq!(w.raw(), 0xFF);
        assert_eq!(Word8::width(), 8);
    }

    #[test]
    fn word16_creation() {
        let w = Word16::new(0xABCD);
        assert_eq!(w.raw(), 0xABCD);
        assert_eq!(Word16::width(), 16);
    }

    #[test]
    fn word32_creation() {
        let w = Word32::new(0xDEAD_BEEF);
        assert_eq!(w.raw(), 0xDEAD_BEEF);
        assert_eq!(Word32::width(), 32);
    }

    #[test]
    fn word64_creation() {
        let w = Word64::new(0xCAFE_BABE_DEAD_BEEF);
        assert_eq!(w.raw(), 0xCAFE_BABE_DEAD_BEEF);
        assert_eq!(Word64::width(), 64);
    }

    #[test]
    fn word_zero_and_max() {
        assert!(Word8::zero().is_zero());
        assert!(!Word8::max_value().is_zero());
        assert_eq!(Word8::max_value().raw(), 0xFF);
        assert_eq!(Word64::max_value().raw(), u64::MAX);
    }

    #[test]
    fn word_from_raw() {
        let w: Word8 = 42u8.into();
        assert_eq!(w.raw(), 42);
    }

    #[test]
    fn word_debug_format() {
        let w = Word8::new(0b1010_1100);
        let dbg = format!("{w:?}");
        assert!(dbg.contains("Word8"));
        assert!(dbg.contains("0b"));
    }

    #[test]
    fn word_display_hex() {
        let w = Word16::new(0xFF);
        assert_eq!(format!("{w}"), "0xFF");
    }

    #[test]
    fn word_binary_format() {
        let w = Word8::new(0b1010);
        let bin = format!("{w:b}");
        assert_eq!(bin, "00001010");
    }

    #[test]
    fn word_equality() {
        assert_eq!(Word32::new(42), Word32::new(42));
        assert_ne!(Word32::new(42), Word32::new(43));
    }

    #[test]
    fn word_ordering() {
        assert!(Word8::new(1) < Word8::new(2));
    }

    #[test]
    fn word_serde_roundtrip() {
        let w = Word32::new(0xDEAD_BEEF);
        let json = serde_json::to_string(&w);
        assert!(json.is_ok());
        let back: Result<Word32, _> =
            serde_json::from_str(json.as_ref().map(|s| s.as_str()).unwrap_or(""));
        assert!(back.is_ok());
        assert_eq!(back.unwrap_or(Word32::new(0)), w);
    }

    #[test]
    fn word_width_shl_overflow_safe() {
        // shl with n >= BITS returns 0
        assert_eq!(42u8.shl(8), 0);
        assert_eq!(42u8.shl(100), 0);
        assert_eq!(42u16.shl(16), 0);
        assert_eq!(42u32.shl(32), 0);
        assert_eq!(42u64.shl(64), 0);
    }

    #[test]
    fn word_width_shr_overflow_safe() {
        assert_eq!(42u8.shr(8), 0);
        assert_eq!(42u64.shr(64), 0);
    }
}
