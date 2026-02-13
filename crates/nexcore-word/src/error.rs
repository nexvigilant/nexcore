//! # Word Error Types
//!
//! Error types for binary word operations.
//!
//! ## Tier: T2-P (∂ Boundary + Σ Sum)
//!
//! Errors represent boundary violations in bit-level state manipulation.

use serde::{Deserialize, Serialize};

/// Errors from binary word operations.
///
/// ## Tier: T2-P
/// ## Dominant: ∂ (Boundary) — errors signal violated constraints
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, thiserror::Error)]
pub enum WordError {
    /// Bit position exceeds word width.
    #[error("bit position {position} exceeds word width {width}")]
    BitOutOfRange {
        /// The requested position.
        position: u32,
        /// The word width in bits.
        width: u32,
    },

    /// Field specification exceeds word width.
    #[error("field at offset {offset} with length {length} exceeds word width {width}")]
    FieldOutOfRange {
        /// Start offset of the field.
        offset: u32,
        /// Length of the field in bits.
        length: u32,
        /// The word width in bits.
        width: u32,
    },

    /// Value does not fit in the specified field width.
    #[error("value {value} does not fit in {field_width}-bit field")]
    ValueOverflow {
        /// The value that overflowed.
        value: u64,
        /// The field width in bits.
        field_width: u32,
    },

    /// Operation requires a non-zero value.
    #[error("operation requires non-zero value")]
    ZeroValue,

    /// Alignment must be a power of two.
    #[error("alignment {alignment} is not a power of two")]
    InvalidAlignment {
        /// The invalid alignment value.
        alignment: u64,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_display_bit_out_of_range() {
        let e = WordError::BitOutOfRange {
            position: 9,
            width: 8,
        };
        assert!(format!("{e}").contains("9"));
        assert!(format!("{e}").contains("8"));
    }

    #[test]
    fn error_display_field_out_of_range() {
        let e = WordError::FieldOutOfRange {
            offset: 6,
            length: 4,
            width: 8,
        };
        assert!(format!("{e}").contains("offset 6"));
    }

    #[test]
    fn error_display_value_overflow() {
        let e = WordError::ValueOverflow {
            value: 256,
            field_width: 8,
        };
        assert!(format!("{e}").contains("256"));
    }

    #[test]
    fn error_display_zero_value() {
        let e = WordError::ZeroValue;
        assert!(format!("{e}").contains("non-zero"));
    }

    #[test]
    fn error_display_invalid_alignment() {
        let e = WordError::InvalidAlignment { alignment: 3 };
        assert!(format!("{e}").contains("3"));
    }

    #[test]
    fn error_serde_roundtrip() {
        let errors = vec![
            WordError::BitOutOfRange {
                position: 10,
                width: 8,
            },
            WordError::FieldOutOfRange {
                offset: 2,
                length: 8,
                width: 8,
            },
            WordError::ValueOverflow {
                value: 999,
                field_width: 4,
            },
            WordError::ZeroValue,
            WordError::InvalidAlignment { alignment: 6 },
        ];
        for e in &errors {
            let json = serde_json::to_string(e);
            assert!(json.is_ok());
            let back: Result<WordError, _> =
                serde_json::from_str(json.as_ref().map(|s| s.as_str()).unwrap_or(""));
            assert!(back.is_ok());
            assert_eq!(&back.unwrap_or(WordError::ZeroValue), e);
        }
    }
}
