//! CnsNumber — multi-digit base-9 Crystalbook numeral.

use core::fmt;

use crate::BASE;
use crate::digit::CnsDigit;

/// A multi-digit CNS number. Digits stored least-significant first (position 0 = index 0).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CnsNumber {
    /// Digits, least-significant first. Position 0 (VIII) is index 0.
    digits: Vec<CnsDigit>,
}

impl CnsNumber {
    /// Create from a slice of digits (most-significant first, as written).
    pub fn from_digits(digits: &[CnsDigit]) -> Self {
        let mut reversed: Vec<CnsDigit> = digits.iter().copied().rev().collect();
        // Strip leading zeros (trailing in our internal repr)
        while reversed.len() > 1 && reversed.last() == Some(&CnsDigit::Void) {
            reversed.pop();
        }
        Self { digits: reversed }
    }

    /// Create from a decimal u64.
    pub fn from_decimal(mut n: u64) -> Self {
        if n == 0 {
            return Self {
                digits: vec![CnsDigit::Void],
            };
        }
        let mut digits = Vec::new();
        while n > 0 {
            let remainder = (n % BASE) as u8;
            digits.push(CnsDigit::from_value(remainder).unwrap_or(CnsDigit::Void));
            n /= BASE;
        }
        Self { digits }
    }

    /// Convert to decimal u64.
    pub fn to_decimal(&self) -> u64 {
        let mut result: u64 = 0;
        let mut place: u64 = 1;
        for &d in &self.digits {
            result += u64::from(d.value()) * place;
            place = place.saturating_mul(BASE);
        }
        result
    }

    /// Number of digits.
    pub fn len(&self) -> usize {
        self.digits.len()
    }

    /// Is this the void (zero)?
    pub fn is_void(&self) -> bool {
        self.digits.len() == 1 && self.digits[0] == CnsDigit::Void
    }

    /// Get digit at position n (0 = ones/VIII, 1 = nines/VII, etc).
    pub fn digit_at(&self, position: usize) -> CnsDigit {
        self.digits.get(position).copied().unwrap_or(CnsDigit::Void)
    }

    /// Get all digits, most-significant first (as written).
    pub fn digits_msb(&self) -> Vec<CnsDigit> {
        self.digits.iter().copied().rev().collect()
    }

    /// Digital root in base 9 (repeated base-9 digit sum until single digit).
    /// Formula: 1 + ((n - 1) mod 8) for n > 0, Void for n = 0.
    /// Returns the governing law of this number.
    pub fn digital_root(&self) -> CnsDigit {
        let decimal = self.to_decimal();
        if decimal == 0 {
            return CnsDigit::Void;
        }
        let root = 1 + ((decimal - 1) % 8);
        CnsDigit::from_value(root as u8).unwrap_or(CnsDigit::Void)
    }

    /// Digit sum (not reduced).
    pub fn digit_sum(&self) -> u64 {
        self.digits.iter().map(|d| u64::from(d.value())).sum()
    }

    /// Percentage of the Pleroma (max 8-digit CNS number).
    pub fn pleroma_ratio(&self) -> f64 {
        self.to_decimal() as f64 / crate::PLEROMA as f64
    }

    /// Count of digits in vice range (V-VIII).
    pub fn vice_count(&self) -> usize {
        self.digits.iter().filter(|d| d.is_vice()).count()
    }

    /// Count of digits in virtue range (I-IV).
    pub fn virtue_count(&self) -> usize {
        self.digits.iter().filter(|d| d.is_virtue()).count()
    }

    /// Count of void digits.
    pub fn void_count(&self) -> usize {
        self.digits.iter().filter(|&&d| d == CnsDigit::Void).count()
    }

    /// The law that governs position n.
    pub fn position_law(position: usize) -> Option<CnsDigit> {
        // Position 0 = VIII, 1 = VII, ..., 7 = I
        match position {
            0 => Some(CnsDigit::VIII),
            1 => Some(CnsDigit::VII),
            2 => Some(CnsDigit::VI),
            3 => Some(CnsDigit::V),
            4 => Some(CnsDigit::IV),
            5 => Some(CnsDigit::III),
            6 => Some(CnsDigit::II),
            7 => Some(CnsDigit::I),
            _ => None,
        }
    }
}

impl fmt::Display for CnsNumber {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let msb = self.digits_msb();
        for (i, d) in msb.iter().enumerate() {
            if i > 0 {
                write!(f, ".")?;
            }
            write!(f, "{d}")?;
        }
        Ok(())
    }
}

