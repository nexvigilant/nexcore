//! WHO-UMC Causality Assessment Algorithm (Full Implementation)
//!
//! The WHO-UMC (World Health Organization - Uppsala Monitoring Centre) causality
//! assessment system is the most widely used method in pharmacovigilance.
//!
//! # Safety Axioms
//!
//! This module implements **Safety Axiom S3: Causality Assessment** - the systematic
//! evaluation of the causal relationship between drug exposure and adverse events.
//! Conservation Law CL5 mandates: "All causality assessments shall consider the
//! complete evidence spectrum: temporal, dechallenge, rechallenge, alternatives."
//!
//! # Categories
//!
//! | Category | Description |
//! |----------|-------------|
//! | Certain | Positive rechallenge, no alternatives |
//! | Probable/Likely | Good evidence, unlikely alternatives |
//! | Possible | Reasonable relationship, could be explained otherwise |
//! | Unlikely | Improbable relationship, other explanations plausible |
//! | Conditional | More data essential |
//! | Unassessable | Insufficient or contradictory information |
//!
//! # Example
//!
//! ```rust
//! use nexcore_vigilance::pv::causality::who_umc::{
//!     WhoUmcInput, WhoUmcTemporalStrength, assess_who_umc_full,
//! };
//!
//! let input = WhoUmcInput {
//!     has_temporal_relationship: true,
//!     temporal_strength: WhoUmcTemporalStrength::Strong,
//!     dechallenge_performed: true,
//!     dechallenge_result: Some(nexcore_vigilance::pv::causality::who_umc::ChallengeResult::Positive),
//!     rechallenge_performed: false,
//!     rechallenge_result: None,
//!     alternative_causes_present: false,
//!     alternatives_likelihood: nexcore_vigilance::pv::causality::who_umc::AlternativesLikelihood::None,
//!     biologically_plausible: true,
//!     plausibility_strength: nexcore_vigilance::pv::causality::who_umc::PlausibilityStrength::High,
//!     previously_reported: true,
//!     known_adverse_reaction: true,
//!     data_complete: true,
//!     data_sufficient: true,
//! };
//!
//! let result = assess_who_umc_full(&input);
//! assert!(matches!(
//!     result.category,
//!     nexcore_vigilance::pv::causality::who_umc::WhoUmcFullCategory::Probable
//!     | nexcore_vigilance::pv::causality::who_umc::WhoUmcFullCategory::Certain
//! ));
//! ```
//!
//! # Reference
//!
//! The WHO-UMC system for standardized case causality assessment.
//! Uppsala Monitoring Centre, WHO Collaborating Centre for International Drug Monitoring.

use serde::{Deserialize, Serialize};

/// Strength of temporal relationship
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum WhoUmcTemporalStrength {
    /// Strong temporal relationship (clear onset after exposure)
    Strong,
    /// Moderate temporal relationship
    #[default]
    Moderate,
    /// Weak temporal relationship
    Weak,
    /// No temporal relationship
    None,
}

/// Result of dechallenge or rechallenge
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChallengeResult {
    /// Event resolved/recurred as expected
    Positive,
    /// Event did not resolve/recur
    Negative,
    /// Result was inconclusive
    Inconclusive,
}

/// Likelihood of alternative causes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum AlternativesLikelihood {
    /// No alternative causes identified
    #[default]
    None,
    /// Alternative causes are possible
    Possible,
    /// Alternative causes are probable
    Probable,
    /// Alternative causes are certain
    Certain,
}

/// Strength of biological plausibility
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum PlausibilityStrength {
    /// High biological plausibility (mechanism well understood)
    High,
    /// Moderate biological plausibility
    #[default]
    Moderate,
    /// Low biological plausibility
    Low,
    /// Unknown biological plausibility
    Unknown,
}

/// Full WHO-UMC causality category
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WhoUmcFullCategory {
    /// Definitive causal relationship
    Certain,
    /// Probable/likely causal relationship
    Probable,
    /// Possible causal relationship
    Possible,
    /// Unlikely causal relationship
    Unlikely,
    /// More data needed (conditional/unclassified)
    Conditional,
    /// Cannot be assessed (insufficient/contradictory)
    Unassessable,
}

