//! T1: Tier — Primitive tier classification per the Codex.
//!
//! Codex II (CLASSIFY): Every type has a tier.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Primitive tier classification per the Codex.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Tier {
    /// T1: Universal primitive (sequence, mapping, recursion, state, void).
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
    #[must_use]
    pub fn transfer_multiplier(&self) -> f64 {
        match self {
            Self::T1Universal => 1.0,
            Self::T2Primitive => 0.9,
            Self::T2Composite => 0.7,
            Self::T3DomainSpecific => 0.4,
        }
    }
}

impl fmt::Display for Tier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::T1Universal => write!(f, "T1-Universal"),
            Self::T2Primitive => write!(f, "T2-Primitive"),
            Self::T2Composite => write!(f, "T2-Composite"),
            Self::T3DomainSpecific => write!(f, "T3-DomainSpecific"),
        }
    }
}

/// Convert Tier to numeric value (Codex I: QUANTIFY).
impl From<Tier> for u8 {
    fn from(tier: Tier) -> u8 {
        match tier {
            Tier::T1Universal => 1,
            Tier::T2Primitive => 2,
            Tier::T2Composite => 3,
            Tier::T3DomainSpecific => 4,
        }
    }
}

/// Convert numeric value back to Tier.
impl TryFrom<u8> for Tier {
    type Error = &'static str;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::T1Universal),
            2 => Ok(Self::T2Primitive),
            3 => Ok(Self::T2Composite),
            4 => Ok(Self::T3DomainSpecific),
            _ => Err("Invalid tier value: must be 1-4"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transfer_multipliers() {
        assert!((Tier::T1Universal.transfer_multiplier() - 1.0).abs() < f64::EPSILON);
        assert!((Tier::T2Primitive.transfer_multiplier() - 0.9).abs() < f64::EPSILON);
        assert!((Tier::T2Composite.transfer_multiplier() - 0.7).abs() < f64::EPSILON);
        assert!((Tier::T3DomainSpecific.transfer_multiplier() - 0.4).abs() < f64::EPSILON);
    }

    #[test]
    fn display() {
        assert_eq!(format!("{}", Tier::T1Universal), "T1-Universal");
        assert_eq!(format!("{}", Tier::T3DomainSpecific), "T3-DomainSpecific");
    }

    #[test]
    fn quantification() {
        assert_eq!(u8::from(Tier::T1Universal), 1);
        assert_eq!(u8::from(Tier::T2Primitive), 2);
        assert_eq!(u8::from(Tier::T2Composite), 3);
        assert_eq!(u8::from(Tier::T3DomainSpecific), 4);
    }

    #[test]
    fn try_from_round_trip() {
        assert_eq!(Tier::try_from(1), Ok(Tier::T1Universal));
        assert_eq!(Tier::try_from(4), Ok(Tier::T3DomainSpecific));
        assert!(Tier::try_from(0).is_err());
        assert!(Tier::try_from(5).is_err());
    }

    #[test]
    fn ordering() {
        assert!(Tier::T1Universal < Tier::T2Primitive);
        assert!(Tier::T2Primitive < Tier::T2Composite);
        assert!(Tier::T2Composite < Tier::T3DomainSpecific);
    }

    #[test]
    fn serde_round_trip() {
        let t = Tier::T2Primitive;
        let json = serde_json::to_string(&t).unwrap();
        let back: Tier = serde_json::from_str(&json).unwrap();
        assert_eq!(t, back);
    }
}
