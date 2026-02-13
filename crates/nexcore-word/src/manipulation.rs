//! # Manipulation Operations
//!
//! Bit-level and field-level manipulation.
//!
//! ## Tier: T2-C (ς State + ∂ Boundary + σ Sequence)
//! ## Dominant: ς (State) — directly transforming state register bits

use crate::error::WordError;
use crate::width::{Word, WordWidth};

/// Operations that modify individual bits or bit fields.
///
/// Blanket-implemented for all `Word<W>`.
pub trait ManipulationOps: Sized {
    /// Set a single bit at `position`. Returns error if out of range.
    fn set_bit(&self, position: u32) -> Result<Self, WordError>;

    /// Clear a single bit at `position`. Returns error if out of range.
    fn clear_bit(&self, position: u32) -> Result<Self, WordError>;

    /// Toggle a single bit at `position`. Returns error if out of range.
    fn toggle_bit(&self, position: u32) -> Result<Self, WordError>;

    /// Test whether bit at `position` is set. Returns error if out of range.
    fn test_bit(&self, position: u32) -> Result<bool, WordError>;

    /// Extract a bit field: `length` bits starting at `offset` from LSB.
    fn extract_field(&self, offset: u32, length: u32) -> Result<Self, WordError>;

    /// Insert `value` into field at `offset` with `length` bits.
    fn insert_field(&self, offset: u32, length: u32, value: Self) -> Result<Self, WordError>;

    /// Reverse all bits in the word.
    fn reverse_bits(&self) -> Self;

    /// Swap byte order (endianness conversion).
    fn swap_bytes(&self) -> Self;
}

impl<W: WordWidth> ManipulationOps for Word<W> {
    fn set_bit(&self, position: u32) -> Result<Self, WordError> {
        if position >= W::BITS {
            return Err(WordError::BitOutOfRange {
                position,
                width: W::BITS,
            });
        }
        Ok(Word(self.0.bitor(W::one().shl(position))))
    }

    fn clear_bit(&self, position: u32) -> Result<Self, WordError> {
        if position >= W::BITS {
            return Err(WordError::BitOutOfRange {
                position,
                width: W::BITS,
            });
        }
        Ok(Word(self.0.bitand(W::one().shl(position).bitnot())))
    }

    fn toggle_bit(&self, position: u32) -> Result<Self, WordError> {
        if position >= W::BITS {
            return Err(WordError::BitOutOfRange {
                position,
                width: W::BITS,
            });
        }
        Ok(Word(self.0.bitxor(W::one().shl(position))))
    }

    fn test_bit(&self, position: u32) -> Result<bool, WordError> {
        if position >= W::BITS {
            return Err(WordError::BitOutOfRange {
                position,
                width: W::BITS,
            });
        }
        Ok(!self.0.bitand(W::one().shl(position)).is_zero())
    }

    fn extract_field(&self, offset: u32, length: u32) -> Result<Self, WordError> {
        if length == 0 {
            return Ok(Word(W::ZERO));
        }
        if offset.saturating_add(length) > W::BITS {
            return Err(WordError::FieldOutOfRange {
                offset,
                length,
                width: W::BITS,
            });
        }
        // Create mask of `length` ones, shift value right by offset, then AND
        let mask = if length >= W::BITS {
            W::MAX
        } else {
            W::one().shl(length).wrapping_sub(W::one())
        };
        Ok(Word(self.0.shr(offset).bitand(mask)))
    }

    fn insert_field(&self, offset: u32, length: u32, value: Self) -> Result<Self, WordError> {
        if length == 0 {
            return Ok(*self);
        }
        if offset.saturating_add(length) > W::BITS {
            return Err(WordError::FieldOutOfRange {
                offset,
                length,
                width: W::BITS,
            });
        }
        // Check value fits in field
        let field_mask = if length >= W::BITS {
            W::MAX
        } else {
            W::one().shl(length).wrapping_sub(W::one())
        };
        if !value.0.bitand(field_mask.bitnot()).is_zero() {
            return Err(WordError::ValueOverflow {
                value: value.0.to_u64(),
                field_width: length,
            });
        }
        // Clear the field in self, then OR in the shifted value
        let shifted_mask = field_mask.shl(offset);
        let cleared = self.0.bitand(shifted_mask.bitnot());
        let inserted = value.0.shl(offset);
        Ok(Word(cleared.bitor(inserted)))
    }

