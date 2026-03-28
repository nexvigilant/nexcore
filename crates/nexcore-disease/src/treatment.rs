//! Treatment landscape — lines of therapy, drug classes, and evidence levels.

use serde::{Deserialize, Serialize};

/// A single line of therapy within the standard of care.
///
/// Lines are ordered: [`LineOfTherapy::First`] is the initial intervention;
/// subsequent lines apply after failure or inadequate response.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TreatmentLine {
    /// Position in the treatment algorithm.
    pub line: LineOfTherapy,
    /// Drug classes used at this line (links to `nexcore-drug` by name).
    pub drug_classes: Vec<String>,
    /// Representative generic drug names at this line.
    pub representative_drugs: Vec<String>,
    /// Strength of evidence supporting this line.
    pub evidence_level: EvidenceLevel,
}

/// Position in the treatment algorithm.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LineOfTherapy {
    /// Initial treatment — used as soon as diagnosis is established.
    First,
    /// Second-line — used after first-line failure or intolerance.
    Second,
    /// Third-line — used after second-line failure.
    Third,
    /// Adjunctive — used in combination with another line, not standalone.
    Adjunct,
    /// Supportive — symptom management, not disease-modifying.
    Supportive,
}

impl std::fmt::Display for LineOfTherapy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::First => f.write_str("First Line"),
            Self::Second => f.write_str("Second Line"),
            Self::Third => f.write_str("Third Line"),
            Self::Adjunct => f.write_str("Adjunct"),
            Self::Supportive => f.write_str("Supportive"),
        }
    }
}

/// Evidence level grading per ICH/EMA conventions.
///
/// Ordered from strongest (IA) to weakest (IV).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceLevel {
    /// Meta-analysis or systematic review of RCTs.
    IA,
    /// At least one large RCT.
    IB,
    /// At least one well-designed controlled study without randomisation.
    IIA,
    /// At least one well-designed quasi-experimental study.
    IIB,
    /// Well-designed non-experimental studies.
    III,
    /// Expert consensus or committee reports.
    IV,
}

impl std::fmt::Display for EvidenceLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IA => f.write_str("IA"),
            Self::IB => f.write_str("IB"),
            Self::IIA => f.write_str("IIA"),
            Self::IIB => f.write_str("IIB"),
            Self::III => f.write_str("III"),
            Self::IV => f.write_str("IV"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn line_ordering_is_correct() {
        assert!(LineOfTherapy::First < LineOfTherapy::Second);
        assert!(LineOfTherapy::Second < LineOfTherapy::Third);
    }

    #[test]
    fn evidence_ordering_ia_strongest() {
        assert!(EvidenceLevel::IA < EvidenceLevel::IB);
        assert!(EvidenceLevel::IB < EvidenceLevel::IIA);
        assert!(EvidenceLevel::III < EvidenceLevel::IV);
    }

    #[test]
    fn treatment_line_round_trip_serde() {
        let line = TreatmentLine {
            line: LineOfTherapy::First,
            drug_classes: vec!["Biguanides".to_string()],
            representative_drugs: vec!["metformin".to_string()],
            evidence_level: EvidenceLevel::IA,
        };
        let json = serde_json::to_string(&line).expect("serialise");
        let parsed: TreatmentLine = serde_json::from_str(&json).expect("deserialise");
        assert_eq!(line, parsed);
    }

    #[test]
    fn line_of_therapy_display() {
        assert_eq!(LineOfTherapy::First.to_string(), "First Line");
        assert_eq!(LineOfTherapy::Adjunct.to_string(), "Adjunct");
    }

    #[test]
    fn evidence_level_display() {
        assert_eq!(EvidenceLevel::IA.to_string(), "IA");
        assert_eq!(EvidenceLevel::IV.to_string(), "IV");
    }
}