impl std::fmt::Display for WhoUmcFullCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Certain => write!(f, "Certain"),
            Self::Probable => write!(f, "Probable/Likely"),
            Self::Possible => write!(f, "Possible"),
            Self::Unlikely => write!(f, "Unlikely"),
            Self::Conditional => write!(f, "Conditional/Unclassified"),
            Self::Unassessable => write!(f, "Unassessable/Unclassifiable"),
        }
    }
}

/// Criteria flags for WHO-UMC assessment
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WhoUmcCriteria {
    /// Temporal relationship criterion met
    pub temporal_relationship: bool,
    /// Dechallenge criterion met
    pub dechallenge: bool,
    /// Rechallenge criterion met
    pub rechallenge: bool,
    /// Alternative causes ruled out
    pub alternative_causes: bool,
    /// Biological plausibility criterion met
    pub biological_plausibility: bool,
}

/// Full WHO-UMC assessment input
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WhoUmcInput {
    // Temporal relationship
    /// Whether temporal relationship exists
    pub has_temporal_relationship: bool,
    /// Strength of temporal relationship
    pub temporal_strength: WhoUmcTemporalStrength,

    // Dechallenge (drug withdrawal)
    /// Whether dechallenge was performed
    pub dechallenge_performed: bool,
    /// Result of dechallenge
    pub dechallenge_result: Option<ChallengeResult>,

    // Rechallenge (drug re-administration)
    /// Whether rechallenge was performed
    pub rechallenge_performed: bool,
    /// Result of rechallenge
    pub rechallenge_result: Option<ChallengeResult>,

    // Alternative explanations
    /// Whether alternative causes are present
    pub alternative_causes_present: bool,
    /// Likelihood of alternatives
    pub alternatives_likelihood: AlternativesLikelihood,

    // Biological plausibility
    /// Whether relationship is biologically plausible
    pub biologically_plausible: bool,
    /// Strength of plausibility
    pub plausibility_strength: PlausibilityStrength,

    // Previous knowledge
    /// Previously reported in literature
    pub previously_reported: bool,
    /// Known adverse reaction for this drug
    pub known_adverse_reaction: bool,

    // Data quality
    /// Data is complete
    pub data_complete: bool,
    /// Data is sufficient for assessment
    pub data_sufficient: bool,
}

/// Full WHO-UMC assessment result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhoUmcFullResult {
    /// Causality category
    pub category: WhoUmcFullCategory,
    /// Numeric score (internal calculation)
    pub score: i32,
    /// Human-readable rationale
    pub rationale: String,
    /// Assessment confidence (0.0-1.0)
    pub confidence: f64,
    /// Individual criteria met
    pub criteria: WhoUmcCriteria,
}

