//! CnsDigit — the 9 symbols of the Crystalbook Numeral System.

use core::fmt;

/// The 9 symbols: ∅ + 8 Laws.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum CnsDigit {
    /// ∅ — Void. Measurable absence of law.
    Void = 0,
    /// I — True Measure. Vice: Pride. Virtue: Humility.
    I = 1,
    /// II — Sufficient Portion. Vice: Greed. Virtue: Charity.
    II = 2,
    /// III — Bounded Pursuit. Vice: Lust. Virtue: Chastity.
    III = 3,
    /// IV — Generous Witness. Vice: Envy. Virtue: Kindness.
    IV = 4,
    /// V — Measured Intake. Vice: Gluttony. Virtue: Temperance.
    V = 5,
    /// VI — Measured Response. Vice: Wrath. Virtue: Patience.
    VI = 6,
    /// VII — Active Maintenance. Vice: Sloth. Virtue: Diligence.
    VII = 7,
    /// VIII — Sovereign Boundary. Vice: Corruption. Virtue: Independence.
    VIII = 8,
}

/// Vice/virtue polarity of a digit value within a health vector position.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Polarity {
    /// No law present.
    Absent,
    /// Virtue, weak (I-II).
    VirtueWeak,
    /// Virtue, strong (III-IV).
    VirtueStrong,
    /// Vice, moderate (V-VI).
    ViceModerate,
    /// Vice, severe (VII-VIII).
    ViceSevere,
}

impl CnsDigit {
    /// All 9 digits in order.
    pub const ALL: [CnsDigit; 9] = [
        Self::Void,
        Self::I,
        Self::II,
        Self::III,
        Self::IV,
        Self::V,
        Self::VI,
        Self::VII,
        Self::VIII,
    ];

    /// Create from a u8 value (0-8). Returns None if out of range.
    pub fn from_value(v: u8) -> Option<Self> {
        match v {
            0 => Some(Self::Void),
            1 => Some(Self::I),
            2 => Some(Self::II),
            3 => Some(Self::III),
            4 => Some(Self::IV),
            5 => Some(Self::V),
            6 => Some(Self::VI),
            7 => Some(Self::VII),
            8 => Some(Self::VIII),
            _ => None,
        }
    }

    /// Numeric value (0-8).
    pub fn value(self) -> u8 {
        self as u8
    }

    /// The law name.
    pub fn law_name(self) -> &'static str {
        match self {
            Self::Void => "Void",
            Self::I => "True Measure",
            Self::II => "Sufficient Portion",
            Self::III => "Bounded Pursuit",
            Self::IV => "Generous Witness",
            Self::V => "Measured Intake",
            Self::VI => "Measured Response",
            Self::VII => "Active Maintenance",
            Self::VIII => "Sovereign Boundary",
        }
    }

    /// The vice associated with this law.
    pub fn vice(self) -> &'static str {
        match self {
            Self::Void => "—",
            Self::I => "Pride",
            Self::II => "Greed",
            Self::III => "Lust",
            Self::IV => "Envy",
            Self::V => "Gluttony",
            Self::VI => "Wrath",
            Self::VII => "Sloth",
            Self::VIII => "Corruption",
        }
    }

    /// The virtue associated with this law.
    pub fn virtue(self) -> &'static str {
        match self {
            Self::Void => "—",
            Self::I => "Humility",
            Self::II => "Charity",
            Self::III => "Chastity",
            Self::IV => "Kindness",
            Self::V => "Temperance",
            Self::VI => "Patience",
            Self::VII => "Diligence",
            Self::VIII => "Independence",
        }
    }

    /// Vice/virtue polarity when this digit appears as a score.
    pub fn polarity(self) -> Polarity {
        match self {
            Self::Void => Polarity::Absent,
            Self::I | Self::II => Polarity::VirtueWeak,
            Self::III | Self::IV => Polarity::VirtueStrong,
            Self::V | Self::VI => Polarity::ViceModerate,
            Self::VII | Self::VIII => Polarity::ViceSevere,
        }
    }

    /// Is this digit in virtue range (I-IV)?
    pub fn is_virtue(self) -> bool {
        matches!(
            self.polarity(),
            Polarity::VirtueWeak | Polarity::VirtueStrong
        )
    }

    /// Is this digit in vice range (V-VIII)?
    pub fn is_vice(self) -> bool {
        matches!(
            self.polarity(),
            Polarity::ViceModerate | Polarity::ViceSevere
        )
    }

    /// Addition mod 9 (Z₉ group operation).
    pub fn add(self, other: Self) -> Self {
        let sum = (self.value() + other.value()) % 9;
        // SAFETY: sum is always 0-8 since both inputs are 0-8 and we mod 9
        Self::from_value(sum).unwrap_or(Self::Void)
    }

    /// Multiplication mod 9.
    pub fn mul(self, other: Self) -> Self {
        let product = (u16::from(self.value()) * u16::from(other.value())) % 9;
        Self::from_value(product as u8).unwrap_or(Self::Void)
    }

    /// Additive inverse in Z₉. The conjugate pair partner.
    pub fn inverse(self) -> Self {
        if self == Self::Void {
            return Self::Void;
        }
        let inv = 9 - self.value();
        Self::from_value(inv).unwrap_or(Self::Void)
    }
}

/// Get the conjugate pair for a law (additive inverse in Z₉).
///
/// I ↔ VIII (Measure ↔ Sovereignty)
/// II ↔ VII (Portion ↔ Maintenance)
/// III ↔ VI (Pursuit ↔ Response)
/// IV ↔ V (Witness ↔ Intake)
pub fn conjugate_pair(digit: CnsDigit) -> CnsDigit {
    digit.inverse()
}

