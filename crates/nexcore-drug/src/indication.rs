//! Indication types.
//!
//! Models a regulatory approval: the disease target, line of therapy,
//! approval year, and the regulatory basis (e.g. surrogate endpoint,
//! accelerated approval, full approval).

use serde::{Deserialize, Serialize};

/// A single regulatory indication for a drug.
///
/// # Examples
///
/// ```
/// use nexcore_drug::{Indication, LineOfTherapy};
///
/// let indication = Indication {
///     disease: "Type 2 Diabetes Mellitus".to_string(),
///     line_of_therapy: Some(LineOfTherapy::First),
///     approval_year: Some(2022),
///     regulatory_basis: Some("HbA1c reduction vs placebo".to_string()),
/// };
/// assert_eq!(indication.disease, "Type 2 Diabetes Mellitus");
/// assert!(indication.approval_year.is_some());
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Indication {
    /// Target disease or condition (ICD-10 name or plain English)
    pub disease: String,
    /// Position in treatment sequence, if defined
    pub line_of_therapy: Option<LineOfTherapy>,
    /// Year of first regulatory approval for this indication
    pub approval_year: Option<u16>,
    /// Regulatory basis or endpoint used to support approval
    pub regulatory_basis: Option<String>,
}

impl Indication {
    /// Returns `true` if this indication was approved as first-line therapy.
    pub fn is_first_line(&self) -> bool {
        matches!(self.line_of_therapy, Some(LineOfTherapy::First))
    }
}

/// Position of a drug in the treatment sequence for an indication.
///
/// # Examples
///
/// ```
/// use nexcore_drug::LineOfTherapy;
///
/// assert_eq!(LineOfTherapy::First.to_string(), "First-line");
/// assert_eq!(LineOfTherapy::Adjunct.to_string(), "Adjunct");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LineOfTherapy {
    /// First-line (initial) therapy
    First,
    /// Second-line (after first-line failure)
    Second,
    /// Third-line (after second-line failure)
    Third,
    /// Adjunct to another therapy
    Adjunct,
    /// Monotherapy (used alone, line unspecified)
    Monotherapy,
}

impl std::fmt::Display for LineOfTherapy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::First => "First-line",
            Self::Second => "Second-line",
            Self::Third => "Third-line",
            Self::Adjunct => "Adjunct",
            Self::Monotherapy => "Monotherapy",
        };
        f.write_str(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn indication_constructs() {
        let ind = Indication {
            disease: "Obesity".to_string(),
            line_of_therapy: Some(LineOfTherapy::First),
            approval_year: Some(2021),
            regulatory_basis: Some("SURMOUNT-1".to_string()),
        };
        assert_eq!(ind.disease, "Obesity");
        assert!(ind.is_first_line());
        assert_eq!(ind.approval_year, Some(2021));
    }

    #[test]
    fn is_first_line_false_when_second() {
        let ind = Indication {
            disease: "T2DM".to_string(),
            line_of_therapy: Some(LineOfTherapy::Second),
            approval_year: None,
            regulatory_basis: None,
        };
        assert!(!ind.is_first_line());
    }

    #[test]
    fn is_first_line_false_when_none() {
        let ind = Indication {
            disease: "T2DM".to_string(),
            line_of_therapy: None,
            approval_year: None,
            regulatory_basis: None,
        };
        assert!(!ind.is_first_line());
    }

    #[test]
    fn line_of_therapy_display() {
        assert_eq!(LineOfTherapy::First.to_string(), "First-line");
        assert_eq!(LineOfTherapy::Second.to_string(), "Second-line");
        assert_eq!(LineOfTherapy::Third.to_string(), "Third-line");
        assert_eq!(LineOfTherapy::Adjunct.to_string(), "Adjunct");
        assert_eq!(LineOfTherapy::Monotherapy.to_string(), "Monotherapy");
    }

    #[test]
    fn line_of_therapy_serializes_round_trip() {
        for lot in [
            LineOfTherapy::First,
            LineOfTherapy::Second,
            LineOfTherapy::Third,
            LineOfTherapy::Adjunct,
            LineOfTherapy::Monotherapy,
        ] {
            let json = serde_json::to_string(&lot)
                .expect("serialization cannot fail on valid enum variant");
            let parsed: LineOfTherapy =
                serde_json::from_str(&json).expect("deserialization cannot fail on valid JSON");
            assert_eq!(lot, parsed);
        }
    }

    #[test]
    fn indication_serializes_round_trip() {
        let ind = Indication {
            disease: "Heart failure with reduced ejection fraction".to_string(),
            line_of_therapy: Some(LineOfTherapy::Adjunct),
            approval_year: Some(2020),
            regulatory_basis: Some("DAPA-HF trial".to_string()),
        };
        let json = serde_json::to_string(&ind).expect("serialization cannot fail on valid struct");
        let parsed: Indication =
            serde_json::from_str(&json).expect("deserialization cannot fail on valid JSON");
        assert_eq!(parsed.disease, ind.disease);
        assert_eq!(parsed.approval_year, ind.approval_year);
    }
}
