//! Therapeutic area classification.
//!
//! Covers the major pharmaceutical therapeutic areas used across
//! company portfolios, pipelines, and competitive analysis.

use std::fmt;

use serde::{Deserialize, Serialize};

/// Major pharmaceutical therapeutic area.
///
/// Used to classify products, pipeline candidates, and competitive focus.
///
/// # Examples
///
/// ```
/// use nexcore_pharma::TherapeuticArea;
///
/// let area = TherapeuticArea::Oncology;
/// assert_eq!(area.to_string(), "Oncology");
/// assert!(area.is_specialty());
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TherapeuticArea {
    /// Cancer and oncology products
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
    /// Prophylactic and therapeutic vaccines
    Vaccines,
    /// Eye disease and vision disorders
    Ophthalmology,
    /// Skin conditions and dermatological disorders
    Dermatology,
    /// Blood disorders and haematologic malignancies
    Hematology,
    /// Asthma, COPD, and pulmonary disorders
    Respiratory,
    /// Bacterial, viral, and fungal infections
    Infectious,
    /// Areas not covered by the named variants
    Other,
}

impl TherapeuticArea {
    /// All therapeutic area variants in a deterministic order.
    ///
    /// Useful for coverage checks and iteration.
    pub fn all() -> [Self; 13] {
        [
            Self::Oncology,
            Self::Immunology,
            Self::Neuroscience,
            Self::Cardiovascular,
            Self::Metabolic,
            Self::RareDisease,
            Self::Vaccines,
            Self::Ophthalmology,
            Self::Dermatology,
            Self::Hematology,
            Self::Respiratory,
            Self::Infectious,
            Self::Other,
        ]
    }

    /// Returns `true` for specialty/high-cost therapeutic areas.
    ///
    /// Oncology, Rare Disease, and Hematology are considered specialty areas
    /// for competitive analysis purposes.
    pub fn is_specialty(&self) -> bool {
        matches!(self, Self::Oncology | Self::RareDisease | Self::Hematology)
    }

    /// Returns `true` for areas with established biosimilar markets.
    pub fn has_biosimilar_market(&self) -> bool {
        matches!(
            self,
            Self::Oncology | Self::Immunology | Self::Hematology | Self::Metabolic
        )
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
            Self::Vaccines => "Vaccines",
            Self::Ophthalmology => "Ophthalmology",
            Self::Dermatology => "Dermatology",
            Self::Hematology => "Hematology",
            Self::Respiratory => "Respiratory",
            Self::Infectious => "Infectious Disease",
            Self::Other => "Other",
        };
        f.write_str(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_returns_13_variants() {
        assert_eq!(TherapeuticArea::all().len(), 13);
    }

    #[test]
    fn all_variants_are_unique() {
        let all = TherapeuticArea::all();
        for i in 0..all.len() {
            for j in (i + 1)..all.len() {
                assert_ne!(all[i], all[j], "duplicate variant at indices {i} and {j}");
            }
        }
    }

    #[test]
    fn display_oncology() {
        assert_eq!(TherapeuticArea::Oncology.to_string(), "Oncology");
    }

    #[test]
    fn display_rare_disease_has_space() {
        assert_eq!(TherapeuticArea::RareDisease.to_string(), "Rare Disease");
    }

    #[test]
    fn display_infectious_is_full_name() {
        assert_eq!(
            TherapeuticArea::Infectious.to_string(),
            "Infectious Disease"
        );
    }

    #[test]
    fn specialty_areas() {
        assert!(TherapeuticArea::Oncology.is_specialty());
        assert!(TherapeuticArea::RareDisease.is_specialty());
        assert!(TherapeuticArea::Hematology.is_specialty());
        assert!(!TherapeuticArea::Cardiovascular.is_specialty());
        assert!(!TherapeuticArea::Vaccines.is_specialty());
    }

    #[test]
    fn biosimilar_market_areas() {
        assert!(TherapeuticArea::Oncology.has_biosimilar_market());
        assert!(TherapeuticArea::Immunology.has_biosimilar_market());
        assert!(!TherapeuticArea::Neuroscience.has_biosimilar_market());
        assert!(!TherapeuticArea::Ophthalmology.has_biosimilar_market());
    }

    #[test]
    fn serializes_round_trip() {
        for area in TherapeuticArea::all() {
            let json =
                serde_json::to_string(&area).expect("serialization cannot fail on valid enum");
            let parsed: TherapeuticArea =
                serde_json::from_str(&json).expect("deserialization cannot fail on valid JSON");
            assert_eq!(area, parsed, "round-trip failed for {area}");
        }
    }
}