/// Assess WHO-UMC causality using full algorithm
///
/// Implements Conservation Law CL5 by evaluating all evidence dimensions:
/// temporal, dechallenge, rechallenge, alternatives, and plausibility.
#[must_use]
pub fn assess_who_umc_full(input: &WhoUmcInput) -> WhoUmcFullResult {
    // Data quality check first
    if !input.data_sufficient || !input.data_complete {
        return WhoUmcFullResult {
            category: WhoUmcFullCategory::Unassessable,
            score: 0,
            rationale: "Insufficient or incomplete data for proper causality assessment"
                .to_string(),
            confidence: 0.5,
            criteria: WhoUmcCriteria {
                temporal_relationship: input.has_temporal_relationship,
                biological_plausibility: input.biologically_plausible,
                ..Default::default()
            },
        };
    }

    let mut score = 0i32;
    let mut criteria = WhoUmcCriteria::default();

    // 1. Temporal relationship (essential criterion)
    if input.has_temporal_relationship {
        match input.temporal_strength {
            WhoUmcTemporalStrength::Strong => {
                score += 3;
                criteria.temporal_relationship = true;
            }
            WhoUmcTemporalStrength::Moderate => {
                score += 2;
                criteria.temporal_relationship = true;
            }
            WhoUmcTemporalStrength::Weak => {
                score += 1;
            }
            WhoUmcTemporalStrength::None => {}
        }
    } else {
        // No temporal relationship = unlikely
        return WhoUmcFullResult {
            category: WhoUmcFullCategory::Unlikely,
            score: 0,
            rationale: "No plausible temporal relationship between drug intake and event"
                .to_string(),
            confidence: 0.9,
            criteria,
        };
    }

    // 2. Dechallenge assessment
    if input.dechallenge_performed {
        match input.dechallenge_result {
            Some(ChallengeResult::Positive) => {
                score += 2;
                criteria.dechallenge = true;
            }
            Some(ChallengeResult::Negative) => {
                score -= 1;
            }
            Some(ChallengeResult::Inconclusive) | None => {}
        }
    }

    // 3. Rechallenge assessment (strongest evidence)
    if input.rechallenge_performed {
        match input.rechallenge_result {
            Some(ChallengeResult::Positive) => {
                score += 3;
                criteria.rechallenge = true;
            }
            Some(ChallengeResult::Negative) => {
                score -= 2;
            }
            Some(ChallengeResult::Inconclusive) | None => {}
        }
    }

    // 4. Alternative causes assessment
    if !input.alternative_causes_present
        || matches!(input.alternatives_likelihood, AlternativesLikelihood::None)
    {
        score += 2;
        criteria.alternative_causes = true;
    } else {
        match input.alternatives_likelihood {
            AlternativesLikelihood::Possible => {}
            AlternativesLikelihood::Probable => score -= 1,
            AlternativesLikelihood::Certain => score -= 2,
            AlternativesLikelihood::None => unreachable!(),
        }
    }

    // 5. Biological plausibility assessment
    if input.biologically_plausible {
        match input.plausibility_strength {
            PlausibilityStrength::High => {
                score += 2;
                criteria.biological_plausibility = true;
            }
            PlausibilityStrength::Moderate => {
                score += 1;
                criteria.biological_plausibility = true;
            }
            PlausibilityStrength::Low => {}
            PlausibilityStrength::Unknown => {}
        }
    } else {
        score -= 1;
    }

    // 6. Determine category based on score and specific criteria
    let (category, rationale, confidence) = determine_category(score, &criteria);

    WhoUmcFullResult {
        category,
        score,
        rationale,
        confidence,
        criteria,
    }
}

/// Determine WHO-UMC category based on score and criteria
fn determine_category(score: i32, criteria: &WhoUmcCriteria) -> (WhoUmcFullCategory, String, f64) {
    // Positive rechallenge = Certain
    if criteria.rechallenge {
        return (
            WhoUmcFullCategory::Certain,
            "Event recurred upon drug re-administration (positive rechallenge), \
             providing definitive evidence of causality"
                .to_string(),
            0.95,
        );
    }

    // High score with dechallenge and ruled-out alternatives = Probable
    if score >= 7 && criteria.dechallenge && criteria.alternative_causes {
        return (
            WhoUmcFullCategory::Probable,
            "Strong temporal relationship with positive dechallenge \
             and no plausible alternative explanations"
                .to_string(),
            0.85,
        );
    }

    // Good score with temporal and ruled-out alternatives = Probable
    if score >= 6 && criteria.temporal_relationship && criteria.alternative_causes {
        return (
            WhoUmcFullCategory::Probable,
            "Strong temporal relationship, unlikely to be attributed to other causes".to_string(),
            0.8,
        );
    }

    // Moderate score = Possible
    if score >= 3 {
        return (
            WhoUmcFullCategory::Possible,
            "Reasonable temporal relationship but could also be explained \
             by disease or other factors"
                .to_string(),
            0.65,
        );
    }

    // Low positive score = Unlikely
    if score >= 0 {
        return (
            WhoUmcFullCategory::Unlikely,
            "Weak temporal relationship or other drugs/diseases \
             provide more plausible explanations"
                .to_string(),
            0.75,
        );
    }

    // Negative score with sufficient data = Conditional
    (
        WhoUmcFullCategory::Conditional,
        "Conflicting evidence requires additional data for proper assessment".to_string(),
        0.5,
    )
}

