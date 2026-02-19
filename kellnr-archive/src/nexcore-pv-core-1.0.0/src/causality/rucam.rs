//! RUCAM (Roussel Uclaf Causality Assessment Method) Algorithm
//!
//! Specific method for assessing causality in drug-induced liver injury (DILI).
//!
//! # Scoring Categories
//!
//! | Score | Category |
//! |-------|----------|
//! | ≥8 | Highly Probable |
//! | 6-7 | Probable |
//! | 3-5 | Possible |
//! | 1-2 | Unlikely |
//! | ≤0 | Excluded |
//!
//! # Assessment Areas
//!
//! 1. Temporal relationship (onset)
//! 2. Course of reaction (dechallenge)
//! 3. Risk factors (age, alcohol, pregnancy)
//! 4. Concomitant drugs
//! 5. Search for non-drug causes
//! 6. Previous hepatotoxicity information
//! 7. Response to re-administration (rechallenge)
//!
//! # Reference
//!
//! Danan G, Benichou C. Causality assessment of adverse reactions to drugs--I.
//! A novel method based on the conclusions of international consensus meetings.
//! J Clin Epidemiol. 1993;46(11):1323-1330.

use serde::{Deserialize, Serialize};

/// Type of liver reaction pattern
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReactionType {
    /// ALT-predominant
    Hepatocellular,
    /// ALP-predominant
    Cholestatic,
    /// Both elevated
    Mixed,
}

/// Serology result for viral hepatitis and other markers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum SerologyResult {
    /// Test result was positive
    Positive,
    /// Test result was negative
    Negative,
    /// Test was not performed
    #[default]
    NotDone,
}

/// Yes/No/NotApplicable for alternative causes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum YesNoNa {
    /// Condition is present
    Yes,
    /// Condition is absent
    No,
    /// Question not applicable to this case
    #[default]
    NotApplicable,
}

/// Rechallenge result after re-exposure to the suspected drug
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RechallengeResult {
    /// Reaction recurred on re-exposure
    Positive,
    /// No reaction on re-exposure
    Negative,
    /// Rechallenge outcome was inconclusive
    NotConclusive,
}

/// Concomitant drug information
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConcomitantDrugs {
    /// Count of known hepatotoxic drugs
    pub hepatotoxic_count: u32,
    /// Known drug-drug interactions
    pub interactions: bool,
}

/// Alternative causes investigation (non-drug causes of liver injury)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AlternativeCauses {
    /// Hepatitis A virus serology
    pub hepatitis_a: SerologyResult,
    /// Hepatitis B virus serology
    pub hepatitis_b: SerologyResult,
    /// Hepatitis C virus serology
    pub hepatitis_c: SerologyResult,
    /// CMV or EBV serology
    pub cmv_ebv: SerologyResult,
    /// Biliary ultrasound/sonography findings
    pub biliary_sonography: SerologyResult,
    /// History of alcohol abuse
    pub alcoholism: YesNoNa,
    /// Pre-existing liver complications
    pub underlying_complications: YesNoNa,
}

/// Previous hepatotoxicity information for the suspected drug
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PreviousHepatotoxicity {
    /// Drug is labeled as hepatotoxic in product information
    pub labeled_hepatotoxic: bool,
    /// Published case reports of hepatotoxicity exist
    pub published_reports: bool,
    /// This specific type of reaction has been documented before
    pub reaction_known: bool,
}

/// RUCAM input for causality assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RucamInput {
    /// Days from drug start to reaction onset
    pub time_to_onset: u32,
    /// Type of liver reaction
    pub reaction_type: ReactionType,
    /// Was drug withdrawn?
    pub drug_withdrawn: bool,
    /// Days from withdrawal to improvement
    pub time_to_improvement: Option<u32>,
    /// Percentage decrease in liver values
    pub percentage_decrease: Option<f64>,
    /// Patient age in years
    pub age: u32,
    /// Alcohol use
    pub alcohol: bool,
    /// Pregnancy
    pub pregnancy: bool,
    /// Concomitant drug information
    pub concomitant_drugs: ConcomitantDrugs,
    /// Alternative causes investigation
    pub alternative_causes: AlternativeCauses,
    /// Previous hepatotoxicity info
    pub previous_hepatotoxicity: PreviousHepatotoxicity,
    /// Was rechallenge performed?
    pub rechallenge_performed: bool,
    /// Result of rechallenge
    pub rechallenge_result: Option<RechallengeResult>,
}

