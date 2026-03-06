//! Primitive State Vector for gap analysis.
//!
//! A 15-dimensional score vector (one per PV domain) supporting
//! current vs. desired state comparison for CCCP Phase 2 assessment.
//! (source: 02-boundary-state-framework.md, State Assessment Matrix)

use crate::caba::Score;
use crate::caba::domain::DomainCategory;
use crate::caba::proficiency::ProficiencyLevel;
use serde::{Deserialize, Serialize};

/// Number of PV domains.
pub const DOMAIN_COUNT: usize = 15;

/// A 15-dimensional proficiency vector — one level per PV domain.
///
/// Used to represent current state, desired state, or gap between them.
/// Domain index = domain number - 1 (D01 = index 0, D15 = index 14).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DomainStateVector([ProficiencyLevel; DOMAIN_COUNT]);

impl DomainStateVector {
    /// Create a new state vector with all domains at L1 Novice.
    #[must_use]
    pub fn novice() -> Self {
        Self([ProficiencyLevel::L1Novice; DOMAIN_COUNT])
    }

    /// Create from an array of proficiency levels.
    #[must_use]
    pub fn new(levels: [ProficiencyLevel; DOMAIN_COUNT]) -> Self {
        Self(levels)
    }

    /// Get proficiency level for a domain.
    #[must_use]
    pub fn get(&self, domain: DomainCategory) -> ProficiencyLevel {
        self.0[domain.number() as usize - 1]
    }

    /// Set proficiency level for a domain.
    pub fn set(&mut self, domain: DomainCategory, level: ProficiencyLevel) {
        self.0[domain.number() as usize - 1] = level;
    }

    /// Compute the gap vector: desired - current.
    ///
    /// Positive values = growth needed. Zero = met. Negative = exceeds.
    /// Returns signed gap per domain (desired.numeric - current.numeric).
    #[must_use]
    pub fn gap_from(&self, desired: &Self) -> [i8; DOMAIN_COUNT] {
        let mut gap = [0i8; DOMAIN_COUNT];
        for i in 0..DOMAIN_COUNT {
            gap[i] = desired.0[i].numeric_value() as i8 - self.0[i].numeric_value() as i8;
        }
        gap
    }

    /// Count domains where current meets or exceeds desired.
    #[must_use]
    pub fn domains_met(&self, desired: &Self) -> usize {
        self.gap_from(desired).iter().filter(|g| **g <= 0).count()
    }

    /// Count domains with a gap (desired > current).
    #[must_use]
    pub fn domains_with_gap(&self, desired: &Self) -> usize {
        DOMAIN_COUNT - self.domains_met(desired)
    }

    /// Compute overall readiness as fraction of domains met.
    ///
    /// # Errors
    /// Returns error if the computed score is somehow out of bounds (should not happen).
    pub fn readiness_score(&self, desired: &Self) -> Result<Score, crate::caba::ScoreError> {
        Score::new(self.domains_met(desired) as f64 / DOMAIN_COUNT as f64)
    }

    /// Get the inner array.
    #[must_use]
    pub fn as_array(&self) -> &[ProficiencyLevel; DOMAIN_COUNT] {
        &self.0
    }
}

impl Default for DomainStateVector {
    fn default() -> Self {
        Self::novice()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_novice_vector() {
        let v = DomainStateVector::novice();
        for i in 0..DOMAIN_COUNT {
            assert_eq!(v.0[i], ProficiencyLevel::L1Novice);
        }
    }

    #[test]
    fn test_get_set() {
        let mut v = DomainStateVector::novice();
        v.set(
            DomainCategory::D05SignalDetection,
            ProficiencyLevel::L4Proficient,
        );
        assert_eq!(
            v.get(DomainCategory::D05SignalDetection),
            ProficiencyLevel::L4Proficient
        );
        assert_eq!(
            v.get(DomainCategory::D01PvFoundations),
            ProficiencyLevel::L1Novice
        );
    }

    #[test]
    fn test_gap_computation() {
        let mut current = DomainStateVector::novice();
        let mut desired = DomainStateVector::novice();

        current.set(
            DomainCategory::D05SignalDetection,
            ProficiencyLevel::L2AdvancedBeginner,
        );
        desired.set(
            DomainCategory::D05SignalDetection,
            ProficiencyLevel::L4Proficient,
        );

        let gap = current.gap_from(&desired);
        // D05 is index 4, gap = 4 - 2 = 2
        assert_eq!(gap[4], 2);
        // All other domains: desired L1 - current L1 = 0
        assert_eq!(gap[0], 0);
    }

    #[test]
    fn test_domains_met_all() {
        let current = DomainStateVector::novice();
        let desired = DomainStateVector::novice();
        assert_eq!(current.domains_met(&desired), 15);
        assert_eq!(current.domains_with_gap(&desired), 0);
    }

    #[test]
    fn test_domains_met_partial() {
        let current = DomainStateVector::novice();
        let mut desired = DomainStateVector::novice();
        desired.set(
            DomainCategory::D04IcsrProcessing,
            ProficiencyLevel::L3Competent,
        );
        desired.set(
            DomainCategory::D05SignalDetection,
            ProficiencyLevel::L3Competent,
        );

        assert_eq!(current.domains_met(&desired), 13);
        assert_eq!(current.domains_with_gap(&desired), 2);
    }

    #[test]
    fn test_readiness_score() {
        let current = DomainStateVector::novice();
        let desired = DomainStateVector::novice();
        let score = current.readiness_score(&desired).expect("valid score");
        assert!((score.value() - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_exceeds_desired() {
        let mut current = DomainStateVector::novice();
        current.set(DomainCategory::D01PvFoundations, ProficiencyLevel::L5Expert);
        let desired = DomainStateVector::novice();

        let gap = current.gap_from(&desired);
        // D01 index 0: desired L1(1) - current L5(5) = -4
        assert_eq!(gap[0], -4);
        // Still counts as "met"
        assert_eq!(current.domains_met(&desired), 15);
    }
}
