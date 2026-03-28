use serde::{Deserialize, Serialize};

/// An unmet clinical need within a disease area.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UnmetNeed {
    /// Description of the unmet need.
    pub description: String,
    /// Severity classification.
    pub severity: NeedSeverity,
    /// What the current treatment gap is.
    pub current_gap: String,
    /// Potential therapeutic approaches being explored.
    pub potential_approaches: Vec<String>,
}

/// Severity of an unmet clinical need.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum NeedSeverity {
    Low,
    Moderate,
    High,
    Critical,
}

impl std::fmt::Display for NeedSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Low => write!(f, "Low"),
            Self::Moderate => write!(f, "Moderate"),
            Self::High => write!(f, "High"),
            Self::Critical => write!(f, "Critical"),
        }
    }
}