/// RUCAM causality category based on total score
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RucamCategory {
    /// Score ≥8: Drug causation highly probable
    HighlyProbable,
    /// Score 6-7: Drug causation probable
    Probable,
    /// Score 3-5: Drug causation possible
    Possible,
    /// Score 1-2: Drug causation unlikely
    Unlikely,
    /// Score ≤0: Drug causation excluded
    Excluded,
}

impl std::fmt::Display for RucamCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::HighlyProbable => write!(f, "Highly Probable"),
            Self::Probable => write!(f, "Probable"),
            Self::Possible => write!(f, "Possible"),
            Self::Unlikely => write!(f, "Unlikely"),
            Self::Excluded => write!(f, "Excluded"),
        }
    }
}

/// Score breakdown by RUCAM assessment area
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RucamBreakdown {
    /// Score for temporal relationship (time to onset)
    pub temporal_relationship: i32,
    /// Score for course of reaction (dechallenge)
    pub course_of_reaction: i32,
    /// Score for risk factors (age, alcohol, pregnancy)
    pub risk_factors: i32,
    /// Score for concomitant hepatotoxic drugs
    pub concomitant_drugs: i32,
    /// Score for exclusion of alternative causes
    pub alternative_causes: i32,
    /// Score for previous hepatotoxicity information
    pub previous_information: i32,
    /// Score for rechallenge (if performed)
    pub rechallenge: i32,
}

/// RUCAM assessment result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RucamResult {
    /// Total score (-4 to 14)
    pub total_score: i32,
    /// Causality category
    pub category: RucamCategory,
    /// Confidence (0.0 to 1.0)
    pub confidence: f64,
    /// Score breakdown
    pub breakdown: RucamBreakdown,
}

/// Assess temporal relationship
fn assess_temporal(input: &RucamInput) -> i32 {
    let days = input.time_to_onset;
    // Score 2 for onset within 90 days, 1 for longer
    if days <= 90 { 2 } else { 1 }
}

/// Assess course of reaction (dechallenge)
fn assess_course(input: &RucamInput) -> i32 {
    if !input.drug_withdrawn {
        return 0;
    }

    let time = match input.time_to_improvement {
        Some(t) => t,
        None => return 0,
    };

    let decrease = match input.percentage_decrease {
        Some(d) => d,
        None => return 0,
    };

    match input.reaction_type {
        ReactionType::Hepatocellular => {
            if decrease >= 50.0 {
                if time <= 30 {
                    3
                } else if time <= 180 {
                    2
                } else {
                    1
                }
            } else if time <= 30 {
                2
            } else if time <= 180 {
                1
            } else {
                0
            }
        }
        ReactionType::Cholestatic | ReactionType::Mixed => {
            if decrease >= 50.0 && time <= 180 {
                2
            } else if time <= 180 {
                1
            } else {
                0
            }
        }
    }
}

/// Assess risk factors
fn assess_risk_factors(input: &RucamInput) -> i32 {
    let mut score = 0;
    if input.age >= 55 {
        score += 1;
    }
    if input.alcohol {
        score += 1;
    }
    if input.pregnancy {
        score += 1;
    }
    score.min(2) // Maximum 2 points
}

/// Assess concomitant drugs
fn assess_concomitant(input: &RucamInput) -> i32 {
    let count = input.concomitant_drugs.hepatotoxic_count;
    let interactions = input.concomitant_drugs.interactions;

    if count == 0 && !interactions {
        2
    } else if count == 1 || interactions {
        1
    } else {
        0
    }
}

/// Assess alternative causes
fn assess_alternatives(input: &RucamInput) -> i32 {
    let causes = &input.alternative_causes;
    let mut positive = 0;
    let mut total = 0;

    // Count serology results
    for result in [
        causes.hepatitis_a,
        causes.hepatitis_b,
        causes.hepatitis_c,
        causes.cmv_ebv,
        causes.biliary_sonography,
    ] {
        if result != SerologyResult::NotDone {
            total += 1;
            if result == SerologyResult::Positive {
                positive += 1;
            }
        }
    }

    // Count yes/no results
    for result in [causes.alcoholism, causes.underlying_complications] {
        if result != YesNoNa::NotApplicable {
            total += 1;
            if result == YesNoNa::Yes {
                positive += 1;
            }
        }
    }

    if total >= 6 {
        if positive == 0 {
            2
        } else if positive <= 2 {
            1
        } else {
            -2
        }
    } else if total >= 4 {
        if positive == 0 { 1 } else { -1 }
    } else {
        0
    }
}