    #[inline]
    fn reverse_bits(&self) -> Self {
        Word(self.0.reverse_bits_raw())
    }

    #[inline]
    fn swap_bytes(&self) -> Self {
        Word(self.0.swap_bytes_raw())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::width::{Word8, Word16, Word32, Word64};

    // ── set_bit ───────────────────────────────────────────────────────

    #[test]
    fn set_bit_basic() {
        let w = Word8::new(0);
        let result = w.set_bit(0);
        assert!(result.is_ok());
        assert_eq!(result.unwrap_or(Word8::new(0)).raw(), 1);
    }

    #[test]
    fn set_bit_high() {
        let result = Word8::new(0).set_bit(7);
        assert!(result.is_ok());
        assert_eq!(result.unwrap_or(Word8::new(0)).raw(), 0x80);
    }

    #[test]
    fn set_bit_idempotent() {
        let w = Word8::new(0b0001);
        let result = w.set_bit(0);
        assert!(result.is_ok());
        assert_eq!(result.unwrap_or(Word8::new(0)).raw(), 1);
    }

    #[test]
    fn set_bit_out_of_range() {
        assert!(Word8::new(0).set_bit(8).is_err());
        assert!(Word8::new(0).set_bit(100).is_err());
    }

    // ── clear_bit ─────────────────────────────────────────────────────

    #[test]
    fn clear_bit_basic() {
        let result = Word8::new(0xFF).clear_bit(0);
        assert!(result.is_ok());
        assert_eq!(result.unwrap_or(Word8::new(0)).raw(), 0xFE);
    }

    #[test]
    fn clear_bit_already_clear() {
        let result = Word8::new(0).clear_bit(3);
        assert!(result.is_ok());
        assert_eq!(result.unwrap_or(Word8::new(0xFF)).raw(), 0);
    }

    #[test]
    fn clear_bit_out_of_range() {
        assert!(Word8::new(0xFF).clear_bit(8).is_err());
    }

    // ── toggle_bit ────────────────────────────────────────────────────

    #[test]
    fn toggle_bit_set_to_clear() {
        let result = Word8::new(0b0001).toggle_bit(0);
        assert!(result.is_ok());
        assert_eq!(result.unwrap_or(Word8::new(0xFF)).raw(), 0);
    }

    #[test]
    fn toggle_bit_clear_to_set() {
        let result = Word8::new(0).toggle_bit(3);
        assert!(result.is_ok());
        assert_eq!(result.unwrap_or(Word8::new(0)).raw(), 0b1000);
    }

    #[test]
    fn toggle_bit_double_is_identity() {
        let w = Word32::new(0xDEAD_BEEF);
        let t1 = w.toggle_bit(15);
        assert!(t1.is_ok());
        let t2 = t1.unwrap_or(Word32::new(0)).toggle_bit(15);
        assert!(t2.is_ok());
        assert_eq!(t2.unwrap_or(Word32::new(0)).raw(), w.raw());
    }

    // ── test_bit ──────────────────────────────────────────────────────

    #[test]
    fn test_bit_set() {
        let result = Word8::new(0b1010).test_bit(1);
        assert!(result.is_ok());
        assert_eq!(result.unwrap_or(false), true);
    }

    #[test]
    fn test_bit_clear() {
        let result = Word8::new(0b1010).test_bit(0);
        assert!(result.is_ok());
        assert_eq!(result.unwrap_or(true), false);
    }

    #[test]
    fn test_bit_out_of_range() {
        assert!(Word8::new(0xFF).test_bit(8).is_err());
    }

    // ── extract_field ─────────────────────────────────────────────────

    #[test]
    fn extract_field_basic() {
        // 0b1010_1100 → extract 4 bits at offset 2 → 0b1011
        let result = Word8::new(0b1010_1100).extract_field(2, 4);
        assert!(result.is_ok());
        assert_eq!(result.unwrap_or(Word8::new(0)).raw(), 0b1011);
    }

    #[test]
    fn extract_field_full_width() {
        let result = Word8::new(0xAB).extract_field(0, 8);
        assert!(result.is_ok());
        assert_eq!(result.unwrap_or(Word8::new(0)).raw(), 0xAB);
    }

    #[test]
    fn extract_field_zero_length() {
        let result = Word8::new(0xFF).extract_field(0, 0);
        assert!(result.is_ok());
        assert_eq!(result.unwrap_or(Word8::new(0xFF)).raw(), 0);
    }

    #[test]
    fn extract_field_out_of_range() {
        assert!(Word8::new(0xFF).extract_field(6, 4).is_err());
    }

    // ── insert_field ──────────────────────────────────────────────────

    #[test]
    fn insert_field_basic() {
        // Insert 0b11 at offset 2 with length 2 into 0b0000_0000
        let result = Word8::new(0).insert_field(2, 2, Word8::new(0b11));
        assert!(result.is_ok());
        assert_eq!(result.unwrap_or(Word8::new(0)).raw(), 0b0000_1100);
    }

    #[test]
    fn insert_field_overwrite() {
        // Insert 0b01 at offset 0 with length 2 into 0b1111_1111
        let result = Word8::new(0xFF).insert_field(0, 2, Word8::new(0b01));
        assert!(result.is_ok());
        assert_eq!(result.unwrap_or(Word8::new(0)).raw(), 0b1111_1101);
    }

    #[test]
    fn insert_field_overflow() {
        // Value 0xFF doesn't fit in 4-bit field
        assert!(Word8::new(0).insert_field(0, 4, Word8::new(0xFF)).is_err());
    }

    #[test]
    fn insert_field_zero_length() {
        let w = Word8::new(0xAB);
        let result = w.insert_field(0, 0, Word8::new(0));
        assert!(result.is_ok());
        assert_eq!(result.unwrap_or(Word8::new(0)).raw(), 0xAB);
    }

    // ── extract/insert roundtrip ──────────────────────────────────────

    #[test]
    fn extract_insert_roundtrip() {
        let original = Word32::new(0xDEAD_BEEF);
        let extracted = original.extract_field(8, 8);
        assert!(extracted.is_ok());
        let cleared = original.insert_field(8, 8, extracted.unwrap_or(Word32::new(0)));
        assert!(cleared.is_ok());
        assert_eq!(cleared.unwrap_or(Word32::new(0)).raw(), original.raw());
    }

    // ── reverse_bits ──────────────────────────────────────────────────

    #[test]
    fn reverse_bits_basic() {
        assert_eq!(Word8::new(0b1000_0000).reverse_bits().raw(), 0b0000_0001);
        assert_eq!(Word8::new(0b1010_0000).reverse_bits().raw(), 0b0000_0101);
    }

    #[test]
    fn reverse_bits_symmetric() {
        assert_eq!(Word8::new(0xFF).reverse_bits().raw(), 0xFF);
        assert_eq!(Word8::new(0x00).reverse_bits().raw(), 0x00);
    }

    #[test]
    fn reverse_bits_double_is_identity() {
        let w = Word32::new(0xDEAD_BEEF);
        assert_eq!(w.reverse_bits().reverse_bits().raw(), w.raw());
    }

    // ── swap_bytes ────────────────────────────────────────────────────

    #[test]
    fn swap_bytes_16() {
        assert_eq!(Word16::new(0xABCD).swap_bytes().raw(), 0xCDAB);
    }

    #[test]
    fn swap_bytes_32() {
        assert_eq!(Word32::new(0x12345678).swap_bytes().raw(), 0x78563412);
    }

    #[test]
    fn swap_bytes_double_is_identity() {
        let w = Word64::new(0xCAFE_BABE_DEAD_BEEF);
        assert_eq!(w.swap_bytes().swap_bytes().raw(), w.raw());
    }

    // ── cross-width tests ─────────────────────────────────────────────

    #[test]
    fn manipulation_word64() {
        let w = Word64::new(0);
        let result = w.set_bit(63);
        assert!(result.is_ok());
        let set_word = result.unwrap_or(Word64::new(0));
        assert_eq!(set_word.raw(), 1u64 << 63);

        let tb = set_word.test_bit(63);
        assert!(tb.is_ok());
        assert_eq!(tb.unwrap_or(false), true);
    }
}
