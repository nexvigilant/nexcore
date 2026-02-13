//! # Causality Assessment
//!
//! Comprehensive causality assessment algorithms for pharmacovigilance.
//!
//! # Safety Axioms
//!
//! This module implements **Safety Axiom S3: Causality Assessment** through
//! three validated algorithms. Conservation Law CL5 mandates evaluation of
//! the complete evidence spectrum: temporal, dechallenge, rechallenge,
//! and alternative causes.
//!
//! ## Algorithms
//!
//! | Algorithm | Use Case | Scoring |
//! |-----------|----------|---------|
//! | Naranjo | General ADR assessment | 10 questions, -4 to +13 |
//! | WHO-UMC | Global PV standard | 6 categories |
//! | RUCAM | Hepatotoxicity (DILI) | 7 criteria, -4 to +14 |
//!
//! ## Example
//!
//! ```rust
//! use nexcore_vigilance::pv::causality::{calculate_naranjo_quick, NaranjoCategory};
//!
//! let result = calculate_naranjo_quick(1, 1, 2, 1, 1);
//! assert!(matches!(result.category, NaranjoCategory::Probable | NaranjoCategory::Definite));
//! ```

pub mod rucam;
pub mod ucas;
pub mod who_umc;

use serde::{Deserialize, Serialize};

// Re-export RUCAM types
pub use rucam::{
    AlternativeCauses, ConcomitantDrugs, PreviousHepatotoxicity, ReactionType, RechallengeResult,
    RucamBreakdown, RucamCategory, RucamInput, RucamResult, SerologyResult, YesNoNa,
    calculate_rucam,
};

// Re-export WHO-UMC full types
pub use who_umc::{
    AlternativesLikelihood, ChallengeResult, PlausibilityStrength, WhoUmcCriteria,
    WhoUmcFullCategory, WhoUmcFullResult, WhoUmcInput, WhoUmcTemporalStrength, assess_who_umc_full,
    generate_who_umc_explanation,
};

// Re-export UCAS types (ToV §36)
pub use ucas::{
    CriterionBreakdown, CriterionResponse, CriterionScore, SIGMOID_MU, SIGMOID_SIGMA, UcasCategory,
    UcasInput, UcasResult, UcasScore, calculate_ucas, calculate_ucas_quick,
};

/// Naranjo score interpretation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NaranjoCategory {
    /// Score >= 9
    Definite,
    /// Score 5-8
    Probable,
    /// Score 1-4
    Possible,
    /// Score <= 0
    Doubtful,
}

impl std::fmt::Display for NaranjoCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Definite => write!(f, "Definite"),
            Self::Probable => write!(f, "Probable"),
            Self::Possible => write!(f, "Possible"),
            Self::Doubtful => write!(f, "Doubtful"),
        }
    }
}

/// Naranjo assessment result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NaranjoResult {
    /// Total score (-4 to 13)
    pub score: i32,
    /// Causality category
    pub category: NaranjoCategory,
    /// Individual question scores
    pub question_scores: Vec<i32>,
}

/// Full Naranjo scale assessment questions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NaranjoInput {
    /// 1. Previous conclusive reports?
    pub previous_reports: i8,
    /// 2. Event after drug administration?
    pub after_drug: i8,
    /// 3. Improvement on withdrawal (dechallenge)?
    pub improved_on_dechallenge: i8,
    /// 4. Recurrence on re-exposure (rechallenge)?
    pub recurred_on_rechallenge: i8,
    /// 5. Alternative causes exist?
    pub alternative_causes: i8,
    /// 6. Reaction on placebo?
    pub reaction_on_placebo: i8,
    /// 7. Drug detected in blood/fluids?
    pub detected_in_fluids: i8,
    /// 8. Dose-response relationship?
    pub dose_response: i8,
    /// 9. Similar reaction in previous exposure?
    pub previous_similar_reaction: i8,
    /// 10. Objective evidence?
    pub objective_evidence: i8,
}