impl From<u64> for CnsNumber {
    fn from(n: u64) -> Self {
        Self::from_decimal(n)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_is_void() {
        let n = CnsNumber::from_decimal(0);
        assert!(n.is_void());
        assert_eq!(n.to_decimal(), 0);
        assert_eq!(format!("{n}"), "∅");
    }

    #[test]
    fn single_digits() {
        for v in 0..=8u8 {
            let d = CnsDigit::from_value(v).unwrap_or(CnsDigit::Void);
            let n = CnsNumber::from_decimal(u64::from(v));
            assert_eq!(n.to_decimal(), u64::from(v));
            assert_eq!(n.digit_at(0), d);
        }
    }

    #[test]
    fn staircase_constant() {
        let n = CnsNumber::from_decimal(crate::STAIRCASE);
        assert_eq!(n.to_decimal(), 42_374_116);
        assert_eq!(n.digital_root(), CnsDigit::IV); // base-9 digital root
    }

    #[test]
    fn pleroma_constant() {
        let n = CnsNumber::from_decimal(crate::PLEROMA);
        assert_eq!(n.to_decimal(), 43_046_720);
        assert_eq!(n.digital_root(), CnsDigit::VIII); // base-9 digital root
    }

    #[test]
    fn mirror_constant() {
        let n = CnsNumber::from_decimal(crate::MIRROR);
        assert_eq!(n.to_decimal(), 6_053_444);
    }

    #[test]
    fn gap_constant() {
        let n = CnsNumber::from_decimal(crate::GAP);
        assert_eq!(n.to_decimal(), 672_604);
        assert_eq!(n.digital_root(), CnsDigit::IV); // base-9 digital root
    }

    #[test]
    fn staircase_plus_mirror_exceeds_pleroma() {
        // Arrow of time: you must choose a direction
        assert!(crate::STAIRCASE + crate::MIRROR > crate::PLEROMA);
    }

    #[test]
    fn roundtrip_decimal() {
        let values = [0, 1, 8, 9, 80, 604, 43_046_720, 42_374_116, 6_053_444];
        for &v in &values {
            assert_eq!(CnsNumber::from_decimal(v).to_decimal(), v);
        }
    }

    #[test]
    fn seven_four_one() {
        // The originating number: 7×81 + 4×9 + 1 = 604
        let n = CnsNumber::from_digits(&[CnsDigit::VII, CnsDigit::IV, CnsDigit::I]);
        assert_eq!(n.to_decimal(), 604);
        assert_eq!(n.digital_root(), CnsDigit::IV); // base-9 digital root: 1+((604-1)%8) = 4
        assert_eq!(format!("{n}"), "VII.IV.I");
    }

    #[test]
    fn digital_root_base_9() {
        // Base-9 digital root: 1 + ((n-1) % 8) for n > 0
        for v in 1..=100u64 {
            let n = CnsNumber::from_decimal(v);
            let expected = 1 + ((v - 1) % 8);
            let root_val = u64::from(n.digital_root().value());
            assert_eq!(root_val, expected, "failed for {v}");
        }
        // Zero maps to Void
        assert_eq!(CnsNumber::from_decimal(0).digital_root(), CnsDigit::Void);
    }

    #[test]
    fn position_laws() {
        assert_eq!(CnsNumber::position_law(0), Some(CnsDigit::VIII));
        assert_eq!(CnsNumber::position_law(1), Some(CnsDigit::VII));
        assert_eq!(CnsNumber::position_law(7), Some(CnsDigit::I));
        assert_eq!(CnsNumber::position_law(8), None);
    }

    #[test]
    fn vice_virtue_counting() {
        // VII.IV.I = vice(VII) + virtue(IV) + virtue(I)
        let n = CnsNumber::from_digits(&[CnsDigit::VII, CnsDigit::IV, CnsDigit::I]);
        assert_eq!(n.vice_count(), 1);
        assert_eq!(n.virtue_count(), 2);
        assert_eq!(n.void_count(), 0);
    }

    #[test]
    fn pleroma_ratio() {
        let n = CnsNumber::from_decimal(crate::PLEROMA);
        let ratio = n.pleroma_ratio();
        assert!((ratio - 1.0).abs() < 1e-10);

        let zero = CnsNumber::from_decimal(0);
        assert!((zero.pleroma_ratio()).abs() < 1e-10);
    }

    #[test]
    fn from_u64_trait() {
        let n: CnsNumber = 604u64.into();
        assert_eq!(n.to_decimal(), 604);
    }
}
