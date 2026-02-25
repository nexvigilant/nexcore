//! # Tier Classification
//!
//! The 4-tier system for primitive classification.
//!
//! - **T1**: Universal (the 16 Lex Primitiva)
//! - **T2-P**: Cross-domain primitive (newtypes over T1)
//! - **T2-C**: Cross-domain composite (combinations with traits)
//! - **T3**: Domain-specific (full domain logic)

use crate::primitiva::PrimitiveComposition;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Primitive tier classification per the Codex.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[repr(u8)]
#[non_exhaustive]
pub enum Tier {
    /// T1: Universal primitive (sequence, mapping, recursion, state, void, etc).
    T1Universal = 1,
    /// T2-P: Cross-domain primitive (reusable across 2+ domains).
    T2Primitive = 2,
    /// T2-C: Cross-domain composite (built from T2-P primitives).
    T2Composite = 3,
    /// T3: Domain-specific (only meaningful in one domain).
    T3DomainSpecific = 4,
}

impl Tier {
    /// Get the transfer confidence multiplier for this tier.
    ///
    /// Higher tiers have lower transfer confidence when applied cross-domain.
    #[must_use]
    pub const fn transfer_multiplier(&self) -> f64 {
        match self {
            Self::T1Universal => 1.0,
            Self::T2Primitive => 0.9,
            Self::T2Composite => 0.7,
            Self::T3DomainSpecific => 0.4,
        }
    }

    /// Get the short code for this tier.
    #[must_use]
    pub const fn code(&self) -> &'static str {
        match self {
            Self::T1Universal => "T1",
            Self::T2Primitive => "T2-P",
            Self::T2Composite => "T2-C",
            Self::T3DomainSpecific => "T3",
        }
    }

    /// Get the full name of this tier.
    #[must_use]
    pub const fn full_name(&self) -> &'static str {
        match self {
            Self::T1Universal => "T1-Universal",
            Self::T2Primitive => "T2-Primitive (Cross-Domain)",
            Self::T2Composite => "T2-Composite (Cross-Domain)",
            Self::T3DomainSpecific => "T3-Domain Specific",
        }
    }

    /// Returns all tiers in order.
    #[must_use]
    pub const fn all() -> [Self; 4] {
        [
            Self::T1Universal,
            Self::T2Primitive,
            Self::T2Composite,
            Self::T3DomainSpecific,
        ]
    }

    /// Classify a composition into a tier based on unique primitive count.
    #[must_use]
    pub fn classify(composition: &PrimitiveComposition) -> Self {
        match composition.unique().len() {
            0..=1 => Self::T1Universal,
            2..=3 => Self::T2Primitive,
            4..=5 => Self::T2Composite,
            _ => Self::T3DomainSpecific,
        }
    }

    /// Convert from numeric value.
    #[must_use]
    pub const fn from_u8(value: u8) -> Option<Self> {
        match value {
            1 => Some(Self::T1Universal),
            2 => Some(Self::T2Primitive),
            3 => Some(Self::T2Composite),
            4 => Some(Self::T3DomainSpecific),
            _ => None,
        }
    }
}

impl fmt::Display for Tier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.code())
    }
}

impl From<Tier> for u8 {
    #[allow(
        clippy::as_conversions,
        reason = "Tier has #[repr(u8)]; cast to u8 is defined and safe"
    )]
    fn from(tier: Tier) -> u8 {
        tier as u8
    }
}

impl TryFrom<u8> for Tier {
    type Error = &'static str;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Self::from_u8(value).ok_or("Invalid tier value: must be 1-4")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::primitiva::LexPrimitiva;

    #[test]
    fn test_transfer_multipliers() {
        assert!((Tier::T1Universal.transfer_multiplier() - 1.0).abs() < f64::EPSILON);
        assert!((Tier::T2Primitive.transfer_multiplier() - 0.9).abs() < f64::EPSILON);
        assert!((Tier::T2Composite.transfer_multiplier() - 0.7).abs() < f64::EPSILON);
        assert!((Tier::T3DomainSpecific.transfer_multiplier() - 0.4).abs() < f64::EPSILON);
    }

    #[test]
    fn test_tier_ordering() {
        assert!(Tier::T1Universal < Tier::T2Primitive);
        assert!(Tier::T2Primitive < Tier::T2Composite);
        assert!(Tier::T2Composite < Tier::T3DomainSpecific);
    }

    #[test]
    fn test_classification() {
        let t1 = PrimitiveComposition::new(vec![LexPrimitiva::Quantity]);
        assert_eq!(Tier::classify(&t1), Tier::T1Universal);

        let t2p = PrimitiveComposition::new(vec![LexPrimitiva::Sequence, LexPrimitiva::Mapping]);
        assert_eq!(Tier::classify(&t2p), Tier::T2Primitive);

        let t2c = PrimitiveComposition::new(vec![
            LexPrimitiva::Sequence,
            LexPrimitiva::Mapping,
            LexPrimitiva::State,
            LexPrimitiva::Boundary,
        ]);
        assert_eq!(Tier::classify(&t2c), Tier::T2Composite);
    }

    #[test]
    fn test_from_u8() {
        assert_eq!(Tier::from_u8(1), Some(Tier::T1Universal));
        assert_eq!(Tier::from_u8(4), Some(Tier::T3DomainSpecific));
        assert_eq!(Tier::from_u8(0), None);
        assert_eq!(Tier::from_u8(5), None);
    }

    #[test]
    fn test_display() {
        assert_eq!(format!("{}", Tier::T1Universal), "T1");
        assert_eq!(format!("{}", Tier::T2Primitive), "T2-P");
    }
}
