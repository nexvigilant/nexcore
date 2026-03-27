//! HealthVector — 8-position CNS health encoding.

use core::fmt;

use crate::digit::{CnsDigit, Polarity, conjugate_pair};

/// Status of a conjugate pair.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConjugateStatus {
    /// Both digits on virtue side — healthy.
    Balanced,
    /// Digits differ significantly but same polarity side.
    Stressed,
    /// One virtue, one vice — internal contradiction.
    Broken,
    /// One or both void.
    Incomplete,
}

/// An 8-position health vector. Each position scores one law.
///
/// Position 0 = Law VIII (Sovereign Boundary)
/// Position 7 = Law I (True Measure)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HealthVector {
    /// Scores for each law position. Index 0 = Law VIII, index 7 = Law I.
    scores: [CnsDigit; 8],
}

/// Diagnostic report from a health vector analysis.
#[derive(Debug, Clone)]
pub struct HealthReport {
    /// The health vector.
    pub vector: HealthVector,
    /// Decimal value.
    pub decimal: u64,
    /// Percentage of Pleroma.
    pub pleroma_pct: f64,
    /// Digital root (governing law).
    pub governing_law: CnsDigit,
    /// Status of each conjugate pair.
    pub conjugate_pairs: [(CnsDigit, CnsDigit, ConjugateStatus); 4],
    /// Number of broken conjugate pairs.
    pub broken_pairs: usize,
    /// Number of digits in vice range.
    pub vice_count: usize,
    /// Number of digits in virtue range.
    pub virtue_count: usize,
}

impl HealthVector {
    /// Create from 8 scores. Order: [Law VIII score, Law VII score, ..., Law I score].
    pub fn new(scores: [CnsDigit; 8]) -> Self {
        Self { scores }
    }

    /// Create from 8 u8 values. Returns None if any value > 8.
    pub fn from_values(values: [u8; 8]) -> Option<Self> {
        let mut scores = [CnsDigit::Void; 8];
        for (i, &v) in values.iter().enumerate() {
            scores[i] = CnsDigit::from_value(v)?;
        }
        Some(Self { scores })
    }

    /// Get the score at a position (0=VIII, 7=I).
    pub fn score_at(&self, position: usize) -> CnsDigit {
        self.scores.get(position).copied().unwrap_or(CnsDigit::Void)
    }

    /// Convert to decimal.
    pub fn to_decimal(&self) -> u64 {
        let mut result: u64 = 0;
        let mut place: u64 = 1;
        for &d in &self.scores {
            result += u64::from(d.value()) * place;
            place = place.saturating_mul(crate::BASE);
        }
        result
    }

    /// Percentage of Pleroma.
    pub fn pleroma_pct(&self) -> f64 {
        self.to_decimal() as f64 / crate::PLEROMA as f64 * 100.0
    }

    /// Digital root in base 9 — the governing law.
    /// Uses the decimal value, not digit sum, for correct base-9 digital root.
    pub fn digital_root(&self) -> CnsDigit {
        let decimal = self.to_decimal();
        if decimal == 0 {
            return CnsDigit::Void;
        }
        let root = 1 + ((decimal - 1) % 8);
        CnsDigit::from_value(root as u8).unwrap_or(CnsDigit::Void)
    }

    /// Evaluate the status of a conjugate pair.
    fn evaluate_pair(a: CnsDigit, b: CnsDigit) -> ConjugateStatus {
        if a == CnsDigit::Void || b == CnsDigit::Void {
            return ConjugateStatus::Incomplete;
        }
        let a_virtue = a.is_virtue();
        let b_virtue = b.is_virtue();
        if a_virtue == b_virtue {
            let diff = if a.value() > b.value() {
                a.value() - b.value()
            } else {
                b.value() - a.value()
            };
            if diff >= 3 {
                ConjugateStatus::Stressed
            } else {
                ConjugateStatus::Balanced
            }
        } else {
            ConjugateStatus::Broken
        }
    }

