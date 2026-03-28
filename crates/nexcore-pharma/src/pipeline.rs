//! Pipeline candidate types.
//!
//! Models drugs in clinical or preclinical development, their current
//! phase, mechanism of action, and therapeutic focus.

use std::fmt;

use serde::{Deserialize, Serialize};

use crate::TherapeuticArea;

/// A pharmaceutical pipeline candidate in development.
///
/// # Examples
///
/// ```
/// use nexcore_pharma::{PipelineCandidate, Phase, TherapeuticArea};
///
/// let candidate = PipelineCandidate {
///     name: "BNT323".to_string(),
///     mechanism: "HER2-directed ADC".to_string(),
///     phase: Phase::Phase3,
///     indication: "HER2+ breast cancer".to_string(),
///     therapeutic_area: TherapeuticArea::Oncology,
/// };
/// assert!(candidate.phase.is_clinical());
/// assert!(!candidate.phase.is_approved());
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineCandidate {
    /// Compound name or development code
    pub name: String,
    /// Mechanism of action description
    pub mechanism: String,
    /// Current development phase
    pub phase: Phase,
    /// Target indication
    pub indication: String,
    /// Therapeutic area classification
    pub therapeutic_area: TherapeuticArea,
}

/// Development phase of a pipeline candidate.
///
/// Ordered from earliest (`Preclinical`) to latest (`Approved`).
/// The `PartialOrd` / `Ord` implementations reflect this ordering,
/// enabling phase-progression comparisons.
///
/// # Examples
///
/// ```
/// use nexcore_pharma::Phase;
///
/// assert!(Phase::Phase1 < Phase::Phase3);
/// assert!(Phase::Phase3 < Phase::Filed);
/// assert!(Phase::Filed < Phase::Approved);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Phase {
    /// Pre-IND / animal studies
    Preclinical,
    /// First-in-human safety and dosing
    Phase1,
    /// Proof of concept / efficacy exploration
    Phase2,
    /// Pivotal trials for registration
    Phase3,
    /// Regulatory submission filed, awaiting approval
    Filed,
    /// Regulatory approval received
    Approved,
}

impl Phase {
    /// All phases in ascending development order.
    pub fn all() -> [Self; 6] {
        [
            Self::Preclinical,
            Self::Phase1,
            Self::Phase2,
            Self::Phase3,
            Self::Filed,
            Self::Approved,
        ]
    }

    /// Returns the numeric rank of this phase (Preclinical=0 … Approved=5).
    ///
    /// Used for ordering and comparison without requiring `Ord` derivation
    /// on enums that serde round-trips.
    pub fn rank(&self) -> u8 {
        match self {
            Self::Preclinical => 0,
            Self::Phase1 => 1,
            Self::Phase2 => 2,
            Self::Phase3 => 3,
            Self::Filed => 4,
            Self::Approved => 5,
        }
    }

    /// Returns `true` if the candidate is in active clinical development
    /// (Phase 1 through Phase 3).
    pub fn is_clinical(&self) -> bool {
        matches!(self, Self::Phase1 | Self::Phase2 | Self::Phase3)
    }

    /// Returns `true` if the candidate has received regulatory approval.
    pub fn is_approved(&self) -> bool {
        matches!(self, Self::Approved)
    }

    /// Returns `true` if the candidate is in late-stage development
    /// (Phase 3 or Filed).
    pub fn is_late_stage(&self) -> bool {
        matches!(self, Self::Phase3 | Self::Filed)
    }
}

impl PartialOrd for Phase {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Phase {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.rank().cmp(&other.rank())
    }
}

impl fmt::Display for Phase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Preclinical => "Preclinical",
            Self::Phase1 => "Phase 1",
            Self::Phase2 => "Phase 2",
            Self::Phase3 => "Phase 3",
            Self::Filed => "Filed",
            Self::Approved => "Approved",
        };
        f.write_str(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn phase_all_returns_6_variants() {
        assert_eq!(Phase::all().len(), 6);
    }

    #[test]
    fn phase_ordering_is_ascending() {
        let all = Phase::all();
        for i in 0..all.len() - 1 {
            assert!(
                all[i] < all[i + 1],
                "expected {:?} < {:?}",
                all[i],
                all[i + 1]
            );
        }
    }

    #[test]
    fn phase_preclinical_is_least() {
        assert!(Phase::Preclinical < Phase::Phase1);
        assert!(Phase::Preclinical < Phase::Approved);
    }

    #[test]
    fn phase_approved_is_greatest() {
        assert!(Phase::Approved > Phase::Filed);
        assert!(Phase::Approved > Phase::Preclinical);
    }

    #[test]
    fn phase_is_clinical() {
        assert!(!Phase::Preclinical.is_clinical());
        assert!(Phase::Phase1.is_clinical());
        assert!(Phase::Phase2.is_clinical());
        assert!(Phase::Phase3.is_clinical());
        assert!(!Phase::Filed.is_clinical());
        assert!(!Phase::Approved.is_clinical());
    }

    #[test]
    fn phase_is_approved() {
        for phase in Phase::all() {
            assert_eq!(phase.is_approved(), phase == Phase::Approved);
        }
    }

    #[test]
    fn phase_is_late_stage() {
        assert!(!Phase::Phase2.is_late_stage());
        assert!(Phase::Phase3.is_late_stage());
        assert!(Phase::Filed.is_late_stage());
        assert!(!Phase::Approved.is_late_stage());
    }

    #[test]
    fn phase_display() {
        assert_eq!(Phase::Preclinical.to_string(), "Preclinical");
        assert_eq!(Phase::Phase1.to_string(), "Phase 1");
        assert_eq!(Phase::Phase2.to_string(), "Phase 2");
        assert_eq!(Phase::Phase3.to_string(), "Phase 3");
        assert_eq!(Phase::Filed.to_string(), "Filed");
        assert_eq!(Phase::Approved.to_string(), "Approved");
    }

    #[test]
    fn pipeline_candidate_constructs() {
        let c = PipelineCandidate {
            name: "AMG-757".to_string(),
            mechanism: "BiTE antibody".to_string(),
            phase: Phase::Phase2,
            indication: "Small cell lung cancer".to_string(),
            therapeutic_area: TherapeuticArea::Oncology,
        };
        assert!(c.phase.is_clinical());
        assert!(!c.phase.is_approved());
    }

    #[test]
    fn pipeline_candidate_serializes_round_trip() {
        let c = PipelineCandidate {
            name: "NVX-CoV2373".to_string(),
            mechanism: "Recombinant spike protein nanoparticle".to_string(),
            phase: Phase::Approved,
            indication: "COVID-19 prevention".to_string(),
            therapeutic_area: TherapeuticArea::Vaccines,
        };
        let json = serde_json::to_string(&c).expect("serialization cannot fail");
        let parsed: PipelineCandidate =
            serde_json::from_str(&json).expect("deserialization cannot fail");
        assert_eq!(parsed.name, "NVX-CoV2373");
        assert_eq!(parsed.phase, Phase::Approved);
    }

    #[test]
    fn phase_serializes_round_trip() {
        for phase in Phase::all() {
            let json =
                serde_json::to_string(&phase).expect("serialization cannot fail on valid enum");
            let parsed: Phase =
                serde_json::from_str(&json).expect("deserialization cannot fail on valid JSON");
            assert_eq!(phase, parsed, "round-trip failed for {phase}");
        }
    }
}
