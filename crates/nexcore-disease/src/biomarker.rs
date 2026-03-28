use serde::{Deserialize, Serialize};

/// A biomarker relevant to disease diagnosis, prognosis, or treatment.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Biomarker {
    /// Biomarker name (e.g., "HbA1c", "ApoE ε4").
    pub name: String,
    /// Type of biomarker.
    pub biomarker_type: BiomarkerType,
    /// Clinical use description.
    pub clinical_use: String,
}

/// Classification of biomarker function.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BiomarkerType {
    /// Confirms disease presence.
    Diagnostic,
    /// Predicts disease course.
    Prognostic,
    /// Predicts treatment response.
    Predictive,
    /// Measures drug effect.
    Pharmacodynamic,
    /// Monitors for adverse effects.
    Safety,
}

impl std::fmt::Display for BiomarkerType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Diagnostic => write!(f, "Diagnostic"),
            Self::Prognostic => write!(f, "Prognostic"),
            Self::Predictive => write!(f, "Predictive"),
            Self::Pharmacodynamic => write!(f, "Pharmacodynamic"),
            Self::Safety => write!(f, "Safety"),
        }
    }
}
