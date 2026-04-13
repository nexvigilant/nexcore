//! Naranjo Algorithm — general ADR causality assessment.
//!
//! 10 questions, score range −4 to +13.
//!
//! | Category  | Score  |
//! |-----------|--------|
//! | Definite  | ≥ 9    |
//! | Probable  | 5 – 8  |
//! | Possible  | 1 – 4  |
//! | Doubtful  | ≤ 0    |

use serde::{Deserialize, Serialize};

/// Naranjo causality category.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NaranjoScore {
    /// Score ≥ 9
    Definite,
    /// Score 5 – 8
    Probable,
    /// Score 1 – 4
    Possible,
    /// Score ≤ 0
    Doubtful,
}

impl std::fmt::Display for NaranjoScore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Definite => write!(f, "Definite"),
            Self::Probable => write!(f, "Probable"),
            Self::Possible => write!(f, "Possible"),
            Self::Doubtful => write!(f, "Doubtful"),
        }
    }
}

/// Full Naranjo assessment result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NaranjoResult {
    /// Total score (−4 to +13).
    pub score: i32,
    /// Causality category.
    pub category: NaranjoScore,
    /// Per-question scores.
    pub question_scores: Vec<i32>,
}

/// Full 10-question Naranjo input.
///
/// Each field encodes the score contribution for that question.
/// Typical values: +1 (yes), 0 (unknown), −1 (no); rechallenge uses +2/−1/0.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NaranjoInput {
    /// Q1. Previous conclusive reports on this reaction?
    pub previous_reports: i8,
    /// Q2. Adverse event appeared after drug administration?
    pub after_drug: i8,
    /// Q3. Reaction improved after dechallenge?
    pub improved_on_dechallenge: i8,
    /// Q4. Reaction recurred on rechallenge?
    pub recurred_on_rechallenge: i8,
    /// Q5. Alternative causes that could explain the reaction?
    pub alternative_causes: i8,
    /// Q6. Reaction appeared with placebo?
    pub reaction_on_placebo: i8,
    /// Q7. Drug detected in blood/fluids at toxic concentration?
    pub detected_in_fluids: i8,
    /// Q8. Dose-response relationship?
    pub dose_response: i8,
    /// Q9. Patient had similar reaction in previous exposure?
    pub previous_similar_reaction: i8,
    /// Q10. Confirmed by any objective evidence?
    pub objective_evidence: i8,
}

/// Calculate Naranjo score from full 10-question input.
#[must_use]
pub fn calculate_naranjo(input: &NaranjoInput) -> NaranjoResult {
    let scores = [
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
    ];
    let score: i32 = scores.iter().sum();
    NaranjoResult {
        score,
        category: naranjo_category(score),
        question_scores: scores.to_vec(),
    }
}

/// Quick Naranjo assessment using the five most discriminating criteria.
///
/// Arguments map to:
/// - `temporal`     — temporal relationship (+1/0/−1)
/// - `dechallenge`  — improved on withdrawal (+1/0/−1)
/// - `rechallenge`  — recurred on re-exposure (+2/−1/0)
/// - `alternatives` — alternative causes (−1/+1/0)
/// - `previous`     — previously reported (+1/0)
#[must_use]
pub fn calculate_naranjo_quick(
    temporal: i32,
    dechallenge: i32,
    rechallenge: i32,
    alternatives: i32,
    previous: i32,
) -> NaranjoResult {
    let score = temporal + dechallenge + rechallenge + alternatives + previous;
    NaranjoResult {
        score,
        category: naranjo_category(score),
        question_scores: vec![temporal, dechallenge, rechallenge, alternatives, previous],
    }
}

fn naranjo_category(score: i32) -> NaranjoScore {
    match score {
        9..=i32::MAX => NaranjoScore::Definite,
        5..=8 => NaranjoScore::Probable,
        1..=4 => NaranjoScore::Possible,
        _ => NaranjoScore::Doubtful,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn naranjo_definite_high_score() {
        // temporal=1, dechallenge=1, rechallenge=2, alternatives=1, previous=1 → 6 = Probable
        let result = calculate_naranjo_quick(1, 1, 2, 1, 1);
        assert!(result.score >= 5);
        assert!(matches!(
            result.category,
            NaranjoScore::Probable | NaranjoScore::Definite
        ));
    }

    #[test]
    fn naranjo_doubtful_negative_score() {
        let result = calculate_naranjo_quick(-1, -1, -1, -1, 0);
        assert!(matches!(result.category, NaranjoScore::Doubtful));
    }

    #[test]
    fn naranjo_full_input() {
        let input = NaranjoInput {
            previous_reports: 1,
            after_drug: 1,
            improved_on_dechallenge: 1,
            recurred_on_rechallenge: 2,
            alternative_causes: 1,
            reaction_on_placebo: 0,
            detected_in_fluids: 0,
            dose_response: 1,
            previous_similar_reaction: 0,
            objective_evidence: 1,
        };
        let result = calculate_naranjo(&input);
        assert!(result.score >= 5);
        assert_eq!(result.question_scores.len(), 10);
    }
}
