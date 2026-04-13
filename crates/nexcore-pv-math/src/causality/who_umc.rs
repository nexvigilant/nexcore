//! WHO-UMC causality assessment — quick and full implementations.
//!
//! | Category                  | Description                          |
//! |---------------------------|--------------------------------------|
//! | Certain                   | Positive rechallenge, no alternatives |
//! | Probable/Likely           | Good evidence, alternatives unlikely |
//! | Possible                  | Reasonable, could be explained       |
//! | Unlikely                  | Improbable, other explanations likely |
//! | Conditional/Unclassified  | More data needed                     |
//! | Unassessable/Unclassifiable | Insufficient information           |
//!
//! Reference: WHO-UMC Standardized Case Causality Assessment,
//! Uppsala Monitoring Centre.

use serde::{Deserialize, Serialize};

/// WHO-UMC causality category (quick six-level scale).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WhoUmcCategory {
    /// Positive rechallenge, no alternative causes — strongest evidence.
    Certain,
    /// Good temporal relationship, dechallenge positive, alternatives unlikely.
    ProbableLikely,
    /// Temporal relationship plausible but alternative explanations exist.
    Possible,
    /// Improbable temporal relationship or strong alternative explanation.
    Unlikely,
    /// Insufficient data to classify — more information needed.
    ConditionalUnclassified,
    /// Cannot assess — contradictory or missing information.
    UnassessableUnclassifiable,
}

impl std::fmt::Display for WhoUmcCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Certain => write!(f, "Certain"),
            Self::ProbableLikely => write!(f, "Probable/Likely"),
            Self::Possible => write!(f, "Possible"),
            Self::Unlikely => write!(f, "Unlikely"),
            Self::ConditionalUnclassified => write!(f, "Conditional/Unclassified"),
            Self::UnassessableUnclassifiable => write!(f, "Unassessable/Unclassifiable"),
        }
    }
}

/// Quick WHO-UMC assessment result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhoUmcResult {
    /// Causality category.
    pub category: WhoUmcCategory,
    /// Plain-English description of the category.
    pub description: &'static str,
}

/// Quick WHO-UMC causality classification from five binary criteria.
///
/// Arguments:
/// - `temporal`      — plausible time relationship (1=yes, 0=unknown, −1=no)
/// - `dechallenge`   — improved after withdrawal (1=yes, 0=unknown, −1=no)
/// - `rechallenge`   — recurred on re-exposure (1=yes, 0=unknown, −1=no)
/// - `alternatives`  — alternative causes excluded (1=yes, 0=unknown, −1=no)
/// - `plausibility`  — biologically plausible (1=yes, 0=unknown, −1=no)
#[must_use]
pub fn calculate_who_umc_quick(
    temporal: i32,
    dechallenge: i32,
    rechallenge: i32,
    alternatives: i32,
    plausibility: i32,
) -> WhoUmcResult {
    let category = if temporal == 1 && dechallenge == 1 && rechallenge == 1 && plausibility == 1 {
        WhoUmcCategory::Certain
    } else if temporal == 1 && dechallenge == 1 && plausibility == 1 && rechallenge != 1 {
        // WHO-UMC Probable/Likely: temporal + dechallenge + plausible, rechallenge not required
        WhoUmcCategory::ProbableLikely
    } else if temporal == 1 && plausibility == 1 {
        WhoUmcCategory::Possible
    } else if temporal == -1 || alternatives == -1 {
        WhoUmcCategory::Unlikely
    } else {
        WhoUmcCategory::ConditionalUnclassified
    };

    let description = match category {
        WhoUmcCategory::Certain => "Event is definitive — positive rechallenge, no alternatives",
        WhoUmcCategory::ProbableLikely => {
            "Event is probably related — good temporal, dechallenge positive"
        }
        WhoUmcCategory::Possible => {
            "Event could be related — temporal plausible but alternatives exist"
        }
        WhoUmcCategory::Unlikely => {
            "Event is unlikely related — poor temporal or strong alternatives"
        }
        WhoUmcCategory::ConditionalUnclassified => "More data needed to classify",
        WhoUmcCategory::UnassessableUnclassifiable => "Cannot assess — insufficient information",
    };

    WhoUmcResult {
        category,
        description,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn who_umc_certain_all_positive() {
        let result = calculate_who_umc_quick(1, 1, 1, 1, 1);
        assert_eq!(result.category, WhoUmcCategory::Certain);
    }

    #[test]
    fn who_umc_probable_no_rechallenge() {
        // temporal=yes, dechallenge=yes, rechallenge=unknown, alternatives=unknown, plausibility=yes
        // alternatives must be 0 (unknown) — not 1 (confirmed) — to reach ProbableLikely
        let result = calculate_who_umc_quick(1, 1, 0, 0, 1);
        assert_eq!(result.category, WhoUmcCategory::ProbableLikely);
    }

    #[test]
    fn who_umc_unlikely_negative_temporal() {
        let result = calculate_who_umc_quick(-1, 0, 0, 0, 0);
        assert_eq!(result.category, WhoUmcCategory::Unlikely);
    }

    #[test]
    fn who_umc_possible_plausible_no_dechallenge() {
        let result = calculate_who_umc_quick(1, 0, 0, 0, 1);
        assert_eq!(result.category, WhoUmcCategory::Possible);
    }
}