/// Assess previous hepatotoxicity information
fn assess_previous(input: &RucamInput) -> i32 {
    let prev = &input.previous_hepatotoxicity;

    if prev.labeled_hepatotoxic && prev.reaction_known {
        2
    } else if prev.published_reports || prev.reaction_known {
        1
    } else {
        0
    }
}

/// Assess rechallenge
fn assess_rechallenge(input: &RucamInput) -> i32 {
    if !input.rechallenge_performed {
        return 0;
    }

    match input.rechallenge_result {
        Some(RechallengeResult::Positive) => 3,
        Some(RechallengeResult::Negative) => -2,
        Some(RechallengeResult::NotConclusive) => 1,
        None => 0,
    }
}

/// Categorize total score
fn categorize(score: i32) -> RucamCategory {
    match score {
        8.. => RucamCategory::HighlyProbable,
        6..=7 => RucamCategory::Probable,
        3..=5 => RucamCategory::Possible,
        1..=2 => RucamCategory::Unlikely,
        _ => RucamCategory::Excluded,
    }
}

/// Calculate confidence based on data completeness
fn calculate_confidence(input: &RucamInput) -> f64 {
    let mut conf = 0.5;

    if input.drug_withdrawn && input.time_to_improvement.is_some() {
        conf += 0.15;
    }
    if input.rechallenge_performed {
        conf += 0.15;
    }

    // Count completed alternative investigations
    let causes = &input.alternative_causes;
    let mut completed = 0;
    for r in [
        causes.hepatitis_a,
        causes.hepatitis_b,
        causes.hepatitis_c,
        causes.cmv_ebv,
        causes.biliary_sonography,
    ] {
        if r != SerologyResult::NotDone {
            completed += 1;
        }
    }
    for r in [causes.alcoholism, causes.underlying_complications] {
        if r != YesNoNa::NotApplicable {
            completed += 1;
        }
    }

    conf += (completed as f64 / 7.0) * 0.2;
    conf.min(1.0)
}

/// Calculate RUCAM score for hepatotoxicity causality assessment
#[must_use]
pub fn calculate_rucam(input: &RucamInput) -> RucamResult {
    let breakdown = RucamBreakdown {
        temporal_relationship: assess_temporal(input),
        course_of_reaction: assess_course(input),
        risk_factors: assess_risk_factors(input),
        concomitant_drugs: assess_concomitant(input),
        alternative_causes: assess_alternatives(input),
        previous_information: assess_previous(input),
        rechallenge: assess_rechallenge(input),
    };

    let total_score = breakdown.temporal_relationship
        + breakdown.course_of_reaction
        + breakdown.risk_factors
        + breakdown.concomitant_drugs
        + breakdown.alternative_causes
        + breakdown.previous_information
        + breakdown.rechallenge;

    RucamResult {
        total_score,
        category: categorize(total_score),
        confidence: calculate_confidence(input),
        breakdown,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_input() -> RucamInput {
        RucamInput {
            time_to_onset: 30,
            reaction_type: ReactionType::Hepatocellular,
            drug_withdrawn: true,
            time_to_improvement: Some(14),
            percentage_decrease: Some(60.0),
            age: 60,
            alcohol: false,
            pregnancy: false,
            concomitant_drugs: ConcomitantDrugs::default(),
            alternative_causes: AlternativeCauses::default(),
            previous_hepatotoxicity: PreviousHepatotoxicity {
                labeled_hepatotoxic: true,
                published_reports: true,
                reaction_known: true,
            },
            rechallenge_performed: false,
            rechallenge_result: None,
        }
    }

    #[test]
    fn test_rucam_probable() {
        let input = make_input();
        let result = calculate_rucam(&input);
        assert!(result.total_score >= 6);
        assert!(matches!(
            result.category,
            RucamCategory::Probable | RucamCategory::HighlyProbable
        ));
    }

    #[test]
    fn test_rechallenge_positive() {
        let mut input = make_input();
        input.rechallenge_performed = true;
        input.rechallenge_result = Some(RechallengeResult::Positive);
        let result = calculate_rucam(&input);
        assert_eq!(result.breakdown.rechallenge, 3);
    }

    #[test]
    fn test_category_display() {
        assert_eq!(RucamCategory::HighlyProbable.to_string(), "Highly Probable");
        assert_eq!(RucamCategory::Excluded.to_string(), "Excluded");
    }
}