/// Check if two digits form a conjugate pair.
pub fn is_conjugate(a: CnsDigit, b: CnsDigit) -> bool {
    a.inverse() == b
}

impl fmt::Display for CnsDigit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Void => write!(f, "∅"),
            Self::I => write!(f, "I"),
            Self::II => write!(f, "II"),
            Self::III => write!(f, "III"),
            Self::IV => write!(f, "IV"),
            Self::V => write!(f, "V"),
            Self::VI => write!(f, "VI"),
            Self::VII => write!(f, "VII"),
            Self::VIII => write!(f, "VIII"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_digits_roundtrip() {
        for d in CnsDigit::ALL {
            assert_eq!(CnsDigit::from_value(d.value()), Some(d));
        }
    }

    #[test]
    fn out_of_range_returns_none() {
        assert_eq!(CnsDigit::from_value(9), None);
        assert_eq!(CnsDigit::from_value(255), None);
    }

    #[test]
    fn void_is_additive_identity() {
        for d in CnsDigit::ALL {
            assert_eq!(d.add(CnsDigit::Void), d);
            assert_eq!(CnsDigit::Void.add(d), d);
        }
    }

    #[test]
    fn viii_plus_i_equals_void() {
        // The conservation cycle
        assert_eq!(CnsDigit::VIII.add(CnsDigit::I), CnsDigit::Void);
    }

    #[test]
    fn conjugate_pairs_sum_to_void() {
        assert_eq!(CnsDigit::I.add(CnsDigit::VIII), CnsDigit::Void);
        assert_eq!(CnsDigit::II.add(CnsDigit::VII), CnsDigit::Void);
        assert_eq!(CnsDigit::III.add(CnsDigit::VI), CnsDigit::Void);
        assert_eq!(CnsDigit::IV.add(CnsDigit::V), CnsDigit::Void);
    }

    #[test]
    fn conjugate_pair_symmetry() {
        for d in CnsDigit::ALL {
            assert!(is_conjugate(d, conjugate_pair(d)));
            assert_eq!(conjugate_pair(conjugate_pair(d)), d);
        }
    }

    #[test]
    fn iii_times_iii_annihilates() {
        // Pursuit applied to itself = void
        assert_eq!(CnsDigit::III.mul(CnsDigit::III), CnsDigit::Void);
    }

    #[test]
    fn vi_times_vi_annihilates() {
        // Response applied to itself = void
        assert_eq!(CnsDigit::VI.mul(CnsDigit::VI), CnsDigit::Void);
    }

    #[test]
    fn iv_times_iv_generates_vii() {
        // Witness applied to itself generates maintenance
        assert_eq!(CnsDigit::IV.mul(CnsDigit::IV), CnsDigit::VII);
    }

    #[test]
    fn void_annihilates_multiplication() {
        for d in CnsDigit::ALL {
            assert_eq!(CnsDigit::Void.mul(d), CnsDigit::Void);
            assert_eq!(d.mul(CnsDigit::Void), CnsDigit::Void);
        }
    }

    #[test]
    fn i_is_multiplicative_identity() {
        for d in CnsDigit::ALL {
            assert_eq!(CnsDigit::I.mul(d), d);
            assert_eq!(d.mul(CnsDigit::I), d);
        }
    }

    #[test]
    fn polarity_ranges() {
        assert_eq!(CnsDigit::Void.polarity(), Polarity::Absent);
        assert_eq!(CnsDigit::I.polarity(), Polarity::VirtueWeak);
        assert_eq!(CnsDigit::II.polarity(), Polarity::VirtueWeak);
        assert_eq!(CnsDigit::III.polarity(), Polarity::VirtueStrong);
        assert_eq!(CnsDigit::IV.polarity(), Polarity::VirtueStrong);
        assert_eq!(CnsDigit::V.polarity(), Polarity::ViceModerate);
        assert_eq!(CnsDigit::VI.polarity(), Polarity::ViceModerate);
        assert_eq!(CnsDigit::VII.polarity(), Polarity::ViceSevere);
        assert_eq!(CnsDigit::VIII.polarity(), Polarity::ViceSevere);
    }

    #[test]
    fn virtue_vice_classification() {
        assert!(!CnsDigit::Void.is_virtue());
        assert!(!CnsDigit::Void.is_vice());
        assert!(CnsDigit::I.is_virtue());
        assert!(CnsDigit::IV.is_virtue());
        assert!(!CnsDigit::IV.is_vice());
        assert!(CnsDigit::V.is_vice());
        assert!(CnsDigit::VIII.is_vice());
        assert!(!CnsDigit::VIII.is_virtue());
    }

    #[test]
    fn addition_is_commutative() {
        for a in CnsDigit::ALL {
            for b in CnsDigit::ALL {
                assert_eq!(a.add(b), b.add(a));
            }
        }
    }

    #[test]
    fn addition_is_associative() {
        for a in CnsDigit::ALL {
            for b in CnsDigit::ALL {
                for c in CnsDigit::ALL {
                    assert_eq!(a.add(b).add(c), a.add(b.add(c)));
                }
            }
        }
    }

    #[test]
    fn multiplication_is_commutative() {
        for a in CnsDigit::ALL {
            for b in CnsDigit::ALL {
                assert_eq!(a.mul(b), b.mul(a));
            }
        }
    }

    #[test]
    fn every_element_has_inverse() {
        for d in CnsDigit::ALL {
            assert_eq!(d.add(d.inverse()), CnsDigit::Void);
        }
    }

    #[test]
    fn display_format() {
        assert_eq!(format!("{}", CnsDigit::Void), "∅");
        assert_eq!(format!("{}", CnsDigit::I), "I");
        assert_eq!(format!("{}", CnsDigit::VIII), "VIII");
    }
}
