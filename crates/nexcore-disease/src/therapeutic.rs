//! Therapeutic area classification for disease domain models.
//!
//! Defined independently of `nexcore-pharma` — all three entity families
//! (pharma, drug, disease) are peers with no cross-dependency.

use std::fmt;

use serde::{Deserialize, Serialize};

/// Major therapeutic area for disease classification.
///
/// # Examples
///
/// ```
/// use nexcore_disease::TherapeuticArea;
///
/// let area = TherapeuticArea::Metabolic;
/// assert_eq!(area.to_string(), "Metabolic");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TherapeuticArea {
    /// Cancer and oncology conditions
    Oncology,
    /// Autoimmune and inflammatory diseases
    Immunology,
    /// Central nervous system and psychiatric disorders
    Neuroscience,
    /// Heart disease, hypertension, and vascular disorders
    Cardiovascular,
    /// Diabetes, obesity, and metabolic disorders
    Metabolic,
    /// Orphan drugs and low-prevalence conditions
    RareDisease,
    /// Asthma, COPD, and pulmonary disorders
    Respiratory,
    /// Bacterial, viral, and fungal infections
    Infectious,
    /// Blood disorders and haematologic malignancies
    Hematology,
    /// Eye disease and vision disorders
    Ophthalmology,
    /// Skin conditions and dermatological disorders
    Dermatology,
    /// Areas not covered by the named variants
    Other,
}

impl TherapeuticArea {
    /// All variants in deterministic order.
    pub fn all() -> [Self; 12] {
        [
            Self::Oncology,
            Self::Immunology,
            Self::Neuroscience,
            Self::Cardiovascular,
            Self::Metabolic,
            Self::RareDisease,
            Self::Respiratory,
            Self::Infectious,
            Self::Hematology,
            Self::Ophthalmology,
            Self::Dermatology,
            Self::Other,
        ]
    }
}

impl fmt::Display for TherapeuticArea {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Oncology => "Oncology",
            Self::Immunology => "Immunology",
            Self::Neuroscience => "Neuroscience",
            Self::Cardiovascular => "Cardiovascular",
            Self::Metabolic => "Metabolic",
            Self::RareDisease => "Rare Disease",
            Self::Respiratory => "Respiratory",
            Self::Infectious => "Infectious Disease",
            Self::Hematology => "Hematology",
            Self::Ophthalmology => "Ophthalmology",
            Self::Dermatology => "Dermatology",
            Self::Other => "Other",
        };
        f.write_str(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_returns_12_variants() {
        assert_eq!(TherapeuticArea::all().len(), 12);
    }

    #[test]
    fn display_rare_disease() {
        assert_eq!(TherapeuticArea::RareDisease.to_string(), "Rare Disease");
    }

    #[test]
    fn round_trip_serde() {
        for area in TherapeuticArea::all() {
            let json = serde_json::to_string(&area).expect("serialise");
            let parsed: TherapeuticArea = serde_json::from_str(&json).expect("deserialise");
            assert_eq!(area, parsed);
        }
    }
}