/// Calculate full Naranjo score
#[must_use]
pub fn calculate_naranjo(input: &NaranjoInput) -> NaranjoResult {
    let score = i32::from(input.previous_reports)
        + i32::from(input.after_drug)
        + i32::from(input.improved_on_dechallenge)
        + i32::from(input.recurred_on_rechallenge)
        + i32::from(input.alternative_causes)
        + i32::from(input.reaction_on_placebo)
        + i32::from(input.detected_in_fluids)
        + i32::from(input.dose_response)
        + i32::from(input.previous_similar_reaction)
        + i32::from(input.objective_evidence);

    let category = match score {
        9..=13 => NaranjoCategory::Definite,
        5..=8 => NaranjoCategory::Probable,
        1..=4 => NaranjoCategory::Possible,
        _ => NaranjoCategory::Doubtful,
    };

    NaranjoResult {
        score,
        category,
        question_scores: vec![
            i32::from(input.previous_reports),
            i32::from(input.after_drug),
            i32::from(input.improved_on_dechallenge),
            i32::from(input.recurred_on_rechallenge),
            i32::from(input.alternative_causes),
            i32::from(input.reaction_on_placebo),
            i32::from(input.detected_in_fluids),
            i32::from(input.dose_response),
            i32::from(input.previous_similar_reaction),
            i32::from(input.objective_evidence),
        ],
    }
}

/// Calculate Naranjo score with quick assessment
///
/// # Arguments
///
/// * `temporal` - Temporal relationship (1=yes, 0=unknown, -1=no)
/// * `dechallenge` - Improved after withdrawal (1=yes, 0=unknown, -1=no)
/// * `rechallenge` - Recurred on re-exposure (2=yes, -1=no, 0=unknown)
/// * `alternatives` - Alternative causes exist (-1=yes, 1=no, 0=unknown)
/// * `previous` - Previously reported (1=yes, 0=no)
#[must_use]
pub fn calculate_naranjo_quick(
    temporal: i32,
    dechallenge: i32,
    rechallenge: i32,
    alternatives: i32,
    previous: i32,
) -> NaranjoResult {
    let score = temporal + dechallenge + rechallenge + alternatives + previous;

    let category = match score {
        9..=13 => NaranjoCategory::Definite,
        5..=8 => NaranjoCategory::Probable,
        1..=4 => NaranjoCategory::Possible,
        _ => NaranjoCategory::Doubtful,
    };

    NaranjoResult {
        score,
        category,
        question_scores: vec![temporal, dechallenge, rechallenge, alternatives, previous],
    }
}

/// WHO-UMC causality category (quick version)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WhoUmcCategory {
    /// Certain
    Certain,
    /// Probable/Likely
    ProbableLikely,
    /// Possible
    Possible,
    /// Unlikely
    Unlikely,
    /// Conditional/Unclassified
    ConditionalUnclassified,
    /// Unassessable/Unclassifiable
    UnassessableUnclassifiable,
}

/// WHO-UMC assessment result (quick version)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhoUmcResult {
    /// Causality category
    pub category: WhoUmcCategory,
    /// Description
    pub description: String,
}

/// Calculate WHO-UMC causality (quick version)
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
    } else if temporal == 1 && dechallenge == 1 && plausibility == 1 {
        WhoUmcCategory::ProbableLikely
    } else if temporal == 1 && plausibility == 1 {
        WhoUmcCategory::Possible
    } else if temporal == -1 || alternatives == 1 {
        WhoUmcCategory::Unlikely
    } else {
        WhoUmcCategory::ConditionalUnclassified
    };

    let description = match category {
        WhoUmcCategory::Certain => "Event is definitive",
        WhoUmcCategory::ProbableLikely => "Event is probably related",
        WhoUmcCategory::Possible => "Event could be related",
        WhoUmcCategory::Unlikely => "Event is unlikely related",
        WhoUmcCategory::ConditionalUnclassified => "More data needed",
        WhoUmcCategory::UnassessableUnclassifiable => "Cannot assess",
    }
    .to_string();

    WhoUmcResult {
        category,
        description,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_naranjo_definite() {
        let result = calculate_naranjo_quick(1, 1, 2, 1, 1);
        assert!(result.score >= 5);
    }

    #[test]
    fn test_who_umc_certain() {
        let result = calculate_who_umc_quick(1, 1, 1, -1, 1);
        assert_eq!(result.category, WhoUmcCategory::Certain);
    }
}
