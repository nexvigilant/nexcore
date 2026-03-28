//! Label status types.
//!
//! `LabelStatus` captures the key regulatory safety features of a US
//! prescribing information label: boxed warning presence and text,
//! REMS requirement, Section 5 warnings and precautions, and the date
//! of the most recent label revision.

use serde::{Deserialize, Serialize};

/// Regulatory label safety status for a drug.
///
/// Derived from the US FDA prescribing information (USPI). Covers the
/// highest-priority safety features that influence prescribing decisions
/// and pharmacovigilance surveillance priority.
///
/// # Examples
///
/// ```
/// use nexcore_drug::LabelStatus;
///
/// let label = LabelStatus {
///     boxed_warning: true,
///     boxed_warning_text: Some("THYROID C-CELL TUMORS".to_string()),
///     rems: false,
///     warnings_precautions: vec!["Pancreatitis".to_string()],
///     last_revision: Some("2024-03".to_string()),
/// };
/// assert!(label.boxed_warning);
/// assert!(label.has_elevated_risk());
/// ```
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LabelStatus {
    /// Product carries an FDA boxed (black box) warning
    pub boxed_warning: bool,
    /// Full text of the boxed warning heading, if present
    pub boxed_warning_text: Option<String>,
    /// Product requires a Risk Evaluation and Mitigation Strategy (REMS)
    pub rems: bool,
    /// Section 5 warnings and precautions (plain-English summaries)
    pub warnings_precautions: Vec<String>,
    /// Date of most recent label revision (YYYY-MM format where possible)
    pub last_revision: Option<String>,
}

impl LabelStatus {
    /// Returns `true` if the label carries any elevated risk marker:
    /// boxed warning or REMS.
    pub fn has_elevated_risk(&self) -> bool {
        self.boxed_warning || self.rems
    }

    /// Returns the number of Section 5 warnings listed.
    pub fn warning_count(&self) -> usize {
        self.warnings_precautions.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_label_is_clean() {
        let label = LabelStatus::default();
        assert!(!label.boxed_warning);
        assert!(label.boxed_warning_text.is_none());
        assert!(!label.rems);
        assert!(label.warnings_precautions.is_empty());
        assert!(label.last_revision.is_none());
        assert!(!label.has_elevated_risk());
        assert_eq!(label.warning_count(), 0);
    }

    #[test]
    fn elevated_risk_on_boxed_warning() {
        let label = LabelStatus {
            boxed_warning: true,
            ..Default::default()
        };
        assert!(label.has_elevated_risk());
    }

    #[test]
    fn elevated_risk_on_rems() {
        let label = LabelStatus {
            rems: true,
            ..Default::default()
        };
        assert!(label.has_elevated_risk());
    }

    #[test]
    fn warning_count_tracks_vec_length() {
        let label = LabelStatus {
            warnings_precautions: vec![
                "Pancreatitis".to_string(),
                "Gastroparesis".to_string(),
                "Thyroid C-cell tumors".to_string(),
            ],
            ..Default::default()
        };
        assert_eq!(label.warning_count(), 3);
    }

    #[test]
    fn label_serializes_round_trip() {
        let label = LabelStatus {
            boxed_warning: true,
            boxed_warning_text: Some("THYROID C-CELL TUMORS AND THYROID CANCER".to_string()),
            rems: false,
            warnings_precautions: vec!["Acute pancreatitis".to_string()],
            last_revision: Some("2024-06".to_string()),
        };
        let json =
            serde_json::to_string(&label).expect("serialization cannot fail on valid struct");
        let parsed: LabelStatus =
            serde_json::from_str(&json).expect("deserialization cannot fail on valid JSON");
        assert_eq!(parsed.boxed_warning, label.boxed_warning);
        assert_eq!(parsed.boxed_warning_text, label.boxed_warning_text);
        assert_eq!(parsed.last_revision, label.last_revision);
        assert_eq!(parsed.warnings_precautions.len(), 1);
    }
}