    /// Analyze all 4 conjugate pairs.
    ///
    /// Pairs: I↔VIII (positions 7,0), II↔VII (positions 6,1), III↔VI (positions 5,2), IV↔V (positions 3,4... wait)
    /// Position mapping: 0=VIII, 1=VII, 2=VI, 3=V, 4=IV, 5=III, 6=II, 7=I
    /// Conjugate pairs by position: (7,0), (6,1), (5,2), (4,3)
    pub fn conjugate_pairs(&self) -> [(CnsDigit, CnsDigit, ConjugateStatus); 4] {
        let pairs = [
            (self.scores[7], self.scores[0]), // I ↔ VIII
            (self.scores[6], self.scores[1]), // II ↔ VII
            (self.scores[5], self.scores[2]), // III ↔ VI
            (self.scores[4], self.scores[3]), // IV ↔ V
        ];
        [
            (
                pairs[0].0,
                pairs[0].1,
                Self::evaluate_pair(pairs[0].0, pairs[0].1),
            ),
            (
                pairs[1].0,
                pairs[1].1,
                Self::evaluate_pair(pairs[1].0, pairs[1].1),
            ),
            (
                pairs[2].0,
                pairs[2].1,
                Self::evaluate_pair(pairs[2].0, pairs[2].1),
            ),
            (
                pairs[3].0,
                pairs[3].1,
                Self::evaluate_pair(pairs[3].0, pairs[3].1),
            ),
        ]
    }

    /// Count broken conjugate pairs.
    pub fn broken_pair_count(&self) -> usize {
        self.conjugate_pairs()
            .iter()
            .filter(|(_, _, s)| *s == ConjugateStatus::Broken)
            .count()
    }

    /// Count digits in vice range.
    pub fn vice_count(&self) -> usize {
        self.scores.iter().filter(|d| d.is_vice()).count()
    }

    /// Count digits in virtue range.
    pub fn virtue_count(&self) -> usize {
        self.scores.iter().filter(|d| d.is_virtue()).count()
    }

    /// Generate full health report.
    pub fn report(&self) -> HealthReport {
        let pairs = self.conjugate_pairs();
        HealthReport {
            vector: self.clone(),
            decimal: self.to_decimal(),
            pleroma_pct: self.pleroma_pct(),
            governing_law: self.digital_root(),
            conjugate_pairs: pairs,
            broken_pairs: pairs
                .iter()
                .filter(|(_, _, s)| *s == ConjugateStatus::Broken)
                .count(),
            vice_count: self.vice_count(),
            virtue_count: self.virtue_count(),
        }
    }

    /// Euclidean distance between two health vectors.
    pub fn distance(&self, other: &Self) -> f64 {
        let sum_sq: f64 = self
            .scores
            .iter()
            .zip(other.scores.iter())
            .map(|(a, b)| {
                let diff = f64::from(a.value()) - f64::from(b.value());
                diff * diff
            })
            .sum();
        sum_sq.sqrt()
    }

    /// Which law position has the weakest score?
    pub fn weakest_position(&self) -> usize {
        self.scores
            .iter()
            .enumerate()
            .min_by_key(|(_, d)| d.value())
            .map(|(i, _)| i)
            .unwrap_or(0)
    }

    /// Prescription: for each broken pair, recommend strengthening the conjugate.
    pub fn prescriptions(&self) -> Vec<(CnsDigit, CnsDigit, &'static str)> {
        let mut rx = Vec::new();
        let pairs = self.conjugate_pairs();
        let pair_laws = [
            (CnsDigit::I, CnsDigit::VIII),
            (CnsDigit::II, CnsDigit::VII),
            (CnsDigit::III, CnsDigit::VI),
            (CnsDigit::IV, CnsDigit::V),
        ];
        for (i, (a_score, b_score, status)) in pairs.iter().enumerate() {
            if *status == ConjugateStatus::Broken {
                let (law_a, law_b) = pair_laws[i];
                if a_score.is_vice() {
                    rx.push((law_a, conjugate_pair(law_a), law_b.law_name()));
                } else {
                    rx.push((law_b, conjugate_pair(law_b), law_a.law_name()));
                }
            }
        }
        rx
    }
}

