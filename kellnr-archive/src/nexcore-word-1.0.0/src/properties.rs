//! # Typed Bit Properties
//!
//! Semantic newtypes for bit-level results, grounded to T1 primitives.
//!
//! | Type | Dominant | Confidence | Tier |
//! |------|----------|-----------|------|
//! | `BitCount` | Quantity (N) | 1.0 | T1 |
//! | `BitPosition` | Sequence (σ) | 0.9 | T2-P |
//! | `Parity` | Comparison (κ) | 1.0 | T1 |
//! | `Alignment` | Boundary (∂) | 1.0 | T1 |

use serde::{Deserialize, Serialize};
use std::fmt;

/// Count of bits (population count, zero count, etc.).
///
/// ## Tier: T1
/// ## Dominant: N (Quantity) — a pure count of set/unset bits
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct BitCount(pub u32);

impl BitCount {
    /// Returns the raw count value.
    #[must_use]
    pub const fn value(self) -> u32 {
        self.0
    }

    /// Returns true if the count is zero.
    #[must_use]
    pub const fn is_zero(self) -> bool {
        self.0 == 0
    }
}

impl fmt::Display for BitCount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} bits", self.0)
    }
}

impl From<u32> for BitCount {
    fn from(v: u32) -> Self {
        Self(v)
    }
}

/// Position of a bit within a word (0-indexed from LSB).
///
/// ## Tier: T2-P
/// ## Dominant: σ (Sequence) + λ (Location) — ordinal position within a state register
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct BitPosition(pub u32);

impl BitPosition {
    /// Returns the raw position value.
    #[must_use]
    pub const fn value(self) -> u32 {
        self.0
    }

    /// Converts to a single-bit mask at this position (as u64).
    #[must_use]
    pub const fn as_mask_u64(self) -> u64 {
        1u64 << self.0
    }
}

impl fmt::Display for BitPosition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "bit[{}]", self.0)
    }
}

impl From<u32> for BitPosition {
    fn from(v: u32) -> Self {
        Self(v)
    }
}

/// Parity of set bits in a word.
///
/// ## Tier: T1
/// ## Dominant: κ (Comparison) — binary classification of oddness/evenness
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Parity {
    /// Even number of set bits.
    Even,
    /// Odd number of set bits.
    Odd,
}

impl Parity {
    /// Returns true if even parity.
    #[must_use]
    pub const fn is_even(self) -> bool {
        matches!(self, Self::Even)
    }

    /// Returns true if odd parity.
    #[must_use]
    pub const fn is_odd(self) -> bool {
        matches!(self, Self::Odd)
    }

    /// Construct from a popcount value.
    #[must_use]
    pub const fn from_popcount(count: u32) -> Self {
        if count % 2 == 0 {
            Self::Even
        } else {
            Self::Odd
        }
    }
}

impl fmt::Display for Parity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Even => write!(f, "even"),
            Self::Odd => write!(f, "odd"),
        }
    }
}

/// Alignment of a value to a power-of-two boundary.
///
/// ## Tier: T1
/// ## Dominant: ∂ (Boundary) — alignment IS a boundary property
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Alignment {
    /// Value is aligned to the given power-of-two.
    Aligned(u32),
    /// Value is not aligned.
    Unaligned,
}

impl Alignment {
    /// Returns true if aligned.
    #[must_use]
    pub const fn is_aligned(self) -> bool {
        matches!(self, Self::Aligned(_))
    }

    /// Returns the alignment power if aligned.
    #[must_use]
    pub const fn power(self) -> Option<u32> {
        match self {
            Self::Aligned(p) => Some(p),
            Self::Unaligned => None,
        }
    }
}

impl fmt::Display for Alignment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Aligned(p) => write!(f, "aligned(2^{})", p),
            Self::Unaligned => write!(f, "unaligned"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bit_count_basics() {
        let c = BitCount(5);
        assert_eq!(c.value(), 5);
        assert!(!c.is_zero());
        assert!(BitCount(0).is_zero());
        assert_eq!(format!("{c}"), "5 bits");
    }

    #[test]
    fn bit_count_ordering() {
        assert!(BitCount(3) < BitCount(5));
        assert!(BitCount(0) < BitCount(1));
    }

    #[test]
    fn bit_count_from_u32() {
        let c: BitCount = 42.into();
        assert_eq!(c.value(), 42);
    }

    #[test]
    fn bit_position_basics() {
        let p = BitPosition(7);
        assert_eq!(p.value(), 7);
        assert_eq!(p.as_mask_u64(), 128);
        assert_eq!(format!("{p}"), "bit[7]");
    }

    #[test]
    fn bit_position_mask() {
        assert_eq!(BitPosition(0).as_mask_u64(), 1);
        assert_eq!(BitPosition(63).as_mask_u64(), 1u64 << 63);
    }

    #[test]
    fn parity_basics() {
        assert!(Parity::Even.is_even());
        assert!(!Parity::Even.is_odd());
        assert!(Parity::Odd.is_odd());
        assert!(!Parity::Odd.is_even());
    }

    #[test]
    fn parity_from_popcount() {
        assert_eq!(Parity::from_popcount(0), Parity::Even);
        assert_eq!(Parity::from_popcount(1), Parity::Odd);
        assert_eq!(Parity::from_popcount(4), Parity::Even);
        assert_eq!(Parity::from_popcount(7), Parity::Odd);
    }

    #[test]
    fn parity_display() {
        assert_eq!(format!("{}", Parity::Even), "even");
        assert_eq!(format!("{}", Parity::Odd), "odd");
    }

    #[test]
    fn alignment_basics() {
        assert!(Alignment::Aligned(3).is_aligned());
        assert!(!Alignment::Unaligned.is_aligned());
        assert_eq!(Alignment::Aligned(4).power(), Some(4));
        assert_eq!(Alignment::Unaligned.power(), None);
    }

    #[test]
    fn alignment_display() {
        assert_eq!(format!("{}", Alignment::Aligned(2)), "aligned(2^2)");
        assert_eq!(format!("{}", Alignment::Unaligned), "unaligned");
    }

    #[test]
    fn properties_serde_roundtrip() {
        // BitCount
        let bc = BitCount(42);
        let json = serde_json::to_string(&bc);
        assert!(json.is_ok());
        let back: Result<BitCount, _> =
            serde_json::from_str(json.as_ref().map(|s| s.as_str()).unwrap_or(""));
        assert!(back.is_ok());

        // BitPosition
        let bp = BitPosition(7);
        let json = serde_json::to_string(&bp);
        assert!(json.is_ok());

        // Parity
        let p = Parity::Odd;
        let json = serde_json::to_string(&p);
        assert!(json.is_ok());
        let back: Result<Parity, _> =
            serde_json::from_str(json.as_ref().map(|s| s.as_str()).unwrap_or(""));
        assert!(back.is_ok());

        // Alignment
        let a = Alignment::Aligned(4);
        let json = serde_json::to_string(&a);
        assert!(json.is_ok());
        let back: Result<Alignment, _> =
            serde_json::from_str(json.as_ref().map(|s| s.as_str()).unwrap_or(""));
        assert!(back.is_ok());
    }
}