/// Generate detailed human-readable explanation
#[must_use]
pub fn generate_who_umc_explanation(result: &WhoUmcFullResult, input: &WhoUmcInput) -> String {
    let mut lines = Vec::new();

    lines.push(format!(
        "**WHO-UMC Causality Assessment: {}**",
        result.category
    ));
    lines.push(format!("\nConfidence: {:.0}%", result.confidence * 100.0));
    lines.push(format!("\n**Rationale:** {}", result.rationale));
    lines.push("\n**Evaluation Criteria:**".to_string());

    // Temporal relationship
    let temporal_mark = if result.criteria.temporal_relationship {
        "✓"
    } else {
        "✗"
    };
    lines.push(format!(
        "- Temporal Relationship: {temporal_mark} ({:?})",
        input.temporal_strength
    ));

    // Dechallenge
    if input.dechallenge_performed {
        let dech_mark = if result.criteria.dechallenge {
            "✓"
        } else {
            "✗"
        };
        let dech_result = input
            .dechallenge_result
            .map(|r| format!("{r:?}"))
            .unwrap_or_else(|| "N/A".to_string());
        lines.push(format!("- Dechallenge: {dech_mark} ({dech_result})"));
    } else {
        lines.push("- Dechallenge: Not performed".to_string());
    }

    // Rechallenge
    if input.rechallenge_performed {
        let rech_mark = if result.criteria.rechallenge {
            "✓"
        } else {
            "✗"
        };
        let rech_result = input
            .rechallenge_result
            .map(|r| format!("{r:?}"))
            .unwrap_or_else(|| "N/A".to_string());
        lines.push(format!("- Rechallenge: {rech_mark} ({rech_result})"));
    } else {
        lines.push("- Rechallenge: Not performed".to_string());
    }

    // Alternative causes
    let alt_mark = if result.criteria.alternative_causes {
        "✓ Ruled out"
    } else {
        "✗ Present"
    };
    lines.push(format!(
        "- Alternative Causes: {alt_mark} (likelihood: {:?})",
        input.alternatives_likelihood
    ));

    // Biological plausibility
    let plaus_mark = if result.criteria.biological_plausibility {
        "✓"
    } else {
        "✗"
    };
    lines.push(format!(
        "- Biological Plausibility: {plaus_mark} ({:?})",
        input.plausibility_strength
    ));

    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_certain_with_rechallenge() {
        let input = WhoUmcInput {
            has_temporal_relationship: true,
            temporal_strength: WhoUmcTemporalStrength::Strong,
            rechallenge_performed: true,
            rechallenge_result: Some(ChallengeResult::Positive),
            biologically_plausible: true,
            plausibility_strength: PlausibilityStrength::High,
            data_complete: true,
            data_sufficient: true,
            ..Default::default()
        };

        let result = assess_who_umc_full(&input);
        assert_eq!(result.category, WhoUmcFullCategory::Certain);
        assert!(result.criteria.rechallenge);
    }

    #[test]
    fn test_probable_with_dechallenge() {
        let input = WhoUmcInput {
            has_temporal_relationship: true,
            temporal_strength: WhoUmcTemporalStrength::Strong,
            dechallenge_performed: true,
            dechallenge_result: Some(ChallengeResult::Positive),
            alternative_causes_present: false,
            alternatives_likelihood: AlternativesLikelihood::None,
            biologically_plausible: true,
            plausibility_strength: PlausibilityStrength::High,
            data_complete: true,
            data_sufficient: true,
            ..Default::default()
        };

        let result = assess_who_umc_full(&input);
        assert_eq!(result.category, WhoUmcFullCategory::Probable);
    }

    #[test]
    fn test_unlikely_no_temporal() {
        let input = WhoUmcInput {
            has_temporal_relationship: false,
            data_complete: true,
            data_sufficient: true,
            ..Default::default()
        };

        let result = assess_who_umc_full(&input);
        assert_eq!(result.category, WhoUmcFullCategory::Unlikely);
    }

    #[test]
    fn test_unassessable_insufficient_data() {
        let input = WhoUmcInput {
            has_temporal_relationship: true,
            data_complete: false,
            data_sufficient: false,
            ..Default::default()
        };

        let result = assess_who_umc_full(&input);
        assert_eq!(result.category, WhoUmcFullCategory::Unassessable);
    }

    #[test]
    fn test_explanation_generation() {
        let input = WhoUmcInput {
            has_temporal_relationship: true,
            temporal_strength: WhoUmcTemporalStrength::Strong,
            dechallenge_performed: true,
            dechallenge_result: Some(ChallengeResult::Positive),
            biologically_plausible: true,
            plausibility_strength: PlausibilityStrength::High,
            data_complete: true,
            data_sufficient: true,
            ..Default::default()
        };

        let result = assess_who_umc_full(&input);
        let explanation = generate_who_umc_explanation(&result, &input);

        assert!(explanation.contains("WHO-UMC"));
        assert!(explanation.contains("Temporal Relationship"));
        assert!(explanation.contains("Dechallenge"));
    }
}