impl fmt::Display for HealthVector {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Display as I-score.II-score...VIII-score (position 7 first = Law I)
        for i in (0..8).rev() {
            if i < 7 {
                write!(f, ".")?;
            }
            write!(f, "{}", self.scores[i])?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn perfect_health() {
        let v = HealthVector::from_values([8, 8, 8, 8, 8, 8, 8, 8]);
        let v = v.unwrap_or_else(|| HealthVector::new([CnsDigit::Void; 8]));
        assert_eq!(v.to_decimal(), crate::PLEROMA);
        assert_eq!(v.digital_root(), CnsDigit::VIII); // base-9 digital root of Pleroma
        assert_eq!(v.vice_count(), 8); // VIII is in vice range by polarity
    }

    #[test]
    fn dead_system() {
        let v = HealthVector::new([CnsDigit::Void; 8]);
        assert_eq!(v.to_decimal(), 0);
        assert_eq!(v.digital_root(), CnsDigit::Void);
        assert_eq!(v.vice_count(), 0);
        assert_eq!(v.virtue_count(), 0);
    }

    #[test]
    fn all_virtue_no_broken_pairs() {
        // All scores III (strong virtue)
        let v = HealthVector::new([CnsDigit::III; 8]);
        assert_eq!(v.broken_pair_count(), 0);
        assert_eq!(v.virtue_count(), 8);
        assert_eq!(v.vice_count(), 0);
    }

    #[test]
    fn broken_pair_detection() {
        // Position 7 (Law I) = V (vice), Position 0 (Law VIII) = II (virtue) → broken
        let mut scores = [CnsDigit::III; 8];
        scores[7] = CnsDigit::V; // Law I in vice
        scores[0] = CnsDigit::II; // Law VIII in virtue
        let v = HealthVector::new(scores);
        assert_eq!(v.broken_pair_count(), 1);
    }

    #[test]
    fn distance_to_self_is_zero() {
        let v = HealthVector::new([CnsDigit::IV; 8]);
        assert!((v.distance(&v)).abs() < 1e-10);
    }

    #[test]
    fn distance_symmetry() {
        let a = HealthVector::new([CnsDigit::I; 8]);
        let b = HealthVector::new([CnsDigit::VIII; 8]);
        assert!((a.distance(&b) - b.distance(&a)).abs() < 1e-10);
    }

    #[test]
    fn pleroma_pct_bounds() {
        let zero = HealthVector::new([CnsDigit::Void; 8]);
        assert!(zero.pleroma_pct() < 0.01);

        let full = HealthVector::from_values([8; 8]);
        let full = full.unwrap_or_else(|| HealthVector::new([CnsDigit::Void; 8]));
        assert!((full.pleroma_pct() - 100.0).abs() < 0.01);
    }

    #[test]
    fn report_generation() {
        let v = HealthVector::from_values([4, 4, 3, 3, 4, 3, 4, 4]);
        let v = v.unwrap_or_else(|| HealthVector::new([CnsDigit::Void; 8]));
        let report = v.report();
        assert_eq!(report.broken_pairs, 0);
        assert_eq!(report.vice_count, 0);
        assert!(report.pleroma_pct > 0.0);
    }

    #[test]
    fn from_values_rejects_invalid() {
        assert!(HealthVector::from_values([0, 1, 2, 3, 4, 5, 6, 9]).is_none());
    }

    #[test]
    fn weakest_position() {
        let mut scores = [CnsDigit::IV; 8];
        scores[3] = CnsDigit::I; // Weakest at position 3 (Law V)
        let v = HealthVector::new(scores);
        assert_eq!(v.weakest_position(), 3);
    }
}
